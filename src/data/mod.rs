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

mod test {
    use std::{
        collections::VecDeque,
        thread::{sleep, Thread},
        time::Duration,
    };

    use super::*;

    struct MockSource {
        receivers: Watchers,
    }
    impl MockSource {
        pub fn new() -> Self {
            Self {
                receivers: Arc::new(Mutex::new(Vec::new())),
            }
        }
    }
    impl DataSource for MockSource {
        fn get_receivers(&self) -> Option<Watchers> {
            Some(self.receivers.clone())
        }
        fn set_receivers(&mut self, receivers: Watchers) {
            self.receivers = receivers.clone();
        }
    }

    struct MockReceiver {
        id: u8,
        buffer: Arc<Mutex<Vec<String>>>,
    }
    impl MockReceiver {
        pub fn new(id: u8, buffer: Arc<Mutex<Vec<String>>>) -> Self {
            Self { id, buffer }
        }

        pub fn get_id(&self) -> u8 {
            self.id
        }
    }
    impl DataReceiver for MockReceiver {
        fn process_message(&self, message: Message) {
            let mut buf = self.buffer.lock().unwrap();
            buf.push(format!("Message received from {}", self.get_id()));
        }
    }

    #[test]
    fn test_message_received() {
        let mut mock_source = MockSource::new();

        let test_buffer = Arc::new(Mutex::new(Vec::new()));
        let mock_receiver_1 = MockReceiver::new(1, test_buffer.clone());
        let mock_receiver_2 = MockReceiver::new(2, test_buffer.clone());
        let mock_receiver_3 = MockReceiver::new(3, test_buffer.clone());

        let mock_receivers: Arc<Mutex<Vec<Box<dyn DataReceiver + Send + 'static>>>> =
            Arc::new(Mutex::new(vec![
                Box::new(mock_receiver_1),
                Box::new(mock_receiver_2),
                Box::new(mock_receiver_3),
            ]));

        mock_source.set_receivers(mock_receivers);
        mock_source.notify_receivers(Message::Log {
            message: "Test".to_string(),
            message_type: LogType::Info,
        });

        sleep(Duration::from_secs(1));

        let buffer_data = test_buffer.lock().unwrap();
        assert!(buffer_data.contains(&"Message received from 1".to_string()));
        assert!(buffer_data.contains(&"Message received from 2".to_string()));
        assert!(buffer_data.contains(&"Message received from 3".to_string()));
    }
}
