use tokio::sync::mpsc::Sender;

use crate::{
    datasource::{DataReceiver, ReceiverType},
    error::{ErrorType, FsmError},
};

pub struct DashboardMessageManager {
    send_channel: Sender<String>,
}

impl DashboardMessageManager {
    pub fn new(send_channel: Sender<String>) -> Self {
        Self { send_channel }
    }
}

impl DataReceiver for DashboardMessageManager {
    fn accept_data(
        &mut self,
        message: &crate::datasource::DataMessage,
    ) -> Result<(), crate::error::FsmError> {
        let new_message = format!(
            "Attempting to move {:?} to {:?}",
            &message.from_path, &message.to_path
        )
        .to_string();
        // run async function via tokio runtime
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(res) => res,
            Err(e) => return Err(FsmError::new(ErrorType::ApplicationError, e.to_string())),
        };

        match rt.block_on(self.send_channel.send(new_message)) {
            Ok(_) => Ok(()),
            Err(e) => Err(FsmError::new(ErrorType::ApplicationError, e.to_string())),
        }
    }
    fn receiver_type(&self) -> ReceiverType {
        ReceiverType::DashboardMessages
    }
}
