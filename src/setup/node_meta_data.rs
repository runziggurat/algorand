//! Utilities for reading setup configuration for the node/kmd daemons.

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use crate::setup::constants::SETUP_CONFIG;

/// Convenience struct for reading Ziggurat's configuration file.
#[derive(Deserialize)]
struct ConfigTomlFile {
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
    /// Creates a new [NodeMetaData].
    pub fn new(setup_path: &Path) -> anyhow::Result<NodeMetaData> {
        // Read Ziggurat's configuration file.
        let path = setup_path.join(SETUP_CONFIG);
        let config_string = fs::read_to_string(path)?;
        let config_toml: ConfigTomlFile = toml::from_str(&config_string)?;

        // Read the args (which includes the start command at index 0).
        let args_from = |command: &str| -> Vec<OsString> {
            command.split_whitespace().map(OsString::from).collect()
        };

        // Separate the start command from the args list.
        let mut start_args = args_from(&config_toml.start_command);
        let start_command = start_args.remove(0);

        Ok(Self {
            path: config_toml.path,
            start_command,
            start_args,
        })
    }
}
