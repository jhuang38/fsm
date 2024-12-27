extern crate regex;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    ffi::OsStr,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    error::{ErrorType, FsmError},
    filepath::FilepathManager,
};

/**
Representation of supported file types for filters.
*/

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileFilter {
    filename_pattern: Option<String>,
    allowed_filetypes: Option<HashSet<String>>,
    min_age: Option<Duration>, // todo - add filter here
    max_age: Option<Duration>,
    directory_key: String,
}

impl FileFilter {
    pub fn is_match<P>(&self, file: P) -> bool
    where
        P: AsRef<Path>,
    {
        let valid_filename = match &self.filename_pattern {
            Some(pattern) => {
                let filename = file.as_ref().file_name().unwrap_or_default();
                let filename = filename.to_str().unwrap_or_default();
                let pattern_regex = Regex::new(format!(r#"{}"#, pattern).as_str());
                match pattern_regex {
                    Ok(regexp) => regexp.is_match(filename),
                    Err(_) => false,
                }
            }
            None => true,
        };
        if !valid_filename {
            return false;
        }

        let allowed_filetype = match &self.allowed_filetypes {
            Some(filetypes) => {
                let extension = file
                    .as_ref()
                    .extension()
                    .and_then(OsStr::to_str)
                    .unwrap_or_default();
                extension.is_empty() || filetypes.contains(extension)
            }
            None => true,
        };
        if !allowed_filetype {
            return false;
        }

        // todo - functionality for min/max age

        true
    }
    pub fn set_filename_pattern(&mut self, pattern: String) {
        self.filename_pattern = Some(pattern);
    }
    pub fn add_allowed_filetype(&mut self, new_type: String) {
        match &mut self.allowed_filetypes {
            Some(allowed_filetypes) => {
                allowed_filetypes.insert(new_type);
            }
            None => {
                let mut allowed_filetypes = HashSet::new();
                allowed_filetypes.insert(new_type);
                self.allowed_filetypes = Some(allowed_filetypes);
            }
        }
    }

    pub fn clear_allowed_filetypes(&mut self) {
        match &mut self.allowed_filetypes {
            Some(allowed_filetypes) => {
                allowed_filetypes.clear();
            }
            None => {}
        }
    }
    pub fn set_directory_key(&mut self, key: String) {
        self.directory_key = key;
    }

    pub fn get_directory_key(&self) -> &str {
        &self.directory_key
    }
}

impl Default for FileFilter {
    fn default() -> Self {
        Self {
            filename_pattern: None,
            allowed_filetypes: Some(HashSet::new()),
            min_age: None,
            max_age: None,
            directory_key: "".to_string(),
        }
    }
}

pub struct FilterManager {
    filters: Vec<FileFilter>,
}

impl FilterManager {
    pub fn new(filters: Vec<FileFilter>) -> Self {
        Self { filters }
    }
    pub fn get_mapped_location<P>(
        &self,
        file_to_move: &P,
        filepath_manager: Arc<Mutex<FilepathManager>>,
    ) -> Result<PathBuf, FsmError>
    where
        P: AsRef<Path>,
    {
        let path_ref = file_to_move.as_ref();
        if !path_ref.exists() {
            return Err(FsmError::new(
                ErrorType::FilterError,
                format!(
                    "The path {} does not exist.",
                    path_ref.to_str().unwrap_or_default()
                ),
            ));
        }
        let matching_filter = match self.filters.iter().find(|f| f.is_match(path_ref)) {
            Some(res) => res,
            None => {
                return Err(FsmError::new(
                    ErrorType::FilterError,
                    format!(
                        "The file {} does not match any filters.",
                        path_ref.to_str().unwrap_or_default()
                    ),
                ))
            }
        };

        let filepath_manager = match filepath_manager.lock() {
            Err(e) => return Err(FsmError::new(ErrorType::FilterError, e.to_string())),
            Ok(res) => res,
        };

        let path_mapping = match filepath_manager.get(matching_filter.get_directory_key()) {
            Some(res) => res,
            None => {
                return Err(FsmError::new(
                    ErrorType::FilterError,
                    format!(
                        "The file {} does not map to any managed directories.",
                        path_ref.to_str().unwrap_or_default()
                    ),
                ))
            }
        };

        let file_name = match path_ref.file_name() {
            Some(res) => res,
            None => {
                return Err(FsmError::new(
                    ErrorType::FilterError,
                    format!("Could not obtain file name."),
                ))
            }
        };
        Ok(path_mapping.join(file_name))
    }
}

mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::Path;
    use std::str::FromStr;

    use super::*;
    use crate::reader::DirectoryEntry;

    // TESTS FOR FILTER STRUCT
    #[test]
    fn test_filename_match() {
        let filter = FileFilter {
            filename_pattern: Some(".abc.".to_string()),
            allowed_filetypes: None,
            min_age: None,
            max_age: None,
            directory_key: "basic_regexp".to_string(),
        };

        assert!(filter.is_match("test_filename_match_abc_suffixpadding.txt"));
        assert!(!filter.is_match("test_filename_match_abb_suffix.rs"));
    }

    #[test]
    fn test_filetype_match() {
        let mut filter = FileFilter {
            filename_pattern: None,
            allowed_filetypes: Some(HashSet::from(["txt".to_string()])),
            min_age: None,
            max_age: None,
            directory_key: "test".to_string(),
        };

        assert!(filter.is_match("test_filename_match_abc_suffixpaddingname.txt"));
        assert!(!filter.is_match("test_filename_match_abb_suffixname.rs"));

        let filetypes = filter.allowed_filetypes.as_mut().unwrap();
        filetypes.insert("rs".to_string());
        assert!(filter.is_match("test_filename_match_abb_suffixname.rs"));
    }

    #[test]
    fn test_filename_filetype_match() {
        let filter = FileFilter {
            filename_pattern: Some(".abc.".to_string()),
            allowed_filetypes: Some(HashSet::from(["txt".to_string()])),
            min_age: None,
            max_age: None,
            directory_key: "test".to_string(),
        };

        assert!(filter.is_match("test_abc_file.txt"));
        assert!(!filter.is_match("test_abc_file.rs"));
        assert!(!filter.is_match("test_file.txt"));
    }

    // TESTS FOR FILTER MANAGER
    struct TestMapSetup;
    impl TestMapSetup {
        fn new() -> Result<Self, std::io::Error> {
            fs::create_dir("./filter_test_managed")?;
            fs::create_dir("./filter_test_managed/categorized")?;
            fs::File::create_new("test_abc.txt")?;
            fs::File::create_new("test_rs_file.rs")?;
            fs::File::create_new("no_match_test.txt")?;
            Ok(Self)
        }
    }

    impl Drop for TestMapSetup {
        fn drop(&mut self) {
            fs::remove_file("test_abc.txt").unwrap();
            fs::remove_file("test_rs_file.rs").unwrap();
            fs::remove_file("no_match_test.txt").unwrap();
            fs::remove_dir_all("./filter_test_managed").unwrap();
        }
    }

    #[test]
    fn test_map_location() {
        // Note that it's important to bind this (not set to _) - immediate drop happens otherwise
        let setup = TestMapSetup::new().unwrap();
        let filter1 = FileFilter {
            filename_pattern: Some(".abc.".to_string()),
            allowed_filetypes: Some(HashSet::from(["txt".to_string()])),
            min_age: None,
            max_age: None,
            directory_key: "dir1key".to_string(),
        };
        let filter2 = FileFilter {
            filename_pattern: None,
            allowed_filetypes: Some(HashSet::from(["rs".to_string()])),
            min_age: None,
            max_age: None,
            directory_key: "dir2key".to_string(),
        };

        let filter_mgr = FilterManager::new(Vec::from([filter1, filter2]));

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

        let filepath_mgr = FilepathManager::new("./filter_test_managed", &dir_structure).unwrap();
        let filepath_mgr = Arc::new(Mutex::new(filepath_mgr));

        let f1 = PathBuf::from_str("test_abc.txt").unwrap();
        let f2 = PathBuf::from_str("test_rs_file.rs").unwrap();
        let f3 = PathBuf::from_str("no_match_test.txt").unwrap();
        let nonexistent = PathBuf::from_str("nonexistent.pdf").unwrap();

        let loc1 = filter_mgr.get_mapped_location(&f1, filepath_mgr.clone());
        assert!(loc1.is_ok());
        let loc1 = loc1.unwrap();
        assert!(loc1.to_str().unwrap().contains("dir4"));

        let loc2 = filter_mgr.get_mapped_location(&f2, filepath_mgr.clone());
        assert!(loc2.is_ok());
        let loc2 = loc2.unwrap();
        assert!(loc2.to_str().unwrap().contains("dir2"));

        let loc3 = filter_mgr.get_mapped_location(&f3, filepath_mgr.clone());
        assert!(loc3.is_err());

        let loc4 = filter_mgr.get_mapped_location(&nonexistent, filepath_mgr.clone());
        assert!(loc4.is_err());
    }
}
