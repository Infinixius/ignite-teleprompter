#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use std::sync::mpsc;
use std::thread;

#[macro_use]
mod macros;

mod gui;
mod web_server;
mod ws_server;

static ADDRESS: &str = "127.0.0.1";
static WEB_PORT: u16 = 29501;
static WS_PORT:  u16 = 29502;

fn main() {
	let (teleprompters_config_tx, teleprompters_config_rx) = mpsc::channel();

	// Set custom panic hook to end the process when any thread panics
	std::panic::set_hook(Box::new(|panic_info: &std::panic::PanicInfo| {
		println!("Panic occurred: {:?}", panic_info);
		std::process::exit(1);
	}));
 

	// thread::spawn(|| { web_server::init_web_server(); });

	// thread::spawn(|| { ws_server::init_ws_server(teleprompters_config_rx); });

	thread::spawn(move || {
		loop {
			if let Ok(config) = teleprompters_config_rx.recv() {
				println!("Received config: {:?}", config);
			}
		}
	});

	// Running egui in a thread is not supported, so we just run it on main
	gui::init_gui(teleprompters_config_tx);
}
