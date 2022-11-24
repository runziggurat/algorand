use std::{
    collections::HashSet,
    io::{self, ErrorKind},
};

use bytes::{Buf, BufMut, Bytes, BytesMut};
use tokio_util::codec::{Decoder, Encoder};

use crate::{
    protocol::{
        codecs::{msgpack::Round, payload::Payload, tagmsg::Tag},
        invalid_data,
    },
    tools::rpc::{BlockHeaderMsgPack, Certificate},
};

/// Topic keys.
const TOPIC_KEY_TAGS: &str = "tags";
const TOPIC_KEY_ROUND: &str = "roundKey";
const TOPIC_KEY_DATA_TYPE: &str = "requestDataType";
const TOPIC_KEY_HASH: &str = "RequestHash";
const TOPIC_KEY_ERROR: &str = "Error";
const TOPIC_KEY_NONCE: &str = "nonce";
const TOPIC_KEY_CERT_DATA: &str = "certData";
const TOPIC_KEY_BLOCK_DATA: &str = "blockData";

/// [MsgOfInterest] contains a tag list in which the node is interested.
#[derive(Debug, Clone)]
pub struct MsgOfInterest {
    /// Message tags for which the node is interested (subscribed).
    pub tags: HashSet<Tag>,
}

/// Universal block request types.
#[derive(Debug, Copy, Clone)]
pub enum UniEnsBlockReqType {
    /// Block-data topic-key in the response.
    Block,
    /// Cert-data topic-key in the response.
    Cert,
    /// block+cert request data (as the value of requestDataTypeKey).
    BlockAndCert,
}

/// Universal block request message.
#[derive(Debug, Clone)]
pub struct UniEnsBlockReq {
    /// Request option.
    pub data_type: UniEnsBlockReqType,
    /// Round in which block was created.
    pub round_key: Round,
    /// Nonce for a unique request identification.
    pub nonce: u64,
}

/// [TopicMsgResp] contains all possible responses which are received in the form of topics.
#[derive(Debug, Clone)]
pub enum TopicMsgResp {
    /// Universal response to a block request message.
    UniEnsBlockRsp(Box<UniEnsBlockRsp>),
    /// Error response.
    ErrorRsp(ErrorRsp),
}

/// Universal block response message.
#[derive(Debug, Clone, Default)]
pub struct UniEnsBlockRsp {
    /// Block header data.
    pub block: Option<BlockHeaderMsgPack>,
    /// Certificate.
    pub cert: Option<Certificate>,
    /// Used to match a request message.
    pub request_hash: Bytes,
}

/// Universal error response message.
#[derive(Debug, Clone, Default)]
pub struct ErrorRsp {
    /// Error description.
    pub error: String,
    /// Used to match a request message.
    pub request_hash: Bytes,
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

        let tags: HashSet<_> = String::from_utf8(tag_topic.value.to_vec())
            .map_err(|_| invalid_data!("'tags' value is not a valid UTF-8 string"))?
            .split(',')
            .map(Tag::try_from)
            .collect::<io::Result<_>>()?;

        Ok(Self { tags })
    }
}

impl TryFrom<Vec<Topic>> for TopicMsgResp {
    type Error = io::Error;

    fn try_from(topics: Vec<Topic>) -> Result<Self, Self::Error> {
        // Simply use the number of topics to identify underlying messages.
        match topics.len() {
            2 => Ok(TopicMsgResp::ErrorRsp(ErrorRsp::try_from(topics)?)),
            3 => Ok(TopicMsgResp::UniEnsBlockRsp(Box::new(
                UniEnsBlockRsp::try_from(topics)?,
            ))),
            _ => Err(invalid_data!("unexpected number of topics")),
        }
    }
}

impl TryFrom<Vec<Topic>> for ErrorRsp {
    type Error = io::Error;

    fn try_from(mut topics: Vec<Topic>) -> Result<Self, Self::Error> {
        let mut err_rsp = ErrorRsp::default();

        while let Some(topic) = topics.pop() {
            match topic.key.as_str() {
                TOPIC_KEY_ERROR => {
                    err_rsp.error = String::from_utf8(topic.value.to_vec())
                        .map_err(|_| invalid_data!("error value is not a valid UTF-8 string"))?
                }
                TOPIC_KEY_HASH => err_rsp.request_hash = topic.value,
                _ => {
                    return Err(invalid_data!(
                        "unexpected topic for an error response message"
                    ))
                }
            }
        }

        Ok(err_rsp)
    }
}

