//! HTTP routes the browser calls.

use std::sync::Arc;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::Router;
use ldk_server_client::error::LdkServerError;
use ldk_server_client::error::LdkServerErrorCode;
use prost::Message;

use crate::node::NodeApi;

pub fn router<N: NodeApi>(node: Arc<N>) -> Router {
	Router::new()
		.route("/api/health", get(health))
		.route("/api/node-info", get(node_info::<N>))
		.with_state(node)
}

/// The sidecar process is up. Says nothing about ldk-server.
async fn health() -> impl IntoResponse {
	(StatusCode::OK, "ok")
}

/// The node's `GetNodeInfoResponse`, protobuf-encoded.
async fn node_info<N: NodeApi>(State(node): State<Arc<N>>) -> Response {
	match node.node_info().await {
		Ok(get_node_info_response) => {
			let enc = get_node_info_response.encode_to_vec();
			(StatusCode::OK, [(axum::http::header::CONTENT_TYPE, "application/x-protobuf")], enc)
				.into_response()
		},
		Err(error) => {
			let status_code = error_status(&error);
			(status_code, error.message).into_response()
		},
	}
}

/// The HTTP status the browser sees for an ldk-server error.
fn error_status(error: &LdkServerError) -> StatusCode {
	match error.error_code {
		LdkServerErrorCode::InvalidRequestError => StatusCode::BAD_REQUEST,
		LdkServerErrorCode::AuthError => StatusCode::BAD_GATEWAY,
		LdkServerErrorCode::AuthorizationError => StatusCode::FORBIDDEN,
		LdkServerErrorCode::LightningError => StatusCode::UNPROCESSABLE_ENTITY,
		LdkServerErrorCode::InternalServerError => StatusCode::BAD_GATEWAY,
		LdkServerErrorCode::InternalError => StatusCode::BAD_GATEWAY,
	}
}

#[cfg(test)]
mod tests {
	use ldk_server_client::error::LdkServerErrorCode;
	use ldk_server_client::ldk_server_grpc::api::GetNodeInfoResponse;
	use prost::Message;

	use super::*;

	/// Stands in for ldk-server: returns whatever result it was built with.
	struct FakeNode(Result<GetNodeInfoResponse, LdkServerError>);

	impl NodeApi for FakeNode {
		async fn node_info(&self) -> Result<GetNodeInfoResponse, LdkServerError> {
			self.0.clone()
		}
	}

	fn error(code: LdkServerErrorCode) -> LdkServerError {
		LdkServerError::new(code, "test error")
	}

	#[test]
	fn error_status_mapping() {
		let cases = [
			// The browser's request was malformed.
			(LdkServerErrorCode::InvalidRequestError, StatusCode::BAD_REQUEST),
			// ldk-server rejected the *sidecar's* macaroon: a sidecar misconfiguration, not the
			// browser's fault, so 502 rather than 401.
			(LdkServerErrorCode::AuthError, StatusCode::BAD_GATEWAY),
			// The sidecar's macaroon is valid but lacks this permission (e.g. a read-only token).
			(LdkServerErrorCode::AuthorizationError, StatusCode::FORBIDDEN),
			// A Lightning-level failure (no route, insufficient balance, ...).
			(LdkServerErrorCode::LightningError, StatusCode::UNPROCESSABLE_ENTITY),
			(LdkServerErrorCode::InternalServerError, StatusCode::BAD_GATEWAY),
			// Client-side failure, e.g. ldk-server unreachable.
			(LdkServerErrorCode::InternalError, StatusCode::BAD_GATEWAY),
		];
		for (code, expected) in cases {
			assert_eq!(error_status(&error(code.clone())), expected, "{code}");
		}
	}

	#[tokio::test]
	async fn node_info_error_returns_mapped_status_and_message() {
		let node = Arc::new(FakeNode(Err(error(LdkServerErrorCode::AuthError))));

		let response = node_info(State(node)).await;

		assert_eq!(response.status(), StatusCode::BAD_GATEWAY);
		let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
		assert_eq!(body, "test error");
	}

	#[tokio::test]
	async fn node_info_success() {
		let expected = GetNodeInfoResponse { node_id: "03ab".into(), ..Default::default() };
		let node = Arc::new(FakeNode(Ok(expected.clone())));

		let response = node_info(State(node)).await;

		assert_eq!(response.status(), StatusCode::OK);
		assert_eq!(response.headers()[axum::http::header::CONTENT_TYPE], "application/x-protobuf");
		let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();
		assert_eq!(GetNodeInfoResponse::decode(body).unwrap(), expected);
	}
}
