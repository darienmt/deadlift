use std::time::Duration;

use clap::Args;
use crossbeam_channel::{bounded, select, tick, Receiver};
use engine::config::EngineConfig;
use serde::Serialize;

#[derive(Args, Serialize)]
pub struct StartArgs {
    #[command(flatten)]
    nats_config: NatsConfig,

    #[command(flatten)]
    plugin_config: PluginConfig,

    /// Set agent workflow
    #[arg(long)]
    workflow: String,
}

// TODO--
// -- how to not redefine these types in multiple places
// -- add clap derives under feature flag in engine crate?
// -- cli NatsAuthentication is camel case vs snake case in engine

#[derive(Args, Serialize)]
pub struct NatsConfig {
    // TODO-- move default nats url to const in engine crate
    #[arg(long, default_value_t = String::from("localhost:4222"))]
    pub nats_url: String,

    #[arg(long, default_value_t = NatsAuthentication::default())]
    pub nats_auth: NatsAuthentication,

    #[arg(long, default_value_t = true)]
    pub enable_execution_thread: bool,

    #[arg(long, default_value_t = true)]
    pub enable_watcher_thread: bool,
}

#[derive(Clone, Debug, Default, Serialize)]
pub enum NatsAuthentication {
    #[default]
    None,

    BearerJwt(String),
}

impl std::str::FromStr for NatsAuthentication {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(NatsAuthentication::None),
            jwt if jwt.starts_with("BearerJwt: ") => Ok(NatsAuthentication::BearerJwt(
                jwt["BearerJwt: ".len()..].to_string(),
            )),
            _ => Err(format!("Invalid authentication method: {}", s)),
        }
    }
}

impl std::fmt::Display for NatsAuthentication {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NatsAuthentication::None => write!(f, "none"),
            NatsAuthentication::BearerJwt(jwt) => write!(f, "BearerJwt: {}", jwt),
        }
    }
}

#[derive(Args, Serialize)]
pub struct PluginConfig {
    #[arg(long, default_value_t = true)]
    pub wasi: bool,

    #[arg(long)]
    pub allowed_hosts: Vec<String>,
}

pub async fn run_start_command(args: StartArgs) -> anyhow::Result<()> {
    let auth = match args.nats_config.nats_auth {
        NatsAuthentication::None => engine::config::NatsAuthentication::None,
        NatsAuthentication::BearerJwt(str) => engine::config::NatsAuthentication::BearerJwt(str),
    };

    let engine_config = EngineConfig {
        wasm: engine::config::WasmConfig::default(),
        workflow: engine::config::WorkflowConfig::default(),
        nats: engine::config::NatsConfig {
            url: args.nats_config.nats_url,
            auth,
            enable_execution_thread: args.nats_config.enable_execution_thread,
            enable_watcher_thread: args.nats_config.enable_watcher_thread,
        },
        plugin: engine::config::PluginConfig {
            wasi: args.plugin_config.wasi,
            allowed_hosts: args.plugin_config.allowed_hosts,
        },
    };

    let mut config_buffer = vec![];
    serde_yaml::to_writer(&mut config_buffer, &engine_config)?;

    let handles = engine::run(config_buffer).await?; // FIXME-- define default agent config

    let ctrl_c_events = ctrl_channel().expect("ctrl c events");
    let ticks = tick(Duration::from_secs(5));

    loop {
        select! {
            recv(ticks) -> _ => {},
            recv(ctrl_c_events) -> _ => {
                if let Some(v) = handles.execution_handle_opt { v.abort() }
                if let Some(v) = handles.watcher_handle_opt { v.abort() }
                break;
            }
            // default => {}
        }
    }

    Ok(())
}

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(0);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}
