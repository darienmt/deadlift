use std::str::FromStr;

use extism::*;
use serde_json::Value;

host_fn!(pub batch_execute_postgres(config_str: String, sql: String) -> String {
    let mut client = postgres::Client::connect(&config_str, postgres::NoTls)?;

    let res = client.batch_execute(&sql)?;

    Ok(res)
});

host_fn!(pub query_postgres(config_str: String, query: String, json_schema: Value) -> Value {
    let mut client = postgres::Client::connect(&config_str, postgres::NoTls)?;

    let rows = client.query(&query, &[])?;

    let mut result = Vec::new();

    for row in rows {
        let mut json_row = serde_json::Map::new();

        if let Value::Object(schema) = &json_schema {
            for (key, schema_type) in schema {
                match schema_type.as_str() {
                    Some("string") => {
                        let value: String = row.try_get(key.as_str())?;
                        json_row.insert(key.clone(), Value::from(value));
                    }
                    Some("integer") => {
                        let value: i32 = row.try_get(key.as_str())?;
                        json_row.insert(key.clone(), Value::from(value));
                    }
                    Some("float") => {
                        let value: f64 = row.try_get(key.as_str())?;
                        json_row.insert(key.clone(), Value::from(value));
                    }
                    Some("boolean") => {
                        let value: bool = row.try_get(key.as_str())?;
                        json_row.insert(key.clone(), Value::from(value));
                    }
                    _ => {
                        return Err(anyhow::anyhow!(format!("unsupported schema type: {}", schema_type)));
                    }
                }
            }
        }

        result.push(Value::Object(json_row));
    }

    Ok(Value::Array(result))
});

extism::host_fn!(pub batch_http_request(requests_json: Value) -> Value {
    let request_values = requests_json.as_array().cloned().ok_or(anyhow::anyhow!("failed to parse requests as array"))?;

    let client = reqwest::Client::new();
    let rt = tokio::runtime::Handle::current();

    let responses_json: Vec<Value> = rt.block_on(async {
        let mut futures = vec![];

        // rayon par_iter ?
        for request_value in request_values {
            let client = client.clone();
            let future = async move {
                let url_str = request_value["url"].as_str().ok_or(anyhow::anyhow!("failed to get request url"))?;
                let url = reqwest::Url::from_str(url_str)?;

                let method_str = request_value["method"].as_str().ok_or(anyhow::anyhow!("failed to get request method"))?;
                let method = reqwest::Method::from_bytes(method_str.as_bytes())?;

                let request = reqwest::Request::new(method, url);

                let mut headers = reqwest::header::HeaderMap::new();
                if let Some(header_values) = request_value["headers"].as_object() {
                    for (header_key, header_value) in header_values {
                        if let Some(header_value_str) = header_value.as_str() {
                            if let (Ok(header_name), Ok(header_value)) = (
                                reqwest::header::HeaderName::from_bytes(header_key.as_bytes()),
                                reqwest::header::HeaderValue::from_str(header_value_str),
                            ) {
                                headers.insert(header_name, header_value);
                            }
                        }
                    }
                }

                let mut request_builder = reqwest::RequestBuilder::from_parts(client, request).headers(headers);

                if let Some(body) = request_value.get("body") {
                    let mut buf = vec![];
                    serde_json::to_writer(&mut buf, body)?;

                    request_builder = request_builder.body(buf);
                }

                let response = request_builder.send().await?;

                let result = serde_json::json!({
                    "url": url_str,
                    "method": method_str,
                    "status": response.status().as_u16(),
                    "body": response.text().await?
                });

                Ok(result) as anyhow::Result<Value>
            };

            futures.push(future);
        }

        futures::future::join_all(futures).await.into_iter().collect::<anyhow::Result::<Vec<Value>>>()
    })?;

    Ok(Value::Array(responses_json))
});
