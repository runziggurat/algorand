# Introduction

The purpose of this index is to provide an overview of the testing approaches to be implemented by Ziggurat. It is intended to evolve as the framework matures, leaving room for novel cases and extensions of existing cases, as called for by any protocol idiosyncrasies that may come to light during the development process.

Some test cases have been consolidated when similar behaviour is tested with differing messages. The final implementation of these cases will be subject to factors such as node setup and teardown details, test run time (and potentially runtime) constraints, readability and maintainability.

## Usage

The tests can be run with `cargo test` once Ziggurat is properly configured and dependencies (node instance to be tested) are satisfied. See the [README](README.md) for details.

Tests are grouped into the following categories: conformance, performance, and resistance. Each test is named after the category it belongs to, in addition to what's being tested. For example, `c001_handshake_when_node_receives_connection` is the first conformance test and tests the handshake behavior on the receiving end. The full naming convention is: `id_part_t(subtest_no)_(message type)_(extra_test_desc)`.

# Types of Tests

## Conformance

The conformance tests aim to verify the node adheres to the network protocol. In addition, they include some naive error cases with malicious and fuzzing cases consigned to the resistance tests. Most cases in this section will only require a socket standing in for the connected peer and a full node running in the background.

### Handshake

These tests verify the proper execution of a handshake between a node and a peer as well as some simple error cases.

### Post-handshake messages

These tests verify the node responds with the correct messages to requests and disconnects in certain trivial non-fuzz, non-malicious cases. These form the basic assumptions necessary for peering and syncing.

### Unsolicited post-handshake messages

These tests aim to evaluate the proper behaviour of a node when receiving unsolicited messages post-handshake.

### Simple peering

These tests evaluate the node's basic peering properties by verifying the data included in the messages are in accordance with the peering status of the node.

### Simple sync

These tests evaluate the node's basic syncing properties for transactions and blocks by verifying the data included in the message payloads are in accordance with the ranges provided by the peer.

## Performance

The performance tests aim to verify the node maintains a healthy throughput under pressure. This is principally done through simulating load with synthetic peers and evaluating the node's responsiveness. Synthetic peers will need to be able to simulate the behaviour of a full node by implementing handshaking, message sending and receiving.

### Load testing

These tests are intended to verify the node remains healthy under "reasonable load". Additionally these tests will be pushed to the extreme for resistance testing with heavier loads.

### Heavy load testing

These tests are meant to explore the impact of malicious network use against a node.

The amount of load and its frequency could be modulated to provide a comprehensive verification of the node's behaviour under different conditions (including synchronized requests from different peers and other worst case scenarios).

## Resistance

The resistance tests are designed for the early detection and avoidance of weaknesses exploitable through malicious behaviour. They attempt to probe boundary conditions with comprehensive fuzz testing and extreme load testing. The nature of the peers in these cases will depend on how accurately they needs to simulate node behaviour. It will likely be a mixture of simple sockets for the simple cases and peers used in the performance tests for the more advanced.

### Fuzz testing

The fuzz tests aim to buttress the message conformance tests with extra verification of expected node behaviour when receiving corrupted or broken messages. Our approach is targeting these specific areas and we anticipate broadening these test scenarios as necessary:

- Messages with any length and any content (random bytes).
- Messages with plausible lengths, e.g. 24 bytes for header and within the expected range for the body.
- Metadata-compliant messages, e.g. correct header, random body.
- Slightly corrupted but otherwise valid messages, e.g. N% of body replaced with random bytes.
- Messages with an incorrect checksum.
- Messages with differing announced and actual lengths.

# Test Index

| Symbol | Meaning                                                                |
|--------|------------------------------------------------------------------------|
| `-> A` | Ziggurat's synthetic node sends a message `A` to Algod                 |
| `<- B` | Algod sends a message `B` to Ziggurat's synthetic node                 |
| `>> C` | Ziggurat's synthetic node broadcasts a message `C` to all its peers    |
| `<< D` | Algod broadcasts a message `D` to all its peers                        |
| `<>`   | Signifies a completed handshake, in either direction                   |

## Network protocol test coverage

|  Message                   | Type                  | Coverage | Tests                             |
|----------------------------|-----------------------|----------|-----------------------------------|
| /v1/{network-name}/gossip  | HTTP (handshake)      | ✅       | `C001`, `C002`, `C003`, `R002`    |
| /v1/block/{round}          | HTTP (get block)      | ✅       | `C004`                            |
| AgreementVoteTag           | WS data (Tag: AV)     | ✅       | `C008`, `R003`                    |
| MsgOfInterestTag           | WS data (Tag: MI)     | ✅       | `C005`, `C006`, `P002`, `R003`    |
| MsgDigestSkipTag           | WS data (Tag: MS)     | ✅       | `C013`, `R003`, `R004`            |
| NetPrioResponseTag         | WS data (Tag: NP)     | ✅       | `C011`, `R003`                    |
| PingTag                    | WS data (Tag: pi)     | ✅       | `C009`, `R003`                    |
| PingReplyTag               | WS data (Tag: pj)     | ✅       | `C009`, `R003`                    |
| ProposalPayloadTag         | WS data (Tag: PP)     | ✅       | `C007`, `C013`, `R003`, `R004`    |
| StateProofSigTag           | WS data (Tag: SP)     | ❌       | `R003`                            |
| UniCatchupReqTag           | WS data (Tag: UC)     | ✅       | `C010`, `R003`                    |
| UniEnsBlockReqTag          | WS data (Tag: UE)     | ✅       | `C010`, `P001`, `P002`, `R003`    |
| TopicMsgRespTag            | WS data (Tag: TS)     | ✅       | `C010`, `P001`, `P002`, `R003`    |
| TxnTag                     | WS data (Tag: TX)     | ✅       | `C012`, `R003`                    |
| VoteBundleTag              | WS data (Tag: VB)     | ❌       | `R003`                            |

