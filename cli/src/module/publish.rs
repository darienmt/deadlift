use clap::Args;
use engine::config::NatsConfig;
use engine::MODULE_BUCKET_NAME;

use engine::utils::get_or_create_object_store;

#[derive(Args)]
pub struct PublishArgs {
    /// Module name
    name: String,

    /// Module .wasm path
    #[arg(long)]
    path: String,

    #[command(flatten)]
    nats_config: NatsConfig,
}

pub async fn run_publish_command(args: PublishArgs) -> anyhow::Result<()> {
    let nc = args.nats_config.connect().await?;
    let js = async_nats::jetstream::new(nc);

    let wasm_store = get_or_create_object_store(&js, MODULE_BUCKET_NAME).await?;

    let mut file = tokio::fs::File::open(args.path).await?;
    wasm_store.put(args.name.as_str(), &mut file).await?;

    println!("successfully published {} module", args.name);

    Ok(())
}
