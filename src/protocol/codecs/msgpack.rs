//! Message pack deserializer for algod messages.

use std::{
    convert::From,
    fmt::{self, Debug, Display, Formatter},
    str,
};

use data_encoding::{BASE32_NOPAD, BASE64};
use serde::{de::Visitor, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer};
use sha2::Digest;

/// Period of time.
type Period = u64;

/// Algorand is organized in logical units (r = 0, 1...) called rounds in which new blocks are created.
pub type Round = u64;

/// Each [Round] is divided into multiple steps.
type Step = u64;

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
    /// Sig is a signature of msg under the key PK.
    #[serde(rename = "s")]
    sig: Ed25519Signature,
    /// Public key.
    #[serde(rename = "p")]
    pk: Ed25519PublicKey,

    /// Old-style signature that does not use proper domain separation.
    /// PKSigOld is unused; however, unfortunately we forgot to mark it
    /// `codec:omitempty` and so it appears (with zero value) in certs.
    /// This means we can't delete the field without breaking catchup.
    #[serde(rename = "ps")]
    pksigold: Ed25519Signature,

    /// Used to verify a new-style two-level ephemeral signature.
    #[serde(rename = "p2")]
    pk2: Ed25519PublicKey,
    /// PK1Sig is a signature of OneTimeSignatureSubkeyOffsetID(PK, Batch, Offset) under the key PK2.
    #[serde(rename = "p1s")]
    pk1sig: Ed25519Signature,
    /// PK2Sig is a signature of OneTimeSignatureSubkeyBatchID(PK2, Batch) under the master key (OneTimeSignatureVerifier).
    #[serde(rename = "p2s")]
    pk2sig: Ed25519Signature,
}

/// An UnauthenticatedCredential is a Credential which has not yet been authenticated.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnauthenticatedCredential {
    /// A VrfProof for a message can be generated with a secret key and verified against a public key, like a signature.
    /// Proofs are malleable, however, for a given message and public key,
    /// the VRF output that can be computed from a proof is unique.
    #[serde(rename = "pf", default)]
    vrf_proof: Option<VrfProof>,
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
    #[serde(rename = "seed")]
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

    /// Seed proof.
    #[serde(default, rename = "sdpf")]
    pub seed_proof: Option<VrfProof>,

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
/// Wraps a transaction in a signature.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignedTransaction {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sig: Option<Ed25519Signature>,

    #[serde(rename = "msig", default, skip_serializing_if = "Option::is_none")]
    pub multisig: Option<MultisigSignature>,

    #[serde(rename = "txn")]
    pub transaction: Transaction,
}

/// A transaction that can appear in a block.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Transaction {
    /// Paid by the sender to the FeeSink to prevent denial-of-service. The minimum fee on Algorand
    /// is currently 1000 microAlgos.
    #[serde(rename = "fee")]
    pub fee: u64,

    /// The first round for when the transaction is valid. If the transaction is sent prior to this
    /// round it will be rejected by the network.
    #[serde(rename = "fv")]
    pub first_valid: Round,

    /// The hash of the genesis block of the network for which the transaction is valid.
    #[serde(rename = "gh")]
    pub genesis_hash: HashDigest,

    /// The ending round for which the transaction is valid. After this round, the transaction will
    /// be rejected by the network.
    #[serde(rename = "lv")]
    pub last_valid: Round,

    /// The address of the account that pays the fee and amount.
    #[serde(rename = "snd")]
    pub sender: Address,

    /// The human-readable string that identifies the network for the transaction. The genesis ID is
    /// found in the genesis block.
    #[serde(default, rename = "gen")]
    pub genesis_id: String,

    /// The group specifies that the transaction is part of a group and, if so, specifies the hash of
    /// the transaction group. Assign a group ID to a transaction through the workflow described in
    /// the Atomic Transfers Guide.
    #[serde(rename = "grp", default)]
    pub group: Option<HashDigest>,

    /// A lease enforces mutual exclusion of transactions. If this field is nonzero, then once the
    /// transaction is confirmed, it acquires the lease identified by the (Sender, Lease) pair of
    /// the transaction until the LastValid round passes. While this transaction possesses the
    /// lease, no other transaction specifying this lease can be confirmed. A lease is often used
    /// in the context of Algorand Smart Contracts to prevent replay attacks. Read more about
    /// Algorand Smart Contracts and see the Delegate Key Registration TEAL template for an example
    /// implementation of leases. Leases can also be used to safeguard against unintended duplicate
    /// spends. For example, if we send a transaction to the network and later realize my fee was too
    /// low, we could send another transaction with a higher fee, but the same lease value. This would
    /// ensure that only one of those transactions ends up getting confirmed during the validity period.
    #[serde(rename = "lx", default)]
    pub lease: Option<HashDigest>,

    /// Any data up to 1024 bytes.
    #[serde(with = "serde_bytes", default)]
    pub note: Vec<u8>,

    /// Specifies the authorized address. This address will be used to authorize all future transactions.
    /// Learn more about Rekeying accounts.
    #[serde(rename = "rekey", default)]
    pub rekey_to: Option<Address>,

    /// Specifies the type of transaction.
    #[serde(flatten)]
    pub txn_type: TransactionType,
}

