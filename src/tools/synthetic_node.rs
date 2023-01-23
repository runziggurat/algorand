//! A lightweight node implementation to be used as peers in tests.

use std::{
    io,
    net::{IpAddr, Ipv4Addr, SocketAddr},
};

use pea2pea::{
    protocols::{Handshake, Reading, Writing},
    Config as NodeConfig, Node, Pea2Pea,
};
use tokio::{
    net::TcpSocket,
    sync::mpsc::{self, Receiver},
    time::{sleep, timeout, Duration},
};
use tracing::trace;

use crate::{
    protocol::{
        codecs::{algomsg::AlgoMsg, payload::Payload},
        handshake::HandshakeCfg,
    },
    tools::{constants::EXPECT_MSG_TIMEOUT, inner_node::InnerNode},
};

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
    /// Network priority challenge sent to clients which try to connect to the node.
    handshake_cfg: HandshakeCfg,
}

impl Default for SyntheticNodeBuilder {
    fn default() -> Self {
        Self {
            network_config: NodeConfig {
                listener_ip: Some(IpAddr::V4(Ipv4Addr::LOCALHOST)),
                ..Default::default()
            },
            handshake: true,
            handshake_cfg: Default::default(),
        }
    }
}

impl SyntheticNodeBuilder {
    /// Creates a [`SyntheticNode`] with the current configuration.
    pub async fn build(&self) -> io::Result<SyntheticNode> {
        // Create the pea2pea node from the config.
        let node = Node::new(self.network_config.clone());

        // Inbound channel size of 100 messages.
        let (tx, rx) = mpsc::channel(100);

        let inner_node = InnerNode::new(node, tx, self.handshake_cfg.clone()).await;

        // Enable the handshake protocol.
        if self.handshake {
            inner_node.enable_handshake().await;
        }
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

    /// Choose the handshake configuration.
    pub fn with_handshake_configuration(mut self, cfg: HandshakeCfg) -> Self {
        self.handshake_cfg = cfg;
        self
    }
}

/// Convenient abstraction over a `pea2pea` node.
pub struct SyntheticNode {
    inner: InnerNode,
    inbound_rx: Receiver<(SocketAddr, AlgoMsg)>,
}

impl SyntheticNode {
    /// Connects to the target address.
    ///
    /// If the handshake protocol is enabled it will be executed as well.
    pub async fn connect(&self, target: SocketAddr) -> io::Result<()> {
        self.inner.node().connect(target).await
    }

    /// Connects to the target address using specified source socket.
    ///
    /// If the handshake protocol is enabled it will be executed as well.
    pub async fn connect_from(&self, target: SocketAddr, source: TcpSocket) -> io::Result<()> {
        self.inner.node().connect_using_socket(target, source).await
    }

    /// Starts listening for inbound connections.
    ///
    /// Returns the listening socket address.
    pub async fn start_listening(&self) -> io::Result<SocketAddr> {
        self.inner.node().start_listening().await
    }

    /// Indicates if the `addr` is registered as a connected peer.
    pub fn is_connected(&self, addr: SocketAddr) -> bool {
        self.inner.node().is_connected(addr)
    }

    /// Returns the number of connected peers.
    pub fn num_connected(&self) -> usize {
        self.inner.node().num_connected()
    }

    /// Returns the list of active connections for this node.
    pub fn connected_peers(&self) -> Vec<SocketAddr> {
        self.inner.node().connected_addrs()
    }

    /// Waits until the node has at least one connection, and returns its SocketAddr.
    pub async fn wait_for_connection(&self) -> SocketAddr {
        const SLEEP: Duration = Duration::from_millis(50);
        loop {
            // Mutating the collection is alright since this is a copy of the connections and not the actual list.
            if let Some(addr) = self.connected_peers().pop() {
                return addr;
            }

            sleep(SLEEP).await;
        }
    }

    /// Returns the listening address of the node.
    pub fn listening_addr(&self) -> io::Result<SocketAddr> {
        self.inner.node().listening_addr()
    }

    /// Gracefully shuts down the node.
    pub async fn shut_down(&self) {
        self.inner.node().shut_down().await
    }

    /// Sends a direct message to the target address.
    pub fn unicast(&self, target: SocketAddr, message: Payload) -> io::Result<()> {
        trace!(parent: self.inner.node().span(), "unicast send msg to {target}: {:?}", message);
        self.inner.unicast(target, message)?;
        Ok(())
    }

    /// Reads a message from the inbound (internal) queue of the node.
    pub async fn recv_message(&mut self) -> (SocketAddr, AlgoMsg) {
        match self.inbound_rx.recv().await {
            Some(msg) => msg,
            None => panic!("all senders dropped"),
        }
    }

    /// Attempts to read a message from the inbound (internal) queue of the node before the
    /// timeout duration has elapsed.
    pub async fn recv_message_timeout(
        &mut self,
        duration: Duration,
    ) -> io::Result<(SocketAddr, AlgoMsg)> {
        match timeout(duration, self.recv_message()).await {
            Ok(msg) => Ok(msg),
            Err(_) => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                format!("could not read the message after: {duration:?}"),
            )),
        }
    }

    /// Expects a message.
    pub async fn expect_message(
        &mut self,
        check: &dyn Fn(&Payload) -> bool,
        override_timeout: Option<Duration>,
    ) -> bool {
        let duration = override_timeout.unwrap_or(EXPECT_MSG_TIMEOUT);

        timeout(duration, async {
            loop {
                let (_, msg) = self.recv_message().await;
                if check(&msg.payload) {
                    return true;
                }
            }
        })
        .await
        .is_ok()
    }
}
