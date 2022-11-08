use std::net::SocketAddr;

use pea2pea::{protocols::Writing, ConnectionSide, Pea2Pea};

use crate::{
    protocol::codecs::{algomsg::AlgoMsgCodec, payload::Payload},
    tools::inner_node::InnerNode,
};

impl Writing for InnerNode {
    type Message = Payload;
    type Codec = AlgoMsgCodec;

    fn codec(&self, _addr: SocketAddr, _side: ConnectionSide) -> Self::Codec {
        AlgoMsgCodec::new(self.node().span().clone())
    }
}
