//! Utilities for setting up and tearing down Algorand node instances.

mod constants;
#[allow(dead_code)]
pub mod kmd;
#[allow(dead_code)]
pub mod node;
mod node_meta_data;

use std::{io, path::PathBuf};

use crate::setup::constants::{ALGORAND_WORK_DIR, ZIGGURAT_DIR};

/// Construct Ziggurat's work path for Algorand.
pub fn get_algorand_work_path() -> io::Result<PathBuf> {
    Ok(home::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "couldn't find the home directory"))?
        .join(ZIGGURAT_DIR)
        .join(ALGORAND_WORK_DIR))
}
