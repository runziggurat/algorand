use std::io;

use bytes::{Bytes, BytesMut};
use tokio_util::codec::Decoder;
use tracing::*;

use crate::protocol::{
    codecs::payload::{Payload, PayloadCodec},
    invalid_data,
};

/// [Tag] represents a message type identifier.
///
/// The original tag list can be found in go-algorand/protocol/tags.go.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    UniCatchupReq,
    UniEnsBlockReq,
    VoteBundle,
}

impl TryFrom<Bytes> for Tag {
    type Error = io::Error;

    fn try_from(tag: bytes::Bytes) -> Result<Self, Self::Error> {
        let tag = std::str::from_utf8(&tag)
            .map_err(|_| invalid_data!("couldn't convert the tag to a UTF-8 string"))?;

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
            "UC" => Self::UniCatchupReq,
            "UE" => Self::UniEnsBlockReq,
            "VB" => Self::VoteBundle,
            _ => return Err(invalid_data!("unexpected tag")),
        })
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
