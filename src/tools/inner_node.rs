use std::net::SocketAddr;

use pea2pea::{protocols::Handshake, Node, Pea2Pea};
use tokio::sync::mpsc::Sender;
use websocket_codec::Message;

#[derive(Clone)]
pub struct InnerNode {
    node: Node,
    pub inbound_tx: Sender<(SocketAddr, Message)>,
}

impl InnerNode {
    pub async fn new(node: Node, tx: Sender<(SocketAddr, Message)>, handshake: bool) -> Self {
        let node = Self {
            node,
            inbound_tx: tx,
        };

        if handshake {
            node.enable_handshake().await;
        }

        node
    }
}

impl Pea2Pea for InnerNode {
    fn node(&self) -> &Node {
        &self.node
    }
}
