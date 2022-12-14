use tempfile::TempDir;

use crate::setup::{kmd::Kmd, node::Node};

#[tokio::test]
#[allow(non_snake_case)]
async fn c012_TXN_submit_txn_and_expect_to_receive_it() {
    // ZG-CONFORMANCE-012
    // TODO: write a description in the SPEC doc (once all is done here)

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

    let wallets = kmd.get_wallets().await.expect("couldn't get the wallets");
    let wallet_id = wallets
        .wallets
        .into_iter()
        .find(|wallet| wallet.name == "unencrypted-default-wallet")
        .expect("couldn't find an unencrypted default wallet")
        .id;

    let init_wallet_rsp = kmd.get_wallet_handle_token(wallet_id, "".to_string()).await;
    println!(
        "a temporary log with init_wallet_rsp: {:?}",
        init_wallet_rsp
    );

    // TODO(Rqnsom):
    // 1. add two synthetic_node nodes
    // 2. prepare a transaction via kmd V1 REST API (ongoing...)
    // 3. the synthetic_node_tx node submits a txn to the node
    // 4. the synthetic_node_rx node expects that same txn from the node

    // temp solution to check all is running well (manual check).
    tokio::time::sleep(tokio::time::Duration::from_secs(300)).await;

    // Gracefully shut down the nodes.
    kmd.stop().expect("unable to stop the kmd instance");
    node.stop().expect("unable to stop the node");
}
