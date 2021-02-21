use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::{messages::Message, models::OutputModel, traits::Executable};
use futures::Stream;
use uuid::Uuid;

pub type BoxedStream = Box<dyn Stream<Item = String> + Unpin + Send>;
pub type ExecutableTask = dyn Executable + Send + Sync;
pub type BoxedTask = Box<ExecutableTask>;
pub type OneShotMessageResponse<T> = tokio::sync::oneshot::Sender<T>;
pub type MPSCMessageSender = tokio::sync::mpsc::Sender<Message>;
pub type MPSCMessageReceiver = tokio::sync::mpsc::Receiver<Message>;

pub type DBMessageResponse<T> = tokio::sync::oneshot::Sender<Result<T, sqlx::Error>>;
pub type ExecutorSender = tokio::sync::mpsc::Sender<Message>;
pub type OutputSender = tokio::sync::broadcast::Sender<OutputModel>;
pub type OutputEmitter = tokio::sync::broadcast::Sender<OutputModel>;
pub type TaskWatcherSender = tokio::sync::mpsc::Sender<Message>;
pub type ServerReceiver = tokio::sync::mpsc::Receiver<Message>;
// pub type BoxedTaskQueue = Arc<Mutex<VecDeque<BoxedTask>>>;

#[macro_export]
macro_rules! now {
    () => {
        chrono::Utc::now().naive_utc()
    };
}
