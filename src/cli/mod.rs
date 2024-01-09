use anyhow::Result;

use crate::{args::Commands, pikpak::Client};

mod download;
mod list;

pub async fn handle(cmd: Commands) -> Result<()> {
    let mut client = Client::new()?;
    client.login().await?;

    match cmd {
        Commands::Download {
            path,
            output,
            parallel,
        } => todo!(),
        Commands::List { long, human, path } => client.list(long, human, path).await,
    }
}
