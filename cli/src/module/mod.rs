use clap::{Args, Subcommand};

mod generate;
use generate::*;

mod publish;
use publish::*;

#[derive(Args)]
pub struct ModuleArgs {
    #[command(subcommand)]
    command: ModuleCommands,
}

#[derive(Subcommand)]
enum ModuleCommands {
    /// Generate a new deadlift module
    Generate(GenerateArgs),

    /// Publish a deadlift module
    Publish(PublishArgs),
}

pub async fn run_module_command(module_args: ModuleArgs) -> anyhow::Result<()> {
    match module_args.command {
        ModuleCommands::Generate(args) => run_generate_command(args).await,
        ModuleCommands::Publish(args) => run_publish_command(args).await,
    }
}
