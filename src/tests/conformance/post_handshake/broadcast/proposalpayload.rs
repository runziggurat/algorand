use tempfile::TempDir;

use crate::{
    protocol::codecs::payload::Payload, setup::node::Node,
    tools::synthetic_node::SyntheticNodeBuilder,
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c007_PROPOSAL_PAYLOAD_expect_after_connect() {
    // ZG-CONFORMANCE-007

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

    let check = |m: &Payload| matches!(&m, Payload::ProposalPayload(..));

    // Wait for two messages at least.
    for _ in 0..2 {
        assert!(synthetic_node.expect_message(&check, None).await);
    }

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("unable to stop the node");
}
