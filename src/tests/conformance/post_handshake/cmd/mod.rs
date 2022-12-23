//! Test suite for command messages - which do not generate a response from the node.

mod msg_digest_skip;
mod transaction;

use std::net::SocketAddr;

use crate::{
    protocol::codecs::{
        msgpack::{Address, Transaction},
        tagmsg::Tag,
    },
    setup::{
        kmd::Kmd,
        node::{rest_api::message::TransactionParams, Node},
    },
    tools::synthetic_node::{SyntheticNode, SyntheticNodeBuilder},
};

pub async fn get_handshaked_synth_node(net_addr: SocketAddr) -> SyntheticNode {
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

pub async fn get_wallet_token(kmd: &mut Kmd) -> String {
    let wallets = kmd.get_wallets().await.expect("couldn't get the wallets");

    let wallet_id = wallets
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "unencrypted-default-wallet")
        .expect("couldn't find an unencrypted default wallet")
        .id;

    kmd.get_wallet_handle_token(wallet_id, "".to_string())
        .await
        .expect("couldn't get the wallet token")
        .wallet_handle_token
}

pub async fn get_txn_params(node: &mut Node) -> TransactionParams {
    node.rest_client()
        .expect("couldn't get the REST client")
        .get_transaction_params()
        .await
        .expect("couldn't get the transaction parameters")
}

pub async fn get_pub_key_addr(kmd: &mut Kmd, wallet_token: String) -> Address {
    let pub_key = kmd
        .get_keys(wallet_token)
        .await
        .expect("couldn't get the wallet keys")
        .addresses
        .pop()
        .expect("couldn't find any public keys in the wallet");

    Address::from_string(&pub_key).expect("couldn't convert public key to address")
}

pub async fn get_signed_tagged_txn(
    kmd: &mut Kmd,
    wallet_token: String,
    txn: &Transaction,
) -> Vec<u8> {
    let mut signed_txn = kmd
        .sign_transaction(wallet_token, "".to_string(), txn)
        .await
        .expect("couldn't sign the transaction")
        .signed_transaction;

    let mut tagged_msg = Tag::get_tag_str(&Tag::Txn).as_bytes().to_vec();
    tagged_msg.append(&mut signed_txn);
    tagged_msg
}
