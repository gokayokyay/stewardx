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
        resp: ComposedResponse<ExecutionReport>
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
        successful: bool
    }
}

type AnyResult<T> = Result<T, anyhow::Error>;
type ComposedResponse<T> = OneShotMessageResponse<AnyResult<T>>;

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
        }
    }
}