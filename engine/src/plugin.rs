use std::sync::{Arc, Mutex, OnceLock};

use anyhow::{anyhow, Result};
use extism::*;

use crate::host_functions::*;

use crate::config::PluginConfig;

static PLUGIN: OnceLock<Arc<Mutex<Plugin>>> = OnceLock::new();

pub async fn require_plugin(
    wasm: Vec<Wasm>,
    plugin_config: &PluginConfig,
) -> Result<Arc<Mutex<Plugin>>> {
    if PLUGIN.get().is_none() {
        let mut manifest =
            Manifest::new(wasm).with_allowed_hosts(plugin_config.allowed_hosts.clone().into_iter());

        if let Some(extism_config) = &plugin_config.extism_config {
            manifest = manifest.with_config(extism_config.iter());
        }

        let plugin = PluginBuilder::new(manifest)
            .with_wasi(plugin_config.wasi)
            .with_function(
                "batch_execute_postgres",
                [PTR, PTR],
                [PTR],
                UserData::default(),
                batch_execute_postgres,
            )
            .with_function(
                "query_postgres",
                [PTR, PTR, PTR],
                [PTR],
                UserData::default(),
                query_postgres,
            )
            .with_function(
                "batch_http_request",
                [PTR],
                [PTR],
                UserData::default(),
                batch_http_request,
            )
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
