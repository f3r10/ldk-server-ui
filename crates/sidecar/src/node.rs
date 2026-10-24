//! The ldk-server calls the sidecar makes, behind a trait so handlers can be tested with a fake.

use std::future::Future;

use ldk_server_client::client::LdkServerClient;
use ldk_server_client::error::LdkServerError;
use ldk_server_client::ldk_server_grpc::api::{GetNodeInfoRequest, GetNodeInfoResponse};

/// The subset of ldk-server the sidecar uses. Grows one method per proxied RPC.
pub trait NodeApi: Send + Sync + 'static {
	#[expect(dead_code, reason = "not called until routes::node_info is implemented")]
	fn node_info(&self)
		-> impl Future<Output = Result<GetNodeInfoResponse, LdkServerError>> + Send;
}

impl NodeApi for LdkServerClient {
	async fn node_info(&self) -> Result<GetNodeInfoResponse, LdkServerError> {
		self.get_node_info(GetNodeInfoRequest {}).await
	}
}
