use chrono::Utc;
use futures::StreamExt;
use tokio::sync;
use tracing::{info, warn};

use crate::{ModelToTask, db::DBMessage, executor::ExecutorMessage, models::{OutputModel, TaskError, TaskModel}, now, traits::BoxedStream, types::{BoxedTask, DBSender, ExecutorSender, OutputSender}};

/// Reactor's main job is to listen for messages
/// And transfer it to certain channel
///
/// It's another duty is to send GET_SCHEDULED_TASKS
/// message every second.
///
///
///
///
///
///
pub struct Reactor {
    pub db_sender: DBSender,
    pub executor_sender: ExecutorSender,
    pub output_sender: OutputSender,
}

impl Reactor {
    pub async fn listen(&mut self) {
        loop {
            let task_models = match self.send_get_scheduled_tasks_message().await {
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
            let mut task_rxs = vec![];
            for task in tasks.next() {
                if let Some(task) = task {
                    let id = task.get_id();
                    // println!("{}", id);
                    // println!("{:?}", task_models);
                    let index = search_iter.position(|t| t.id == id);
                    let task_model = task_models[index.unwrap()].clone();
                    task_rxs.push((
                        Self::send_executor_execute_message(self.executor_sender.clone(), task)
                            .await,
                        id,
                        task_model,
                    ));
                }
            }
            for (result_receiver, id, mut task_model) in task_rxs.into_iter() {
                let db_sender = self.db_sender.clone();
                let output_sender = self.output_sender.clone();
                let (db_tx, _) = tokio::sync::oneshot::channel();
                let (dberr_tx, _) = tokio::sync::oneshot::channel();
                tokio::spawn(async move {
                    info!("Waiting for result of task: {}", id);
                    match result_receiver.await.unwrap() {
                        // TODO: Handle output
                        Ok(mut r) => {
                            while let Some(output) = r.next().await {
                                // println!("{}", x);
                                output_sender.send(OutputModel::new(id, output));
                            }
                            info!("Execution of task {} is finished successfully.", id);
                            task_model.exec_count += 1;
                            task_model.last_execution = Some(now!());
                            task_model.next_execution = task_model.calc_next_execution();
                            task_model.last_exec_succeeded = true;
                            db_sender
                                .send(DBMessage::UPTADE_TASK {
                                    task: task_model,
                                    resp: db_tx,
                                })
                                .await;
                        }
                        Err(e) => {
                            warn!("Execution of task {} is finished with an error {}.", id, e.to_string());
                            db_sender
                                .send(DBMessage::CREATE_ERROR {
                                    resp: dberr_tx,
                                    error: e,
                                })
                                .await
                                .unwrap();
                        }
                    };
                });
            }
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
    async fn send_get_scheduled_tasks_message(&self) -> Result<Vec<TaskModel>, sqlx::Error> {
        let when = now!();
        info!(
            "Sending GET_SCHEDULED_TASKS message to DBManager time: {}",
            when.to_string()
        );
        let (tx, rx) = sync::oneshot::channel();
        self.db_sender
            .send(DBMessage::GET_SCHEDULED_TASKS { when, resp: tx })
            .await;
        return rx.await.unwrap();
    }
    async fn send_executor_execute_message(
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
}
