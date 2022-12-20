//! An implementation of the Algorand network protocol types and messages.

pub mod codecs;
pub mod constants;
pub mod handshake;
mod reading;
mod writing;

macro_rules! invalid_data {
    ($msg: expr) => {
        std::io::Error::new(std::io::ErrorKind::InvalidData, $msg)
    };
}

pub(crate) use invalid_data;
