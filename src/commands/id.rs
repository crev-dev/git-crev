use structopt;
use structopt::StructOpt;

use crev::TrustOrDistrust::*;
use crev_lib as crev;

use crate::prelude::*;
use crate::shared::*;

pub fn run_command(subcommand: Id) -> Result<()> {
    match subcommand {
        Id::New(args) => {
            let local = crev::Local::auto_create_or_open()?;
            let res = local.generate_id(args.url, args.github_username, args.use_https_push);
            if res.is_err() {
                eprintln!("Visit https://github.com/dpc/crev/wiki/Proof-Repository for help.");
            }
            let _ = crev::Local::auto_open()?;
            res?;
        }
        Id::Switch(args) => {
            let local = crev::Local::auto_open()?;
            local.switch_id(&args.id)?
        }
        Id::Edit(args) => match args {
            Edit::Readme => {
                let local = crev::Local::auto_open()?;
                local.edit_readme()?;
            }
            Edit::Config => {
                let local = crev::Local::auto_create_or_open()?;
                local.edit_user_config()?;
            }
        },
        Id::Show => {
            let local = crev::Local::auto_open()?;
            local.show_own_ids()?;
        }
        Id::Trust(args) => {
            create_trust_proof(args.pub_ids, Trust, &args.common_proof_create)?;
        }
        Id::Distrust(args) => {
            create_trust_proof(args.pub_ids, Distrust, &args.common_proof_create)?;
        }
        Id::Export(params) => {
            let local = crev::Local::auto_open()?;
            println!("{}", local.export_locked_id(params.id)?);
        }
        Id::Import => {
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
    }
    Ok(())
}

#[derive(Debug, StructOpt, Clone)]
pub enum Id {
    /// Create a new Id
    #[structopt(name = "new")]
    New(NewId),

    /// Export your own Id
    #[structopt(name = "export")]
    Export(ExportId),

    /// Import an Id as your own
    #[structopt(name = "import")]
    Import,

    /// Show your own Id
    #[structopt(name = "show")]
    Show,

    /// Change current Id
    #[structopt(name = "switch")]
    Switch(SwitchId),

    /// Edit README.md of the current ID
    #[structopt(name = "edit")]
    Edit(Edit),

    /// Trust an Id
    #[structopt(name = "trust")]
    Trust(Trust),

    /// Distrust an Id
    #[structopt(name = "distrust")]
    Distrust(Trust),
}

#[derive(Debug, StructOpt, Clone)]
pub struct NewId {
    #[structopt(long = "url")]
    /// URL of a git repository to be associated with the new Id
    pub url: Option<String>,
    #[structopt(long = "github-username")]
    /// Github username (instead of --url)
    pub github_username: Option<String>,
    #[structopt(long = "https-push")]
    /// Setup `https` instead of recommended `ssh`-based push url
    pub use_https_push: bool,
}

#[derive(Debug, StructOpt, Clone)]
pub enum New {
    #[structopt(name = "id")]
    /// Generate a CrevID
    Id(NewId),
}

#[derive(Debug, StructOpt, Clone)]
pub enum Edit {
    /// Edit your README.md file
    #[structopt(name = "readme")]
    Readme,

    /// Edit your user config
    #[structopt(name = "config")]
    Config,
}

// TODO: is the right module for this?
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

#[derive(Debug, StructOpt, Clone)]
pub struct Trust {
    /// Public IDs to create Trust Proof for
    pub pub_ids: Vec<String>,

    #[structopt(flatten)]
    pub common_proof_create: CommonProofCreate,
}

#[derive(Debug, StructOpt, Clone)]
pub struct SwitchId {
    /// Own Id to switch to
    pub id: String,
}

#[derive(Debug, StructOpt, Clone)]
pub struct ExportId {
    pub id: Option<String>,
}
