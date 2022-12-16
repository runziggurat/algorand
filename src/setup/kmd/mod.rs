//! The Key Management Daemon (kmd) is a low level wallet and key management
//! tool. It works in conjunction with algod and goal to keep secrets safe.

mod config;
mod constants;
pub mod rest_api;

use std::{
    fs, io,
    net::SocketAddr,
    path::Path,
    process::{Child, Command, Stdio},
};

use anyhow::anyhow;
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    time::{sleep, Duration},
};

use self::rest_api::message::{ListKeysResponse, SignTransactionResponse};
use crate::{
    protocol::codecs::msgpack::Transaction,
    setup::{
        constants::ALGORAND_SETUP_DIR,
        get_algorand_work_path,
        kmd::{
            config::KmdConfig,
            constants::{CONNECTION_TIMEOUT, REST_ADDR_FILE},
            rest_api::{
                client::ClientV1,
                message::{InitWalletHandleResponse, ListWalletsResponse},
            },
        },
        node::ChildExitCode,
        node_meta_data::NodeMetaData,
    },
};

pub struct KmdBuilder {
    /// Node's process metadata read from Ziggurat configuration files.
    meta: NodeMetaData,
}

impl KmdBuilder {
    /// Creates a new [KmdBuilder].
    pub fn new() -> anyhow::Result<Self> {
        let setup_path = get_algorand_work_path()?.join(ALGORAND_SETUP_DIR);
        let meta = NodeMetaData::new(&setup_path)?;

        Ok(Self { meta })
    }

    /// Creates a [Kmd] according to configuration.
    pub async fn build(&self, node_path: &Path) -> anyhow::Result<Kmd> {
        if !node_path.exists() {
            return Err(anyhow!("couldn't find the {:?} directory", node_path));
        }

        Ok(Kmd {
            child: None,
            conf: KmdConfig::new(node_path).await?,
            meta: self.meta.clone(),
            rest_client: None,
        })
    }
}

pub struct Kmd {
    /// Kmd's process.
    child: Option<Child>,
    /// Kmd's startup configuration.
    conf: KmdConfig,
    /// Node's process metadata read from Ziggurat configuration files.
    meta: NodeMetaData,
    /// REST API client.
    rest_client: Option<ClientV1>,
}

impl Kmd {
    /// Creates a KmdBuilder.
    pub fn builder() -> KmdBuilder {
        KmdBuilder::new()
            .map_err(|e| format!("unable to create a builder: {:?}", e))
            .unwrap()
    }

    /// Waits the kmd instance to start responding.
    async fn wait_for_start(addr: SocketAddr) {
        tokio::time::timeout(CONNECTION_TIMEOUT, async {
            const SLEEP: Duration = Duration::from_millis(100);

            loop {
                if let Ok(mut stream) = TcpStream::connect(addr).await {
                    stream.shutdown().await.unwrap();
                    break;
                }

                sleep(SLEEP).await;
            }
        })
        .await
        .unwrap();
    }

    /// Starts the kmd instance.
    pub async fn start(&mut self) {
        // Specify kmd's data path location with the `-d` option.
        self.meta.start_args.push("-d".into());
        self.meta.start_args.push(self.conf.path.clone().into());

        let full_path = fs::canonicalize(self.meta.path.join("kmd")).unwrap();
        let child = Command::new(full_path)
            .current_dir(&self.meta.path)
            .args(&self.meta.start_args)
            .stdin(Stdio::null())
            .spawn()
            .expect("the kmd instance failed to start");
        self.child = Some(child);

        // Once the kmd instance is started, fetch its address.
        self.conf
            .load_addr()
            .await
            .expect("couldn't load the kmd's address");

        // Get the API addr - unwrap will always work here (ensured by the block above).
        let rest_api_addr = self.conf.rest_api_addr.unwrap();

        Kmd::wait_for_start(rest_api_addr).await;

        self.rest_client = Some(ClientV1::new(
            rest_api_addr.to_string(),
            self.conf.token.clone(),
        ));
    }

    /// Stops the kmd instance.
    pub fn stop(&mut self) -> io::Result<ChildExitCode> {
        // Cannot use 'mut self' due to the Drop impl.

        // Remove the address file since the address may change if the kmd is restarted.
        match fs::remove_file(self.conf.path.join(REST_ADDR_FILE)) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => panic!("unexpected error: {:?}", e),
            _ => (),
        };
        self.conf.rest_api_addr = None;

        let child = match self.child {
            Some(ref mut child) => child,
            None => return Ok(ChildExitCode::Success),
        };

        match child.try_wait()? {
            None => child.kill()?,
            Some(code) => return Ok(ChildExitCode::ErrorCode(code.code())),
        }
        let exit = child.wait()?;

        match exit.code() {
            None => Ok(ChildExitCode::Success),
            Some(exit) if exit == 0 => Ok(ChildExitCode::Success),
            Some(exit) => Ok(ChildExitCode::ErrorCode(Some(exit))),
        }
    }

    /// Get the list of wallets.
    pub async fn get_wallets(&mut self) -> anyhow::Result<ListWalletsResponse> {
        if let Some(rest_client) = &self.rest_client {
            return rest_client.get_wallets().await;
        }

        Err(anyhow!("the kmd instance is not started"))
    }

    /// Unlock the wallet and return a wallet handle token that can be used for subsequent operations.
    pub async fn get_wallet_handle_token(
        &mut self,
        wallet_id: String,
        wallet_password: String,
    ) -> anyhow::Result<InitWalletHandleResponse> {
        if let Some(rest_client) = &self.rest_client {
            return rest_client
                .get_wallet_handle_token(wallet_id, wallet_password)
                .await;
        }

        Err(anyhow!("the kmd instance is not started"))
    }

    /// Get the list of public keys in the wallet.
    pub async fn get_keys(
        &mut self,
        wallet_handle_token: String,
    ) -> anyhow::Result<ListKeysResponse> {
        if let Some(rest_client) = &self.rest_client {
            return rest_client.get_keys(wallet_handle_token).await;
        }

        Err(anyhow!("the kmd instance is not started"))
    }

    /// Sign a transaction.
    pub async fn sign_transaction(
        &self,
        wallet_handle_token: String,
        wallet_password: String,
        transaction: &Transaction,
    ) -> anyhow::Result<SignTransactionResponse> {
        if let Some(rest_client) = &self.rest_client {
            return rest_client
                .sign_transaction(wallet_handle_token, wallet_password, transaction)
                .await;
        }

        Err(anyhow!("the kmd instance is not started"))
    }
}

impl Drop for Kmd {
    fn drop(&mut self) {
        // We should avoid a panic.
        if let Err(e) = self.stop() {
            eprintln!("Failed to stop the kmd instance: {}", e);
        }
    }
}
