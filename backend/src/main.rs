use std::io::Read;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use axum::ServiceExt;
use env_logger;

use fsm_backend::dashboard::init_app;
use fsm_backend::error::FsmError;
use fsm_backend::{init_fsm_managers, AppState};
use log::info;

#[tokio::main]
async fn main() -> Result<(), FsmError> {
    env_logger::init();
    let app_state = init_fsm_managers("fsm_config.json")?;
    info!("Running FSM");

    let fsm_app = init_app(app_state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:5000")
        .await
        .unwrap();
    info!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(
        listener,
        fsm_app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .unwrap();

    info!("Exiting");
    Ok(())
}
