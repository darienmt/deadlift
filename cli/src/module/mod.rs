use clap::{Args, Subcommand};

mod publish;
use publish::*;

#[derive(Args)]
pub struct ModuleArgs {
    #[command(subcommand)]
    command: ModuleCommands,
}

#[derive(Subcommand)]
enum ModuleCommands {
    /// Publish a deadlift module
    Publish(PublishArgs),
}

pub async fn run_module_command(module_args: ModuleArgs) -> anyhow::Result<()> {
    match module_args.command {
        ModuleCommands::Publish(args) => run_publish_command(args).await,
    }
}
