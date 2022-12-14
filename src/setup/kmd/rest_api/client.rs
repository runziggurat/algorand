//! The kmd's REST API client.
//!
//! The kmd daemons provide their API specifications here:
//! https://developer.algorand.org/docs/rest-apis/kmd/

use crate::setup::kmd::rest_api::message::{
    InitWalletHandleRequest, InitWalletHandleResponse, ListWalletsResponse,
};

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

    /// Unlock the wallet and return a wallet handle token that can be used for subsequent operations.
    ///
    /// These tokens expire periodically and must be renewed. You can POST the token to
    /// /v1/wallet/info to see how much time remains until expiration, and renew it with
    /// /v1/wallet/renew. When you're done, you can invalidate the token with /v1/wallet/release.
    pub async fn get_wallet_handle_token(
        &self,
        wallet_id: String,
        wallet_password: String,
    ) -> anyhow::Result<InitWalletHandleResponse> {
        let req = InitWalletHandleRequest {
            wallet_id,
            wallet_password,
        };

        self.http_client
            .post(&format!("{}v1/wallet/init", self.address))
            .header(API_HEADER_TOKEN, &self.token)
            .header(reqwest::header::ACCEPT, API_HEADER_ACCEPT_JSON)
            .json(&req)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
            .map_err(|e| {
                anyhow::anyhow!(
                    "couldn't initialize the wallet (id: {}: {e})",
                    req.wallet_id
                )
            })
    }
}
