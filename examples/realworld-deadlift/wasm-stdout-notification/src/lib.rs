use extism_pdk::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Deserialize, Serialize)]
struct FetchPayload {
    status: u16,
    status_text: Option<String>,
    ok: bool,
    body: Value,
    url: String,
    method: String,
}

#[plugin_fn]
pub fn _main(Json(input): Json<FetchPayload>) -> FnResult<Json<()>> {
    let created_article_title = input.body["article"]
        .as_object()
        .and_then(|v| v["title"].as_str())
        .ok_or(Error::msg("article title not found"))?;

    info!(
        "A new blog post titled '{}' was created",
        created_article_title
    );

    Ok(Json(()))
}
