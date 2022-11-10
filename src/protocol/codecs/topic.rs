use std::{
    collections::HashSet,
    io::{self, ErrorKind},
};

use bytes::{Buf, BufMut, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::protocol::{
    codecs::{payload::Payload, tagmsg::Tag},
    invalid_data,
};

/// Topic keys.
const TOPIC_KEY_TAGS: &str = "tags";

/// [MsgOfInterest] contains a tag list in which the node is interested.
#[derive(Debug)]
pub struct MsgOfInterest {
    /// Message tags for which the node is interested (subscribed).
    pub tags: HashSet<Tag>,
}

impl TryFrom<Vec<Topic>> for MsgOfInterest {
    type Error = io::Error;

    fn try_from(mut topics: Vec<Topic>) -> Result<Self, Self::Error> {
        if topics.len() != 1 {
            return Err(invalid_data!("expected a single topic"));
        }

        let tag_topic = topics.pop().unwrap();
        if tag_topic.key != TOPIC_KEY_TAGS {
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

impl MsgOfInterest {
    /// Convert the message to a corresponding [Topic].
    #[allow(clippy::wrong_self_convention)]
    fn to_topic(self) -> Topic {
        let value = self
            .tags
            .into_iter()
            .map(|tag| tag.get_tag_str().to_string())
            .collect::<Vec<String>>()
            .join(",");

        Topic {
            key: TOPIC_KEY_TAGS.into(),
            value,
        }
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
    /// Unmarshall topics from a byte stream.
    fn unmarshall_topics(&mut self, src: &mut BytesMut) -> Result<Vec<Topic>, io::Error> {
        // The maximum number of topics allowed is 32.
        let num_topics = src.get_u8();
        let mut topics = Vec::with_capacity(num_topics as usize);

        for _ in 0..num_topics {
            // Each topic key can be 64 characters long and cannot be size 0.
            let key_len = src.get_u8() as usize;
            if key_len > src.len() {
                return Err(invalid_data!("invalid topic length"));
            }
            let key = src.copy_to_bytes(key_len).to_vec();

            // For handled messages so far, the max data size fits into the u8 integer.
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

    /// Marshall topics to a byte stream.
    fn marshall_topics(&mut self, topics: Vec<Topic>) -> BytesMut {
        // The maximum number of topics allowed is 32.
        let num_topics = topics.len() as u8;

        let mut raw_data = BytesMut::new();
        raw_data.put_u8(num_topics);

        for topic in topics.into_iter() {
            // Each topic key can be 64 characters long and cannot be size 0.
            raw_data.put_u8(topic.key.len() as u8);
            raw_data.put(topic.key.as_bytes());

            // For messages so far, the max data size fits into the u8 integer.
            raw_data.put_u8(topic.value.len() as u8);
            raw_data.put(topic.value.as_bytes());
        }

        raw_data
    }
}

impl Decoder for TopicCodec {
    type Item = Payload;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let tag = self.tag.expect("tag not set");
        let topics = self.unmarshall_topics(src)?;

        let payload = match tag {
            Tag::MsgOfInterest => Payload::MsgOfInterest(MsgOfInterest::try_from(topics)?),
            _ => Payload::NotImplemented,
        };

        Ok(Some(payload))
    }
}

impl Encoder<Payload> for TopicCodec {
    type Error = io::Error;

    fn encode(&mut self, message: Payload, dst: &mut BytesMut) -> Result<(), Self::Error> {
        //TODO: remove the allow statement once we add more topics messages here.
        #[allow(clippy::single_match)]
        let topics: Vec<Topic> = match message {
            Payload::MsgOfInterest(msg) => vec![msg.to_topic()],
            _ => panic!("a topic encoder can only encode topic messages"),
        };

        dst.put(self.marshall_topics(topics));

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[rustfmt::skip]
    const VALID_TOPIC_BYTE_STREAM: [u8; 16] = [
        2, // two topics
        3, b'k', b'e', b'y', // "key"
        3, b'v', b'a', b'l', // "val"
        1, b'a', // "a"
        4, b'b', b'c', b'd', b'e', // "bcde"
    ];

    #[test]
    fn unmarshall_invalid_byte_stream() {
        #[rustfmt::skip]
        let byte_stream = [
            2, // two topics
            100, b'k', b'e', b'y', // invalid data length
        ];

        let mut bytes_mut = BytesMut::new();
        bytes_mut.extend_from_slice(&byte_stream);

        assert!(TopicCodec::default()
            .unmarshall_topics(&mut bytes_mut)
            .is_err());
    }

    #[test]
    fn unmarshall_valid_byte_stream() {
        let mut bytes_mut = BytesMut::new();
        bytes_mut.extend_from_slice(&VALID_TOPIC_BYTE_STREAM);

        let mut topics = TopicCodec::default()
            .unmarshall_topics(&mut bytes_mut)
            .expect("couldn't unmarshall the byte stream");
        let mut take_and_check_topic = |key, val| {
            let topic = topics.remove(0);
            assert_eq!(topic.key, key);
            assert_eq!(topic.value, val);
        };

        take_and_check_topic("key", "val");
        take_and_check_topic("a", "bcde");
        assert!(topics.is_empty());
    }

    #[test]
    fn marshall_topics() {
        let topics = vec![
            Topic {
                key: "key".into(),
                value: "val".into(),
            },
            Topic {
                key: "a".into(),
                value: "bcde".into(),
            },
        ];

        let mut bytes_mut = BytesMut::new();
        bytes_mut.extend_from_slice(&VALID_TOPIC_BYTE_STREAM);

        assert_eq!(bytes_mut, TopicCodec::default().marshall_topics(topics));
    }
}
