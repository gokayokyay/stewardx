use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::{
    models::{ExecutionReport, TaskError, TaskModel},
    types::OneShotMessageResponse,
};

#[derive(Debug)]
pub enum DBMessage {
    GET_TASK {
        id: Uuid,
        resp: DBMessageResponse<TaskModel>,
    },
    GET_TASKS {
        offset: Option<i64>,
        resp: DBMessageResponse<Vec<TaskModel>>,
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
        resp: DBMessageResponse<ExecutionReport>,
    },
    GET_EXECUTION_REPORTS {
        task_id: Uuid,
        offset: Option<i64>,
        resp: DBMessageResponse<Vec<ExecutionReport>>,
    },
}

impl DBMessage {
    pub fn get_type<'a>(&'a self) -> &'a str {
        match self {
            DBMessage::GET_TASK { .. } => {
                return "GET_TASK";
            }
            DBMessage::GET_TASKS { .. } => "GET_TASKS",
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
            DBMessage::GET_EXECUTION_REPORTS { .. } => "GET_EXECUTION_REPORTS",
        }
    }
}

pub type DBMessageResponse<T> = OneShotMessageResponse<Result<T, anyhow::Error>>;
