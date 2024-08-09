use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;
use std::thread;

use crate::error::ErrorType;
use crate::error::FsmError;
use crate::filepath::FilepathManager;
use crate::filter::FilterManager;

use super::DataReceiver;
use super::Message;

pub struct PathWriter {
    overwrite_on_move: bool,
    filepath_manager: Arc<Mutex<FilepathManager>>,
    filter_manager: Arc<Mutex<FilterManager>>,
}

fn move_file(
    file_to_move: impl AsRef<Path>,
    new_location: impl AsRef<Path>,
    overwrite_on_move: bool,
) -> Result<(), FsmError> {
    let old_location = file_to_move.as_ref().to_path_buf();
    let mut new_location = new_location.as_ref().to_path_buf();
    if overwrite_on_move || !new_location.exists() {
        // ideal to add a mechanism for logging
        match fs::rename(&old_location, &new_location) {
            Ok(_) => Ok(()),
            Err(e) => Err(FsmError::new(ErrorType::FilterError, e.to_string())),
        }
    } else {
        let new_path = {
            // todo - come up with a better renaming scheme
            let mut increment_id: u64 = 0;
            while new_location.exists() {
                let mut new_filename = new_location
                    .file_stem()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or_default()
                    .to_owned();
                new_filename.push_str(&("_".to_string() + increment_id.to_string().as_str()));
                let file_extension = new_location.extension().unwrap_or_default().to_owned();
                new_location.set_file_name(new_filename);
                new_location.set_extension(file_extension);
                increment_id += 1;
            }
            new_location
        };
        // todo - potentially reduce some duplication here
        match fs::rename(&old_location, &new_path) {
            Ok(_) => Ok(()),
            Err(e) => Err(FsmError::new(ErrorType::FilterError, e.to_string())),
        }
    }
}

impl PathWriter {
    pub fn new(
        overwrite_on_move: bool,
        filepath_manager: Arc<Mutex<FilepathManager>>,
        filter_manager: Arc<Mutex<FilterManager>>,
    ) -> Self {
        Self {
            overwrite_on_move,
            filepath_manager,
            filter_manager,
        }
    }
}

impl DataReceiver for PathWriter {
    fn process_message(&self, message: Message) {
        match message {
            Message::FileMove { file_to_move } => {
                let write_on_move = self.overwrite_on_move;
                // ideally have a nice way to log this without using macro directly (through logger)
                let filepath_manager = self.filepath_manager.clone();
                let filter_manager = self.filter_manager.clone();
                let _handle = thread::spawn(move || {
                    let filter_manager = match filter_manager.lock() {
                        // need to fix error types as well
                        Err(e) => return Err(FsmError::new(ErrorType::FilterError, e.to_string())),
                        Ok(res) => res,
                    };
                    let mapped_location = match filter_manager
                        .get_mapped_location(&file_to_move, filepath_manager)
                    {
                        Err(e) => return Err(FsmError::new(ErrorType::FilterError, e.to_string())),
                        Ok(res) => res,
                    };
                    move_file(file_to_move.clone(), mapped_location, write_on_move)
                });
            }
            _ => {}
        }
    }
}
