use tokio_stream::StreamExt;
use tracing::{info, instrument, warn};

use crate::models::{ExecutionReport, OutputModel};

use super::TaskWatcherMessage;

pub struct TaskWatcher {}

impl TaskWatcher {
    #[instrument(skip(self))]
    pub async fn listen(&self, mut receiver: tokio::sync::mpsc::Receiver<TaskWatcherMessage>) {
        info!("TaskWatcher started listening");
        while let Some(message) = receiver.recv().await {
            info!("Got a message {}", message.get_type());
            tokio::spawn(async move {
                match message {
                    TaskWatcherMessage::WatchExecution {
                        task_id,
                        exec_process,
                        output_resp,
                        resp,
                    } => match exec_process {
                        Ok(mut stream) => {
                            let mut output_vec = vec![];
                            while let Some(output) = stream.next().await {
                                match output_resp.send(OutputModel::new(task_id, output.clone())) {
                                    Err(_) => {
                                        warn!("Output received for task: {}, but nothing listens for it.", task_id);
                                    },
                                    _ => {}
                                };
                                output_vec.push(output);
                            }
                            let exec_report = ExecutionReport::new(task_id, true, output_vec);
                            // If this fails, output isn't really a problem...
                            let _ = resp.send(exec_report);
                        }
                        Err(e) => {
                            let report = ExecutionReport::new_string_output(uuid::Uuid::new_v4(), task_id, chrono::Utc::now().naive_utc(), false, e.to_string());
                            // Same reason as above
                            let _ = resp.send(report);
                        }
                    },
                }
            });
        }
    }
}
