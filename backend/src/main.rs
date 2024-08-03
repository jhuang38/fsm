use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use fsm_backend::config::{self, ConfigManager};
use fsm_backend::datasource::sweep::FileSweepManager;
use fsm_backend::datasource::watcher;
use fsm_backend::error::FsmError;
use fsm_backend::filepath::FilepathManager;
use fsm_backend::filter::FilterManager;
use fsm_backend::reader::read_fsm_config;
use log::info;
use env_logger;

fn main() -> Result<(), FsmError> {
    env_logger::init();
    let fsm_config = read_fsm_config("fsm_config.json")?;

    let mut config_manager = ConfigManager::new();
    config_manager.set_watch_path(&fsm_config.watch_path)?;
    config_manager.set_manage_path(&fsm_config.managed_path)?;
    

    let filepath_manager = FilepathManager::new(
        config_manager.get_manage_path(),
        &fsm_config.managed_directory_structure,
    )?;

    let filter_manager = FilterManager::new(fsm_config.filters);

    let mut sweep_manager = FileSweepManager::new(Duration::from_secs(30));
    sweep_manager.start_sweep(config_manager.get_watch_path().clone());

    // create arc mutexes for multithreaded stuff with notify
    let config_manager = Arc::new(Mutex::new(config_manager));
    let filepath_manager = Arc::new(Mutex::new(filepath_manager));
    let filter_manager = Arc::new(Mutex::new(filter_manager));

    let watcher = watcher::DirectoryWatcher::new(filter_manager, filepath_manager, config_manager);

    // stdin controls
    loop {

    }
    info!("Exiting");
    Ok(())
}
