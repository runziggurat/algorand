//! Useful setup constants.

use tokio::time::Duration;

/// Directory of the kmd instance.
/// This directory is generated automatically within the node's directory when the node is created.
pub const KMD_DIR: &str = "kmd-v0.5";

/// Security token file needed for the REST API authentication.
pub const TOKEN_FILE: &str = "kmd.token";

/// The address on which the kmd instance listens for REST API calls.
pub const REST_ADDR_FILE: &str = "kmd.net";

/// Timeout when waiting for kmd instance to start responding.
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(5);
