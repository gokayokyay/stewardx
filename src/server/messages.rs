use crate::{models::TaskModel, types::OneShotMessageResponse};

#[derive(Debug)]
pub enum ServerMessage {
    GET_TASKS {
        offset: Option<i64>,
        resp: OneShotMessageResponse<Result<Vec<TaskModel>, sqlx::Error>>,
    },
}

impl ServerMessage {
    pub fn get_type<'a>(&'a self) -> &'a str {
        return match self {
            ServerMessage::GET_TASKS { .. } => "GET_TASK",
        };
    }
}
