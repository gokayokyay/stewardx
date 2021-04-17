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

use crate::{models::TaskError, traits::{BoxedStream, Executable, FromJson, GetSerdeFromProps}};

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
        let err = TaskError::invalid_cmd(*id, command.to_string());
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
        return Err(TaskError::malformed_serde(uuid::Uuid::default(), json));
    }
}

impl GetSerdeFromProps for CmdTask {
    fn get_serde_from_props(id: Uuid, value: serde_json::Value) -> Result<String, anyhow::Error> {
        let command = value["command"].as_str();
        if command.is_none() {
            return Err(Self::prop_not_found("command"));
        }
        let cmd_task = crate::tasks::CmdTask::new(id, Box::new(command.unwrap().to_string()));
        return Ok(cmd_task.to_string());
    }
}


#[cfg(test)]
mod tests {
    use crate::tasks::CmdTask;
    use super::*;

    async fn create_long_task() -> CmdTask {
        let sleep_and_print_and_create_file_command = r#"
            sleep 0.2s
            echo "Hey hey hey"
        "#;
        let _file = tokio::fs::write("temp_script.sh", sleep_and_print_and_create_file_command).await;
        let task = CmdTask::new(Uuid::new_v4(), Box::new("/bin/bash temp_script.sh".into()));
        task
    }
    async fn cleanup() {
        match tokio::fs::remove_file(std::path::Path::new("testing_cmd.temp")).await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e.to_string());
                println!("Couldn't cleanup after test, please locate and remove \"testing_cmd.temp\"");
            }
        };
        match tokio::fs::remove_file("temp_script.sh").await {
            Ok(_) => {}
            Err(e) => {
                eprintln!("{}", e.to_string());
                println!("Couldn't cleanup after test, please locate and remove \"temp_script.sh\"");
            }
        };
    }
    #[tokio::test]
    async fn can_execute() {
        let mut task = create_long_task().await;
        let result = task.exec().await;
        let expected_output = format!("Hey hey hey");
        let mut output_stream = result.unwrap();
        let output = output_stream.next().await;
        assert_eq!(output.unwrap(), expected_output);
        cleanup().await;
    }
    #[tokio::test]
    async fn can_abort() {
        let mut task = create_long_task().await;
        let mut e = task.exec().await.unwrap();
        let a = task.abort().await;
        assert_eq!(a, true);
        let none_output = e.next().await;
        assert_eq!(none_output, None);
        cleanup().await;
    }
}