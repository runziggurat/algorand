//! Algod's REST API message definitions.
//!
//! There are two REST API versions for algod:
//! - [V1](https://developer.algorand.org/docs/rest-apis/algod/v1/) - which is deprecated but still used by the node.
//! - [V2](https://developer.algorand.org/docs/rest-apis/algod/v2/)

use serde::{Deserialize, Serialize};

use crate::protocol::codecs::msgpack::{Ed25519Seed, HashDigest};

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
    #[serde(rename = "seed", default)]
    pub sortition_seed: Option<Ed25519Seed>,

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
