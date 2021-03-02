use tokio::sync::mpsc::Sender;
use std::{net::SocketAddr, str::FromStr};
use hyper::{Server as HyperServer};

use routerify::{Middleware, RequestInfo, Router, RouterService, ext::RequestExt};

mod messages;
mod handlers;
use handlers::{get_tasks, exec_task};
pub use messages::ServerMessage;
use tracing::info;

pub struct Server {
    message_sender: Sender<ServerMessage>,
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
            .post("/tasks/execute", exec_task)
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
