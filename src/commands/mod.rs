use crev_data::Level;
use structopt::StructOpt;

use crate::prelude::*;

mod add;
mod fetch;
pub mod id;
mod import;
mod publish;

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
    }

    Ok(())
}

/// Verification Requirements
#[derive(Debug, StructOpt, Clone, Default)]
pub struct VerificationRequirements {
    /// Minimum trust level of the reviewers for reviews
    #[structopt(long = "trust", default_value = "low")]
    pub trust_level: crev_data::Level,

    /// Number of reviews required
    #[structopt(long = "redundancy", default_value = "1")]
    pub redundancy: u64,
    /// Required understanding
    #[structopt(long = "understanding", default_value = "none")]
    pub understanding_level: Level,
    /// Required thoroughness
    #[structopt(long = "thoroughness", default_value = "none")]
    pub thoroughness_level: Level,
}

impl From<VerificationRequirements> for crev_lib::VerificationRequirements {
    fn from(req: VerificationRequirements) -> Self {
        crev_lib::VerificationRequirements {
            trust_level: req.trust_level,
            redundancy: req.redundancy,
            understanding: req.understanding_level,
            thoroughness: req.thoroughness_level,
        }
    }
}

#[derive(Debug, StructOpt, Clone, Default)]
pub struct Update {}

#[derive(Debug, Clone, Default)]
pub struct AdviseCommon {
    /// This release contains advisory (important fix)
    pub affected: crev_data::proof::review::package::VersionRange,
    pub severity: Level,
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
}

#[derive(Debug, StructOpt, Clone)]
#[structopt(about = "Distributed code review system")]
#[structopt(raw(global_setting = "structopt::clap::AppSettings::ColoredHelp"))]
#[structopt(raw(global_setting = "structopt::clap::AppSettings::DeriveDisplayOrder"))]
pub struct Opts {
    #[structopt(subcommand)]
    pub command: Command,
}
