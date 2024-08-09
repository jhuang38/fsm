use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
};

use data_receiver::DataReceiver;
use data_source::DataSource;

pub mod data_receiver;
pub mod data_source;

type Producers = Arc<Mutex<Vec<Box<dyn DataSource + Send>>>>;
type Watchers = Arc<Mutex<Vec<Box<dyn DataReceiver + Send>>>>;

#[derive(Debug, Clone)]
pub enum LogType {
    Info,
    Error,
    Warning,
}

#[derive(Debug, Clone)]
pub enum Message {
    FileMove {
        file_to_move: PathBuf,
    },
    Log {
        message: String,
        message_type: LogType,
    },
}

pub struct MessageManager {
    sources: Producers,
    receivers: Watchers,
}

impl MessageManager {
    pub fn new() -> Self {
        Self {
            sources: Arc::new(Mutex::new(vec![])),
            receivers: Arc::new(Mutex::new(vec![])),
        }
    }
    pub fn add_source(&mut self, mut source: Box<dyn DataSource + Send>) {
        let sources = self.sources.clone();
        let mut sources = match sources.lock() {
            Err(_) => return,
            Ok(res) => res,
        };
        source.set_receivers(self.receivers.clone());
        sources.push(source);
    }
    pub fn add_receiver(&mut self, receiver: Box<dyn DataReceiver + Send>) {
        let receivers = self.receivers.clone();
        let mut receivers = match receivers.lock() {
            Err(_) => return,
            Ok(res) => res,
        };
        receivers.push(receiver);
        let sources = self.sources.clone();
        let mut sources = match sources.lock() {
            Err(_) => return,
            Ok(res) => res,
        };
        for source in sources.iter_mut() {
            source.set_receivers(self.receivers.clone());
        }
    }
    pub fn get_receivers(&self) -> Watchers {
        self.receivers.clone()
    }
}
