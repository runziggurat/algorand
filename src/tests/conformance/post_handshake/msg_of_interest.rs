use std::collections::HashSet;

use tempfile::TempDir;
use tokio::time::Duration;

use crate::{
    protocol::codecs::{payload::Payload, tagmsg::Tag, topic::MsgOfInterest},
    setup::node::Node,
    tools::{
        constants::{
            ERR_NODE_ADDR, ERR_NODE_BUILD, ERR_NODE_STOP, ERR_SYNTH_BUILD, ERR_SYNTH_CONNECT,
            ERR_TEMPDIR_NEW,
        },
        synthetic_node::SyntheticNodeBuilder,
    },
};

// All MsgOfInterest messages should be received immediately after the connetion is established.
const MSG_TIMEOUT: Option<Duration> = Some(Duration::from_secs(3));

#[tokio::test]
#[allow(non_snake_case)]
async fn c005_t1_MSG_OF_INTEREST_expect_after_connect() {
    // ZG-CONFORMANCE-005

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

    let check = |m: &Payload| matches!(&m, Payload::MsgOfInterest(..));
    assert!(synthetic_node.expect_message(&check, MSG_TIMEOUT).await);

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c005_t2_MSG_OF_INTEREST_send_after_connect() {
    // ZG-CONFORMANCE-005

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

    // Send a MsgOfInterest message with all expected tags included.
    let tags = HashSet::from([
        Tag::ProposalPayload,
        Tag::AgreementVote,
        Tag::MsgOfInterest,
        Tag::MsgDigestSkip,
        Tag::NetPrioResponse,
        Tag::Ping,
        Tag::PingReply,
        Tag::ProposalPayload,
        Tag::StateProofSig,
        Tag::TopicMsgResp,
        Tag::Txn,
        Tag::UniEnsBlockReq,
        Tag::VoteBundle,
    ]);
    let message = Payload::MsgOfInterest(MsgOfInterest { tags });
    assert!(synthetic_node.unicast(net_addr, message).is_ok());

    // Clear any remaining received messages in the inbound queue
    // before the node processes our MsgOfInterest message.
    while synthetic_node
        .recv_message_timeout(Duration::from_millis(10))
        .await
        .is_ok()
    {}

    // Wait for any message from the 'tags' list above.
    let expect_any_msg = |_: &Payload| true;
    assert!(synthetic_node.expect_message(&expect_any_msg, None).await);

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c006_MSG_OF_INTEREST_expect_no_messages_after_sending_empty_tag_list() {
    // ZG-CONFORMANCE-006

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

    let check = |m: &Payload| matches!(&m, Payload::MsgOfInterest(..));
    assert!(synthetic_node.expect_message(&check, MSG_TIMEOUT).await);

    // Send a MsgOfInterest message with no tags enabled.
    let no_tags = HashSet::new();
    let message = Payload::MsgOfInterest(MsgOfInterest { tags: no_tags });
    assert!(synthetic_node.unicast(net_addr, message).is_ok());

    // Clear any remaining received messages in the inbound queue
    // before the node processes our MsgOfInterest message.
    while synthetic_node
        .recv_message_timeout(Duration::from_millis(10))
        .await
        .is_ok()
    {}

    // Verify the node won't send us any messages afterwards.
    let expect_any_msg = |_: &Payload| true;
    let duration = Some(Duration::from_secs(5)); // Usually, it broadcasts messages every few seconds.
    assert!(
        !synthetic_node
            .expect_message(&expect_any_msg, duration)
            .await
    );

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}
