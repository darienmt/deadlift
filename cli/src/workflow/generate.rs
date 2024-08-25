use clap::Args;

use crate::utils::run_cmd_in_dir;

#[derive(Args)]
pub struct GenerateArgs {
    /// Workflow name and location
    name: String,

    /// Template repository tag
    #[arg(long, default_value_t = String::from("main"))]
    tag: String,
}

pub async fn run_generate_command(args: GenerateArgs) -> anyhow::Result<()> {
    let template_url = "https://github.com/zerosync-co/deadlift-workflow-template";

    run_cmd_in_dir(
        None,
        "git",
        &[
            "clone",
            "--depth=1",
            template_url,
            "--branch",
            &args.tag,
            "--recurse-submodules",
            &args.name,
        ],
    )?;

    run_cmd_in_dir(
        Some(&args.name),
        "git",
        &["checkout", "--orphan", "extism-init", "main"],
    )?;

    run_cmd_in_dir(Some(&args.name), "git", &["commit", "-am", "init: extism"])?;

    run_cmd_in_dir(
        Some(&args.name),
        "git",
        &["branch", "-M", "extism-init", "main"],
    )?;

    run_cmd_in_dir(Some(&args.name), "git", &["remote", "remove", "origin"])?;

    println!("successfully generated {} workflow", args.name);

    Ok(())
}
