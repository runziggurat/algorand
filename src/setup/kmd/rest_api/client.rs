//! The kmd's REST API client.
//!
//! The kmd daemons provide their API specifications here:
//! https://developer.algorand.org/docs/rest-apis/kmd/

use crate::setup::kmd::rest_api::message::ListWalletsResponse;

const API_HEADER_TOKEN: &str = "X-KMD-API-Token";
const API_HEADER_ACCEPT_JSON: &str = "application/json";

/// Client for interacting with the key management daemon via V1 REST API.
pub struct ClientV1 {
    pub address: String,
    pub token: String,
    pub http_client: reqwest::Client,
}

impl ClientV1 {
    /// Creates a new [ClientV1].
    ///
    /// The function creates an HTTP URL with the address, so the address should use only `<ip>:<port>` format.
    pub fn new(address: &str, token: String) -> Self {
        Self {
            address: format!("http://{address}/"),
            token,
            http_client: reqwest::Client::new(),
        }
    }

    /// Get the list of wallets.
    pub async fn get_wallets(&self) -> anyhow::Result<ListWalletsResponse> {
        self.http_client
            .get(&format!("{}v1/wallets", self.address))
            .header(API_HEADER_TOKEN, &self.token)
            .header(reqwest::header::ACCEPT, API_HEADER_ACCEPT_JSON)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("couldn't get the wallets: {e}"))
    }
}
