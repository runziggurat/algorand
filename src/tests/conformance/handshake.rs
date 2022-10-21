use tempfile::TempDir;
use tokio::time::timeout;

use crate::{
    setup::node::Node,
    tools::{constants::CONNECTION_TIMEOUT, synthetic_node::SyntheticNodeBuilder},
};

#[tokio::test]
async fn c001_handshake_when_node_receives_connection() {
    // ZG-CONFORMANCE-001

    // Spin up a node instance.
    let target = TempDir::new().expect("Couldn't create a temporary directory");
    let mut node = Node::builder()
        .build(target.path())
        .expect("Unable to build the node");
    node.start().await;

    // Create a synthetic node and enable handshaking.
    let synthetic_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect("Unable to build a synthetic node");

    let net_addr = node.net_addr().expect("Network address not found");

    // Connect to the node and initiate the handshake.
    synthetic_node
        .connect(net_addr)
        .await
        .expect("Unable to connect");

    // This is only set post-handshake (if enabled).
    assert!(
        synthetic_node.is_connected(net_addr),
        "Synthetic node is not connected to the node"
    );

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("Unable to stop the node");
}

#[tokio::test]
async fn c002_handshake_when_node_initiates_connection() {
    // ZG-CONFORMANCE-002

    // Create a synthetic node and enable handshaking.
    let synthetic_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect("Unable to build a synthetic node");

    // Spin up a node instance.
    let target = TempDir::new().expect("Couldn't create a temporary directory");
    let mut node = Node::builder()
        .initial_peers([synthetic_node
            .listening_addr()
            .expect("Listening address not found")])
        .build(target.path())
        .expect("Unable to build the node");
    node.start().await;

    let node_addr = timeout(CONNECTION_TIMEOUT, synthetic_node.wait_for_connection())
        .await
        .expect("Couldn't establish a connection");

    // Check the connection has been established (this is only set post-handshake). We can't check
    // for the addr as nodes use ephemeral addresses when initiating connections.
    assert_ne!(
        node_addr,
        node.net_addr().expect("Network address not found")
    );

    // The node sends multiple get_block HTTP queries from different TCP sockets in parallel,
    // so on rare occasions we might have additional few short-lasting connections.
    assert!(
        synthetic_node.num_connected() >= 1,
        "At least one connection is expected"
    );

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("Unable to stop the node");
}
