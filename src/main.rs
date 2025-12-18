mod capture;
mod cli;
mod config;
mod event;

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    cli::run().await
}
