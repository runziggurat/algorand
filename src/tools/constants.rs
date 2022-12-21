//! Useful tools constants.

use tokio::time::Duration;

/// Connection timeout.
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

/// Timeout when waiting for an expected message or a change in the node's state.
pub const EXPECT_MSG_TIMEOUT: Duration = Duration::from_secs(10);

// Below common error constants and messages to be used across the project.
/// Error message when binding to specified socket fails.
pub const ERR_BIND_TO_SOCKET_FAILED: &str = "unable to bind to socket";

/// Error message when node network address is not found.
pub const ERR_NET_ADDR_NOT_FOUND: &str = "network address not found";

/// Error message for a failed connection.
pub const ERR_NODE_CONNECTION_FAILED: &str = "unable to connect to the node";

/// Error message when building a node fails.
pub const ERR_NODE_BUILD_FAILED: &str = "unable to build the node";

/// Error message when a node fails to stop.
pub const ERR_NODE_UNABLE_TO_STOP: &str = "unable to stop the node";

/// Error message when sending message fails.
pub const ERR_SEND_MESSAGE_FAILED: &str = "unable to send message";

/// Error message when synthetic node creation fails.
pub const ERR_SYNTH_NODE_BUILD_FAILED: &str = "unable to build a synthetic node";

/// Error message when temporary directory creation fails.
pub const ERR_TEMPDIR_CREATION_FAILED: &str = "couldn't create a temporary directory";
