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

pub async fn get_or_create_object_store(
    js: &async_nats::jetstream::Context,
    bucket_name: &str,
) -> anyhow::Result<async_nats::jetstream::object_store::ObjectStore> {
    match js.get_object_store(bucket_name).await {
        Ok(store) => Ok(store),
        Err(e) => {
            if e.kind() == async_nats::jetstream::context::ObjectStoreErrorKind::GetStore {
                js.create_object_store(async_nats::jetstream::object_store::Config {
                    bucket: bucket_name.to_string(),
                    ..Default::default()
                })
                .await
                .map_err(anyhow::Error::from)
            } else {
                Err(anyhow::Error::from(e))
            }
        }
    }
}
