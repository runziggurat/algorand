use tempfile::TempDir;
use tokio::time::{sleep, timeout, Duration};
use ziggurat_core_utils::err_constants::{
    ERR_KMD_BUILD, ERR_KMD_STOP, ERR_NODE_ADDR, ERR_NODE_BUILD, ERR_NODE_STOP, ERR_SYNTH_UNICAST,
    ERR_TEMPDIR_NEW,
};

use crate::{
    protocol::codecs::{
        algomsg::AlgoMsg,
        msgpack::{Payment, Transaction, TransactionType},
        payload::Payload,
        tagmsg::Tag,
    },
    setup::{kmd::Kmd, node::Node},
    tests::conformance::post_handshake::cmd::{
        get_handshaked_synth_node, get_pub_key_addr, get_signed_tagged_txn, get_txn_params,
        get_wallet_token,
    },
    tools::constants::EXPECT_MSG_TIMEOUT,
};

// Generates a valid proposal payload message which contains a massive amount of transactions.
pub async fn get_huge_proposal_payload() -> AlgoMsg {
    // Spin up a node instance.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder().build(target.path()).expect(ERR_NODE_BUILD);
    node.start().await;

    let mut kmd = Kmd::builder()
        .build(target.path())
        .await
        .expect(ERR_KMD_BUILD);
    kmd.start().await;

    let wallet_token = get_wallet_token(&mut kmd).await;
    let txn_params = get_txn_params(&mut node).await;

    let rx_addr = get_pub_key_addr(&mut kmd, wallet_token.clone()).await;
    let tx_addr = rx_addr;

    const TXN_CNT: u64 = 1000;
    let mut txns = Vec::with_capacity(TXN_CNT as usize);

    for i in 0..TXN_CNT {
        let txn_type = TransactionType::Payment(Payment {
            receiver: rx_addr,
            amount: 1000 + i,
            close_remainder_to: None,
        });

        // Create a huge transaction - use a maximum note length.
        let txn = Transaction {
            sender: tx_addr,
            fee: txn_params.min_fee,
            first_valid: txn_params.last_round,
            last_valid: txn_params.last_round + 1000,
            note: vec![b'y'; 1024],
            genesis_id: txn_params.genesis_id.clone(),
            genesis_hash: txn_params.genesis_hash,
            group: None,
            lease: None,
            txn_type,
            rekey_to: None,
        };

        txns.push(Payload::RawBytes(
            get_signed_tagged_txn(&mut kmd, wallet_token.clone(), &txn).await,
        ));
    }

    // Transactions are prepared, shut down the kmd instance.
    kmd.stop().expect(ERR_KMD_STOP);

    // Create a synthetic node.
    let net_addr = node.net_addr().expect(ERR_NODE_ADDR);
    let mut synthetic_node = get_handshaked_synth_node(net_addr).await;

    // Dump all transactions to the node which will end up in the next ProposalPayload message.
    for txn in txns {
        if synthetic_node.unicast(net_addr, txn).is_err() {
            // Sometimes the synthetic_node cannot process sending so much data at once, so
            // a small sleep helps here.
            sleep(Duration::from_millis(10)).await;
        }
    }

    let proposal_payload_msg = timeout(EXPECT_MSG_TIMEOUT, async {
        // Proposal payload message size - empirical value.
        const PP_MSG_LEN: usize = 1000000;

        loop {
            let m = synthetic_node.recv_message().await.1;
            if matches!(&m, AlgoMsg { payload: Payload::ProposalPayload(_), raw } if raw.len() > PP_MSG_LEN) {
                return m
            }
        }
    }).await.expect("couldn't receive ProposalPayload");

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);

    proposal_payload_msg
}

#[tokio::test]
#[allow(non_snake_case)]
async fn r004_t1_PROPOPSAL_PAYLOAD_send_a_huge_valid_msg() {
    // ZG-RESISTANCE-004
    //
    // Send a huge valid proposal payload message to the node and expect to remain connected.

    // Get a huge proposal payload message from the dead node.
    let pp_msg = get_huge_proposal_payload().await;

    // Spin up a node instance.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder().build(target.path()).expect(ERR_NODE_BUILD);
    node.start().await;

    // Create a synthetic node.
    let net_addr = node.net_addr().expect(ERR_NODE_ADDR);
    let mut synthetic_node = get_handshaked_synth_node(net_addr).await;

    // Wait for at least one ProposalPayload message.
    let check_pp_msg = |m: &Payload| matches!(&m, Payload::ProposalPayload(_));
    assert!(synthetic_node.expect_message(&check_pp_msg, None).await);

    // Send a massive ProposalPayload message recorded previously.
    let msg = Payload::RawBytes(pp_msg.raw);
    assert!(synthetic_node.unicast(net_addr, msg.clone()).is_ok());

    // Clear the inbound queue.
    while synthetic_node
        .recv_message_timeout(Duration::from_millis(10))
        .await
        .is_ok()
    {}

    // Check that we are still receiving ProposalPayload messages.
    assert!(
        synthetic_node.expect_message(&check_pp_msg, None).await,
        "a proposal payload message is missing"
    );

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}

#[tokio::test]
#[allow(non_snake_case)]
async fn r004_t2_MSG_DIGEST_SKIP_send_a_huge_invalid_msg() {
    // ZG-RESISTANCE-004
    //
    // Send a huge invalid message to the node and expect to lose connection.

    // MsgDigestSkip imitation - expected message length is tag (2 bytes) + hash (32 bytes) = 34 bytes.
    let mut msg = Tag::get_tag_str(&Tag::MsgDigestSkip).as_bytes().to_vec();
    // Empirical value which the node won't reject.
    let simple_data = vec![b'y'; 6_250_000];
    msg.extend(simple_data);

    // Spin up a node instance.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder().build(target.path()).expect(ERR_NODE_BUILD);
    node.start().await;

    // Create a synthetic node.
    let net_addr = node.net_addr().expect(ERR_NODE_ADDR);
    let mut synthetic_node = get_handshaked_synth_node(net_addr).await;

    synthetic_node
        .unicast(net_addr, Payload::RawBytes(msg))
        .expect(ERR_SYNTH_UNICAST);

    // Clear the inbound queue.
    while synthetic_node
        .recv_message_timeout(Duration::from_millis(50))
        .await
        .is_ok()
    {}

    // Check that we are still receiving messages.
    assert!(
        !synthetic_node
            .expect_message(&|m: &Payload| matches!(&m, _), None)
            .await,
        "the connection is still established"
    );

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);
}
