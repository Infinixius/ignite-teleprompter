use std::net::IpAddr;

use crossbeam::channel::Receiver;
use futures_util::{SinkExt, StreamExt, TryFutureExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, Ws, WebSocket};
use warp::Filter;

use crate::gui::TeleprompterConfig;

use crate::{{ADDRESS, PORT}};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct Teleprompter {
	pub ip: String,
	pub connected_at: i64,
}

#[tokio::main]
pub async fn start_server(teleprompters_config_rx: Receiver<TeleprompterConfig>) {
    let html: String = include_str!("./teleprompter.html").replace("%PORT%", &PORT.to_string());
    let teleprompters_config_rx_clone = Arc::new(teleprompters_config_rx.clone());
    // TODO: We should be caching the config to send it to newly connected clients

    let chat = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: Ws| {
            let teleprompters_config_rx_clone = Arc::clone(&teleprompters_config_rx_clone);

            ws.on_upgrade(move |socket| websocket_user_connected(socket, teleprompters_config_rx_clone))
        });

    // GET / -> index html
    let index = warp::path::end().map(move || warp::reply::html(html.clone()));

    let routes = index.or(chat);

    warp::serve(routes).run(std::net::SocketAddr::new(ADDRESS.parse().unwrap(), PORT)).await;
}

async fn websocket_user_connected(ws: WebSocket, teleprompters_config_rx: Arc<Receiver<TeleprompterConfig>>) {
    let _ip: IpAddr; // TODO: Get the user's IP
    log!("WebSocket user connected with IP: ??", );

    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (_tx, rx) = mpsc::unbounded_channel::<Message>();
    let mut _rx = UnboundedReceiverStream::new(rx);


    tokio::task::spawn(async move {
        while let Ok(msg) = teleprompters_config_rx.recv() {
            log!("Received config: {:?}", msg);

            let message = Message::text(serde_json::to_string(&msg).unwrap());

            user_ws_tx
                .send(message)
                .unwrap_or_else(|e| {
                    log!("websocket send error: {}", e);
                })
                .await;
        }        
    });

    while let Some(result) = user_ws_rx.next().await {
        match result {
            Ok(msg) => log!("Received message: {:?}", msg),
            Err(err) => {
                log!("Websocket error: {}", err);
                break;
            }
        };
    }

    // This code will run once the connection closes
    log!("Websocket user disconnected with IP: ??", );
}