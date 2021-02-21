use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::{models::{ExecutionReport, TaskError, TaskModel}, traits::BoxedStream, types::{BoxedTask, DBMessageResponse, OneShotMessageResponse, OutputSender}};

pub enum Message {
    TaskWatcher_WATCH_EXECUTION {
        task_id: Uuid,
        exec_process: Result<BoxedStream, TaskError>,
        output_resp: OutputSender,
        resp: OneShotMessageResponse<ExecutionReport>,
    },
    Server_GET_TASKS {
        offset: Option<i64>,
        resp: OneShotMessageResponse<Result<Vec<TaskModel>, sqlx::Error>>,
    },
    Server_EXECUTE_TASK {
        task_id: Uuid,
        resp: OneShotMessageResponse<Result<BoxedStream, sqlx::Error>>,
    },
    Executor_Execute {
        task: BoxedTask,
        resp: OneShotMessageResponse<Result<BoxedStream, TaskError>>,
    },
    DB_GET_TASK {
        id: Uuid,
        resp: DBMessageResponse<TaskModel>,
    },
    DB_GET_TASKS {
        offset: Option<i64>,
        resp: DBMessageResponse<Vec<TaskModel>>,
    },
    DB_CREATE_TASK {
        task: TaskModel,
        resp: DBMessageResponse<TaskModel>,
    },
    DB_GET_SCHEDULED_TASKS {
        when: NaiveDateTime,
        resp: DBMessageResponse<Vec<TaskModel>>,
    },
    DB_UPDATE_NEXT_EXECUTION {
        id: Uuid,
        next_execution: Option<NaiveDateTime>,
        resp: DBMessageResponse<TaskModel>,
    },
    DB_UPTADE_TASK {
        task: TaskModel,
        resp: DBMessageResponse<TaskModel>,
    },
    DB_DELETE_TASK {
        id: Uuid,
        resp: DBMessageResponse<TaskModel>,
    },
    DB_CREATE_ERROR {
        error: TaskError,
        resp: DBMessageResponse<TaskError>,
    },
    DB_CREATE_EXECUTION_REPORT {
        report: ExecutionReport,
        resp: DBMessageResponse<ExecutionReport>,
    },
    DB_GET_EXECUTION_REPORTS {
        task_id: Uuid,
        offset: Option<i64>,
        resp: DBMessageResponse<Vec<ExecutionReport>>,
    },
}

impl Message {
    pub fn get_type<'a>(&'a self) -> &'a str {
        return match self {
            Message::TaskWatcher_WATCH_EXECUTION { .. } => "TaskWatcher_WATCH_EXECUTION",
            Message::Server_GET_TASKS { .. } => "Server_GET_TASKS",
            Message::Server_EXECUTE_TASK { .. } => "Server_EXECUTE_TASK",
            Message::Executor_Execute { .. } => "Executor_Execute",
            Message::DB_GET_TASK { .. } => "DB_GET_TASK",
            Message::DB_GET_TASKS { .. } => "DB_GET_TASKS",
            Message::DB_CREATE_TASK { .. } => "DB_CREATE_TASK",
            Message::DB_GET_SCHEDULED_TASKS { .. } => "DB_GET_SCHEDULED_TASKS",
            Message::DB_UPDATE_NEXT_EXECUTION { .. } => "DB_UPDATE_NEXT_EXECUTION",
            Message::DB_UPTADE_TASK { .. } => "DB_UPTADE_TASK",
            Message::DB_DELETE_TASK { .. } => "DB_DELETE_TASK",
            Message::DB_CREATE_ERROR { .. } => "DB_CREATE_ERROR",
            Message::DB_CREATE_EXECUTION_REPORT { .. } => "DB_CREATE_EXECUTION_REPORT",
            Message::DB_GET_EXECUTION_REPORTS { .. } => "DB_GET_EXECUTION_REPORTS"
        };
    }
}
