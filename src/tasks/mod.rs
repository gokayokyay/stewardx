mod cmd_async;
mod errors;
mod frequency;
mod messages;
mod watcher;
pub use cmd_async::CmdTask;

#[cfg(feature = "docker")]
mod docker_async;
#[cfg(feature = "docker")]
pub use docker_async::{DockerImageType, DockerTask};

// pub use errors::TaskError;
pub use frequency::Frequency;
pub use messages::TaskWatcherMessage;
pub use watcher::TaskWatcher;

#[macro_export]
macro_rules! ModelToTask {
    ($r: ident => $m:expr) => {
        use crate::tasks::CmdTask;
        #[cfg(feature = "docker")]
        use crate::tasks::DockerTask;

        use crate::traits::FromJson;
        use crate::types::BoxedTask;
        let task: Option<BoxedTask> = match $r.task_type.as_str() {
            "CmdTask" => Some(Box::new(
                CmdTask::from_json($r.serde_string.clone()).unwrap(),
            )),
            #[cfg(feature = "docker")]
            "DockerTask" => Some(Box::new(
                DockerTask::from_json($r.serde_string.clone()).unwrap(),
            )),
            // "HttpTask" => Some(Box::new(HttpTask::from_json($r.serde_string).unwrap())),
            _ => None,
        };
        $m = task;
    };
}
