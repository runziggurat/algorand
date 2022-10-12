//! Utilities for node configuration.

use std::{
    ffi::OsString,
    fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Result;
use serde::Deserialize;
use tokio::time::{sleep, timeout, Duration};

use crate::setup::constants::{
    LOAD_ADDR_TIMEOUT_SECS, NET_ADDR_FILE, REST_ADDR_FILE, SETUP_CONFIG,
};

/// Startup configuration for the node.
#[derive(Debug, Clone, Default)]
pub struct NodeConfig {
    /// Setting this option to true will enable node logging to stdout.
    pub log_to_stdout: bool,
    /// The path of the cache directory of the node.
    pub path: PathBuf,
    /// The network socket address of the node.
    pub net_addr: Option<SocketAddr>,
    /// The REST API socket address of the node.
    pub rest_api_addr: Option<SocketAddr>,
}

impl NodeConfig {
    /// Continuously try to read a string from a file.
    async fn try_read_to_string(file_path: &Path) -> String {
        loop {
            match tokio::fs::read_to_string(&file_path).await {
                Ok(content) => {
                    if SocketAddr::from_str(content.trim().trim_start_matches("http://")).is_err() {
                        continue;
                    }
                    return content;
                }
                Err(e) => {
                    if e.kind() == tokio::io::ErrorKind::NotFound {
                        sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                }
            };
        }
    }

    /// Fetches the node's addresses.
    pub async fn load_addrs(&mut self) -> Result<()> {
        let mut net_addr = String::new();
        let mut rest_addr = String::new();

        timeout(LOAD_ADDR_TIMEOUT_SECS, async {
            let net_addr_path = self.path.join(NET_ADDR_FILE);
            let rest_addr_path = self.path.join(REST_ADDR_FILE);

            net_addr = NodeConfig::try_read_to_string(&net_addr_path).await;
            rest_addr = NodeConfig::try_read_to_string(&rest_addr_path).await;
        })
        .await
        .expect("Couldn't fetch node's addresses");

        self.net_addr = Some(
            SocketAddr::from_str(
                net_addr
                    .trim()
                    .strip_prefix("http://")
                    .expect("The http prefix is missing."),
            )
            .expect("Couldn't create the network socket address."),
        );
        self.rest_api_addr = Some(
            SocketAddr::from_str(rest_addr.trim())
                .expect("Couldn't create the REST API socket address."),
        );
        Ok(())
    }
}

/// Convenience struct for reading Ziggurat's configuration file.
#[derive(Deserialize)]
struct ConfigFile {
    /// The absolute path of where to run the start command.
    path: PathBuf,
    /// The command to start the node.
    start_command: String,
}

/// The node metadata read from Ziggurat's configuration file.
#[derive(Debug, Clone)]
pub struct NodeMetaData {
    /// The absolute path of where to run the start command.
    pub path: PathBuf,
    /// The command to start the node.
    pub start_command: OsString,
    /// The arguments to the start command of the node.
    pub start_args: Vec<OsString>,
}

impl NodeMetaData {
    pub fn new(setup_path: &Path) -> Result<NodeMetaData> {
        // Read Ziggurat's configuration file.
        let path = setup_path.join(SETUP_CONFIG);
        let config_string = fs::read_to_string(path)?;
        let config_file: ConfigFile = toml::from_str(&config_string)?;

        // Read the args (which includes the start command at index 0).
        let args_from = |command: &str| -> Vec<OsString> {
            command.split_whitespace().map(OsString::from).collect()
        };

        // Separate the start command from the args list.
        let mut start_args = args_from(&config_file.start_command);
        let start_command = start_args.remove(0);

        Ok(Self {
            path: config_file.path,
            start_command,
            start_args,
        })
    }
}
