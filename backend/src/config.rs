use std::path::Path;
use std::path::PathBuf;

use crate::error::ErrorType;
use crate::error::FsmError;
/**
Struct to manage the various system properties (e.g. base path, etc.)
*/
#[derive(Debug)]
pub struct ConfigManager {
    base_path_to_watch: PathBuf,
    base_path_to_manage: PathBuf,
    overwrite_on_move: bool,
}

impl ConfigManager {
    pub fn new<P>(base_path_to_watch: P, base_path_to_manage: P, overwrite_on_move: bool) -> Self
    where
        P: AsRef<Path>,
    {
        Self {
            base_path_to_watch: base_path_to_watch.as_ref().to_owned(),
            base_path_to_manage: base_path_to_manage.as_ref().to_owned(),
            overwrite_on_move,
        }
    }

    pub fn set_watch_path<P>(&mut self, new_path: P) -> Result<(), FsmError>
    where
        P: AsRef<Path>,
    {
        if !new_path.as_ref().exists() {
            return Err(FsmError::new(
                ErrorType::ConfigError,
                format!(
                    "Invalid path {} provided.",
                    new_path.as_ref().to_str().unwrap_or_default()
                ),
            ));
        }
        self.base_path_to_watch.clear();
        self.base_path_to_watch.push(new_path);
        Ok(())
    }

    pub fn set_manage_path<P>(&mut self, new_path: P) -> Result<(), FsmError>
    where
        P: AsRef<Path>,
    {
        if !new_path.as_ref().exists() {
            return Err(FsmError::new(
                ErrorType::ConfigError,
                format!(
                    "Invalid path {} provided.",
                    new_path.as_ref().to_str().unwrap_or_default()
                ),
            ));
        }
        self.base_path_to_manage.clear();
        self.base_path_to_manage.push(new_path);
        Ok(())
    }

    pub fn get_watch_path(&self) -> &PathBuf {
        &self.base_path_to_watch
    }

    pub fn get_manage_path(&self) -> &PathBuf {
        &self.base_path_to_manage
    }
    pub fn perform_overwrite_on_move(&self) -> bool {
        self.overwrite_on_move
    }
}
