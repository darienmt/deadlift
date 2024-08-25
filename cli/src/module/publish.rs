use clap::Args;

#[derive(Args)]
pub struct PublishArgs {
    /// Module name
    name: String,

    /// Module .wasm path
    #[arg(long)]
    path: String,

    /// NATS server url
    #[arg(long, default_value_t = String::from("localhost:4222"))]
    nats_url: String,
}

pub async fn run_publish_command(args: PublishArgs) -> anyhow::Result<()> {
    let nc = async_nats::connect(&args.nats_url).await?;
    let js = async_nats::jetstream::new(nc);

    let wasm_store = js.get_object_store("wasm").await?;

    let mut file = tokio::fs::File::open(args.path).await?;
    wasm_store.put(args.name.as_str(), &mut file).await?;

    println!("successfully published {} module", args.name);

    Ok(())
}