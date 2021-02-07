use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::now;
#[derive(sqlx::FromRow, Debug)]
pub struct TaskError {
    pub id: Uuid,
    pub task_id: Uuid,
    pub created_at: NaiveDateTime,
    pub error_type: String,
    pub error_message: String,
}

impl ToString for TaskError {
    fn to_string(&self) -> String {
        format!("TaskError for task: {}. Error type: {}, error message: {}", self.id, self.error_type, self.error_message)
    }
}

impl TaskError {
    pub fn new_raw(
        id: Uuid,
        task_id: Uuid,
        created_at: NaiveDateTime,
        error_type: String,
        error_message: String,
    ) -> Self {
        Self {
            id,
            task_id,
            created_at,
            error_type,
            error_message,
        }
    }
    pub fn new(task_id: Uuid, error_type: String, error_message: String) -> Self {
        return Self::new_raw(Uuid::new_v4(), task_id, now!(), error_type, error_message);
    }
    pub fn InvalidCmd(task_id: Uuid, command: String) -> Self {
        return Self::new(
            task_id,
            "InvalidCmd".to_string(),
            "Invalid command specified.".to_string(),
        );
    }
    pub fn MalformedSerde(task_id: Uuid, serde_string: String) -> Self {
        return Self::new(
            task_id,
            "MalformedSerde".to_string(),
            "Task's serde string is malformed.".to_string(),
        );
    }
}
