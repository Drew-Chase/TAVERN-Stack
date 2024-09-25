use actix_files::{file_extension_to_mime};
use actix_web::{error::ErrorInternalServerError, get, middleware, web, App, HttpResponse, HttpServer, Responder};
use include_dir::{include_dir, Dir};
use serde_json::json;

// Include the wwwroot directory from the OUT_DIR
static WWWROOT: Dir = include_dir!("dist/wwwroot");

// Function to serve the index.html file
async fn index() -> Result<impl Responder, actix_web::Error> {
	if let Some(file) = WWWROOT.get_file("index.html") {
		let body = file.contents();
		return Ok(HttpResponse::Ok().content_type("text/html").body(body));
	}
	Err(ErrorInternalServerError("Failed to find index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	std::env::set_var("RUST_LOG", "trace");
	env_logger::init();

	let port = 1420; // Port to listen on

	println!(
		"Starting server at http://127.0.0.1:{port}",
		port = port,
	);

	HttpServer::new(move || {
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
						)
							.into()
					}),
			)
			// Handle API routes here
			.service(web::scope("api").service(status))
			.service(web::scope("assets/{file}").service(assets))
			// Handle all other routes by serving the index.html file
			.default_service(web::route().to(index))
	})
		.workers(4)
		.bind(format!("0.0.0.0:{port}", port = port))?
		.run()
		.await
}

#[get("/")]
async fn status() -> impl Responder {
	HttpResponse::Ok().json(json!({ "status": "ok" }))
}

#[get("")]
async fn assets(file: web::Path<String>) -> impl Responder {
	if let Some(file) = WWWROOT.get_file(format!("assets/{}", file.as_str())) {
		let body = file.contents();
		return Ok(HttpResponse::Ok().content_type(file_extension_to_mime(file.path().extension().unwrap().to_str().unwrap())).body(body));
	}
	Err(ErrorInternalServerError(format!("Failed to find {}", file)))
}