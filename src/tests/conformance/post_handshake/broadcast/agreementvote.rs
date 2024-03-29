use tempfile::TempDir;
use ziggurat_core_utils::err_constants::{
    ERR_NODE_ADDR, ERR_NODE_BUILD, ERR_NODE_STOP, ERR_SYNTH_BUILD, ERR_SYNTH_CONNECT,
    ERR_TEMPDIR_NEW,
};

use crate::{
    protocol::codecs::payload::Payload, setup::node::Node,
    tools::synthetic_node::SyntheticNodeBuilder,
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c008_AGREEMENT_VOTE_expect_after_connect() {
    // ZG-CONFORMANCE-008

    // Spin up a node instance.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder().build(target.path()).expect(ERR_NODE_BUILD);
    node.start().await;

    // Create a synthetic node and enable handshaking.
    let mut synthetic_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect(ERR_SYNTH_BUILD);

    let net_addr = node.net_addr().expect(ERR_NODE_ADDR);

    // Connect to the node and initiate the handshake.
    synthetic_node
        .connect(net_addr)
        .await
        .expect(ERR_SYNTH_CONNECT);

    let check = |m: &Payload| matches!(&m, Payload::AgreementVote(..));

    // Wait for two messages at least.
    for _ in 0..2 {
        assert!(synthetic_node.expect_message(&check, None).await);
    }

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}
