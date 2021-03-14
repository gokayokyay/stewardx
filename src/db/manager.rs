use std::vec;

use chrono::{NaiveDate, NaiveDateTime, Utc};
use sqlx::{Encode, Pool, pool::PoolConnection, Postgres, Type};
use tokio::sync::mpsc::Receiver;
use tracing::{info, info_span, instrument};
// use futures::TryStreamExt;
use sqlx::Row;
use uuid::Uuid;

use crate::models::{ExecutionReport, TaskError, TaskModel};

use super::DBMessage;

pub struct DBManager {
    pool: Pool<Postgres>,
    rx: Receiver<DBMessage>,
}

macro_rules! now {
    () => {
        Utc::now().naive_utc()
    };
}

type Connection = PoolConnection<Postgres>;

impl DBManager {
    pub fn new(pool: Pool<Postgres>, rx: Receiver<DBMessage>) -> Self {
        Self { pool, rx }
    }
    #[instrument(
        name = "Adding a new task to database.",
        skip(conn, task),
        fields(
            task_id = %task.id,
            frequency = %task.frequency
        )
    )]
    pub async fn create_task(conn: &mut Connection, task: TaskModel) -> Result<TaskModel, sqlx::Error> {
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
            .fetch_one(conn).await;
        return row;
    }
    #[instrument(
        name = "Fetching a task from database.",
        skip(conn),
        fields(
            task_id = %id,
        )
    )]
    pub async fn get_task(conn: &mut Connection, id: Uuid) -> Result<TaskModel, sqlx::Error> {
        let task = sqlx::query_as!(TaskModel, "SELECT * FROM steward_tasks WHERE id = $1", id)
            .fetch_one(conn)
            .await;
        return task;
    }
    #[instrument(name = "Fetching tasks from database.", skip(conn), fields(offset))]
    pub async fn get_tasks(conn: &mut Connection, offset: Option<i64>) -> Result<Vec<TaskModel>, sqlx::Error> {
        let offset = offset.unwrap_or(0);
        let task = sqlx::query_as!(
            TaskModel,
            r#"
        SELECT * FROM steward_tasks
        ORDER BY created_at DESC
        LIMIT 100
        OFFSET $1"#,
            offset
        )
        .fetch_all(conn)
        .await;
        return task;
    }
    #[instrument(
        name = "Get scheduled tasks from database.",
        skip(conn),
        fields(
            scheduled_for = %when,
        )
    )]
    pub async fn get_scheduled_tasks(
        conn: &mut Connection,
        when: NaiveDateTime,
    ) -> Result<Vec<TaskModel>, sqlx::Error> {
        let rows = sqlx::query_as!(
            TaskModel,
            "SELECT * FROM steward_tasks WHERE next_execution <= $1",
            when
        )
        .fetch_all(conn)
        .await;
        rows
    }
    #[instrument(
        name = "Update task's next execution.",
        skip(conn),
        fields(
            task_id = %id,
            next_execution
        )
    )]
    pub async fn update_next_execution(
        conn: &mut Connection,
        id: Uuid,
        next_execution: Option<NaiveDateTime>,
    ) -> Result<TaskModel, sqlx::Error> {
        let row = sqlx::query_as!(TaskModel, "UPDATE steward_tasks SET next_execution = $1, updated_at = $2 WHERE id = $3 RETURNING *", next_execution, now!(), id)
            .fetch_one(conn).await;
        row
    }
    #[instrument(
        name = "Adding a new error to database.",
        skip(conn),
        fields(
            task_id = %error.task_id,
            error
        )
    )]
    pub async fn create_error(conn: &mut Connection, error: TaskError) -> Result<TaskError, sqlx::Error> {
        let row = sqlx::query_as!(
            TaskError,
            r#"
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
        )
        .fetch_one(conn)
        .await;
        return row;
    }
    #[instrument(
        name = "Updating task.",
        skip(conn),
        fields(
            task_id = %task.id,
        )
    )]
    pub async fn update_task(conn: &mut Connection, task: TaskModel) -> Result<TaskModel, sqlx::Error> {
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
            .fetch_one(conn).await;
        row
    }
    #[instrument(
        name = "Deleting task.",
        skip(conn),
        fields(
            task_id = %id,
        )
    )]
    pub async fn delete_task(conn: &mut Connection, id: Uuid) -> Result<TaskModel, sqlx::Error> {
        let row = sqlx::query_as!(
            TaskModel,
            "DELETE FROM steward_tasks WHERE id = $1 RETURNING *",
            id
        )
        .fetch_one(conn)
        .await;
        row
    }
    #[instrument(
        name = "Creating execution report.",
        skip(conn, report),
        fields(
            task_id = %report.task_id,
            successful = %report.successful
        )
    )]
    pub async fn create_execution_report(
        conn: &mut Connection,
        report: ExecutionReport,
    ) -> Result<ExecutionReport, sqlx::Error> {
        let output = report.output_as_string();
        let row = sqlx::query!(
            r#"
            INSERT INTO steward_task_execution_report
                ( id, task_id, created_at, successful, output )
                VALUES
                ( $1, $2, $3, $4, $5 )
                RETURNING *
            "#,
            report.id,
            report.task_id,
            report.created_at,
            report.successful,
            output
        )
        .fetch_one(conn)
        .await?;
        let result = ExecutionReport::new_string_output(
            row.id,
            row.task_id,
            row.created_at,
            row.successful,
            row.output,
        );
        Ok(result)
    }
    #[instrument(name = "Get execution reports.", skip(conn))]
    pub async fn get_execution_reports(
        conn: &mut Connection,
        task_id: Uuid,
        offset: Option<i64>,
    ) -> Result<Vec<ExecutionReport>, sqlx::Error> {
        let offset = offset.unwrap_or(0);
        let rows = sqlx::query!(
            r#"
            SELECT * FROM steward_task_execution_report
            WHERE task_id = $1
            ORDER BY created_at DESC
            LIMIT 100
            OFFSET $2
            "#,
            task_id,
            offset
        )
        .fetch_all(conn)
        .await?;
        let mut results = vec![];
        for row in rows {
            let result = ExecutionReport::new_string_output(
                row.id,
                row.task_id,
                row.created_at,
                row.successful,
                row.output,
            );
            results.push(result);
        }
        Ok(results)
    }
}

