use clap::Args;
use engine::config::WorkflowConfig;
use tokio::io::AsyncReadExt;

#[derive(Args)]
pub struct PublishArgs {
    /// Workflow YAML path
    path: String,

    /// NATS server url
    #[arg(long, default_value_t = String::from("localhost:4222"))]
    nats_url: String,
}

pub async fn run_publish_command(args: PublishArgs) -> anyhow::Result<()> {
    let mut file = tokio::fs::File::open(args.path).await?;

    let mut workflow_bytes = vec![];
    file.read_to_end(&mut workflow_bytes).await?;

    let workflow = serde_yaml::from_slice::<WorkflowConfig>(workflow_bytes.as_slice())?;

    let nc = async_nats::connect(&args.nats_url).await?;
    let js = async_nats::jetstream::new(nc);

    let wasm_store = js.get_object_store("workflow").await?;

    wasm_store.put(workflow.name.as_str(), &mut file).await?;

    println!("successfully published {} workflow", workflow.name);

    Ok(())
}
