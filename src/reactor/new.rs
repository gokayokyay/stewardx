use std::sync::Arc;

use tokio::sync::Mutex;
use tracing::{info, warn};

use crate::{messages::Message, types::{MPSCMessageReceiver, MPSCMessageSender, OutputEmitter, ServerReceiver}};
use super::{now, ModelToTask};

// pub struct ReactorEX {
//     pub db_sender: MPSCMessageSender,
//     pub executor_sender: MPSCMessageSender,
//     pub task_watcher_sender: MPSCMessageSender,
//     pub output_emitter: OutputEmitter,
//     pub server_receiver: Arc<Mutex<ServerReceiver>>,
// }


pub struct Reactor {
    pub receiver: MPSCMessageReceiver,
    pub sender_copy: MPSCMessageSender,
    pub db_sender: MPSCMessageSender,
    pub executor_sender: MPSCMessageSender,
    pub task_watcher_sender: MPSCMessageSender,
}

impl Reactor {
    pub async fn listen(&mut self) {
        let sender_copy = self.sender_copy.clone();
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            let when = now!();
            info!(
                "Sending GET_SCHEDULED_TASKS message to DBManager time: {}",
                when.to_string()
            );
            let (tx, rx) = tokio::sync::oneshot::channel();
            let second_sender = sender_copy.clone();
            sender_copy.send(Message::DB_GET_SCHEDULED_TASKS {
                when,
                resp: tx
            }).await;
            let tasks = rx.await.unwrap();
            match tasks {
                Ok(tasks) => {
                    for task in tasks {
                        let boxed_task;
                        ModelToTask!(task => boxed_task);
                        second_sender.send(Message::Executor_Execute {
                            task: boxed_task.unwrap(),
                            resp: (),
                        }).await;
                    }
                }
                Err(_) => {}
            }
            // self.db_sender
            //     .send(Message::DB_GET_SCHEDULED_TASKS { when, resp: tx })
            //     .await;
            // return rx.await.unwrap();
            
        });
        while let Some(message) = self.receiver.recv().await {
            let db_sender = self.db_sender.clone();
            let sender_copy = self.sender_copy.clone();
            if message.get_type().starts_with("DB") {
                tokio::spawn(async move {
                    Self::handle_db_messages(message, db_sender, sender_copy).await;
                });
            }
            else if message.get_type().starts_with("TaskWatcher") {
                tokio::spawn(async move {
                    Self::handle_task_watcher_messages(message).await;
                });
            } 
        }
    }
    pub async fn handle_db_messages(message: Message, sender: MPSCMessageSender, sender_copy: MPSCMessageSender) {
        
    }
    pub async fn handle_task_watcher_messages(message: Message) {
        
    }
}