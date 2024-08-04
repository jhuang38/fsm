use std::{thread, time::Duration};

use axum::{
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use log::info;
use serde::{Deserialize, Serialize};

use crate::AppState;

pub mod logs;
pub mod messages;

#[derive(Serialize, Deserialize)]
struct Test {
    pub name: String,
}

pub fn init_app(state: AppState) -> Router {
    let app = Router::new()
        .route("/fsm/ws", get(handle_ws))
        .with_state(state);

    app
}

async fn handle_ws(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    //todo - add some verification here  for ws
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    // initial ping
    if socket.send(Message::Ping(vec![1, 2, 3])).await.is_ok() {
        info!("Pinged succesfully!");
    } else {
        return;
    }
    //receive messages from client
    let messages = state.dashboard_messages.clone();
    let mut messages = messages.lock().await;
    loop {
        match messages.try_recv() {
            Ok(message) => {
                if socket.send(Message::Text(message)).await.is_err() {
                    return;
                }
            }
            Err(_) => {}
        }
        tokio::time::sleep(Duration::from_secs(5)).await;
    }
}
