//! `git-crev` - `crev` ecosystem fronted for Git.
//!
#![cfg_attr(
    feature = "documentation",
    doc = "See [user documentation module](./doc/user/index.html)."
)]
#![cfg_attr(feature = "documentation", feature(external_doc))]
use self::prelude::*;

use structopt::StructOpt;

// #[cfg(feature = "documentation")]
// /// Documentation
// pub mod doc;

mod index;
mod local;
mod commands;
mod prelude;
mod shared;
mod term;

fn run_command(command: commands::Command) -> Result<()> {
    match command {
        commands::Command::Id(subcommand) => {
            commands::id::run_command(subcommand)?;
        }
        commands::Command::Publish => {
            commands::publish::run_command()?;
        }
        commands::Command::Fetch(subcommand) => {
            commands::fetch::run_command(subcommand)?;
        }
        commands::Command::Import(subcommand) => {
            commands::import::run_command(subcommand)?;
        },
        commands::Command::Add(args) => {
            commands::add::run_command(&args)?;
        }
    }

    Ok(())
}

fn main() {
    env_logger::init();
    let commands = commands::Opts::from_args();
    match run_command(commands.command) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e.display_causes_and_backtrace());
            std::process::exit(-2)
        }
    }
}
