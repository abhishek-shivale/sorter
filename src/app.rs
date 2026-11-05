use clap::{Parser, Subcommand};

use crate::daemon;
use crate::error::SorterError;

#[derive(Debug, Parser)]
#[command(name = "Sorter", version, about = "Rust based sorter for sorting files in sub directory", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    Start,
    Stop,
    Set {
        #[arg(long)]
        dir: String,
    },
    #[command(hide = true)]
    Run,
}

impl Commands {
    pub fn start() -> Result<(), SorterError> {
        daemon::start()
    }
}