//! The Key Management Daemon (kmd) is a low level wallet and key management
//! tool. It works in conjunction with algod and goal to keep secrets safe.

pub mod config;
pub mod constants;

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

use crate::setup::{
    config::NodeMetaData,
    constants::ALGORAND_SETUP_DIR,
    get_algorand_work_path,
    kmd::{
        config::KmdConfig,
        constants::{CONNECTION_TIMEOUT, REST_ADDR_FILE},
    },
    node::ChildExitCode,
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

        Kmd::wait_for_start(self.conf.rest_api_addr.unwrap()).await;
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
}

impl Drop for Kmd {
    fn drop(&mut self) {
        // We should avoid a panic.
        if let Err(e) = self.stop() {
            eprintln!("Failed to stop the kmd instance: {}", e);
        }
    }
}
