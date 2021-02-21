use std::{
    ops::{Deref, DerefMut},
    sync::Arc,
};

use chrono::Utc;
use futures::StreamExt;
use tokio::sync::{self, Mutex};
use tracing::{info, instrument, warn};

use crate::{ModelToTask, models::{ExecutionReport, OutputModel, TaskError, TaskModel}, now, server, traits::BoxedStream, types::{BoxedTask, ExecutorSender, MPSCMessageSender, OutputEmitter, OutputSender, ServerReceiver, TaskWatcherSender}};
use crate::messages::Message;
mod new;
pub use new::Reactor;
/// ReactorEX's main job is to listen for messages
/// And transfer it to certain channel
///
/// It's another duty is to send GET_SCHEDULED_TASKS
/// message every second.
///
/// By design, I try to encapsulate all db interactions here
/// Which results into easier debugging/maintainance
///
///
///
pub struct ReactorEX {
    pub db_sender: MPSCMessageSender,
    pub executor_sender: MPSCMessageSender,
    pub task_watcher_sender: MPSCMessageSender,
    pub output_emitter: OutputEmitter,
    pub server_receiver: Arc<Mutex<ServerReceiver>>,
}

impl ReactorEX {
    pub async fn listen(&mut self) {
        let mut server_receiver = self.server_receiver.clone();
        let db_sender = self.db_sender.clone();
        let exec_sender = self.executor_sender.clone();
        tokio::spawn(async move {
            let mut receiver = server_receiver.lock().await;
            let receiver = &mut *receiver;
            Self::listen_for_server(receiver, db_sender, exec_sender).await;
        });
        loop {
            let task_models = match self.send_db_get_scheduled_tasks_message().await {
                Ok(tasks) => tasks,
                Err(e) => {
                    warn!("Couldn't get the scheduled tasks, error: {}", e.to_string());
                    continue;
                }
            };
            let mut tasks = task_models.iter().map(|task| {
                let boxed_task;
                ModelToTask!(task => boxed_task);
                return boxed_task;
            });
            let mut search_iter = task_models.iter();
            for task in tasks.next() {
                if let Some(task) = task {
                    let id = task.get_id();
                    let index = search_iter.position(|t| t.id == id);
                    let mut task_model = task_models[index.unwrap()].clone();

                    let result =
                        Self::send_executor_execute_message(self.executor_sender.clone(), task)
                            .await;
                    let o_emitter = self.output_emitter.clone();
                    let tw_sender = self.task_watcher_sender.clone();
                    let db_sender = self.db_sender.clone();
                    let db_report_sender = self.db_sender.clone();
                    tokio::spawn(async move {
                        let result = result.await.unwrap();
                        let (o_tx, mut o_rx) = tokio::sync::broadcast::channel(128);
                        let (er_tx, er_rx) = tokio::sync::oneshot::channel();
                        tw_sender
                            .send(Message::TaskWatcher_WATCH_EXECUTION {
                                task_id: id,
                                exec_process: result,
                                output_resp: o_tx,
                                resp: er_tx,
                            })
                            .await;
                        while let Ok(output) = o_rx.recv().await {
                            o_emitter.send(output);
                        }
                        let report = er_rx.await.unwrap();
                        Self::send_db_create_execution_report_message(db_report_sender, report)
                            .await;
                    });
                    task_model.exec_count += 1;
                    task_model.last_execution = Some(now!());
                    task_model.next_execution = task_model.calc_next_execution();

                    // TODO: Remove last_exec_succeeded property
                    task_model.last_exec_succeeded = true;
                    let (db_tx, _) = tokio::sync::oneshot::channel();
                    db_sender
                        .send(Message::DB_UPTADE_TASK {
                            task: task_model,
                            resp: db_tx,
                        })
                        .await;
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
    #[instrument(
        name = "Listening for messages from server."
        skip(rx, db_sender)
    )]
    pub async fn listen_for_server(
        rx: &mut ServerReceiver,
        db_sender: MPSCMessageSender,
        executor_sender: ExecutorSender,
    ) {
        while let Some(message) = rx.recv().await {
            info!("Got a message from server: {}", message.get_type());
            match message {
                Message::Server_GET_TASKS { offset, resp } => {
                    let (db_tx, db_rx) = tokio::sync::oneshot::channel();
                    db_sender
                        .send(Message::DB_GET_TASKS {
                            offset,
                            resp: db_tx,
                        })
                        .await;
                    let result = db_rx.await.unwrap();
                    resp.send(result);
                }
                Message::Server_EXECUTE_TASK { task_id, resp } => {
                    let (db_tx, db_rx) = tokio::sync::oneshot::channel();
                    db_sender
                        .send(Message::DB_GET_TASK {
                            id: task_id,
                            resp: db_tx,
                        })
                        .await;
                    let task = db_rx.await.unwrap();
                    match task {
                        Ok(task) => {
                            let boxed_task;
                            ModelToTask!(task => boxed_task);
                            match boxed_task {
                                Some(task) => {
                                    let o = Self::send_executor_execute_message(
                                        executor_sender.clone(),
                                        task,
                                    )
                                    .await
                                    .await
                                    .unwrap()
                                    .unwrap();
                                    resp.send(Ok(o));
                                }
                                None => {}
                            }
                        }
                        Err(e) => {
                            resp.send(Err(e));
                        }
                    }
                },
                _ => panic!()
            }
        }
    }
    async fn send_db_get_scheduled_tasks_message(&self) -> Result<Vec<TaskModel>, sqlx::Error> {
        let when = now!();
        info!(
            "Sending GET_SCHEDULED_TASKS message to DBManager time: {}",
            when.to_string()
        );
        let (tx, rx) = sync::oneshot::channel();
        self.db_sender
            .send(Message::DB_GET_SCHEDULED_TASKS { when, resp: tx })
            .await;
        return rx.await.unwrap();
    }
    pub async fn send_executor_execute_message(
        sender: ExecutorSender,
        task: BoxedTask,
    ) -> tokio::sync::oneshot::Receiver<Result<BoxedStream, TaskError>> {
        let id = task.get_id();
        info!("Sending Execute message to Executor for task {}", id);
        let (t_tx, t_rx) = tokio::sync::oneshot::channel();
        let message = Message::Executor_Execute { task, resp: t_tx };
        sender.send(message).await;
        return t_rx;
    }
    async fn send_db_create_execution_report_message(
        sender: MPSCMessageSender,
        report: ExecutionReport,
    ) -> Result<ExecutionReport, sqlx::Error> {
        let when = now!();
        info!(
            "Sending CREATE_EXECUTION_REPORT message to DBManager: {}",
            report.task_id
        );
        let (tx, rx) = sync::oneshot::channel();
        sender
            .send(Message::DB_CREATE_EXECUTION_REPORT { resp: tx, report })
            .await;
        return rx.await.unwrap();
    }
}
