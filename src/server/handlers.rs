use futures::TryStreamExt;
use routerify::ext::RequestExt;
use tokio::sync::mpsc::Sender;
use hyper::{Body, Request, Response, Server as HyperServer, StatusCode, body::{Bytes, HttpBody}};
use tokio_stream::StreamExt;
use super::ServerMessage;

#[macro_export]
macro_rules! response_json {
    (body: $body:expr) => {
        match hyper::Response::builder()
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string($body).unwrap().into())
            {
                Ok(x) => Ok(x),
                Err(e) => Err(anyhow::anyhow!(e.to_string()))
            }
    };
    (status: $status:expr, body: $body:expr) => {
        match hyper::Response::builder()
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .status($status)
            .body(serde_json::to_string($body).unwrap().into())
            {
                Ok(x) => Ok(x),
                Err(e) => Err(anyhow::anyhow!(e.to_string()))
            }
    };
}

pub async fn get_tasks(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sender = req.data::<Sender<ServerMessage>>().unwrap();
    sender
        .send(ServerMessage::GET_TASKS {
            offset: None,
            resp: tx,
        })
        .await;
    
    let result = rx.await.unwrap();
    match result {
        Ok(result) => response_json!(body: &result),
        Err(e) => {
            let obj = serde_json::json!({
                "error": e.to_string()
            });
            let obj = obj.as_str();
            let obj = obj.unwrap();
            response_json!(status: hyper::StatusCode::INTERNAL_SERVER_ERROR, body: &obj)
        }
    }
}


/// End point to execute a task
/// "task_id" parameter is required
pub async fn exec_task(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    // let (tx, rx) = tokio::sync::oneshot::channel();
    let sender = req.data::<Sender<ServerMessage>>().unwrap();
    let task_id = req.into_body().data().await.unwrap();
    println!("{:?}", task_id);
    // sender.send(ServerMessage::EXECUTE_TASK {
    //     task_id: (),
    //     resp: tx,
    // }).await;
    panic!("AAAA")
}