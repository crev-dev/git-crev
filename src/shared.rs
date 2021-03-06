// Miscellaneous structures and functions.
//
use std::io::BufRead;

use structopt::StructOpt;
use crate::prelude::*;
use crev_lib::TrustOrDistrust;
use crev_lib::{self, local::Local, ProofStore};
use failure::format_err;
use serde::Deserialize;

use crate::term;

pub fn load_stdin_with_prompt() -> Result<Vec<u8>> {
    let term = term::Term::new();
    if term.stdin_is_tty {
        eprintln!("Paste in the text and press Ctrl+D.")
    }
    let mut s = vec![];
    std::io::stdin().lock().read_until(0, &mut s)?;
    Ok(s)
}

/// Data from `.cargo_vcs_info.json`
#[derive(Debug, Clone, Deserialize)]
pub struct VcsInfoJson {
    git: VcsInfoJsonGit,
}

#[derive(Debug, Clone, Deserialize)]
pub enum VcsInfoJsonGit {
    #[serde(rename = "sha1")]
    Sha1(String),
}

#[derive(Debug, StructOpt, Clone)]
pub struct CommonProofCreate {
    /// Don't auto-commit local Proof Repository
    #[structopt(long = "no-commit")]
    pub no_commit: bool,

    /// Print unsigned proof content on stdout
    #[structopt(long = "print-unsigned")]
    pub print_unsigned: bool,

    /// Print signed proof content on stdout
    #[structopt(long = "print-signed")]
    pub print_signed: bool,

    /// Don't store the proof
    #[structopt(long = "no-store")]
    pub no_store: bool,
}

pub fn create_trust_proof(
    ids: Vec<String>,
    trust_or_distrust: TrustOrDistrust,
    proof_create_opt: &CommonProofCreate,
) -> Result<()> {
    let local = Local::auto_open()?;

    let own_id = local.read_current_unlocked_id(&crev_common::read_passphrase)?;

    let trust = local.build_trust_proof(own_id.as_pubid(), ids.clone(), trust_or_distrust)?;

    let proof = trust.sign_by(&own_id)?;
    let commit_msg = format!(
        "Add {t_or_d} for {ids}",
        t_or_d = trust_or_distrust,
        ids = ids.join(", ")
    );

    maybe_store(&local, &proof, &commit_msg, proof_create_opt)?;

    Ok(())
}

pub fn maybe_store(
    local: &Local,
    proof: &crev_data::proof::Proof,
    commit_msg: &str,
    proof_create_opt: &CommonProofCreate,
) -> Result<()> {
    if proof_create_opt.print_unsigned {
        print!("{}", proof.body);
    }

    if proof_create_opt.print_signed {
        print!("{}", proof);
    }

    if !proof_create_opt.no_store {
        local.insert(&proof)?;

        if !proof_create_opt.no_commit {
            local
                .proof_dir_commit(&commit_msg)
                .with_context(|_| format_err!("Could not not automatically commit"))?;
        }
    }

    Ok(())
}
