use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_stream::wrappers::LinesStream;
use uuid::Uuid;

use crate::{
    models::TaskError,
    traits::{BoxedStream, Executable, FromJson},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CmdTask {
    pub id: Uuid,
    pub command: Box<String>,
}

impl CmdTask {
    pub fn new(id: Uuid, command: Box<String>) -> Self {
        let id_literal = id.to_string();
        let cmd_literal = command.to_string();
        tracing::info_span!(
            "Creating a new CmdTask",
            id = %id_literal,
            command = %cmd_literal
        );
        Self { id, command }
    }
    pub fn parse_cmd(id: &uuid::Uuid, command: &str) -> Result<(String, Vec<String>), TaskError> {
        let mut s = command.split(" ");
        let err = TaskError::InvalidCmd(*id, command.to_string());
        let prog = match s.next() {
            Some(x) => x,
            None => return Err(err),
        };
        let a: Vec<&str> = s.collect();
        let b: Vec<String> = a.into_iter().map(|x| return x.to_string()).collect();
        return Ok((prog.to_string(), b));
    }
    pub fn get_task_type() -> String {
        String::from("CmdTask")
    }
}

impl ToString for CmdTask {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
#[async_trait]
impl Executable for CmdTask {
    async fn exec(&self) -> Result<BoxedStream, TaskError> {
        let (prog, args) = CmdTask::parse_cmd(&self.id, &self.command).unwrap();
        let mut cmd = tokio::process::Command::new(prog);
        for arg in args {
            cmd.arg(arg);
        }
        cmd.stdout(Stdio::piped());
        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(_e) => {
                panic!();
                // return Err(TaskError::TaskExecution(self.get_id(), e.to_string()));
            }
        };
        let stdout = match child.stdout.take() {
            Some(s) => s,
            None => {
                panic!();
            }
        };
        let reader = BufReader::new(stdout);
        let lines = reader.lines();
        let stream = LinesStream::new(lines);
        let stream = stream.map(|l| l.unwrap());
        Ok(Box::new(stream))
    }

    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_type(&self) -> String {
        Self::get_task_type()
    }
}

impl FromJson for CmdTask {
    fn from_json(json: String) -> Result<Self, TaskError> {
        match serde_json::from_str::<Self>(&json) {
            Ok(task) => {
                return Ok(task);
            }
            Err(_) => {
                match serde_json::from_str::<serde_json::Value>(&json) {
                    Ok(value) => {
                        let real_task = serde_json::from_str::<Self>(value.as_str().unwrap());
                        match real_task {
                            Ok(task) => {
                                return Ok(task);
                            }
                            Err(_) => {}
                        }
                    }
                    _ => {}
                };
            }
        }
        return Err(TaskError::MalformedSerde(uuid::Uuid::default(), json));
    }
}
