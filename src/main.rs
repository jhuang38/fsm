use std::io::{self};


use env_logger;

use fsm::error::FsmError;
use fsm::init_fsm;
use log::info;

fn main() -> Result<(), FsmError> {
    env_logger::init();
    let app_state = init_fsm("fsm_config.json")?;
    info!("Running FSM. Provide stdin to modify app behavior.");

    loop {
        let mut buffer = String::new();
        let _ = match io::stdin().read_line(&mut buffer) {
            Ok(res) => res,
            Err(_) => continue,
        };
        match buffer.trim().to_lowercase().as_str() {
            "q" | "quit" => {
                break;
            }
            _ => continue,
        };
    }

    info!("Exiting");
    Ok(())
}
