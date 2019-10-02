use crev_lib as crev;
use crate::prelude::*;

pub fn run_command() -> Result<()> {
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
