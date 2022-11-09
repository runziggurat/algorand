use std::{collections::HashSet, io};

use tempfile::TempDir;
use tokio::time::Duration;

use crate::{
    protocol::codecs::{payload::Payload, tagmsg::Tag, topic::MsgOfInterest},
    setup::node::Node,
    tools::synthetic_node::SyntheticNodeBuilder,
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c005_t1_MSG_OF_INTEREST_expect_after_connect() {
    // ZG-CONFORMANCE-005

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

    let check = |m: &Payload| matches!(&m, Payload::MsgOfInterest(..));
    assert!(synthetic_node.expect_message(&check).await);

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("unable to stop the node");
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c005_t2_MSG_OF_INTEREST_send_after_connect() {
    // ZG-CONFORMANCE-005

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
    assert!(synthetic_node.expect_message(&expect_any_msg).await);

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("unable to stop the node");
}

#[tokio::test]
#[allow(non_snake_case)]
async fn c006_MSG_OF_INTEREST_expect_no_messages_after_sending_empty_tag_list() {
    // ZG-CONFORMANCE-006

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

    let check = |m: &Payload| matches!(&m, Payload::MsgOfInterest(..));
    assert!(synthetic_node.expect_message(&check).await);

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
    match synthetic_node
        .recv_message_timeout(Duration::from_secs(5)) // Usually, it broadcasts messages every few seconds.
        .await
    {
        Err(e) if e.kind() == io::ErrorKind::TimedOut => (),
        _ => panic!("no messages expected after enabling filter on all messages"),
    }

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("unable to stop the node");
}
