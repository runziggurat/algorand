use std::time::Duration;

use tempfile::TempDir;

use crate::{
    protocol::codecs::{
        msgpack::{Payment, Transaction, TransactionType},
        payload::Payload,
    },
    setup::{kmd::Kmd, node::Node},
    tests::conformance::post_handshake::cmd::{
        get_handshaked_synth_node, get_pub_key_addr, get_signed_tagged_txn, get_txn_params,
        get_wallet_token,
    },
    tools::constants::{
        ERR_KMD_BUILD, ERR_KMD_STOP, ERR_NODE_ADDR, ERR_NODE_BUILD, ERR_NODE_STOP, ERR_TEMPDIR_NEW,
    },
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c012_TXN_submit_txn_and_expect_to_receive_it() {
    // ZG-CONFORMANCE-012

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

    // Just send payment to the same address - good enough for the test.
    let rx_addr = get_pub_key_addr(&mut kmd, wallet_token.clone()).await;
    let tx_addr = rx_addr;

    let txn_type = TransactionType::Payment(Payment {
        receiver: rx_addr,
        amount: 1000,
        close_remainder_to: None,
    });

    let txn_params = get_txn_params(&mut node).await;

    let txn = Transaction {
        sender: tx_addr,
        fee: txn_params.min_fee,
        first_valid: txn_params.last_round,
        last_valid: txn_params.last_round + 1000,
        note: Vec::new(),
        genesis_id: txn_params.genesis_id,
        genesis_hash: txn_params.genesis_hash,
        group: None,
        lease: None,
        txn_type,
        rekey_to: None,
    };

    let signed_tagged_txn = get_signed_tagged_txn(&mut kmd, wallet_token, &txn).await;

    let net_addr = node.net_addr().expect(ERR_NODE_ADDR);

    // Create synthetic nodes.
    let synthetic_node_tx = get_handshaked_synth_node(net_addr).await;
    let mut synthetic_node_rx = get_handshaked_synth_node(net_addr).await;

    // Send a signed transaction.
    let signed_tagged_txn = Payload::RawBytes(signed_tagged_txn);
    assert!(synthetic_node_tx
        .unicast(net_addr, signed_tagged_txn)
        .is_ok());

    let check = |m: &Payload| matches!(&m, Payload::Transaction(_));
    assert!(
        synthetic_node_rx
            .expect_message(&check, Some(Duration::from_secs(3)))
            .await,
        "a broadcasted transaction is missing"
    );

    // Gracefully shut down the nodes.
    synthetic_node_rx.shut_down().await;
    synthetic_node_tx.shut_down().await;
    kmd.stop().expect(ERR_KMD_STOP);
    node.stop().expect(ERR_NODE_STOP);
}
