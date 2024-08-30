use clap::{Args, Subcommand};

mod create;
use create::*;

#[derive(Args)]
pub struct UserArgs {
    #[command(subcommand)]
    command: UserCommands,
}

#[derive(Subcommand)]
enum UserCommands {
    /// Create a new ZeroSync user account
    Create(CreateArgs),
}

pub async fn run_user_command(user_args: UserArgs) -> anyhow::Result<()> {
    match user_args.command {
        UserCommands::Create(args) => run_create_command(args).await,
    }
}
