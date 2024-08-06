use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex, OnceLock, RwLock},
};

use anyhow::{anyhow, Result};
use extism::{Plugin, PluginBuilder};
use futures_util::StreamExt;
use petgraph::graph::DiGraph;

use crate::{config::Node, plugin::create_manifest};

static NATS_CLIENT: OnceLock<Arc<RwLock<async_nats::Client>>> = OnceLock::new();

static WASM_MAP: LazyLock<Arc<RwLock<HashMap<String, String>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

pub async fn require_nats() -> Result<async_nats::Client> {
    if NATS_CLIENT.get().is_none() {
        let default_nats_endpoint = "localhost:4222";
        let nc = async_nats::connect(default_nats_endpoint).await?;

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

pub async fn start_watcher_thread(
    graph: DiGraph<Node, ()>,
    nc: async_nats::Client,
    wasm_map: Arc<RwLock<HashMap<String, String>>>,
    plugin: Arc<Mutex<Plugin>>,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        let js = async_nats::jetstream::new(nc.clone());

        let wasm_store = js.get_object_store("wasm").await.unwrap();

        let mut watcher = wasm_store.watch().await.unwrap();

        while let Some(res) = watcher.next().await {
            if let Ok(change) = res {
                let is_relevant_change = {
                    let arc = wasm_map.clone();
                    let map = arc.read().unwrap();
                    map.get(&change.name).is_some()
                };

                if is_relevant_change {
                    let updated_manifest =
                        create_manifest(graph.clone(), nc.clone(), wasm_map.clone())
                            .await
                            .unwrap();

                    let updated_plugin = PluginBuilder::new(updated_manifest)
                        .with_wasi(true)
                        .build()
                        .unwrap();

                    let plugin_guard = plugin.clone();

                    {
                        let mut plugin = plugin_guard.lock().unwrap();
                        *plugin = updated_plugin;
                    }
                }
            }
        }
    })
}

pub async fn start_execution_thread(
    nc: async_nats::Client,
    plugin: Arc<Mutex<Plugin>>,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        let mut subscriber = nc.subscribe("deadlift.executions.*").await.unwrap();

        while let Some(msg) = subscriber.next().await {
            if let Ok(mut plugin) = plugin.try_lock() {
                let fn_name = msg.subject.as_str().split('.').last().unwrap();

                let res = plugin.call::<Vec<u8>, Vec<u8>>(fn_name, msg.payload.into());

                if let Ok(bytes) = res {
                    println!(
                        "successfully called agent; result: {}",
                        String::from_utf8_lossy(bytes.as_slice())
                    );
                } else {
                    eprintln!("failed to call agent; result: {res:?}");
                }

                continue;
            }

            eprintln!("failed to acquire plugin lock");
        }
    })
}
