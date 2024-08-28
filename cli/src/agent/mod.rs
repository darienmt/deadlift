use clap::{Args, Subcommand};

mod start;
use start::*;

#[derive(Args)]
pub struct AgentArgs {
    #[command(subcommand)]
    command: AgentCommands,
}

#[derive(Subcommand)]
enum AgentCommands {
    /// Start a deadlift agent
    Start(StartArgs),
}

pub async fn run_agent_command(agent_args: AgentArgs) -> anyhow::Result<()> {
    match agent_args.command {
        AgentCommands::Start(args) => run_start_command(args).await,
    }
}
