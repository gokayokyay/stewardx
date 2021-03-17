use crate::{db::DBMessage, executor::ExecutorMessage, models::OutputModel, reactor::ReactorMessage, server::ServerMessage, tasks::TaskWatcherMessage, traits::Executable};
use futures::{Stream, channel::mpsc::Receiver};

pub type BoxedStream = Box<dyn Stream<Item = String> + Unpin + Send>;
pub type ExecutableTask = dyn Executable + Send + Sync;
pub type BoxedTask = Box<ExecutableTask>;
pub type OneShotMessageResponse<T> = tokio::sync::oneshot::Sender<T>;

pub type DBSender = tokio::sync::mpsc::Sender<DBMessage>;
pub type ExecutorSender = tokio::sync::mpsc::Sender<ExecutorMessage>;
pub type OutputSender = tokio::sync::broadcast::Sender<OutputModel>;
pub type ReactorSender = tokio::sync::mpsc::Sender<ReactorMessage>;
pub type ReactorReceiver = tokio::sync::mpsc::Receiver<ReactorMessage>;
// pub type OutputEmitter = tokio::sync::broadcast::Sender<OutputModel>;
pub type TaskWatcherSender = tokio::sync::mpsc::Sender<TaskWatcherMessage>;
pub type ServerReceiver = tokio::sync::mpsc::Receiver<ServerMessage>;
// pub type BoxedTaskQueue = Arc<Mutex<VecDeque<BoxedTask>>>;

#[macro_export]
macro_rules! now {
    () => {
        chrono::Utc::now().naive_utc()
    };
}
