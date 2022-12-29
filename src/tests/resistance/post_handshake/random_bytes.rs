use tempfile::TempDir;
use tokio::time::sleep;

use crate::{
    protocol::codecs::{payload::Payload, tagmsg::Tag},
    setup::node::Node,
    tests::resistance::WAIT_FOR_DISCONNECT,
    tools::{
        constants::{
            ERR_NODE_ADDR, ERR_NODE_BUILD, ERR_NODE_STOP, ERR_SYNTH_BUILD, ERR_SYNTH_CONNECT,
            ERR_SYNTH_UNICAST, ERR_TEMPDIR_NEW,
        },
        synthetic_node::SyntheticNodeBuilder,
        util::gen_rand_bytes,
    },
};

/// Send given bytes directly to the node after the handshake and return the connection status.
async fn send_bytes_to_the_node(data: Vec<u8>, debug: bool) -> bool {
    // Spin up a node instance.
    let target = TempDir::new().expect(ERR_TEMPDIR_NEW);
    let mut node = Node::builder()
        .log_to_stdout(debug)
        .build(target.path())
        .expect(ERR_NODE_BUILD);
    node.start().await;

    // Create a synthetic node and disable handshaking.
    let synthetic_node = SyntheticNodeBuilder::default()
        .build()
        .await
        .expect(ERR_SYNTH_BUILD);

    let net_addr = node.net_addr().expect(ERR_NODE_ADDR);

    // Create a connection without the handshake.
    synthetic_node
        .connect(net_addr)
        .await
        .expect(ERR_SYNTH_CONNECT);

    // Send some random data.
    let random_data_msg = Payload::RawBytes(data);
    synthetic_node
        .unicast(net_addr, random_data_msg)
        .expect(ERR_SYNTH_UNICAST);

    // Give some time to the node to kill our connection.
    sleep(WAIT_FOR_DISCONNECT).await;

    let is_connected = synthetic_node.is_connected(net_addr);

    // Gracefully shut down the nodes.
    synthetic_node.shut_down().await;
    node.stop().expect(ERR_NODE_STOP);

    is_connected
}

fn gen_tagged_msg_with_random_data(tag: Tag, len: usize) -> Vec<u8> {
    let mut msg_content_random = Tag::get_tag_str(&tag).as_bytes().to_vec();
    msg_content_random.extend(&gen_rand_bytes(len));
    msg_content_random
}

struct TagRandDataTestCfg {
    tag: Tag,
    debug_logs: bool,
    data_len_normal: usize,
    data_len_huge: usize,
}

impl TagRandDataTestCfg {
    fn with_tag(mut self, tag: Tag) -> Self {
        self.tag = tag;
        self
    }
}

impl Default for TagRandDataTestCfg {
    fn default() -> Self {
        Self {
            tag: Tag::RawBytes,
            debug_logs: false,
            data_len_normal: 15,
            data_len_huge: 1_000_000,
        }
    }
}

async fn send_tagged_rand_data_to_the_node(cfg: TagRandDataTestCfg) {
    // Run the test with normal data length.
    let tagged_random_data = gen_tagged_msg_with_random_data(cfg.tag, cfg.data_len_normal);
    assert!(
        !send_bytes_to_the_node(tagged_random_data, cfg.debug_logs).await,
        "the node shouldn't keep the connection alive after receiving random data"
    );

    // Run the test with huge data length.
    let tagged_random_data = gen_tagged_msg_with_random_data(cfg.tag, cfg.data_len_huge);
    assert!(
        !send_bytes_to_the_node(tagged_random_data, cfg.debug_logs).await,
        "the node shouldn't keep the connection alive after receiving random data"
    );
}

#[tokio::test]
#[allow(non_snake_case)]
async fn r003_t1_RANDOM_DATA_send_completely_random_data() {
    // ZG-RESISTANCE-003

    // Test status: always fails.
    send_tagged_rand_data_to_the_node(TagRandDataTestCfg::default()).await;
}

macro_rules! make_test {
    ($fn_name:ident, $tag:expr) => {
        paste::item! {
            #[tokio::test]
            #[allow(non_snake_case)]
            async fn [< r003_ $fn_name >] () {
                // ZG-RESISTANCE-003

                let cfg = TagRandDataTestCfg::default().with_tag($tag);
                send_tagged_rand_data_to_the_node(cfg).await;
            }
        }
    };
}

// Test status: mixed - sometimes pass, sometimes fail.
make_test!(
    t2_AGREEMENT_VOTE_send_random_data_after_tag,
    Tag::AgreementVote
);

// Test status: pass.
make_test!(
    t3_PROPOSAL_PAYLOAD_send_random_data_after_tag,
    Tag::ProposalPayload
);

// Test status: fails.
make_test!(
    t4_MSG_OF_INTEREST_send_random_data_after_tag,
    Tag::MsgOfInterest
);

// Test status: fails.
make_test!(
    t5_MSG_DIGEST_SKIP_send_random_data_after_tag,
    Tag::MsgDigestSkip
);

// Test status: fails.
make_test!(
    t6_NET_PRIO_RESPONSE_send_random_data_after_tag,
    Tag::NetPrioResponse
);

// Test status: fails.
make_test!(t7_PING_send_random_data_after_tag, Tag::Ping);

// Test status: fails.
make_test!(t8_PING_REPLY_send_random_data_after_tag, Tag::PingReply);

// Test status: fails.
make_test!(
    t9_STATE_PROOF_SIG_send_random_data_after_tag,
    Tag::StateProofSig
);

// Test status: fails.
make_test!(
    t10_UNI_CATCHUP_REQ_send_random_data_after_tag,
    Tag::UniCatchupReq
);

// Test status: fails.
make_test!(
    t11_UNI_ENS_BLOCK_REQ_send_random_data_after_tag,
    Tag::UniEnsBlockReq
);

// Test status: fail.
make_test!(
    t12_TOPIC_MSG_RESP_send_random_data_after_tag,
    Tag::TopicMsgResp
);

// Test status: pass.
make_test!(t13_TXN_send_random_data_after_tag, Tag::Txn);

// Test status: pass.
make_test!(t14_VOTE_BUNDLE_send_random_data_after_tag, Tag::VoteBundle);
