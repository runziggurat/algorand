use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use tempfile::TempDir;
use tokio::{net::TcpSocket, sync::Barrier, task::JoinSet, time::timeout};
use ziggurat_core_metrics::{
    latency_tables::{LatencyRequestStats, LatencyRequestsTable},
    recorder::TestMetrics,
    tables::duration_as_ms,
};
use ziggurat_core_utils::err_constants::{
    ERR_NODE_ADDR, ERR_NODE_BUILD, ERR_NODE_STOP, ERR_SOCKET_BIND, ERR_SYNTH_BUILD,
    ERR_SYNTH_CONNECT, ERR_SYNTH_UNICAST, ERR_TEMPDIR_NEW,
};

use crate::{
    protocol::{
        codecs::{
            algomsg::AlgoMsg,
            msgpack::Round,
            payload::Payload,
            topic::{TopicMsgResp, UniEnsBlockReq, UniEnsBlockReqType},
        },
        payload_factory::PayloadFactory,
    },
    setup::node::Node,
    tools::{ips::IPS, synthetic_node::SyntheticNodeBuilder},
};

const METRIC_LATENCY: &str = "block_test_latency";
// number of requests to send per peer
const REQUESTS: u16 = 100;
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(3);

#[cfg_attr(
    not(feature = "performance"),
    ignore = "run this test with the 'performance' feature enabled"
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[allow(non_snake_case)]
async fn p001_GET_BLOCKS_latency() {
    // ZG-PERFORMANCE-001, Block getting latency
    //
    // This test checks if node behaves as expected under load from other peers.
    // We test the overall performance of a node's get blocks (with certs) latency.
    //
    // Results should be inspected manually as they are strongly dependent on the machine.
    // Sample results can be observed in the file:
    //    algorand/src/tests/performance/results/p001_GET_BLOCKS_latency.txt.
    //
    // *NOTE* run with `cargo test --release  tests::performance::get_blocks -- --nocapture`
    // Before running test generate dummy devices with different ips using toos/ips.py

    let synth_counts = vec![1, 50, 100, 200, 300, 400, 500, 600, 700, 800];

    let mut table = LatencyRequestsTable::default();

    for synth_count in synth_counts {
        let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
        let mut node = Node::builder().build(target.path()).expect(ERR_NODE_BUILD);
        node.start().await;

        let node_addr = node.net_addr().expect(ERR_NODE_ADDR);

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

            socket.bind(ip).expect(ERR_SOCKET_BIND);
            synth_sockets.push(socket);
        }

        // setup metrics recorder
        let test_metrics = TestMetrics::default();
        // clear metrics and register metrics
        metrics::register_histogram!(METRIC_LATENCY);

        let mut synth_handles = JoinSet::new();
        let test_start = tokio::time::Instant::now();

        let barrier = Arc::new(Barrier::new(synth_count));

        for socket in synth_sockets {
            let arc_barrier = barrier.clone();
            synth_handles.spawn(simulate_peer(node_addr, socket, arc_barrier));
        }

        // wait for peers to complete
        while (synth_handles.join_next().await).is_some() {}

        let time_taken_secs = test_start.elapsed().as_secs_f64();

        let snapshot = test_metrics.take_snapshot();
        if let Some(latencies) = snapshot.construct_histogram(METRIC_LATENCY) {
            if latencies.entries() >= 1 {
                // add stats to table display
                table.add_row(LatencyRequestStats::new(
                    synth_count as u16,
                    REQUESTS,
                    latencies,
                    time_taken_secs,
                ));
            }
        }

        node.stop().expect(ERR_NODE_STOP);
    }

    // Display results table
    println!("\r\n{}", table);
}

const ROUND_KEY: Round = 1;
#[allow(unused_must_use)] // just for result of the timeout
async fn simulate_peer(node_addr: SocketAddr, socket: TcpSocket, start_barrier: Arc<Barrier>) {
    let mut synth_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect(ERR_SYNTH_BUILD);

    // Establish peer connection
    synth_node
        .connect_from(node_addr, socket)
        .await
        .expect(ERR_SYNTH_CONNECT);

    let mut payload_factory = PayloadFactory::new(
        Payload::UniEnsBlockReq(UniEnsBlockReq {
            data_type: UniEnsBlockReqType::BlockAndCert,
            round_key: ROUND_KEY,
            nonce: 1,
        }),
        None,
    );

    let requests = payload_factory.generate_payloads(REQUESTS as usize);

    // Wait for all peers to connect
    start_barrier.wait().await;

    for message in requests {
        // Query transaction via peer protocol.
        if !synth_node.is_connected(node_addr) {
            break;
        }

        synth_node
            .unicast(node_addr, message)
            .expect(ERR_SYNTH_UNICAST);

        let now = Instant::now();

        // We can safely drop the result here because we don't care about it - if the message is
        // received and it's our response we simply register it for histogram and break the loop.
        // In every other case we simply move out and go to another request iteration.
        timeout(RESPONSE_TIMEOUT, async {
            loop {
                let m = synth_node.recv_message().await;
                if matches!(&m.1, AlgoMsg { payload: Payload::TopicMsgResp(TopicMsgResp::UniEnsBlockRsp(rsp)), .. }
                     if rsp.block.is_some() && rsp.block.as_ref().unwrap().round == ROUND_KEY && rsp.cert.is_some()) {
                    metrics::histogram!(METRIC_LATENCY, duration_as_ms(now.elapsed()));
                    break;
                }
            }
        }).await;
    }

    synth_node.shut_down().await
}