macro_rules! sqlx_to_anyhow {
    ($result: expr) => {
        match $result {
            Ok(r) => {
                Ok(r)
            },
            Err(e) => {
                Err(anyhow::anyhow!(e))
            }
        }
    };
}

impl DBManager {
    pub async fn listen(&mut self) {
        info!("DBManager started listening for messages.");
        while let Some(message) = self.rx.recv().await {
            info!("Got a {} message", message.get_type());
            let mut connection = self.pool.acquire().await.unwrap();
            tokio::spawn(async move {
                match message {
                    DBMessage::GetTask { id, resp } => {
                        let task = sqlx_to_anyhow!(Self::get_task(&mut connection, id).await);
                        resp.send(task);
                    }
                    DBMessage::GetTasks { offset, resp } => {
                        let tasks = sqlx_to_anyhow!(Self::get_tasks(&mut connection, offset).await);
                        resp.send(tasks);
                    }
                    DBMessage::CreateTask { task, resp } => {
                        let task = sqlx_to_anyhow!(Self::create_task(&mut connection, task).await);
                        resp.send(task);
                    }
                    DBMessage::GetScheduledTasks { when, resp } => {
                        let tasks = sqlx_to_anyhow!(Self::get_scheduled_tasks(&mut connection, when).await);
                        resp.send(tasks);
                    }
                    DBMessage::UpdateNextExecution {
                        id,
                        next_execution,
                        resp,
                    } => {
                        let task = sqlx_to_anyhow!(Self::update_next_execution(&mut connection, id, next_execution).await);
                        resp.send(task);
                    }
                    DBMessage::CreateError { error, resp } => {
                        let error = sqlx_to_anyhow!(Self::create_error(&mut connection, error).await);
                        resp.send(error);
                    }
                    DBMessage::UptadeTask { task, resp } => {
                        let task = sqlx_to_anyhow!(Self::update_task(&mut connection, task).await);
                        resp.send(task);
                    }
                    DBMessage::DeleteTask { id, resp } => {
                        let task = sqlx_to_anyhow!(Self::delete_task(&mut connection, id).await);
                        resp.send(task);
                    }
                    DBMessage::CreateExecutionReport { report, resp } => {
                        let report = sqlx_to_anyhow!(Self::create_execution_report(&mut connection, report).await);
                        resp.send(report);
                    }
                    DBMessage::GetExecutionReports {
                        task_id,
                        offset,
                        resp,
                    } => {
                        let reports = sqlx_to_anyhow!(Self::get_execution_reports(&mut connection, task_id, offset).await);
                        resp.send(reports);
                    }
                };
            });
        }
    }
}
