use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "sorter", version, about = "Background daemon that sorts downloaded files into category subfolders", long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Start the daemon, and keep it running across logins and crashes
    Start,
    /// Stop the daemon, and stop it starting at login
    Stop,
    /// Set the directory to sort
    Set {
        /// Directory to sort; the home directory and its parents are refused
        #[arg(long)]
        dir: String,
    },
    #[command(hide = true)]
    Run,
}