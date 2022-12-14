//! The kmd's REST API message definitions.
//!
//! The kmd daemons provide their API specifications here:
//! https://developer.algorand.org/docs/rest-apis/kmd/

use serde::{Deserialize, Serialize};

/// APIV1Wallet is the API's representation of a wallet.
#[derive(Debug, Deserialize)]
pub struct ApiV1Wallet {
    pub driver_name: String,
    pub driver_version: u32,
    pub id: String,
    pub mnemonic_ux: bool,
    pub name: String,
    pub supported_txs: Vec<String>,
}

/// ListWalletsResponse is the response to `GET /v1/wallets`.
#[derive(Debug, Deserialize)]
pub struct ListWalletsResponse {
    #[serde(default)]
    pub wallets: Vec<ApiV1Wallet>,
}

/// InitWalletHandleRequest is the request for `POST /v1/wallet/init`.
#[derive(Serialize)]
pub(super) struct InitWalletHandleRequest {
    pub wallet_id: String,
    pub wallet_password: String,
}

/// InitWalletHandleResponse is the response to `POST /v1/wallet/init`.
#[derive(Debug, Deserialize)]
pub struct InitWalletHandleResponse {
    pub wallet_handle_token: String,
}
