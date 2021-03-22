use futures::StreamExt;
// use tokio_stream::StreamExt;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use async_trait::async_trait;
use shiplift::{ContainerOptions, PullOptions, tty::TtyChunk};

use crate::{GLOBAL_DOCKER, traits::{BoxedStream, Executable, FromJson}};
use crate::models::TaskError;

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerTask {
    pub id: Uuid,
    pub image: String,
    pub env: Vec<String>,
    container_id: String,
}

impl DockerTask {
    pub fn new(id: Uuid, image: String, env: Vec<String>) -> Self {
        Self { id, image, env, container_id: String::default() }
    }
}

impl ToString for DockerTask {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}

#[async_trait]
impl Executable for DockerTask {
    async fn exec(&mut self) -> Result<BoxedStream, TaskError> {
        let docker = &GLOBAL_DOCKER;
        let mut stream = docker
            .images()
            .pull(&PullOptions::builder().image(&self.image).build());

        while let Some(pull_result) = stream.next().await {
            match pull_result {
                Err(e) => return Err(TaskError::Generic(self.id, e.to_string())),
                _ => ()
            }
        }
        let options = ContainerOptions::builder(&self.image).env(&self.env).build();
        let info = docker.containers().create(&options).await.unwrap();
        let id = info.id;
        self.container_id = id.clone();
        let tty_multiplexer = docker.containers().get(&id).attach().await.unwrap();
        let (reader, _writer) = tty_multiplexer.split();
        let stream = reader.map(|result| {
            match result {
                Ok(chunk) => {
                    let output = match chunk {
                        TtyChunk::StdOut(bytes) => std::str::from_utf8(&bytes).unwrap().to_string(),
                        TtyChunk::StdErr(bytes) => std::str::from_utf8(&bytes).unwrap().to_string(),
                        TtyChunk::StdIn(_) => unreachable!()
                    };
                    return output;
                }
                Err(e) => {
                    return e.to_string()
                }
            }
        });

        return Ok(Box::new(stream));
    }

    async fn abort(&mut self) -> bool {
        todo!()
    }

    fn get_id(&self) -> Uuid {
        self.id
    }

    fn get_type(&self) -> String {
        String::from("DockerTask")
    }
}

impl FromJson for DockerTask {
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
