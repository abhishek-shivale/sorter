use clap::Parser;

mod app;
mod config;
mod daemon;
mod error;
mod files;
mod paths;

fn main() -> Result<(), error::SorterError> {
    let application = app::Args::parse();

    let result = match application.command {
        app::Commands::Set { dir } => config::Config::set_dir(&paths::config_path(), &dir),
        app::Commands::Start => app::Commands::start(),
        app::Commands::Stop => daemon::stop(),
        app::Commands::Run => daemon::run(),
    };

    if let Err(problem) = result {
        eprintln!("{problem}");
        std::process::exit(1);
    } else {
        Ok(())
    }
}
