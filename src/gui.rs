use clipboard::{ClipboardContext, ClipboardProvider};
use crossbeam::channel::Sender;
use eframe::egui;
use egui::{ScrollArea, TextEdit};
use local_ip_address::local_ip;
use serde::{Serialize, Deserialize};

use crate::PORT;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeleprompterConfig {
	playing: bool,
	text: String,

	speed: f32,
	progress: f32,

	font: Font,
	font_size: i16,

	font_color: [f32; 3],
	background_color: [f32; 3],

	mirrored: bool,
	reversed: bool
}

impl Default for TeleprompterConfig {
	fn default() -> Self {
		TeleprompterConfig {
			playing: false,
			text: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string(),
	
			speed: 1.0,
			progress: 0.0,
			
			font: Font::Arial,
			font_size: 16,
	
			font_color: [1.0, 1.0, 1.0],
			background_color: [0.0, 0.0, 0.0],
	
			mirrored: false,
			reversed: false
		}
	}
}

#[derive(Debug, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub enum Font {
	Arial,
	Calibri,
	CourierNew,
	Georgia,
	Helvetica,
	TimesNewRoman,
	Verdana
}

impl ToString for Font {
	fn to_string(&self) -> String {
		match self {
			Font::Arial => "Arial".to_string(),
			Font::Calibri => "Calibri".to_string(),
			Font::CourierNew => "Courier New".to_string(),
			Font::Helvetica => "Helvetica".to_string(),
			Font::Georgia => "Georgia".to_string(),
			Font::TimesNewRoman => "Times New Roman".to_string(),
			Font::Verdana => "Verdana".to_string(),
		}
	}
}

pub fn init_gui(teleprompters_config_tx: Sender<TeleprompterConfig>) {
	let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([300.0, 530.0]),
        ..Default::default()
    };

	let mut config = TeleprompterConfig::default();
	let mut config_previous =  TeleprompterConfig::default();

	let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
	let mut clipboard_timeout: i8 = 0;

	let mut frame: u64 = 0;

    eframe::run_simple_native("Ignite Teleprompter Manager", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
			// request_repaint puts egui into "Continuous mode",
			// which means that the UI will be redrawn every frame
			// (which is needed for the progress counter to work properly)
			frame += 1;
			ctx.request_repaint();

			// Set to light mode
			// ui.ctx().set_visuals(egui::Visuals::light());

			// Update the progress every 30 frames if playing is true
			if config.playing == true {
				config.progress += config.speed / 100.0;
				
				if config.progress >= 100.0 {
					config.progress = 100.0;
					config.playing = false;
				} 
			}

            ui.heading("Ignite Teleprompter Manager");
			ui.horizontal(|ui| {
				// Get our IP address on the network that other devices use to connect to us
				// If the call fails, we default to 127.0.0.1
				let ip = local_ip().unwrap_or(std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)));

				ui.label("Teleprompter URL:");

				// The following code will copy the URL to the clipboard when the user clicks on it,
				// and will show a message for 60 frames saying that the URL was copied

				let url_text = match clipboard_timeout > 0 {
					true => "Copied to clipboard!".to_string(),
					false => format!("http://{}:{}", ip, PORT)
				};

				if ui.link(url_text).clicked() {
					clipboard_timeout = 60;
					clipboard.set_contents(format!("http://{}:{}", ip, PORT)).unwrap();
				}

				if clipboard_timeout > 0 {
					clipboard_timeout -= 1;
				}
			});

			ui.horizontal(|ui| {
				ui.label("Playing: ");
				ui.checkbox(&mut config.playing, "");
			});

			// Disable the progress slider if the teleprompter is playing
			ui.add_enabled_ui(!config.playing, |ui| {
				// Progress slider
				ui.horizontal(|ui| {
					let label = ui.label("Progress: ");
					ui.add(egui::Slider::new(&mut config.progress, 0.0..=100.0))
						.labelled_by(label.id);
				});
			});
			// Speed slider
			ui.horizontal(|ui| {
                let label = ui.label("Speed: ");
                ui.add(egui::Slider::new(&mut config.speed, 0.0..=10.0))
                    .labelled_by(label.id);
            });

			ui.separator();
			ui.heading("Text Options");

			// Font dropdown
			ui.horizontal(|ui| {
                ui.label("Font: ");
                egui::ComboBox::from_label("")
					.selected_text(format!("{}", config.font.to_string()))
					.show_ui(ui, |ui| {
						ui.style_mut().wrap = Some(false);
						ui.set_min_width(60.0);
						ui.selectable_value(&mut config.font, Font::Arial, "Arial");
						ui.selectable_value(&mut config.font, Font::Calibri, "Calibri");
						ui.selectable_value(&mut config.font, Font::CourierNew, "Courier New");
						ui.selectable_value(&mut config.font, Font::Georgia, "Georgia");
						ui.selectable_value(&mut config.font, Font::Helvetica, "Helvetica");
						ui.selectable_value(&mut config.font, Font::TimesNewRoman, "Times New Roman");
						ui.selectable_value(&mut config.font, Font::Verdana, "Verdana");
					});
            });

			// Font Size
			ui.horizontal(|ui| {
                let label = ui.label("Font Size: ");
                ui.add(egui::Slider::new(&mut config.font_size, 0..=256))
                    .labelled_by(label.id);
            });

			// Font Color
			ui.horizontal(|ui| {
				ui.label("Font Color: ");
				ui.color_edit_button_rgb(&mut config.font_color);
			});

			// Background Color
			ui.horizontal(|ui| {
				ui.label("Background Color: ");
				ui.color_edit_button_rgb(&mut config.background_color);
			});

			// Mirrored
			ui.horizontal(|ui| {
				ui.label("Mirrored: ");
				ui.checkbox(&mut config.mirrored, "");
			});

			// Reversed
			ui.horizontal(|ui| {
				ui.label("Reversed: ");
				ui.checkbox(&mut config.reversed, "");
			});

			ui.separator();

			// Scrollable textbox
			ScrollArea::vertical()
				.auto_shrink(false)
				.show(ui, |ui| {
					ui.add(
						TextEdit::multiline(&mut config.text)
							.desired_width(ui.available_width())
							.desired_rows(16)
					);
				});

        });

		// Send the teleprompter config on the channel, but only if it has changed
		if config != config_previous {
			// If only the progress has changed (while playing), only send it every 30 frames
			if config.progress == config_previous.progress || config.playing == false {
				teleprompters_config_tx.send(config.clone()).unwrap();
				config_previous = config.clone();
			} else if frame % 30 == 0 {
				teleprompters_config_tx.send(config.clone()).unwrap();
				config_previous = config.clone();
			}
		}
    }).unwrap();
}