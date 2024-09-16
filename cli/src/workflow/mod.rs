use clap::{Args, Subcommand};

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
    /// Publish a deadlift workflow
    Publish(PublishArgs),

    /// Call a deadlift workflow
    Call(CallArgs),
}

pub async fn run_workflow_command(module_args: WorkflowArgs) -> anyhow::Result<()> {
    match module_args.command {
        WorkflowCommands::Publish(args) => run_publish_command(args).await,
        WorkflowCommands::Call(args) => run_call_command(args).await,
    }
}
