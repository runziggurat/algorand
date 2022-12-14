use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    time::{Duration, Instant},
};

use tempfile::TempDir;
use tokio::{net::TcpSocket, time::timeout};

use crate::{
    protocol::codecs::{
        msgpack::Round,
        payload::Payload,
        topic::{TopicMsgResp, UniEnsBlockReq, UniEnsBlockReqType},
    },
    setup::node::Node,
    tools::{
        ips::IPS,
        metrics::{
            recorder::TestMetrics,
            tables::{duration_as_ms, RequestStats, RequestsTable},
        },
        synthetic_node::SyntheticNodeBuilder,
    },
};

const METRIC_LATENCY: &str = "block_test_latency";
// number of requests to send per peer
const REQUESTS: u16 = 100;
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(3);

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[allow(non_snake_case)]
async fn p001_t1_GET_BLOCKS_latency() {
    // ZG-PERFORMANCE-001, Block getting latency
    //
    // This test checks if node behaves as expected under load from other peers.
    // We test the overall performance of a node's get blocks (with certs) latency.
    //
    // Results should be inspected manually as they are strongly dependent on the machine.
    //
    // Sample results:
    // ┌─────────┬────────────┬────────────┬────────────┬────────────────┬────────────┬────────────┬────────────┬────────────┬────────────┬────────────────┬────────────┬──────────────┐
    // │  peers  │  requests  │  min (ms)  │  max (ms)  │  std dev (ms)  │  10% (ms)  │  50% (ms)  │  75% (ms)  │  90% (ms)  │  99% (ms)  │  completion %  │  time (s)  │  requests/s  │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │       1 │        100 │          1 │          2 │              1 │          1 │          1 │          1 │          1 │          2 │         100.00 │       1.16 │        86.38 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │      50 │        100 │          0 │          4 │              1 │          1 │          1 │          1 │          2 │          2 │         100.00 │       1.24 │      4021.24 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     100 │        100 │          0 │          9 │              2 │          1 │          1 │          1 │          2 │          5 │         100.00 │       1.30 │      7663.57 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     200 │        100 │          0 │         11 │              2 │          1 │          1 │          1 │          2 │          4 │          99.81 │       4.31 │      4630.10 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     300 │        100 │          0 │          9 │              1 │          1 │          1 │          1 │          2 │          4 │          99.62 │       4.35 │      6871.47 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     400 │        100 │          0 │         19 │              2 │          1 │          1 │          2 │          2 │          5 │          99.59 │       4.41 │      9028.24 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     500 │        100 │          0 │         14 │              2 │          1 │          1 │          2 │          2 │          5 │          99.17 │       7.39 │      6706.93 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     600 │        100 │          0 │         14 │              2 │          1 │          1 │          2 │          2 │          6 │          99.00 │       7.46 │      7961.44 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     700 │        100 │          0 │         11 │              2 │          1 │          1 │          2 │          3 │          6 │          98.77 │      10.43 │      6629.62 │
    // ├─────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼────────────┼────────────┼────────────┼────────────┼────────────────┼────────────┼──────────────┤
    // │     800 │        100 │          0 │         11 │              1 │          1 │          1 │          2 │          2 │          5 │          99.02 │       7.62 │     10396.44 │
    // └─────────┴────────────┴────────────┴────────────┴────────────────┴────────────┴────────────┴────────────┴────────────┴────────────┴────────────────┴────────────┴──────────────┘
    // *NOTE* run with `cargo test --release  tests::performance::get_blocks -- --nocapture`
    // Before running test generate dummy devices with different ips using toos/ips.py

    let synth_counts = vec![1, 50, 100, 200, 300, 400, 500, 600, 700, 800];

    let mut table = RequestsTable::default();

    for synth_count in synth_counts {
        let target = TempDir::new().expect("couldn't create a temporary directory");
        let mut node = Node::builder()
            .build(target.path())
            .expect("unable to build the node");
        node.start().await;

        let node_addr = node.net_addr().expect("network address not found");

        let mut synth_sockets = Vec::with_capacity(synth_count);
        let mut ips = IPS.to_vec();

        for _ in 0..synth_count {
            // If there is address for our thread in the pool we can use it.
            // Otherwise we'll not set bound_addr and use local IP addr (127.0.0.1).
            let ip = ips.pop().unwrap_or("127.0.0.1");

            let ip = SocketAddr::new(IpAddr::V4(Ipv4Addr::from_str(ip).unwrap()), 0);
            let socket = TcpSocket::new_v4().unwrap();

            // Make sure we can reuse the address and port
            socket.set_reuseaddr(true).unwrap();
            socket.set_reuseport(true).unwrap();

            socket.bind(ip).expect("unable to bind to socket");
            synth_sockets.push(socket);
        }

        // setup metrics recorder
        let test_metrics = TestMetrics::default();
        // clear metrics and register metrics
        metrics::register_histogram!(METRIC_LATENCY);

        let mut synth_handles = Vec::with_capacity(synth_count);
        let test_start = tokio::time::Instant::now();

        for socket in synth_sockets {
            synth_handles.push(tokio::spawn(simulate_peer(node_addr, socket)));
        }

        // wait for peers to complete
        for handle in synth_handles {
            let _ = handle.await;
        }

        let time_taken_secs = test_start.elapsed().as_secs_f64();

        let snapshot = test_metrics.take_snapshot();
        if let Some(latencies) = snapshot.construct_histogram(METRIC_LATENCY) {
            if latencies.entries() >= 1 {
                // add stats to table display
                table.add_row(RequestStats::new(
                    synth_count as u16,
                    REQUESTS,
                    latencies,
                    time_taken_secs,
                ));
            }
        }

        node.stop().expect("unable to stop the node");
    }

    // Display results table
    println!("\r\n{}", table);
}

const ROUND_KEY: Round = 1;
#[allow(unused_must_use)] // just for result of the timeout
async fn simulate_peer(node_addr: SocketAddr, socket: TcpSocket) {
    let mut synth_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect("unable to build a synthetic node");

    // Establish peer connection
    synth_node
        .connect_from(node_addr, socket)
        .await
        .expect("unable to connect to node");

    for i in 0..REQUESTS {
        let message = Payload::UniEnsBlockReq(UniEnsBlockReq {
            data_type: UniEnsBlockReqType::BlockAndCert,
            round_key: ROUND_KEY,
            nonce: i as u64,
        });

        // Query transaction via peer protocol.
        if !synth_node.is_connected(node_addr) {
            break;
        }

        synth_node
            .unicast(node_addr, message)
            .expect("unable to send message");

        let now = Instant::now();

        // We can safely drop the result here because we don't care about it - if the message is
        // received and it's our response we simply register it for histogram and break the loop.
        // In every other case we simply move out and go to another request iteration.
        timeout(RESPONSE_TIMEOUT, async {
            loop {
                let m = synth_node.recv_message().await;
                if matches!(&m.1, Payload::TopicMsgResp(TopicMsgResp::UniEnsBlockRsp(rsp))
                     if rsp.block.is_some() && rsp.block.as_ref().unwrap().round == ROUND_KEY && rsp.cert.is_some()) {
                    metrics::histogram!(METRIC_LATENCY, duration_as_ms(now.elapsed()));
                    break;
                }
            }
        }).await;
    }

    synth_node.shut_down().await
}
