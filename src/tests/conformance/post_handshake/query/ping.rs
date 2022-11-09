use tempfile::TempDir;

use crate::{
    protocol::codecs::payload::{Payload, PingData},
    setup::node::Node,
    tools::synthetic_node::SyntheticNodeBuilder,
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c009_PING_PING_REPLY_send_req_expect_reply() {
    // ZG-CONFORMANCE-009

    // Spin up a node instance.
    let target = TempDir::new().expect("couldn't create a temporary directory");
    let mut node = Node::builder()
        .build(target.path())
        .expect("unable to build the node");
    node.start().await;

    // Create a synthetic node and enable handshaking.
    let mut synthetic_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect("unable to build a synthetic node");

    let net_addr = node.net_addr().expect("network address not found");

    // Connect to the node and initiate the handshake.
    synthetic_node
        .connect(net_addr)
        .await
        .expect("unable to connect");

    // Send a Ping with rand_bytes data.
    let nonce: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    let message = Payload::Ping(PingData { nonce });
    assert!(synthetic_node.unicast(net_addr, message).is_ok());

    // Expect a PingReply response with the same data.
    let check =
        |m: &Payload| matches!(&m, Payload::PingReply(PingData{nonce: data}) if *data == nonce);
    assert!(
        synthetic_node.expect_message(&check).await,
        "the PingReply response is missing"
    );

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("unable to stop the node");
}
