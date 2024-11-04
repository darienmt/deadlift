use std::sync::LazyLock;

use anyhow::Result;
use config::require_config;
use extism::Wasm;
use nats::{require_nats, start_execution_thread};
use plugin::require_plugin_pool;
use tokio::{io::AsyncReadExt, task::JoinHandle};

pub mod config;
pub mod nats;
pub mod plugin;
pub mod utils;

pub struct EngineThreadHandles {
    pub execution_handle_opt: Option<JoinHandle<()>>,
}

pub const MODULE_BUCKET_NAME: &str = "wasm";
pub const WORKFLOW_BUCKET_NAME: &str = "workflows";

pub static DEFAULT_NATS_URL: LazyLock<&'static str> =
    LazyLock::new(|| option_env!("NATS_URL").unwrap_or("localhost:4222"));

// refactor into agent crate? then engine mainly exports call fn for embedded? or split that into another new sdk crate
pub async fn run(config_bytes: Vec<u8>) -> Result<EngineThreadHandles> {
    extism::set_log_callback(|v| print!("{}", v), "info")?;

    let config = require_config(config_bytes)?;

    // TODO-- move all object items into nats crate
    let nc = require_nats(&config.nats).await?;

    let js = async_nats::jetstream::new(nc.clone());
    let workflow_bucket = js.get_object_store(WORKFLOW_BUCKET_NAME).await?;
    let module_bucket = js.get_object_store(MODULE_BUCKET_NAME).await?;

    let mut workflow_object = workflow_bucket.get(&config.workflow.name).await?;

    let mut workflow_bytes = vec![];
    workflow_object.read_to_end(&mut workflow_bytes).await?;

    let workflow = serde_yaml::from_slice::<config::WorkflowConfig>(&workflow_bytes)?;

    let mut modules = vec![];
    let mut module_names = std::collections::HashSet::new();
    for idx in workflow.graph.node_indices() {
        let stage = &workflow.graph[idx];

        let mut wasm_object = module_bucket.get(&stage.object_name).await?;

        let mut wasm_bytes = vec![];
        wasm_object.read_to_end(&mut wasm_bytes).await?;

        modules.push(Wasm::Data {
            data: wasm_bytes,
            meta: extism::WasmMetadata {
                name: stage.namespace.clone(),
                hash: stage.hash.clone(),
            },
        });

        module_names.insert(stage.object_name.clone());
    }

    let pool = require_plugin_pool(modules.clone(), &config.plugin).await?;

    let execution_handle_opt = if config.nats.enable_execution_thread {
        Some(start_execution_thread(nc.clone(), pool).await)
    } else {
        None
    };

    Ok(EngineThreadHandles {
        execution_handle_opt,
    })
}
