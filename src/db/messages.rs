use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::models::{ExecutionReport, TaskError, TaskModel};

#[derive(Debug)]
pub enum DBMessage {
    GET_TASK {
        id: Uuid,
        resp: DBMessageResponse<TaskModel>,
    },
    CREATE_TASK {
        task: TaskModel,
        resp: DBMessageResponse<TaskModel>,
    },
    GET_SCHEDULED_TASKS {
        when: NaiveDateTime,
        resp: DBMessageResponse<Vec<TaskModel>>,
    },
    UPDATE_NEXT_EXECUTION {
        id: Uuid,
        next_execution: Option<NaiveDateTime>,
        resp: DBMessageResponse<TaskModel>,
    },
    UPTADE_TASK {
        task: TaskModel,
        resp: DBMessageResponse<TaskModel>,
    },
    DELETE_TASK {
        id: Uuid,
        resp: DBMessageResponse<TaskModel>,
    },
    CREATE_ERROR {
        error: TaskError,
        resp: DBMessageResponse<TaskError>,
    },
    CREATE_EXECUTION_REPORT {
        report: ExecutionReport,
        resp: DBMessageResponse<ExecutionReport>
    }
}

impl DBMessage {
    pub fn get_type<'a>(&'a self) -> &'a str {
        match self {
            DBMessage::GET_TASK { .. } => {
                return "GET_TASK";
            }
            DBMessage::CREATE_TASK { .. } => {
                return "CREATE_TASK";
            }
            DBMessage::GET_SCHEDULED_TASKS { .. } => {
                return "GET_SCHEDULED_TASKS";
            }
            DBMessage::UPDATE_NEXT_EXECUTION { .. } => return "UPDATE_NEXT_EXECUTION",
            DBMessage::CREATE_ERROR { .. } => return "CREATE_ERROR",
            DBMessage::UPTADE_TASK { .. } => return "UPDATE_TASK",
            DBMessage::DELETE_TASK { .. } => return "DELETE_TASK",
            DBMessage::CREATE_EXECUTION_REPORT { .. } => "CREATE_EXECUTION_REPORT",
        }
    }
}

pub type DBMessageResponse<T> = tokio::sync::oneshot::Sender<Result<T, sqlx::Error>>;
