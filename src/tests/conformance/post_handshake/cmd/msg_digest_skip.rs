use tempfile::TempDir;
use tokio::time::timeout;

use crate::{
    protocol::codecs::{algomsg::AlgoMsg, msgpack::HashDigest, payload::Payload},
    setup::node::Node,
    tests::{
        conformance::post_handshake::cmd::get_handshaked_synth_node,
        resistance::post_handshake::enormous_message::get_huge_proposal_payload,
    },
    tools::constants::{
        ERR_NODE_ADDR, ERR_NODE_BUILD, ERR_NODE_STOP, ERR_TEMPDIR_NEW, EXPECT_MSG_TIMEOUT,
    },
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c013_t1_MSG_DIGEST_SKIP_receive_a_msg() {
    // ZG-CONFORMANCE-013
    //
    // Send a huge valid proposal payload message to the node from one synthetic node
    // and expect to receive a MsgDigestSkip message on the other synthetic node.

    // Get a huge proposal payload message from the dead node.
    let tx_pp_msg = get_huge_proposal_payload().await;
    let tx_msg_hash = HashDigest::from(&tx_pp_msg.raw);

    // Spin up a node instance.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder().build(target.path()).expect(ERR_NODE_BUILD);
    node.start().await;

    // Create synthetic nodes.
    let net_addr = node.net_addr().expect(ERR_NODE_ADDR);
    let mut synthetic_node_rx = get_handshaked_synth_node(net_addr).await;
    let synthetic_node_tx = get_handshaked_synth_node(net_addr).await;

    // Send a massive ProposalPayload message recorded previously.
    let msg = Payload::RawBytes(tx_pp_msg.raw);
    assert!(synthetic_node_tx.unicast(net_addr, msg.clone()).is_ok());

    // For messages bigger than 5000 bytes, the node broadcasts a filter message (MsgDigestSkip) to everyone else.
    let rx_msg_hash = timeout(EXPECT_MSG_TIMEOUT, async {
        loop {
            if let AlgoMsg {
                payload: Payload::MsgDigestSkip(hash),
                ..
            } = synthetic_node_rx.recv_message().await.1
            {
                return hash;
            }
        }
    })
    .await
    .expect("couldn't receive a MsgDigestSkip message");

    // Pre-calculated tx message hash should be the same as the rx message hash received in the MsgDigestSkip message.
    assert_eq!(
        rx_msg_hash, tx_msg_hash,
        "received MsgDigestSkip hash is invalid"
    );

    // Gracefully shut down the nodes.
    synthetic_node_rx.shut_down().await;
    synthetic_node_tx.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}
