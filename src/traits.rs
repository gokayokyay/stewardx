use crate::models::TaskError;
use std::fmt::Debug;

pub use super::types::BoxedStream;
use async_trait::async_trait;
use serde_json::Value;

#[async_trait]
pub trait Executable: Debug + ToString {
    async fn exec(&mut self) -> Result<BoxedStream, TaskError>;
    async fn abort(&mut self) -> bool;
    fn get_id(&self) -> uuid::Uuid;
    fn get_type(&self) -> String;
}

pub trait FromJson {
    fn from_json(json: String) -> Result<Self, TaskError>
    where
        Self: Sized;
}

pub trait GetSerdeFromProps {
    fn get_serde_from_props(value: Value) -> Result<String, anyhow::Error>;
    fn prop_not_found(prop: &str) -> anyhow::Error {
        return anyhow::anyhow!("Required property not specified: '{}'", prop);
    }
}