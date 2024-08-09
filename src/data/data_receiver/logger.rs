use log::error;
use log::info;
use log::warn;

use crate::data::LogType;

use super::DataReceiver;
use super::Message;

pub struct Logger {}

impl Logger {
    pub fn new() -> Self {
        Self {}
    }
}

impl DataReceiver for Logger {
    fn process_message(&self, message: Message) {
        match message {
            Message::Log {
                message,
                message_type,
            } => match message_type {
                LogType::Info => info!("{:#?}", message),
                LogType::Warning => warn!("{:#?}", message),
                LogType::Error => error!("{:#?}", message),
            },
            Message::FileMove { file_to_move } => {
                info!(
                    "Attempting to move file {:#?} to mapped location.",
                    file_to_move
                );
            }
        }
    }
}
