use serde::{Serialize, Deserialize};
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "c")]
pub enum DockerImageType {
    File(String),
    Image(String)
}