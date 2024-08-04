use std::{
    borrow::Borrow,
    sync::{Arc, Mutex},
    time::Duration,
};

use log::{error, info};
use notify::{event::CreateKind, EventKind, RecursiveMode};
use notify_debouncer_full::{new_debouncer, notify::Watcher, DebounceEventResult};

use crate::{
    config::ConfigManager,
    error::{ErrorType, FsmError},
    filepath::{self, FilepathManager},
    filter::FilterManager,
};

use super::DataReceiver;

extern crate notify;
extern crate notify_debouncer_full;

pub struct DirectoryWatcher {
    debounced_watcher: notify_debouncer_full::Debouncer<
        notify::ReadDirectoryChangesWatcher,
        notify_debouncer_full::FileIdMap,
    >,
}

impl DirectoryWatcher {
    pub fn new(
        filter_manager: Arc<Mutex<FilterManager>>,
        filepath_manager: Arc<Mutex<FilepathManager>>,
        config_manager: Arc<Mutex<ConfigManager>>,
        receivers: Arc<Mutex<Vec<Box<dyn DataReceiver + Send>>>>,
    ) -> Result<Self, FsmError> {
        let config_manager_arc = config_manager.clone();
        let config_manager = match config_manager_arc.lock() {
            Ok(res) => res,
            Err(e) => return Err(FsmError::new(ErrorType::WatcherError, e.to_string())),
        };

        let watch_path = config_manager.get_watch_path();

        let mut debouncer = new_debouncer(
            Duration::from_secs(1),
            None,
            move |result: DebounceEventResult| {
                match result {
                    Ok(events) => {
                        let filter_manager = filter_manager.clone();
                        let filepath_manager = filepath_manager.clone();

                        // todo - handling without unwrap
                        let filter_manager = filter_manager.lock().unwrap();

                        events.iter().for_each(|event| {
                            let event = &event.event;
                            if event.kind != EventKind::Create(CreateKind::Any) {
                                return;
                            }
                            let filepath = match event.paths.get(0) {
                                Some(res) => res,
                                None => return,
                            };
                            match filepath_manager.lock() {
                                Ok(mutex_guard) => {
                                    let res = (*filter_manager).place_file_in_mapped_location(
                                        filepath,
                                        &*mutex_guard,
                                        receivers.clone(),
                                    );
                                    match res {
                                        Ok(_) => {}
                                        Err(e) => {
                                            error!("{e:?}");
                                        }
                                    };
                                }
                                Err(_) => {}
                            }

                            // todo - perhaps introduce more indirection here, ideally don't wanna pass stuff in here
                        });
                    }
                    Err(errors) => {
                        //todo - figure out what to do with these
                        for err in errors {
                            error!("{:#?}", err);
                        }
                    }
                }
            },
        );
        // todo - clean wtf is going on here
        if debouncer.is_err() {
            return Err(FsmError::new(
                ErrorType::WatcherError,
                debouncer.unwrap_err().to_string(),
            ));
        }
        let mut debouncer: notify_debouncer_full::Debouncer<
            notify::ReadDirectoryChangesWatcher,
            notify_debouncer_full::FileIdMap,
        > = debouncer.unwrap();

        let _ = debouncer
            .watcher()
            .watch(watch_path, RecursiveMode::Recursive);
        let _ = debouncer
            .cache()
            .add_root(watch_path, RecursiveMode::Recursive);
        info!("Watching directory {watch_path:#?}");
        Ok(Self {
            debounced_watcher: debouncer,
        })
    }
}
