use extism_pdk::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize)]
struct RealworldFetchPayload {
    request: RequestInfo,
    response: ResponseInfo,
}

#[derive(Deserialize, Serialize)]
struct RequestInfo {
    opts: Value,
    url: String,
}

#[derive(Deserialize, Serialize)]
struct ResponseInfo {
    body: Value,
    url: String,
}

#[plugin_fn]
pub fn _main(Json(input): Json<RealworldFetchPayload>) -> FnResult<Json<()>> {
    let created_article_title = input.response.body["article"]
        .as_object()
        .and_then(|v| v["title"].as_str())
        .ok_or(Error::msg("article title not found"))?;

    info!(
        "A new blog post titled '{}' was created",
        created_article_title
    );

    Ok(Json(()))
}
