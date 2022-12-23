use tokio::time::Duration;

mod handshake;
pub mod post_handshake;
mod random_bytes;

/// Time after which the synthetic node expects to be disconnected from the node.
pub const WAIT_FOR_DISCONNECT: Duration = Duration::from_millis(500);
