use std::io;

use bytes::BytesMut;
use tokio_util::codec::Decoder;
use tracing::Span;

use crate::protocol::{
    codecs::{
        tagmsg::Tag,
        topic::{MsgOfInterest, TopicCodec},
    },
    invalid_data,
};

#[derive(Debug)]
pub enum Payload {
    MsgOfInterest(MsgOfInterest),
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
            _ => return Ok(Some(Payload::NotImplemented)),
        };

        tracing::debug!(parent: &self.span, "decoded the payload");
        Ok(Some(payload))
    }
}
