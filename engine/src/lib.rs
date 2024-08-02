use anyhow::Result;
use config::get_config;
use nats::{get_wasm_map, require_nats, start_watcher_thread};
use plugin::require_plugin;

mod config;
mod nats;
mod plugin;

pub async fn run() -> Result<()> {
    let graph = get_config();
    let nc = require_nats().await?;
    let wasm_map = get_wasm_map();
    let plugin = require_plugin(&graph, nc.clone()).await?;

    let _handle = start_watcher_thread(graph, nc, wasm_map, plugin);

    // TODO-- start_inbound_executions_thread fn

    Ok(())
}
