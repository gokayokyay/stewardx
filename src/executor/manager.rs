use tracing::{info, instrument};

use crate::{models::TaskError, traits::BoxedStream, types::BoxedTask};

use super::ExecutorMessage;

pub struct Executor {}

impl Executor {
    #[instrument(skip(task), fields(task = %task.get_id()))]
    async fn execute(task: BoxedTask) -> Result<BoxedStream, TaskError> {
        info!("Executing task");
        let handle = task.exec().await;
        info!("Task execution finished.");
        return handle;
    }
    pub async fn listen(&self, mut rx: tokio::sync::mpsc::Receiver<ExecutorMessage>) {
        while let Some(message) = rx.recv().await {
            match message {
                ExecutorMessage::Execute { task, resp } => {
                    let task = tokio::spawn(async move {
                        let result = Self::execute(task).await;
                        resp.send(result);
                    });
                    // Reactor panics because of dropped response.
                    // To truly abort the task we need to store its resp too
                    // task.abort();
                }
            }
        }
    }
}
