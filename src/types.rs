use std::{collections::VecDeque, sync::{Arc, Mutex}};

use futures::Stream;
use crate::{db::DBMessage, executor::ExecutorMessage, traits::Executable};

pub type BoxedStream = Box<dyn Stream<Item = String> + Unpin + Send>;
pub type ExecutableTask = dyn Executable + Send + Sync;
pub type BoxedTask = Box<ExecutableTask>;
pub type DBSender = tokio::sync::mpsc::Sender<DBMessage>;
pub type ExecutorSender = tokio::sync::mpsc::Sender<ExecutorMessage>;
// pub type BoxedTaskQueue = Arc<Mutex<VecDeque<BoxedTask>>>;
