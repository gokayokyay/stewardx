mod errors;
mod cmd_async;
// mod frequency;
pub use cmd_async::CmdTask;
pub use errors::TaskError;
// pub use frequency::Frequency;

use crate::traits::Executable;

pub type ExecutableTask = dyn Executable + Send + Sync;
pub type BoxedTask = Box<ExecutableTask>;
// pub type BoxedTaskQueue = Arc<Mutex<VecDeque<BoxedTask>>>;