## Conformance

### ZG-CONFORMANCE-001

    The node correctly performs a handshake from the responder side.

    ->
    -> http handshake request (websocket upgrade)
    <- http handshake response (websocket upgrade accept)

    Assert: the node’s peer count has increased to 1 and the synthetic node is an established peer.

### ZG-CONFORMANCE-002

    The node correctly performs a handshake from the initiator side.

    <-
    <- http handshake request (websocket upgrade)
    -> http handshake response (websocket upgrade accept)

    Assert: the node’s peer count has increased to 1 and the synthetic node is an established peer.

### ZG-CONFORMANCE-003

    The node should *NOT* send any messages after connection if there was no handshake.
    The test waits for the predefined amount of time, ensuring no messages were received.

### ZG-CONFORMANCE-004

    The node responds correctly to a block request message (V1 algod API) which is how newly connected node queries for block data.

    <>
    -> http GET /v1/block/{round}
    <- http response with block-certificate data

    Assert: the appropriate response is sent.

### ZG-CONFORMANCE-005

    The node and the synthetic node exchange MsgOfInterest messages after the handshake.

    <>
    <- MsgOfInterest
    -> MsgOfInterest

### ZG-CONFORMANCE-006

    The synthetic node sends the MsgOfInterest message with an empty list to indicate it's not interested in any of the nodes' messages.
    <>
    <- MsgOfInterest
    -> MsgOfInterest (empty list)

    Assert: expect the node will stop sending messages to the syntethic node.

### ZG-CONFORMANCE-007

    The node broadcasts ProposalPayload messages after the handshake.

    <>
    <- ProposalPayload

### ZG-CONFORMANCE-008

    The node broadcasts AgreementVote messages after the handshake.

    <>
    <- AgreementVote

### ZG-CONFORMANCE-009

    The synthetic node sends a Ping request message to the node.

    <>
    -> Ping (data)
    <- PingReply (data)

    Assert: The node replies with the PingReply message.

    or alternatively:

    Assert: Perform the handshake and then idly wait for a Ping message request.

### ZG-CONFORMANCE-010

    The node responds correctly to a block request message for the UniEnsBlockReq/UniCatchupReq message request.

    <>
    -> UniEnsBlockReq / UniCatchupReq
    <- TopicMsgResp

    Assert: the response contains block for a requested round.

### ZG-CONFORMANCE-011

    The node sends a handshake request to which a synthetic node replies with a handshake response containing a
    network priority challenge. A synthetic node then expects to receive an answer to that challenge within
    the NetPrioResponse message.

    <-
    <- http handshake request
    -> http handshake response (priority challenge)
    <- NetPrioResponse

    Assert: the node answers the challenge and replies with the NetPrioResponse message.

### ZG-CONFORMANCE-012

    One synthetic node sends a transcation to the node.
    The node then broadcasts that transcation to all other nodes.
    Another synthetic node receives the broadcasted transcation.

    <>
    -> Txn
    << Txn

    Assert: the node successfully broadcasts the transaction.

### ZG-CONFORMANCE-013

    Send a huge valid proposal payload message to the node from one synthetic node and expect to receive a
    MsgDigestSkip message on the other synthetic node as a filter for a massive proposal payload message.

    <>
    -> ProposalPayload (enormous size)
    << MsgDigestSkip (hash of enormous proposal payload)

    Assert: the node successfully broadcasts the filter message.

## Performance

### ZG-PERFORMANCE-001

    The node behaves as expected under load when other peers are requesting blocks with certificates.

    <>
    In loop:
        -> UniEnsBlockReq
        <- TopicMsgResp

    Results should be introspected manually to check the node's health and responsiveness
    (latency, throughput) when requesting block data.

### ZG-PERFORMANCE-002

    The node behaves as expected under load when some peers are sending much traffic and some other peer 
    is trying to perform some other operations. There are different test cases checked especially the ones when
    nodes are sending traffic with messages with different priorities as well as the ones when nodes are sending
    messages with the same priority.

    Normal peer (example):
    <>
    In loop:
        -> UniEnsBlockReq
        <- TopicMsgResp

    High traffic peers (example):
        <>
    In loop:
        -> MsgOfInterest

    Results should be introspected manually to check the normal peer responsiveness (latency, throughput) when performing its operations.
    There are multiple test scenarios with different high-traffic messages:
    - MsgOfInterest
    - UniEnsBlockReq
    - MsgDigestSkip
    - NetPrioResponse
    - AggreementVote
    - ProposalPayload

### Results

[ZG-PERFORMANCE-001-TEST-1](src/tests/performance/results/p001_GET_BLOCKS_latency.txt)

## Resistance

### ZG-RESISTANCE-001

    The node rejects various random bytes pre-handshake.

    -> random bytes

    Assert: The synthetic node is disconnected after sending random bytes.

### ZG-RESISTANCE-002

    The node rejects the handshake in case the request handshake message contains invalid data.

    ->
    -> http handshake request (with an invalid data)
    <- http handshake response (with a reject reason)

    Assert: the node rejects all invalid handshake requests.

### ZG-RESISTANCE-003

    The node rejects various random bytes post-handshake.

    -> random bytes

    or alternatively

    -> random bytes prefixed with a valid Algod message tag

    Assert: The synthetic node is disconnected after sending random bytes.

### ZG-RESISTANCE-004

    The node can handle enormous messages.

    <>
    -> ProposalPayload (enormous size)

    Assert: the node handles enormous valid messages properly.

    or

    <>
    -> MsgDigestSkip (enormous hash length)

    Assert: the node rejects the connection for invalid length messages.

