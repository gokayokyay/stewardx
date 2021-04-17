use tokio::task::JoinHandle;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::{
    models::TaskError,
    traits::BoxedStream,
    types::{BoxedTask, OneShotMessageResponse},
};

use super::ExecutorMessage;

#[derive(Debug)]
pub struct TaskHandle {
    inner_handle: JoinHandle<()>,
    id: Uuid,
    // task_resp: OneShotMessageResponse<Result<BoxedStream, TaskError>>,
    abort_tx: OneShotMessageResponse<bool>,
}

pub struct Executor {
    pub task_handles: Vec<TaskHandle>,
}

impl Executor {
    #[instrument(skip(task), fields(task = %task.get_id()))]
    async fn execute(task: &mut BoxedTask) -> Result<BoxedStream, TaskError> {
        info!("Executing task");
        let handle = task.exec().await;
        info!("Task execution finished.");
        return handle;
    }
    pub async fn listen(
        &mut self,
        mut rx: tokio::sync::mpsc::Receiver<ExecutorMessage>,
        tx: tokio::sync::mpsc::Sender<ExecutorMessage>,
    ) {
        info!("Executor started listening...");
        while let Some(message) = rx.recv().await {
            info!("Executor got message: {}", message.get_type());
            let inner_tx = tx.clone();
            match message {
                ExecutorMessage::Execute { mut task, resp } => {
                    let id = task.get_id();
                    let (abort_tx, abort_rx) = tokio::sync::oneshot::channel::<bool>();
                    info!("Executing task: {}", id);
                    let handle = tokio::spawn(async move {
                        let result = Self::execute(&mut task).await;
                        resp.send(result).unwrap_or_default();
                        // task.abort().await;
                        // We can listen for the abort oneshot

                        if let Ok(_) = abort_rx.await {
                            info!("Aborting task {}", id);
                            task.abort().await;
                            // if let Ok(_) = inner_tx.send(ExecutorMessage::Abort { id }).await {}
                        }
                        info!("Aborting timespan finished.");
                    });
                    // println!("handle-end {}", task.get_id());
                    self.task_handles.push(TaskHandle {
                        inner_handle: handle,
                        id,
                        abort_tx,
                    });
                    // Reactor panics because of dropped response.
                    // To truly abort the task we need to store its resp too
                    // handle.abort();
                }
                ExecutorMessage::ExecutionFinished { id } => {
                    info!("Execution of task: {} is finished", id);
                    if let Some(index) = get_handle_index(&mut self.task_handles, id) {
                        let _val = self.task_handles.remove(index);
                    }
                }
                ExecutorMessage::Abort { id, resp } => {
                    if self.abort_task(id, resp).await {
                        inner_tx
                            .send(ExecutorMessage::ExecutionFinished { id })
                            .await.unwrap_or_default();
                    }
                }
                ExecutorMessage::GetActiveTaskIDs { resp } => {
                    resp.send(
                        self.task_handles.iter().map(|t| t.id).collect::<Vec<Uuid>>()
                    ).unwrap_or_default();
                }
            }
        }
    }
    pub async fn abort_task(
        &mut self,
        task_id: Uuid,
        resp: tokio::sync::oneshot::Sender<bool>,
    ) -> bool {
        if let Some(index) = get_handle_index(&mut self.task_handles, task_id) {
            info!("Found the task to abort: {}", task_id);
            let val = self.task_handles.remove(index);
            val.abort_tx.send(true).unwrap_or_default();
            // val.inner_handle.abort();
            resp.send(true).unwrap_or_default();
            return true;
        }
        resp.send(false).unwrap_or_default();
        return false;
    }
}

fn get_handle_index(task_handles: &mut Vec<TaskHandle>, task_id: Uuid) -> Option<usize> {
    let predicate = |t: &TaskHandle| t.id == task_id;
    let mut i: usize = 0;
    while i != task_handles.len() {
        if predicate(&mut task_handles[i]) {
            return Some(i);
        } else {
            i += 1;
        }
    }
    return None;
}

#[cfg(test)]
mod tests {
    use crate::tasks::CmdTask;
    use tokio::sync::*;
    use super::*;

    fn create_executor() -> Executor {
        let executor = Executor { task_handles: Vec::default() };
        return executor;
    }
    async fn create_boxed_long_task() -> BoxedTask {
        let sleep_and_print_and_create_file_command = r#"
            sleep 0.2s
            echo "Hey hey hey"
            touch testing.temp
        "#;
        let _file = tokio::fs::write("temp_script.sh", sleep_and_print_and_create_file_command).await;
        let task = CmdTask::new(Uuid::new_v4(), Box::new("/bin/bash temp_script.sh".into()));
        return Box::new(task);
    }
    async fn cleanup() {
        match tokio::fs::remove_file("testing.temp").await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e.to_string());
                println!("Couldn't cleanup after test, please locate and remove \"testing.temp\"");
            }
        };
        match tokio::fs::remove_file("temp_script.sh").await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e.to_string());
                println!("Couldn't cleanup after test, please locate and remove \"temp_script.sh\"");
            }
        };

    }
    #[tokio::test]
    async fn abort_task() {
        let (tx, rx) = mpsc::channel(32);
        let mut executor = create_executor();
        let inner_tx = tx.clone();
        let _handle = tokio::spawn(async move {
            executor.listen(rx, inner_tx).await;
        });
        let task = create_boxed_long_task().await;
        let id = task.get_id();
        let (exec_tx, _exec_rx) = oneshot::channel();
        match tx.send(ExecutorMessage::Execute {
            task,
            resp: exec_tx,
        }).await {
            Ok(_) => {},
            Err(_) => panic!("Should never happen")
        };
        let (abort_tx, abort_rx) = oneshot::channel();
        match tx.send(ExecutorMessage::Abort {
            id,
            resp: abort_tx,
        }).await {
            Ok(_) => {},
            Err(_) => panic!("Check abort task, probably receiver is dropped?")
        };
        match abort_rx.await {
            Ok(r) => {
                if !r {
                    panic!("ERROR! Aborting functionality doesn't work properly!")
                }
            },
            Err(_) => {
                panic!("ERROR! Aborting functionality doesn't work properly!")
            }
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(110)).await;
        let res = tokio::fs::read("testing.tmp").await;
        assert_eq!(res.is_err(), true);
    }
    
    #[tokio::test]
    async fn task_handles() {
        let task = create_boxed_long_task().await;
        let id = task.get_id();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
        });
        let (tx, _rx) = oneshot::channel();
        let handle = TaskHandle {
            inner_handle: handle,
            id,
            abort_tx: tx,
        };
        let mut task_handles = vec![handle];
        let index = get_handle_index(&mut task_handles, id);
        assert_eq!(index.is_some(), true);
        assert_eq!(index.unwrap(), 0);
    }

    #[tokio::test]
    async fn execute() {
        let (tx, rx) = mpsc::channel(32);
        let mut executor = create_executor();
        let inner_tx = tx.clone();
        let _handle = tokio::spawn(async move {
            executor.listen(rx, inner_tx).await;
        });
        let task = create_boxed_long_task().await;
        let (exec_tx, _exec_rx) = oneshot::channel();
        match tx.send(ExecutorMessage::Execute {
            task,
            resp: exec_tx,
        }).await {
            Ok(_) => {},
            Err(_) => panic!("Should never happen")
        };
        tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.25)).await;
        match std::path::Path::new("testing.temp").exists() {
            true => {
                cleanup().await;
            },
            false => {
                cleanup().await;
                panic!("ERROR! Executing task - that creates a temporary file - failed!")
            }
        };
    }
}