use uuid::Uuid;

use crate::{
    models::TaskError,
    traits::BoxedStream,
    types::{BoxedTask, OneShotMessageResponse},
};

pub enum ExecutorMessage {
    Execute {
        task: BoxedTask,
        resp: OneShotMessageResponse<Result<BoxedStream, TaskError>>,
    },
    ExecutionFinished {
        id: Uuid,
        // resp: OneShotMessageResponse<bool>
    },
    Abort {
        id: Uuid,
        resp: OneShotMessageResponse<bool>
    }
}
