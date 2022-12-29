use std::time::Duration;

use tempfile::TempDir;
use tokio::time::timeout;

use crate::{
    protocol::codecs::payload::Payload,
    setup::node::Node,
    tools::{
        constants::{
            CONNECTION_TIMEOUT, ERR_NODE_ADDR, ERR_NODE_BUILD, ERR_NODE_STOP, ERR_SYNTH_BUILD,
            ERR_SYNTH_CONNECT, ERR_TEMPDIR_NEW,
        },
        synthetic_node::SyntheticNodeBuilder,
    },
};

#[tokio::test]
async fn c001_handshake_when_node_receives_connection() {
    // ZG-CONFORMANCE-001

    // Spin up a node instance.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder().build(target.path()).expect(ERR_NODE_BUILD);
    node.start().await;

    // Create a synthetic node and enable handshaking.
    let synthetic_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect(ERR_SYNTH_BUILD);

    let net_addr = node.net_addr().expect(ERR_NODE_ADDR);

    // Connect to the node and initiate the handshake.
    synthetic_node
        .connect(net_addr)
        .await
        .expect(ERR_SYNTH_CONNECT);

    // This is only set post-handshake (if enabled).
    assert!(
        synthetic_node.is_connected(net_addr),
        "synthetic node is not connected to the node"
    );

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}

#[tokio::test]
async fn c002_handshake_when_node_initiates_connection() {
    // ZG-CONFORMANCE-002

    // Create a synthetic node and enable handshaking.
    let synthetic_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect(ERR_SYNTH_BUILD);

    let listening_addr = synthetic_node
        .start_listening()
        .await
        .expect("a synthetic node couldn't start listening");

    // Spin up a node instance.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder()
        .initial_peers([listening_addr])
        .build(target.path())
        .expect(ERR_NODE_BUILD);
    node.start().await;

    let node_addr = timeout(CONNECTION_TIMEOUT, synthetic_node.wait_for_connection())
        .await
        .expect("couldn't establish a connection");

    // Check the connection has been established (this is only set post-handshake). We can't check
    // for the addr as nodes use ephemeral addresses when initiating connections.
    assert_ne!(node_addr, node.net_addr().expect(ERR_NODE_ADDR));

    // The node sends multiple get_block HTTP queries from different TCP sockets in parallel,
    // so on rare occasions we might have additional few short-lasting connections.
    assert!(
        synthetic_node.num_connected() >= 1,
        "at least one connection is expected"
    );

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}

const NO_MSG_TIMEOUT: Option<Duration> = Some(Duration::from_secs(5));

#[tokio::test]
async fn c003_t1_expect_no_messages_before_handshake() {
    // ZG-CONFORMANCE-003
    //
    // A synthetic node with a disabled handshake procedure expects zero messages
    // after it initiates a connection with the node.

    // Spin up a node instance.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder().build(target.path()).expect(ERR_NODE_BUILD);
    node.start().await;

    // Create a synthetic node and enable handshaking.
    let mut synthetic_node = SyntheticNodeBuilder::default()
        .with_handshake(false)
        .build()
        .await
        .expect(ERR_SYNTH_BUILD);

    let net_addr = node.net_addr().expect(ERR_NODE_ADDR);

    // Connect to the node and initiate the handshake.
    synthetic_node
        .connect(net_addr)
        .await
        .expect(ERR_SYNTH_CONNECT);

    let expect_any_msg = |_: &Payload| true;
    assert!(
        !synthetic_node
            .expect_message(&expect_any_msg, NO_MSG_TIMEOUT)
            .await
    );

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}

// NOTE: Maybe this test makes no sense because we do get bombarded with the GET_BLOCK requests,
// but our Reading thread still doesn't know how to parse those so we get a pea2pea invalid data error.
// Conclusion: We'll keep the test - to be used for future testing.
#[tokio::test]
async fn c003_t2_expect_no_messages_before_handshake() {
    // ZG-CONFORMANCE-003
    //
    // A synthetic node with a disabled handshake procedure expects zero messages
    // after receiving a connection initiated by the node.

    // Create a synthetic node and enable handshaking.
    let mut synthetic_node = SyntheticNodeBuilder::default()
        .with_handshake(false)
        .build()
        .await
        .expect(ERR_SYNTH_BUILD);

    let listening_addr = synthetic_node
        .start_listening()
        .await
        .expect("a synthetic node couldn't start listening");

    // Spin up a node instance.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder()
        .initial_peers([listening_addr])
        .build(target.path())
        .expect(ERR_NODE_BUILD);
    node.start().await;

    let expect_any_msg = |_: &Payload| true;
    assert!(
        !synthetic_node
            .expect_message(&expect_any_msg, NO_MSG_TIMEOUT)
            .await
    );

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}
