//! Message pack deserializer for algod messages.
//!
//! Note:
//!   Not all fields are yet deserialized in the messages below, but all fields are at least listed.
//!   The naming of the fields and messages correspond to those in the original go-algorand repo.
//!
//! TODO(Rqnsom): deserialize 64-byte arrays (fully deserialize all the fields).
//!

use std::{
    fmt::{self, Debug, Display, Formatter},
    str,
};

use data_encoding::{BASE32_NOPAD, BASE64};
use serde::{
    de::{Error, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use sha2::Digest;

// Period of time.
type Period = u64;

// Algorand is organized in logical units (r = 0, 1...) called rounds in which new blocks are created.
pub type Round = u64;

// Each [Round] is divided into multiple steps.
type Step = u64;

// A Seed holds the entropy needed to generate cryptographic keys.
type Seed = Ed25519Seed;

// Verifiable Random Function proof.
#[allow(unused)]
type VrfProof = [u8; 80];

/* Classical signatures */
#[allow(unused)]
type Ed25519Signature = [u8; 64];
type Ed25519PublicKey = [u8; 32];
#[allow(unused)]
type Ed25519PrivateKey = [u8; 64];
type Ed25519Seed = [u8; 32];

/// A [NetPrioResponse] contains an answer to the challenge provided within handshake accept
/// message from the server.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NetPrioResponse {
    /// Response.
    #[serde(rename = "Response")]
    pub response: Response,

    /// Round represents a protocol round index.
    #[serde(rename = "Round")]
    round: Round,

    /// Sender address.
    #[serde(rename = "Sender")]
    sender_addr: Address,

    /// Signature.
    #[serde(rename = "Sig")]
    sig: OneTimeSignature,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Response {
    #[serde(rename = "Nonce")]
    pub nonce: String,
}

/// A [ProposalValue] is a triplet of a block hashes (the contents themselves and the encoding of the block),
/// its proposer, and the period in which it was proposed.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProposalValue {
    #[serde(default, rename = "oper")]
    original_period: Period,

    #[serde(rename = "oprop")]
    original_proposer: Address,

    #[serde(rename = "dig")]
    block_digest: HashDigest,

    #[serde(rename = "encdig")]
    encoding_digest: HashDigest,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RawVote {
    /// Sender address.
    #[serde(rename = "snd")]
    sender_addr: Address,

    /// Round represents a protocol round index.
    #[serde(rename = "rnd")]
    round: Round,

    /// Time period.
    #[serde(default, rename = "per")]
    period: Period,

    /// Step of the round.
    #[serde(default, rename = "step")]
    step: Step,

    /// Proposal vote.
    #[serde(default, rename = "prop")]
    proposal: Option<ProposalValue>,
}

/// A OneTimeSignature is a cryptographic signature that is produced a limited
/// number of times and provides forward integrity.
///
/// Specifically, a OneTimeSignature is generated from an ephemeral secret. After
/// some number of messages is signed under a given OneTimeSignatureIdentifier
/// identifier, the corresponding secret is deleted. This prevents the
/// secret-holder from signing a contradictory message in the future in the event
/// of a secret-key compromise.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OneTimeSignature {
    // Sig is a signature of msg under the key PK.
    //#[serde(rename = "s", deserialize_with = "deserialize_byte64_arr_opt")]
    //sig: Ed25519Signature,
    /// Public key.
    #[serde(rename = "p", deserialize_with = "deserialize_byte32_arr_opt")]
    pk: Option<Ed25519PublicKey>,

    // Old-style signature that does not use proper domain separation.
    // PKSigOld is unused; however, unfortunately we forgot to mark it
    // `codec:omitempty` and so it appears (with zero value) in certs.
    // This means we can't delete the field without breaking catchup.
    //#[serde(rename = "ps", deserialize_with = "deserialize_byte64_arr_opt")]
    //pksigold: Ed25519Signature,

    // Used to verify a new-style two-level ephemeral signature.
    // PK1Sig is a signature of OneTimeSignatureSubkeyOffsetID(PK, Batch, Offset) under the key PK2.
    // PK2Sig is a signature of OneTimeSignatureSubkeyBatchID(PK2, Batch) under the master key (OneTimeSignatureVerifier).
    #[serde(rename = "p2", deserialize_with = "deserialize_byte32_arr_opt")]
    pk2: Option<Ed25519PublicKey>,
    //#[serde(rename = "p1s", deserialize_with = "deserialize_byte64_arr_opt")]
    //pk1sig: Option<Ed25519Signature>,
    //#[serde(rename = "p2s", deserialize_with = "deserialize_byte64_arr_opt")]
    //pk2sig: Ed25519Signature,
}

// An UnauthenticatedCredential is a Credential which has not yet been authenticated.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnauthenticatedCredential {
    // A VrfProof for a message can be generated with a secret key and verified against a public key, like a signature.
    // Proofs are malleable, however, for a given message and public key, the VRF output that can be computed from a proof is unique.
    //#[serde(default, rename = "pf")]
    //vrf_proof: Option<VrfProof>,
}

/// [UnauthenticatedVote] is a vote which has not been verified.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnauthenticatedVote {
    /// Raw vote.
    #[serde(default, rename = "r")]
    pub raw_vote: Option<RawVote>,

    /// Unauthenticated credential.
    #[serde(default, rename = "cred")]
    pub unauthenticated_credential: Option<UnauthenticatedCredential>,

    /// Signature.
    #[serde(default, rename = "sig")]
    pub sig: Option<OneTimeSignature>,
}

