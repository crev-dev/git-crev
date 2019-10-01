use crate::local;
use crate::prelude::*;
use std::io::prelude::*;

use crev_lib as crev;
use git2;

use serde;
use serde_yaml;

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

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
struct IndexEntry {
    commit_id: String,
    commit_summary: String,
}

impl Ord for IndexEntry {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.commit_id.cmp(&other.commit_id)
    }
}

impl PartialOrd for IndexEntry {
    fn partial_cmp(&self, other: &IndexEntry) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for IndexEntry {}

#[derive(Debug, Default, PartialEq, serde::Serialize, serde::Deserialize)]
struct Index {
    trust: std::collections::BTreeSet<IndexEntry>,
    distrust: std::collections::BTreeSet<IndexEntry>,
}

impl Index {
    /// Insert entries into the index. Ensures any existing duplicates are handled correctly.
    fn insert(
        &mut self,
        new_entries: &mut std::collections::BTreeSet<IndexEntry>,
        trust_status: &crev::TrustOrDistrust,
    ) {
        match trust_status {
            crev::TrustOrDistrust::Trust => {
                self.trust.append(new_entries);
                let intersection: Vec<IndexEntry> =
                    self.trust.intersection(&self.distrust).cloned().collect();
                for element in intersection {
                    self.distrust.remove(&element);
                }
            }
            crev::TrustOrDistrust::Distrust => {
                self.distrust.append(new_entries);
                let intersection: Vec<IndexEntry> =
                    self.distrust.intersection(&self.trust).cloned().collect();
                for element in intersection {
                    self.distrust.remove(&element);
                }
            }
        };
    }
}

/// Add commits to index file.
fn add_commits(
    commits: &Vec<git2::Commit>,
    trust_status: &crev::TrustOrDistrust,
    local: &local::Local,
) -> Result<()> {
    // TODO: use into_iter here?
    let mut new_entries: std::collections::BTreeSet<IndexEntry> = std::collections::BTreeSet::new();
    for commit in commits {
        new_entries.insert(IndexEntry {
            commit_id: commit.id().to_string(),
            commit_summary: commit.summary().unwrap_or("").to_string(),
        });
    }

    let contents = std::fs::read_to_string(&local.index_path).unwrap_or("".to_owned());
    let mut index: Index = serde_yaml::from_str(&contents).unwrap_or(Index::default());
    index.insert(&mut new_entries, trust_status);

    let mut file = std::fs::File::create(&local.index_path)?;
    file.write_all(serde_yaml::to_string(&index)?.as_bytes())?;

    Ok(())
}
