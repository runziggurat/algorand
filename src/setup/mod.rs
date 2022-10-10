//! Utilities for setting up and tearing down Algorand node instances.

pub mod config;
pub mod constants;
#[allow(dead_code)]
pub mod node;

use std::{io, path::PathBuf};

use crate::setup::constants::{ALGORAND_WORK_DIR, ZIGGURAT_DIR};

/// Construct Ziggurat's work path for Algorand
pub fn get_algorand_work_path() -> io::Result<PathBuf> {
    Ok(home::home_dir()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Couldn't find the home directory"))?
        .join(ZIGGURAT_DIR)
        .join(ALGORAND_WORK_DIR))
}
