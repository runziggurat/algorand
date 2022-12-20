use data_encoding::BASE64;
use tempfile::TempDir;

use crate::{
    protocol::{
        codecs::{
            msgpack::{NetPrioResponse, Response},
            payload::Payload,
        },
        handshake::HandshakeCfg,
    },
    setup::node::Node,
    tools::synthetic_node::SyntheticNodeBuilder,
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c011_t1_NET_PRIO_RESPONSE_expect_rsp_from_the_node() {
    // ZG-CONFORMANCE-011

    // A simple non-random challenge.
    let challenge = BASE64.encode(&[1u8; 32]);

    let cfg = HandshakeCfg {
        challenge: Some(challenge.clone()),
        ..Default::default()
    };

    // Create a synthetic node and enable handshaking.
    let mut synthetic_node = SyntheticNodeBuilder::default()
        .with_handshake_configuration(cfg)
        .build()
        .await
        .expect("unable to build a synthetic node");

    let listening_addr = synthetic_node
        .start_listening()
        .await
        .expect("a synthetic node couldn't start listening");

    // Spin up a node instance.
    let target = TempDir::new().expect("couldn't create a temporary directory");
    let mut node = Node::builder()
        .initial_peers([listening_addr])
        .build(target.path())
        .expect("unable to build the node");
    node.start().await;

    let check = |m: &Payload| {
        matches!(&m, Payload::NetPrioResponse(NetPrioResponse{response: Response { nonce }, ..})
                 if *nonce == challenge)
    };
    assert!(synthetic_node.expect_message(&check).await);

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("unable to stop the node");
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c011_t2_NET_PRIO_RESPONSE_no_rsp_if_challenge_not_sent() {
    // ZG-CONFORMANCE-011
    //
    // Do not send a challenge when answering a handshake request.
    // A NetPrioResponse message should not be received in that case.

    // Create a synthetic node and enable handshaking.
    let mut synthetic_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect("unable to build a synthetic node");

    let listening_addr = synthetic_node
        .start_listening()
        .await
        .expect("a synthetic node couldn't start listening");

    // Spin up a node instance.
    let target = TempDir::new().expect("couldn't create a temporary directory");
    let mut node = Node::builder()
        .initial_peers([listening_addr])
        .build(target.path())
        .expect("unable to build the node");
    node.start().await;

    let check = |m: &Payload| matches!(&m, Payload::NetPrioResponse(..));
    assert!(!synthetic_node.expect_message(&check).await);

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("unable to stop the node");
}
