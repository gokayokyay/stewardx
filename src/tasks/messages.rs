use uuid::Uuid;

use crate::{
    models::{ExecutionReport, TaskError},
    traits::BoxedStream,
    types::{OneShotMessageResponse, OutputSender},
};

pub enum TaskWatcherMessage {
    // TASK_EXECUTION_SUCCESSFUL {
    //     output_stream: BoxedStream,
    // },
    WATCH_EXECUTION {
        task_id: Uuid,
        exec_process: Result<BoxedStream, TaskError>,
        output_resp: OutputSender,
        resp: OneShotMessageResponse<ExecutionReport>,
    },
}
