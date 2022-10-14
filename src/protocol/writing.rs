use std::net::SocketAddr;

use pea2pea::{protocols::Writing, ConnectionSide};

use crate::{protocol::codecs::websocketcodec::WebsocketCodec, tools::inner_node::InnerNode};

impl Writing for InnerNode {
    type Message = Vec<u8>;
    type Codec = WebsocketCodec;

    fn codec(&self, _addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        Default::default()
    }
}
