//! Useful tools constants.

use tokio::time::Duration;

/// Connection timeout.
pub const CONNECTION_TIMEOUT: Duration = Duration::from_secs(10);

/// Timeout when waiting for expected message / node's state.
pub const EXPECT_MSG_TIMEOUT: Duration = Duration::from_secs(10);
