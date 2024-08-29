use std::time::Duration;

use clap::Args;
use crossbeam_channel::{bounded, select, tick, Receiver};
use engine::config::EngineConfig;
use serde::Serialize;

#[derive(Args, Serialize)]
pub struct StartArgs {
    #[command(flatten)]
    config: EngineConfig,
}

pub async fn run_start_command(args: StartArgs) -> anyhow::Result<()> {
    let mut config_buffer = vec![];
    serde_yaml::to_writer(&mut config_buffer, &args.config)?;

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
