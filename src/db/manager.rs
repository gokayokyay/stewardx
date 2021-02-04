use chrono::{NaiveDateTime, Utc, NaiveDate};
use sqlx::{Pool, Postgres};
use tokio::sync::mpsc::Receiver;

use super::DBMessage;

pub struct DBManager {
    pool: Pool<Postgres>,
    rx: Receiver<DBMessage>
}

impl DBManager {
    pub fn new(pool: Pool<Postgres>, rx: Receiver<DBMessage>) -> Self { Self { pool, rx } }
    // pub async fn create_task(&self, task)
}
