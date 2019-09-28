use crate::prelude::*;
use git2;
use std::io::prelude::*;

/// Manages the Git repository's local crev state.
pub struct Local {
    pub root_path: std::path::PathBuf,
    pub index_path: std::path::PathBuf,
    pub repository: git2::Repository,
}

impl Local {
    #[allow(clippy::new_ret_no_self)]
    fn new() -> Result<Self> {
        let repository = git2::Repository::open_from_env().or(Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Current working directory does not seem to be within a Git repository.",
        )))?;
        let root_path = repository
            .workdir()
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not find Git working directory.",
            ))?
            .join(".crev");
        Ok(Self {
            root_path: root_path.clone(),
            index_path: root_path.join("index").into(),
            repository: repository.into(),
        })
    }

    pub fn auto_create_or_open() -> Result<Self> {
        let local = Self::new()?;
        if !local.root_path.exists() {
            std::fs::create_dir_all(&local.root_path)?;
            modify_git_exclude(&local.repository)?;
        }
        Ok(local)
    }
}

/// Ensure .crev is included in .git/info/exclude.
fn modify_git_exclude(repository: &git2::Repository) -> std::io::Result<()> {
    let exclude_file_path = repository.path().join("info").join("exclude");
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .append(true)
        .open(exclude_file_path)?;
    let reader = std::io::BufReader::new(&file);
    for line in reader.lines() {
        if line?.trim() == ".crev" {
            return Ok(());
        }
    }
    writeln!(file, ".crev")?;
    Ok(())
}
