use std::io::Read;

use clap::Args;
use engine::{config::WorkflowConfig, MODULE_BUCKET_NAME, WORKFLOW_BUCKET_NAME};
use extism::*;
use tokio::io::AsyncReadExt;

#[derive(Args)]
pub struct CallArgs {
    /// Workflow name
    name: String,

    /// NATS server url
    #[arg(long, default_value_t = String::from("localhost:4222"))]
    nats_url: String,

    /// Raw string input; can also be passed from stdin
    #[arg(long)]
    input: Option<String>,
}

pub async fn run_call_command(args: CallArgs) -> anyhow::Result<()> {
    let input = match &args.input {
        Some(input) => input.clone(),
        None => {
            let mut buffer = String::new();
            std::io::stdin().read_to_string(&mut buffer)?;
            buffer
        }
    };

    let nc = async_nats::connect(&args.nats_url).await?;
    let js = async_nats::jetstream::new(nc);

    let workflow_store = js.get_object_store(WORKFLOW_BUCKET_NAME).await?;

    let mut workflow_config_object = workflow_store.get(&args.name).await?;

    let mut workflow_bytes = vec![];
    workflow_config_object
        .read_to_end(&mut workflow_bytes)
        .await?;

    let workflow = serde_yaml::from_slice::<WorkflowConfig>(workflow_bytes.as_slice())?;

    let module_store = js.get_object_store(MODULE_BUCKET_NAME).await?;

    let mut next_input = input;

    // FIXME-- call recursively with graph traversal
    for idx in workflow.graph.node_indices() {
        let stage = &workflow.graph[idx];

        let mut node_object = module_store.get(&stage.object_name).await?;

        let mut module_bytes = vec![];
        node_object.read_to_end(&mut module_bytes).await?;

        let data = Wasm::data(module_bytes);
        let manifest = Manifest::new([data]);

        // FIXME--
        // -- should use deadlift engine or agent here
        // -- caching
        // -- don't create new instance on each iteration
        let mut plugin = PluginBuilder::new(manifest).with_wasi(true).build()?;

        let result =
            plugin.call::<String, String>(&stage.plugin_function_name, next_input.clone())?;
        next_input = result;
    }

    println!(
        "successfully called {} workflow; result: {}",
        workflow.name, next_input
    );

    Ok(())
}
