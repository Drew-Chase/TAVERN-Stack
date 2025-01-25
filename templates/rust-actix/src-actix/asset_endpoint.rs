use std::process::Child;
use actix_files::file_extension_to_mime;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, Responder};
use actix_web::error::ErrorInternalServerError;
use awc::Client;
use futures_util::StreamExt;
use include_dir::{include_dir, Dir};
use log::{debug, error};
use crate::DEBUG;

// The maximum payload size allowed for forwarding requests and responses.
//
// This constant defines the maximum size (in bytes) for the request and response payloads
// when proxying. Any payload exceeding this size will result in an error.
//
// Currently, it is set to 1 GB.
const MAX_PAYLOAD_SIZE: usize = 1024 * 1024 * 1024; // 1 GB

// Static directory including all files under `target/wwwroot`.
//
// This static directory is used to embed files into the binary at compile time.
// The `WWWROOT` directory will be used to serve static files such as `index.html`.
static WWWROOT: Dir = include_dir!("target/wwwroot");
// Handles the request for the index.html file.
//
// This function serves the `index.html` file from the embedded directory
// if it exists, and returns an internal server error if the file is not found.
//
// # Arguments
//
// * `_req` - The HTTP request object.
//
// # Returns
//
// An `impl Responder` which can either be a successful HTTP response containing
// the `index.html` file, or an internal server error.
pub async fn index(_req: HttpRequest) -> anyhow::Result<impl Responder, Error> {
	if let Some(file) = WWWROOT.get_file("index.html") {
		let body = file.contents();
		return Ok(HttpResponse::Ok().content_type("text/html").body(body));
	}
	Err(ErrorInternalServerError("Failed to find index.html"))
}

#[get("")]
async fn assets(file: web::Path<String>) -> impl Responder {
	if let Some(file) = WWWROOT.get_file(format!("assets/{}", file.as_str())) {
		let body = file.contents();
		return Ok(HttpResponse::Ok()
			.content_type(file_extension_to_mime(
				file.path().extension().unwrap().to_str().unwrap(),
			))
			.body(body));
	}
	Err(ErrorInternalServerError(format!("Failed to find {}", file)))
}

// Proxies requests to the Vite development server.
//
// This function forwards incoming requests to a local Vite server running on port 3000.
// It buffers the entire request payload and response payload to avoid partial transfers.
// Requests and responses larger than the maximum payload size will result in an error.
//
// # Arguments
//
// * `req` - The HTTP request object.
// * `payload` - The request payload.
//
// # Returns
//
// An `HttpResponse` which contains the response from the Vite server,
// or an error response in case of failure.
pub async fn proxy_to_vite(req: HttpRequest, mut payload: web::Payload) -> anyhow::Result<HttpResponse, Error> {
	let client = Client::new();
	let forward_url = format!("http://localhost:3000{}", req.uri());

	// Buffer the entire payload
	let mut body_bytes = web::BytesMut::new();
	while let Some(chunk) = payload.next().await {
		let chunk = chunk?;
		if (body_bytes.len() + chunk.len()) > MAX_PAYLOAD_SIZE {
			return Err(actix_web::error::ErrorPayloadTooLarge("Payload overflow"));
		}
		body_bytes.extend_from_slice(&chunk);
	}

	let mut forwarded_resp = client
		.request_from(forward_url.as_str(), req.head())
		.no_decompress()
		.send_body(body_bytes)
		.await
		.map_err(|err| ErrorInternalServerError(format!("Failed to forward request: {}", err)))?;

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

pub fn start_vite_server() -> anyhow::Result<Child> {
	#[cfg(target_os = "windows")]
	let find_cmd = "where";
	#[cfg(not(target_os = "windows"))]
	let find_cmd = "which";

	let vite = std::process::Command::new(find_cmd)
		.arg("vite")
		.stdout(std::process::Stdio::piped())
		.output()?
		.stdout;

	let vite = String::from_utf8(vite)?;
	let vite = vite.as_str().trim();

	if vite.is_empty() {
		error!("vite not found, make sure its installed with npm install -g vite");
		return Err(std::io::Error::new(
			std::io::ErrorKind::NotFound,
			"vite not found",
		))?;
	}

	// Get the first occurrence
	let vite = vite
		.split("\n")
		.collect::<Vec<_>>()
		.last()
		.expect("Failed to get vite executable")
		.trim();

	debug!("found vite at: {:?}", vite);

	// Start the vite server
	Ok(std::process::Command::new(vite)
		.current_dir(r#"../../"#)
		.spawn()
		.expect("Failed to start vite server"))
}

pub trait AppConfig {
	fn configure_routes(self) -> Self;
}

impl<T> AppConfig for App<T>
where
	T: actix_web::dev::ServiceFactory<
		actix_web::dev::ServiceRequest,
		Config = (),
		Error = actix_web::Error,
		InitError = (),
	>,
{
	fn configure_routes(self) -> Self {
		if DEBUG {
			self.default_service(web::route().to(proxy_to_vite))
			    .service(web::resource("/assets/{file:.*}").route(web::get().to(proxy_to_vite)))
			    .service(web::resource("/node_modules/{file:.*}").route(web::get().to(proxy_to_vite)))
		} else {
			self.default_service(web::route().to(index))
			    .service(web::scope("/assets/{file:.*}").service(assets))
		}
	}
}