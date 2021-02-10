use tokio_stream::StreamExt;

use crate::models::{ExecutionReport, OutputModel};

use super::TaskWatcherMessage;

pub struct TaskWatcher {}

impl TaskWatcher {
    pub async fn listen(&self, mut receiver: tokio::sync::mpsc::Receiver<TaskWatcherMessage>) {
        while let Some(message) = receiver.recv().await {
            tokio::spawn(async move {
                match message {
                    TaskWatcherMessage::WATCH_EXECUTION {
                        task_id,
                        exec_process,
                        output_resp,
                        resp,
                    } => match exec_process {
                        Ok(mut stream) => {
                            let mut output_vec = vec![];
                            while let Some(output) = stream.next().await {
                                output_resp.send(OutputModel::new(task_id, output.clone()));
                                output_vec.push(output);
                            }
                            let exec_report = ExecutionReport::new(task_id, true, output_vec);
                            resp.send(exec_report);
                        }
                        Err(e) => {}
                    },
                }
            });
        }
    }
}