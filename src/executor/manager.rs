use tracing::{info, instrument};

use crate::{messages::Message, models::TaskError, traits::BoxedStream, types::BoxedTask};

pub struct Executor {}

impl Executor {
    #[instrument(skip(task), fields(task = %task.get_id()))]
    async fn execute(task: BoxedTask) -> Result<BoxedStream, TaskError> {
        info!("Executing task");
        let handle = task.exec().await;
        info!("Task execution finished.");
        return handle;
    }
    pub async fn listen(&self, mut rx: tokio::sync::mpsc::Receiver<Message>) {
        while let Some(message) = rx.recv().await {
            match message {
                Message::Executor_Execute { task, resp } => {
                    tokio::spawn(async move {
                        let result = Self::execute(task).await;
                        resp.send(result);
                    });
                }
                _ => panic!()
            }
        }
    }
}
