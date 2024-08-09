use crate::data::LogType;
use crate::data::Message;
use crate::data::Watchers;
use crate::error::ErrorType;
use crate::error::FsmError;
use crate::ConfigManager;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify::{event::CreateKind, EventKind, RecursiveMode};
use notify_debouncer_full::{new_debouncer, notify::Watcher, DebounceEventResult};

use super::notify_receivers;
use super::DataSource;

extern crate notify;
extern crate notify_debouncer_full;

pub struct DirectoryWatcher {
    debounced_watcher: notify_debouncer_full::Debouncer<
        notify::ReadDirectoryChangesWatcher,
        notify_debouncer_full::FileIdMap,
    >,
    receivers: Option<Watchers>,
}

impl DirectoryWatcher {
    pub fn new(
        config_manager: Arc<Mutex<ConfigManager>>,
        receivers: Watchers,
    ) -> Result<Self, FsmError> {
        let new_receivers = receivers.clone();
        let mut debouncer = match new_debouncer(
            Duration::from_secs(1),
            None,
            move |result: DebounceEventResult| {
                match result {
                    Ok(events) => {
                        for event in events.into_iter() {
                            let event = event.event;
                            if event.kind != EventKind::Create(CreateKind::Any) {
                                continue;
                            }
                            let filepath = match event.paths.get(0) {
                                Some(res) => res,
                                None => continue,
                            }
                            .to_owned();
                            notify_receivers(
                                Message::FileMove {
                                    file_to_move: filepath,
                                },
                                receivers.clone(),
                            );
                        }
                    }
                    Err(errors) => {
                        for err in errors {
                            notify_receivers(
                                Message::Log {
                                    message: err.to_string(),
                                    message_type: LogType::Error,
                                },
                                receivers.clone(),
                            );
                        }
                    }
                };
            },
        ) {
            Err(e) => {
                return Err(FsmError::new(ErrorType::WatcherError, e.to_string()));
            }
            Ok(res) => res,
        };

        let config_manager = config_manager.clone();
        let config_manager = match config_manager.lock() {
            Ok(res) => res,
            Err(e) => return Err(FsmError::new(ErrorType::WatcherError, e.to_string())),
        };

        let watch_path = config_manager.get_watch_path();

        let _ = debouncer
            .watcher()
            .watch(watch_path, RecursiveMode::Recursive);
        let _ = debouncer
            .cache()
            .add_root(watch_path, RecursiveMode::Recursive);
        Ok(Self {
            debounced_watcher: debouncer,
            receivers: Some(new_receivers),
        })
    }
}

impl DataSource for DirectoryWatcher {
    fn get_receivers(&self) -> Option<Watchers> {
        self.receivers.clone()
    }
    fn set_receivers(&mut self, receivers: Watchers) {
        self.receivers = Some(receivers);
    }
}
