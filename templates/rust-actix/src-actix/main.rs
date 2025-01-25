use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use serde_json::json;
use log::*;
use anyhow::Result;
use crate::asset_endpoint::{start_vite_server, AppConfig};

mod asset_endpoint;
mod test_endpoint;
mod http_error;

pub static DEBUG: bool = cfg!(debug_assertions);
const PORT: u16 = 1421;

#[actix_web::main]
async fn main() -> Result<()> {
	std::env::set_var("RUST_LOG", "debug");
	env_logger::init();

	let server = HttpServer::new(move || {
		App::new()
			.wrap(middleware::Logger::default())
			.app_data(
				web::JsonConfig::default()
					.limit(4096)
					.error_handler(|err, _req| {
						let error = json!({ "error": format!("{}", err) });
						actix_web::error::InternalError::from_response(
							err,
							HttpResponse::BadRequest().json(error),
						).into()
					})
			)
			.service(
				web::scope("api")
					.configure(test_endpoint::configure)
			)

			.configure_routes()
	})
		.workers(4)
		.bind(format!("0.0.0.0:{port}", port = PORT))?
		.run();

	info!(
        "Starting {} server at http://127.0.0.1:{}...",
        if DEBUG { "development" } else { "production" },
        PORT
    );



	if DEBUG {
		start_vite_server().expect("Failed to start vite server");
	}


	let stop_result = server.await;
	debug!("Server stopped");

	Ok(stop_result?)
}
