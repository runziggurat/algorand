//! The kmd's REST API message definitions.
//!
//! The kmd daemons provide their API specifications here:
//! https://developer.algorand.org/docs/rest-apis/kmd/

use data_encoding::BASE64;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

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

/// ListKeysRequest is the request for `POST /v1/key/list`.
#[derive(Serialize)]
pub struct ListKeysRequest {
    pub wallet_handle_token: String,
}

/// ListKeysResponse is the response to `POST /v1/key/list`.
#[derive(Debug, Deserialize)]
pub struct ListKeysResponse {
    #[serde(default)]
    pub addresses: Vec<String>,
}

/// SignTransactionRequest is the request for `POST /v1/transaction/sign`.
#[derive(Serialize)]
pub struct SignTransactionRequest {
    pub wallet_handle_token: String,
    #[serde(serialize_with = "serialize_bytes")]
    pub transaction: Vec<u8>,
    pub wallet_password: String,
}

/// SignTransactionResponse is the response to `POST /v1/transaction/sign`.
#[derive(Debug, Deserialize)]
pub struct SignTransactionResponse {
    #[serde(deserialize_with = "deserialize_bytes")]
    pub signed_transaction: Vec<u8>,
}

fn deserialize_bytes<'de, D>(deserializer: D) -> Result<Vec<u8>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = <&str>::deserialize(deserializer)?;
    Ok(BASE64.decode(s.as_bytes()).unwrap())
}

fn serialize_bytes<S>(bytes: &[u8], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&BASE64.encode(bytes))
}
