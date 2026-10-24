//! Sidecar configuration, read from environment variables.

use std::env;
use std::net::SocketAddr;
use std::path::PathBuf;

/// Where the sidecar listens. Loopback by default: the sidecar holds a macaroon, so it must not
/// be reachable from other machines unless the operator says so.
const DEFAULT_LISTEN: &str = "127.0.0.1:8080";
const DEFAULT_LDK_SERVER_URL: &str = "127.0.0.1:3536";

pub struct Config {
	/// `SIDECAR_LISTEN`, e.g. `127.0.0.1:8080`.
	pub listen: SocketAddr,
	/// `LDK_SERVER_URL`, host:port without a scheme, e.g. `127.0.0.1:3536`.
	pub ldk_server_url: String,
	/// `LDK_SERVER_TLS_CERT`, path to the server's `tls.crt` (PEM). Required.
	pub tls_cert: PathBuf,
	/// `LDK_SERVER_MACAROON`, path to a file holding a hex macaroon. Required.
	pub macaroon: PathBuf,
}

impl Config {
	pub fn from_env() -> Result<Self, String> {
		let listen = env::var("SIDECAR_LISTEN").unwrap_or_else(|_| DEFAULT_LISTEN.to_string());
		let listen = listen
			.parse()
			.map_err(|e| format!("SIDECAR_LISTEN is not host:port ({listen}): {e}"))?;
		let ldk_server_url =
			env::var("LDK_SERVER_URL").unwrap_or_else(|_| DEFAULT_LDK_SERVER_URL.to_string());
		Ok(Self {
			listen,
			ldk_server_url,
			tls_cert: required_path("LDK_SERVER_TLS_CERT")?,
			macaroon: required_path("LDK_SERVER_MACAROON")?,
		})
	}
}

fn required_path(name: &str) -> Result<PathBuf, String> {
	env::var(name).map(PathBuf::from).map_err(|_| format!("{name} is not set"))
}
