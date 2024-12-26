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
    pub fn new<P1, P2>(
        base_path_to_watch: P1,
        base_path_to_manage: P2,
        overwrite_on_move: bool,
    ) -> Self
    where
        P1: AsRef<Path>,
        P2: AsRef<Path>,
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

mod test {
    use std::str::FromStr;

    use super::*;

    #[test]
    fn test_init_valid() {
        std::fs::create_dir("test_config");

        let p1 = PathBuf::from_str("./").unwrap();
        let p2 = PathBuf::from_str("test_config").unwrap();
        let cfg_mgr = ConfigManager::new("./", p2.clone(), false);

        assert_eq!(cfg_mgr.get_watch_path(), &p1);
        assert_eq!(cfg_mgr.get_manage_path(), &p2);
        assert!(!cfg_mgr.perform_overwrite_on_move());

        std::fs::remove_dir("test_config");
    }

    fn test_init_invalid() {
        let p1 = PathBuf::from_str("fdskfjldasj").unwrap();
        let p2 = PathBuf::from_str("afsdkfjalsdfs").unwrap();
        let cfg_mgr = ConfigManager::new("fdskfjldasj", p2.clone(), false);

        assert_eq!(cfg_mgr.get_watch_path(), &p1);
        assert_eq!(cfg_mgr.get_manage_path(), &p2);
        assert!(!cfg_mgr.perform_overwrite_on_move());
    }

    // OPTIONAL: Consider adding more here once the setters/getters do more
    // #[test]
    fn test_config_manager_getters() {
        todo!()
    }

    // #[test]
    fn test_config_manager_setters() {
        todo!()
    }
}
