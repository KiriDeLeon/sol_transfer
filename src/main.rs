use anyhow::Result;
mod util;
use util::read_config::read_config;
use util::transfer::token_transfers;

#[tokio::main]
async fn main() -> Result<()> {
    let config = read_config("data/config.yaml")?;
    token_transfers(config).await?;

    Ok(())
}
