use std::{
    fmt::{self, Display, Formatter},
    time::Duration,
};

use data_encoding::{BASE32_NOPAD, BASE64};
use fmt::Debug;
use reqwest::{header, Client};
use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use tokio::time::{error::Elapsed, sleep};

use crate::protocol::constants::USER_AGENT;

/// Timeout time for RPC requests.
const RPC_TIMEOUT: Duration = Duration::from_secs(10);

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

    tokio::time::timeout(RPC_TIMEOUT, async move {
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
            sleep(Duration::from_secs(1)).await;
        }
    })
    .await?
}

/// [EncodedBlockCert] defines how get-block response encodes a block and its certificate.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EncodedBlockCert {
    /// Block header data.
    pub block: BlockHeaderMsgPack,
    /// Certificate.
    pub cert: Certificate,
}

/// A Certificate contains a cryptographic proof that agreement was reached on a
/// given block in a given round.
///
/// When a client first joins the network or has fallen behind and needs to catch
/// up, certificates allow the client to verify that a block someone gives them
/// is the real one.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Certificate {
    /// Proposal value.
    #[serde(default, rename = "prop")]
    pub proposal: Option<CertificateProposal>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CertificateProposal {
    /// Block header's hash.
    #[serde(rename = "dig")]
    pub block_digest: HashDigest,
}

/// BlockHeader
/// Deserialized from MessagePack format.
///
/// See [block.go](https://github.com/algorand/go-algorand/blob/master/data/bookkeeping/block.go) for more details.
// Comments below are simply copied from the go-algorand repo.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BlockHeaderMsgPack {
    /// RewardsLevel specifies how many rewards, in MicroAlgos, have been distributed
    /// to each config.Protocol.RewardUnit of MicroAlgos since genesis.
    #[serde(default)]
    pub earn: u64,

    /// The FeeSink accepts transaction fees. It can only spend to the incentive pool.
    #[serde(default, rename = "fees")]
    pub fee_sink: Option<HashDigest>,

    /// The number of leftover MicroAlgos after the distribution of RewardsRate/rewardUnits
    /// MicroAlgos for every reward unit in the next round.
    #[serde(default, rename = "frac")]
    pub leftover_fraction: u64,

    /// Genesis ID to which this block belongs.
    #[serde(default, rename = "gen")]
    pub genensis_id: String,

    /// Genesis hash to which this block belongs.
    #[serde(default, rename = "gh")]
    pub genesis_id_hash: Option<HashDigest>,

    /// The hash of the previous block.
    #[serde(default, rename = "prev")]
    pub prevous_block_hash: Option<HashDigest>,

    /// Current protocol.
    #[serde(default, rename = "proto")]
    pub protocol_current: String,

    /// The number of new MicroAlgos added to the participation stake from rewards at the next round.
    #[serde(default, rename = "rate")]
    pub rewards_rate: u64,

    /// Round represents a protocol round index.
    #[serde(default, rename = "rnd")]
    pub round: u64,

    /// The round at which the RewardsRate will be recalculated.
    #[serde(default, rename = "rwcalr")]
    pub rewards_rate_recalc_round: u64,

    /// The RewardsPool accepts periodic injections from the FeeSink and continually
    /// redistributes them to addresses as rewards.
    #[serde(default, rename = "rwd")]
    pub rewards_pool: Option<HashDigest>,

    /// Sortition seed.
    #[serde(
        default,
        rename = "seed",
        deserialize_with = "deserialize_byte32_arr_opt"
    )]
    pub sortition_seed: Option<[u8; 32]>,

    /// TimeStamp in seconds since epoch.
    #[serde(default, rename = "ts")]
    pub timestamp: i64,

    /// Root of transaction merkle tree using SHA512_256 hash function.
    /// This commitment is computed based on the PaysetCommit type specified in the block's consensus protocol.
    #[serde(default, rename = "txn")]
    pub tx_merke_root_hash: Option<HashDigest>,

    /// Root of transaction vector commitment merkle tree using SHA256 hash function.
    #[serde(default, rename = "txn256")]
    pub tx_merke_root_hash256: Option<HashDigest>,
}

/// A SHA512_256 hash.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct HashDigest(pub [u8; 32]);

impl Display for HashDigest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", BASE64.encode(&self.0))
    }
}

impl Debug for HashDigest {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", BASE32_NOPAD.encode(&self.0))
    }
}

impl Serialize for HashDigest {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for HashDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(HashDigest(deserializer.deserialize_bytes(VisitorU8_32)?))
    }
}

pub struct VisitorU8_32;

impl<'de> Visitor<'de> for VisitorU8_32 {
    type Value = [u8; 32];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("expecting a 32 byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() != 32 {
            return Err(E::custom(format!("Invalid byte array length: {}", v.len())));
        }

        let mut bytes = [0; 32];
        bytes.copy_from_slice(v);
        Ok(bytes)
    }
}

pub fn deserialize_byte32_arr_opt<'de, D>(deserializer: D) -> Result<Option<[u8; 32]>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match <Option<&[u8]>>::deserialize(deserializer)? {
        Some(slice) => Some(slice.try_into().map_err(D::Error::custom)?),
        None => None,
    })
}
