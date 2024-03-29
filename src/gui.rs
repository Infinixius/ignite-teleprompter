use std::sync::mpsc;

use clipboard::{ClipboardContext, ClipboardProvider};
use eframe::egui;
use egui::{Color32, ScrollArea, TextEdit, Vec2};
use local_ip_address::local_ip;
use serde::{Serialize, Deserialize};

use crate::{{ADDRESS, WEB_PORT}};
use crate::ws_server::Teleprompter;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeleprompterConfig {
	playing: bool,
	text: String,

	speed: i8,
	progress: i8,

	font: Font,
	font_size: i8,

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
	
			speed: 0,
			progress: 0,
			
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
	ComicSansMS,
	Georgia,
	TimesNewRoman,
	Verdana
}

impl ToString for Font {
	fn to_string(&self) -> String {
		match self {
			Font::Arial => "Arial".to_string(),
			Font::Calibri => "Calibri".to_string(),
			Font::CourierNew => "Courier New".to_string(),
			Font::ComicSansMS => "Comic Sans MS".to_string(),
			Font::Georgia => "Georgia".to_string(),
			Font::TimesNewRoman => "Times New Roman".to_string(),
			Font::Verdana => "Verdana".to_string(),
		}
	}
}

pub fn init_gui(teleprompters_config_tx: mpsc::Sender<TeleprompterConfig>) {
	let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([300.0, 530.0]),
        ..Default::default()
    };

	let mut teleprompter_config_previous =  TeleprompterConfig::default();
	let mut teleprompter_config = TeleprompterConfig::default();

	let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
	let mut clipboard_timeout: i8 = 0;

    eframe::run_simple_native("Ignite Teleprompter Manager", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
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
					false => format!("http://{}:{}", ip, WEB_PORT)
				};

				if ui.link(url_text).clicked() {
					clipboard_timeout = 60;
					clipboard.set_contents(format!("http://{}:{}", ip, WEB_PORT)).unwrap();
				}

				if clipboard_timeout > 0 {
					clipboard_timeout -= 1;
				}
			});

			ui.horizontal(|ui| {
				ui.label("Playing: ");
				ui.checkbox(&mut teleprompter_config.playing, "");
			});

			// Progress slider
			ui.horizontal(|ui| {
                let label = ui.label("Progress: ");
                ui.add(egui::Slider::new(&mut teleprompter_config.progress, 0..=100))
                    .labelled_by(label.id);
            });

			// Speed slider
			ui.horizontal(|ui| {
                let label = ui.label("Speed: ");
                ui.add(egui::Slider::new(&mut teleprompter_config.speed, 0..=100))
                    .labelled_by(label.id);
            });

			ui.separator();
			ui.heading("Text Options");

			// Font dropdown
			ui.horizontal(|ui| {
                ui.label("Font: ");
                egui::ComboBox::from_label("")
					.selected_text(format!("{}", teleprompter_config.font.to_string()))
					.show_ui(ui, |ui| {
						ui.style_mut().wrap = Some(false);
						ui.set_min_width(60.0);
						ui.selectable_value(&mut teleprompter_config.font, Font::Arial, "Arial");
						ui.selectable_value(&mut teleprompter_config.font, Font::Calibri, "Calibri");
						ui.selectable_value(&mut teleprompter_config.font, Font::CourierNew, "Courier New");
						ui.selectable_value(&mut teleprompter_config.font, Font::ComicSansMS, "Comic Sans MS");
						ui.selectable_value(&mut teleprompter_config.font, Font::Georgia, "Georgia");
						ui.selectable_value(&mut teleprompter_config.font, Font::TimesNewRoman, "Times New Roman");
						ui.selectable_value(&mut teleprompter_config.font, Font::Verdana, "Verdana");
					});
            });

			// Font Size
			ui.horizontal(|ui| {
                let label = ui.label("Font Size: ");
                ui.add(egui::Slider::new(&mut teleprompter_config.font_size, 0..=100))
                    .labelled_by(label.id);
            });

			// Font Color
			ui.horizontal(|ui| {
				ui.label("Font Color: ");
				ui.color_edit_button_rgb(&mut teleprompter_config.font_color);
			});

			// Background Color
			ui.horizontal(|ui| {
				ui.label("Background Color: ");
				ui.color_edit_button_rgb(&mut teleprompter_config.background_color);
			});

			// Mirrored
			ui.horizontal(|ui| {
				ui.label("Mirrored: ");
				ui.checkbox(&mut teleprompter_config.mirrored, "");
			});

			// Reversed
			ui.horizontal(|ui| {
				ui.label("Reversed: ");
				ui.checkbox(&mut teleprompter_config.reversed, "");
			});

			ui.separator();

			ScrollArea::vertical()
				.auto_shrink(false)
				.show(ui, |ui| {
					ui.add(
						TextEdit::multiline(&mut teleprompter_config.text)
							.desired_width(ui.available_width())
							.desired_rows(16)
					);
				});

        });

		// Send the teleprompter config on the channel, but only if it has changed
		if teleprompter_config != teleprompter_config_previous {
			teleprompters_config_tx.send(teleprompter_config.clone()).unwrap();
			teleprompter_config_previous = teleprompter_config.clone();
		}
    }).unwrap();
}