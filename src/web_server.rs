use std::collections::HashMap;
use web_server::{self, Response};
use std::{sync::Arc, sync::Mutex};

use crate::{ADDRESS, WEB_PORT, WS_PORT};

pub fn init_web_server() {
	log!("HTTP server started at http://{}:{}", ADDRESS, WEB_PORT);

	let html: String = include_str!("../web/teleprompter.html").replace("%PORT%", &WS_PORT.to_string());

	web_server::new()
		.get("/", Box::new(move |request: web_server::Request, mut _response: web_server::Response| {
			Response {
				response_code: web_server::HttpCode::_200,
				http_version: request.get_http_version(),
				headers: HashMap::new(),
				body: web_server::Body::S(html.clone()),
			}
		}))
		.launch(WEB_PORT.into());
}