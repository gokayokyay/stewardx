use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::{models::{TaskError, TaskModel}, traits::BoxedStream, types::{BoxedTask, OneShotMessageResponse}};

pub enum ReactorMessage {
    GetScheduledTasks {
        when: NaiveDateTime,
        resp: ComposedResponse<Vec<TaskModel>>,
    }
}

type AnyResult<T> = Result<T, anyhow::Error>;
type ComposedResponse<T> = OneShotMessageResponse<AnyResult<T>>;
