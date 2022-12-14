//! A REST API implementation is named RPC in the go-algorand code base.
//!
//! There are two REST API versions for algod:
//! - [V1](https://developer.algorand.org/docs/rest-apis/algod/v1/) - which is deprecated but still used by the node.
//! - [V2](https://developer.algorand.org/docs/rest-apis/algod/v2/)

use std::time::Duration;

use reqwest::{header, Client};
use tokio::time::{error::Elapsed, sleep};

use crate::{protocol::constants::USER_AGENT, setup::node::rest_api::message::EncodedBlockCert};

/// Timeout time for REST requests.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

#[derive(Default)]
struct HttpClient {
    client: Client,
}

impl HttpClient {
    async fn get_block(
        &self,
        rpc_addr: &str,
        round: &str,
    ) -> anyhow::Result<reqwest::Response, reqwest::Error> {
        // Replica of the HTTP request our synth node receives from the node.
        self.client
            .get(format!("http://{}/v1/private-v1/block/{}", rpc_addr, round))
            .header(header::HOST, rpc_addr)
            .header(header::USER_AGENT, USER_AGENT)
            .header(header::ACCEPT_ENCODING, "gzip")
            .send()
            .await
    }
}

/// Returns a block for a provided round.
pub async fn wait_for_block(rpc_addr: &str, round: u64) -> Result<EncodedBlockCert, Elapsed> {
    // Algod V1 documentation states that the round format is 'integer (int64)',
    // but it's actually an int64 integer encoded in base36.
    let round = radix_fmt::radix_36(round).to_string();
    let client = HttpClient::default();

    tokio::time::timeout(REQUEST_TIMEOUT, async move {
        loop {
            if let Ok(rsp) = client.get_block(rpc_addr, &round).await {
                if rsp.error_for_status_ref().is_err() {
                    tracing::trace!("invalid status for the response {:?}", rsp);
                    continue;
                }
                tracing::info!("correct status for the response {:?}", rsp);

                let block = rmp_serde::from_slice(&rsp.bytes().await.unwrap()).unwrap();
                tracing::info!("block data {:?}", block);
                return Ok(block);
            }

            // On average, new blocks are generated every 4 seconds, so a long wait is fine here.
            sleep(Duration::from_secs(1)).await;
        }
    })
    .await?
}
