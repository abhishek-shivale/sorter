use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    pub dir: PathBuf,

    #[arg(long)]
    pub dry_run: bool,
}
