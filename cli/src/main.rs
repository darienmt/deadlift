use clap::{Parser, Subcommand};

mod module;
use module::*;

/// deadlift
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct DeadliftArgs {
    #[command(subcommand)]
    command: DeadliftCommands,
}

#[derive(Subcommand)]
enum DeadliftCommands {
    /// Commands for interacting with deadlift modules
    Module(ModuleArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = DeadliftArgs::parse();

    match args.command {
        DeadliftCommands::Module(module_args) => run_module_command(module_args).await,
    }
}
