use std::{
    collections::{HashMap, VecDeque},
    fs,
    path::Path,
    path::PathBuf,
};

use crate::error::FsmError;
use crate::reader::DirectoryEntry;

#[derive(Debug)]
pub struct FilepathManager {
    directory_mapping: HashMap<String, PathBuf>,
}

impl FilepathManager {
    pub fn new<P>(
        base_managed_directory: P,
        directory_structure: &DirectoryEntry,
    ) -> Result<Self, FsmError>
    where
        P: AsRef<Path>,
    {
        let mut traversal_queue: VecDeque<(&DirectoryEntry, PathBuf)> = VecDeque::from([(
            directory_structure,
            PathBuf::from(base_managed_directory.as_ref()).join("categorized"),
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

#[cfg(test)]
mod test {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn test_flat_directories() {
        let dir_structure = DirectoryEntry::ParentDirectory(HashMap::from([
            (
                "dir1".to_string(),
                DirectoryEntry::LeafDirectory("dir1key".to_string()),
            ),
            (
                "dir2".to_string(),
                DirectoryEntry::LeafDirectory("dir2key".to_string()),
            ),
        ]));
        fs::create_dir("filepath_manager_test_flat").unwrap();
        let base_dir = PathBuf::from_str("./filepath_manager_test_flat").unwrap();

        let filepath_mgr = FilepathManager::new(&base_dir, &dir_structure).unwrap();

        let dir1key_result = filepath_mgr.get("dir1key").unwrap();
        assert!(dir1key_result.to_str().unwrap().len() > 0);

        let dir2key_result = filepath_mgr.get("dir2key").unwrap();
        assert!(dir2key_result.to_str().unwrap().len() > 0);

        let none_result = filepath_mgr.get("nonexistent");
        assert!(none_result.is_none());

        fs::remove_dir_all("filepath_manager_test_flat").unwrap();
    }

    #[test]
    fn test_nested_directories() {
        let dir_structure = DirectoryEntry::ParentDirectory(HashMap::from([(
            "dir1".to_string(),
            DirectoryEntry::ParentDirectory(HashMap::from([
                (
                    "dir2".to_string(),
                    DirectoryEntry::LeafDirectory("dir2key".to_string()),
                ),
                (
                    "dir3".to_string(),
                    DirectoryEntry::ParentDirectory(HashMap::from([(
                        "dir4".to_string(),
                        DirectoryEntry::LeafDirectory("dir1key".to_string()),
                    )])),
                ),
            ])),
        )]));
        fs::create_dir("filepath_manager_test_nested").unwrap();
        let base_dir = PathBuf::from_str("./filepath_manager_test_nested").unwrap();

        let filepath_mgr = FilepathManager::new(&base_dir, &dir_structure).unwrap();

        let dir1key_result = filepath_mgr.get("dir1key").unwrap();
        assert!(dir1key_result.to_str().unwrap().len() > 0);

        let dir2key_result = filepath_mgr.get("dir2key").unwrap();
        assert!(dir2key_result.to_str().unwrap().len() > 0);

        assert!(dir1key_result.to_str().unwrap().len() > dir2key_result.to_str().unwrap().len());

        let none_result = filepath_mgr.get("nonexistent");
        assert!(none_result.is_none());

        fs::remove_dir_all("filepath_manager_test_nested").unwrap();
    }
}
