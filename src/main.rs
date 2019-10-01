//! `git-crev` - `crev` ecosystem fronted for Git.
//!
#![cfg_attr(
    feature = "documentation",
    doc = "See [user documentation module](./doc/user/index.html)."
)]
#![cfg_attr(feature = "documentation", feature(external_doc))]
use self::prelude::*;

use crev::TrustOrDistrust::*;
use crev_data;
use crev_lib as crev;

use std::io::BufRead;
use structopt::StructOpt;

// #[cfg(feature = "documentation")]
// /// Documentation
// pub mod doc;

mod index;
mod local;
mod opts;
mod prelude;
mod shared;
mod term;

use crate::shared::*;

fn run_command(command: opts::Command) -> Result<()> {
    match command {
        opts::Command::Id(opts::Id::New(args)) => {
            let local = crev::Local::auto_create_or_open()?;
            let res = local.generate_id(args.url, args.github_username, args.use_https_push);
            if res.is_err() {
                eprintln!("Visit https://github.com/dpc/crev/wiki/Proof-Repository for help.");
            }
            let _ = crev::Local::auto_open()?;
            res?;
        }
        opts::Command::Id(opts::Id::Switch(args)) => {
            let local = crev::Local::auto_open()?;
            local.switch_id(&args.id)?
        }
        opts::Command::Id(opts::Id::Edit(args)) => match args {
            opts::Edit::Readme => {
                let local = crev::Local::auto_open()?;
                local.edit_readme()?;
            }
            opts::Edit::Config => {
                let local = crev::Local::auto_create_or_open()?;
                local.edit_user_config()?;
            }
        },
        opts::Command::Id(opts::Id::Show) => {
            let local = crev::Local::auto_open()?;
            local.show_own_ids()?;
        }
        opts::Command::Id(opts::Id::Trust(args)) => {
            create_trust_proof(args.pub_ids, Trust, &args.common_proof_create)?;
        }
        opts::Command::Id(opts::Id::Distrust(args)) => {
            create_trust_proof(args.pub_ids, Distrust, &args.common_proof_create)?;
        }
        opts::Command::Id(opts::Id::Export(params)) => {
            let local = crev::Local::auto_open()?;
            println!("{}", local.export_locked_id(params.id)?);
        }
        opts::Command::Id(opts::Id::Import) => {
            let local = crev::Local::auto_create_or_open()?;
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
        // opts::Command::Update(_) => {
        //     let local = crev::Local::auto_open()?;
        //     let status = local.run_git(vec!["pull".into(), "--rebase".into()])?;
        //     if !status.success() {
        //         std::process::exit(status.code().unwrap_or(-159));
        //     }
        // }
        opts::Command::Publish => {
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
        opts::Command::Fetch(cmd) => match cmd {
            opts::Fetch::Trusted(params) => {
                let local = crev::Local::auto_create_or_open()?;
                local.fetch_trusted(params.into())?;
            }
            opts::Fetch::Url(params) => {
                let local = crev::Local::auto_create_or_open()?;
                local.fetch_url(&params.url)?;
            }
            opts::Fetch::All => {
                let local = crev::Local::auto_create_or_open()?;
                local.fetch_all()?;
            }
        },
        opts::Command::Import(cmd) => match cmd {
            opts::Import::Proof(args) => {
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
        opts::Command::Add(args) => {
            let local = local::Local::auto_create_or_open()?;

            let trust_status = if args.trust {
                crev::TrustOrDistrust::Trust
            } else if args.distrust {
                crev::TrustOrDistrust::Distrust
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "One of --trust or --distrust must be specified.",
                ))?;
            };

            index::add_revision_specification_commits(&args.revision_range, &local, &trust_status)?;
        }
    }

    Ok(())
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
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e.display_causes_and_backtrace());
            std::process::exit(-2)
        }
    }
}
