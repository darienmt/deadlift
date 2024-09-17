use std::path::Path;
use std::process::{Command, Output, Stdio};

pub fn run_cmd_in_dir(dir: Option<&str>, name: &str, args: &[&str]) -> anyhow::Result<Output> {
    let mut cmd = Command::new(name);
    cmd.args(args);

    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    if let Some(d) = dir {
        cmd.current_dir(Path::new(d));
    }

    let output = cmd.spawn()?.wait_with_output()?;

    Ok(output)
}
