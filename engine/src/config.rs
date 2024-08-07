use anyhow::Result;
use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};

// TODO-- refactor config pieces into separate files under config mod, encapsulate fields, add field defaults
#[derive(Clone, Debug, Deserialize)]
pub struct EngineConfig {
    pub graph: GraphConfig,
    pub nats: NatsConfig,
    pub plugin: PluginConfig,
}

pub type GraphConfig = DiGraph<Node, ()>;

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
pub struct Node {
    pub name: String,
    pub bucket: String,
    pub object: String,
    pub namespace: String,
    pub hash: String,
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
            graph:
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
                node_holes: []
                edge_property: directed
                edges:
                  - - 0
                    - 1
                    - null
                  - - 1
                    - 2
                    - null
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
