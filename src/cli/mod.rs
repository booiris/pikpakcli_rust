use anyhow::Result;

use crate::{args::Commands, pikpak::Client};

mod download;
mod list;

pub async fn handle(cmd: Commands, retry_times: i8) -> Result<()> {
    let mut client = Client::new(retry_times)?;
    client.login().await?;

    match cmd {
        Commands::Download {
            paths,
            output,
            parallel,
        } => client.download(paths, output, parallel).await,
        Commands::List { long, human, path } => client.list(long, human, path).await,
    }
}
