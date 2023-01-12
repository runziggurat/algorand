use data_encoding::BASE64;
use tempfile::TempDir;
use tokio::time::Duration;
use ziggurat_core_utils::err_constants::{
    ERR_NODE_BUILD, ERR_NODE_STOP, ERR_SYNTH_BUILD, ERR_SYNTH_START_LISTENING, ERR_TEMPDIR_NEW,
};

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

const MSG_TIMEOUT: Option<Duration> = Some(Duration::from_secs(3));

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
        .expect(ERR_SYNTH_BUILD);

    let listening_addr = synthetic_node
        .start_listening()
        .await
        .expect(ERR_SYNTH_START_LISTENING);

    // Spin up a node instance.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder()
        .initial_peers([listening_addr])
        .build(target.path())
        .expect(ERR_NODE_BUILD);
    node.start().await;

    let check = |m: &Payload| {
        matches!(&m, Payload::NetPrioResponse(NetPrioResponse{response: Response { nonce }, ..})
                 if *nonce == challenge)
    };
    assert!(synthetic_node.expect_message(&check, MSG_TIMEOUT).await);

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
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
        .expect(ERR_SYNTH_BUILD);

    let listening_addr = synthetic_node
        .start_listening()
        .await
        .expect(ERR_SYNTH_START_LISTENING);

    // Spin up a node instance.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder()
        .initial_peers([listening_addr])
        .build(target.path())
        .expect(ERR_NODE_BUILD);
    node.start().await;

    let check = |m: &Payload| matches!(&m, Payload::NetPrioResponse(..));
    assert!(!synthetic_node.expect_message(&check, MSG_TIMEOUT).await);

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}
