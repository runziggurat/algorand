use std::net::SocketAddr;

use pea2pea::{Node, Pea2Pea};
use tokio::sync::mpsc::Sender;

use crate::protocol::codecs::payload::Payload;

#[derive(Clone)]
pub struct InnerNode {
    node: Node,
    pub challenge: Option<String>,
    pub inbound_tx: Sender<(SocketAddr, Payload)>,
}

impl InnerNode {
    pub async fn new(
        node: Node,
        tx: Sender<(SocketAddr, Payload)>,
        challenge: Option<String>,
    ) -> Self {
        Self {
            node,
            inbound_tx: tx,
            challenge,
        }
    }
}

impl Pea2Pea for InnerNode {
    fn node(&self) -> &Node {
        &self.node
    }
}
