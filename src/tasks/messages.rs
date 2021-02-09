use uuid::Uuid;

use crate::{models::{ExecutionReport, TaskError}, traits::BoxedStream, types::OutputSender};

pub enum TaskWatcherMessage {
    // TASK_EXECUTION_SUCCESSFUL {
    //     output_stream: BoxedStream,
    // },
    WATCH_EXECUTION {
        task_id: Uuid,
        exec_process: Result<BoxedStream, TaskError>,
        output_resp: OutputSender,
        resp: WatcherResponse<ExecutionReport>
    }
}

type WatcherResponse<T> = tokio::sync::oneshot::Sender<T>;
