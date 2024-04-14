use clipboard::{ClipboardContext, ClipboardProvider};
use eframe::egui;
use egui::{IconData, ScrollArea, TextEdit, Vec2};
use image;
use local_ip_address::local_ip;
use serde::{Serialize, Deserialize};
use std::time::Instant;
use tokio::sync::broadcast::Sender;

use crate::{DEBUG_LATENCY, PORT};
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TeleprompterConfig {
	playing: bool,
	text: String,

	speed: f32,
	progress: f32,

	align: Align,
	font: Font,
	font_size: i16,

	font_color: [f32; 3],
	background_color: [f32; 3],

	mirrored: bool,
	reversed: bool,

	// TODO: Debugging mode, set by the command-line flag "--debug"
	// Not intended for normal use
	debug: bool,
}

impl Default for TeleprompterConfig {
	fn default() -> Self {
		TeleprompterConfig {
			playing: false,
			text: "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_string(),
	
			speed: 1.0,
			progress: 0.0,
			
			align: Align::Left,
			font: Font::Arial,
			font_size: 128,
	
			font_color: [1.0, 1.0, 1.0],
			background_color: [0.0, 0.0, 0.0],
	
			mirrored: false,
			reversed: false,

			debug: true,
		}
	}
}

impl TeleprompterConfig {
	fn only_progress_changed(self: &TeleprompterConfig, other: &TeleprompterConfig) -> bool {
		if self.playing == other.playing &&
		   self.text == other.text &&
		   self.speed == other.speed &&
		   self.progress != other.progress &&
		   self.font == other.font &&
		   self.font_size == other.font_size &&
		   self.font_color == other.font_color &&
		   self.background_color == other.background_color &&
		   self.align == other.align &&
		   self.mirrored == other.mirrored &&
		   self.reversed == other.reversed {
			true
		} else {
			false
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

#[derive(Debug, Hash, PartialEq, Clone, Serialize, Deserialize)]
pub enum Align {
	Left,
	Center,
	Right,
	Justify
}

impl ToString for Align {
	fn to_string(&self) -> String {
		match self {
			Align::Left => "Left".to_string(),
			Align::Center => "Center".to_string(),
			Align::Right => "Right".to_string(),
			Align::Justify => "Justify".to_string(),
		}
	}
}

// Helper function for loading the icon
fn load_icon() -> IconData {
    let (icon_rgba, icon_width, icon_height) = {
        let icon = include_bytes!("../assets/icon.png");
		let image = image::load_from_memory(icon)
            .expect("Failed to open icon path")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    IconData {
        rgba: icon_rgba,
        width: icon_width,
        height: icon_height,
    }
}

pub fn init_gui(teleprompters_config_bus: Sender<TeleprompterConfig>) {
	let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder {
			inner_size: Some(Vec2::new(300.0, 530.0)),
			title: Some("Teleprompter Manager".to_owned()),
			icon: Some(load_icon().into()),
			..Default::default()
		
		},
        ..Default::default()
    };

	let mut config = TeleprompterConfig::default();
	let mut config_previous =  TeleprompterConfig::default();

	let mut clipboard: ClipboardContext = ClipboardProvider::new().unwrap();
	let mut clipboard_timeout: i8 = 0;

	let mut frame: u128 = 0;
	let mut last_frame = Instant::now();
	let mut time_elapsed: u128 = 0; // Represents the time elapsed since the program started

    eframe::run_simple_native("Teleprompter Manager", options, move |ctx, _frame| {
        egui::CentralPanel::default().show(ctx, |ui| {
			// request_repaint puts egui into "Continuous mode",
			// which means that the UI will be redrawn every frame
			// (which is needed for the progress counter to work properly)
			ctx.request_repaint();
			
			// Calculate the time since the last frame
			frame += 1;
			let now = Instant::now();
			let last_frame_time = now - last_frame;
			
			last_frame = now;
			time_elapsed += last_frame_time.as_millis();

			// Set to dark mode
			ui.ctx().set_visuals(egui::Visuals::dark());

			// Update the progress every five milliseconds
			if config.playing == true && time_elapsed % 5 == 0 {
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
                egui::ComboBox::from_id_source("dropdown_font")
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

			// Align
			ui.horizontal(|ui| {
                ui.label("Align: ");
                egui::ComboBox::from_id_source("dropdown_align")
					.selected_text(format!("{}", config.align.to_string()))
					.show_ui(ui, |ui| {
						ui.style_mut().wrap = Some(false);
						ui.set_min_width(60.0);
						ui.selectable_value(&mut config.align, Align::Left, "Left");
						ui.selectable_value(&mut config.align, Align::Center, "Center");
						ui.selectable_value(&mut config.align, Align::Right, "Right");
						ui.selectable_value(&mut config.align, Align::Justify, "Justify");
					});
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
			// If only the progress has changed (while playing), only send it every 50 ms
			if config.playing && !config.only_progress_changed(&config_previous) ||
			   config.playing && config.only_progress_changed(&config_previous) && time_elapsed % DEBUG_LATENCY == 0 ||
			  !config.playing {
				teleprompters_config_bus.send(config.clone()).unwrap();
				config_previous = config.clone();
			}
		}
    }).unwrap();
}