use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, OnceLock, RwLock},
};

use anyhow::{anyhow, Result};
use futures_util::StreamExt;

use crate::config::NatsConfig;

const DEADLIFT_EXECUTIONS_QUEUE_GROUP: &str = "deadlift_executions";

// TODO-- no pins are needed since everything is passed around
static NATS_CLIENT: OnceLock<Arc<RwLock<async_nats::Client>>> = OnceLock::new();

static WASM_MAP: LazyLock<Arc<RwLock<HashMap<String, String>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

pub async fn require_nats(config: &NatsConfig) -> Result<async_nats::Client> {
    if NATS_CLIENT.get().is_none() {
        let nc = config.connect().await?;

        if NATS_CLIENT.set(Arc::new(RwLock::new(nc))).is_err() {
            // log instead of return here?
            return Err(anyhow!("failed to initialize nats client"));
        }
    }

    if let Some(guard) = NATS_CLIENT.get() {
        // lifetime ? dont want to hold an extended lock somehow
        if let Ok(nc) = guard.try_read() {
            return Ok(nc.clone());
        }
    }

    Err(anyhow!("failed to get nats client"))
}

pub fn get_wasm_map() -> Arc<RwLock<HashMap<String, String>>> {
    WASM_MAP.clone()
}

pub async fn start_execution_thread(
    nc: async_nats::Client,
    pool: extism::Pool,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        let subscriber = nc
            .queue_subscribe(
                "deadlift.executions.*",
                String::from(DEADLIFT_EXECUTIONS_QUEUE_GROUP),
            )
            .await
            .unwrap();

        let _ = subscriber
            .for_each_concurrent(100, |msg| {
                let nc = nc.clone();
                let pool = pool.clone();

                async move {
                    let res = tokio::task::spawn_blocking(move || {
                        match pool.get(
                            &String::from(msg.subject.as_str()),
                            std::time::Duration::from_millis(500),
                        ) {
                            Ok(Some(pool_plugin)) => pool_plugin
                                .call::<Vec<u8>, Vec<u8>>("probe_urls", msg.payload.to_vec()),
                            Ok(None) => {
                                Err(extism::Error::msg("failed to resolve plugin; timed out"))
                            }
                            Err(e) => {
                                Err(extism::Error::msg(format!("failed to resolve plugin; {e}")))
                            }
                        }
                    })
                    .await
                    .unwrap();

                    if let Some(reply) = msg.reply {
                        nc.publish(
                            reply,
                            res.unwrap_or_else(|e| e.to_string().into_bytes()).into(),
                        )
                        .await
                        .unwrap();
                    }
                }
            })
            .await;
    })
}
