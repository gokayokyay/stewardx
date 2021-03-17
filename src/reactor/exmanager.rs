use std::sync::Arc;

use tokio::sync::{self, Mutex};
use tracing::{info, instrument, warn};
use uuid::Uuid;

use crate::{
    db::DBMessage,
    executor::ExecutorMessage,
    models::{ExecutionReport, TaskError, TaskModel},
    now,
    server::ServerMessage,
    traits::BoxedStream,
    types::{
        BoxedTask, DBSender, ExecutorSender, OutputEmitter, ServerReceiver, TaskWatcherSender,
    },
    ModelToTask,
};

/// Reactor's main job is to listen for messages
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
pub struct Reactor {
    pub db_sender: DBSender,
    pub executor_sender: ExecutorSender,
    pub task_watcher_sender: TaskWatcherSender,
    pub output_emitter: OutputEmitter,
    pub server_receiver: Arc<Mutex<ServerReceiver>>,
}

impl Reactor {
    pub async fn listen(&mut self) {
        let server_receiver = self.server_receiver.clone();
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
                    let exec_sender = self.executor_sender.clone();
                    tokio::spawn(async move {
                        let result = match result.await {
                            Ok(r) => r,
                            Err(_e) => {
                                // Receiver dropped
                                let report = ExecutionReport::new(id, false, Vec::default());
                                Self::send_db_create_execution_report_message(
                                    db_report_sender,
                                    report,
                                )
                                .await;
                                return;
                            }
                        };
                        let (o_tx, mut o_rx) = tokio::sync::broadcast::channel(128);
                        let (er_tx, er_rx) = tokio::sync::oneshot::channel();
                        tw_sender
                            .send(crate::tasks::TaskWatcherMessage::WatchExecution {
                                task_id: id,
                                exec_process: result,
                                output_resp: o_tx,
                                resp: er_tx,
                            })
                            .await;
                        while let Ok(output) = o_rx.recv().await {
                            o_emitter.send(output);
                        }
                        // If there's no output handle it means that task has finished execution
                        Self::send_executor_exec_finished_message(exec_sender, id).await;
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
                        .send(DBMessage::UptadeTask {
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
        db_sender: DBSender,
        executor_sender: ExecutorSender,
    ) {
        while let Some(message) = rx.recv().await {
            info!("Got a message from server: {}", message.get_type());
            match message {
                ServerMessage::GetTasks { offset, resp } => {
                    let (db_tx, db_rx) = tokio::sync::oneshot::channel();
                    db_sender
                        .send(DBMessage::GetTasks {
                            offset,
                            resp: db_tx,
                        })
                        .await;
                    let result = db_rx.await.unwrap();
                    resp.send(result);
                }
                ServerMessage::ExecuteTask { task_id, resp } => {
                    let (db_tx, db_rx) = tokio::sync::oneshot::channel();
                    db_sender
                        .send(DBMessage::GetTask {
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
                                None => {
                                    // resp.send(t)
                                }
                            }
                        }
                        Err(e) => {
                            resp.send(Err(e));
                        }
                    }
                }
                ServerMessage::AbortTask { task_id, resp } => {
                    let rx =
                        Self::send_executor_abort_message(executor_sender.clone(), task_id).await;
                    match rx.await {
                        Ok(result) => {
                            resp.send(result);
                        }
                        Err(e) => {
                            info!("Non critical error while aborting task {}", e);
                            resp.send(false);
                        }
                    };
                }
            }
        }
    }
    async fn send_db_get_scheduled_tasks_message(&self) -> Result<Vec<TaskModel>, anyhow::Error> {
        let when = now!();
        info!(
            "Sending GET_SCHEDULED_TASKS message to DBManager time: {}",
            when.to_string()
        );
        let (tx, rx) = sync::oneshot::channel();
        self.db_sender
            .send(DBMessage::GetScheduledTasks { when, resp: tx })
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
        let message = ExecutorMessage::Execute { task, resp: t_tx };
        sender.send(message).await;
        return t_rx;
    }
    pub async fn send_executor_abort_message(
        sender: ExecutorSender,
        id: Uuid,
    ) -> tokio::sync::oneshot::Receiver<bool> {
        info!("Sending Abort message to Executor for task {}", id);
        let (t_tx, t_rx) = tokio::sync::oneshot::channel();
        let message = ExecutorMessage::Abort { id, resp: t_tx };
        sender.send(message).await;
        return t_rx;
    }
    pub async fn send_executor_exec_finished_message(sender: ExecutorSender, id: Uuid) {
        info!(
            "Sending ExecutionFinished message to Executor for task {}",
            id
        );
        // let (t_tx, t_rx) = tokio::sync::oneshot::channel();
        let message = ExecutorMessage::ExecutionFinished { id };
        sender.send(message).await;
        // return t_rx;
    }
    async fn send_db_create_execution_report_message(
        sender: DBSender,
        report: ExecutionReport,
    ) -> Result<ExecutionReport, anyhow::Error> {
        info!(
            "Sending CREATE_EXECUTION_REPORT message to DBManager: {}",
            report.task_id
        );
        let (tx, rx) = sync::oneshot::channel();
        sender
            .send(DBMessage::CreateExecutionReport { resp: tx, report })
            .await;
        return rx.await.unwrap();
    }
}
