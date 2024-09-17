use clap::Args;
use engine::{
    config::{NatsConfig, WorkflowConfig},
    utils::get_or_create_object_store,
    MODULE_BUCKET_NAME, WORKFLOW_BUCKET_NAME,
};
use tokio::io::AsyncReadExt;

use crate::utils::run_cmd_in_dir;

#[derive(Args)]
pub struct PublishArgs {
    // TODO--
    // /// Optional list of module names and paths
    // #[arg(long)]
    // module_data: Option<Vec<ModuleData>>,
    //
    // /// Path to deadlift project directory
    // #[arg(long, default_value_t = String::from(".")))]
    // path: String
    #[command(flatten)]
    nats_config: NatsConfig,
}

pub async fn run_publish_command(args: PublishArgs) -> anyhow::Result<()> {
    let module_data = compile_rust_project()?; // TODO-- handle more languages

    let nc = args.nats_config.connect().await?;
    let js = async_nats::jetstream::new(nc);

    for data in module_data {
        // TODO-- validation

        let wasm_store = get_or_create_object_store(&js, MODULE_BUCKET_NAME).await?;

        let mut file = tokio::fs::File::open(data.wasm_path).await?;
        wasm_store.put(data.name.as_str(), &mut file).await?;

        println!("successfully published {} module", data.name);
    }

    // TODO-- allow user to pass workflow.yml path

    let mut file = tokio::fs::File::open("./workflow.yml").await?;

    let mut workflow_bytes = vec![];
    file.read_to_end(&mut workflow_bytes).await?;

    let workflow = serde_yaml::from_slice::<WorkflowConfig>(workflow_bytes.as_slice())?;

    let nc = args.nats_config.connect().await?;
    let js = async_nats::jetstream::new(nc);

    let workflow_store = get_or_create_object_store(&js, WORKFLOW_BUCKET_NAME).await?;

    workflow_store
        .put(workflow.name.as_str(), &mut workflow_bytes.as_slice())
        .await?;

    println!("successfully published project modules and workflow");

    Ok(())
}

struct ModuleData {
    name: String,
    wasm_path: String,
}

fn compile_rust_project() -> anyhow::Result<Vec<ModuleData>> {
    let build_output = run_cmd_in_dir(
        None,
        "cargo",
        &["build", "--release", "--message-format=json"],
    )?;

    if !build_output.status.success() {
        return Err(anyhow::anyhow!(
            "failed to publish; build output status: {}",
            build_output.status
        ));
    }

    let mut module_data = vec![];

    let mut start = 0;
    for (i, &byte) in build_output.stdout.iter().enumerate() {
        if byte == b'\n' {
            let line = &build_output.stdout[start..i];
            let json_line = serde_json::from_slice::<serde_json::Value>(line)?;

            let is_module_output = json_line["target"]
                .as_object()
                .and_then(|target| target["kind"].as_array())
                .and_then(|kinds| kinds.iter().find(|&kind| kind.as_str() == Some("cdylib")))
                .is_some();

            if is_module_output {
                let wasm_path = json_line["filenames"]
                    .as_array()
                    .and_then(|filenames| filenames.first())
                    .and_then(|filename| filename.as_str())
                    .ok_or(anyhow::anyhow!("failed to resolve filename"))?
                    .to_string();

                let name = json_line["target"]
                    .as_object()
                    .and_then(|target| target["name"].as_str())
                    .ok_or(anyhow::anyhow!("failed to resolve target name"))?
                    .to_string();

                module_data.push(ModuleData { name, wasm_path });
            }

            start = i + 1;
        }
    }

    Ok(module_data)
}
