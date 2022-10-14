use std::io;

use bytes::BytesMut;
use tokio_util::codec::{Decoder, Encoder};

pub struct WebsocketCodec {
    codec: websocket_codec::MessageCodec,
}

impl Default for WebsocketCodec {
    fn default() -> Self {
        Self {
            // websocket_codec uses `true` for the client and `false` for the server
            codec: websocket_codec::MessageCodec::with_masked_encode(true),
        }
    }
}

impl Decoder for WebsocketCodec {
    type Item = websocket_codec::Message;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        self.codec
            .decode(src)
            .map_err(|_| io::ErrorKind::InvalidData.into())
    }
}

impl Encoder<Vec<u8>> for WebsocketCodec {
    type Error = io::Error;

    fn encode(&mut self, item: Vec<u8>, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let message = websocket_codec::Message::binary(item);
        self.codec
            .encode(message, dst)
            .map_err(|_| io::ErrorKind::InvalidData.into())
    }
}
