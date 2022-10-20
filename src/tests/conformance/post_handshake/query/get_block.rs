use tempfile::TempDir;

use crate::{
    setup::node::Node,
    tools::{rpc, synthetic_node::SyntheticNodeBuilder},
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c003_V1_BLOCK_ROUND_get_block() {
    // ZG-CONFORMANCE-003

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

    let rpc_addr = net_addr.to_string();

    // Get block for the round 0.
    let round = 0;
    let block_cert = rpc::wait_for_block(&rpc_addr, round)
        .await
        .expect("Couldn't get a block");
    assert_eq!(round, block_cert.block.round, "Invalid round");
    assert!(block_cert.block.sortition_seed.is_some(), "Seed not found");
    assert!(
        block_cert.block.genesis_id_hash.is_some(),
        "Genesis hash not found"
    );
    assert!(
        block_cert.block.prevous_block_hash.is_none(),
        "Previous block hash shouldn't be found for the first block"
    );

    // Get block for the round 1.
    let round = 1;
    let block_cert = rpc::wait_for_block(&rpc_addr, round)
        .await
        .expect("Couldn't get a block");
    assert_eq!(round, block_cert.block.round, "Invalid round");
    assert!(block_cert.block.sortition_seed.is_some(), "Seed not found");
    assert!(
        block_cert.block.genesis_id_hash.is_some(),
        "Genesis hash not found"
    );
    assert!(
        block_cert.block.prevous_block_hash.is_some(),
        "Previous block hash not found"
    );

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("Unable to stop the node");
}
