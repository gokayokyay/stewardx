use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::now;

#[derive(Debug, Clone)]
pub struct OutputModel {
    task_id: Uuid,
    timestamp: NaiveDateTime,
    output: String,
}

impl OutputModel {
    pub fn new(task_id: Uuid, output: String) -> Self {
        Self {
            task_id,
            timestamp: now!(),
            output,
        }
    }
    pub fn manual(task_id: Uuid, timestamp: NaiveDateTime, output: String) -> Self {
        Self {
            task_id,
            timestamp,
            output,
        }
    }
}