impl TryFrom<Vec<Topic>> for UniEnsBlockRsp {
    type Error = io::Error;

    fn try_from(mut topics: Vec<Topic>) -> Result<Self, Self::Error> {
        let mut err_rsp = UniEnsBlockRsp::default();

        while let Some(topic) = topics.pop() {
            match topic.key.as_str() {
                TOPIC_KEY_BLOCK_DATA => {
                    err_rsp.block = rmp_serde::from_slice(&topic.value)
                        .map_err(|_| invalid_data!("couldn't deserialize the block data"))?
                }
                TOPIC_KEY_CERT_DATA => {
                    err_rsp.cert = rmp_serde::from_slice(&topic.value)
                        .map_err(|_| invalid_data!("couldn't deserialize the cert data"))?
                }
                TOPIC_KEY_HASH => err_rsp.request_hash = topic.value,
                _ => {
                    return Err(invalid_data!(
                        "unexpected topic for an error response message"
                    ))
                }
            }
        }

        Ok(err_rsp)
    }
}

impl UniEnsBlockReqType {
    fn get_string(self) -> String {
        match self {
            Self::Block => "blockData".into(),
            Self::Cert => "certData".into(),
            Self::BlockAndCert => "blockAndCert".into(),
        }
    }
}

impl From<UniEnsBlockReq> for Vec<Topic> {
    fn from(msg: UniEnsBlockReq) -> Self {
        let u64_to_bytes = |num| {
            let mut value = BytesMut::new();
            value.put_u64_le(num);
            value.freeze()
        };

        let round_key_topic = Topic {
            key: TOPIC_KEY_ROUND.into(),
            value: u64_to_bytes(msg.round_key),
        };
        let data_type_topic = Topic {
            key: TOPIC_KEY_DATA_TYPE.into(),
            value: Bytes::from(msg.data_type.get_string()),
        };
        let nonce_topic = Topic {
            key: TOPIC_KEY_NONCE.into(),
            value: u64_to_bytes(msg.nonce),
        };

        vec![round_key_topic, data_type_topic, nonce_topic]
    }
}

impl From<MsgOfInterest> for Vec<Topic> {
    fn from(msg: MsgOfInterest) -> Self {
        let value = msg
            .tags
            .into_iter()
            .map(|tag| tag.get_tag_str().to_string())
            .collect::<Vec<String>>()
            .join(",");

        vec![Topic {
            key: TOPIC_KEY_TAGS.into(),
            value: Bytes::from(value),
        }]
    }
}

/// Topic is a key-value string pair.
pub struct Topic {
    /// Key.
    pub key: String,

    /// Value.
    pub value: Bytes,
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

            // For handled messages so far, the max data size fits into u8/u16 integers.
            let val_len = if src[0] & 0x80 == 0 {
                src.get_u8() as usize
            } else {
                // The varint functions encode and decode single integer values using a variable-length encoding;
                // smaller values require fewer bytes. For a specification,
                // see https://developers.google.com/protocol-buffers/docs/encoding.
                // Original comment source: https://pkg.go.dev/encoding/binary#pkg-overview
                let tmp = src.get_u16_le() as usize;
                ((tmp & 0x7f00) >> 1) | tmp & 0x7f
            };
            if val_len > src.len() {
                return Err(invalid_data!("invalid topic length"));
            }
            let val = src.copy_to_bytes(val_len).to_vec();

            let key = String::from_utf8(key).map_err(|_| ErrorKind::InvalidData)?;
            let value = Bytes::from(val);
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
            raw_data.put(topic.value);
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
            Tag::TopicMsgResp => Payload::TopicMsgResp(TopicMsgResp::try_from(topics)?),
            _ => Payload::NotImplemented,
        };

        Ok(Some(payload))
    }
}

impl Encoder<Payload> for TopicCodec {
    type Error = io::Error;

    fn encode(&mut self, message: Payload, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let topics: Vec<Topic> = match message {
            Payload::MsgOfInterest(msg) => msg.into(),
            Payload::UniEnsBlockReq(msg) => msg.into(),
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
