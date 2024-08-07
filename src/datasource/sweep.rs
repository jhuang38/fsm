use std::thread::JoinHandle;
use std::{
    borrow::Borrow,
    fmt::format,
    path::Path,
    sync::{
        mpsc::{self, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use log::{error, info};

use crate::datasource::{DataReceiver, ReceiverType};
use crate::error::{ErrorType, FsmError};
use crate::filepath::FilepathManager;
use crate::filter::FilterManager;

/**
Manages a thread that sweeps all files under a given directory.
*/
pub struct FileSweepManager {
    repeat_duration: Arc<Mutex<Duration>>,
    sender: Option<Sender<bool>>,
    thread_handle: Option<JoinHandle<Result<(), FsmError>>>,
}

fn sweep(
    path_to_sweep: impl AsRef<Path>,
    filter_manager: Arc<Mutex<FilterManager>>,
    filepath_manager: Arc<Mutex<FilepathManager>>,
    receivers: Arc<Mutex<Vec<Box<dyn DataReceiver + Send>>>>,
) -> Result<(), FsmError> {
    let filepath_manager = filepath_manager.clone();
    let filepath_manager = match filepath_manager.lock() {
        Ok(res) => res,
        Err(e) => return Err(FsmError::new(ErrorType::SweepError, e.to_string())),
    };

    let filter_manager = filter_manager.clone();
    let filter_manager = match filter_manager.lock() {
        Ok(res) => res,
        Err(e) => return Err(FsmError::new(ErrorType::SweepError, e.to_string())),
    };

    if !path_to_sweep.as_ref().exists() {
        return Err(FsmError::new(
            ErrorType::SweepError,
            format!("The path {:#?} does not exist.", path_to_sweep.as_ref()),
        ));
    }
    if !path_to_sweep.as_ref().is_dir() {
        return Err(FsmError::new(
            ErrorType::SweepError,
            format!("The path {:#?} is not a directory.", path_to_sweep.as_ref()),
        ));
    }
    let directory_files = match std::fs::read_dir(path_to_sweep.as_ref()) {
        Ok(res) => res,
        Err(e) => return Err(FsmError::new(ErrorType::SweepError, e.to_string())),
    };
    let mut count = 0;
    let mut successes = 0;

    for file in directory_files {
        count += 1;
        match file {
            Ok(entry) => {
                let _ = filter_manager.place_file_in_mapped_location(
                    entry.path(),
                    &filepath_manager,
                    receivers.clone(),
                );
                successes += 1;
            }
            Err(e) => {
                error!("{e:#?}");
            }
        };
    }

    info!(
        "Found {:#?} files in {:?}, {:#?} succesfully moved.",
        count,
        path_to_sweep.as_ref(),
        successes
    );
    Ok(())
}

impl FileSweepManager {
    pub fn new(repeat_duration: Duration) -> Self {
        Self {
            repeat_duration: Arc::new(Mutex::new(repeat_duration)),
            sender: None,
            thread_handle: None,
        }
    }

    pub fn start_sweep<P>(
        &mut self,
        path_to_watch: P,
        filter_manager: Arc<Mutex<FilterManager>>,
        filepath_manager: Arc<Mutex<FilepathManager>>,
        receivers: Arc<Mutex<Vec<Box<dyn DataReceiver + Send>>>>,
    ) -> Result<(), FsmError>
    where
        P: AsRef<Path> + Send + 'static,
    {
        if self.sender.is_some() {
            return Ok(());
        }
        let (tx, rx) = mpsc::channel::<bool>();
        let repeat_duration = self.repeat_duration.clone();
        let repeat_duration = match repeat_duration.lock() {
            Err(e) => return Err(FsmError::new(ErrorType::SweepError, e.to_string())),
            Ok(res) => res,
        };
        let repeat_duration = *repeat_duration;

        let handle = thread::spawn(move || -> Result<(), FsmError> {
            let mut done = false;
            while !done {
                thread::sleep(repeat_duration);
                // note this locks the corresponding managers
                let _ = sweep(
                    path_to_watch.as_ref(),
                    filter_manager.clone(),
                    filepath_manager.clone(),
                    receivers.clone(),
                );
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
            Err(e) => return Err(FsmError::new(ErrorType::SweepError, e.to_string())),
            Ok(s) => s,
        };
        match handle.join() {
            Err(e) => Err(FsmError::new(ErrorType::SweepError, format!("{:#?}", e))),
            Ok(res) => res,
        }
    }
}
