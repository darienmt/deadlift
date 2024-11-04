use std::sync::OnceLock;

use anyhow::{anyhow, Result};
use extism::*;

use crate::config::PluginConfig;

static PLUGIN_POOL: OnceLock<extism::Pool> = OnceLock::new();

pub async fn require_plugin_pool(
    wasm: Vec<Wasm>,
    plugin_config: &PluginConfig,
) -> Result<extism::Pool> {
    if PLUGIN_POOL.get().is_none() {
        let mut manifest =
            Manifest::new(wasm).with_allowed_hosts(plugin_config.allowed_hosts.clone().into_iter());

        if let Some(extism_config) = &plugin_config.extism_config {
            manifest = manifest.with_config(extism_config.iter());
        }

        let plugin_builder = PluginBuilder::new(manifest).with_wasi(plugin_config.wasi);

        let pool = extism::Pool::new(100);
        pool.add_builder("probe_urls".to_string(), plugin_builder);

        if PLUGIN_POOL.set(pool).is_err() {
            // log instead of return here?
            return Err(anyhow!("failed to initialize plugin"));
        }
    }

    if let Some(guard) = PLUGIN_POOL.get() {
        return Ok(guard.clone());
    }

    Err(anyhow!("failed to get plugin"))
}
