use std::fmt::Debug;
use crate::models::TaskError;

pub use super::types::BoxedStream;
use async_trait::async_trait;

#[async_trait]
pub trait Executable: Debug + ToString {
    async fn exec(&self) -> Result<BoxedStream, TaskError>;
    fn get_id(&self) -> uuid::Uuid;
    fn get_type(&self) -> String;
}

pub trait FromJson {
    fn from_json(json: String) -> Result<Self, TaskError>
    where
        Self: Sized;
}
