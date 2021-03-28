use futures::StreamExt;
use tokio::fs::File;
// use tokio_stream::StreamExt;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use shiplift::{BuildOptions, ContainerOptions, PullOptions, tty::TtyChunk};
use tempfile::{Builder, TempDir};

use crate::{GLOBAL_DOCKER, traits::{BoxedStream, Executable, FromJson}};
use crate::models::TaskError;

use super::DockerImageType;

#[derive(Debug, Serialize, Deserialize)]
pub struct DockerTask {
    pub id: Uuid,
    pub image: DockerImageType,
    pub env: Vec<String>,
    container_id: String,
}

impl DockerTask {
    pub fn new(id: Uuid, image: DockerImageType, env: Vec<String>) -> Self {
        Self { id, image, env, container_id: String::default() }
    }
    pub fn create_temp_dir(named: String) -> Result<TempDir, anyhow::Error> {
        let tmp_dir = Builder::new().prefix(&named).tempdir()?;
        Ok(tmp_dir)
    }
}

impl ToString for DockerTask {
    fn to_string(&self) -> String {
        // println!("{:?}", self);
        serde_json::to_string(&self).unwrap()
    }
}

#[async_trait]
impl Executable for DockerTask {
    async fn exec(&mut self) -> Result<BoxedStream, TaskError> {
        let docker = &GLOBAL_DOCKER;
        // let mut stream = docker.images().build(BuildOptions::builder())
        let image = match &self.image {
            DockerImageType::File(contents) => {
                let temp_dir = Self::create_temp_dir(self.id.to_string());
                let temp_dir = match temp_dir {
                    Ok(d) => d,
                    Err(e) => {
                        panic!(e.to_string());
                    }
                };
                let path = temp_dir.path().join("Dockerfile");
                tokio::fs::write(path.clone(), contents).await;
                let image = format!("stewardx:{}", self.id.to_simple().encode_lower(&mut Uuid::encode_buffer()));
                let mut file = docker.images().build(&shiplift::BuildOptions::builder(temp_dir.path().to_str().unwrap().to_string()).tag(image.clone()).build());
                while let Some(build_result) = file.next().await {
                    match build_result {
                        Ok(output) => println!("{:?}", output),
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            panic!("{}", e.to_string());
                        },
                    }
                }
                image
            }
            DockerImageType::Image(image) => {
                let mut stream = docker
                    .images()
                    .pull(&PullOptions::builder().image(image).build());

                while let Some(pull_result) = stream.next().await {
                    println!("{:?}", pull_result);
                }
                image
            }.to_string()
        };
        let options = ContainerOptions::builder(&image).env(&self.env).build();
        let info = docker.containers().create(&options).await.unwrap();
        let id = info.id;
        self.container_id = id.clone();
        let container = docker.containers().get(&id);
        if let Err(a) = container.start().await {
            return Err(TaskError::Generic(self.id, a.to_string()));
        }

        let tty_multiplexer = container.attach().await.unwrap();
        let (reader, _writer) = tty_multiplexer.split();
        let stream = reader.map(|result| {
            println!("{:?}", result);
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
            Err(e) => {
                println!("json: {}", &json);
                println!("{}", e.to_string());
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
