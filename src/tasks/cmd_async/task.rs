use async_trait::async_trait;
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Child,
};
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
    #[serde(skip)]
    child_handle: Arc<Mutex<Option<Child>>>,
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
        Self {
            id,
            command,
            child_handle: Arc::default(),
        }
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
    async fn exec(&mut self) -> Result<BoxedStream, TaskError> {
        let (prog, args) = CmdTask::parse_cmd(&self.id, &self.command).unwrap();
        let mut cmd = tokio::process::Command::new(prog);
        for arg in args {
            cmd.arg(arg);
        }
        cmd.stdout(Stdio::piped());
        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(_e) => {
                panic!();
                // return Err(TaskError::TaskExecution(self.get_id(), e.to_string()));
            }
        };
        self.child_handle = Arc::new(Mutex::new(Some(child)));
        let child = self.child_handle.clone();
        let mut child = child.lock().await;
        let child = child.as_mut().unwrap();
        let child_out = &mut child.stdout;
        let stdout = match child_out.take() {
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

    async fn abort(&mut self) -> bool {
        // println!("{:?}", self);
        let handle = &mut self.child_handle.lock().await;
        let handle = handle.as_mut().unwrap();
        match handle.kill().await {
            Ok(_) => return true,
            Err(_) => return false,
        };
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
