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

The test index makes use of symbolic language in describing connection and message sending directions. As a convention, Ziggurat test nodes are to the left of the connection/message arrows, and Algod instances are to the right: `A -> B` and `A <- B`. In this way, `->` signifies "Ziggurat connects to Algod" and `<-` signifies the opposite. Furthermore, `-> ping` signifies "Ziggurat sends a `Ping` message to Algod" and `<- pong` signifies "Algod sends a `Pong` message to Ziggurat". Lastly, `<>` signifies a completed handshake, in either direction.

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

    The node responds correctly to a block request message (V1 algod API) which is how newly connected node queries for block data.

    <>
    -> http GET /v1/block/{round}
    <- http response with block-certificate data

    Assert: the appropriate response is sent.

