use std::error::Error;
use std::{fmt, io};
use thiserror::Error;
pub mod rithmic_proto_objects;
pub mod api_client;
pub mod credentials;
pub mod test;

#[derive(Debug, Error)]
pub enum RithmicApiError {
    #[error("Server error: {0}")]
    ServerErrorDebug(String),

    #[error("Client error: {0}")]
    ClientErrorDebug(String),

    #[error("IO error occurred: {0}")]
    Io(#[from] io::Error),
}
