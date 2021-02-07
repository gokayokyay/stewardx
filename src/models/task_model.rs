use chrono::{Duration, NaiveDateTime, Utc};
use tracing::info;
use uuid::Uuid;

use crate::{now, tasks::Frequency, types::BoxedTask};
#[derive(sqlx::FromRow, Debug, Clone)]
pub struct TaskModel {
    pub id: Uuid,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub task_type: String,
    pub serde_string: String,
    pub frequency: String,
    pub interval: Option<i64>,
    pub last_execution: Option<NaiveDateTime>,
    pub next_execution: Option<NaiveDateTime>,
    pub last_exec_succeeded: bool,
    pub exec_count: i64,
}

impl TaskModel {
    pub fn calc_next_execution(&self) -> Option<NaiveDateTime> {
        let frequency: Frequency = match <Frequency as std::str::FromStr>::from_str(&self.frequency)
        {
            Ok(f) => f,
            Err(_) => return None,
        };
        let next_execution = match frequency {
            Frequency::Every(_) => {
                let next = &frequency.get_next().unwrap();

                Some(next.naive_utc())
            }
            Frequency::AfterInterval => {
                let last_execution = match &self.last_execution {
                    Some(dt) => dt,
                    None => return None,
                };
                let interval = &self.interval.unwrap();
                let next_execution = *last_execution + Duration::seconds(*interval);
                Some(next_execution)
            }
        };
        return next_execution;
    }
    pub fn update_next_execution(&mut self) -> &mut Self {
        self.next_execution = self.calc_next_execution();
        info!(
            "Next execution date for task {:?} is {:?}",
            self.id, self.next_execution
        );
        self
    }
    pub fn from_boxed_task(task: BoxedTask, frequency: String) -> Self {
        let serde_string = task.to_string();
        let mut task = Self {
            id: task.get_id(),
            created_at: now!(),
            updated_at: now!(),
            task_type: task.get_type(),
            serde_string,
            frequency,
            interval: None,
            last_execution: None,
            next_execution: None,
            last_exec_succeeded: false,
            exec_count: 0,
        };
        task.next_execution = task.calc_next_execution();
        return task;
    }
}
