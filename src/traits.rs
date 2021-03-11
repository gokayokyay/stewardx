use crate::models::TaskError;
use std::fmt::Debug;

pub use super::types::BoxedStream;
use async_trait::async_trait;

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
