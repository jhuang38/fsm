use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use config::ConfigManager;
use datasource::sweep::FileSweepManager;
use datasource::watcher::DirectoryWatcher;
use error::FsmError;
use filepath::FilepathManager;
use filter::FilterManager;
use reader::read_fsm_config;

pub mod config;
pub mod dashboard;
pub mod datasource;
pub mod error;
pub mod filepath;
pub mod filter;
pub mod reader;

pub struct AppState {
    pub config_manager: Arc<Mutex<ConfigManager>>,
    pub filepath_manager: Arc<Mutex<FilepathManager>>,
    pub filter_manager: Arc<Mutex<FilterManager>>,
    pub sweep_manager: Arc<Mutex<FileSweepManager>>,
    pub directory_watcher: DirectoryWatcher,
}

pub fn init_fsm_managers(config_file_path: impl AsRef<Path>) -> Result<AppState, FsmError> {
    let fsm_config = read_fsm_config(config_file_path)?;

    let config_manager = ConfigManager::new(
        fsm_config.watch_path,
        fsm_config.managed_path,
        fsm_config.overwrite_on_move,
    );

    let filepath_manager = FilepathManager::new(
        config_manager.get_manage_path(),
        &fsm_config.managed_directory_structure,
    )?;

    let filter_manager = FilterManager::new(fsm_config.filters);

    let mut sweep_manager = FileSweepManager::new(Duration::from_secs(fsm_config.sweep_loop_time));

    // create arc mutexes for multithreaded stuff with notify

    let filepath_manager = Arc::new(Mutex::new(filepath_manager));
    let filter_manager = Arc::new(Mutex::new(filter_manager));

    let _ = sweep_manager.start_sweep(
        config_manager.get_watch_path().clone(),
        filter_manager.clone(),
        filepath_manager.clone(),
        config_manager.perform_overwrite_on_move(),
    )?;

    let config_manager = Arc::new(Mutex::new(config_manager));

    let directory_watcher = DirectoryWatcher::new(
        filter_manager.clone(),
        filepath_manager.clone(),
        config_manager.clone(),
    )?;

    Ok(AppState {
        config_manager,
        filepath_manager,
        filter_manager,
        sweep_manager: Arc::new(Mutex::new(sweep_manager)),
        directory_watcher,
    })
}
