//! `git-crev` - `crev` ecosystem fronted for Git.
//!
#![cfg_attr(
    feature = "documentation",
    doc = "See [user documentation module](./doc/user/index.html)."
)]
#![cfg_attr(feature = "documentation", feature(external_doc))]
use self::prelude::*;

use crev_common::convert::OptionDeref;
use crev_lib::{self, local::Local};
use std::io::BufRead;
use structopt::StructOpt;

#[cfg(feature = "documentation")]
/// Documentation
pub mod doc;

mod opts;
mod prelude;
mod shared;
mod term;

use crate::shared::*;
use crev_lib::TrustOrDistrust::*;

fn run_command(command: opts::Command) -> Result<CommandExitStatus> {
    match command {
        opts::Command::Id(opts::Id::New(args)) => {
            let local = Local::auto_create_or_open()?;
            let res = local.generate_id(args.url, args.github_username, args.use_https_push);
            if res.is_err() {
                eprintln!("Visit https://github.com/dpc/crev/wiki/Proof-Repository for help.");
            }
            let _ = crev_lib::Local::auto_open()?;
            res?;
        }
        opts::Command::Id(opts::Id::Switch(args)) => {
            let local = Local::auto_open()?;
            local.switch_id(&args.id)?
        }
        opts::Command::Id(opts::Id::Edit(args)) => match args {
            opts::Edit::Readme => {
                let local = crev_lib::Local::auto_open()?;
                local.edit_readme()?;
            }
            opts::Edit::Config => {
                let local = crev_lib::Local::auto_create_or_open()?;
                local.edit_user_config()?;
            }
        },
        opts::Command::Id(opts::Id::Show) => {
            let local = Local::auto_open()?;
            local.show_own_ids()?;
        }
        opts::Command::Id(opts::Id::Trust(args)) => {
            create_trust_proof(args.pub_ids, Trust, &args.common_proof_create)?;
        }
        opts::Command::Distrust(args) => {
            create_trust_proof(args.pub_ids, Distrust, &args.common_proof_create)?;
        }
        opts::Command::Update(_) => {
            let local = Local::auto_open()?;
            let status = local.run_git(vec!["pull".into(), "--rebase".into()])?;
            if !status.success() {
                std::process::exit(status.code().unwrap_or(-159));
            }
        }
        opts::Command::Query(cmd) => match cmd {
            opts::Query::Id(cmd) => match cmd {
                opts::QueryId::Current => {
                    let local = Local::auto_open()?;
                    local.show_current_id()?
                }
                opts::QueryId::Own => {
                    let local = Local::auto_open()?;
                    local.list_own_ids()?
                }
                opts::QueryId::Trusted {
                    for_id,
                    trust_params,
                } => {
                    let local = crev_lib::Local::auto_open()?;
                    let db = local.load_db()?;
                    let for_id = local.get_for_id_from_str(for_id.as_deref())?;
                    let trust_set = db.calculate_trust_set(&for_id, &trust_params.into());

                    for id in trust_set.trusted_ids() {
                        println!(
                            "{} {:6} {}",
                            id,
                            trust_set
                                .get_effective_trust_level(id)
                                .expect("Some trust level"),
                            db.lookup_url(id).map(|url| url.url.as_str()).unwrap_or("")
                        );
                    }
                }
                // TODO: move to crev-lib
                opts::QueryId::All => {
                    let local = crev_lib::Local::auto_create_or_open()?;
                    let db = local.load_db()?;

                    for id in &db.all_known_ids() {
                        println!(
                            "{} {}",
                            id,
                            db.lookup_url(id).map(|url| url.url.as_str()).unwrap_or("")
                        );
                    }
                }
            },
        },
        opts::Command::Publish => {
            let local = Local::auto_open()?;
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
        opts::Command::Fetch(cmd) => match cmd {
            opts::Fetch::Trusted(params) => {
                let local = Local::auto_create_or_open()?;
                local.fetch_trusted(params.into())?;
            }
            opts::Fetch::Url(params) => {
                let local = Local::auto_create_or_open()?;
                local.fetch_url(&params.url)?;
            }
            opts::Fetch::All => {
                let local = Local::auto_create_or_open()?;
                local.fetch_all()?;
            }
        },
        opts::Command::Id(opts::Id::Export(params)) => {
            let local = Local::auto_open()?;
            println!("{}", local.export_locked_id(params.id)?);
        }
        opts::Command::Id(opts::Id::Import) => {
            let local = Local::auto_create_or_open()?;
            let s = load_stdin_with_prompt()?;
            let id = local.import_locked_id(&String::from_utf8(s)?)?;
            // Note: It's unclear how much of this should be done by
            // the library
            local.save_current_id(&id.id)?;

            let proof_dir_path = local.get_proofs_dir_path_for_url(&id.url)?;
            if !proof_dir_path.exists() {
                local.clone_proof_dir_from_git(&id.url.url, false)?;
            }
        }
        opts::Command::Import(cmd) => match cmd {
            opts::Import::Proof(args) => {
                let local = Local::auto_create_or_open()?;
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
    }

    Ok(CommandExitStatus::Success)
}

fn load_stdin_with_prompt() -> Result<Vec<u8>> {
    let term = term::Term::new();

    if term.stdin_is_tty {
        eprintln!("Paste in the text and press Ctrl+D.")
    }
    let mut s = vec![];

    std::io::stdin().lock().read_until(0, &mut s)?;
    Ok(s)
}

fn main() {
    env_logger::init();
    let opts = opts::Opts::from_args();
    match run_command(opts.command) {
        Ok(CommandExitStatus::Success) => {}
        Ok(CommandExitStatus::VerificationFailed) => std::process::exit(-1),
        Err(e) => {
            eprintln!("{}", e.display_causes_and_backtrace());
            std::process::exit(-2)
        }
    }
}
