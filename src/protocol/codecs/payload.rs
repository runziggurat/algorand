use std::io;

use bytes::BytesMut;
use tokio_util::codec::Decoder;
use tracing::Span;

use crate::protocol::{
    codecs::{
        msgpack::{AgreementVote, ProposalPayload},
        tagmsg::Tag,
        topic::{MsgOfInterest, TopicCodec},
    },
    invalid_data,
};

#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Payload {
    MsgOfInterest(MsgOfInterest),
    ProposalPayload(Box<ProposalPayload>),
    AgreementVote(Box<AgreementVote>),
    NotImplemented,
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
            Tag::MsgOfInterest => {
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
            _ => return Ok(Some(Payload::NotImplemented)),
        };

        tracing::debug!(parent: &self.span, "decoded the payload");
        Ok(Some(payload))
    }
}
