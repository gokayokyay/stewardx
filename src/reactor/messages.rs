use chrono::NaiveDateTime;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::{models::{ExecutionReport, OutputModel, TaskError, TaskModel}, traits::BoxedStream, types::{BoxedTask, OneShotMessageResponse, OutputSender}};

pub enum ReactorMessage {
    GetScheduledTasks {
        when: NaiveDateTime,
        resp: ComposedResponse<Vec<TaskModel>>,
    },
    ExecuteScheduledTasks {
        when: NaiveDateTime
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
        should_update: bool
    },
    ServerGetTasks {
        offset: Option<i64>,
        resp: ComposedResponse<Vec<TaskModel>>
    },
    ServerExecuteTask {
        task_id: Uuid,
        resp: OneShotMessageResponse<bool>
    },
    ServerAbortTask {
        task_id: Uuid,
        resp: OneShotMessageResponse<bool>
    },
    ServerDeleteTask {
        task_id: Uuid,
        resp: OneShotMessageResponse<anyhow::Result<TaskModel>>
    },
    UpdateTaskExecution {
        task_id: Uuid
    }
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
            ReactorMessage::ServerGetTasks {.. } => "ServerGetTasks",
            ReactorMessage::ServerExecuteTask { .. } => "ServerExecuteTask",
            ReactorMessage::ServerAbortTask { .. } => "ServerAbortTask",
            ReactorMessage::ServerDeleteTask { .. } => "ServerDeleteTask",
            ReactorMessage::UpdateTaskExecution { .. } => "UpdateTaskExecution"
        }
    }
}