use tempfile::TempDir;
use tokio::time::{timeout, Duration};

use crate::{
    protocol::codecs::payload::{Payload, PingData},
    setup::node::Node,
    tools::synthetic_node::SyntheticNodeBuilder,
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c009_t1_PING_PING_REPLY_send_req_expect_reply() {
    // ZG-CONFORMANCE-009

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

    // Send a Ping with rand_bytes data.
    let nonce: [u8; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    let message = Payload::Ping(PingData { nonce });
    assert!(synthetic_node.unicast(net_addr, message).is_ok());

    // Expect a PingReply response with the same data.
    let check =
        |m: &Payload| matches!(&m, Payload::PingReply(PingData{nonce: data}) if *data == nonce);
    assert!(
        synthetic_node.expect_message(&check).await,
        "the PingReply response is missing"
    );

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("unable to stop the node");
}

#[tokio::test]
#[allow(non_snake_case)]
#[ignore]
async fn c009_t2_PING_PING_REPLY_wait_for_a_ping_req() {
    // ZG-CONFORMANCE-009

    crate::tools::synthetic_node::enable_tracing();
    // Spin up a node instance.
    let target = TempDir::new().expect("couldn't create a temporary directory");
    let mut node = Node::builder()
        .log_to_stdout(true)
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

    // TODO: reminder: uncomment or delete eventually.
    //// Expect a Ping request.
    //let check = |m: &Payload| matches!(&m, Payload::Ping(_));
    //assert!(
    //    synthetic_node.expect_message(&check).await,
    //    "the Ping request is missing"
    //);

    // Alternative approach at the moment: wait for any non-broadcast message:
    // Filter out the MsgOfInterest message.
    let check = |m: &Payload| matches!(&m, Payload::MsgOfInterest(..));
    assert!(synthetic_node.expect_message(&check).await);
    // Wait for at least 10 minutes.
    assert!(timeout(Duration::from_secs(610), async {
        loop {
            match synthetic_node.recv_message().await {
                (_, Payload::AgreementVote(_) | Payload::ProposalPayload(_)) => continue,
                msg => {
                    tracing::info!("Received a message: {:?}", msg);
                    return true;
                }
            }
        }
    })
    .await
    .is_ok());

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect("unable to stop the node");
}
