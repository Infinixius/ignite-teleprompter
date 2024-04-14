use futures_util::{SinkExt, StreamExt};
use local_ip_address::local_ip;
use std::sync::{Arc, RwLock};
use tokio::sync::{mpsc, broadcast::{Sender,Receiver}};
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, Ws, WebSocket};
use warp::Filter;

use crate::gui::TeleprompterConfig;

#[derive(Debug, Clone)]
pub struct Teleprompter {
	pub ip: String,
	pub connected_at: i64,
}

#[allow(non_snake_case)]
pub async fn start_server(teleprompters_config_bus: Sender<TeleprompterConfig>, ADDRESS: String, PORT: u16) {
    let initial_config: Arc<RwLock<TeleprompterConfig>> = Arc::new(RwLock::new(TeleprompterConfig::default()));
    let mut initial_config_rx = teleprompters_config_bus.subscribe();

    // Non blocking while loop with initial config
    let initial_config_clone = initial_config.clone();
    tokio::task::spawn(async move {
        while let Ok(msg) = initial_config_rx.recv().await {
            let mut writer = initial_config_clone.write().expect("Failed to get write lock on initial config");

            *writer = msg;
        }
    });

    // Logs information on every request
    let logger = warp::log::custom(|info| {
        log!(
            "{:?} -> {:?} {} \"{}\" (Response: {}) (User-Agent: \"{:?}\")", 
            info.remote_addr(),
            
            info.version(),
            info.method(),
            info.path(),

            info.status(),

            info.user_agent(),

        );
    });

    let ws = warp::path("ws")
        .and(warp::ws())
        .map(move |ws: Ws| {
            let teleprompter_config_rx = teleprompters_config_bus.subscribe();
            let initial_config_clone = initial_config.read().expect("Failed to get read lock on initial config").clone();

            ws.on_upgrade(move |socket| websocket_user_connected(socket, teleprompter_config_rx, initial_config_clone))
        });

    // GET / -> index html
    let html: String = include_str!("../assets/teleprompter.html").replace("%PORT%", &PORT.to_string()); // Automatically replace the port in the html with the one we are using
    let index = warp::path::end().map(move || warp::reply::html(html.clone()));

    let routes = index.or(ws).with(logger);
    let server_address = std::net::SocketAddr::new(ADDRESS.parse().expect("Failed to parse server_address"), PORT);

    log!("Server listening at: http://{} (Remote IP: http://{}:{})", server_address, local_ip().unwrap_or(std::net::Ipv4Addr::new(127, 0, 0, 1).into()), PORT);
    
    warp::serve(routes).run(server_address).await;
}

async fn websocket_user_connected(ws: WebSocket, mut teleprompter_config_rx: Receiver<TeleprompterConfig>, initial_config: TeleprompterConfig) {
    // Split the socket into a sender and receive of messages.
    let (mut user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (_tx, rx) = mpsc::unbounded_channel::<Message>();
    let mut _rx = UnboundedReceiverStream::new(rx);

    // // Send the initial config to the user
    let message = Message::text(serde_json::to_string(&initial_config).expect("Failed to serialize initial config"));
    user_ws_tx.send(message).await.expect("Failed to send initial config to user");

    tokio::task::spawn(async move {
        while let Ok(msg) = teleprompter_config_rx.recv().await {
            // println!("Received message: {:?}", msg);
            let message = Message::text(serde_json::to_string(&msg).expect("Failed to serialize message"));

            if let Err(e) = user_ws_tx.send(message).await {
                log!("Websocket send error: {}", e);
                return;
            }
        } 
    });

    while let Some(result) = user_ws_rx.next().await {
        match result {
            Ok(_msg) => {},
            Err(err) => {
                log!("Websocket error: {}", err);
                break;
            }
        };
    }

    // This code will run once the connection closes
    log!("Websocket user disconnected with IP: ??", );
}