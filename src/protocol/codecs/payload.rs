use std::io;

use bytes::{BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};
use tracing::Span;

use crate::protocol::{
    codecs::{
        msgpack::{AgreementVote, HashDigest, NetPrioResponse, ProposalPayload},
        tagmsg::Tag,
        topic::{MsgOfInterest, TopicCodec, TopicMsgResp, UniCatchupReq, UniEnsBlockReq},
    },
    invalid_data,
};

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
pub enum Payload {
    MsgOfInterest(MsgOfInterest),
    ProposalPayload(Box<ProposalPayload>),
    AgreementVote(Box<AgreementVote>),
    Ping(PingData),
    PingReply(PingData),
    UniEnsBlockReq(UniEnsBlockReq),
    UniCatchupReq(UniCatchupReq),
    TopicMsgResp(TopicMsgResp),
    NetPrioResponse(NetPrioResponse),
    MsgDigestSkip(HashDigest),
    NotImplemented,
}

/// Payload data for the [Ping] and [PingReply] messages.
#[derive(Debug, Clone)]
pub struct PingData {
    /// It usually contains random bytes used for matching Ping-PingReply messages.
    pub nonce: [u8; 8],
}

/// [PayloadCodec] decodes the Algod message payload using a provided tag.
#[derive(Clone)]
pub struct PayloadCodec {
    /// The associated node's span.
    span: Span,

    /// Represents a message payload type identifier.
    // Should be set by the outer codec so that this codec knows how to interpret the payload.
    pub tag: Option<Tag>,

    /// Codec for topics which are key-value string pairs.
    topic: TopicCodec,
}

impl PayloadCodec {
    pub fn new(span: Span) -> Self {
        Self {
            span,
            tag: None,
            topic: TopicCodec::default(),
        }
    }
}

impl Decoder for PayloadCodec {
    type Item = Payload;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let tag = self.tag.expect("tag not set");

        let payload = match tag {
            Tag::MsgOfInterest | Tag::TopicMsgResp => {
                self.topic.tag = Some(tag);
                self.topic
                    .decode(src)?
                    .ok_or_else(|| invalid_data!("payload not found"))?
            }
            Tag::ProposalPayload => {
                Payload::ProposalPayload(rmp_serde::from_slice(src).map_err(|_| {
                    invalid_data!("couldn't deserialize the ProposalPayload message")
                })?)
            }
            Tag::AgreementVote => Payload::AgreementVote(
                rmp_serde::from_slice(src)
                    .map_err(|_| invalid_data!("couldn't deserialize the AgreementVote message"))?,
            ),
            Tag::MsgDigestSkip => Payload::MsgDigestSkip(HashDigest(
                src.to_vec()
                    .try_into()
                    .map_err(|_| invalid_data!("invalid hash digest for MsgDigestSkip"))?,
            )),
            Tag::NetPrioResponse => {
                Payload::NetPrioResponse(rmp_serde::from_slice(src).map_err(|_| {
                    invalid_data!("couldn't deserialize the NetPrioResponse message")
                })?)
            }
            _ => return Ok(Some(Payload::NotImplemented)),
        };

        tracing::debug!(parent: &self.span, "decoded the payload");
        Ok(Some(payload))
    }
}

impl Encoder<Payload> for PayloadCodec {
    type Error = io::Error;

    fn encode(&mut self, message: Payload, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let raw_data = match message {
            Payload::MsgOfInterest(_) | Payload::UniEnsBlockReq(_) | Payload::UniCatchupReq(_) => {
                return self
                    .topic
                    .encode(message, dst)
                    .map_err(|_| invalid_data!("couldn't encode a payload message"));
            }
            Payload::ProposalPayload(pp) => rmp_serde::encode::to_vec(&pp)
                .map_err(|_| invalid_data!("couldn't encode a payload message"))?,
            Payload::AgreementVote(av) => rmp_serde::encode::to_vec(&av)
                .map_err(|_| invalid_data!("couldn't encode an agreement vote message"))?,
            Payload::MsgDigestSkip(hash) => hash.0.to_vec(),
            Payload::Ping(ping) => ping.nonce.to_vec(),
            _ => unimplemented!(),
        };

        dst.put(raw_data.as_slice());
        Ok(())
    }
}
