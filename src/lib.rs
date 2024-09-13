use std::error::Error;
use std::fmt;
pub mod rithmic_proto_objects;
pub mod api_client;
pub mod credentials;
pub mod test;

#[derive(Debug)]
pub enum RithmicApiError {
    ServerErrorDebug(String),
    ClientErrorDebug(String)
}
impl fmt::Display for RithmicApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl Error for RithmicApiError {}
