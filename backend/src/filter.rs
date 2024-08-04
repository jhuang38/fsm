extern crate regex;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use log::info;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Borrow,
    collections::HashSet,
    ffi::OsStr,
    fmt::format,
    fs::{self, File},
    path::{Path, PathBuf},
    str::FromStr,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    config::ConfigManager,
    datasource::{DataMessage, DataReceiver},
    error::{ErrorType, FsmError},
    filepath::FilepathManager,
};

/**
Representation of supported file types for filters.
*/

#[derive(Serialize, Deserialize, Debug)]
pub struct FileFilter {
    filename_pattern: Option<String>,
    allowed_filetypes: Option<HashSet<String>>,
    min_age: Option<Duration>, // todo - add filter here
    max_age: Option<Duration>,
    directory_key: String,
}

impl FileFilter {
    pub fn is_match(&self, file: &Path) -> bool {
        let valid_filename = match &self.filename_pattern {
            Some(pattern) => {
                let filename = file.file_name().unwrap_or_default();
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
                let extension = file.extension().and_then(OsStr::to_str).unwrap_or_default();
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
    pub fn place_file_in_mapped_location<P>(
        &self,
        file_to_move: P,
        filepath_manager: &FilepathManager,
        receivers: Arc<Mutex<Vec<Box<dyn DataReceiver + Send>>>>,
    ) -> Result<(), FsmError>
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

        let mut new_location = path_mapping.join(file_name);
        let request_message = DataMessage {
            from_path: path_ref.to_owned(),
            to_path: new_location,
        };
        let mut receivers = receivers.clone();
        let mut receivers = match receivers.lock() {
            Ok(res) => res,
            Err(e) => return Err(FsmError::new(ErrorType::FilterError, e.to_string())),
        };
        for receiver in receivers.iter_mut() {
            receiver.accept_data(&request_message);
        }
        Ok(())
    }
}
