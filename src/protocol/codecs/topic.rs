use std::{
    collections::HashSet,
    io::{self, ErrorKind},
};

use bytes::{Buf, BytesMut};
use tokio_util::codec::Decoder;

use crate::protocol::{
    codecs::{payload::Payload, tagmsg::Tag},
    invalid_data,
};

/// [MsgOfInterest] contains a tag list in which the node is interested.
#[derive(Debug)]
pub struct MsgOfInterest {
    /// Message tags for which the node is interested (subscribed).
    pub tags: HashSet<Tag>,
}

impl MsgOfInterest {
    /// Constructs a [MsgOfInterest] payload from the [topics] list.
    pub fn construct(mut topics: Vec<Topic>) -> Result<Self, io::Error> {
        if topics.len() != 1 {
            return Err(invalid_data!("expected a single topic"));
        }

        let tag_topic = topics.pop().unwrap();
        if tag_topic.key != "tags" {
            return Err(invalid_data!("expected 'tags' topic"));
        }

        let tags: HashSet<_> = tag_topic
            .value
            .split(',')
            .map(Tag::try_from)
            .collect::<io::Result<_>>()?;

        Ok(Self { tags })
    }
}

/// Topic is a key-value string pair.
pub struct Topic {
    /// Key.
    pub key: String,

    /// Value.
    pub value: String,
}

#[derive(Default, Clone)]
pub struct TopicCodec {
    /// Represents a message payload type identifier.
    // Should be set by the outer codec so that this codec knows how to interpret the payload.
    pub tag: Option<Tag>,
}

impl TopicCodec {
    fn parse_topics(&mut self, src: &mut BytesMut) -> Result<Vec<Topic>, io::Error> {
        let num_topics = src.get_u8();
        let mut topics = Vec::with_capacity(num_topics as usize);

        for _ in 0..num_topics {
            let key_len = src.get_u8() as usize;
            if key_len > src.len() {
                return Err(invalid_data!("invalid topic length"));
            }
            let key = src.copy_to_bytes(key_len).to_vec();

            let val_len = src.get_u8() as usize;
            if val_len > src.len() {
                return Err(invalid_data!("invalid topic length"));
            }
            let val = src.copy_to_bytes(val_len).to_vec();

            let key = String::from_utf8(key).map_err(|_| ErrorKind::InvalidData)?;
            let value = String::from_utf8(val).map_err(|_| ErrorKind::InvalidData)?;
            topics.push(Topic { key, value });
        }

        Ok(topics)
    }
}

impl Decoder for TopicCodec {
    type Item = Payload;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let tag = self.tag.expect("tag not set");
        let topics = self.parse_topics(src)?;

        let payload = match tag {
            Tag::MsgOfInterest => Payload::MsgOfInterest(MsgOfInterest::construct(topics)?),
            _ => Payload::NotImplemented,
        };

        Ok(Some(payload))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_invalid_byte_stream() {
        #[rustfmt::skip]
        let byte_stream = [
            2, // two topics
            100, b'k', b'e', b'y', // invalid data length
        ];

        let mut bytes_mut = BytesMut::new();
        bytes_mut.extend_from_slice(&byte_stream);

        assert!(TopicCodec::default().parse_topics(&mut bytes_mut).is_err());
    }

    #[test]
    fn decode_valid_byte_stream() {
        #[rustfmt::skip]
        let byte_stream = [
            2, // two topics
            3, b'k', b'e', b'y', // "key"
            3, b'v', b'a', b'l', // "val"
            1, b'a', // "a"
            4, b'b', b'c', b'd', b'e', // "bcde"
        ];

        let mut bytes_mut = BytesMut::new();
        bytes_mut.extend_from_slice(&byte_stream);

        let mut topics = TopicCodec::default()
            .parse_topics(&mut bytes_mut)
            .expect("couldn't parse the byte stream");
        let mut take_and_check_topic = |key, val| {
            let topic = topics.remove(0);
            assert_eq!(topic.key, key);
            assert_eq!(topic.value, val);
        };

        take_and_check_topic("key", "val");
        take_and_check_topic("a", "bcde");
        assert!(topics.is_empty());
    }
}
