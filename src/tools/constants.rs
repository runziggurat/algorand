//! Useful tools constants.

use tokio::time::Duration;

/// Connection timeout.
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

/// Timeout when waiting for an expected message or a change in the node's state.
pub const EXPECT_MSG_TIMEOUT: Duration = Duration::from_secs(10);

// Below common error constants and messages to be used across the project.
// The main rule to create const names is: ERR_<WHO>_<FUNCTION_NAME>

/// Error message when binding to specified socket fails.
pub const ERR_SOCKET_BIND: &str = "unable to bind to socket";

/// Error message when the node's network address is not found.
pub const ERR_NODE_ADDR: &str = "network address not found";

/// Error message for a failed connection.
pub const ERR_NODE_CONNECT: &str = "unable to connect to the node";

/// Error message when building a node fails.
pub const ERR_NODE_BUILD: &str = "unable to build the node";

/// Error message when a node fails to stop.
pub const ERR_NODE_STOP: &str = "unable to stop the node";

/// Error message when building a kmd instance fails.
pub const ERR_KMD_BUILD: &str = "unable to build the kmd instance";

/// Error message when a kmd instance fails to stop.
pub const ERR_KMD_STOP: &str = "unable to stop the kmd instance";

/// Error message when sending message fails.
pub const ERR_SYNTH_UNICAST: &str = "unable to send a message";

/// Error message when synthetic node creation fails.
pub const ERR_SYNTH_BUILD: &str = "unable to build a synthetic node";

/// Error message for a failed connection.
pub const ERR_SYNTH_CONNECT: &str = "unable to connect to the node";

/// Error message for a failed connection.
pub const ERR_SYNTH_START_LISTENING: &str = "a synthetic node couldn't start listening";

/// Error message when temporary directory creation fails.
pub const ERR_TEMPDIR_NEW: &str = "couldn't create a temporary directory";
