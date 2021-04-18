use hyper::{Body, Request, Response};
use routerify::Router;

pub async fn app(_req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    let index = tokio::fs::read_to_string("panel/index.html").await.unwrap();
    return Ok(Response::builder().body(index.into()).unwrap());
}

pub fn app_router() -> Router<Body, anyhow::Error> {
    Router::builder().any(app).build().unwrap()
}
