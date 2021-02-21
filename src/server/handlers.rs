use routerify::ext::RequestExt;
use tokio::sync::mpsc::Sender;
use hyper::{Body, Request, Response, Server as HyperServer, StatusCode};
use super::ServerMessage;

#[macro_export]
macro_rules! response_json {
    (body: $body:expr) => {
        hyper::Response::builder()
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string($body).unwrap().into())
    };
    (status: $status:expr, body: $body:expr) => {
        hyper::Response::builder()
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .status($status)
            .body(serde_json::to_string($body).unwrap().into())
    };
}

pub async fn get_tasks(req: Request<Body>) -> Result<Response<Body>, hyper::http::Error> {
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