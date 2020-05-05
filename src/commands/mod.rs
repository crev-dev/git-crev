use structopt::StructOpt;

use crate::prelude::*;

mod add;
mod fetch;
mod id;
mod import;
mod publish;
mod status;

pub fn run_command(command: Command) -> Result<()> {
    match command {
        Command::Id(subcommand) => {
            id::run_command(subcommand)?;
        }
        Command::Publish => {
            publish::run_command()?;
        }
        Command::Fetch(subcommand) => {
            fetch::run_command(subcommand)?;
        }
        Command::Import(subcommand) => {
            import::run_command(subcommand)?;
        }
        Command::Add(args) => {
            add::run_command(&args)?;
        }
        Command::Status => {
            status::run_command()?;
        }
    }

    Ok(())
}

#[derive(Debug, StructOpt, Clone)]
pub enum Command {
    /// Manage IDs
    #[structopt(name = "id", alias = "new")]
    Id(id::Id),

    /// Fetch proofs from external sources
    #[structopt(name = "fetch")]
    Fetch(fetch::Fetch),

    // TODO: should rename publish to push?
    /// Publish local changes to the public proof repository
    #[structopt(name = "publish", alias = "push")]
    Publish,

    /// Import proofs
    #[structopt(name = "import")]
    Import(import::Import),

    /// Stage commits for review
    #[structopt(name = "add")]
    Add(add::Add),

    #[structopt(name = "status")]
    Status,
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "Distributed code review system")]
#[structopt(raw(global_setting = "structopt::clap::AppSettings::ColoredHelp"))]
#[structopt(raw(global_setting = "structopt::clap::AppSettings::DeriveDisplayOrder"))]
pub struct Opts {
    #[structopt(subcommand)]
    pub command: Command,
}
