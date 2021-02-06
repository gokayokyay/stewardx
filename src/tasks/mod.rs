mod errors;
mod cmd_async;
mod frequency;
pub use cmd_async::CmdTask;
// pub use errors::TaskError;
pub use frequency::Frequency;


#[macro_export]
macro_rules! ModelToTask {
    ($r: ident => $m:expr) => {
        use crate::types::BoxedTask;
        use crate::tasks::CmdTask;
        use crate::traits::FromJson;
        let task: Option<BoxedTask> = match $r.task_type.as_str() {
            "CmdTask" => Some(Box::new(CmdTask::from_json($r.serde_string.clone()).unwrap())),
            // "HttpTask" => Some(Box::new(HttpTask::from_json($r.serde_string).unwrap())),
            _ => None,
        };
        $m = task;
    };
}