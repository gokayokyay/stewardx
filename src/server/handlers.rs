use std::str::FromStr;

use super::ServerMessage;
use hyper::{body::HttpBody, Body, Request, Response};
use routerify::{ext::RequestExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;
use tracing::error;
use uuid::Uuid;

#[macro_export]
macro_rules! response_json {
    (body: $body:expr) => {
        match hyper::Response::builder()
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .body(serde_json::to_string($body).unwrap().into())
        {
            Ok(x) => Ok(x),
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
        }
    };
    (status: $status:expr, body: $body:expr) => {
        match hyper::Response::builder()
            .header(hyper::header::CONTENT_TYPE, "application/json")
            .status($status)
            .body(serde_json::to_string($body).unwrap().into())
        {
            Ok(x) => Ok(x),
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
        }
    };
}

macro_rules! reactor_failed {
    ($send: expr, $msg: tt) => {
        match $send {
            Ok(_) => {},
            Err(e) => {
                error!("Reactor didnt receive the {} message! Either it's failed or didn't start.", $msg);
                error!("{}", e.to_string());
                return Err(anyhow::anyhow!(serde_json::json!({
                    "error": "Reactor isn't awake."
                })));
            }
        }
    }
}

macro_rules! empty_malformed_body {
    () => {
        return {
            let obj = serde_json::json!({
                "error": "Body is malformed/empty, please try again."
            });
            let obj = obj.to_string();
            response_json!(status: hyper::StatusCode::BAD_REQUEST, body: &obj)
        };
    }
}

pub async fn get_tasks(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sender = req.data::<Sender<ServerMessage>>().unwrap();
    reactor_failed!(sender
        .send(ServerMessage::GetTasks {
            offset: None,
            resp: tx,
        })
        .await, "GetTasks");
    let result = rx.await.unwrap();
    match result {
        Ok(result) => response_json!(body: &result),
        Err(e) => {
            let obj = serde_json::json!({
                "error": e.to_string()
            });
            let obj = obj.to_string();
            response_json!(status: hyper::StatusCode::INTERNAL_SERVER_ERROR, body: &obj)
        }
    }
}

pub async fn get_task(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sender = req.data::<Sender<ServerMessage>>().unwrap();
    let task_id = req.param("id").unwrap();
    reactor_failed!(sender
        .send(ServerMessage::GetTask {
            task_id: Uuid::from_str(task_id).unwrap(),
            resp: tx,
        })
        .await, "GetTask");

    let result = rx.await.unwrap();
    match result {
        Ok(result) => response_json!(body: &result),
        Err(e) => {
            let obj = serde_json::json!({
                "error": e.to_string()
            });
            let obj = obj.to_string();
            response_json!(status: hyper::StatusCode::INTERNAL_SERVER_ERROR, body: &obj)
        }
    }
}

/// End point to execute a task
/// "task_id" parameter is required
pub async fn exec_task(mut req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    #[derive(Debug, Serialize, Deserialize)]
    struct RequestBody {
        task_id: Uuid,
    }
    let (tx, rx) = tokio::sync::oneshot::channel();
    let body = req.body_mut();
    if let Some(Ok(body)) = body.data().await {
        if let Ok(json_value) =
            serde_json::from_slice(&body) as Result<RequestBody, serde_json::Error>
        {
            let sender = req.data::<Sender<ServerMessage>>().unwrap();
            let task_id = json_value.task_id;
            reactor_failed!(sender
                .send(ServerMessage::ExecuteTask { task_id, resp: tx })
                .await, "ExecuteTask");
            if let Ok(_) = rx.await {
                return response_json!(
                    body: &serde_json::json!({
                         "status": "success"
                     })
                );
            } else {
                return response_json!(
                    status: hyper::StatusCode::NOT_FOUND,
                    body: &serde_json::json!({
                         "status": "error"
                     })
                );
            }
        }
    };
    empty_malformed_body!()
}

pub async fn exec_task_url(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sender = req.data::<Sender<ServerMessage>>().unwrap();
    let task_id = match req.param("id") {
        Some(id) => Uuid::from_str(id).unwrap(),
        None => {
            let obj = serde_json::json!({
                "error": "Missing url parameter: id."
            });
            let obj = obj.to_string();
            println!("obj {:?}", obj);

            return response_json!(status: hyper::StatusCode::BAD_REQUEST, body: &obj);
        }
    };
    reactor_failed!(sender
        .send(ServerMessage::ExecuteTask { task_id, resp: tx })
        .await, "ExecuteTask");
    if let Ok(_) = rx.await {
        return response_json!(
            body: &serde_json::json!({
                 "status": "success"
             })
        );
    } else {
        return response_json!(
            status: hyper::StatusCode::NOT_FOUND,
            body: &serde_json::json!({
                 "status": "error"
             })
        );
    }
}

pub async fn abort_task(mut req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    #[derive(Debug, Serialize, Deserialize)]
    struct RequestBody {
        task_id: Uuid,
    }
    let (tx, rx) = tokio::sync::oneshot::channel();
    let body = req.body_mut();
    if let Some(Ok(body)) = body.data().await {
        if let Ok(json_value) =
            serde_json::from_slice(&body) as Result<RequestBody, serde_json::Error>
        {
            let sender = req.data::<Sender<ServerMessage>>().unwrap();
            let task_id = json_value.task_id;
            reactor_failed!(sender
                .send(ServerMessage::AbortTask { task_id, resp: tx })
                .await, "AbortTask");
            if let Ok(result) = rx.await {
                let status;
                if result {
                    status = "success"
                } else {
                    status = "error"
                }
                return response_json!(body: &serde_json::json!({ "status": status }));
            } else {
                println!("Abort RX - server waiting failed");
                return response_json!(
                    status: hyper::StatusCode::NOT_FOUND,
                    body: &serde_json::json!({
                         "status": "error"
                     })
                );
            }
        }
    }
    empty_malformed_body!()
}

pub async fn abort_task_url(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sender = req.data::<Sender<ServerMessage>>().unwrap();
    let task_id = match req.param("id") {
        Some(id) => Uuid::from_str(id).unwrap(),
        None => {
            let obj = serde_json::json!({
                "error": "Missing url parameter: id."
            });
            let obj = obj.to_string();
            println!("obj {:?}", obj);

            return response_json!(status: hyper::StatusCode::BAD_REQUEST, body: &obj);
        }
    };
    reactor_failed!(sender
        .send(ServerMessage::AbortTask { task_id, resp: tx })
        .await, "AbortTask");
    if let Ok(_) = rx.await {
        return response_json!(
            body: &serde_json::json!({
                 "status": "success"
             })
        );
    } else {
        return response_json!(
            status: hyper::StatusCode::NOT_FOUND,
            body: &serde_json::json!({
                 "status": "error"
             })
        );
    }
}

pub async fn delete_task(mut req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    #[derive(Debug, Serialize, Deserialize)]
    struct RequestBody {
        task_id: Uuid,
    }
    let (tx, rx) = tokio::sync::oneshot::channel();
    let body = req.body_mut();
    if let Some(Ok(body)) = body.data().await {
        if let Ok(json_value) =
            serde_json::from_slice(&body) as Result<RequestBody, serde_json::Error>
        {
            let sender = req.data::<Sender<ServerMessage>>().unwrap();
            let task_id = json_value.task_id;
            reactor_failed!(sender
                .send(ServerMessage::DeleteTask { task_id, resp: tx })
                .await, "DeleteTask");
            if let Ok(_result) = rx.await {
                let status = "success";
                return response_json!(body: &serde_json::json!({ "status": status }));
            } else {
                println!("Abort RX - server waiting failed");
                return response_json!(
                    status: hyper::StatusCode::NOT_FOUND,
                    body: &serde_json::json!({
                         "status": "error"
                     })
                );
            }
        } else {
            println!("{:?}", &body);
        }
    }
    empty_malformed_body!()
}

pub async fn create_task(mut req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    #[derive(Debug, Serialize, Deserialize)]
    struct RequestBody {
        task_name: String,
        frequency: String,
        task_type: String,
        task_props: Value,
    }
    let (tx, rx) = tokio::sync::oneshot::channel();
    let body = req.body_mut();
    if let Some(Ok(body)) = body.data().await {
        if let Ok(json_value) =
            serde_json::from_slice(&body) as Result<RequestBody, serde_json::Error>
        {
            let sender = req.data::<Sender<ServerMessage>>().unwrap();
            reactor_failed!(sender
                .send(ServerMessage::CreateTask {
                    task_name: json_value.task_name,
                    frequency: json_value.frequency,
                    task_type: json_value.task_type,
                    task_props: json_value.task_props,
                    resp: tx,
                })
                .await, "CreateTask");
            let res = match rx.await {
                Ok(res) => res,
                Err(e) => {
                    let obj = serde_json::json!({
                        "error": e.to_string()
                    });
                    let obj = obj.to_string();
                    return response_json!(
                        status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                        body: &obj
                    );
                }
            };
            match res {
                Ok(task) => {
                    return response_json!(body: &task);
                }
                Err(e) => {
                    // DB Error
                    println!("{}", e.to_string());
                    let obj = serde_json::json!({
                        "error": e.to_string()
                    });
                    let obj = obj.to_string();

                    return response_json!(status: hyper::StatusCode::BAD_REQUEST, body: &obj);
                }
            }
        } else {
            println!("{:?}", &body);
        }
    }
    empty_malformed_body!()
}

pub async fn get_active_tasks(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sender = req.data::<Sender<ServerMessage>>().unwrap();
    reactor_failed!(sender
        .send(ServerMessage::GetActiveTasks { resp: tx })
        .await, "GetActiveTasks");

    let result = rx.await.unwrap();
    match result {
        Ok(result) => response_json!(body: &result),
        Err(e) => {
            let obj = serde_json::json!({
                "error": e.to_string()
            });
            println!("{}", e.to_string());
            let obj = obj.to_string();

            response_json!(status: hyper::StatusCode::INTERNAL_SERVER_ERROR, body: &obj)
        }
    }
}

pub async fn update_task(mut req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    #[derive(Debug, Serialize, Deserialize)]
    struct RequestBody {
        task_name: String,
        frequency: String,
        task_type: String,
        task_props: Value,
    }
    let (tx, rx) = tokio::sync::oneshot::channel();
    let task_id = match req.param("id") {
        Some(id) => Uuid::from_str(id).unwrap(),
        None => {
            let obj = serde_json::json!({
                "error": "Missing url parameter."
            });
            let obj = obj.to_string();

            return response_json!(status: hyper::StatusCode::BAD_REQUEST, body: &obj);
        }
    };
    let body = req.body_mut();
    if let Some(Ok(body)) = body.data().await {
        if let Ok(json_value) =
            serde_json::from_slice(&body) as Result<RequestBody, serde_json::Error>
        {
            let sender = req.data::<Sender<ServerMessage>>().unwrap();
            reactor_failed!(sender
                .send(ServerMessage::UpdateTask {
                    task_id: task_id,
                    task_name: json_value.task_name,
                    frequency: json_value.frequency,
                    task_props: json_value.task_props,
                    resp: tx,
                })
                .await, "UpdateTask");
            let res = match rx.await {
                Ok(res) => res,
                Err(e) => {
                    let obj = serde_json::json!({
                        "error": e.to_string()
                    });
                    let obj = obj.to_string();

                    return response_json!(
                        status: hyper::StatusCode::INTERNAL_SERVER_ERROR,
                        body: &obj
                    );
                }
            };
            match res {
                Ok(task) => {
                    return response_json!(body: &task);
                }
                Err(e) => {
                    // DB Error
                    println!("{}", e.to_string());
                    let obj = serde_json::json!({
                        "error": e.to_string()
                    });
                    let obj = obj.to_string();

                    return response_json!(status: hyper::StatusCode::BAD_REQUEST, body: &obj);
                }
            }
        } else {
            println!("{:?}", body);
        }
    }
    empty_malformed_body!()
}

pub async fn get_reports_for_task(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sender = req.data::<Sender<ServerMessage>>().unwrap();
    let task_id = match req.param("id") {
        Some(id) => Uuid::from_str(id).unwrap(),
        None => {
            let obj = serde_json::json!({
                "error": "Missing url parameter: id."
            });
            let obj = obj.to_string();
            println!("obj {:?}", obj);

            return response_json!(status: hyper::StatusCode::BAD_REQUEST, body: &obj);
        }
    };
    let query_map = match super::ServerUtils::get_qs(&req.uri().to_string()) {
        Ok(q) => q,
        Err(e) => {
            eprintln!("{}", e.to_string());
            let obj = serde_json::json!({
                "error": "Malformed query."
            });
            let obj = obj.to_string();
            return response_json!(status: hyper::StatusCode::BAD_REQUEST, body: &obj);
        }
    };
    let offset = query_map.get("offset").and_then(|x| x.parse::<i64>().ok());
    reactor_failed!(sender
        .send(ServerMessage::GetExecutionReportsForTask {
            task_id,
            offset,
            resp: tx,
        })
        .await, "GetExecutionReportsForTask");
    let result = rx.await.unwrap();
    match result {
        Ok(reports) => {
            return response_json!(body: &reports);
        }
        Err(e) => {
            error!("{}", e.to_string());
            return Err(anyhow::anyhow!(serde_json::json!({
                "error": "DB Error."
            })));
        }
    };
}

pub async fn get_report(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sender = req.data::<Sender<ServerMessage>>().unwrap();
    let report_id = match req.param("id") {
        Some(id) => Uuid::from_str(id).unwrap(),
        None => {
            let obj = serde_json::json!({
                "error": "Missing url parameter: id."
            });
            let obj = obj.to_string();
            println!("obj {:?}", obj);

            return response_json!(status: hyper::StatusCode::BAD_REQUEST, body: &obj);
        }
    };
    reactor_failed!(sender
        .send(ServerMessage::GetExecutionReport {
            resp: tx,
            report_id,
        })
        .await, "GetExecutionReport");
    let result = rx.await.unwrap();
    match result {
        Ok(reports) => {
            return response_json!(body: &reports);
        }
        Err(e) => {
            error!("{}", e.to_string());
            return Err(anyhow::anyhow!(serde_json::json!({
                "error": "DB Error."
            })));
        }
    };
}

pub async fn get_reports(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sender = req.data::<Sender<ServerMessage>>().unwrap();
    let query_map = match super::ServerUtils::get_qs(&req.uri().to_string()) {
        Ok(q) => q,
        Err(e) => {
            eprintln!("{}", e.to_string());
            let obj = serde_json::json!({
                "error": "Malformed query."
            });
            let obj = obj.to_string();
            return response_json!(status: hyper::StatusCode::BAD_REQUEST, body: &obj);
        }
    };
    let offset = query_map.get("offset").and_then(|x| x.parse::<i64>().ok());
    reactor_failed!(sender
        .send(ServerMessage::GetExecutionReports { offset, resp: tx })
        .await, "GetExecutionReports");
    let result = rx.await.unwrap();
    match result {
        Ok(reports) => {
            return response_json!(body: &reports);
        }
        Err(e) => {
            error!("{}", e.to_string());
            return Err(anyhow::anyhow!(serde_json::json!({
                "error": "DB Error."
            })));
        }
    };
}
