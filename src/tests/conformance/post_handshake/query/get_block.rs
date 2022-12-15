use tempfile::TempDir;

use crate::{
    protocol::codecs::{
        payload::Payload,
        topic::{TopicMsgResp, UniCatchupReq, UniEnsBlockReq, UniEnsBlockReqType},
    },
    setup::node::Node,
    tools::synthetic_node::SyntheticNodeBuilder,
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c004_V1_BLOCK_ROUND_get_block() {
    // ZG-CONFORMANCE-004

    // Spin up a node instance.
    let target = TempDir::new().expect("couldn't create a temporary directory");
    let mut node = Node::builder()
        .build(target.path())
        .expect("unable to build the node");
    node.start().await;

    // Create a synthetic node and enable handshaking.
    let synthetic_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect("unable to build a synthetic node");

    let net_addr = node.net_addr().expect("network address not found");

    // Connect to the node and initiate the handshake.
    synthetic_node
        .connect(net_addr)
        .await
        .expect("unable to connect");

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
    node.stop().expect("unable to stop the node");
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c010_t1_UNI_ENS_BLOCK_REQ_get_block_and_cert() {
    // ZG-CONFORMANCE-010

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
            synthetic_node.expect_message(&check).await,
            "the UniEnsBlockRsp response is missing"
        );
    }

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("unable to stop the node");
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c010_t2_UNI_ENS_BLOCK_REQ_get_block_only() {
    // ZG-CONFORMANCE-010

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
            synthetic_node.expect_message(&check).await,
            "the UniEnsBlockRsp response is missing"
        );
    }

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("unable to stop the node");
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c010_t3_UNI_ENS_BLOCK_REQ_get_cert_only() {
    // ZG-CONFORMANCE-010

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
            synthetic_node.expect_message(&check).await,
            "the UniEnsBlockRsp response is missing"
        );
    }

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("unable to stop the node");
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c010_t4_UNI_ENS_BLOCK_REQ_cannot_get_non_existent_block() {
    // ZG-CONFORMANCE-010

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
        synthetic_node.expect_message(&check).await,
        "the UniEnsBlockRsp response is missing"
    );

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("unable to stop the node");
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c010_t5_UNI_CATCHUP_REQ_get_block_all_variations() {
    // ZG-CONFORMANCE-010
    //
    // This test is based on t1-t4 tests, since UniCatchupReq behaves exactly the same as the
    // UniEnsBlockReq message. So, all subtests for this message are grouped together here.

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

    let round = 1; // A random value from the 0..4 range.

    // Get a block and certificate (as in the t1 test).
    {
        let message = Payload::UniCatchupReq(UniCatchupReq {
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
            synthetic_node.expect_message(&check).await,
            "the UniEnsBlockRsp response is missing"
        );
    }

    // Get only a block (as in the t2 test).
    {
        let message = Payload::UniCatchupReq(UniCatchupReq {
            data_type: UniEnsBlockReqType::Block,
            round_key: round,
            nonce: round,
        });
        assert!(synthetic_node.unicast(net_addr, message).is_ok());
        // TODO: Still unsupported, check with the Algorand team.
        // Alternative check to ensure it's unsupported :-)
        let check = |m: &Payload| {
            matches!(&m, Payload::TopicMsgResp(TopicMsgResp::ErrorRsp(rsp))
                     if rsp.error.as_str() == "requested data type is unsupported")
        };
        assert!(
            synthetic_node.expect_message(&check).await,
            "the UniEnsBlockRsp response is missing"
        );
    }

    // Get only a certificate (as in the t3 test).
    {
        let message = Payload::UniCatchupReq(UniCatchupReq {
            data_type: UniEnsBlockReqType::Cert,
            round_key: round,
            nonce: round,
        });
        assert!(synthetic_node.unicast(net_addr, message).is_ok());
        // TODO: Still unsupported, check with the Algorand team.
        // Alternative check to ensure it's unsupported :-)
        let check = |m: &Payload| {
            matches!(&m, Payload::TopicMsgResp(TopicMsgResp::ErrorRsp(rsp))
                     if rsp.error.as_str() == "requested data type is unsupported")
        };
        assert!(
            synthetic_node.expect_message(&check).await,
            "the UniEnsBlockRsp response is missing"
        );
    }

    // Ask for a non-existent block and get a valid response (as in the t4 test).
    {
        let message = Payload::UniCatchupReq(UniCatchupReq {
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
            synthetic_node.expect_message(&check).await,
            "the UniEnsBlockRsp response is missing"
        );
    }

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("unable to stop the node");
}
