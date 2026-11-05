use std::path::Path;

use clap::Parser;

mod app;
mod error;
mod files;
mod paths;

fn main() -> Result<(), error::SorterError> {
    let application = app::Args::parse();

    let result = match application.command {
        app::Commands::Set { dir } => files::read_file(Path::new(&dir)),
        app::Commands::Start => todo!("start"),
        app::Commands::Stop => todo!("stop"),
    };

    if let Err(problem) = result {
        eprintln!("{problem}");
        std::process::exit(1);
    } else {
        Ok(())
    }
}
