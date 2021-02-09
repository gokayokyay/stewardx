mod cmd_async;
mod errors;
mod frequency;
mod watcher;
mod messages;
pub use cmd_async::CmdTask;
// pub use errors::TaskError;
pub use frequency::Frequency;
pub use watcher::TaskWatcher;
pub use messages::TaskWatcherMessage;

#[macro_export]
macro_rules! ModelToTask {
    ($r: ident => $m:expr) => {
        use crate::tasks::CmdTask;
        use crate::traits::FromJson;
        use crate::types::BoxedTask;
        let task: Option<BoxedTask> = match $r.task_type.as_str() {
            "CmdTask" => Some(Box::new(
                CmdTask::from_json($r.serde_string.clone()).unwrap(),
            )),
            // "HttpTask" => Some(Box::new(HttpTask::from_json($r.serde_string).unwrap())),
            _ => None,
        };
        $m = task;
    };
}
