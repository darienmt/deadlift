use anyhow::Result;
use config::require_config;
use extism::Wasm;
use nats::{require_nats, start_execution_thread, start_watcher_thread};
use plugin::require_plugin;
use tokio::{io::AsyncReadExt, task::JoinHandle};

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

    let plugin = require_plugin(modules.clone(), &config.plugin).await?;

    let execution_handle_opt = if config.nats.enable_execution_thread {
        Some(start_execution_thread(nc.clone(), plugin.clone()).await)
    } else {
        None
    };

    let watcher_handle_opt = if config.nats.enable_watcher_thread {
        Some(start_watcher_thread(modules, std::sync::Arc::new(module_names), nc, plugin).await)
    } else {
        None
    };

    Ok(EngineThreadHandles {
        execution_handle_opt,
        watcher_handle_opt,
    })
}
