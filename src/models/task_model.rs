use chrono::NaiveDateTime;
use serde::Serialize;
use uuid::Uuid;

use crate::{now, tasks::Frequency};
#[derive(sqlx::FromRow, Debug, Clone, Serialize)]
pub struct TaskModel {
    pub id: Uuid,
    pub task_name: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
    pub task_type: String,
    pub serde_string: String,
    pub frequency: String,
    pub last_execution: Option<NaiveDateTime>,
    pub next_execution: Option<NaiveDateTime>,
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
            // Frequency::AfterInterval => {
            //     let last_execution = match &self.last_execution {
            //         Some(dt) => dt,
            //         None => return None,
            //     };
            //     let interval = &self.interval.unwrap();
            //     let next_execution = *last_execution + Duration::seconds(*interval);
            //     Some(next_execution)
            // }
            Frequency::Hook => None,
        };
        return next_execution;
    }
    // TODO: This fn is used in tests but can be used in app too, when we use it please remove the line below
    #[allow(dead_code)]
    pub fn from_boxed_task(task: crate::types::BoxedTask, name: String, frequency: String) -> Self {
        let serde_string = task.to_string();
        let mut task = Self {
            id: task.get_id(),
            task_name: name,
            created_at: now!(),
            updated_at: now!(),
            task_type: task.get_type(),
            serde_string,
            frequency,
            last_execution: None,
            next_execution: None,
            exec_count: 0,
        };
        task.next_execution = task.calc_next_execution();
        return task;
    }
    pub fn new(
        id: Option<Uuid>,
        task_name: String,
        task_type: String,
        serde_string: String,
        frequency: String,
    ) -> Self {
        let id = match id {
            Some(id) => id,
            None => Uuid::new_v4(),
        };
        let mut task = Self {
            id,
            task_name,
            created_at: now!(),
            updated_at: now!(),
            task_type,
            serde_string,
            frequency,
            last_execution: None,
            next_execution: None,
            exec_count: 0,
        };
        task.next_execution = task.calc_next_execution();
        return task;
    }
    pub fn get_serde_from_props(
        id: Uuid,
        task_type: String,
        task_props: serde_json::Value,
    ) -> Result<String, anyhow::Error> {
        use crate::traits::GetSerdeFromProps;
        match task_type.as_str() {
            #[cfg(feature = "cmd")]
            "CmdTask" => {
                return crate::tasks::CmdTask::get_serde_from_props(id, task_props);
            }
            #[cfg(feature = "docker")]
            "DockerTask" => {
                return crate::tasks::DockerTask::get_serde_from_props(id, task_props);
            }
            _ => return Err(anyhow::anyhow!("Unknown task type {}", task_type)),
        };
    }
}
