use extism_pdk::*;
use hyper::{header::*, StatusCode};
use json::Value;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize, Serialize)]
struct SendParams {
    method: String,
    url: String,
    data: Option<Value>,
    token: Option<String>,
}

#[plugin_fn]
pub fn _main(Json(params): Json<SendParams>) -> FnResult<Value> {
    let mut req = HttpRequest::new(&params.url).with_method(&params.method);

    if params.data.is_some() {
        req = req.with_header(CONTENT_TYPE.as_str(), "application/json");
    }

    if let Some(token) = params.token {
        req = req.with_header(AUTHORIZATION.as_str(), format!("Token {}", token))
    }

    let res = http::request(&req, params.data)?;

    let status_code = StatusCode::from_u16(res.status_code())?;
    let status_ok = status_code.is_redirection() || status_code.is_success();

    let body = if let Ok(json) = res.json::<Value>() {
        Some(json)
    } else {
        None
    };

    // TODO-- broadcast to NATS

    Ok(json!({
        "status": status_code.as_u16(),
        "status_text": status_code.canonical_reason(),
        "ok": status_ok,
        "body": body,
        "url": params.url,
        "method": params.method
    }))
}
