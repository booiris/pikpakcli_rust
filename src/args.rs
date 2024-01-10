use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    pub debug: bool,

    #[arg(short, long, action = clap::ArgAction::Set,default_value_t=0,help = "net error retry times, if it is set to negative, it will infinitely retry")]
    pub retry: i8,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Download a dir or a file", visible_alias = "d")]
    Download {
        #[arg(help = "specify path")]
        path: String,
        #[arg(short, long, default_value_t = String::from("./"), help = "output directory")]
        output: String,
        #[arg(short, long, default_value_t = 4, help = "download parallel count")]
        parallel: usize,
    },

    #[command(about = "List file", visible_alias = "ls")]
    List {
        #[arg(short, long, action = clap::ArgAction::SetTrue, help="display long format")]
        long: bool,
        #[arg(short='H', long, action = clap::ArgAction::SetTrue, help="display human readable format")]
        human: bool,
        #[arg(help = "specify path", default_value_t = String::from("/"))]
        path: String,
    },
}

pub fn parse_cli() -> Cli {
    Cli::parse()
}

pub fn print_cli_help() -> Result<()> {
    Ok(Cli::command().print_help()?)
}
