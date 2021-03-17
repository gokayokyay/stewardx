mod messages;

use std::sync::Arc;
use tokio::sync::{Mutex, oneshot, broadcast};

use crate::{now, executor::ExecutorMessage, db::DBMessage, models::ExecutionReport, tasks::TaskWatcherMessage, ModelToTask, types::{DBSender, ExecutorSender, OutputSender, ReactorReceiver, ReactorSender, ServerReceiver, TaskWatcherSender}};
pub use messages::ReactorMessage;

use tracing::{error, info};

pub struct Reactor {
    pub db_sender: DBSender,
    pub executor_sender: ExecutorSender,
    pub task_watcher_sender: TaskWatcherSender,
    pub output_emitter: OutputSender,
    pub server_receiver: Arc<Mutex<ServerReceiver>>,
    pub inner_sender: ReactorSender
}

impl Reactor {
    pub async fn schedule(sender: ReactorSender) {
        loop {
            sender.send(ReactorMessage::ExecuteScheduledTasks {
                when: now!()
            }).await;
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
    pub async fn listen(&mut self, mut receiver: ReactorReceiver) {
        let schedule_sender = self.inner_sender.clone();
        tokio::spawn(async move {
            Self::schedule(schedule_sender.clone()).await;
        });
        while let Some(message) = receiver.recv().await {
            let db_sender = self.db_sender.clone();
            let executor_sender = self.executor_sender.clone();
            let task_watcher_sender = self.task_watcher_sender.clone();
            let output_emitter = self.output_emitter.clone();
            let inner_sender = self.inner_sender.clone();
            tokio::spawn(async move {
                match message {
                    // TODO wrong use refactor later
                    ReactorMessage::GetScheduledTasks { when, resp } => {
                        db_sender.send(crate::db::DBMessage::GetScheduledTasks {
                            when,
                            resp,
                        }).await;
                    }
                    ReactorMessage::ExecuteScheduledTasks { when } => {
                        let (tx, rx) = oneshot::channel();
                        let task_models = inner_sender.send(ReactorMessage::GetScheduledTasks {
                            when,
                            resp: tx,
                        }).await;
                        let task_models = match rx.await {
                            Ok(models) => {
                                match models {
                                    Ok(models) => models,
                                    Err(e) => {
                                        error!("{}", e.to_string());
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                error!("{}", e.to_string());
                                return;
                            }
                        };
                        let mut tasks = task_models.iter().map(|task| {
                            let boxed_task;
                            ModelToTask!(task => boxed_task);
                            return boxed_task;
                        });
                        for task in tasks.next() {
                            if let Some(task) = task {
                                let id = task.get_id();
                                inner_sender.send(ReactorMessage::ExecuteTask {
                                    task,
                                }).await;
                            }
                        }
                    },
                    
                    ReactorMessage::ExecuteTask { task } => {
                        let id = task.get_id();
                        info!("Sending Execute message to Executor for task {}", id);
                        let (t_tx, t_rx) = oneshot::channel();
                        let message = ExecutorMessage::Execute { task, resp: t_tx };
                        executor_sender.send(message).await;
                        let result = match t_rx.await {
                            Ok(r) => {
                                match r {
                                    Ok(o) => o,
                                    Err(e) => {
                                        error!("{}", e.to_string());
                                        // TODO: Handle error and add it to db
                                        return;
                                    }
                                }
                            },
                            Err(e) => {
                                // Receiver dropped
                                error!("{}", e.to_string());
                                let report = ExecutionReport::new(id, false, Vec::default());
                                let (er_tx, _) = oneshot::channel();
                                inner_sender.send(ReactorMessage::CreateExecutionReport {
                                    report,
                                    resp: er_tx,
                                }).await;
                                return;
                            }
                        };
                        // let (o_tx, mut o_rx) = tokio::sync::broadcast::channel(128);
                        // let (er_tx, er_rx) = tokio::sync::oneshot::channel();
                        // task_watcher_sender.send(TaskWatcherMessage::WatchExecution {
                            
                        // }).await;
                        inner_sender.send(ReactorMessage::WatchExecution {
                            task_id: id,
                            exec_process: Ok(result),
                        }).await;

                    },
                    ReactorMessage::CreateExecutionReport { report, resp } => {
                        info!("Sending CreateExecutionReport message to DBManager: {}", report.task_id);
                        let (tx, rx) = oneshot::channel();
                        db_sender
                            .send(DBMessage::CreateExecutionReport { resp: tx, report })
                            .await;
                        let report = rx.await.unwrap();
                        resp.send(report);
                    },
                    ReactorMessage::WatchExecution { task_id, exec_process } => {
                        let (o_tx, mut o_rx) = broadcast::channel(128);
                        let (er_tx, er_rx) = oneshot::channel();
                        task_watcher_sender.send(TaskWatcherMessage::WatchExecution {
                            task_id,
                            exec_process,
                            output_resp: o_tx,
                            resp: er_tx,
                        }).await;
                        while let Ok(output) = o_rx.recv().await {
                            inner_sender.send(ReactorMessage::OutputReceived {
                                model: output
                            }).await;
                        }
                        // If there's no output handle, then it means the task has finished execution
                        inner_sender.send(ReactorMessage::ExecutionFinished {
                            id: task_id,
                            successful: true
                        }).await;
                    },
                    ReactorMessage::OutputReceived { model } => {
                        output_emitter.send(model);
                    }
                    ReactorMessage::ExecutionFinished { id, successful } => {
                        info!("{}'s execution has finished", id);
                        let message = ExecutorMessage::ExecutionFinished { id };
                        executor_sender.send(message).await;
                        // Update task
                        let (db_tx, db_rx) = oneshot::channel();
                        let _res = db_sender.send(DBMessage::GetTask {
                            id,
                            resp: db_tx,
                        }).await;
                        // Unwrapping it because we're sure it exists on db possible TODO ?
                        let mut task_model = db_rx.await.unwrap().unwrap();
                        task_model.exec_count += 1;
                        task_model.last_execution = Some(now!());
                        task_model.next_execution = task_model.calc_next_execution();
                        let (db_tx, _) = tokio::sync::oneshot::channel();
                        db_sender.send(DBMessage::UptadeTask {
                            task: task_model,
                            resp: db_tx,
                        }).await;
                    }
                }
            });
        }
    }
}