use tempfile::TempDir;
use ziggurat_core_utils::err_constants::{
    ERR_NODE_ADDR, ERR_NODE_BUILD, ERR_NODE_STOP, ERR_SYNTH_BUILD, ERR_SYNTH_CONNECT,
    ERR_TEMPDIR_NEW,
};

use crate::{
    protocol::codecs::{
        payload::Payload,
        topic::{TopicMsgResp, UniEnsBlockReq, UniEnsBlockReqType},
    },
    setup::node::Node,
    tools::synthetic_node::SyntheticNodeBuilder,
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c004_V1_BLOCK_ROUND_get_block() {
    // ZG-CONFORMANCE-004

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

    let rest_client = node.rest_client().expect("couldn't get the rest client");

    for round in 0..4 {
        let block_cert = rest_client
            .wait_for_block(round)
            .await
            .expect("couldn't get a block");

        assert_eq!(round, block_cert.block.round, "invalid round");
        assert!(block_cert.block.sortition_seed.is_some(), "seed not found");
        assert!(
            block_cert.block.genesis_id_hash.is_some(),
            "genesis hash not found"
        );

        if round == 0 {
            assert!(
                block_cert.block.prevous_block_hash.is_none(),
                "previous block hash shouldn't be found for the first round"
            );
        } else {
            assert!(
                block_cert.block.prevous_block_hash.is_some(),
                "previous block hash not found"
            );
        }
    }

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c010_t1_UNI_ENS_BLOCK_REQ_get_block_and_cert() {
    // ZG-CONFORMANCE-010

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

    for round in 0..4 {
        let message = Payload::UniEnsBlockReq(UniEnsBlockReq {
            data_type: UniEnsBlockReqType::BlockAndCert,
            round_key: round,
            nonce: round,
        });
        assert!(synthetic_node.unicast(net_addr, message).is_ok());

        // Expect a UniEnsBlockRsp response with a block with the same round and also a certificate.
        let check = |m: &Payload| {
            matches!(&m, Payload::TopicMsgResp(TopicMsgResp::UniEnsBlockRsp(rsp))
                     if rsp.block.is_some() && rsp.block.as_ref().unwrap().round == round && rsp.cert.is_some())
        };
        assert!(
            synthetic_node.expect_message(&check, None).await,
            "the UniEnsBlockRsp response is missing"
        );
    }

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c010_t2_UNI_ENS_BLOCK_REQ_get_block_only() {
    // ZG-CONFORMANCE-010

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

    for round in 0..4 {
        let message = Payload::UniEnsBlockReq(UniEnsBlockReq {
            data_type: UniEnsBlockReqType::Block,
            round_key: round,
            nonce: round,
        });
        assert!(synthetic_node.unicast(net_addr, message).is_ok());

        // TODO: Still unsupported, check with the Algorand team.
        //// Expect a UniEnsBlockRsp response with only a block with the same round, no certificate.
        //let check = |m: &Payload| {
        //    matches!(&m, Payload::TopicMsgResp(TopicMsgResp::UniEnsBlockRsp(rsp))
        //             if rsp.block.is_some() && rsp.block.as_ref().unwrap().round == round && rsp.cert.is_none())
        //};

        // Alternative check to ensure it's unsupported :-)
        let check = |m: &Payload| {
            matches!(&m, Payload::TopicMsgResp(TopicMsgResp::ErrorRsp(rsp))
                     if rsp.error.as_str() == "requested data type is unsupported")
        };
        assert!(
            synthetic_node.expect_message(&check, None).await,
            "the UniEnsBlockRsp response is missing"
        );
    }

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c010_t3_UNI_ENS_BLOCK_REQ_get_cert_only() {
    // ZG-CONFORMANCE-010

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

    for round in 0..4 {
        let message = Payload::UniEnsBlockReq(UniEnsBlockReq {
            data_type: UniEnsBlockReqType::Cert,
            round_key: round,
            nonce: round,
        });
        assert!(synthetic_node.unicast(net_addr, message).is_ok());

        // TODO: Still unsupported, check with the Algorand team.
        //// Expect a UniEnsBlockRsp response with only a certificate, no block.
        //let check = |m: &Payload| {
        //    matches!(&m, Payload::TopicMsgResp(TopicMsgResp::UniEnsBlockRsp(rsp))
        //             if rsp.block.is_none() && rsp.cert.is_some())
        //};

        // Alternative check to ensure it's unsupported :-)
        let check = |m: &Payload| {
            matches!(&m, Payload::TopicMsgResp(TopicMsgResp::ErrorRsp(rsp))
                     if rsp.error.as_str() == "requested data type is unsupported")
        };
        assert!(
            synthetic_node.expect_message(&check, None).await,
            "the UniEnsBlockRsp response is missing"
        );
    }

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c010_t4_UNI_ENS_BLOCK_REQ_cannot_get_non_existent_block() {
    // ZG-CONFORMANCE-010

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

    let message = Payload::UniEnsBlockReq(UniEnsBlockReq {
        data_type: UniEnsBlockReqType::BlockAndCert,
        round_key: 9999,
        nonce: 0,
    });
    assert!(synthetic_node.unicast(net_addr, message).is_ok());

    let check = |m: &Payload| {
        matches!(&m, Payload::TopicMsgResp(TopicMsgResp::ErrorRsp(rsp))
                 if rsp.error.as_str() == "requested block is not available")
    };
    assert!(
        synthetic_node.expect_message(&check, None).await,
        "the UniEnsBlockRsp response is missing"
    );

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}
