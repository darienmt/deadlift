use anyhow::Result;
use config::require_config;
use nats::{get_wasm_map, require_nats, start_execution_thread, start_watcher_thread};
use plugin::require_plugin;
use tokio::task::JoinHandle;

pub mod config;
pub mod nats;
pub mod plugin;
pub mod utils;

pub struct EngineThreadHandles {
    pub execution_handle_opt: Option<JoinHandle<()>>,
    pub watcher_handle_opt: Option<JoinHandle<()>>,
}

pub const MODULE_BUCKET_NAME: &str = "wasm";
pub const WORKFLOW_BUCKET_NAME: &str = "workflows";

// refactor into agent crate? then engine mainly exports call fn for embedded? or split that into another new sdk crate
pub async fn run(config_bytes: Vec<u8>) -> Result<EngineThreadHandles> {
    let config = require_config(config_bytes)?;
    let nc = require_nats(&config.nats).await?;
    let wasm_map = get_wasm_map();
    let plugin = require_plugin(&config.wasm, &config.plugin, nc.clone()).await?;

    let execution_handle_opt = if config.nats.enable_execution_thread {
        Some(start_execution_thread(nc.clone(), plugin.clone()).await)
    } else {
        None
    };

    let watcher_handle_opt = if config.nats.enable_watcher_thread {
        Some(start_watcher_thread(config.wasm, nc, wasm_map, plugin).await)
    } else {
        None
    };

    Ok(EngineThreadHandles {
        execution_handle_opt,
        watcher_handle_opt,
    })
}
