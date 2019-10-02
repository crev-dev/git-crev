use crate::index;
use crate::local;
use crate::prelude::*;

pub fn run_command() -> Result<()> {
    let local = local::Local::auto_create_or_open()?;
    let contents = std::fs::read_to_string(&local.index_path).unwrap_or("".to_owned());
    let index: index::Index = serde_yaml::from_str(&contents).unwrap_or(index::Index::default());
    println!(
        "Commits staged as part of an ongoing review.\n\
        \t(use \"git crev commit\" to commit the review)\n"
    );

    println!("Trusted:\n");
    print_commits(&index.trust);
    println!("\n");
    println!("Distrusted:\n");
    print_commits(&index.distrust);
    println!();
    Ok(())
}

fn print_commits(index_entries: &std::collections::BTreeSet<index::IndexEntry>) {
    for entry in index_entries {
        let short_id: String = entry.commit_id.chars().take(8).collect();

        let truncate_length = 100;
        let mut short_summary: String =
            entry.commit_summary.chars().take(truncate_length).collect();
        if entry.commit_summary.chars().count() > truncate_length {
            short_summary += "...";
        }
        println!("\t{}  {}", short_id, short_summary);
    }
}
