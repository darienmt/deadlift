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
pub fn _main(Json(input): Json<FetchPayload>) -> FnResult<Json<Option<FetchPayload>>> {
    let is_post_request = input.method == "POST";
    let is_create_article_url = input.url == "https://api.realworld.io/api/articles";

    if is_post_request && is_create_article_url {
        Ok(Json(Some(input)))
    } else {
        Ok(Json(None))
    }
}
