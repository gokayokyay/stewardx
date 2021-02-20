use std::{net::SocketAddr, str::FromStr};

use tokio::sync::mpsc::Sender;
use hyper::{Body, Request, Response, Server as HyperServer, StatusCode};
use routerify::{Middleware, RequestInfo, Router, RouterService, ext::RequestExt};

mod messages;
pub use messages::ServerMessage;
use tracing::info;

pub struct Server {
    message_sender: Sender<ServerMessage>,
}

macro_rules! json {
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

async fn get_tasks(req: Request<Body>) -> Result<Response<Body>, hyper::http::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sender = req.data::<Sender<ServerMessage>>().unwrap();
    sender
        .send(ServerMessage::GET_TASKS {
            offset: None,
            resp: tx,
        })
        .await.unwrap();
    
    let result = rx.await.unwrap();
    let a = hyper::StatusCode::INTERNAL_SERVER_ERROR;
    match result {
        Ok(result) => json!(body: &result),
        Err(e) => {
            let obj = serde_json::json!({
                "error": e.to_string()
            });
            let obj = obj.as_str();
            let obj = obj.unwrap();
            json!(status: hyper::StatusCode::INTERNAL_SERVER_ERROR, body: &obj)
        }
    }
}

impl Server {
    pub fn new(message_sender: Sender<ServerMessage>) -> Self {
        Self { message_sender }
    }
    pub async fn listen(&self, host: String, port: i64) {
        let router = Router::builder()
            // Specify the state data which will be available to every route handlers,
            // error handler and middlewares.
            .data(self.message_sender.clone())
            // .middleware(Middleware::pre(logger))
            .get("/tasks", get_tasks)
            // .get("/users/:userId", user_handler)
            // .err_handler_with_info(error_handler)
            .build()
            .unwrap();
        let service = RouterService::new(router).unwrap();
        let addr = SocketAddr::from_str(format!("{}:{}", host, port).as_str()).expect("Invalid host or port.");
        let server = HyperServer::bind(&addr).serve(service);
        info!("Server started listening on {}", addr);
        if let Err(err) = server.await {
            eprintln!("Server error: {}", err);
        }
    }
}
