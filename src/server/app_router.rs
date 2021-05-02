use hyper::{Body, Request, Response};
use routerify::Router;

use crate::CONFIG;

const NOT_FOUND_HTML: &str = r#"
<html>
<style>
  html, body {
    width: 100%;
    height: 100%;
    font-family: monospace;
    display: flex;
    justify-content: center;
    align-items: center;
    font-size: large;
  }
</style>
<body>
  Panel's index.html file couldn't be found. Please check configuration file.
</body>
</html>
"#;

const NOT_ENABLED: &str = r#"
<html>
<style>
  html, body {
    width: 100%;
    height: 100%;
    font-family: monospace;
    display: flex;
    justify-content: center;
    align-items: center;
    font-size: large;
  }
</style>
<body>
  Panel feature is not enabled. Please check the configuration file.
</body>
</html>
"#;

// NOTE: NOT_ENABLED string is for future use, currently 
// if panel feature is disabled, StewardX needs to be
// restarted, so it'll never return NOT_ENABLED

pub async fn app(_req: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    if !**&CONFIG.get_features().get("panel").unwrap() {
        return Ok(Response::builder().body(NOT_ENABLED.into()).unwrap());
    }
    let index_path = &CONFIG.get_index_file_path();
    let index = tokio::fs::read_to_string(index_path).await.unwrap_or_else(|_| {
        NOT_FOUND_HTML.to_string()
    });
    return Ok(Response::builder().body(index.into()).unwrap());
}

pub fn app_router() -> Router<Body, anyhow::Error> {
    Router::builder().any(app).build().unwrap()
}
