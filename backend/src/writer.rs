use crate::{
    datasource::{DataMessage, DataReceiver},
    error::ErrorType,
    error::FsmError,
};
use log::info;
use std::fs;

pub struct FileWriter {
    overwrite_on_move: bool,
}

impl FileWriter {
    pub fn new(overwrite_on_move: bool) -> Self {
        Self { overwrite_on_move }
    }
    pub fn do_overwrite_on_move(&self) -> bool {
        self.overwrite_on_move
    }
}

impl DataReceiver for FileWriter {
    fn accept_data(&mut self, message: &DataMessage) -> Result<(), FsmError> {
        let mut old_location = message.from_path.to_owned();
        let mut new_location = message.to_path.to_owned();
        if self.overwrite_on_move || !new_location.exists() {
            match fs::rename(&old_location, &new_location) {
                Ok(_) => {
                    info!("File {old_location:#?} has been moved to {new_location:#?}");
                    Ok(())
                }
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
                Ok(_) => {
                    info!("File {old_location:#?} has been moved to {new_path:#?}");
                    Ok(())
                }
                Err(e) => Err(FsmError::new(ErrorType::FilterError, e.to_string())),
            }
        }
    }
}
