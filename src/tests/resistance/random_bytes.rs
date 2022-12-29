use tempfile::TempDir;
use tokio::time::sleep;

use crate::{
    protocol::codecs::payload::Payload,
    setup::node::Node,
    tests::resistance::WAIT_FOR_DISCONNECT,
    tools::{
        constants::{
            ERR_NODE_ADDR, ERR_NODE_BUILD, ERR_NODE_STOP, ERR_SYNTH_BUILD, ERR_SYNTH_CONNECT,
            ERR_SYNTH_UNICAST, ERR_TEMPDIR_NEW,
        },
        synthetic_node::SyntheticNodeBuilder,
        util::gen_rand_bytes,
    },
};

/// Send some randomly generated data to the node before the handshake and check the connection status.
async fn send_random_data_to_the_node_pre_handshake(len: usize, debug: bool) -> bool {
    // Spin up a node instance.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder()
        .log_to_stdout(debug)
        .build(target.path())
        .expect(ERR_NODE_BUILD);
    node.start().await;

    // Create a synthetic node and disable handshaking.
    let synthetic_node = SyntheticNodeBuilder::default()
        .with_handshake(false)
        .build()
        .await
        .expect(ERR_SYNTH_BUILD);

    let net_addr = node.net_addr().expect(ERR_NODE_ADDR);

    // Create a connection without the handshake.
    synthetic_node
        .connect(net_addr)
        .await
        .expect(ERR_SYNTH_CONNECT);

    // Send some random data.
    let random_data_msg = Payload::RawBytes(gen_rand_bytes(len));
    synthetic_node
        .unicast(net_addr, random_data_msg)
        .expect(ERR_SYNTH_UNICAST);

    // Give some time to the node to kill our connection.
    sleep(WAIT_FOR_DISCONNECT).await;

    let is_connected = synthetic_node.is_connected(net_addr);

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);

    is_connected
}

#[tokio::test]
#[allow(non_snake_case)]
async fn r001_t1_NO_HANDSHAKE_send_random_data_but_huge_amount() {
    // ZG-RESISTANCE-001

    let debug_logs = false;

    // Test status: pass.
    let random_data_len = 100_000;
    assert!(
        !send_random_data_to_the_node_pre_handshake(random_data_len, debug_logs).await,
        "the node shouldn't keep the connection alive after sending random data"
    );
}

#[tokio::test]
#[allow(non_snake_case)]
async fn r001_t2_NO_HANDSHAKE_send_random_data_but_mid_amount() {
    // ZG-RESISTANCE-001

    let debug_logs = false;
    // Test status: mostly pass.
    let random_data_len = 1000;
    assert!(
        !send_random_data_to_the_node_pre_handshake(random_data_len, debug_logs).await,
        "the node shouldn't keep the connection alive after sending random data"
    );
}

#[tokio::test]
#[allow(non_snake_case)]
async fn r001_t3_NO_HANDSHAKE_send_random_data_but_small_amount() {
    // ZG-RESISTANCE-001

    let debug_logs = false;
    // Test status: almost always fails.
    let random_data_len = 50;
    assert!(
        !send_random_data_to_the_node_pre_handshake(random_data_len, debug_logs).await,
        "the node shouldn't keep the connection alive after sending random data"
    );
}

#[tokio::test]
#[allow(non_snake_case)]
async fn r001_t4_NO_HANDSHAKE_send_random_data_but_tiny_amount() {
    // ZG-RESISTANCE-001

    let debug_logs = false;
    // Test status: almost always fails.
    let random_data_len = 5;
    assert!(
        !send_random_data_to_the_node_pre_handshake(random_data_len, debug_logs).await,
        "the node shouldn't keep the connection alive after sending random data"
    );
}
