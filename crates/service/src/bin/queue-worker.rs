use std::time::Duration;

use crossbeam_channel::{bounded, select, tick, Receiver};
use deadlift_service::schema::{modules, workflow_modules};
use deadlift_service::{
    modules::engine::get_plugin_from_data, services::db, workflows::module::WorkflowModule,
};
use diesel::prelude::*;
use extism::*;
use futures_util::StreamExt;
use serde_json::Value;

fn ctrl_channel() -> Result<Receiver<()>, ctrlc::Error> {
    let (sender, receiver) = bounded(0);
    ctrlc::set_handler(move || {
        let _ = sender.send(());
    })?;

    Ok(receiver)
}

// FIXME-- tracing!!!
static LOGS: std::sync::Mutex<Vec<String>> = std::sync::Mutex::new(Vec::new());

fn handle_logs(msg: &str) {
    LOGS.lock().unwrap().push(msg.to_string())
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    db::init();

    set_log_callback(handle_logs, "extism=info").unwrap();

    tokio::spawn(async move {
        let conn = async_nats::connect("localhost").await.expect("nats conn");

        let mut sub = conn
            .subscribe("deadlift.modules.ingest.>")
            .await
            .expect("ingest modules sub");

        while let Some(message) = sub.next().await {
            println!("received message; subject: {}", message.subject);
            let conn = &mut db::connection().unwrap();

            let res: Vec<i32> = modules::table
                .inner_join(
                    workflow_modules::table.on(modules::hash.eq(workflow_modules::module_hash)),
                )
                .filter(
                    modules::subject
                        .eq(message.subject.to_string())
                        .and(workflow_modules::parent_workflow_module_id.is_null()),
                )
                .select(workflow_modules::workflow_id)
                .load(conn)
                .unwrap();

            if res.is_empty() {
                println!("failed to resolve workflow(s)");
                continue;
            }

            let payload_val =
                serde_json::from_slice::<serde_json::Value>(&message.payload).unwrap();
            tokio::task::spawn_blocking(move || {
                for workflow_id in res {
                    println!("resolved workflow id: {:?}", workflow_id);

                    let pipeline =
                        WorkflowModule::get_pipeline_from_workflow_id(workflow_id).unwrap();

                    let mut current_value = payload_val.clone();
                    for workflow_module in pipeline.iter().skip(1) {
                        if current_value == Value::default() {
                            println!("payload is null; stopping execution");
                            break;
                        }

                        let binary = deadlift_service::modules::module::Module::get_binary_by_hash(
                            workflow_module.get_hash(),
                        )
                        .unwrap(); // FIXME--

                        let mut plugin = get_plugin_from_data(binary);

                        current_value =
                            plugin.call::<Value, Value>("_main", current_value).unwrap();
                        // FIXME--
                    }

                    {
                        let mut logs = LOGS.lock().unwrap();
                        for line in logs.iter() {
                            print!("{}", line);
                        }

                        logs.clear();
                    }
                }
            })
            .await
            .unwrap();
        }
    });

    let ctrl_c_events = ctrl_channel().expect("ctrl c events");
    let ticks = tick(Duration::from_secs(5));

    loop {
        select! {
            recv(ticks) -> _ => {

            },
            recv(ctrl_c_events) -> _ => {
                break;
            }
        }
    }

    Ok(())
}
