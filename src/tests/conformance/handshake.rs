use tempfile::TempDir;

use crate::{setup::node::Node, tools::synthetic_node::SyntheticNodeBuilder};

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

    // Connect to the node and initiate the handshake.
    synthetic_node
        .connect(node.net_addr().unwrap())
        .await
        .unwrap();

    // This is only set post-handshake (if enabled).
    assert!(synthetic_node.is_connected(node.net_addr().unwrap()));

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().unwrap();
}
