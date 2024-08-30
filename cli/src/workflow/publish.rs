use clap::Args;
use engine::config::{NatsConfig, WorkflowConfig};
use tokio::io::AsyncReadExt;

use engine::WORKFLOW_BUCKET_NAME;

use engine::utils::get_or_create_object_store;

#[derive(Args)]
pub struct PublishArgs {
    /// Workflow YAML path
    path: String,

    #[command(flatten)]
    nats_config: NatsConfig,
}

pub async fn run_publish_command(args: PublishArgs) -> anyhow::Result<()> {
    let mut file = tokio::fs::File::open(args.path).await?;

    let mut workflow_bytes = vec![];
    file.read_to_end(&mut workflow_bytes).await?;

    let workflow = serde_yaml::from_slice::<WorkflowConfig>(workflow_bytes.as_slice())?;

    let nc = args.nats_config.connect().await?;
    let js = async_nats::jetstream::new(nc);

    let wasm_store = get_or_create_object_store(&js, WORKFLOW_BUCKET_NAME).await?;

    wasm_store
        .put(workflow.name.as_str(), &mut workflow_bytes.as_slice())
        .await?;

    println!("successfully published {} workflow", workflow.name);

    Ok(())
}
