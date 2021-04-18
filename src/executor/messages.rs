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
        resp: OneShotMessageResponse<bool>,
    },
    GetActiveTaskIDs {
        resp: OneShotMessageResponse<Vec<Uuid>>,
    },
}

impl ExecutorMessage {
    pub fn get_type<'a>(&'a self) -> &'a str {
        match self {
            ExecutorMessage::Execute { .. } => "Execute",
            ExecutorMessage::ExecutionFinished { .. } => "ExecutionFinished",
            ExecutorMessage::Abort { .. } => "Abort",
            ExecutorMessage::GetActiveTaskIDs { .. } => "GetActiveTaskIDs",
        }
    }
}
