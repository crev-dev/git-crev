use structopt::StructOpt;
use crate::local;
use crate::prelude::*;
use std::io::prelude::*;

use crev_lib as crev;
use git2;

use serde_yaml;

use crate::index;

#[derive(Debug, StructOpt, Clone)]
#[structopt(name = "add")]
pub struct Add {
    /// Git revision range
    #[structopt(name = "revision range", default_value = "HEAD")]
    pub revision_range: String,

    #[structopt(long = "trust", short = "t")]
    pub trust: bool,

    #[structopt(long = "distrust", short = "d")]
    pub distrust: bool,
}

/// Run 'add' subcommand.
pub fn run_command(args: &Add) -> Result<()> {
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

    add_revision_range_commits(&args.revision_range, &local, &trust_status)?;
    Ok(())
}

macro_rules! try_unwrap_res {
    ($e:expr) => {
        match $e {
            Ok(t) => t,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))?;
            }
        }
    };
}

macro_rules! try_unwrap_opt {
    ($e:expr) => {
        match $e {
            Some(t) => t,
            None => return Err(std::io::Error::new(std::io::ErrorKind::Other, "None"))?,
        }
    };
}

/// Add commit(s) to the index based on a revision specification.
pub fn add_revision_range_commits(
    revision_specification: &str,
    local: &local::Local,
    trust_status: &crev::TrustOrDistrust,
) -> Result<()> {
    let revision_specification =
        local
            .repository
            .revparse(revision_specification)
            .or(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!(
                    "Could not parse given revision specification: {}",
                    revision_specification
                ),
            )))?;
    let commits = commits_from_revision_specification(&revision_specification, &local.repository)?;
    add_commits(&commits, &trust_status, &local)?;

    Ok(())
}

/// Given a revision specification, returns the corresponding commits.
fn commits_from_revision_specification<'a>(
    revision_specification: &'a git2::Revspec,
    repository: &'a git2::Repository,
) -> Result<Vec<git2::Commit<'a>>> {
    let from_commit = try_unwrap_opt!(try_unwrap_opt!(revision_specification.from()).as_commit());

    if revision_specification.mode() == git2::RevparseMode::SINGLE {
        return Ok(vec![from_commit.clone()]);
    } else if revision_specification.mode() == git2::RevparseMode::RANGE {
        let to_commit = try_unwrap_opt!(try_unwrap_opt!(revision_specification.to()).as_commit());

        let mut revision_walk = try_unwrap_res!(repository.revwalk());
        try_unwrap_res!(revision_walk.push(to_commit.id()));

        let mut commits: Vec<git2::Commit> = Vec::new();
        for commit_id in revision_walk {
            let commit_id = try_unwrap_res!(commit_id);
            if commit_id == from_commit.id() {
                break;
            }
            let commit = try_unwrap_res!(repository.find_commit(commit_id));
            commits.push(commit.clone());
        }
        return Ok(commits);
    };

    Ok(vec![])
}

/// Add commits to index file.
fn add_commits(
    commits: &Vec<git2::Commit>,
    trust_status: &crev::TrustOrDistrust,
    local: &local::Local,
) -> Result<()> {
    // TODO: use into_iter here?
    let mut new_entries: std::collections::BTreeSet<index::IndexEntry> = std::collections::BTreeSet::new();
    for commit in commits {
        new_entries.insert(index::IndexEntry {
            commit_id: commit.id().to_string(),
            commit_summary: commit.summary().unwrap_or("").to_string(),
        });
    }

    let contents = std::fs::read_to_string(&local.index_path).unwrap_or("".to_owned());
    let mut index: index::Index = serde_yaml::from_str(&contents).unwrap_or(index::Index::default());
    index.insert(&mut new_entries, trust_status);

    let mut file = std::fs::File::create(&local.index_path)?;
    file.write_all(serde_yaml::to_string(&index)?.as_bytes())?;

    Ok(())
}