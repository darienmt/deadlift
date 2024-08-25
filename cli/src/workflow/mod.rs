use clap::{Args, Subcommand};

mod generate;
use generate::*;

#[derive(Args)]
pub struct WorkflowArgs {
    #[command(subcommand)]
    command: WorkflowCommands,
}

#[derive(Subcommand)]
enum WorkflowCommands {
    /// Generate a new deadlift workflow
    Generate(GenerateArgs),
}

pub async fn run_workflow_command(module_args: WorkflowArgs) -> anyhow::Result<()> {
    match module_args.command {
        WorkflowCommands::Generate(args) => run_generate_command(args).await,
    }
}
