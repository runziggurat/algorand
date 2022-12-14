//! The kmd daemon serves it's API from the `kmd.net` files found in
//! the `~/node/data` and `~/node/data/kmd-{version}` directories.
//!
//! The kmd daemons provide their API specifications here:
//! https://developer.algorand.org/docs/rest-apis/kmd/

pub(super) mod client;
pub mod message;
