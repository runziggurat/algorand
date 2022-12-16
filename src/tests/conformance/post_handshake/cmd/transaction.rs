use std::net::SocketAddr;

use tempfile::TempDir;

use crate::{
    protocol::codecs::{
        msgpack::{Address, Payment, Transaction, TransactionType},
        payload::Payload,
        tagmsg::Tag,
    },
    setup::{kmd::Kmd, node::Node},
    tools::synthetic_node::{SyntheticNode, SyntheticNodeBuilder},
};

#[tokio::test]
#[allow(non_snake_case)]
async fn c012_TXN_submit_txn_and_expect_to_receive_it() {
    // ZG-CONFORMANCE-012

    // Spin up a node instance.
    let target = TempDir::new().expect("couldn't create a temporary directory");
    let mut node = Node::builder()
        .build(target.path())
        .expect("unable to build the node");
    node.start().await;

    let mut kmd = Kmd::builder()
        .build(target.path())
        .await
        .expect("unable to build the kmd instance");
    kmd.start().await;

    // TODO(Rqnsom): Move transaction creation to a function for the next test.
    let wallets = kmd.get_wallets().await.expect("couldn't get the wallets");
    let wallet_id = wallets
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "unencrypted-default-wallet")
        .expect("couldn't find an unencrypted default wallet")
        .id;

    let wallet_token = kmd
        .get_wallet_handle_token(wallet_id, "".to_string())
        .await
        .expect("couldn't get the wallet token")
        .wallet_handle_token;

    let txn_params = node
        .rest_client()
        .expect("couldn't get the REST client")
        .get_transaction_params()
        .await
        .expect("couldn't get the transaction parameters");

    let pub_key = kmd
        .get_keys(wallet_token.clone())
        .await
        .expect("couldn't get the wallet keys")
        .addresses
        .pop()
        .expect("couldn't find any public keys in the wallet");

    // Just send payment to the same address - good enough for the test.
    let rx_addr = Address::from_string(&pub_key).expect("couldn't convert pub key to address");
    let tx_addr = rx_addr;

    let txn_type = TransactionType::Payment(Payment {
        receiver: rx_addr,
        amount: 1000,
        close_remainder_to: None,
    });

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

    let mut signed_txn = kmd
        .sign_transaction(wallet_token, "".to_string(), &txn)
        .await
        .expect("couldn't sign the transaction")
        .signed_transaction;
    let mut tagged_msg = Tag::get_tag_str(&Tag::Txn).as_bytes().to_vec();
    tagged_msg.append(&mut signed_txn);

    let net_addr = node.net_addr().expect("network address not found");

    let synthetic_node_tx = get_handshaked_synth_node(net_addr).await;
    let mut synthetic_node_rx = get_handshaked_synth_node(net_addr).await;

    // Send a signed transaction.
    let signed_tagged_txn = Payload::RawBytes(tagged_msg);
    assert!(synthetic_node_tx
        .unicast(net_addr, signed_tagged_txn)
        .is_ok());

    let check = |m: &Payload| matches!(&m, Payload::Transaction(_));
    assert!(
        synthetic_node_rx.expect_message(&check).await,
        "a broadcasted transaction is missing"
    );

    // Gracefully shut down the nodes.
    synthetic_node_rx.shut_down().await;
    synthetic_node_tx.shut_down().await;
    kmd.stop().expect("unable to stop the kmd instance");
    node.stop().expect("unable to stop the node");
}

async fn get_handshaked_synth_node(net_addr: SocketAddr) -> SyntheticNode {
    // Create a synthetic node and enable handshaking.
    let synthetic_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect("unable to build a synthetic node");

    // Connect to the node and initiate the handshake.
    synthetic_node
        .connect(net_addr)
        .await
        .expect("unable to connect");

    synthetic_node
}
