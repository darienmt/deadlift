use std::io::Write;

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use clap::Args;
use directories::ProjectDirs;
use engine::config::NatsConfig;

#[derive(Args)]
pub struct CreateArgs {
    /// User email
    #[arg(long)]
    email: String,

    /// User password
    #[arg(long)]
    password: String,

    #[command(flatten)]
    nats_config: NatsConfig,
}

const CLI_CREATE_USER_SUBJECT: &str = "deadlift.executions.cli_create_user";
const CLI_CREATE_USER_REPLY_SUBJECT: &str = "deadlift.executions.cli_create_user.reply";

pub async fn run_create_command(args: CreateArgs) -> anyhow::Result<()> {
    let nc = args.nats_config.connect().await?;

    let key = Aes256Gcm::generate_key(OsRng);
    let payload = get_create_user_payload_bytes(&args.email, &args.password, key.to_vec());

    let req = async_nats::Request::new()
        .inbox(String::from(CLI_CREATE_USER_REPLY_SUBJECT))
        .payload(payload.into());

    let response = nc.send_request(CLI_CREATE_USER_SUBJECT, req).await?;
    let response_buf: Vec<u8> = response.payload.into();

    // TODO-- check if response_buf is error by wrapping response str in Ok() or Err()

    let cipher = Aes256Gcm::new(&key);
    let user_creds_buf = cipher
        .decrypt(&Nonce::default(), response_buf.as_slice())
        .map_err(|_| anyhow::anyhow!("failed to decrypt user creds"))?;

    let user_creds = String::from_utf8_lossy(user_creds_buf.as_slice());

    let proj_dirs = ProjectDirs::from("com", "ZeroSync", "deadlift").unwrap(); // TODO-- no unwrap, move to util for reusability, check if config should be stored here or something cargo related
    let creds_path = proj_dirs.config_dir().join("user.creds");
    if let Some(parent) = creds_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let mut creds_file = std::fs::File::create(creds_path)?;
    creds_file.write_all(user_creds.as_bytes())?;

    println!("successfully created {} user", args.email);

    Ok(())
}

fn get_create_user_payload_bytes(email: &str, password: &str, key: Vec<u8>) -> Vec<u8> {
    format!(
        r#"{{
            "email": "{}",
            "password": "{}",
            "key": "{}"
        }}"#,
        email,
        password,
        hex::encode(key)
    )
    .as_bytes()
    .to_vec()
}
