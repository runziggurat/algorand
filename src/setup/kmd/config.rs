//! Utilities for kmd configuration.

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::{anyhow, Result};
use tokio::time::timeout;

use crate::setup::{
    self,
    constants::LOAD_FILE_TIMEOUT_SECS,
    kmd::constants::{KMD_DIR, REST_ADDR_FILE, TOKEN_FILE},
};

/// Startup configuration for the kmd daemon.
#[derive(Debug, Clone, Default)]
pub struct KmdConfig {
    /// The kmd's directory of the running node.
    pub path: PathBuf,
    /// The REST API socket address of the kmd instance.
    pub rest_api_addr: Option<SocketAddr>,
    /// Security token needed for the REST API authentication.
    pub token: String,
}

impl KmdConfig {
    /// Creates a new [KmdConfig].
    pub async fn new(node_path: &Path) -> anyhow::Result<Self> {
        let mut token = String::new();

        let path = node_path.join(KMD_DIR);
        if !path.exists() {
            return Err(anyhow!("couldn't find the {:?} directory", path));
        }

        timeout(LOAD_FILE_TIMEOUT_SECS, async {
            let token_path = path.join(TOKEN_FILE);

            token = setup::try_read_to_string(&token_path).await;
        })
        .await
        .expect("couldn't fetch the kmd's token");

        Ok(KmdConfig {
            path,
            rest_api_addr: None,
            token,
        })
    }

    /// Fetches the kmd's address.
    pub async fn load_addr(&mut self) -> Result<()> {
        let mut rest_addr = String::new();

        timeout(LOAD_FILE_TIMEOUT_SECS, async {
            let rest_addr_path = self.path.join(REST_ADDR_FILE);

            rest_addr = setup::try_read_to_string(&rest_addr_path).await;
        })
        .await
        .expect("couldn't fetch kmd's address");

        self.rest_api_addr = Some(
            SocketAddr::from_str(rest_addr.trim())
                .expect("couldn't create the REST API socket address"),
        );
        Ok(())
    }
}
