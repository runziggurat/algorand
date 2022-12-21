use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use histogram::Histogram;
use tabled::{Table, Tabled};
use tempfile::TempDir;
use tokio::{net::TcpSocket, sync::Barrier, task::JoinSet, time::timeout};

use crate::{
    protocol::codecs::{
        msgpack::Round,
        payload::Payload,
        tagmsg::Tag,
        topic::{MsgOfInterest, TopicMsgResp, UniEnsBlockReq, UniEnsBlockReqType},
    },
    setup::node::Node,
    tools::{
        ips::IPS,
        metrics::{
            recorder::TestMetrics,
            tables::{duration_as_ms, fmt_table, table_float_display},
        },
        synthetic_node::SyntheticNodeBuilder,
    },
};

#[derive(Default)]
pub struct RequestsTable {
    rows: Vec<RequestStats>,
}

#[derive(Tabled, Default, Debug, Clone)]
pub struct RequestStats {
    #[tabled(rename = " peers ")]
    peers: u16,
    #[tabled(rename = " malicious peers ")]
    mpeers: u16,
    #[tabled(rename = " requests ")]
    requests: u16,
    #[tabled(rename = " min (ms) ")]
    latency_min: u16,
    #[tabled(rename = " max (ms) ")]
    latency_max: u16,
    #[tabled(rename = " std dev (ms) ")]
    latency_std_dev: u16,
    #[tabled(rename = " completion % ")]
    #[tabled(display_with = "table_float_display")]
    completion: f64,
    #[tabled(rename = " time (s) ")]
    #[tabled(display_with = "table_float_display")]
    time: f64,
}

impl RequestStats {
    pub fn new(peers: u16, mpeers: u16, requests: u16, latencies: Histogram, time: f64) -> Self {
        Self {
            peers,
            mpeers,
            requests,
            completion: (latencies.entries() as f64) / (peers as f64 * requests as f64) * 100.00,
            latency_min: latencies.minimum().unwrap() as u16,
            latency_max: latencies.maximum().unwrap() as u16,
            latency_std_dev: latencies.stddev().unwrap() as u16,
            time,
        }
    }
}

impl RequestsTable {
    pub fn add_row(&mut self, row: RequestStats) {
        self.rows.push(row);
    }
}

impl std::fmt::Display for RequestsTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&fmt_table(Table::new(&self.rows)))
    }
}

const METRIC_LATENCY: &str = "prio_test_latency";
// number of requests to send per peer
const REQUESTS: u16 = 300;
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(3);

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[allow(non_snake_case)]
async fn p002_t1_PRIO_MSG_latency() {
    // ZG-PERFORMANCE-002, Block getting latency while malicious nodes send higher priority msgs
    //
    // We test the overall performance of a node's get blocks (with certs) latency while other
    // malicious nodes send higher priority messages.
    //
    // Test should be inspected manually to check how other malicious nodes affect the latency of
    // the node under test.
    //
    // Sample results:
    // ┌─────────┬───────────────────┬────────────┬────────────┬────────────┬────────────────┬────────────────┬────────────┐
    // │  peers  │  malicious peers  │  requests  │  min (ms)  │  max (ms)  │  std dev (ms)  │  completion %  │  time (s)  │
    // ├─────────┼───────────────────┼────────────┼────────────┼────────────┼────────────────┼────────────────┼────────────┤
    // │       1 │                 1 │        300 │          1 │          1 │              0 │         100.00 │       0.41 │
    // ├─────────┼───────────────────┼────────────┼────────────┼────────────┼────────────────┼────────────────┼────────────┤
    // │       1 │                50 │        300 │          1 │          2 │              1 │         100.00 │       0.51 │
    // ├─────────┼───────────────────┼────────────┼────────────┼────────────┼────────────────┼────────────────┼────────────┤
    // │       1 │               100 │        300 │          1 │          3 │              1 │         100.00 │       0.64 │
    // ├─────────┼───────────────────┼────────────┼────────────┼────────────┼────────────────┼────────────────┼────────────┤
    // │       1 │               200 │        300 │          1 │          5 │              1 │         100.00 │       0.62 │
    // ├─────────┼───────────────────┼────────────┼────────────┼────────────┼────────────────┼────────────────┼────────────┤
    // │       1 │               300 │        300 │          1 │         14 │              2 │         100.00 │       0.80 │
    // ├─────────┼───────────────────┼────────────┼────────────┼────────────┼────────────────┼────────────────┼────────────┤
    // │       1 │               400 │        300 │          1 │          9 │              1 │         100.00 │       0.82 │
    // └─────────┴───────────────────┴────────────┴────────────┴────────────┴────────────────┴────────────────┴────────────┘
    // *NOTE* run with `cargo test --release  tests::performance::prio -- --nocapture`
    // Before running test generate dummy devices with different ips using toos/ips.py

    let synth_counts = vec![1, 50, 100, 200, 300, 400];

    let mut table = RequestsTable::default();

    for synth_count in synth_counts {
        // synth_count malicious tasks plus one normal synth_node
        let barrier = Arc::new(Barrier::new(synth_count + 1));

        let target = TempDir::new().expect("couldn't create a temporary directory");
        let mut node = Node::builder()
            .build(target.path())
            .expect("unable to build the node");
        node.start().await;

        let node_addr = node.net_addr().expect("network address not found");

        let mut synth_sockets = Vec::with_capacity(synth_count + 1);
        let mut ips = IPS.to_vec();

        for _ in 0..synth_count + 1 {
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

        let mut synth_handles = JoinSet::new();
        let test_start = tokio::time::Instant::now();

        let arc_barrier = barrier.clone();
        synth_handles.spawn(simulate_normal_peer(
            node_addr,
            synth_sockets.pop().unwrap(),
            arc_barrier,
        ));

        for socket in synth_sockets {
            let arc_barrier = barrier.clone();
            synth_handles.spawn(simulate_malicious_peer(node_addr, socket, arc_barrier));
        }

        // wait for peers to complete
        while (synth_handles.join_next().await).is_some() {}

        let time_taken_secs = test_start.elapsed().as_secs_f64();

        let snapshot = test_metrics.take_snapshot();
        if let Some(latencies) = snapshot.construct_histogram(METRIC_LATENCY) {
            if latencies.entries() >= 1 {
                // add stats to table display
                table.add_row(RequestStats::new(
                    1_u16, // only one normal peer
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
async fn simulate_normal_peer(
    node_addr: SocketAddr,
    socket: TcpSocket,
    start_barrier: Arc<Barrier>,
) {
    let mut synth_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect("unable to build a synthetic node");

    // Establish peer connection
    synth_node
        .connect_from(node_addr, socket)
        .await
        .expect("unable to connect to node");

    // Wait for all peers to connect
    start_barrier.wait().await;

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

async fn simulate_malicious_peer(
    node_addr: SocketAddr,
    socket: TcpSocket,
    start_barrier: Arc<Barrier>,
) {
    let mut synth_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect("unable to build a synthetic node");

    // Establish peer connection
    synth_node
        .connect_from(node_addr, socket)
        .await
        .expect("unable to connect to node");

    // Wait for all peers to start
    start_barrier.wait().await;

    for _ in 0..REQUESTS {
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

        if !synth_node.is_connected(node_addr) {
            break;
        }

        synth_node
            .unicast(node_addr, message)
            .expect("unable to send message");

        // Just check if there is anything to read in the incoming queue. If so, read and
        // discard it. We don't care about the response.
        let _ = synth_node
            .recv_message_timeout(Duration::from_micros(10))
            .await;
    }

    synth_node.shut_down().await
}
