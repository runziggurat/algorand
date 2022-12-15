//! Utilities for setting up and tearing down Algorand node instances.

mod constants;
#[allow(dead_code)]
pub mod kmd;
#[allow(dead_code)]
pub mod node;
mod node_meta_data;

use std::{
    io,
    path::{Path, PathBuf},
};

use tokio::time::{sleep, Duration};

use crate::setup::constants::{ALGORAND_WORK_DIR, ZIGGURAT_DIR};

/// Construct Ziggurat's work path for Algorand.
fn get_algorand_work_path() -> io::Result<PathBuf> {
    Ok(home::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "couldn't find the home directory"))?
        .join(ZIGGURAT_DIR)
        .join(ALGORAND_WORK_DIR))
}

/// Continuously try to read a string from a file.
async fn try_read_to_string(file_path: &Path) -> String {
    loop {
        match tokio::fs::read_to_string(&file_path).await {
            Ok(content) => return content,
            Err(e) => {
                if e.kind() == tokio::io::ErrorKind::NotFound {
                    sleep(Duration::from_millis(100)).await;
                    continue;
                }
            }
        };
    }
}
