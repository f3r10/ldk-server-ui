//! Serves the web UI and proxies it to ldk-server.

mod config;
mod node;
mod routes;

use std::error::Error;
use std::sync::Arc;

use ldk_server_client::client::LdkServerClient;

use crate::config::Config;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let config = Config::from_env()?;

	let cert = std::fs::read(&config.tls_cert)
		.map_err(|e| format!("reading {}: {e}", config.tls_cert.display()))?;
	let macaroon = std::fs::read_to_string(&config.macaroon)
		.map_err(|e| format!("reading {}: {e}", config.macaroon.display()))?;
	let client =
		LdkServerClient::new(config.ldk_server_url.clone(), macaroon.trim().to_string(), &cert)?;

	let listener = tokio::net::TcpListener::bind(config.listen).await?;
	println!(
		"sidecar listening on http://{} -> ldk-server {}",
		config.listen, config.ldk_server_url
	);
	axum::serve(listener, routes::router(Arc::new(client))).await?;
	Ok(())
}
