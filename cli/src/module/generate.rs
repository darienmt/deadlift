use clap::Args;
use serde::Deserialize;

use std::path::Path;
use std::process::{Command, Stdio};

// from https://github.com/extism/cli/blob/main/generate.go

#[derive(Args)]
pub struct GenerateArgs {
    /// Module name and location
    name: String,

    /// Module source language
    #[arg(long, default_value_t = String::from("rust"))]
    lang: String,

    /// Template repository tag
    #[arg(long, default_value_t = String::from("main"))]
    tag: String,
}

#[derive(Deserialize)]
struct TemplateData {
    name: String,
    url: String,
}

pub async fn run_generate_command(args: GenerateArgs) -> anyhow::Result<()> {
    let template_url = get_template_url(&args.lang).await?;

    run_cmd_in_dir(
        None,
        "git",
        &[
            "clone",
            "--depth=1",
            &template_url,
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

    println!("successfully generated {} module", args.name);

    Ok(())
}

async fn get_template_url(lang: &str) -> anyhow::Result<String> {
    let templates_data =
        reqwest::get("https://raw.githubusercontent.com/extism/cli/main/pdk-templates.json")
            .await?
            .json::<Vec<TemplateData>>()
            .await?;

    let available_langs_str = templates_data
        .iter()
        .map(|v| v.name.clone())
        .collect::<Vec<String>>()
        .join(", ");

    let template_data = templates_data
        .into_iter()
        .find(|v| v.name.to_lowercase() == lang.to_lowercase())
        .ok_or(anyhow::anyhow!(format!(
            "unsupported template: '{}'. Supported templates are: {}",
            lang, available_langs_str
        )))?;

    Ok(template_data.url)
}

fn run_cmd_in_dir(dir: Option<&str>, name: &str, args: &[&str]) -> anyhow::Result<()> {
    let mut cmd = Command::new(name);
    cmd.args(args);

    cmd.stdout(Stdio::inherit());
    cmd.stderr(Stdio::inherit());

    if let Some(d) = dir {
        cmd.current_dir(Path::new(d));
    }

    cmd.spawn()?.wait()?;

    Ok(())
}