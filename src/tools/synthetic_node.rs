//! A lightweight node implementation to be used as peers in tests.

use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use pea2pea::{
    protocols::{Reading, Writing},
    Config as NodeConfig, Node, Pea2Pea,
};
use tokio::sync::mpsc::{self, Receiver};
use websocket_codec::Message;

use crate::tools::inner_node::InnerNode;

/// Enables tracing for all [`SyntheticNode`] instances (usually scoped by test).
pub fn enable_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};

    fmt()
        .with_test_writer()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

/// A builder for [`SyntheticNode`].
#[derive(Debug, Clone)]
pub struct SyntheticNodeBuilder {
    /// [`pea2pea`] node configuration.
    network_config: NodeConfig,
    /// Whether or not to call `enable_handshake` when creating a new node.
    handshake: bool,
}

impl Default for SyntheticNodeBuilder {
    fn default() -> Self {
        Self {
            network_config: NodeConfig {
                listener_ip: Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                ..Default::default()
            },
            handshake: true,
        }
    }
}

impl SyntheticNodeBuilder {
    /// Creates a [`SyntheticNode`] with the current configuration.
    pub async fn build(&self) -> io::Result<SyntheticNode> {
        // Create the pea2pea node from the config.
        let node = Node::new(self.network_config.clone()).await?;

        // Inbound channel size of 100 messages.
        let (tx, rx) = mpsc::channel(100);

        let inner_node = InnerNode::new(node, tx, self.handshake).await;

        // Enable the read and write protocols.
        inner_node.enable_reading().await;
        inner_node.enable_writing().await;

        Ok(SyntheticNode {
            inner: inner_node,
            inbound_rx: rx,
        })
    }

    /// Choose whether or not the node should perform the handshake procedure.
    pub fn with_handshake(mut self, handshake: bool) -> Self {
        self.handshake = handshake;
        self
    }
}

/// Convenient abstraction over a `pea2pea` node.
pub struct SyntheticNode {
    inner: InnerNode,
    inbound_rx: Receiver<(SocketAddr, Message)>,
}

impl SyntheticNode {
    /// Connects to the target address.
    ///
    /// If the handshake protocol is enabled it will be executed as well.
    pub async fn connect(&self, target: SocketAddr) -> io::Result<()> {
        self.inner.node().connect(target).await
    }

    /// Indicates if the `addr` is registered as a connected peer.
    pub fn is_connected(&self, addr: SocketAddr) -> bool {
        self.inner.node().is_connected(addr)
    }

    /// Gracefully shuts down the node.
    pub async fn shut_down(&self) {
        self.inner.node().shut_down().await
    }

    /// Reads a message from the inbound (internal) queue of the node.
    pub async fn recv_message(&mut self) -> (SocketAddr, Message) {
        // TODO(Rqnsom): Decode message.data()
        match self.inbound_rx.recv().await {
            Some(message) => message,
            None => panic!("all senders dropped!"),
        }
    }
}
