use clap::{Parser, Subcommand};

pub mod utils;

mod agent;
use agent::*;

mod user;
use user::*;

mod project;
use project::*;

/// deadlift
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct DeadliftArgs {
    #[command(subcommand)]
    command: DeadliftCommands,
}

#[derive(Subcommand)]
enum DeadliftCommands {
    /// Commands for interacting with deadlift agents
    Agent(AgentArgs),

    /// Commands for interacting with your ZeroSync user account
    User(UserArgs),

    /// Command for interacting with deadlift source projects
    Project(ProjectArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = DeadliftArgs::parse();

    match args.command {
        DeadliftCommands::Agent(agent_args) => run_agent_command(agent_args).await,
        DeadliftCommands::User(user_args) => run_user_command(user_args).await,
        DeadliftCommands::Project(project_args) => run_project_command(project_args).await,
    }
}
