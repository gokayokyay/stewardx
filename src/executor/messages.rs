use crate::{models::TaskError, traits::BoxedStream, types::BoxedTask};

pub enum ExecutorMessage {
    Execute {
        task: BoxedTask,
        resp: ExecutorResponse<Result<BoxedStream, TaskError>>,
    },
}

type ExecutorResponse<T> = tokio::sync::oneshot::Sender<T>;
