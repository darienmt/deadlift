use std::{
    collections::HashMap,
    sync::{Arc, LazyLock, Mutex, OnceLock, RwLock},
};

use anyhow::{anyhow, Result};
use extism::{Manifest, Plugin, PluginBuilder, Wasm};
use futures_util::StreamExt;
use tokio::io::AsyncReadExt;

use crate::{config::NatsConfig, utils::get_or_create_object_store, MODULE_BUCKET_NAME};

const DEADLIFT_EXECUTIONS_QUEUE_GROUP: &str = "deadlift_executions";

// TODO-- no pins are needed since everything is passed around
static NATS_CLIENT: OnceLock<Arc<RwLock<async_nats::Client>>> = OnceLock::new();

static WASM_MAP: LazyLock<Arc<RwLock<HashMap<String, String>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

pub async fn require_nats(config: &NatsConfig) -> Result<async_nats::Client> {
    if NATS_CLIENT.get().is_none() {
        let nc = config.connect().await?;

        if NATS_CLIENT.set(Arc::new(RwLock::new(nc))).is_err() {
            // log instead of return here?
            return Err(anyhow!("failed to initialize nats client"));
        }
    }

    if let Some(guard) = NATS_CLIENT.get() {
        // lifetime ? dont want to hold an extended lock somehow
        if let Ok(nc) = guard.try_read() {
            return Ok(nc.clone());
        }
    }

    Err(anyhow!("failed to get nats client"))
}

pub fn get_wasm_map() -> Arc<RwLock<HashMap<String, String>>> {
    WASM_MAP.clone()
}

pub async fn start_watcher_thread(
    module_object_names: Arc<std::collections::HashSet<String>>,
    nc: async_nats::Client,
    plugin: Arc<Mutex<Plugin>>,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        // FIXME------ this does not currently work
        let js = async_nats::jetstream::new(nc.clone());

        let wasm_store = get_or_create_object_store(&js, MODULE_BUCKET_NAME)
            .await
            .unwrap();

        let mut watcher = wasm_store.watch().await.unwrap();

        while let Some(res) = watcher.next().await {
            if let Ok(change) = res {
                // compare nats object digests
                if module_object_names.clone().contains(&change.name) {
                    let mut updated_modules = vec![];
                    for object_name in module_object_names.iter() {
                        let mut wasm_object = wasm_store.get(&object_name).await.unwrap();

                        let mut wasm_bytes = vec![];
                        wasm_object.read_to_end(&mut wasm_bytes).await.unwrap();

                        updated_modules.push(Wasm::Data {
                            data: wasm_bytes,
                            meta: extism::WasmMetadata::default(),
                        });
                    }

                    // TODO-- use util from plugin mod to do this
                    let updated_manifest = Manifest::new(updated_modules);

                    let updated_plugin = PluginBuilder::new(updated_manifest)
                        .with_wasi(true)
                        .build()
                        .unwrap();

                    let plugin_guard = plugin.clone();

                    {
                        let mut plugin = plugin_guard.lock().unwrap();
                        *plugin = updated_plugin;
                    }
                }
            }
        }
    })
}

pub async fn start_execution_thread(
    nc: async_nats::Client,
    plugin: Arc<Mutex<Plugin>>,
) -> tokio::task::JoinHandle<()> {
    tokio::task::spawn(async move {
        let mut subscriber = nc
            .queue_subscribe(
                "deadlift.executions.*",
                String::from(DEADLIFT_EXECUTIONS_QUEUE_GROUP),
            )
            .await
            .unwrap();

        while let Some(msg) = subscriber.next().await {
            let output = if let Ok(mut plugin) = plugin.try_lock() {
                let fn_name = msg.subject.as_str().split('.').last().unwrap();

                // should the last subject part be the fn_name?
                // it should have more connection to a workflow

                let res = plugin.call::<Vec<u8>, Vec<u8>>(fn_name, msg.payload.into());
                // reset memory ?

                match res {
                    Ok(bytes) => bytes,
                    Err(e) => e.to_string().into_bytes(),
                }
            } else {
                anyhow::anyhow!("failed to acquire plugin lock")
                    .to_string()
                    .into_bytes()
            };

            if let Some(reply_subject) = msg.reply {
                nc.publish(reply_subject, output.into()).await.unwrap();
            }
        }
    })
}
