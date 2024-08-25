use clap::{Args, Subcommand};

mod generate;
use generate::*;

#[derive(Args)]
pub struct ModuleArgs {
    #[command(subcommand)]
    command: ModuleCommands,
}

#[derive(Subcommand)]
enum ModuleCommands {
    /// Generate a new deadlift module
    Generate(GenerateArgs),
}

pub async fn run_module_command(module_args: ModuleArgs) -> anyhow::Result<()> {
    match module_args.command {
        ModuleCommands::Generate(args) => run_generate_command(args).await,
    }
}
