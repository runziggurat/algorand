use std::{
    collections::HashSet,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant},
};

use data_encoding::BASE64;
use histogram::Histogram;
use tabled::{Table, Tabled};
use tempfile::TempDir;
use tokio::{net::TcpSocket, sync::Barrier, task::JoinSet, time::timeout};

use crate::{
    protocol::{
        codecs::{
            algomsg::AlgoMsg,
            msgpack::{
                Address, Ed25519PublicKey, Ed25519Signature, HashDigest, NetPrioResponse,
                OneTimeSignature, Response, Round,
            },
            payload::Payload,
            tagmsg::Tag,
            topic::{MsgOfInterest, TopicMsgResp, UniEnsBlockReq, UniEnsBlockReqType},
        },
        payload_factory::PayloadFactory,
    },
    setup::node::Node,
    tools::{
        constants::{
            ERR_NODE_ADDR, ERR_NODE_BUILD, ERR_NODE_STOP, ERR_SOCKET_BIND, ERR_SYNTH_BUILD,
            ERR_SYNTH_CONNECT, ERR_SYNTH_UNICAST, ERR_TEMPDIR_NEW,
        },
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
    #[tabled(rename = " normal peers ")]
    normal_peers: u16,
    #[tabled(rename = " high-traffic peers ")]
    high_traffic_peers: u16,
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
    pub fn new(
        normal_peers: u16,
        high_traffic_peers: u16,
        requests: u16,
        latency: Histogram,
        time: f64,
    ) -> Self {
        Self {
            normal_peers,
            high_traffic_peers,
            requests,
            completion: (latency.entries() as f64) / (normal_peers as f64 * requests as f64)
                * 100.00,
            latency_min: latency.minimum().unwrap() as u16,
            latency_max: latency.maximum().unwrap() as u16,
            latency_std_dev: latency.stddev().unwrap() as u16,
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

const METRIC_LATENCY: &str = "traffic_test_latency";
// number of requests to send per peer
const REQUESTS: u16 = 300;
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(3);
const ROUND_KEY: Round = 1;

// ZG-PERFORMANCE-002, Getting messages of one kind while other nodes send some other traffic
//
// We test the overall performance of a node's certain message types latency while other
// nodes send some other traffic, especially higher priority traffic.
//
// Test should be inspected manually to check how other nodes affect the latency of
// the node under test. Each test case prints a table with results.
//
// Sample results:
// ┌────────────────┬──────────────────────┬────────────┬────────────┬────────────┬────────────────┬────────────────┬────────────┐
// │  normal peers  │  high-traffic peers  │  requests  │  min (ms)  │  max (ms)  │  std dev (ms)  │  completion %  │  time (s)  │
// ├────────────────┼──────────────────────┼────────────┼────────────┼────────────┼────────────────┼────────────────┼────────────┤
// │              1 │                    1 │        300 │          0 │          1 │              1 │         100.00 │       0.44 │
// ├────────────────┼──────────────────────┼────────────┼────────────┼────────────┼────────────────┼────────────────┼────────────┤
// │              1 │                   50 │        300 │          1 │          2 │              1 │         100.00 │       0.50 │
// ├────────────────┼──────────────────────┼────────────┼────────────┼────────────┼────────────────┼────────────────┼────────────┤
// │              1 │                  100 │        300 │          1 │          2 │              1 │         100.00 │       0.58 │
// ├────────────────┼──────────────────────┼────────────┼────────────┼────────────┼────────────────┼────────────────┼────────────┤
// │              1 │                  200 │        300 │          1 │          6 │              1 │         100.00 │       0.63 │
// ├────────────────┼──────────────────────┼────────────┼────────────┼────────────┼────────────────┼────────────────┼────────────┤
// │              1 │                  300 │        300 │          1 │         24 │              2 │         100.00 │       0.78 │
// ├────────────────┼──────────────────────┼────────────┼────────────┼────────────┼────────────────┼────────────────┼────────────┤
// │              1 │                  400 │        300 │          1 │          7 │              1 │         100.00 │       0.77 │
// ├────────────────┼──────────────────────┼────────────┼────────────┼────────────┼────────────────┼────────────────┼────────────┤
// │              1 │                  799 │        300 │          1 │         10 │              1 │         100.00 │       0.82 │
// └────────────────┴──────────────────────┴────────────┴────────────┴────────────┴────────────────┴────────────────┴────────────┘
// *NOTE* run with `cargo test --release  tests::performance::prio -- --nocapture --test-threads=1`
// Before running test generate dummy devices with different ips using toos/ips.py

#[cfg_attr(
    not(feature = "performance"),
    ignore = "run this test with the 'performance' feature enabled"
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[allow(non_snake_case)]
async fn p002_t1_TRAFFIC_HIGH_LOW_latency() {
    // ZG-PERFORMANCE-002

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
    let high_prio_factory =
        PayloadFactory::new(Payload::MsgOfInterest(MsgOfInterest { tags }), None);
    let low_prio_factory = PayloadFactory::new(
        Payload::UniEnsBlockReq(UniEnsBlockReq {
            data_type: UniEnsBlockReqType::BlockAndCert,
            round_key: ROUND_KEY,
            nonce: 123,
        }),
        None,
    );
    run_traffic_test(high_prio_factory, low_prio_factory).await;
}

#[cfg_attr(
    not(feature = "performance"),
    ignore = "run this test with the 'performance' feature enabled"
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[allow(non_snake_case)]
async fn p002_t2_TRAFFIC_SAME_PRIO_latency() {
    // ZG-PERFORMANCE-002

    let high_traffic_factory = PayloadFactory::new(
        Payload::UniEnsBlockReq(UniEnsBlockReq {
            data_type: UniEnsBlockReqType::BlockAndCert,
            round_key: 3,
            nonce: 1,
        }),
        None,
    );
    let normal_traffic_factory = PayloadFactory::new(
        Payload::UniEnsBlockReq(UniEnsBlockReq {
            data_type: UniEnsBlockReqType::BlockAndCert,
            round_key: ROUND_KEY,
            nonce: 123,
        }),
        None,
    );
    run_traffic_test(high_traffic_factory, normal_traffic_factory).await;
}

#[cfg_attr(
    not(feature = "performance"),
    ignore = "run this test with the 'performance' feature enabled"
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[allow(non_snake_case)]
async fn p002_t3_COMB_MSG_DIGEST_latency() {
    // ZG-PERFORMANCE-002

    let hash = vec![2u8; 32];
    let high_traffic_factory =
        PayloadFactory::new(Payload::MsgDigestSkip(HashDigest::from(&hash)), None);
    let normal_traffic_factory = PayloadFactory::new(
        Payload::UniEnsBlockReq(UniEnsBlockReq {
            data_type: UniEnsBlockReqType::BlockAndCert,
            round_key: ROUND_KEY,
            nonce: 123,
        }),
        None,
    );
    run_traffic_test(high_traffic_factory, normal_traffic_factory).await;
}

#[cfg_attr(
    not(feature = "performance"),
    ignore = "run this test with the 'performance' feature enabled"
)]
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
#[allow(non_snake_case)]
async fn p002_t4_NET_PRIO_latency() {
    // ZG-PERFORMANCE-002
    let nonce = BASE64.encode(&[0u8; 32]);

    let high_traffic_factory = PayloadFactory::new(
        Payload::NetPrioResponse(NetPrioResponse {
            response: Response { nonce },
            round: ROUND_KEY,
            sender_addr: Address::new([1u8; 32]),
            sig: OneTimeSignature {
                pk: Ed25519PublicKey([2u8; 32]),
                pk2: Ed25519PublicKey([3u8; 32]),
                sig: Ed25519Signature([4u8; 64]),
                pk1sig: Ed25519Signature([5u8; 64]),
                pk2sig: Ed25519Signature([6u8; 64]),
                pksigold: Ed25519Signature([7u8; 64]),
            },
        }),
        None,
    );
    let normal_traffic_factory = PayloadFactory::new(
        Payload::UniEnsBlockReq(UniEnsBlockReq {
            data_type: UniEnsBlockReqType::BlockAndCert,
            round_key: ROUND_KEY,
            nonce: 123,
        }),
        None,
    );
    run_traffic_test(high_traffic_factory, normal_traffic_factory).await;
}

async fn run_traffic_test(
    high_traffic_factory: PayloadFactory,
    normal_traffic_factory: PayloadFactory,
) {
    let h_traffic_peer_set = vec![1, 50, 100, 200, 300, 400, 799];
    let n_traffic_peers = 1;

    let mut table = RequestsTable::default();

    for h_traffic_peers in h_traffic_peer_set {
        let total_peers = n_traffic_peers + h_traffic_peers;
        let barrier = Arc::new(Barrier::new(total_peers));

        let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
        let mut node = Node::builder().build(target.path()).expect(ERR_NODE_BUILD);
        node.start().await;

        let node_addr = node.net_addr().expect(ERR_NODE_ADDR);

        let mut synth_sockets = Vec::with_capacity(total_peers);
        let mut ips = IPS.to_vec();

        for _ in 0..total_peers {
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

        let arc_barrier = barrier.clone();
        synth_handles.spawn(simulate_normal_traffic_peer(
            node_addr,
            synth_sockets.pop().unwrap(),
            arc_barrier,
            normal_traffic_factory.clone(),
        ));

        for socket in synth_sockets {
            let arc_barrier = barrier.clone();
            synth_handles.spawn(simulate_high_priority_peer(
                node_addr,
                socket,
                arc_barrier,
                high_traffic_factory.clone(),
            ));
        }

        // wait for peers to complete
        while (synth_handles.join_next().await).is_some() {}

        let time_taken_secs = test_start.elapsed().as_secs_f64();

        let snapshot = test_metrics.take_snapshot();
        if let Some(latencies) = snapshot.construct_histogram(METRIC_LATENCY) {
            if latencies.entries() >= 1 {
                // add stats to table display
                table.add_row(RequestStats::new(
                    n_traffic_peers as u16, // only one normal peer
                    h_traffic_peers as u16,
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

#[allow(unused_must_use)]
async fn simulate_normal_traffic_peer(
    node_addr: SocketAddr,
    socket: TcpSocket,
    start_barrier: Arc<Barrier>,
    mut normal_traffic_factory: PayloadFactory,
) {
    let mut synth_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect(ERR_SYNTH_BUILD);

    // Establish peer connection
    synth_node
        .connect_from(node_addr, socket)
        .await
        .expect(ERR_SYNTH_CONNECT);

    let requests = normal_traffic_factory.generate_payloads(REQUESTS as usize);

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
        // We cannot simply put Unwrap here because it will panic on timeout - that's not our
        // intention - we want to run the test further and gather other results.
        timeout(RESPONSE_TIMEOUT, async {
            loop {
                let m = synth_node.recv_message().await.1;
                // TODO[asmie]: matcher should be taken from the factory or should depened on factory payload type used
                if matches!(&m, AlgoMsg { payload: Payload::TopicMsgResp(TopicMsgResp::UniEnsBlockRsp(rsp)), ..}
                     if rsp.block.is_some() && rsp.block.as_ref().unwrap().round == ROUND_KEY && rsp.cert.is_some()) {
                    metrics::histogram!(METRIC_LATENCY, duration_as_ms(now.elapsed()));
                    break;
                }
            }
        }).await;
    }

    synth_node.shut_down().await
}

async fn simulate_high_priority_peer(
    node_addr: SocketAddr,
    socket: TcpSocket,
    start_barrier: Arc<Barrier>,
    mut high_traffic_factory: PayloadFactory,
) {
    let mut synth_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect(ERR_SYNTH_BUILD);

    // Establish peer connection
    synth_node
        .connect_from(node_addr, socket)
        .await
        .expect(ERR_SYNTH_CONNECT);

    let requests = high_traffic_factory.generate_payloads(REQUESTS as usize);

    // Wait for all peers to start
    start_barrier.wait().await;

    for message in requests {
        if !synth_node.is_connected(node_addr) {
            break;
        }

        synth_node
            .unicast(node_addr, message)
            .expect(ERR_SYNTH_UNICAST);

        // Just check if there is anything to read in the incoming queue. If so, read and
        // discard it. We don't care about the response.
        let _ = synth_node
            .recv_message_timeout(Duration::from_micros(10))
            .await;
    }

    synth_node.shut_down().await
}
