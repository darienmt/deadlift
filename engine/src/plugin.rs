use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock, RwLock},
};

use anyhow::{anyhow, Result};
use extism::{Manifest, Plugin, PluginBuilder, Wasm, WasmMetadata};
use tokio::io::AsyncReadExt;

use crate::config::{GraphConfig, PluginConfig};

static PLUGIN: OnceLock<Arc<Mutex<Plugin>>> = OnceLock::new();

pub async fn create_manifest(
    graph_config: &GraphConfig,
    nc: async_nats::Client,
    wasm_map: Arc<RwLock<HashMap<String, String>>>,
) -> Result<Manifest> {
    let js = async_nats::jetstream::new(nc);

    let wasm_store = js.get_object_store("wasm").await?;

    let mut modules = vec![];
    let mut info = HashMap::new();

    for node_index in graph_config.node_indices() {
        let node = graph_config
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
        if let Ok(mut write_lock) = wasm_map.clone().write() {
            write_lock.clone_from(&info);
        }
    }

    Ok(Manifest::new(modules))
}

pub async fn require_plugin(
    graph_config: &GraphConfig,
    plugin_config: &PluginConfig,
    nc: async_nats::Client,
) -> Result<Arc<Mutex<Plugin>>> {
    if PLUGIN.get().is_none() {
        let js = async_nats::jetstream::new(nc);

        let wasm_store = js.get_object_store("wasm").await?;

        let mut modules = vec![];

        for node_index in graph_config.node_indices() {
            let node = graph_config
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

        let mut manifest = Manifest::new(modules);
        if !plugin_config.allowed_hosts.is_empty() {
            manifest = manifest.with_allowed_hosts(plugin_config.allowed_hosts.clone().into_iter())
        }

        let plugin = PluginBuilder::new(manifest)
            .with_wasi(plugin_config.wasi)
            .build()?;

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
