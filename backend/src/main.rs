use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use env_logger;
use fsm_backend::config::{self, ConfigManager};
use fsm_backend::datasource::sweep::FileSweepManager;
use fsm_backend::datasource::watcher;
use fsm_backend::error::FsmError;
use fsm_backend::filepath::FilepathManager;
use fsm_backend::filter::FilterManager;
use fsm_backend::reader::read_fsm_config;
use log::info;

fn main() -> Result<(), FsmError> {
    env_logger::init();
    let fsm_config = read_fsm_config("fsm_config.json")?;

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

    let mut sweep_manager = FileSweepManager::new(Duration::from_secs(30));

    // create arc mutexes for multithreaded stuff with notify

    let filepath_manager = Arc::new(Mutex::new(filepath_manager));
    let filter_manager = Arc::new(Mutex::new(filter_manager));

    sweep_manager.start_sweep(
        config_manager.get_watch_path().clone(),
        filter_manager.clone(),
        filepath_manager.clone(),
        config_manager.perform_overwrite_on_move(),
    );

    let config_manager = Arc::new(Mutex::new(config_manager));

    let watcher = watcher::DirectoryWatcher::new(
        filter_manager.clone(),
        filepath_manager.clone(),
        config_manager.clone(),
    );

    // stdin controls
    loop {}
    info!("Exiting");
    Ok(())
}
