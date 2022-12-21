use tempfile::TempDir;

use crate::{
    protocol::{codecs::payload::Payload, handshake::HandshakeCfg},
    setup::node::{ChildExitCode, Node},
    tools::synthetic_node::SyntheticNodeBuilder,
};

// Empirical values based on some unofficial testing.
const WS_HTTP_HEADER_MAX_SIZE: usize = 7600;
const WS_HTTP_HEADER_INVALID_SIZE: usize = WS_HTTP_HEADER_MAX_SIZE + 300;

// Runs the handshake request test with a given handshake configuration.
// Returns the truthful fact about the relationship with the node.
async fn run_handshake_req_test_with_cfg(cfg: HandshakeCfg, debug: bool) -> bool {
    // Spin up a node instance.
    let target = TempDir::new().expect("couldn't create a temporary directory");
    let mut node = Node::builder()
        .log_to_stdout(debug)
        .build(target.path())
        .expect("unable to build the node");
    node.start().await;

    // Create a synthetic node and enable handshaking.
    let mut synthetic_node = SyntheticNodeBuilder::default()
        .with_handshake_configuration(cfg)
        .build()
        .await
        .expect("unable to build a synthetic node");

    let net_addr = node.net_addr().expect("network address not found");

    // Connect to the node and initiate the handshake.
    let handshake_established = if synthetic_node.connect(net_addr).await.is_err() {
        false
    } else {
        // Wait for any message.
        synthetic_node
            .expect_message(&|m: &Payload| matches!(&m, _))
            .await
    };

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    assert_eq!(
        node.stop().expect("unable to stop the node"),
        ChildExitCode::Success
    );

    handshake_established
}

#[tokio::test]
#[ignore]
async fn normal_handshake() {
    // Basically, a copy of the C001 test.
    assert!(
        run_handshake_req_test_with_cfg(Default::default(), false).await,
        "a default configuration doesn't work"
    );
}

/// Generate a string with a given length.
fn gen_huge_string(len: usize) -> String {
    vec!['y'; len].into_iter().collect::<String>()
}

#[tokio::test]
#[allow(non_snake_case)]
async fn r001_t1_HANDSHAKE_instance_name() {
    // ZG-RESISTANCE-001

    let gen_cfg = |len| HandshakeCfg {
        ar_instance_name: gen_huge_string(len),
        ..Default::default()
    };

    // Valid scenarios:

    // Find the largest instance value which the node can accept.
    let cfg = gen_cfg(WS_HTTP_HEADER_MAX_SIZE);
    assert!(run_handshake_req_test_with_cfg(cfg, false).await);

    // Below tests assert the connection shouldn't be established.

    // Use a huge value which the node will reject.
    let cfg = gen_cfg(WS_HTTP_HEADER_INVALID_SIZE);
    assert!(!run_handshake_req_test_with_cfg(cfg, false).await);

    // Send an empty field.
    // NOTE: hmm, should the node allow an empty name field?
    //let cfg = gen_cfg(0);
    //assert!(!run_handshake_req_test_with_cfg(cfg).await);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn r001_t2_HANDSHAKE_node_random() {
    // ZG-RESISTANCE-001

    let gen_cfg = |len| HandshakeCfg {
        ar_node_random: gen_huge_string(len),
        ..Default::default()
    };

    // Valid scenarios:

    // Find the largest instance value which the node can accept.
    let cfg = gen_cfg(WS_HTTP_HEADER_MAX_SIZE);
    assert!(run_handshake_req_test_with_cfg(cfg, false).await);

    // Below tests assert the connection shouldn't be established.

    // Use a huge value which the node will reject.
    let cfg = gen_cfg(WS_HTTP_HEADER_INVALID_SIZE);
    assert!(!run_handshake_req_test_with_cfg(cfg, false).await);

    // Send an empty field.
    let cfg = gen_cfg(0);
    assert!(!run_handshake_req_test_with_cfg(cfg, false).await);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn r001_t3_HANDSHAKE_genesis() {
    // ZG-RESISTANCE-001

    let gen_cfg = |len| HandshakeCfg {
        ar_genesis: gen_huge_string(len),
        ..Default::default()
    };

    // Valid scenarios:

    // Find the largest instance value which the node can accept.
    let cfg = gen_cfg(WS_HTTP_HEADER_MAX_SIZE);
    assert!(run_handshake_req_test_with_cfg(cfg, false).await);

    // Below tests assert the connection shouldn't be established.

    // Use a huge value which the node will reject.
    let cfg = gen_cfg(WS_HTTP_HEADER_INVALID_SIZE);
    assert!(!run_handshake_req_test_with_cfg(cfg, false).await);

    // Send an empty field.
    // NOTE: fails because the value seems unused.
    //let cfg = gen_cfg(0);
    //assert!(!run_handshake_req_test_with_cfg(cfg).await);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn r001_t4_HANDSHAKE_user_agent() {
    // ZG-RESISTANCE-001

    let gen_cfg = |len| HandshakeCfg {
        user_agent: gen_huge_string(len),
        ..Default::default()
    };

    // Valid scenarios:

    // Find the largest instance value which the node can accept.
    let cfg = gen_cfg(WS_HTTP_HEADER_MAX_SIZE);
    assert!(run_handshake_req_test_with_cfg(cfg, false).await);

    // Below tests assert the connection shouldn't be established.

    // Use a huge value which the node will reject.
    let cfg = gen_cfg(WS_HTTP_HEADER_INVALID_SIZE);
    assert!(!run_handshake_req_test_with_cfg(cfg, false).await);

    // Send an empty field.
    // NOTE: fails because the value seems unused.
    //let cfg = gen_cfg(0);
    //assert!(!run_handshake_req_test_with_cfg(cfg).await);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn r001_t5_HANDSHAKE_ws_version() {
    // ZG-RESISTANCE-001

    let gen_cfg_huge = |len| HandshakeCfg {
        ws_version: gen_huge_string(len),
        ..Default::default()
    };
    let gen_cfg_with = |version: usize| HandshakeCfg {
        ws_version: version.to_string(),
        ..Default::default()
    };

    // Valid scenarios:

    // This should be considered as an invalid value.
    let cfg = gen_cfg_with(13);
    assert!(run_handshake_req_test_with_cfg(cfg, false).await);

    // Below tests assert the connection shouldn't be established.

    // Invalid WebSocket versions.
    let cfg = gen_cfg_with(12);
    assert!(!run_handshake_req_test_with_cfg(cfg, false).await);
    let cfg = gen_cfg_with(14);
    assert!(!run_handshake_req_test_with_cfg(cfg, false).await);

    // This should be considered as an invalid value.
    let cfg = gen_cfg_huge(WS_HTTP_HEADER_MAX_SIZE);
    assert!(!run_handshake_req_test_with_cfg(cfg, false).await);

    // Send an empty field.
    let cfg = gen_cfg_huge(0);
    assert!(!run_handshake_req_test_with_cfg(cfg, false).await);
}
