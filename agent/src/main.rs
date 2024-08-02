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

#[tokio::main]
async fn main() -> Result<()> {
    let watcher_handle = engine::run().await?;

    let ctrl_c_events = ctrl_channel().expect("ctrl c events");
    let ticks = tick(Duration::from_secs(5));

    loop {
        select! {
            recv(ticks) -> _ => {

            },
            recv(ctrl_c_events) -> _ => {
                watcher_handle.abort();
                break;
            }
            // default => {}
        }
    }

    Ok(())
}
