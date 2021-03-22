mod messages;

use std::sync::Arc;
use tokio::sync::{Mutex, oneshot, broadcast};

use crate::{ModelToTask, db::DBMessage, executor::ExecutorMessage, models::ExecutionReport, now, server::ServerMessage, tasks::TaskWatcherMessage, types::{DBSender, ExecutorSender, OutputSender, ReactorReceiver, ReactorSender, ServerReceiver, TaskWatcherSender}};
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
    pub async fn listen_for_server(receiver: &mut ServerReceiver, inner_sender: ReactorSender) {
        while let Some(message) = receiver.recv().await {
            let reactor_message = match message {
                ServerMessage::GetTasks { offset, resp } => {
                    ReactorMessage::ServerGetTasks {
                        offset,
                        resp,
                    }
                }
                ServerMessage::ExecuteTask { task_id, resp } => {
                    ReactorMessage::ServerExecuteTask {
                        task_id,
                        resp,
                    }
                }
                ServerMessage::AbortTask { task_id, resp } => {
                    ReactorMessage::ServerAbortTask {
                        task_id,
                        resp,
                    }
                }
                ServerMessage::DeleteTask { task_id, resp } => {
                    ReactorMessage::ServerDeleteTask {
                        task_id,
                        resp
                    }
                }
            };
            inner_sender.send(reactor_message).await.unwrap_or_default();
        }
    }
    pub async fn listen(&mut self, mut receiver: ReactorReceiver) {
        let schedule_sender = self.inner_sender.clone();
        let inner_sender = self.inner_sender.clone();
        tokio::spawn(async move {
            Self::schedule(schedule_sender).await;
        });
        let server_receiver = self.server_receiver.clone();
        tokio::spawn(async move {
            let mut server_receiver = server_receiver.lock().await;
            let server_receiver = &mut *server_receiver;
            Self::listen_for_server(server_receiver, inner_sender).await;
        });
        while let Some(message) = receiver.recv().await {
            // println!("Received message {}", message.get_type());
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
                        inner_sender.send(ReactorMessage::GetScheduledTasks {
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
                        // println!("{:?}", task_models);
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
                                inner_sender.send(ReactorMessage::CreateExecutionReport {
                                    report,
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
                        inner_sender.send(ReactorMessage::UpdateTaskExecution {
                            task_id: id
                        }).await;
                    },
                    ReactorMessage::CreateExecutionReport { report } => {
                        info!("Sending CreateExecutionReport message to DBManager: {}", report.task_id);
                        let (tx, rx) = oneshot::channel();
                        db_sender
                            .send(DBMessage::CreateExecutionReport { resp: tx, report })
                            .await;
                        let report = rx.await.unwrap();
                        // resp.send(report);
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
                        if let Ok(report) = er_rx.await {
                            inner_sender.send(ReactorMessage::CreateExecutionReport {
                                report,
                            }).await;
                        }
                        inner_sender.send(ReactorMessage::ExecutionFinished {
                            id: task_id,
                            should_update: false
                        }).await;
                    },
                    ReactorMessage::OutputReceived { model } => {
                        output_emitter.send(model);
                    }
                    ReactorMessage::ExecutionFinished { id, should_update } => {
                        info!("{}'s execution has finished", id);
                        let message = ExecutorMessage::ExecutionFinished { id };
                        executor_sender.send(message).await;
                        if should_update {
                            inner_sender.send(ReactorMessage::UpdateTaskExecution {
                                task_id: id,
                            }).await;
                        }
                    }
                    ReactorMessage::ServerGetTasks { offset, resp } => {
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
                    ReactorMessage::ServerExecuteTask { task_id, resp } => {
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
                                        inner_sender.send(ReactorMessage::ExecuteTask {
                                            task,
                                        }).await;
                                        resp.send(true);
                                    }
                                    None => {
                                        resp.send(false);
                                    }
                                }
                            }
                            Err(e) => {
                                // TODO save error
                                error!("{}", e.to_string());
                                eprintln!("{}", e.to_string());
                                resp.send(false);
                            }
                        }
                    }
                    ReactorMessage::ServerAbortTask { task_id, resp } => {
                        executor_sender.send(ExecutorMessage::Abort {
                            id: task_id,
                            resp,
                        }).await;
                    }
                    ReactorMessage::ServerDeleteTask { task_id, resp } => {
                        db_sender.send(DBMessage::DeleteTask {
                            id: task_id,
                            resp,
                        }).await;
                    }
                    ReactorMessage::UpdateTaskExecution { task_id } => {
                        // Update task
                        let (db_tx, db_rx) = oneshot::channel();
                        let _res = db_sender.send(DBMessage::GetTask {
                            id: task_id,
                            resp: db_tx,
                        }).await;
                        // Unwrapping it because we're sure it exists on db possible TODO ?
                        let mut task_model = db_rx.await.unwrap().unwrap();
                        task_model.exec_count += 1;
                        task_model.last_execution = Some(now!());
                        task_model.next_execution = task_model.calc_next_execution();
                        let (db_tx, _) = tokio::sync::oneshot::channel();
                        db_sender.send(DBMessage::UpdateTask {
                            task: task_model,
                            resp: db_tx,
                        }).await;
                    }
                }
            });
        }
    }
}