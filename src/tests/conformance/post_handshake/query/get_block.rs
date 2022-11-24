use tempfile::TempDir;

use crate::{
    protocol::codecs::{
        payload::Payload,
        topic::{TopicMsgResp, UniEnsBlockReq, UniEnsBlockReqType},
    },
    setup::node::Node,
    tools::{rpc, synthetic_node::SyntheticNodeBuilder},
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

    let rpc_addr = net_addr.to_string();

    for round in 0..4 {
        let block_cert = rpc::wait_for_block(&rpc_addr, round)
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
async fn c010_UNI_ENS_BLOCK_REQ_get_block() {
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

        // Expect a UniEnsBlockRsp response with a block with the same round.
        let check = |m: &Payload| {
            matches!(&m, Payload::TopicMsgResp(TopicMsgResp::UniEnsBlockRsp(rsp))
                     if rsp.block.is_some() && rsp.block.as_ref().unwrap().round == round)
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
