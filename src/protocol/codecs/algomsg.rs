use std::io::{self, ErrorKind};

use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};
use tracing::{debug, warn, Span};
use websocket_codec::Opcode;

use crate::protocol::{
    codecs::{payload::Payload, tagmsg::TagMsgCodec, websocket::WebsocketCodec},
    invalid_data,
};

/// Algorand message.
#[derive(Debug)]
pub struct AlgoMsg {
    /// Message in the raw byte format.
    pub raw: Vec<u8>,
    /// Parsed message.
    pub payload: Payload,
}

pub struct AlgoMsgCodec {
    websocket: WebsocketCodec,
    tagmsg: TagMsgCodec,
    span: Span,
}

impl AlgoMsgCodec {
    pub fn new(span: Span) -> Self {
        Self {
            websocket: WebsocketCodec::default(),
            tagmsg: TagMsgCodec::new(span.clone()),
            span,
        }
    }
}

impl Decoder for AlgoMsgCodec {
    type Item = AlgoMsg;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let ws_msg = if let Some(src) = self.websocket.decode(src)? {
            src
        } else {
            return Ok(None);
        };

        debug!(parent: &self.span, "got a WebSocket message: {:?}", ws_msg);

        // Only binary messages are expected.
        if ws_msg.opcode() != Opcode::Binary {
            warn!(parent: &self.span, "not a binary opcode");
            return Err(invalid_data!("expected a binary opcode"));
        }

        let mut ws_data =
            BytesMut::try_from(ws_msg.data().as_ref()).map_err(|_| ErrorKind::InvalidData)?;
        let raw = ws_data.to_vec();

        let payload = self
            .tagmsg
            .decode(&mut ws_data)
            .map_err(|_| invalid_data!("invalid algod message"))?
            .ok_or_else(|| invalid_data!("missing algod message"))?;

        Ok(Some(AlgoMsg { raw, payload }))
    }
}

impl Encoder<Payload> for AlgoMsgCodec {
    type Error = io::Error;

    fn encode(&mut self, message: Payload, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let mut tag_msg = BytesMut::new();

        self.tagmsg
            .encode(message, &mut tag_msg)
            .map_err(|_| invalid_data!("couldn't encode a tagmsg message"))?;

        self.websocket
            .encode(tag_msg.to_vec(), dst)
            .map_err(|_| invalid_data!("couldn't encode a WebSocket message"))
    }
}
