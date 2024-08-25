use clap::{Args, Subcommand};

mod generate;
use generate::*;

mod publish;
use publish::*;

mod call;
use call::*;

#[derive(Args)]
pub struct WorkflowArgs {
    #[command(subcommand)]
    command: WorkflowCommands,
}

#[derive(Subcommand)]
enum WorkflowCommands {
    /// Generate a new deadlift workflow
    Generate(GenerateArgs),

    /// Publish a deadlift workflow
    Publish(PublishArgs),

    /// Call a deadlift workflow
    Call(CallArgs),
}

pub async fn run_workflow_command(module_args: WorkflowArgs) -> anyhow::Result<()> {
    match module_args.command {
        WorkflowCommands::Generate(args) => run_generate_command(args).await,
        WorkflowCommands::Publish(args) => run_publish_command(args).await,
        WorkflowCommands::Call(args) => run_call_command(args).await,
    }
}
