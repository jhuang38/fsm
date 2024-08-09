use super::{Message, Watchers};

pub mod sweep;
pub mod watch;

pub fn notify_receivers(message: Message, receivers: Watchers) {
    let receivers = match receivers.lock() {
        Err(_) => return,
        Ok(res) => res,
    };
    for receiver in receivers.iter() {
        // ideally it would be nice to avoid cloning here
        receiver.as_ref().process_message(message.clone());
    }
}

pub trait DataSource {
    fn get_receivers(&self) -> Option<Watchers>;
    fn notify_receivers(&self, message: Message) {
        match &self.get_receivers() {
            Some(receivers) => {
                notify_receivers(message, receivers.clone());
            }
            None => {}
        }
    }
    fn set_receivers(&mut self, receivers: Watchers);
}
