use std::io::Read;

use clap::Args;
use engine::config::NatsConfig;

#[derive(Args)]
pub struct CallArgs {
    /// WASM function name
    #[arg(long)]
    fn_name: String,

    /// Raw string input; can also be passed from stdin
    #[arg(long)]
    input: Option<String>,

    #[command(flatten)]
    nats_config: NatsConfig,
}

pub async fn run_call_command(args: CallArgs) -> anyhow::Result<()> {
    let input = match args.input {
        Some(input) => input.as_bytes().to_vec(),
        None => {
            let mut buffer = vec![];
            std::io::stdin().read_to_end(&mut buffer)?;
            buffer
        }
    };

    let nc = args.nats_config.connect().await?;

    let req = async_nats::Request::new()
        .inbox(format!("deadlift.executions.{}.reply", args.fn_name))
        .payload(input.into());

    let response = nc
        .send_request(format!("deadlift.executions.{}", args.fn_name), req)
        .await?;

    let response_payload: Vec<u8> = response.payload.into();

    println!(
        "successfully called {}; response: {}",
        args.fn_name,
        String::from_utf8_lossy(&response_payload)
    );

    Ok(())
}
