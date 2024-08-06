use anyhow::Result;
use config::get_config;
use nats::{get_wasm_map, require_nats, start_execution_thread, start_watcher_thread};
use plugin::require_plugin;
use tokio::task::JoinHandle;

mod config;
mod nats;
mod plugin;

pub struct EngineThreadHandles {
    pub execution_handle: JoinHandle<()>,
    pub watcher_handle: JoinHandle<()>,
}

pub async fn run() -> Result<EngineThreadHandles> {
    let graph = get_config(); // TODO-- pass config to run function instead of initializing here
    let nc = require_nats().await?;
    let wasm_map = get_wasm_map();
    let plugin = require_plugin(&graph, nc.clone()).await?;

    let execution_handle = start_execution_thread(nc.clone(), plugin.clone()).await;
    let watcher_handle = start_watcher_thread(graph, nc, wasm_map, plugin).await;

    Ok(EngineThreadHandles {
        execution_handle,
        watcher_handle,
    })
}
