use crev_data::Level;
use structopt::StructOpt;

pub mod id;
pub mod add;


impl From<TrustDistanceParams> for crev_lib::TrustDistanceParams {
    fn from(params: TrustDistanceParams) -> Self {
        crev_lib::TrustDistanceParams {
            max_distance: params.depth,
            high_trust_distance: params.high_cost,
            medium_trust_distance: params.medium_cost,
            low_trust_distance: params.low_cost,
        }
    }
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

#[derive(Debug, StructOpt, Clone, Default)]
pub struct Verify {
    #[structopt(long = "verbose", short = "v")]
    /// Display more information
    pub verbose: bool,

    #[structopt(long = "interactive", short = "i")]
    pub interactive: bool,

    #[structopt(flatten)]
    pub trust_params: TrustDistanceParams,

    #[structopt(flatten)]
    pub requirements: VerificationRequirements,

    #[structopt(long = "skip-verified")]
    /// Display only commits not passing the verification
    pub skip_verified: bool,

    #[structopt(long = "skip-known-owners")]
    /// Skip commits from known owners (use `edit known` to edit the list)
    pub skip_known_owners: bool,

    #[structopt(long = "for-id")]
    /// Root identity to calculate the Web of Trust for [default: current user id]
    pub for_id: Option<String>,

    #[structopt(long = "recursive")]
    /// Calculate recursive metrics for your packages
    pub recursive: bool,
}

#[derive(Debug, Clone, Default)]
pub struct AdviseCommon {
    /// This release contains advisory (important fix)
    pub affected: crev_data::proof::review::package::VersionRange,
    pub severity: Level,
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
    pub common: id::CommonProofCreate,
}

/// Parameters describing trust graph traversal
#[derive(Debug, StructOpt, Clone, Default)]
pub struct TrustDistanceParams {
    #[structopt(long = "depth", default_value = "10")]
    /// Maximum allowed distance from the root identity when traversing trust graph
    pub depth: u64,

    /// Cost of traversing trust graph edge of high trust level
    #[structopt(long = "high-cost", default_value = "0")]
    pub high_cost: u64,
    /// Cost of traversing trust graph edge of medium trust level
    #[structopt(long = "medium-cost", default_value = "1")]
    pub medium_cost: u64,
    /// Cost of traversing trust graph edge of low trust level
    #[structopt(long = "low-cost", default_value = "5")]
    pub low_cost: u64,
}

#[derive(Debug, StructOpt, Clone)]
pub struct FetchUrl {
    /// URL to public proof repository
    pub url: String,
}

#[derive(Debug, StructOpt, Clone)]
pub enum Fetch {
    #[structopt(name = "trusted")]
    /// Fetch updates from trusted Ids
    Trusted(TrustDistanceParams),

    #[structopt(name = "url")]
    /// Fetch from a single public proof repository
    Url(FetchUrl),

    #[structopt(name = "all")]
    /// Fetch all previously retrieved public proof repositories
    All,
}

#[derive(Debug, StructOpt, Clone)]
pub enum Command {
    /// Manage IDs
    #[structopt(name = "id", alias = "new")]
    Id(id::Id),

    /// Fetch proofs from external sources
    #[structopt(name = "fetch")]
    Fetch(Fetch),

    // TODO: should rename publish to push?
    /// Publish local changes to the public proof repository
    #[structopt(name = "publish", alias = "push")]
    Publish,

    /// Import proofs
    #[structopt(name = "import")]
    Import(Import),

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
