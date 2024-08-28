use clap::{Parser, Subcommand};

mod module;
use module::*;

mod workflow;
use workflow::*;

pub mod utils;

mod agent;
use agent::*;

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

    /// Commands for interacting with deadlift workflows
    Workflow(WorkflowArgs),

    /// Commands for interacting with deadlift agents
    Agent(AgentArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = DeadliftArgs::parse();

    match args.command {
        DeadliftCommands::Module(module_args) => run_module_command(module_args).await,
        DeadliftCommands::Workflow(module_args) => run_workflow_command(module_args).await,
        DeadliftCommands::Agent(agent_args) => run_agent_command(agent_args).await,
    }
}
