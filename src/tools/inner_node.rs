use std::net::SocketAddr;

use pea2pea::{Node, Pea2Pea};
use tokio::sync::mpsc::Sender;

use crate::protocol::{codecs::algomsg::AlgoMsg, handshake::HandshakeCfg};

#[derive(Clone)]
pub struct InnerNode {
    node: Node,
    pub handshake_cfg: HandshakeCfg,
    pub inbound_tx: Sender<(SocketAddr, AlgoMsg)>,
}

impl InnerNode {
    pub async fn new(
        node: Node,
        tx: Sender<(SocketAddr, AlgoMsg)>,
        handshake_cfg: HandshakeCfg,
    ) -> Self {
        Self {
            node,
            inbound_tx: tx,
            handshake_cfg,
        }
    }
}

impl Pea2Pea for InnerNode {
    fn node(&self) -> &Node {
        &self.node
    }
}
