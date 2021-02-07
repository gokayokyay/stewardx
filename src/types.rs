use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
};

use crate::{db::DBMessage, executor::ExecutorMessage, models::OutputModel, traits::Executable};
use futures::Stream;
use uuid::Uuid;

pub type BoxedStream = Box<dyn Stream<Item = String> + Unpin + Send>;
pub type ExecutableTask = dyn Executable + Send + Sync;
pub type BoxedTask = Box<ExecutableTask>;
pub type DBSender = tokio::sync::mpsc::Sender<DBMessage>;
pub type ExecutorSender = tokio::sync::mpsc::Sender<ExecutorMessage>;
pub type OutputSender = tokio::sync::broadcast::Sender<OutputModel>;
// pub type BoxedTaskQueue = Arc<Mutex<VecDeque<BoxedTask>>>;

#[macro_export]
macro_rules! now {
    () => {
        chrono::Utc::now().naive_utc()
    };
}
