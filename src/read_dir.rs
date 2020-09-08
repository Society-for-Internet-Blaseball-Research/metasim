use std::collections::BTreeSet;
use std::ffi::OsString;
use std::io::Result;
use std::path::Path;
use std::time::SystemTime;

pub type Entries = BTreeSet<Entry>;

#[derive(Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entry {
    pub file_name: OsString,
    pub len: u64,
    pub modified: SystemTime,
}

pub fn read_dir(path: &Path) -> Result<Entries> {
    let mut set = BTreeSet::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            set.insert(Entry {
                file_name: entry.file_name(),
                len: metadata.len(),
                modified: metadata.modified()?,
            });
        }
    }
    Ok(set)
}
