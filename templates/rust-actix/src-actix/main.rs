use actix_web::{get, middleware, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use awc::Client;
use futures_util::stream::StreamExt;
use include_dir::{include_dir, Dir};
use serde_json::json;
use log::info;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	std::env::set_var("RUST_LOG", "trace");
	env_logger::init();

	let port = 1420; // Port to listen on
	let config = if cfg!(debug_assertions) {
		"development"
	} else {
		"production"
	};

	info!("Starting {} server at http://127.0.0.1:{}", config, port);


	HttpServer::new(move || {
		let app = App::new()
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
			.service(web::scope("api").service(status));

		// Add conditional routing based on the config
		if config == "development" {
			app.default_service(web::route().to(proxy_to_vite))
			   .service(
				   web::resource("/assets/{file:.*}")
					   .route(web::get().to(proxy_to_vite))
			   )
			   .service(
				   web::resource("/node_modules/{file:.*}")
					   .route(web::get().to(proxy_to_vite))
			   )
		} else {
			app.default_service(web::route().to(index))
			   .service(web::resource("/assets/{file:.*}").route(web::get().to(index)))
		}
	})
		.workers(4)
		.bind(format!("0.0.0.0:{port}", port = port))?
		.run()
		.await
}


/// The maximum payload size allowed for forwarding requests and responses.
///
/// This constant defines the maximum size (in bytes) for the request and response payloads
/// when proxying. Any payload exceeding this size will result in an error.
///
/// Currently, it is set to 1 GB.
const MAX_PAYLOAD_SIZE: usize = 1024 * 1024 * 1024; // 1 GB

/// Static directory including all files under `target/wwwroot`.
///
/// This static directory is used to embed files into the binary at compile time.
/// The `WWWROOT` directory will be used to serve static files such as `index.html`.
static WWWROOT: Dir = include_dir!("target/wwwroot");
/// Handles the request for the index.html file.
///
/// This function serves the `index.html` file from the embedded directory
/// if it exists, and returns an internal server error if the file is not found.
///
/// # Arguments
///
/// * `_req` - The HTTP request object.
///
/// # Returns
///
/// An `impl Responder` which can either be a successful HTTP response containing
/// the `index.html` file, or an internal server error.
async fn index(_req: HttpRequest) -> Result<impl Responder, Error> {
	if let Some(file) = WWWROOT.get_file("index.html") {
		let body = file.contents();
		return Ok(HttpResponse::Ok().content_type("text/html").body(body));
	}
	Err(actix_web::error::ErrorInternalServerError("Failed to find index.html"))
}

/// Proxies requests to the Vite development server.
///
/// This function forwards incoming requests to a local Vite server running on port 3000.
/// It buffers the entire request payload and response payload to avoid partial transfers.
/// Requests and responses larger than the maximum payload size will result in an error.
///
/// # Arguments
///
/// * `req` - The HTTP request object.
/// * `payload` - The request payload.
///
/// # Returns
///
/// An `HttpResponse` which contains the response from the Vite server,
/// or an error response in case of failure.
async fn proxy_to_vite(req: HttpRequest, mut payload: web::Payload) -> Result<HttpResponse, Error> {
	let client = Client::new();
	let forward_url = format!("http://localhost:3000{}", req.uri());

	// Buffer the entire payload
	let mut body_bytes = web::BytesMut::new();
	while let Some(chunk) = payload.next().await {
		let chunk = chunk?;
		if (body_bytes.len() + chunk.len()) > MAX_PAYLOAD_SIZE {
			return Err(actix_web::error::ErrorPayloadTooLarge(
				"Payload overflow",
			));
		}
		body_bytes.extend_from_slice(&chunk);
	}

	let mut forwarded_resp = client
		.request_from(forward_url.as_str(), req.head())
		.no_decompress()
		.send_body(body_bytes)
		.await
		.map_err(|err| {
			actix_web::error::ErrorInternalServerError(format!(
				"Failed to forward request: {}",
				err
			))
		})?;

	// Buffer the entire response body
	let mut resp_body_bytes = web::BytesMut::new();
	while let Some(chunk) = forwarded_resp.next().await {
		let chunk = chunk?;
		if (resp_body_bytes.len() + chunk.len()) > MAX_PAYLOAD_SIZE {
			return Err(actix_web::error::ErrorPayloadTooLarge(
				"Response payload overflow",
			));
		}
		resp_body_bytes.extend_from_slice(&chunk);
	}

	// Build the response
	let mut res = HttpResponse::build(forwarded_resp.status());

	// Copy headers
	for (header_name, header_value) in forwarded_resp.headers().iter() {
		res.insert_header((header_name.clone(), header_value.clone()));
	}

	Ok(res.body(resp_body_bytes))
}

/// Handles requests to check the server status.
///
/// This endpoint responds to GET requests with a JSON object indicating
/// that the server is running correctly. It can be used for health checks
/// or monitoring server status.
///
/// # Returns
///
/// A JSON object with a `status` field set to "ok".
#[get("/")]
async fn status() -> impl Responder {
	HttpResponse::Ok().json(json!({ "status": "ok" }))
}