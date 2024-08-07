use std::{
    collections::{HashMap, VecDeque},
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use crate::error::ErrorType;
use crate::error::FsmError;
use crate::reader::{DirectoryEntry, FsmConfigRepresentation};

#[derive(Debug)]
pub struct FilepathManager {
    directory_mapping: HashMap<String, PathBuf>,
}

impl FilepathManager {
    pub fn new(
        base_managed_directory: &PathBuf,
        directory_structure: &DirectoryEntry,
    ) -> Result<Self, FsmError> {
        let mut traversal_queue: VecDeque<(&DirectoryEntry, PathBuf)> = VecDeque::from([(
            directory_structure,
            PathBuf::from(base_managed_directory).join("categorized"),
        )]);
        let mut directory_mapping: HashMap<String, PathBuf> = HashMap::new();

        while let Some((entry, curr_path)) = traversal_queue.pop_front() {
            if !curr_path.exists() {
                fs::create_dir(&curr_path)?;
            }
            match entry {
                DirectoryEntry::LeafDirectory(key_name) => {
                    // TODO - avoid using clone here
                    directory_mapping.insert(key_name.clone(), curr_path);
                }
                DirectoryEntry::ParentDirectory(children) => {
                    // create children entries
                    for (key, value) in children.into_iter() {
                        traversal_queue.push_back((value, curr_path.join(key)));
                    }
                }
            }
        }
        Ok(Self { directory_mapping })
    }
    pub fn get(&self, directory_key: &str) -> Option<&PathBuf> {
        self.directory_mapping.get(directory_key)
    }
    pub fn map_ref(&self) -> &HashMap<String, PathBuf> {
        &self.directory_mapping
    }
}
