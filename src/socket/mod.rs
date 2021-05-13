mod socket_listener;
pub mod unix_utils;
use std::convert::Infallible;

use hyper::{Body, Request, Response, Server, service::{make_service_fn, service_fn}};

async fn handle_request(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    // We look for the host header for now, might change in future
    // This way we can just curl into it like: curl --unix-socket /tmp/stewardx.sock http:/stop
    let possible_hosts = ["stop"];
    let headers = req.headers();
    let host = headers.get("host");
    let no_host = Ok(Response::new(format!("Possible commands: {}", possible_hosts.join(", ")).into()));
    match host {
        Some(host) => {
            match host.to_str() {
                Ok(o) => {
                    match o {
                        "stop" => {
                            tokio::spawn(async {
                                tokio::time::sleep(tokio::time::Duration::from_secs_f32(0.25)).await;
                                unix_utils::exit();
                            });
                            return Ok(Response::new("Goodbye!".into()));
                        }
                        _ => {
                            return no_host;
                        }
                    }
                }
                Err(_) => {
                    return no_host;
                }
            }
        }
        None => {
            return no_host;
        }
    };
}

pub struct SocketManager {}

impl SocketManager {
    pub async fn listen() {
        let listener = socket_listener::UDSAccept {
            inner: tokio::net::UnixListener::bind(unix_utils::get_socket_path().to_str().unwrap()).unwrap(),
        };
        let make_svc = make_service_fn(|_conn| async {
            // service_fn converts our function into a `Service`
            Ok::<_, Infallible>(service_fn(handle_request))
        });
    
        let server = Server::builder(listener).serve(make_svc);
        let _result = server.await;
    }
}
