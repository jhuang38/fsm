use std::path::PathBuf;

use crate::error::FsmError;

pub mod sweep;
pub mod watcher;

pub struct DataMessage {
    pub from_path: PathBuf,
    pub to_path: PathBuf,
}

pub trait DataReceiver {
    fn accept_data(&mut self, message: &DataMessage) -> Result<(), FsmError>;
}
