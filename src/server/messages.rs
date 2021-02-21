use uuid::Uuid;

use crate::{models::TaskModel, traits::BoxedStream, types::OneShotMessageResponse};

// #[derive(Debug)]
pub enum ServerMessage {
    GET_TASKS {
        offset: Option<i64>,
        resp: OneShotMessageResponse<Result<Vec<TaskModel>, sqlx::Error>>,
    },
    EXECUTE_TASK {
        task_id: Uuid,
        resp: OneShotMessageResponse<Result<BoxedStream, sqlx::Error>>,
    }
}

impl ServerMessage {
    pub fn get_type<'a>(&'a self) -> &'a str {
        return match self {
            ServerMessage::GET_TASKS { .. } => "GET_TASKS",
            ServerMessage::EXECUTE_TASK { .. } => "EXECUTE_TASK"
        };
    }
}
