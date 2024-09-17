use clap::{Args, Subcommand};

mod generate;
use generate::*;

mod publish;
use publish::*;

#[derive(Args)]
pub struct ProjectArgs {
    #[command(subcommand)]
    command: ProjectCommands,
}

#[derive(Subcommand)]
enum ProjectCommands {
    /// Generate a deadlift project
    Generate(GenerateArgs),

    /// Publish a deadlift project
    Publish(PublishArgs),
}

pub async fn run_project_command(project_args: ProjectArgs) -> anyhow::Result<()> {
    match project_args.command {
        ProjectCommands::Generate(args) => run_generate_command(args).await,
        ProjectCommands::Publish(args) => run_publish_command(args).await,
    }
}
