use std::net;
use std::thread;
use std::sync::mpsc;

use chrono;
use tungstenite;

use crate::{ADDRESS, WS_PORT};

use crate::gui::TeleprompterConfig;

#[derive(Debug, Clone)]
pub struct Teleprompter {
	pub ip: String,
	pub connected_at: i64,
}

pub fn init_ws_server(teleprompters_config_rx: mpsc::Receiver<TeleprompterConfig>) {
	let server = net::TcpListener::bind((ADDRESS, WS_PORT)).unwrap();
	let clients: Vec<Teleprompter> = Vec::new();

	log!("Websocket server started at ws://{}:{}", ADDRESS, WS_PORT);

	for stream in server.incoming() {
		let ip = stream.as_ref().unwrap().peer_addr().unwrap();
		log!("Websocket client connected with IP: {}", ip);

		let mut clients = clients.clone();

        thread::spawn(move || {
			let mut websocket = tungstenite::accept(stream.unwrap()).unwrap();

			clients.push(Teleprompter {
				ip: ip.to_string(),
				connected_at: chrono::Utc::now().timestamp(),
			});

            loop {
				match websocket.read() {
					Ok(msg) => {
						// We do not want to send back ping/pong messages.
						if msg.is_text() {
							log!("Received websocket data ({}): {:?}", ip, msg);
							
							// let state = state.lock().unwrap();
							// let options = serde_json::to_string(&state.options).unwrap();

							// websocket.send(tungstenite::Message::Text(options)).unwrap();
						}
					}
					Err(e) => {
						log!("Websocket client disconnected ({}): {:?}", ip, e);
						clients.retain(|client| client.ip != ip.to_string());

						break;
					}
				}
			}
        });
    }
}