use anyhow::{Ok, Result};
use args::*;
use cli::handle;
use config::{get_config, load_config};
use log::*;
use logger::setup_logger;

mod args;
mod cli;
mod config;
mod logger;
mod pikpak;
mod utils;

pub fn run_cmd() -> Result<()> {
    let cli = parse_cli();

    if cli.command.is_none() {
        print_cli_help()?;
        return Ok(());
    }

    load_config("config.yml")?;
    let log_path = get_config().log_path.as_str();
    if cli.debug {
        setup_logger(LevelFilter::Debug, log_path)?;
    } else {
        setup_logger(LevelFilter::Error, log_path)?;
    }

    if let Some(x) = cli.command {
        let rt = tokio::runtime::Runtime::new()?;
        rt.block_on(async { handle(x).await })?;
    }

    Ok(())
}
