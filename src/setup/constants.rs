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

/// Timeout when waiting for loading of files.
pub const LOAD_FILE_TIMEOUT_SECS: Duration = Duration::from_secs(3);
