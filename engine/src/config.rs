use anyhow::Result;
use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};

// TODO-- refactor config pieces into separate files under config mod, encapsulate fields, add field defaults
#[derive(Clone, Debug, Deserialize)]
pub struct EngineConfig {
    pub wasm: WasmConfig,
    pub workflow: WorkflowConfig,
    pub nats: NatsConfig,
    pub plugin: PluginConfig,
}

pub type WasmConfig = Vec<Wasm>;

// how to define whether the workflow starts in this config, or ends or is simply a piece
// receive the message/make the plugin call, if is next stage, make call
pub type WorkflowConfig = DiGraph<WorkflowStage, ()>;

#[derive(Clone, Debug, Deserialize)]
pub struct WorkflowStage {
    pub object_name: String,
    pub plugin_function_name: String,
}

// TODO
// -- update to encompass async_nats::ToServerAddrs
// -- naming
#[derive(Clone, Debug, Deserialize)]
pub struct NatsConfig {
    pub url: String,
    pub enable_execution_thread: bool,
    pub enable_watcher_thread: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PluginConfig {
    pub wasi: bool,
    pub allowed_hosts: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Wasm {
    // pub name: String,
    // pub bucket: String, // assume always 'wasm' bucket
    pub object_name: String, // rename to nats_object_name and convert to enum to support local wasm files
    pub namespace: String,
    pub hash: String,
    pub plugin_functions: Vec<String>, // TODO-- should be able to get this from analyzing wasm bytes, so that user does not have to provide
                                       // shared_functions ?
}

// TODO-- rename
pub fn require_config(bytes: Vec<u8>) -> Result<EngineConfig> {
    let config = serde_yaml::from_slice::<EngineConfig>(&bytes)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_config() {
        let result = serde_yaml::from_str::<EngineConfig>(
            "
            wasm:
                - object_name: make-auth-call
                  namespace: main
                  hash: 123abc
                  plugin_functions:
                    - _main
                - object_name: create-pop-token
                  namespace: create_pop_token
                  hash: 123abd
                  plugin_functions: []
                - object_name: create-jti
                  namespace: create_jti
                  hash: 123abe
                  plugin_functions: []
            workflow:
                nodes: []
                node_holes: []
                edge_property: directed
                edges: []
            nats:
                url: localhost:4222
                enable_execution_thread: true
                enable_watcher_thread: true
            plugin:
                wasi: true
                allowed_hosts: []
    ",
        );

        assert!(result.is_ok(), "{}", result.unwrap_err());
    }
}
