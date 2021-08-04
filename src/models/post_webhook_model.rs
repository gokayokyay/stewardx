use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::now;

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct PostWebhook {
    pub id: Uuid,
    pub task_id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub hook_url: String
}

impl PostWebhook {
    pub fn new(task_id: Uuid, hook_url: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            created_at: now!(),
            updated_at: now!(),
            hook_url
        }
    }
    pub fn new_raw(
        id: Uuid,
        task_id: Uuid,
        created_at: NaiveDateTime,
        updated_at: NaiveDateTime,
        hook_url: String
    ) -> Self {
        Self {
            id,
            task_id,
            created_at,
            updated_at,
            hook_url,
        }
    }
}
