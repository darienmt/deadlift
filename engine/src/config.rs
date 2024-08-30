use std::collections::HashMap;

use anyhow::Result;
use async_nats::ConnectOptions;
use directories::ProjectDirs;
use petgraph::graph::DiGraph;
use serde::{Deserialize, Serialize};

use crate::DEFAULT_NATS_URL;

// add top level engine/deadlift/type field that is 'sdk/engine' or 'agent'

// TODO-- refactor config pieces into separate files under config mod, encapsulate fields, add field defaults
#[cfg_attr(feature = "clap", derive(clap::Args))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EngineConfig {
    #[cfg_attr(feature = "clap", command(flatten))]
    #[serde(default)]
    pub workflow: WorkflowConfig,

    #[cfg_attr(feature = "clap", command(flatten))]
    pub nats: NatsConfig,

    #[cfg_attr(feature = "clap", command(flatten))]
    pub plugin: PluginConfig,
}

// how to define whether the workflow starts in this config, or ends or is simply a piece
// receive the message/make the plugin call, if is next stage, make call

#[cfg_attr(feature = "clap", derive(clap::Args))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct WorkflowConfig {
    #[cfg_attr(feature = "clap", arg(long))]
    pub name: String,

    #[cfg_attr(feature = "clap", arg(skip))]
    #[serde(flatten)]
    pub graph: DiGraph<WorkflowStage, ()>,
}

#[cfg_attr(feature = "clap", derive(clap::Args))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkflowStage {
    pub object_name: String, // rename to nats_object_name and convert to enum to support local wasm files
    pub namespace: Option<String>, // make optional- should be able to get this from wasm bytes, or generate and assign random namespaces if multiple top level wasm
    pub hash: Option<String>,
    pub plugin_function_name: String,
    // plugin_functions
    // TODO-- should be able to get this from analyzing wasm bytes, so that user does not have to provide
    // shared_functions ?
    //
    // TODO-- depends_on field with list of other modules that are depended on
}

// TODO
// -- update to encompass async_nats::ToServerAddrs
// -- naming
#[cfg_attr(feature = "clap", derive(clap::Args))]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NatsConfig {
    #[cfg_attr(feature = "clap", arg(long, default_value_t = DEFAULT_NATS_URL.to_string()))]
    #[serde(default = "default_nats_url")]
    pub url: String,

    #[cfg_attr(feature = "clap", arg(long, default_value_t = NatsAuthentication::default()))]
    #[serde(default)]
    pub auth: NatsAuthentication,

    #[cfg_attr(feature = "clap", arg(long, default_value_t = true))]
    #[serde(default = "default_true")]
    pub enable_execution_thread: bool,

    #[cfg_attr(feature = "clap", arg(long, default_value_t = true))]
    #[serde(default = "default_true")]
    pub enable_watcher_thread: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum NatsAuthentication {
    None,
    BearerJwt(String),
    Creds(String),
}

impl Default for NatsAuthentication {
    // FIXME-- wrong use of default, should also be scoped to cli feature flag
    fn default() -> Self {
        let proj_dirs = ProjectDirs::from("com", "ZeroSync", "deadlift").unwrap(); // FIXME--
        let creds_path = proj_dirs.config_dir().join("user.creds");

        if creds_path.is_file() {
            Self::Creds(creds_path.to_string_lossy().to_string()) // FIXME--
        } else if let Some(jwt) = option_env!("NATS_BEARER_JWT") {
            Self::BearerJwt(String::from(jwt))
        } else {
            Self::None
        }
    }
}

#[cfg(feature = "clap")]
impl std::str::FromStr for NatsAuthentication {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(NatsAuthentication::None),
            jwt if jwt.starts_with("BearerJwt: ") => Ok(NatsAuthentication::BearerJwt(
                jwt["BearerJwt: ".len()..].to_string(),
            )),
            creds if creds.starts_with("Creds: ") => Ok(NatsAuthentication::Creds(
                creds["Creds: ".len()..].to_string(),
            )),
            _ => Err(format!("Invalid authentication method: {}", s)),
        }
    }
}

#[cfg(feature = "clap")]
impl std::fmt::Display for NatsAuthentication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NatsAuthentication::None => write!(f, "none"),
            NatsAuthentication::BearerJwt(jwt) => write!(f, "BearerJwt: {}", jwt),
            NatsAuthentication::Creds(creds) => write!(f, "Creds: {}", creds),
        }
    }
}

impl NatsConfig {
    pub async fn connect(&self) -> Result<async_nats::Client> {
        match &self.auth {
            NatsAuthentication::None => ConnectOptions::default(),
            NatsAuthentication::BearerJwt(jwt) => async_nats::ConnectOptions::with_jwt(
                jwt.clone(),
                move |_| async move { Ok(vec![]) },
            ),
            NatsAuthentication::Creds(creds_path) => {
                ConnectOptions::with_credentials_file(creds_path).await?
            }
        }
        .connect(&self.url)
        .await
        .map_err(anyhow::Error::from)
    }
}

#[cfg_attr(feature = "clap", derive(clap::Args))]
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PluginConfig {
    #[cfg_attr(feature = "clap", arg(long, default_value_t = true))]
    #[serde(default = "default_true")]
    pub wasi: bool,

    #[cfg_attr(feature = "clap", arg(long))]
    #[serde(default)]
    pub allowed_hosts: Vec<String>,

    #[cfg_attr(feature = "clap", arg(long,  value_parser=get_extism_config_from_str))]
    #[serde(default)]
    pub extism_config: Option<HashMap<String, String>>,
}

// feature scope this?
fn get_extism_config_from_str(s: &str) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();

    for pair in s.split(',') {
        let mut iter = pair.splitn(2, '=');
        let key = iter
            .next()
            .ok_or_else(|| String::from("key must be a string"))?;
        let value = iter
            .next()
            .ok_or_else(|| String::from("value must be a string"))?;
        map.insert(key.to_string(), value.to_string());
    }

    Ok(map)
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
    DEFAULT_NATS_URL.to_string()
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
                auth: !BearerJwt jwt
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
