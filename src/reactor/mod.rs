mod messages;

use std::sync::Arc;
use tokio::sync::{broadcast, oneshot, Mutex};

use crate::{
    db::DBMessage,
    executor::ExecutorMessage,
    models::{ExecutionReport, TaskError, TaskModel},
    now,
    server::ServerMessage,
    tasks::TaskWatcherMessage,
    types::{
        DBSender, ExecutorSender, OutputSender, ReactorReceiver, ReactorSender, ServerReceiver,
        TaskWatcherSender,
    },
    ModelToTask,
};
pub use messages::ReactorMessage;

use tracing::{error, info};

pub struct Reactor {
    pub db_sender: DBSender,
    pub executor_sender: ExecutorSender,
    pub task_watcher_sender: TaskWatcherSender,
    pub output_emitter: OutputSender,
    pub server_receiver: Arc<Mutex<ServerReceiver>>,
    pub inner_sender: ReactorSender,
}

impl Reactor {
    pub async fn schedule(sender: ReactorSender) {
        loop {
            match sender
                .send(ReactorMessage::ExecuteScheduledTasks { when: now!() })
                .await {
                    Ok(_) => {},
                    Err(e) => {
                        error!("FATAL: Reactor couldn't get the ExecuteScheduledTasks message.");
                        panic!("{}", e.to_string());
                    }
                };
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        }
    }
    pub async fn listen_for_server(receiver: &mut ServerReceiver, inner_sender: ReactorSender) {
        while let Some(message) = receiver.recv().await {
            let reactor_message = match message {
                ServerMessage::GetTasks { offset, resp } => {
                    ReactorMessage::ServerGetTasks { offset, resp }
                }
                ServerMessage::ExecuteTask { task_id, resp } => {
                    ReactorMessage::ServerExecuteTask { task_id, resp }
                }
                ServerMessage::AbortTask { task_id, resp } => {
                    ReactorMessage::ServerAbortTask { task_id, resp }
                }
                ServerMessage::DeleteTask { task_id, resp } => {
                    ReactorMessage::ServerDeleteTask { task_id, resp }
                }

                ServerMessage::CreateTask {
                    task_name,
                    frequency,
                    task_type,
                    task_props,
                    resp,
                } => ReactorMessage::ServerCreateTask {
                    task_name,
                    frequency,
                    task_type,
                    task_props,
                    resp,
                },
                ServerMessage::GetActiveTasks { resp } => {
                    ReactorMessage::ServerGetActiveTasks { resp }
                }
                ServerMessage::GetTask { task_id, resp } => {
                    ReactorMessage::ServerGetTask { task_id, resp }
                }
                ServerMessage::UpdateTask {
                    task_id,
                    task_name,
                    frequency,
                    task_props,
                    resp,
                } => ReactorMessage::ServerUpdateTask {
                    task_id,
                    task_name,
                    frequency,
                    task_props,
                    resp,
                },
                ServerMessage::GetExecutionReportsForTask {
                    task_id,
                    offset,
                    resp,
                } => ReactorMessage::ServerGetExecutionReportsForTask {
                    task_id,
                    offset,
                    resp,
                },
                ServerMessage::GetExecutionReports { offset, resp } => {
                    ReactorMessage::ServerGetExecutionReports { offset, resp }
                }
                ServerMessage::GetExecutionReport { report_id, resp } => {
                    ReactorMessage::ServerGetExecutionReport { report_id, resp }
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
            macro_rules! didnt_receive {
                ($send: expr, $sender_type: expr, $msg: tt) => {
                    match $send {
                        Ok(_) => {},
                        Err(e) => {
                            error!("{} didnt receive the {} message!", $sender_type, $msg);
                            panic!("{}", e.to_string());
                        }
                    }
                }
            }
            macro_rules! server_receiver_dropped {
                ($resp: expr, $msg: tt) => {
                    $resp.expect(&format!("Server receiver is dropped, {}", $msg));
                }
            }
            let msg_type = message.get_type().to_string();
            info!("Reactor got message {}", msg_type);
            let db_sender = self.db_sender.clone();
            let executor_sender = self.executor_sender.clone();
            let task_watcher_sender = self.task_watcher_sender.clone();
            let output_emitter = self.output_emitter.clone();
            let inner_sender = self.inner_sender.clone();
            tokio::spawn(async move {
                match message {
                    // TODO wrong use refactor later
                    ReactorMessage::GetScheduledTasks { when, resp } => {
                        didnt_receive!(db_sender
                            .send(crate::db::DBMessage::GetScheduledTasks { when, resp })
                            .await, "Database", "GetScheduledTasks");
                    }
                    ReactorMessage::ExecuteScheduledTasks { when } => {
                        let (tx, rx) = oneshot::channel();
                        didnt_receive!(inner_sender
                            .send(ReactorMessage::GetScheduledTasks { when, resp: tx })
                            .await, "Reactor", "GetScheduledTasks");
                        let task_models = match rx.await {
                            Ok(models) => match models {
                                Ok(models) => models,
                                Err(e) => {
                                    // If somehow we got here, I'm assuming that the DB has an error
                                    // therefore I wont be saving the error.
                                    error!("{}", e.to_string());
                                    return;
                                }
                            },
                            Err(e) => {
                                // Same as above
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
                                let _id = task.get_id();
                                didnt_receive!(inner_sender
                                    .send(ReactorMessage::ExecuteTask { task })
                                    .await, "Reactor", "ExecuteTask");
                            }
                        }
                    }
                    ReactorMessage::ExecuteTask { task } => {
                        let id = task.get_id();
                        info!("Sending Execute message to Executor for task {}", id);
                        let (t_tx, t_rx) = oneshot::channel();
                        let message = ExecutorMessage::Execute { task, resp: t_tx };
                        didnt_receive!(executor_sender.send(message).await, "Executor", msg_type);
                        didnt_receive!(inner_sender
                            .send(ReactorMessage::UpdateTaskExecution { task_id: id })
                            .await, "Reactor", msg_type);
                        let result = match t_rx.await {
                            Ok(r) => {
                                match r {
                                    Ok(o) => o,
                                    Err(e) => {
                                        error!("{}", e.to_string());
                                        didnt_receive!(inner_sender.send(ReactorMessage::CreateError {
                                            error: e,
                                        }).await, "Reactor", "CreateError");
                                        return;
                                    }
                                }
                            }
                            Err(e) => {
                                // Receiver dropped
                                error!("{}", e.to_string());
                                let report = ExecutionReport::new(id, false, Vec::default());
                                // We wont be creating an error, because in this case, well, I forgot
                                // But I didn't add a TODO here so it should be the expected behavior?
                                didnt_receive!(inner_sender
                                    .send(ReactorMessage::CreateExecutionReport { report })
                                    .await, "Reactor", "CreateExecutionReport");
                                return;
                            }
                        };
                        // let (o_tx, mut o_rx) = tokio::sync::broadcast::channel(128);
                        // let (er_tx, er_rx) = tokio::sync::oneshot::channel();
                        // task_watcher_sender.send(TaskWatcherMessage::WatchExecution {

                        // }).await;
                        didnt_receive!(inner_sender
                            .send(ReactorMessage::WatchExecution {
                                task_id: id,
                                exec_process: Ok(result),
                            })
                            .await, "Reactor", "WatchExecution");
                    }
                    ReactorMessage::CreateExecutionReport { report } => {
                        info!(
                            "Sending CreateExecutionReport message to DBManager: {}",
                            report.task_id
                        );
                        let (tx, rx) = oneshot::channel();
                        didnt_receive!(db_sender
                            .send(DBMessage::CreateExecutionReport { resp: tx, report })
                            .await, "Database", "CreateExecutionReport");
                        let _report = rx.await.unwrap();
                        // resp.send(report);
                    }
                    ReactorMessage::WatchExecution {
                        task_id,
                        exec_process,
                    } => {
                        let (o_tx, mut o_rx) = broadcast::channel(128);
                        let (er_tx, er_rx) = oneshot::channel();
                        didnt_receive!(task_watcher_sender
                            .send(TaskWatcherMessage::WatchExecution {
                                task_id,
                                exec_process,
                                output_resp: o_tx,
                                resp: er_tx,
                            })
                            .await, "TaskWatcher", "WatchExecution");
                        while let Ok(output) = o_rx.recv().await {
                            didnt_receive!(inner_sender
                                .send(ReactorMessage::OutputReceived { model: output })
                                .await, "Reactor", "OutputReceived");
                        }
                        // If output receiver is dropped, it means that execution has finished!
                        if let Ok(report) = er_rx.await {
                            didnt_receive!(inner_sender
                                .send(ReactorMessage::CreateExecutionReport { report })
                                .await, "Reactor", "CreateExecutionReport");
                        }
                        didnt_receive!(inner_sender
                            .send(ReactorMessage::ExecutionFinished {
                                id: task_id,
                                should_update: false,
                            })
                            .await, "Reactor", "ExecutionFinished");
                    }
                    ReactorMessage::OutputReceived { model } => {
                        match output_emitter.send(model) {
                            Ok(_) => {},
                            Err(e) => {
                                error!("Couldn't send OutputModel to OutputListener, is it awake?");
                                panic!("{}", e.to_string());
                            }
                        };
                    }
                    ReactorMessage::ExecutionFinished { id, should_update } => {
                        info!("{}'s execution has finished", id);
                        let message = ExecutorMessage::ExecutionFinished { id };
                        didnt_receive!(executor_sender.send(message).await, "Executor", "ExecutionFinished");
                        if should_update {
                            didnt_receive!(inner_sender
                                .send(ReactorMessage::UpdateTaskExecution { task_id: id })
                                .await, "Reactor", "UpdateTaskExecution");
                        }
                    }
                    ReactorMessage::ServerGetTasks { offset, resp } => {
                        let (db_tx, db_rx) = tokio::sync::oneshot::channel();
                        didnt_receive!(
                            db_sender
                                .send(DBMessage::GetTasks {
                                    offset,
                                    resp: db_tx,
                                })
                                .await, "Database", "GetTasks");
                        let result = db_rx.await.unwrap();
                        server_receiver_dropped!(resp.send(result), "ServerGetTasks");
                    }
                    ReactorMessage::ServerExecuteTask { task_id, resp } => {
                        let (db_tx, db_rx) = tokio::sync::oneshot::channel();
                        didnt_receive!(db_sender
                            .send(DBMessage::GetTask {
                                id: task_id,
                                resp: db_tx,
                            })
                            .await, "Database", "GetTask");
                        let task = db_rx.await.unwrap();
                        match task {
                            Ok(task) => {
                                let boxed_task;
                                ModelToTask!(task => boxed_task);
                                match boxed_task {
                                    Some(task) => {
                                        didnt_receive!(inner_sender
                                            .send(ReactorMessage::ExecuteTask { task })
                                            .await, "Reactor", "ExecuteTask");
                                        server_receiver_dropped!(resp.send(true), "ServerExecuteTask");
                                    }
                                    None => {
                                        let task_json = serde_json::to_string(&task).unwrap_or(task.task_type);
                                        didnt_receive!(inner_sender.send(ReactorMessage::CreateError {
                                            error: TaskError::generic(task.id, format!("Task couldn't be parsed to boxed task, {}", task_json))
                                        }).await, "Reactor", "CreateError");
                                        server_receiver_dropped!(resp.send(false), "ServerExecuteTask");
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("{}", e.to_string());
                                error!("{}", e.to_string());
                                let error = TaskError::generic(task_id, e.to_string());
                                didnt_receive!(inner_sender
                                    .send(ReactorMessage::CreateError { error })
                                    .await, "Reactor", "CreateError");
                                server_receiver_dropped!(resp.send(false), "ServerExecuteTask");
                            }
                        }
                    }
                    ReactorMessage::ServerAbortTask { task_id, resp } => {
                        didnt_receive!(executor_sender
                            .send(ExecutorMessage::Abort { id: task_id, resp })
                            .await, "Executor", "Abort");
                    }
                    ReactorMessage::ServerDeleteTask { task_id, resp } => {
                        didnt_receive!(db_sender
                            .send(DBMessage::DeleteTask { id: task_id, resp })
                            .await, "Database", "DeleteTask");
                    }
                    ReactorMessage::UpdateTaskExecution { task_id } => {
                        // Update task
                        let (db_tx, db_rx) = oneshot::channel();
                        didnt_receive!(db_sender
                            .send(DBMessage::GetTask {
                                id: task_id,
                                resp: db_tx,
                            })
                            .await, "Database", "GetTask");
                        // Unwrapping it because we're sure it exists on db possible TODO ?
                        let mut task_model = db_rx.await.unwrap().unwrap();
                        task_model.exec_count += 1;
                        task_model.last_execution = Some(now!());
                        task_model.next_execution = task_model.calc_next_execution();
                        let (db_tx, _) = tokio::sync::oneshot::channel();
                        didnt_receive!(db_sender
                            .send(DBMessage::UpdateTask {
                                task: task_model,
                                resp: db_tx,
                            })
                            .await, "Database", "UpdateTask");
                    }
                    ReactorMessage::ServerCreateTask {
                        task_name,
                        frequency,
                        task_type,
                        task_props,
                        resp,
                    } => {
                        let new_id = uuid::Uuid::new_v4();
                        let serde_string = match TaskModel::get_serde_from_props(
                            new_id,
                            task_type.clone(),
                            task_props.clone(),
                        ) {
                            Ok(s) => s,
                            Err(e) => {
                                server_receiver_dropped!(resp.send(Err(e)), "ServerCreateTask");
                                return;
                            }
                        };
                        let task = TaskModel::new(
                            Some(new_id),
                            task_name,
                            task_type,
                            serde_string,
                            frequency,
                        );
                        let (tx, rx) = tokio::sync::oneshot::channel();
                        didnt_receive!(db_sender
                            .send(DBMessage::CreateTask { task, resp: tx })
                            .await, "Database", "CreateTask");
                        let result = rx.await.unwrap();
                        server_receiver_dropped!(resp.send(result), "ServerCreateTask");
                    }
                    ReactorMessage::ServerGetActiveTasks { resp } => {
                        let (e_tx, e_rx) = oneshot::channel();
                        didnt_receive!(executor_sender
                            .send(ExecutorMessage::GetActiveTaskIDs { resp: e_tx })
                            .await, "Executor", "GetActiveTaskIDs");
                        let active_task_ids = e_rx.await.unwrap();
                        let mut active_tasks = vec![];
                        // TODO: Find a better way in future
                        for task_id in active_task_ids {
                            let (db_tx, db_rx) = oneshot::channel();
                            didnt_receive!(db_sender
                                .clone()
                                .send(DBMessage::GetTask {
                                    id: task_id,
                                    resp: db_tx,
                                })
                                .await, "Database", "GetTask");
                            let task = db_rx.await.unwrap();
                            match task {
                                Ok(task) => {
                                    active_tasks.push(task);
                                }
                                Err(e) => {
                                    let error_str = e.to_string();
                                    server_receiver_dropped!(resp.send(Err(e)), "ServerGetActiveTasks");
                                    didnt_receive!(inner_sender.send(ReactorMessage::CreateError {
                                        error: TaskError::generic(uuid::Uuid::default(), error_str)
                                    }).await, "Reactor", "CreateError");
                                    return;
                                }
                            };
                        }
                        server_receiver_dropped!(resp.send(Ok(active_tasks)), "ServerGetActiveTasks");
                    }
                    ReactorMessage::ServerGetTask { task_id, resp } => {
                        let (tx, rx) = oneshot::channel();
                        didnt_receive!(db_sender
                            .send(DBMessage::GetTask {
                                id: task_id,
                                resp: tx,
                            })
                            .await, "Database", "GetTask");

                        let res = rx.await.unwrap();
                        server_receiver_dropped!(resp.send(res), "ServerGetTask");
                    }
                    ReactorMessage::ServerUpdateTask {
                        task_id,
                        task_name,
                        frequency,
                        task_props,
                        resp,
                    } => {
                        let (task_tx, task_rx) = oneshot::channel();
                        didnt_receive!(inner_sender
                            .clone()
                            .send(ReactorMessage::ServerGetTask {
                                task_id,
                                resp: task_tx,
                            })
                            .await, "Reactor", "ServerGetTask");
                        let task = task_rx.await.unwrap();
                        let mut task = match task {
                            Ok(t) => t,
                            Err(e) => {
                                server_receiver_dropped!(resp.send(Err(e)), "ServerUpdateTask");
                                return;
                            }
                        };
                        task.task_name = task_name;
                        task.frequency = frequency;
                        let serde_string = match TaskModel::get_serde_from_props(
                            task_id,
                            task.task_type.clone(),
                            task_props,
                        ) {
                            Ok(s) => s,
                            Err(e) => {
                                let err_str = e.to_string();
                                server_receiver_dropped!(resp.send(Err(e)), "ServerUpdateTask");
                                didnt_receive!(inner_sender.send(ReactorMessage::CreateError {
                                    error: TaskError::generic(task.id, err_str)
                                }).await, "Reactor", "CreateError");
                                return;
                            }
                        };
                        task.serde_string = serde_string;
                        let (db_tx, db_rx) = oneshot::channel();
                        didnt_receive!(db_sender
                            .send(DBMessage::UpdateTask { task, resp: db_tx })
                            .await, "Database", "UpdateTask");
                        let result = db_rx.await.unwrap();
                        server_receiver_dropped!(resp.send(result), "ServerUpdateTask");
                    }
                    ReactorMessage::ServerGetExecutionReportsForTask {
                        task_id,
                        offset,
                        resp,
                    } => {
                        let (db_tx, db_rx) = oneshot::channel();
                        didnt_receive!(db_sender
                            .send(DBMessage::GetExecutionReportsForTask {
                                task_id,
                                offset,
                                resp: db_tx,
                            })
                            .await, "Database", "GetExecutionReportsForTask");
                        let result = db_rx.await.unwrap();
                        server_receiver_dropped!(resp.send(result), "ServerGetExecutionReportsForTask");
                    }
                    ReactorMessage::ServerGetExecutionReports { offset, resp } => {
                        let (db_tx, db_rx) = oneshot::channel();
                        didnt_receive!(db_sender
                            .send(DBMessage::GetExecutionReports {
                                offset,
                                resp: db_tx,
                            })
                            .await, "Database", "GetExecutionReports");
                        let result = db_rx.await.unwrap();
                        server_receiver_dropped!(resp.send(result), "ServerGetExecutionReports");
                    }
                    ReactorMessage::ServerGetExecutionReport { report_id, resp } => {
                        didnt_receive!(db_sender
                            .send(DBMessage::GetExecutionReport { report_id, resp })
                            .await, "Database", "GetExecutionReport");
                    }
                    ReactorMessage::CreateError { error } => {
                        let (tx, _rx) = oneshot::channel();
                        didnt_receive!(db_sender.send(DBMessage::CreateError { error, resp: tx }).await, "Database", "CreateError");
                    }
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{tasks::CmdTask};
    use tokio::sync::*;
    use uuid::Uuid;

    async fn create_long_task() -> CmdTask {
        let sleep_and_print_and_create_file_command = r#"
            sleep 0.2s
            echo "Hey hey hey"
        "#;
        let _file =
            tokio::fs::write("temp_script.sh", sleep_and_print_and_create_file_command).await;
        let task = CmdTask::new(Uuid::new_v4(), Box::new("/bin/bash temp_script.sh".into()));
        return task;
    }

    #[tokio::test]
    async fn execution_flow() {
        let (db_tx, mut db_rx) = mpsc::channel(32);
        let (ex_tx, ex_rx) = mpsc::channel(32);
        let (sv_tx, sv_rx) = mpsc::channel(32);
        let (tw_tx, tw_rx) = mpsc::channel(32);
        let (oe_tx, oe_rx) = broadcast::channel(32);
        let server_receiver = std::sync::Arc::new(Mutex::new(sv_rx));
        let (r_tx, r_rx) = mpsc::channel(32);
        let mut reactor = Reactor {
            db_sender: db_tx,
            executor_sender: ex_tx,
            task_watcher_sender: tw_tx,
            output_emitter: oe_tx,
            server_receiver: server_receiver,
            inner_sender: r_tx,
        };
        tokio::spawn(async move {
            let mut fake_db = vec![create_long_task().await];
            match db_rx.recv().await.unwrap() {
                DBMessage::GetTask { resp, .. } => {
                    let task = fake_db.pop().unwrap();
                    fake_db.push(create_long_task().await);
                    resp.send(Ok(TaskModel::from_boxed_task(
                        Box::new(task),
                        "aaa".into(),
                        "Hook".into(),
                    )))
                    .unwrap();
                }
                DBMessage::GetScheduledTasks { resp, .. } => {
                    let task = fake_db.pop().unwrap();
                    fake_db.push(create_long_task().await);
                    resp.send(Ok(vec![TaskModel::from_boxed_task(
                        Box::new(task),
                        "aaa".into(),
                        "Hook".into(),
                    )]))
                    .unwrap();
                }
                DBMessage::UpdateNextExecution { resp, .. } => {
                    let task = fake_db.pop().unwrap();
                    fake_db.push(create_long_task().await);
                    resp.send(Ok(TaskModel::from_boxed_task(
                        Box::new(task),
                        "aaa".into(),
                        "Hook".into(),
                    )))
                    .unwrap();
                }
                DBMessage::UpdateTask { task, resp } => {
                    resp.send(Ok(task)).unwrap();
                }
                _ => panic!("Shouldn't happen! But when it does, please update the test :)"),
            };
        });
        let result = tokio::time::timeout(
            tokio::time::Duration::from_secs_f32(1.2),
            tokio::spawn(async move {
                reactor.listen(r_rx).await;
            }),
        )
        .await;
        // Meaning that (kinda) it has a timeout
        assert_eq!(result.is_err(), true);
    }
}
