use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::{sync::mpsc, sync::mpsc::Sender, thread::JoinHandle, time::Duration};

use super::{notify_receivers, DataSource};
use crate::data::LogType;
use crate::data::{Message, Watchers};
use crate::error::ErrorType;
use crate::error::FsmError;

pub struct DirectorySweeper {
    repeat_duration: Arc<Mutex<Duration>>,
    sender: Option<Sender<bool>>,
    thread_handle: Option<JoinHandle<Result<(), FsmError>>>,
    receivers: Option<Watchers>,
}

fn sweep(path_to_sweep: impl AsRef<Path>, watchers: Watchers) {
    if !path_to_sweep.as_ref().exists() {
        notify_receivers(
            Message::Log {
                message: format!("The path {:#?} does not exist.", path_to_sweep.as_ref()),
                message_type: LogType::Error,
            },
            watchers.clone(),
        );
        return;
    }
    if !path_to_sweep.as_ref().is_dir() {
        notify_receivers(
            Message::Log {
                message: format!("The path {:#?} is not a directory.", path_to_sweep.as_ref()),
                message_type: LogType::Error,
            },
            watchers.clone(),
        );
        return;
    }
    let directory_files = match std::fs::read_dir(path_to_sweep.as_ref()) {
        Ok(res) => res,
        Err(e) => {
            notify_receivers(
                Message::Log {
                    message: e.to_string(),
                    message_type: LogType::Error,
                },
                watchers.clone(),
            );
            return;
        }
    };
    let mut count = 0;
    let mut successes = 0;

    for file in directory_files {
        count += 1;
        match file {
            Ok(entry) => {
                notify_receivers(
                    Message::FileMove {
                        file_to_move: entry.path(),
                    },
                    watchers.clone(),
                );
                successes += 1;
            }
            Err(e) => {
                notify_receivers(
                    Message::Log {
                        message: e.to_string(),
                        message_type: LogType::Error,
                    },
                    watchers.clone(),
                );
            }
        };
    }

    notify_receivers(
        Message::Log {
            message: format!(
                "Found {:#?} files in {:?}, {:#?} succesfully moved.",
                count,
                path_to_sweep.as_ref(),
                successes
            ),
            message_type: LogType::Info,
        },
        watchers.clone(),
    );
}

impl DirectorySweeper {
    pub fn new(repeat_duration: Arc<Mutex<Duration>>) -> Self {
        Self {
            repeat_duration,
            sender: None,
            thread_handle: None,
            receivers: None,
        }
    }

    pub fn start_sweep<P>(&mut self, path_to_watch: P) -> Result<(), FsmError>
    where
        P: AsRef<Path> + Send + 'static,
    {
        if self.sender.is_some() {
            return Ok(());
        }
        let (tx, rx) = mpsc::channel::<bool>();
        let repeat_duration = self.repeat_duration.clone();
        let repeat_duration = match repeat_duration.lock() {
            Err(e) => {
                self.notify_receivers(Message::Log {
                    message: e.to_string(),
                    message_type: LogType::Error,
                });
                return Err(FsmError::new(ErrorType::SweepError, e.to_string()));
            }
            Ok(res) => res,
        }
        .clone();
        let receivers = match &self.receivers {
            None => {
                return Err(FsmError::new(
                    ErrorType::SweepError,
                    "No receivers.".to_string(),
                ))
            }
            Some(res) => res,
        }
        .clone();
        let handle = thread::spawn(move || -> Result<(), FsmError> {
            let mut done = false;
            while !done {
                thread::sleep(repeat_duration);
                // note this locks the corresponding managers
                let _ = sweep(path_to_watch.as_ref(), receivers.clone());
                done = rx.try_recv().unwrap_or_default();
            }
            Ok(())
        });

        self.sender = Some(tx);
        self.thread_handle = Some(handle);

        Ok(())
    }

    pub fn end_sweep(&mut self) -> Result<(), FsmError> {
        if self.thread_handle.is_none() {
            return Ok(());
        }
        let sender = std::mem::replace(&mut self.sender, None).unwrap();
        let handle = std::mem::replace(&mut self.thread_handle, None).unwrap();

        match sender.send(true) {
            Err(e) => {
                self.notify_receivers(Message::Log {
                    message: format!("{:#?}", &e),
                    message_type: LogType::Error,
                });
                return Err(FsmError::new(ErrorType::SweepError, e.to_string()));
            }
            Ok(s) => s,
        };
        match handle.join() {
            Err(e) => {
                self.notify_receivers(Message::Log {
                    message: format!("{:#?}", &e),
                    message_type: LogType::Error,
                });
                return Err(FsmError::new(ErrorType::SweepError, format!("{:#?}", e)));
            }
            Ok(res) => res,
        }
    }
}

impl DataSource for DirectorySweeper {
    fn get_receivers(&self) -> Option<Watchers> {
        self.receivers.clone()
    }
    fn set_receivers(&mut self, receivers: Watchers) {
        self.receivers = Some(receivers);
    }
}
