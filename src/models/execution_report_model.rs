use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::now;

#[derive(Debug)]
pub struct ExecutionReport {
    pub id: Uuid,
    pub task_id: Uuid,
    pub created_at: NaiveDateTime,
    pub successful: bool,
    pub output: Vec<String>,
}

impl ExecutionReport {
    pub fn new(task_id: Uuid, successful: bool, output: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            created_at: now!(),
            successful,
            output,
        }
    }
    pub fn new_raw(
        id: Uuid,
        task_id: Uuid,
        created_at: NaiveDateTime,
        successful: bool,
        output: Vec<String>,
    ) -> Self {
        Self {
            id,
            task_id,
            created_at,
            successful,
            output,
        }
    }
    pub fn output_as_string(&self) -> String {
        self.output.join("\n").to_string()
    }
    pub fn new_string_output(
        id: Uuid,
        task_id: Uuid,
        created_at: NaiveDateTime,
        successful: bool,
        output: String,
    ) -> Self {
        let output = output.split("\n").map(|l| l.to_string()).collect();
        Self::new_raw(id, task_id, created_at, successful, output)
    }
}
