use structopt::StructOpt;

use crate::prelude::*;
use crate::shared::*;
use crev_lib as crev;

pub fn run_command(subcommand: Import) -> Result<()> {
    match subcommand {
        Import::Proof(args) => {
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
    }
    Ok(())
}

#[derive(Debug, StructOpt, Clone)]
pub enum Import {
    /// Import proofs: resign proofs using current id
    ///
    /// Useful for mass-import of proofs signed by another ID
    #[structopt(name = "proof")]
    Proof(ImportProof),
}

#[derive(Debug, StructOpt, Clone)]
pub struct ImportProof {
    /// Reset proof date to current date
    #[structopt(long = "reset-date")]
    pub reset_date: bool,

    #[structopt(flatten)]
    pub common: CommonProofCreate,
}
