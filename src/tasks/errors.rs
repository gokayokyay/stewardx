// pub use serde::{Deserialize, Serialize};
// #[derive(Debug, Serialize, Deserialize)]
// pub enum TaskError {
//     TimeOut(uuid::Uuid),
//     InvalidCmd(uuid::Uuid, String),
//     MalformedSerde(uuid::Uuid, String),
//     InvalidTaskType(uuid::Uuid, String),
//     TaskExecution(uuid::Uuid, String),
// }

// impl TaskError {
//     pub fn get_type(&self) -> String {
//         match self {
//             TaskError::TimeOut(..) => {
//                 return String::from("Timeout");
//             }
//             TaskError::InvalidCmd(..) => {
//                 return String::from("InvalidCmd");
//             }
//             TaskError::MalformedSerde(..) => {
//                 return String::from("MalformedSerde");
//             }
//             TaskError::InvalidTaskType(..) => {
//                 return String::from("InvalidTaskType");
//             }
//             TaskError::TaskExecution(..) => {
//                 return String::from("TaskExecution");
//             }
//         }
//     }
//     pub fn get_message(&self) -> String {
//         match self {
//             TaskError::TimeOut(id) => {
//                 format!("Timeout occured for task: {}", id)
//             }
//             TaskError::InvalidCmd(_, message) => {
//                 return message.to_string();
//             }
//             TaskError::MalformedSerde(_, message) => message.to_string(),
//             TaskError::InvalidTaskType(_, message) => message.to_string(),
//             TaskError::TaskExecution(_, message) => message.to_string(),
//         }
//     }
//     pub fn get_id(&self) -> uuid::Uuid {
//         match *self {
//             TaskError::TimeOut(id) => id,
//             TaskError::InvalidCmd(id, _) => id,
//             TaskError::MalformedSerde(id, _) => id,
//             TaskError::InvalidTaskType(id, _) => id,
//             TaskError::TaskExecution(id, _) => id,
//         }
//     }
// }

// impl std::fmt::Display for TaskError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match *self {
//             TaskError::TimeOut(x) => {
//                 tracing::error_span!(
//                     "Timeout occurred when executing task.",
//                     id = %x
//                 );
//                 f.write_fmt(format_args!("Timeout occurred, id: {}", x))
//             }
//             TaskError::InvalidCmd(id, ref cmd) => {
//                 tracing::error_span!(
//                     "Invalid command specified.",
//                     id = %id,
//                     %cmd
//                 );
//                 f.write_fmt(format_args!(
//                     "Invalid command specified\n ID: {}\nCommand: {}",
//                     id, cmd
//                 ))
//             }
//             TaskError::MalformedSerde(id, ref serde_string) => {
//                 tracing::error_span!(
//                     "Task's serde string is malformed.",
//                     %id,
//                     serde = %serde_string
//                 );
//                 f.write_fmt(format_args!(
//                     "Invalid command specified\n ID: {}\nSerde String: {}",
//                     id, serde_string
//                 ))
//             }
//             TaskError::InvalidTaskType(id, ref task_type) => {
//                 tracing::error_span!(
//                     "Task type is invalid.",
//                     id = %id,
//                     %task_type
//                 );
//                 f.write_fmt(format_args!(
//                     "Invalid task type specified\n ID: {}\nTask Type: {}",
//                     id, task_type
//                 ))
//             }
//             TaskError::TaskExecution(id, ref error) => {
//                 tracing::error_span!(
//                     "Error while executing task.",
//                     %id,
//                     %error
//                 );
//                 f.write_fmt(format_args!(
//                     "Error while executing task.\n ID: {}\nError: {}",
//                     id, error
//                 ))
//             }
//         }
//     }
// }
