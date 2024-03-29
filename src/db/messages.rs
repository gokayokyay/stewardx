use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::{
    models::{ExecutionReport, TaskError, TaskModel},
    types::OneShotMessageResponse,
};
// TODO: Remove allow when we need unconstructed variants
#[allow(dead_code)]
#[derive(Debug)]
pub enum DBMessage {
    GetTask {
        id: Uuid,
        resp: DBMessageResponse<TaskModel>,
    },
    GetTasks {
        offset: Option<i64>,
        resp: DBMessageResponse<Vec<TaskModel>>,
    },
    CreateTask {
        task: TaskModel,
        resp: DBMessageResponse<TaskModel>,
    },
    GetScheduledTasks {
        when: NaiveDateTime,
        resp: DBMessageResponse<Vec<TaskModel>>,
    },
    UpdateNextExecution {
        id: Uuid,
        next_execution: Option<NaiveDateTime>,
        resp: DBMessageResponse<TaskModel>,
    },
    UpdateTask {
        task: TaskModel,
        resp: DBMessageResponse<TaskModel>,
    },
    DeleteTask {
        id: Uuid,
        resp: DBMessageResponse<TaskModel>,
    },
    CreateError {
        error: TaskError,
        resp: DBMessageResponse<TaskError>,
    },
    CreateExecutionReport {
        report: ExecutionReport,
        resp: DBMessageResponse<ExecutionReport>,
    },
    GetExecutionReportsForTask {
        task_id: Uuid,
        offset: Option<i64>,
        resp: DBMessageResponse<Vec<ExecutionReport>>,
    },
    DeleteExecutionReport {
        id: Uuid,
        resp: DBMessageResponse<ExecutionReport>,
    },
    DeleteExecutionReportsForTask {
        task_id: Uuid,
        resp: DBMessageResponse<Vec<ExecutionReport>>,
    },
    DeleteErrorsForTask {
        task_id: Uuid,
        resp: DBMessageResponse<Vec<TaskError>>,
    },
    GetExecutionReports {
        offset: Option<i64>,
        resp: DBMessageResponse<Vec<ExecutionReport>>,
    },
    GetExecutionReport {
        report_id: Uuid,
        resp: DBMessageResponse<ExecutionReport>,
    },
}

impl DBMessage {
    pub fn get_type<'a>(&'a self) -> &'a str {
        match self {
            DBMessage::GetTask { .. } => {
                return "GetTask";
            }
            DBMessage::GetTasks { .. } => "GetTasks",
            DBMessage::CreateTask { .. } => {
                return "CreateTask";
            }
            DBMessage::GetScheduledTasks { .. } => {
                return "GetScheduledTasks";
            }
            DBMessage::UpdateNextExecution { .. } => return "UpdateNextExecution",
            DBMessage::CreateError { .. } => return "CreateError",
            DBMessage::UpdateTask { .. } => return "UpdateTask",
            DBMessage::DeleteTask { .. } => return "DeleteTask",
            DBMessage::CreateExecutionReport { .. } => "CreateExecutionReport",
            DBMessage::GetExecutionReportsForTask { .. } => "GetExecutionReportsForTask",
            DBMessage::DeleteExecutionReport { .. } => "DeleteExecutionReport",
            DBMessage::DeleteExecutionReportsForTask { .. } => "DeleteExecutionReportsForTask",
            DBMessage::DeleteErrorsForTask { .. } => "DeleteErrorsForTask",
            DBMessage::GetExecutionReports { .. } => "GetExecutionReports",
            DBMessage::GetExecutionReport { .. } => "GetExecutionReport",
        }
    }
}

pub type DBMessageResponse<T> = OneShotMessageResponse<Result<T, anyhow::Error>>;
