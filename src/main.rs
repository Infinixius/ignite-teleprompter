#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#[warn(clippy::unwrap_used)]

#[macro_use]
mod macros;

mod gui;
mod server;

use crate::gui::TeleprompterConfig;
use tokio::sync::broadcast;

// TODO: Allow custom port/address via commandline or env
static ADDRESS: &str = "127.0.0.1";
static PORT: u16 = 29501;
static DEBUG_LATENCY: u128 = 50; // Set to zero to disable

#[tokio::main]
async fn main() {
	let (teleprompters_config_tx, mut _teleprompters_config_rx) = broadcast::channel::<TeleprompterConfig>(16);

	// TODO: On panic, if the GUI is running, show a dialog with the panic message
	// Set custom panic hook to end the process when any thread panics
	std::panic::set_hook(Box::new(|panic_info: &std::panic::PanicInfo| {
		println!("Panic occurred: {:?}", panic_info);
		std::process::exit(1);
	}));

	let teleprompers_config_tx_clone = teleprompters_config_tx.clone();
	tokio::spawn(async move {
		server::start_server(teleprompers_config_tx_clone).await;
	});

	// Running egui in a thread is not supported, so we just run it on main
	gui::init_gui(teleprompters_config_tx.clone());
}
