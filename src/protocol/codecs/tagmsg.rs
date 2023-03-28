use std::io;

use bytes::{Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};
use tracing::*;

use crate::protocol::{
    codecs::payload::{Payload, PayloadCodec},
    invalid_data,
};

/// [Tag] represents a message type identifier.
///
/// The original tag list can be found in go-algorand/protocol/tags.go.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tag {
    UnknownMsg,
    AgreementVote,
    MsgOfInterest,
    MsgDigestSkip,
    NetPrioResponse,
    Ping,
    PingReply,
    ProposalPayload,
    StateProofSig,
    TopicMsgResp,
    Txn,
    UniEnsBlockReq,
    VoteBundle,

    /// Below tag is not part of the official go-algorand SPEC.
    RawBytes,
}

impl Tag {
    pub fn get_tag_str(&self) -> &str {
        match self {
            Self::UnknownMsg => "??",
            Self::AgreementVote => "AV",
            Self::MsgOfInterest => "MI",
            Self::MsgDigestSkip => "MS",
            Self::NetPrioResponse => "NP",
            Self::Ping => "pi",
            Self::PingReply => "pj",
            Self::ProposalPayload => "PP",
            Self::StateProofSig => "SP",
            Self::TopicMsgResp => "TS",
            Self::Txn => "TX",
            Self::UniEnsBlockReq => "UE",
            Self::VoteBundle => "VB",
            Self::RawBytes => "",
        }
    }
}

impl TryFrom<Bytes> for Tag {
    type Error = io::Error;

    fn try_from(tag: bytes::Bytes) -> Result<Self, Self::Error> {
        let tag = std::str::from_utf8(&tag)
            .map_err(|_| invalid_data!("couldn't convert the tag to a UTF-8 string"))?;

        Self::try_from(tag)
    }
}

impl TryFrom<&str> for Tag {
    type Error = io::Error;

    fn try_from(tag: &str) -> Result<Self, Self::Error> {
        Ok(match tag {
            "??" => Self::UnknownMsg,
            "AV" => Self::AgreementVote,
            "MI" => Self::MsgOfInterest,
            "MS" => Self::MsgDigestSkip,
            "NP" => Self::NetPrioResponse,
            "pi" => Self::Ping,
            "pj" => Self::PingReply,
            "PP" => Self::ProposalPayload,
            "SP" => Self::StateProofSig,
            "TS" => Self::TopicMsgResp,
            "TX" => Self::Txn,
            "UE" => Self::UniEnsBlockReq,
            "VB" => Self::VoteBundle,
            _ => return Err(invalid_data!("unexpected tag")),
        })
    }
}

impl From<&Payload> for Tag {
    fn from(payload: &Payload) -> Self {
        match *payload {
            Payload::MsgOfInterest(_) => Self::MsgOfInterest,
            Payload::ProposalPayload(_) => Self::ProposalPayload,
            Payload::AgreementVote(_) => Self::AgreementVote,
            Payload::Ping(_) => Self::Ping,
            Payload::PingReply(_) => Self::PingReply,
            Payload::UniEnsBlockReq(_) => Self::UniEnsBlockReq,
            Payload::TopicMsgResp(_) => Self::TopicMsgResp,
            Payload::NetPrioResponse(_) => Self::NetPrioResponse,
            Payload::MsgDigestSkip(_) => Self::MsgDigestSkip,
            Payload::Transaction(_) => Self::Txn,
            Payload::RawBytes(_) => Self::RawBytes,
            Payload::NotImplemented => Self::UnknownMsg,
        }
    }
}

/// [TagMsgCodec] is the codec for tagged Algod messages.
#[derive(Clone)]
pub struct TagMsgCodec {
    /// The associated node's span.
    span: Span,

    /// [Payload] codec for Algod messages.
    payload: PayloadCodec,
}

impl TagMsgCodec {
    pub fn new(span: Span) -> Self {
        Self {
            payload: PayloadCodec::new(span.clone()),
            span,
        }
    }
}

impl Decoder for TagMsgCodec {
    type Item = Payload;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        const TAG_LEN: usize = 2;

        let tag = Tag::try_from(src.split_to(TAG_LEN).freeze())?;
        debug!(parent: &self.span, "decoded a tag: {:?}", tag);

        self.payload.tag = Some(tag);
        self.payload.decode(src)
    }
}

impl Encoder<Payload> for TagMsgCodec {
    type Error = io::Error;

    fn encode(&mut self, message: Payload, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let tag = Tag::from(&message);
        dst.extend_from_slice(tag.get_tag_str().as_bytes());

        let mut payload_data = BytesMut::new();
        self.payload
            .encode(message, &mut payload_data)
            .map_err(|_| invalid_data!("couldn't encode a tagmsg message"))?;

        dst.extend_from_slice(&payload_data);
        Ok(())
    }
}
