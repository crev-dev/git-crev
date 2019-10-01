use crev_lib as crev;
use serde;

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct IndexEntry {
    pub commit_id: String,
    pub commit_summary: String,
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
pub struct Index {
    pub trust: std::collections::BTreeSet<IndexEntry>,
    pub distrust: std::collections::BTreeSet<IndexEntry>,
}

impl Index {
    /// Insert entries into the index. Ensures any existing duplicates are handled correctly.
    pub fn insert(
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
