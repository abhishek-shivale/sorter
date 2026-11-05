use clap::{Parser, Subcommand};

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