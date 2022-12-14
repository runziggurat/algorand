//! High level APIs and types for node setup and teardown.

mod config;
mod constants;
pub mod rest_api;

use std::{
    collections::HashSet,
    fs, io,
    net::SocketAddr,
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
};

use anyhow::Result;
use fs_extra::dir;
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    time::{sleep, Duration},
};

use crate::setup::{
    constants::{ALGORAND_SETUP_DIR, PRIVATE_NETWORK_DIR},
    get_algorand_work_path,
    node::{
        config::NodeConfig,
        constants::{CONNECTION_TIMEOUT, NET_ADDR_FILE, NODE_DIR, REST_ADDR_FILE},
    },
    node_meta_data::NodeMetaData,
};

pub enum ChildExitCode {
    Success,
    ErrorCode(Option<i32>),
}

pub struct NodeBuilder {
    /// Node's startup configuration.
    conf: NodeConfig,
    /// Node's process metadata read from Ziggurat configuration files.
    meta: NodeMetaData,
}

impl NodeBuilder {
    /// Creates a new [NodeBuilder].
    pub fn new() -> anyhow::Result<Self> {
        let setup_path = get_algorand_work_path()?.join(ALGORAND_SETUP_DIR);

        let conf = NodeConfig::default();
        let meta = NodeMetaData::new(&setup_path)?;

        Ok(Self { conf, meta })
    }

    /// Creates a [Node] according to configuration.
    pub fn build(&self, target: &Path) -> Result<Node> {
        if !target.exists() {
            fs::create_dir_all(target)?;
        }

        // Currently we can start only the first node.
        let source = Node::get_path(0)?;

        let mut copy_options = dir::CopyOptions::new();
        copy_options.content_only = true;
        copy_options.overwrite = true;
        dir::copy(&source, target, &copy_options)?;

        // Note: we would implement dynamic node configuration here if the need occurs.

        let mut conf = self.conf.clone();
        conf.path = target.to_path_buf();

        Ok(Node {
            child: None,
            conf,
            meta: self.meta.clone(),
        })
    }

    /// Sets whether to log the node's output to Ziggurat's output stream.
    pub fn log_to_stdout(mut self, log_to_stdout: bool) -> Self {
        self.conf.log_to_stdout = log_to_stdout;
        self
    }

    /// Sets initial peers for the node.
    pub fn initial_peers<I: IntoIterator<Item = SocketAddr>>(mut self, addrs: I) -> Self {
        self.conf.initial_peers = addrs.into_iter().collect::<HashSet<SocketAddr>>();
        self
    }
}

pub struct Node {
    /// Node's process.
    child: Option<Child>,
    /// Node's startup configuration.
    conf: NodeConfig,
    /// Node's process metadata read from Ziggurat configuration files.
    meta: NodeMetaData,
}

impl Node {
    /// Creates a NodeBuilder.
    pub fn builder() -> NodeBuilder {
        NodeBuilder::new()
            .map_err(|e| format!("unable to create a builder: {:?}", e))
            .unwrap()
    }

    /// Waits the node to start responding.
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

    /// Starts the node instance.
    pub async fn start(&mut self) {
        let (stdout, stderr) = match self.conf.log_to_stdout {
            true => (Stdio::inherit(), Stdio::inherit()),
            false => (Stdio::null(), Stdio::null()),
        };

        // Specify node's data path location with the `-d` option.
        self.meta.start_args.push("-d".into());
        self.meta.start_args.push(self.conf.path.clone().into());

        if self.conf.log_to_stdout {
            // Write to stdout instead of node.log using the option '-o'.
            self.meta.start_args.push("-o".into());
        }

        if !self.conf.initial_peers.is_empty() {
            // Override phonebook with peer ip:port (or semicolon separated list: ip:port;ip:port;...)
            // with the option '-p'
            self.meta.start_args.push("-p".into());

            let mut ip_list = String::new();
            for ip in self.conf.initial_peers.iter() {
                ip_list.push_str(&format!("{};", ip));
            }
            ip_list.pop().unwrap(); // Remove a trailing ';'

            self.meta.start_args.push(ip_list.into());
        }

        let full_path = fs::canonicalize(self.meta.path.join(&self.meta.start_command)).unwrap();
        let child = Command::new(full_path)
            .current_dir(&self.meta.path)
            .args(&self.meta.start_args)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr)
            .spawn()
            .expect("node failed to start");
        self.child = Some(child);

        // Once the node is started, fetch its addresses.
        self.conf
            .load_addrs()
            .await
            .expect("couldn't load the node's addresses");

        Node::wait_for_start(self.conf.net_addr.unwrap()).await;
    }

    /// Stops the node instance.
    pub fn stop(&mut self) -> io::Result<ChildExitCode> {
        // Cannot use 'mut self' due to the Drop impl.

        // Remove address files since addresses may change if the node is restarted.
        let remove_file = |file_name| match fs::remove_file(self.conf.path.join(file_name)) {
            Err(e) if e.kind() != io::ErrorKind::NotFound => panic!("unexpected error: {:?}", e),
            _ => (),
        };
        remove_file(NET_ADDR_FILE);
        self.conf.net_addr = None;
        remove_file(REST_ADDR_FILE);
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

    /// Returns the listening network address of the node.
    /// Non-relay nodes do not have this address configured.
    pub fn net_addr(&self) -> Option<SocketAddr> {
        self.conf.net_addr
    }

    /// Returns the REST API address of the node.
    pub fn rest_api_addr(&self) -> Option<SocketAddr> {
        self.conf.rest_api_addr
    }

    fn get_path(node_dir_idx: usize) -> io::Result<PathBuf> {
        Ok(get_algorand_work_path()?
            .join(PRIVATE_NETWORK_DIR)
            .join(format!("{NODE_DIR}{node_dir_idx}")))
    }
}

impl Drop for Node {
    fn drop(&mut self) {
        // We should avoid a panic.
        if let Err(e) = self.stop() {
            eprintln!("Failed to stop the node: {}", e);
        }
    }
}

#[cfg(test)]
mod test {
    use tempfile::TempDir;

    use super::*;

    const SLEEP: Duration = Duration::from_millis(500);

    #[tokio::test]
    async fn start_stop_the_node() {
        let builder = Node::builder();
        let target = TempDir::new().expect("couldn't create a temporary directory");

        let mut node = builder
            .log_to_stdout(false)
            .build(target.path())
            .expect("unable to build the node");

        // No addresses before the node is started.
        assert!(node.rest_api_addr().is_none());
        assert!(node.net_addr().is_none());

        node.start().await;
        // Addresses are available once the node is started.
        assert!(node.rest_api_addr().is_some());
        assert!(node.net_addr().is_some());

        sleep(SLEEP).await;

        assert!(node.stop().is_ok());
        // Addresses are deleted after the node is stopped.
        assert!(node.rest_api_addr().is_none());
        assert!(node.net_addr().is_none());

        // Restart the node.
        node.start().await;
        sleep(SLEEP).await;
        // The node will be stopped via the Drop impl.
    }
}
