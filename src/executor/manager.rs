use tokio::task::JoinHandle;
use tracing::{info, instrument};
use uuid::Uuid;

use crate::{models::TaskError, traits::BoxedStream, types::{BoxedTask, OneShotMessageResponse}};

use super::ExecutorMessage;

pub struct TaskHandle {
    inner_handle: JoinHandle<()>,
    id: Uuid,
    // task_resp: OneShotMessageResponse<Result<BoxedStream, TaskError>>,
    abort_tx: OneShotMessageResponse<bool>
}

pub struct Executor {
    pub task_handles: Vec<TaskHandle>
}

impl Executor {
    #[instrument(skip(task), fields(task = %task.get_id()))]
    async fn execute(task: &mut BoxedTask) -> Result<BoxedStream, TaskError> {
        info!("Executing task");
        let handle = task.exec().await;
        info!("Task execution finished.");
        return handle;
    }
    pub async fn listen(&mut self, mut rx: tokio::sync::mpsc::Receiver<ExecutorMessage>, tx: tokio::sync::mpsc::Sender<ExecutorMessage>) {
        while let Some(message) = rx.recv().await {
            let inner_tx = tx.clone();
            match message {
                ExecutorMessage::Execute { mut task, resp } => {
                    let id = task.get_id();
                    let (abort_tx, abort_rx) = tokio::sync::oneshot::channel::<bool>();
                    info!("Executing task: {}", id);
                    let handle = tokio::spawn(async move {
                        let result = Self::execute(&mut task).await;
                        resp.send(result);
                        // task.abort().await;
                        // We can listen for the abort oneshot

                        if let Ok(_) = abort_rx.await {
                            info!("Aborting task {}", id);
                            println!("Aborting task {}", id);
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
                },
                ExecutorMessage::ExecutionFinished { id } => {
                    info!("Execution of task: {} is finished", id);
                    if let Some(index) = self.get_handle_index(id) {
                        let _val = self.task_handles.remove(index);
                    }
                }
                ExecutorMessage::Abort { id, resp } => {
                    if self.abort_task(id, resp).await {
                        inner_tx.send(ExecutorMessage::ExecutionFinished {
                            id,
                        }).await;
                    }
                }
            }
        }
    }
    pub async fn abort_task(&mut self, task_id: Uuid, resp: tokio::sync::oneshot::Sender<bool>) -> bool {
        if let Some(index) = self.get_handle_index(task_id) {
            let val = self.task_handles.remove(index);
            val.abort_tx.send(true);
            // val.inner_handle.abort();
            resp.send(true);
            return true;
        }
        resp.send(false);
        return false;
    }
    fn get_handle_index(&mut self, task_id: Uuid) -> Option<usize> {
        let predicate = |t: &TaskHandle| { t.id == task_id };
        let mut i: usize = 0;
        while i != self.task_handles.len() {
            if predicate(&mut self.task_handles[i]) {
                return Some(i);
            } else {
                i += 1;
            }
        }
        return None;
    }
}