/// A [ProposalPayload] is a struct reflecting [transmittedPayload] struct from the
/// go-algorand/agreement/proposal.go file.
///
/// A [transmittedPayload] is the representation of a proposal payload on the wire.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProposalPayload {
    /// RewardsLevel specifies how many rewards, in MicroAlgos, have been distributed
    /// to each config.Protocol.RewardUnit of MicroAlgos since genesis.
    #[serde(default)]
    pub earn: u64,

    /// The FeeSink accepts transaction fees. It can only spend to the incentive pool.
    #[serde(rename = "fees")]
    pub fee_sink: Address,

    /// The number of leftover MicroAlgos after the distribution of RewardsRate/rewardUnits
    /// MicroAlgos for every reward unit in the next round.
    #[serde(default, rename = "frac")]
    pub leftover_fraction: u64,

    /// Genesis ID to which this block belongs.
    #[serde(rename = "gen")]
    pub genensis_id: String,

    /// Genesis hash to which this block belongs.
    #[serde(rename = "gh")]
    pub genesis_id_hash: HashDigest,

    /// The hash of the previous block.
    #[serde(default, rename = "prev")]
    pub prevous_block_hash: Option<HashDigest>,

    /// Current protocol.
    #[serde(rename = "proto")]
    pub protocol_current: String,

    /// The number of new MicroAlgos added to the participation stake from rewards at the next round.
    #[serde(rename = "rate")]
    pub rewards_rate: u64,

    /// Round represents a protocol round index.
    #[serde(default, rename = "rnd")]
    pub round: u64,

    /// The round at which the RewardsRate will be recalculated.
    #[serde(rename = "rwcalr")]
    pub rewards_rate_recalc_round: u64,

    /// The RewardsPool accepts periodic injections from the FeeSink and continually
    /// redistributes them to addresses as rewards.
    #[serde(rename = "rwd")]
    pub rewards_pool: Address,

    /// Sortition seed.
    #[serde(rename = "seed", deserialize_with = "deserialize_byte32_arr_opt")]
    pub sortition_seed: Option<Seed>,

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

    ///// Seed proof.
    //#[serde(default, rename = "sdpf")]
    //seed_proof: Option<VrfProof>,
    //
    /// Original period.
    #[serde(default, rename = "oper")]
    pub original_period: u64,

    /// Original proposal.
    #[serde(rename = "oprop")]
    pub original_proposal: Address,

    /// Prior vote.
    #[serde(default, rename = "pv")]
    pub prior_vote: Option<UnauthenticatedVote>,
}

/// A vote is an endorsement of a particular proposal in Algorand.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AgreementVote {
    /// Raw vote.
    #[serde(rename = "r")]
    pub raw_vote: RawVote,

    /// Unauthenticated credential.
    #[serde(rename = "cred")]
    pub unauthenticated_credential: UnauthenticatedCredential,

    /// Signature.
    #[serde(rename = "sig")]
    pub sig: OneTimeSignature,
}

const CHECKSUM_LEN: usize = 4;
const HASH_LEN: usize = 32;

/// Public key address.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Address([u8; HASH_LEN]);

impl Address {
    /// Create a new [Address].
    pub fn new(bytes: [u8; HASH_LEN]) -> Address {
        Address(bytes)
    }

    /// Decode an address from a base64 string with a checksum.
    pub fn from_string(string: &str) -> Result<Address, String> {
        let checksum_address = match BASE32_NOPAD.decode(string.as_bytes()) {
            Ok(decoded) => decoded,
            Err(err) => return Err(format!("error decoding base32: {:?}", err)),
        };

        if checksum_address.len() != (HASH_LEN + CHECKSUM_LEN) {
            return Err(format!("wrong address length: {}", checksum_address.len()));
        }

        let (address, checksum) = checksum_address.split_at(HASH_LEN);
        let hashed = sha2::Sha512_256::digest(address);
        if &hashed[(HASH_LEN - CHECKSUM_LEN)..] != checksum {
            return Err("input checksum did not validate".to_string());
        }

        let mut bytes = [0; HASH_LEN];
        bytes.copy_from_slice(address);
        Ok(Address::new(bytes))
    }

    /// Encode an address to a base64 string with a checksum.
    pub fn encode_string(&self) -> String {
        let hashed = sha2::Sha512_256::digest(self.0);
        let checksum = &hashed[(HASH_LEN - CHECKSUM_LEN)..];
        let checksum_address = [&self.0, checksum].concat();

        BASE32_NOPAD.encode(&checksum_address)
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.encode_string())
    }
}

impl Debug for Address {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Serialize for Address {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for Address {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Address(deserializer.deserialize_bytes(VisitorU8_32)?))
    }
}

/// A SHA512_256 hash.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct HashDigest(pub [u8; HASH_LEN]);

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
    type Value = [u8; HASH_LEN];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("expecting a 32 byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if v.len() != HASH_LEN {
            return Err(E::custom(format!("invalid byte array length: {}", v.len())));
        }

        let mut bytes = [0; HASH_LEN];
        bytes.copy_from_slice(v);
        Ok(bytes)
    }
}

pub fn deserialize_byte32_arr_opt<'de, D>(
    deserializer: D,
) -> Result<Option<[u8; HASH_LEN]>, D::Error>
where
    D: Deserializer<'de>,
{
    Ok(match <Option<&[u8]>>::deserialize(deserializer)? {
        Some(slice) => Some(slice.try_into().map_err(D::Error::custom)?),
        None => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn address_decode() {
        let s = "737777777777777777777777777777777777777777777777777UFEJ2CI";

        let addr = Address::from_string(s).expect("failed to decode an address from a string");
        assert_eq!(s, addr.encode_string());
    }

    #[test]
    fn address_decode_invalid_checksum() {
        let invalid_csum = "737777777777777777777777777777777777777777777777777UFEJ2CJ";

        assert!(Address::from_string(invalid_csum).is_err());
    }
}
