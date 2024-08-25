use std::path::Path;
use std::process::{Command, Stdio};

pub fn run_cmd_in_dir(dir: Option<&str>, name: &str, args: &[&str]) -> anyhow::Result<()> {
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
