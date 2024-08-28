use anyhow::Result;
use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};

// add top level engine/deadlift/type field that is 'sdk/engine' or 'agent'

// TODO-- refactor config pieces into separate files under config mod, encapsulate fields, add field defaults
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineConfig {
    #[serde(default)]
    pub wasm: WasmConfig,

    #[serde(default)]
    pub workflow: WorkflowConfig,

    pub nats: NatsConfig,

    pub plugin: PluginConfig,
}

pub type WasmConfig = Vec<Wasm>;

// how to define whether the workflow starts in this config, or ends or is simply a piece
// receive the message/make the plugin call, if is next stage, make call
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WorkflowConfig {
    pub name: String,
    #[serde(flatten)]
    pub graph: DiGraph<WorkflowStage, ()>,
}

// add hash field ?
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowStage {
    pub object_name: String,
    pub plugin_function_name: String,
}

// TODO
// -- update to encompass async_nats::ToServerAddrs
// -- naming
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NatsConfig {
    #[serde(default = "default_nats_url")]
    pub url: String,

    #[serde(default)]
    pub auth: NatsAuthentication,

    #[serde(default = "default_true")]
    pub enable_execution_thread: bool,

    #[serde(default = "default_true")]
    pub enable_watcher_thread: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NatsAuthentication {
    #[default]
    None,

    BearerJwt(String),
}

impl NatsAuthentication {
    pub fn into_connect_options(self) -> async_nats::ConnectOptions {
        match self {
            Self::None => async_nats::ConnectOptions::default(),
            Self::BearerJwt(jwt) => {
                async_nats::ConnectOptions::with_jwt(jwt, move |_| async move { Ok(vec![]) })
            }
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PluginConfig {
    #[serde(default = "default_true")]
    pub wasi: bool,

    #[serde(default)]
    pub allowed_hosts: Vec<String>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Wasm {
    // pub name: String,
    // pub bucket: String, // assume always 'wasm' bucket
    pub object_name: String, // rename to nats_object_name and convert to enum to support local wasm files
    pub namespace: String, // make optional- should be able to get this from wasm bytes, or generate and assign random namespaces if multiple top level wasm
    pub hash: String,      // make optional
    pub plugin_functions: Vec<String>, // TODO-- should be able to get this from analyzing wasm bytes, so that user does not have to provide
                                       // shared_functions ?
                                       //
                                       // TODO-- depends_on field with list of other modules that are depended on
}

// TODO-- rename
pub fn require_config(bytes: Vec<u8>) -> Result<EngineConfig> {
    let config = serde_yaml::from_slice::<EngineConfig>(&bytes)?;
    Ok(config)
}

fn default_true() -> bool {
    true
}

fn default_nats_url() -> String {
    String::from("localhost:4222")
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
                name: test
                nodes: []
                node_holes: []
                edge_property: directed
                edges: []
            nats:
                url: localhost:4222
                auth: !bearer_jwt jwt
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
