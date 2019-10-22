use crate::prelude::*;

use std::io::prelude::*;
use std::io::{self, BufRead};
use structopt::StructOpt;

use crev_lib as crev;

use crate::index;
use crate::local;

pub fn run(local: &local::Local) -> Result<()> {
    let contents = std::fs::read_to_string(&local.index_path).unwrap_or("".to_owned());
    let mut index: index::Index =
        serde_yaml::from_str(&contents).unwrap_or(index::Index::default());

    let mut revwalk = local.repository.revwalk()?;
    revwalk.push_head()?;
    for commit_id in revwalk {
        let commit_id = commit_id?;
        if index.contains_commit_id(commit_id.to_string()) {
            continue;
        }

        let mut git_command = std::process::Command::new("git")
            .arg("log")
            .arg("--patch")
            .arg("-1")
            .arg(commit_id.to_string())
            .spawn()?;
        git_command.wait()?;

        let commands = parse_live_commands()?;
        if commands.skip {
            continue;
        }

        let commit = local.repository.find_commit(commit_id)?;
        let mut new_entry: std::collections::BTreeSet<index::IndexEntry> =
            std::collections::BTreeSet::new();
        new_entry.insert(index::IndexEntry {
            commit_id: commit.id().to_string(),
            commit_summary: commit.summary().unwrap_or("").to_string(),
        });

        if commands.trust {
            index.insert(&mut new_entry, &crev::TrustOrDistrust::Trust);
        } else if commands.distrust {
            index.insert(&mut new_entry, &crev::TrustOrDistrust::Distrust);
        }

        let mut file = std::fs::File::create(&local.index_path)?;
        file.write_all(serde_yaml::to_string(&index)?.as_bytes())?;
    }
    Ok(())
}

fn parse_live_commands() -> Result<ReviewCommands> {
    let stdin = io::stdin();

    let mut line: String = "".into();
    while line == "" {
        print!("Review (skip:-s; trust:-t; distrust:-d): ");
        io::stdout().flush()?;

        stdin.lock().read_line(&mut line)?;
        line = line.trim().into();
        if line == "" {
            continue;
        }

        let mut commands: Vec<&str> = vec!["git"];
        commands.extend(line.split(" "));

        match ReviewCommands::from_iter_safe(commands) {
            Ok(commands) => {
                return Ok(commands);
            }
            Err(_) => {
                eprintln!("Invalid commands: {}", line);
                line = "".into();
                continue;
            }
        };
    }

    Ok(ReviewCommands::default())
}

#[derive(Debug, Default, StructOpt, Clone)]
pub struct ReviewCommands {
    #[structopt(short = "s")]
    pub skip: bool,

    #[structopt(short = "t")]
    pub trust: bool,

    #[structopt(short = "d")]
    pub distrust: bool,
}
