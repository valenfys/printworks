mod cli;
mod convert;
mod develop;
mod discover;
mod formats;
mod info;
mod metadata;

use clap::Parser;
use cli::{Cli, Command};

fn main() {
    let cli = Cli::parse();

    let level = if cli.quiet {
        log::LevelFilter::Error
    } else {
        match cli.verbose {
            0 => log::LevelFilter::Warn,
            1 => log::LevelFilter::Info,
            _ => log::LevelFilter::Debug,
        }
    };
    env_logger::Builder::new().filter_level(level).init();

    let result = match &cli.command {
        Command::Convert(args) => convert::run(args),
        Command::Info(args) => info::run(args),
    };

    if let Err(e) = result {
        log::error!("{e:#}");
        std::process::exit(1);
    }
}
