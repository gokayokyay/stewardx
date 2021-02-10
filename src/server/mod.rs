use actix_web::{web, App, Error, HttpResponse, HttpServer, rt};
use tokio::sync::mpsc::Sender;

mod messages;
pub use messages::ServerMessage;

pub struct Server {
    message_sender: Sender<ServerMessage>,
}

async fn get_tasks(sender: web::Data<Sender<ServerMessage>>) -> HttpResponse {
    let (tx, rx) = tokio::sync::oneshot::channel();
    sender
        .send(ServerMessage::GET_TASKS {
            offset: None,
            resp: tx,
        })
        .await.unwrap();
    let result = rx.await.unwrap();
    match result {
        Ok(result) => HttpResponse::Ok().json(result),
        Err(e) => {
            let obj = serde_json::json!({
                "error": e.to_string()
            });
            HttpResponse::InternalServerError().json(obj)
        }
    }
}

impl Server {
    pub fn new(message_sender: Sender<ServerMessage>) -> Self {
        Self { message_sender }
    }
    pub fn listen(&self, host: String, port: i64) {
        let mut sys = rt::System::new("stewardx");
        let sender = self.message_sender.clone();
        let server = HttpServer::new(move || {
            App::new()
                .data(sender.clone())
                .service(web::resource("/").to(|| async { "Hello steward" }))
                .service(web::resource("/tasks").route(web::get().to(get_tasks)))
        })
        .bind(format!("{}:{}", host, port))
        .unwrap()
        .run();
        sys.block_on(server);
    }
}
