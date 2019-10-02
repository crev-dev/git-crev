use structopt;
use structopt::StructOpt;

use crev_lib as crev;

use crate::prelude::*;

pub fn run_command(subcommand: Fetch) -> Result<()> {
    match subcommand {
        Fetch::Trusted(params) => {
            let local = crev::Local::auto_create_or_open()?;
            local.fetch_trusted(params.into())?;
        }
        Fetch::Url(params) => {
            let local = crev::Local::auto_create_or_open()?;
            local.fetch_url(&params.url)?;
        }
        Fetch::All => {
            let local = crev::Local::auto_create_or_open()?;
            local.fetch_all()?;
        }
    }
    Ok(())
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
pub struct FetchUrl {
    /// URL to public proof repository
    pub url: String,
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
