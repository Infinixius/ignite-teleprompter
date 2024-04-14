#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![deny(clippy::unwrap_used)]

#[macro_use]
mod macros;

mod gui;
mod server;

use crate::gui::TeleprompterConfig;
use getopts::Options;
use tokio::sync::broadcast;


#[allow(non_snake_case)]
#[tokio::main]
async fn main() {
	// Parse command-line arguments
	let args: Vec<String> = std::env::args().collect();
	let mut opts = Options::new();
	opts.optflag("h", "help", "Print this help menu");
	opts.optflag("d", "debug", "Enable debug mode");
	opts.optopt("p", "port", "Set the port to listen on (default: 29501)", "PORT");
	opts.optopt("a", "address", "Set the address to listen on (default: 127.0.0.1)", "ADDRESS");

	let matches = match opts.parse(&args[1..]) {
		Ok(m) => m,
		Err(f) => {
			eprintln!("{}", f.to_string());
			std::process::exit(1);
		}
	};

	if matches.opt_present("h") {
		println!("{}", opts.usage("Usage: teleprompter [options]"));
		std::process::exit(0);
	}

	let mut ADDRESS: String = String::from("127.0.0.1");
	let mut PORT: u16 = 29501;
	let mut DEBUG: bool = false;

	if matches.opt_present("d") {
		println!("Debug mode enabled");
		DEBUG = true;
	}
	if matches.opt_present("p") {
		PORT = matches.opt_str("p").unwrap().parse().unwrap();
	}
	if matches.opt_present("a") {
		ADDRESS = matches.opt_str("a").unwrap();
	}

	let (teleprompters_config_tx, mut _teleprompters_config_rx) = broadcast::channel::<TeleprompterConfig>(16);

	// TODO: On panic, if the GUI is running, show a dialog with the panic message
	// Set custom panic hook to end the process when any thread panics
	std::panic::set_hook(Box::new(|panic_info: &std::panic::PanicInfo| {
		println!("Panic occurred: {:?}", panic_info);
		std::process::exit(1);
	}));

	let teleprompers_config_tx_clone = teleprompters_config_tx.clone();
	tokio::spawn(async move {
		server::start_server(teleprompers_config_tx_clone, ADDRESS, PORT).await;
	});

	// Running egui in a thread is not supported, so we just run it on main
	gui::init_gui(teleprompters_config_tx.clone(), DEBUG, PORT);
}
