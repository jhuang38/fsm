use super::Message;

pub mod logger;
pub mod writer;

pub trait DataReceiver {
    fn process_message(&self, message: Message);
}
