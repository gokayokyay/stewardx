mod messages;

use std::sync::Arc;
use tokio::sync::Mutex;

use crate::types::{DBSender, ExecutorSender, OutputSender, ReactorReceiver, ReactorSender, ServerReceiver, TaskWatcherSender};
pub use messages::ReactorMessage;

pub struct Reactor {
    pub db_sender: DBSender,
    pub executor_sender: ExecutorSender,
    pub task_watcher_sender: TaskWatcherSender,
    pub output_emitter: OutputSender,
    pub server_receiver: Arc<Mutex<ServerReceiver>>,
    pub inner_sender: ReactorSender
}

impl Reactor {
    pub async fn listen(&mut self, mut receiver: ReactorReceiver) {
        while let Some(message) = receiver.recv().await {
            let db_sender = self.db_sender.clone();
            let executor_sender = self.executor_sender.clone();
            let task_watcher_sender = self.task_watcher_sender.clone();
            let output_emitter = self.output_emitter.clone();
            let inner_sender = self.inner_sender.clone();
            tokio::spawn(async move {
                match message {
                    ReactorMessage::GetScheduledTasks { when, resp } => {
                        db_sender.send(crate::db::DBMessage::GetScheduledTasks {
                            when,
                            resp,
                        }).await;
                    }
                }
            });
        }
    }
}