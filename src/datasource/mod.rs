use std::path::PathBuf;

use crate::error::FsmError;

pub mod sweep;
pub mod watcher;

/**
 * Struct for keeping track of file movements
 */
pub struct DataMessage {
    pub from_path: PathBuf,
    pub to_path: PathBuf,
}

pub enum ReceiverType {
    FileWriter,
    DashboardMessages,
}

pub trait DataReceiver {
    fn accept_data(&mut self, message: &DataMessage) -> Result<(), FsmError>;
    fn receiver_type(&self) -> ReceiverType;
}
