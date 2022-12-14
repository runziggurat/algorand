//! Useful node constants.

use tokio::time::Duration;

/// Node directory without an index. The correctly indexed node directory is "Node0".
pub const NODE_DIR: &str = "Node";

/// The address on which the relay node listens for incoming connections.
///
/// Non-relay nodes do not have this address configured.
/// The address is named `NetAddress` in the [official Algorand
/// documentation](https://developer.algorand.org/docs/run-a-node/reference/config/).
pub const NET_ADDR_FILE: &str = "algod-listen.net";

/// The address on which the node listens for REST API calls.
///
/// The address is named `EndpointAddress` in the [official Algorand
/// documentation](https://developer.algorand.org/docs/run-a-node/reference/config/).
pub const REST_ADDR_FILE: &str = "algod.net";

/// Timeout when waiting for [Node](crate::setup::node::Node)'s start.
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
