use hyper::Server as HyperServer;
use std::{net::SocketAddr, str::FromStr};
use tokio::sync::mpsc::Sender;

use routerify::{Router, RouterService};

mod handlers;
mod messages;
mod utils;
use handlers::{
    abort_task, create_task, delete_task, exec_task, exec_task_url, get_active_tasks, get_report,
    get_reports, get_reports_for_task, get_task, get_tasks, update_task, abort_task_url
};
pub use messages::ServerMessage;
use tracing::info;
use utils as ServerUtils;

mod app_router;
use app_router::app_router;

use crate::CONFIG;

pub struct Server {
    message_sender: Sender<ServerMessage>,
}

impl Server {
    pub fn new(message_sender: Sender<ServerMessage>) -> Self {
        Self { message_sender }
    }
    pub async fn listen(&self, host: String, port: i64) {
        let mut router = Router::builder()
            // Specify the state data which will be available to every route handlers,
            // error handler and middlewares.
            .data(self.message_sender.clone())
            // .middleware(Middleware::pre(logger))
            .post("/execute", exec_task)
            .post("/execute/:id", exec_task_url)
            .post("/abort", abort_task)
            .post("/abort/:id", abort_task_url)
            .get("/activetasks", get_active_tasks)
            .get("/task/:id/reports", get_reports_for_task)
            .get("/reports", get_reports)
            .get("/reports/:id", get_report);
            // .err_handler_with_info(error_handler)
        
        #[cfg(feature = "server-crud")]
        if CONFIG.get_features().get("server_crud").unwrap().eq(&true) {
            router = router
                .get("/tasks", get_tasks)
                .get("/tasks/:id", get_task)
                .post("/tasks", create_task)
                .post("/tasks/:id", update_task)
                .delete("/tasks", delete_task);
        }

        #[cfg(feature = "panel")]
        if CONFIG.get_features().get("panel").unwrap().eq(&true) {
            router = router.scope("/app", app_router());
        }
        let router = router.build().unwrap();
        let service = RouterService::new(router).unwrap();
        let addr = SocketAddr::from_str(format!("{}:{}", host, port).as_str())
            .expect("Invalid host or port.");
        let server = HyperServer::bind(&addr).serve(service);
        info!("Server started listening on {}", addr);
        if let Err(err) = server.await {
            eprintln!("Server error: {}", err);
        }
    }
}
