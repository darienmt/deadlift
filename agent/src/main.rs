use std::time::Duration;

use anyhow::Result;
use crossbeam_channel::{bounded, select, tick, Receiver};

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(0);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

// how to make this useful
// run nats-server with jetstream
// run 'deadlift wasm add <wasm path> --name <name>' to add wasm to nats object store, returns result
// run 'deadlift agent run' cmd to start agent, passing config file path, starts lasting process
// run 'deadlift agent call <fn name> --input <input>' to pass input into wasm modules, returns result
//
// then differentiate with extism:
// - create multiple agents, do distributed wasm execution
// - do high throughput wasm execution
// - create demo video that showcases high throughput distributed execution

#[tokio::main]
async fn main() -> Result<()> {
    let handles = engine::run(vec![]).await?; // FIXME-- define default agent config

    let ctrl_c_events = ctrl_channel().expect("ctrl c events");
    let ticks = tick(Duration::from_secs(5));

    loop {
        select! {
            recv(ticks) -> _ => {

            },
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
