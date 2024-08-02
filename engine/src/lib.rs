use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex, OnceLock, RwLock},
};

use anyhow::{anyhow, Result};
use extism::{Manifest, Plugin, PluginBuilder, Wasm, WasmMetadata};
use futures_util::StreamExt;
use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};
use tokio::{io::AsyncReadExt, task::JoinHandle};

#[derive(Clone, Debug, Deserialize, Serialize)]
struct Node {
    name: String,
    bucket: String,
    object: String,
    namespace: String,
    hash: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct Edge {
    source: String,
    target: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct GraphConfig {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
}

static CONFIG: LazyLock<DiGraph<Node, ()>> = LazyLock::new(|| {
    let config_path = "./config.yaml";
    let config_contents = std::fs::read_to_string(config_path).unwrap();
    let graph_config = serde_yaml::from_str::<GraphConfig>(&config_contents).unwrap();
    build_graph_from_config(graph_config).unwrap()
});

static NATS_CLIENT: OnceLock<Arc<RwLock<async_nats::Client>>> = OnceLock::new();

static WASM_MAP: LazyLock<Arc<RwLock<HashMap<String, String>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

static WATCHER_THREAD: LazyLock<JoinHandle<()>> = LazyLock::new(|| {
    tokio::task::spawn(async move {
        let nc = require_nats().await.unwrap();
        let js = async_nats::jetstream::new(nc.clone());

        let wasm_store = js.get_object_store("wasm").await.unwrap();

        let mut watcher = wasm_store.watch().await.unwrap();

        while let Some(res) = watcher.next().await {
            if let Ok(change) = res {
                let is_relevant_change = {
                    let arc = WASM_MAP.clone();
                    let wasm_map = arc.read().unwrap();
                    wasm_map.get(&change.name).is_some()
                };

                if is_relevant_change {
                    let updated_manifest =
                        create_manifest(CONFIG.clone(), nc.clone()).await.unwrap();

                    let updated_plugin = PluginBuilder::new(updated_manifest)
                        .with_wasi(true)
                        .build()
                        .unwrap();

                    let plugin_guard = PLUGIN.get().cloned().unwrap();

                    {
                        let mut plugin = plugin_guard.lock().unwrap();
                        *plugin = updated_plugin;
                    }
                }
            }
        }
    })
});

static PLUGIN: OnceLock<Arc<Mutex<Plugin>>> = OnceLock::new();

fn build_graph_from_config(config: GraphConfig) -> Result<DiGraph<Node, ()>> {
    let mut graph = DiGraph::new();

    let mut indices = std::collections::HashMap::new();
    for node in &config.nodes {
        let index = graph.add_node(node.clone());
        indices.insert(node.name.clone(), index);
    }

    for edge in &config.edges {
        let source_index = *indices
            .get(&edge.source)
            .ok_or(anyhow!("failed to get source index"))?;
        let target_index = *indices
            .get(&edge.target)
            .ok_or(anyhow!("failed to get target index"))?;
        graph.add_edge(source_index, target_index, ());
    }

    Ok(graph)
}

async fn require_nats() -> Result<async_nats::Client> {
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

async fn create_manifest(graph: DiGraph<Node, ()>, nc: async_nats::Client) -> Result<Manifest> {
    let js = async_nats::jetstream::new(nc);

    let wasm_store = js.get_object_store("wasm").await?;

    let mut modules = vec![];
    let mut info = HashMap::new();

    for node_index in graph.node_indices() {
        let node = graph
            .node_weight(node_index)
            .ok_or(anyhow!("failed to get node from index"))?;

        let mut object = wasm_store.get(&node.object).await?;

        info.insert(
            node.name.clone(),
            object.info.digest.clone().unwrap_or_default(),
        );
        // insert name and digest into hashmap

        let mut data = vec![];
        object.read_to_end(&mut data).await?;

        modules.push(Wasm::Data {
            data,
            meta: WasmMetadata {
                name: Some(node.namespace.clone()),
                hash: Some(node.hash.clone()),
            },
        });
    }

    {
        if let Ok(mut write_lock) = WASM_MAP.clone().write() {
            write_lock.clone_from(&info);
        }
    }

    Ok(Manifest::new(modules))
}

pub async fn require_plugin(graph: DiGraph<Node, ()>) -> Result<Arc<Mutex<Plugin>>> {
    if PLUGIN.get().is_none() {
        let nc = require_nats().await?;
        let js = async_nats::jetstream::new(nc);

        let wasm_store = js.get_object_store("wasm").await?;

        let mut modules = vec![];

        for node_index in graph.node_indices() {
            let node = graph
                .node_weight(node_index)
                .ok_or(anyhow!("failed to get node from index"))?;

            let mut object = wasm_store.get(&node.object).await?;

            let mut data = vec![];
            object.read_to_end(&mut data).await?;

            modules.push(Wasm::Data {
                data,
                meta: WasmMetadata {
                    name: Some(node.namespace.clone()),
                    hash: Some(node.hash.clone()),
                },
            });
        }

        let manifest = Manifest::new(modules);

        let plugin = PluginBuilder::new(manifest).with_wasi(true).build()?;

        if PLUGIN.set(Arc::new(Mutex::new(plugin))).is_err() {
            // log instead of return here?
            return Err(anyhow!("failed to initialize plugin"));
        }
    }

    if let Some(guard) = PLUGIN.get() {
        return Ok(guard.clone());
    }

    Err(anyhow!("failed to get plugin"))
}

pub async fn start_watcher_thread() -> tokio::task::AbortHandle {
    WATCHER_THREAD.abort_handle()
}

// static NATS_CLIENT: LazyLock<Result<async_nats::Client>> = LazyLock::new(|| {
//     let default_nats_endpoint = "localhost:4222";
// });

// pub async fn call() -> Result<Vec<u8>> {
//     let config = require_config().await?;

//     let nats = require_nats().await?;
// }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_graph_from_config() {
        let graph_config = serde_yaml::from_str::<GraphConfig>(
            "
            nodes:
              - name: make_auth_call
                bucket: wasm
                object: make-auth-call
                namespace: main
                hash: 123abc
              - name: create_pop_token
                bucket: wasm
                object: create-pop-token
                namespace: create_pop_token
                hash: 123abd
              - name: create_jti
                bucket: wasm
                object: create-jti
                namespace: create_jti
                hash: 123abe

            edges:
              - source: make_auth_call
                target: create_pop_token
              - source: create_pop_token
                target: create_jti
        ",
        )
        .unwrap();

        let expected_graph = serde_json::from_str::<DiGraph<Node, ()>>(
            r#"{
            "nodes": [
                {
                    "name": "make_auth_call",
                    "bucket": "wasm",
                    "object": "make-auth-call",
                    "namespace": "main",
                    "hash": "123abc"
                },
                {
                    "name": "create_pop_token",
                    "bucket": "wasm",
                    "object": "create-pop-token",
                    "namespace": "create_pop_token",
                    "hash": "123abd"
                },
                {
                    "name": "create_jti",
                    "bucket": "wasm",
                    "object": "create-jti",
                    "namespace": "create_jti",
                    "hash": "123abe"
                }
            ],
            "node_holes": [],
            "edge_property": "directed",
            "edges": [
                [
                    0,
                    1,
                    null
                ],
                [
                    1,
                    2,
                    null
                ]
            ]
            }"#,
        )
        .unwrap();

        let actual_graph = build_graph_from_config(graph_config).unwrap();

        let actual_value = serde_json::to_value(actual_graph).unwrap();
        let expected_value = serde_json::to_value(expected_graph).unwrap();
        assert_eq!(actual_value, expected_value);
    }
}
