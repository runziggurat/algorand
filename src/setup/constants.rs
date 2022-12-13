//! Useful setup constants.

use tokio::time::Duration;

/// Ziggurat's configuration directory.
pub const ZIGGURAT_DIR: &str = ".ziggurat";

/// Ziggurat's Algorand's subdir.
pub const ALGORAND_WORK_DIR: &str = "algorand";

/// Initial setup dir for algod.
pub const ALGORAND_SETUP_DIR: &str = "setup";

/// Configuration file with paths to start algod.
pub const SETUP_CONFIG: &str = "config.toml";

/// Directory for the preloaded network of nodes which contain saved ledger and configuration data.
pub const PRIVATE_NETWORK_DIR: &str = "private_network";

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

/// Timeout when waiting for loading of files.
pub const LOAD_FILE_TIMEOUT_SECS: Duration = Duration::from_secs(3);

/// Timeout when waiting for [Node](crate::setup::node::Node)'s start.
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