/// Enum containing the types of transactions and their specific fields.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type")]
pub enum TransactionType {
    /// Payment transaction.
    #[serde(rename = "pay")]
    Payment(Payment),
    // Maybe include more types here later.
}

/// Fields for a payment transaction.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Payment {
    /// The address of the account that receives the amount.
    #[serde(rename = "rcv")]
    pub receiver: Address,

    /// The total amount to be sent in microAlgos.
    #[serde(rename = "amt")]
    pub amount: u64,

    /// When set, it indicates that the transaction is requesting that the Sender account should
    /// be closed, and all remaining funds, after the fee and amount are paid, be transferred to
    /// this address.
    #[serde(rename = "close", default)]
    pub close_remainder_to: Option<Address>,
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

impl From<&Vec<u8>> for HashDigest {
    fn from(data: &Vec<u8>) -> Self {
        let hashed = sha2::Sha512_256::digest(data);
        let mut hash = [0; 32];
        hash.copy_from_slice(&hashed);
        HashDigest(hash)
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

/// An Ed25519 Signature.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Ed25519Signature(pub [u8; 64]);

impl Serialize for Ed25519Signature {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for Ed25519Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Ed25519Signature(
            deserializer.deserialize_bytes(SignatureVisitor)?,
        ))
    }
}

/// A MultisigSignature.
#[derive(Default, Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct MultisigSignature {
    #[serde(rename = "subsig")]
    pub subsigs: Vec<MultisigSubsig>,

    #[serde(rename = "thr")]
    pub threshold: u8,

    #[serde(rename = "v")]
    pub version: u8,
}

impl Serialize for MultisigSignature {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_map(Some(3))?;

        state.serialize_entry("subsig", &self.subsigs)?;
        state.serialize_entry("thr", &self.threshold)?;
        state.serialize_entry("v", &self.version)?;
        state.end()
    }
}

/// A MultisigSubsig.
#[derive(Debug, Eq, PartialEq, Clone, Deserialize)]
pub struct MultisigSubsig {
    #[serde(rename = "pk")]
    pub key: Ed25519PublicKey,

    #[serde(rename = "s")]
    pub sig: Option<Ed25519Signature>,
}

impl Serialize for MultisigSubsig {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        let len = if self.sig.is_some() { 2 } else { 1 };
        let mut state = serializer.serialize_map(Some(len))?;

        state.serialize_entry("pk", &self.key)?;
        if let Some(sig) = &self.sig {
            state.serialize_entry("s", sig)?;
        }
        state.end()
    }
}

/// An Ed25519PublicKey.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Ed25519PublicKey(pub [u8; 32]);

impl Serialize for Ed25519PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for Ed25519PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Ed25519PublicKey(
            deserializer.deserialize_bytes(VisitorU8_32)?,
        ))
    }
}

/// An [Ed25519Seed] holds the entropy needed to generate cryptographic keys.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct Ed25519Seed(pub [u8; 32]);

impl Serialize for Ed25519Seed {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for Ed25519Seed {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Ed25519Seed(deserializer.deserialize_bytes(VisitorU8_32)?))
    }
}

/// Verifiable Random Function proof.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct VrfProof(pub [u8; 80]);

impl Serialize for VrfProof {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0[..])
    }
}

impl<'de> Deserialize<'de> for VrfProof {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(VrfProof(deserializer.deserialize_bytes(VisitorU8_80)?))
    }
}

/// Signature Visitor (`[u8; 64]` arrays).
pub struct SignatureVisitor;

impl<'de> Visitor<'de> for SignatureVisitor {
    type Value = [u8; 64];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("expecting a 64 byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        TryInto::<Self::Value>::try_into(v)
            .map_err(|_| E::custom(format!("invalid byte array length: {}", v.len())))
    }
}

/// Visitor for `[u8; 80]` array.
pub struct VisitorU8_80;

impl<'de> Visitor<'de> for VisitorU8_80 {
    type Value = [u8; 80];

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("expecting a 80 byte array")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        TryInto::<Self::Value>::try_into(v)
            .map_err(|_| E::custom(format!("invalid byte array length: {}", v.len())))
    }
}

/// Visitor for `[u8; 32]` array.
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
        TryInto::<Self::Value>::try_into(v)
            .map_err(|_| E::custom(format!("invalid byte array length: {}", v.len())))
    }
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
