use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::{
    models::{ExecutionReport, OutputModel, TaskError, TaskModel},
    traits::BoxedStream,
    types::{BoxedTask, OneShotMessageResponse},
};

pub enum ReactorMessage {
    GetScheduledTasks {
        when: NaiveDateTime,
        resp: ComposedResponse<Vec<TaskModel>>,
    },
    ExecuteScheduledTasks {
        when: NaiveDateTime,
    },
    ExecuteTask {
        task: BoxedTask,
        // resp: ComposedResponse<BoxedStream>
    },
    CreateExecutionReport {
        report: ExecutionReport,
    },
    WatchExecution {
        task_id: Uuid,
        exec_process: Result<BoxedStream, TaskError>,
        // output_resp: OutputSender,
        // resp: OneShotMessageResponse<ExecutionReport>
    },
    OutputReceived {
        model: OutputModel,
        // resp: broadcast::Receiver<OutputModel>
    },
    ExecutionFinished {
        id: Uuid,
        should_update: bool,
    },
    CreateError {
        error: TaskError
    },
    ServerGetTasks {
        offset: Option<i64>,
        resp: ComposedResponse<Vec<TaskModel>>,
    },
    ServerGetTask {
        task_id: Uuid,
        resp: OneShotMessageResponse<anyhow::Result<TaskModel>>,
    },
    ServerCreateTask {
        task_name: String,
        frequency: String,
        task_type: String,
        task_props: serde_json::Value,
        resp: OneShotMessageResponse<anyhow::Result<TaskModel>>,
    },
    ServerExecuteTask {
        task_id: Uuid,
        resp: OneShotMessageResponse<bool>,
    },
    ServerAbortTask {
        task_id: Uuid,
        resp: OneShotMessageResponse<bool>,
    },
    ServerDeleteTask {
        task_id: Uuid,
        resp: OneShotMessageResponse<anyhow::Result<TaskModel>>,
    },
    ServerGetActiveTasks {
        resp: OneShotMessageResponse<anyhow::Result<Vec<TaskModel>>>,
    },
    ServerUpdateTask {
        task_id: Uuid,
        task_name: String,
        frequency: String,
        task_props: serde_json::Value,
        resp: OneShotMessageResponse<anyhow::Result<TaskModel>>,
    },
    ServerGetExecutionReportsForTask {
        task_id: Uuid,
        offset: Option<i64>,
        resp: OneShotMessageResponse<anyhow::Result<Vec<ExecutionReport>>>,
    },
    ServerGetExecutionReports {
        offset: Option<i64>,
        resp: OneShotMessageResponse<anyhow::Result<Vec<ExecutionReport>>>,
    },
    ServerGetExecutionReport {
        report_id: Uuid,
        resp: OneShotMessageResponse<anyhow::Result<ExecutionReport>>,
    },
    UpdateTaskExecution {
        task_id: Uuid,
    },
}

// type AnyResult<T> = Result<T, anyhow::Error>;
type ComposedResponse<T> = OneShotMessageResponse<anyhow::Result<T>>;

impl ReactorMessage {
    pub fn get_type<'a>(&'a self) -> &'a str {
        return match self {
            ReactorMessage::GetScheduledTasks { .. } => "GetScheduledTasks",
            ReactorMessage::ExecuteScheduledTasks { .. } => "ExecuteScheduledTasks",
            ReactorMessage::ExecuteTask { .. } => "ExecuteTask",
            ReactorMessage::CreateExecutionReport { .. } => "CreateExecutionReport",
            ReactorMessage::WatchExecution { .. } => "WatchExecution",
            ReactorMessage::OutputReceived { .. } => "OutputReceived",
            ReactorMessage::ExecutionFinished { .. } => "ExecutionFinished",
            ReactorMessage::ServerGetTasks { .. } => "ServerGetTasks",
            ReactorMessage::ServerCreateTask { .. } => "ServerCreateTask",
            ReactorMessage::ServerExecuteTask { .. } => "ServerExecuteTask",
            ReactorMessage::ServerAbortTask { .. } => "ServerAbortTask",
            ReactorMessage::ServerDeleteTask { .. } => "ServerDeleteTask",
            ReactorMessage::ServerGetActiveTasks { .. } => "ServerGetActiveTasks",
            ReactorMessage::UpdateTaskExecution { .. } => "UpdateTaskExecution",
            ReactorMessage::ServerGetTask { .. } => "ServerGetTask",
            ReactorMessage::ServerUpdateTask { .. } => "ServerUpdateTask",
            ReactorMessage::ServerGetExecutionReportsForTask { .. } => {
                "ServerGetExecutionReportsForTask"
            }
            ReactorMessage::ServerGetExecutionReports { .. } => "ServerGetExecutionReports",
            ReactorMessage::ServerGetExecutionReport { .. } => "ServerGetExecutionReport",
            ReactorMessage::CreateError { .. } => "CreateError",
        };
    }
}
