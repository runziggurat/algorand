//! Utilities for node configuration.

use std::{collections::HashSet, net::SocketAddr, path::PathBuf, str::FromStr};

use tokio::time::timeout;

use crate::setup::{
    self,
    constants::LOAD_FILE_TIMEOUT_SECS,
    node::constants::{NET_ADDR_FILE, REST_ADDR_FILE},
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
    /// The initial peer set of the node.
    pub initial_peers: HashSet<SocketAddr>,
}

impl NodeConfig {
    /// Fetches the node's runtime configuration - addresses and authorization tokens.
    pub async fn load_runtime_cfg(&mut self) -> anyhow::Result<()> {
        let mut net_addr = String::new();
        let mut rest_addr = String::new();

        timeout(LOAD_FILE_TIMEOUT_SECS, async {
            let net_addr_path = self.path.join(NET_ADDR_FILE);
            let rest_addr_path = self.path.join(REST_ADDR_FILE);

            net_addr = setup::try_read_to_string(&net_addr_path).await;
            rest_addr = setup::try_read_to_string(&rest_addr_path).await;
        })
        .await
        .expect("couldn't fetch node's addresses");

        self.net_addr = Some(
            SocketAddr::from_str(
                net_addr
                    .trim()
                    .strip_prefix("http://")
                    .expect("the http prefix is missing"),
            )
            .expect("couldn't create the network socket address"),
        );
        self.rest_api_addr = Some(
            SocketAddr::from_str(rest_addr.trim())
                .expect("couldn't create the REST API socket address"),
        );

        Ok(())
    }
}
