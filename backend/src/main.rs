use std::io::Read;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use env_logger;

use fsm_backend::error::FsmError;
use fsm_backend::{init_fsm_managers, AppState};
use log::info;

fn main() -> Result<(), FsmError> {
    env_logger::init();
    let app_state = init_fsm_managers("fsm_config.json")?;
    info!("Running FSM");

    // stdin controls
    loop {}
    info!("Exiting");
    Ok(())
}
