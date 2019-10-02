//! `git-crev` - `crev` ecosystem fronted for Git.
//!
#![cfg_attr(
    feature = "documentation",
    doc = "See [user documentation module](./doc/user/index.html)."
)]
#![cfg_attr(feature = "documentation", feature(external_doc))]
use self::prelude::*;

use crev_data;
use crev_lib as crev;

use std::io::BufRead;
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

use crate::shared::*;

fn run_command(command: commands::Command) -> Result<()> {
    match command {
        commands::Command::Id(subcommand) => {
            commands::id::run_command(subcommand)?;
        }
        commands::Command::Publish => {
            let local = crev::Local::auto_open()?;
            let mut status = local.run_git(vec!["diff".into(), "--exit-code".into()])?;

            if status.code().unwrap_or(-2) == 1 {
                status = local.run_git(vec![
                    "commit".into(),
                    "-a".into(),
                    "-m".into(),
                    "auto-commit on `crev publish`".into(),
                ])?;
            }

            if status.code().unwrap_or(-1) == 0 {
                status = local.run_git(vec!["pull".into(), "--rebase".into()])?;
            }
            if status.code().unwrap_or(-1) == 0 {
                status = local.run_git(vec!["push".into()])?;
            }
            std::process::exit(status.code().unwrap_or(-159));
        }
        commands::Command::Fetch(cmd) => match cmd {
            commands::Fetch::Trusted(params) => {
                let local = crev::Local::auto_create_or_open()?;
                local.fetch_trusted(params.into())?;
            }
            commands::Fetch::Url(params) => {
                let local = crev::Local::auto_create_or_open()?;
                local.fetch_url(&params.url)?;
            }
            commands::Fetch::All => {
                let local = crev::Local::auto_create_or_open()?;
                local.fetch_all()?;
            }
        },
        commands::Command::Import(cmd) => match cmd {
            commands::Import::Proof(args) => {
                let local = crev::Local::auto_create_or_open()?;
                let id = local.read_current_unlocked_id(&crev_common::read_passphrase)?;

                let s = load_stdin_with_prompt()?;
                let proofs = crev_data::proof::Proof::parse(s.as_slice())?;
                let commit_msg = "Import proofs";

                for proof in proofs {
                    let mut content = proof.content;
                    if args.reset_date {
                        content.set_date(&crev_common::now());
                    }
                    content.set_author(&id.as_pubid());
                    let proof = content.sign_by(&id)?;
                    maybe_store(&local, &proof, &commit_msg, &args.common)?;
                }
            }
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
