use std::any;

use uuid::Uuid;

use crate::{models::TaskModel, traits::BoxedStream, types::OneShotMessageResponse};

// #[derive(Debug)]
pub enum ServerMessage {
    GetTasks {
        offset: Option<i64>,
        resp: OneShotMessageResponse<anyhow::Result<Vec<TaskModel>>>,
    },
    GetTask {
        task_id: Uuid,
        resp: OneShotMessageResponse<anyhow::Result<TaskModel>>
    },
    ExecuteTask {
        task_id: Uuid,
        resp: OneShotMessageResponse<bool>,
    },
    AbortTask {
        task_id: Uuid,
        resp: OneShotMessageResponse<bool>,
    },
    DeleteTask {
        task_id: Uuid,
        resp: OneShotMessageResponse<anyhow::Result<TaskModel>>,
    },
    CreateTask {
        task_name: String,
        frequency: String,
        task_type: String,
        task_props: String,
        resp: OneShotMessageResponse<anyhow::Result<TaskModel>>
    },
    GetActiveTasks {
        resp: OneShotMessageResponse<anyhow::Result<Vec<TaskModel>>>
    }
}

impl ServerMessage {
    pub fn get_type<'a>(&'a self) -> &'a str {
        return match self {
            ServerMessage::GetTasks { .. } => "GetTasks",
            ServerMessage::GetTask { .. } => "GetTask",
            ServerMessage::ExecuteTask { .. } => "ExecuteTask",
            ServerMessage::AbortTask { .. } => "AbortTask",
            ServerMessage::DeleteTask { .. } => "DeleteTask",
            ServerMessage::CreateTask { .. } => "CreateTask",
            ServerMessage::GetActiveTasks { .. } => "GetActiveTasks"
        };
    }
}
