use std::str::FromStr;

use super::ServerMessage;
use hyper::{body::HttpBody, Body, Request, Response};
use routerify::{Router, ext::RequestExt};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::mpsc::Sender;
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

pub async fn get_tasks(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sender = req.data::<Sender<ServerMessage>>().unwrap();
    sender
        .send(ServerMessage::GetTasks {
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

pub async fn get_task(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sender = req.data::<Sender<ServerMessage>>().unwrap();
    let task_id = req.param("id").unwrap();
    sender
        .send(ServerMessage::GetTask {
            task_id: Uuid::from_str(task_id).unwrap(),
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
            sender
                .send(ServerMessage::ExecuteTask { task_id, resp: tx })
                .await;
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
    return Err(anyhow::anyhow!(""));
    // println!("{:?}", task_id);
    // sender.send(ServerMessage::EXECUTE_TASK {
    //     task_id: (),
    //     resp: tx,
    // }).await;
    // panic!("AAAA")
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
            sender
                .send(ServerMessage::AbortTask { task_id, resp: tx })
                .await;
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
    panic!();
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
            sender
                .send(ServerMessage::DeleteTask { task_id, resp: tx })
                .await;
            if let Ok(result) = rx.await {
                let status = "success";
                // if result {
                //     status = "success"
                // } else {
                //     status = "error"
                // }
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
    panic!();
}

pub async fn create_task(mut req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    #[derive(Debug, Serialize, Deserialize)]
    struct RequestBody {
        task_name: String,
        frequency: String,
        task_type: String,
        task_props: Value
    }
    let (tx, rx) = tokio::sync::oneshot::channel();
    let body = req.body_mut();
    if let Some(Ok(body)) = body.data().await {
        if let Ok(json_value) =
            serde_json::from_slice(&body) as Result<RequestBody, serde_json::Error>
        {
            let sender = req.data::<Sender<ServerMessage>>().unwrap();
            sender
                .send(ServerMessage::CreateTask { task_name: json_value.task_name, frequency: json_value.frequency, task_type: json_value.task_type, task_props: json_value.task_props, resp: tx })
                .await;
            let res = match rx.await {
                Ok(res) => res,
                Err(e) => {
                    let obj = serde_json::json!({
                        "error": e.to_string()
                    });
                    let obj = obj.as_str();
                    let obj = obj.unwrap();
                    return response_json!(status: hyper::StatusCode::INTERNAL_SERVER_ERROR, body: &obj);
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
                    let obj = obj.as_str();
                    let obj = obj.unwrap();
                    return response_json!(status: hyper::StatusCode::BAD_REQUEST, body: &obj);
                }
            }
        } else {
            println!("{:?}", &body);
        }
    }
    panic!();
}

pub async fn get_active_tasks(req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    let sender = req.data::<Sender<ServerMessage>>().unwrap();
    sender
        .send(ServerMessage::GetActiveTasks {
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
            println!("{}", e.to_string());
            let obj = obj.as_str();
            let obj = obj.unwrap();
            response_json!(status: hyper::StatusCode::INTERNAL_SERVER_ERROR, body: &obj)
        }
    }
}