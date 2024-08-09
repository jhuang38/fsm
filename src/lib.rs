use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;

use config::ConfigManager;
use data::data_receiver::logger::Logger;
use data::data_receiver::writer::PathWriter;
use data::data_source::sweep::DirectorySweeper;
use data::data_source::watch::DirectoryWatcher;
use data::data_source::DataSource;
use data::MessageManager;
use error::FsmError;
use filepath::FilepathManager;
use filter::FilterManager;
use reader::read_fsm_config;

pub mod config;
pub mod data;
pub mod error;
pub mod filepath;
pub mod filter;
pub mod reader;

pub struct FsmState {
    pub config_manager: Arc<Mutex<ConfigManager>>,
    pub filepath_manager: Arc<Mutex<FilepathManager>>,
    pub filter_manager: Arc<Mutex<FilterManager>>,
    pub message_manager: MessageManager,
}

pub fn init_fsm(config_file_path: impl AsRef<Path>) -> Result<FsmState, FsmError> {
    let fsm_config = read_fsm_config(config_file_path)?;

    // init basic managers
    let config_manager = ConfigManager::new(
        fsm_config.watch_path,
        fsm_config.managed_path,
        fsm_config.overwrite_on_move,
    );

    let filepath_manager = FilepathManager::new(
        config_manager.get_manage_path(),
        &fsm_config.managed_directory_structure,
    )?;
    let filepath_manager = Arc::new(Mutex::new(filepath_manager));

    let filter_manager = FilterManager::new(fsm_config.filters);
    let filter_manager = Arc::new(Mutex::new(filter_manager));

    let mut message_manager = MessageManager::new();
    // add receivers
    message_manager.add_receiver(Box::new(Logger::new()));

    let file_writer = PathWriter::new(
        config_manager.perform_overwrite_on_move(),
        filepath_manager.clone(),
        filter_manager.clone(),
    );
    message_manager.add_receiver(Box::new(file_writer));

    // add data sources
    let mut directory_sweeper = DirectorySweeper::new(Arc::new(Mutex::new(Duration::from_secs(
        fsm_config.sweep_loop_time,
    ))));
    directory_sweeper.set_receivers(message_manager.get_receivers());

    // todo - deal with clone here
    let _ = directory_sweeper.start_sweep(config_manager.get_watch_path().clone());

    let config_manager = Arc::new(Mutex::new(config_manager));
    message_manager.add_source(Box::new(directory_sweeper));

    message_manager.add_source(Box::new(
        DirectoryWatcher::new(config_manager.clone(), message_manager.get_receivers()).unwrap(),
    ));

    // add data receivers
    Ok(FsmState {
        config_manager,
        filepath_manager,
        filter_manager,
        message_manager,
    })
}
