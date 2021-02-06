use std::vec;

use chrono::{NaiveDateTime, Utc, NaiveDate};
use sqlx::{Encode, Pool, Postgres, Type};
use tokio::sync::mpsc::Receiver;
use tracing::{info, info_span, instrument};
// use futures::TryStreamExt;
use sqlx::Row;
use uuid::Uuid;

use crate::models::{TaskError, TaskModel};

use super::DBMessage;

pub struct DBManager {
    pool: Pool<Postgres>,
    rx: Receiver<DBMessage>
}

macro_rules! now {
    () => {
        Utc::now().naive_utc()
    };
}

impl DBManager {
    pub fn new(pool: Pool<Postgres>, rx: Receiver<DBMessage>) -> Self { Self { pool, rx } }
    #[instrument(
        name = "Adding a new task to database.",
        skip(self, task),
        fields(
            task_id = %task.id,
            frequency = %task.frequency
        )
    )]
    pub async fn create_task(&self, task: TaskModel) -> Result<TaskModel, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let row = sqlx::query_as!(TaskModel, r#"
            INSERT INTO steward_tasks
                ( id, created_at, updated_at, task_type, last_execution, next_execution, serde_string, last_exec_succeeded, frequency, interval, exec_count )
                VALUES
                ( $1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11 )
                RETURNING *
            "#,
            task.id,
            task.created_at,
            task.updated_at,
            task.task_type,
            task.last_execution,
            task.next_execution,
            task.serde_string,
            task.last_exec_succeeded,
            task.frequency,
            task.interval,
            task.exec_count
            )
            .fetch_one(&mut conn).await;
        return row;
    }
    #[instrument(
        name = "Fetching a task from database.",
        skip(self),
        fields(
            task_id = %id,
        )
    )]
    pub async fn get_task(&self, id: Uuid) -> Result<TaskModel, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let task = sqlx::query_as!(TaskModel, "SELECT * FROM steward_tasks WHERE id = $1", id)
            .fetch_one(&mut conn).await;
        return task;
    }
    #[instrument(
        name = "Get scheduled tasks from database.",
        skip(self),
        fields(
            scheduled_for = %when,
        )
    )]
    pub async fn get_scheduled_tasks(&self, when: NaiveDateTime) ->Result<Vec<TaskModel>, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let rows = sqlx::query_as!(TaskModel, "SELECT * FROM steward_tasks WHERE next_execution <= $1", when)
            .fetch_all(&mut conn).await;
        rows
    }
    #[instrument(
        name = "Update task's next execution.",
        skip(self),
        fields(
            task_id = %id,
            next_execution
        )
    )]
    pub async fn update_next_execution(&self, id: Uuid, next_execution: Option<NaiveDateTime>) -> Result<TaskModel, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let row = sqlx::query_as!(TaskModel, "UPDATE steward_tasks SET next_execution = $1, updated_at = $2 WHERE id = $3 RETURNING *", next_execution, now!(), id)
            .fetch_one(&mut conn).await;
        row
    }
    #[instrument(
        name = "Adding a new error to database.",
        skip(self),
        fields(
            task_id = %error.task_id,
            error
        )
    )]
    pub async fn create_error(&self, error: TaskError) -> Result<TaskError, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let row = sqlx::query_as!(TaskError, r#"
            INSERT INTO steward_task_errors
                ( id, task_id, created_at, error_type, error_message )
                VALUES
                ( $1, $2, $3, $4, $5 )
                RETURNING *
            "#,
            error.id,
            error.task_id,
            error.created_at,
            error.error_type,
            error.error_message
        ).fetch_one(&mut conn).await;
        return row;
    }
    #[instrument(
        name = "Updating task.",
        skip(self),
        fields(
            task_id = %task.id,
        )
    )]
    pub async fn update_task(&self, task: TaskModel) -> Result<TaskModel, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let row = sqlx::query_as!(TaskModel,
            "UPDATE steward_tasks SET updated_at = $2, serde_string = $3, frequency = $4, interval = $5, last_execution = $6, next_execution = $7, last_exec_succeeded = $8, exec_count = $9 WHERE id = $1 RETURNING *",
            task.id,
            now!(),
            task.serde_string,
            task.frequency,
            task.interval,
            task.last_execution,
            task.next_execution,
            task.last_exec_succeeded,
            task.exec_count
            )
            .fetch_one(&mut conn).await;
        row
    }
    #[instrument(
        name = "Deleting task.",
        skip(self),
        fields(
            task_id = %id,
        )
    )]
    pub async fn delete_task(&self, id: Uuid) -> Result<TaskModel, sqlx::Error> {
        let mut conn = self.pool.acquire().await.unwrap();
        let row = sqlx::query_as!(TaskModel,
            "DELETE FROM steward_tasks WHERE id = $1 RETURNING *",
            id
            )
            .fetch_one(&mut conn).await;
        row
    }
}

impl DBManager {
    pub async fn listen(&mut self) {
        info!("DBManager started listening for messages.");
        while let Some(message) = self.rx.recv().await {
            info!("Got a {} message", message.get_type());
            match message {
                DBMessage::GET_TASK { id, resp } => {
                    let task = self.get_task(id).await;
                    resp.send(task);
                }
                DBMessage::CREATE_TASK { task, resp } => {
                    let task = self.create_task(task).await;
                    resp.send(task);
                }
                DBMessage::GET_SCHEDULED_TASKS { when, resp } => {
                    let tasks = self.get_scheduled_tasks(when).await;
                    resp.send(tasks);
                }
                DBMessage::UPDATE_NEXT_EXECUTION { id, next_execution, resp } => {
                    let task = self.update_next_execution(id, next_execution).await;
                    resp.send(task);
                }
                DBMessage::CREATE_ERROR { error, resp } => {
                    let error = self.create_error(error).await;
                    resp.send(error);
                }
                DBMessage::UPTADE_TASK { task, resp } => {
                    let task = self.update_task(task).await;
                    resp.send(task);
                }
                DBMessage::DELETE_TASK { id, resp } => {
                    let task = self.delete_task(id).await;
                    resp.send(task);
                }
            };
        }
    }
}
