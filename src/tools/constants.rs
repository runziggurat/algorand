//! Useful tools constants.

use tokio::time::Duration;

/// Timeout when waiting for expected message / node's state.
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);
