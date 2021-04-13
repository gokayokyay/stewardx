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
        format!(
            "TaskError for task: {}. Error type: {}, error message: {}",
            self.id, self.error_type, self.error_message
        )
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
    pub fn generic(task_id: Uuid, error: String) -> Self {
        return Self::new(
            task_id,
            "Generic".to_string(),
            format!("Generic error occurred: {}", error),
        );
    }
    pub fn invalid_cmd(task_id: Uuid, command: String) -> Self {
        return Self::new(
            task_id,
            "InvalidCmd".to_string(),
            format!("Invalid command specified. Command: {}", command),
        );
    }
    // pub fn docker_image_not_found(task_id: Uuid, image: String) -> Self {
    //     return Self::new(
    //         task_id,
    //         "DockerImageNotFound".to_string(),
    //         format!("Docker task's image not found: {}", image),
    //     );
    // }
    pub fn malformed_serde(task_id: Uuid, serde_string: String) -> Self {
        return Self::new(
            task_id,
            "MalformedSerde".to_string(),
            format!("Task's serde string is malformed. Serde: {}", serde_string),
        );
    }
}
