mod json;
pub use json::JsonEncoder;

#[cfg(feature = "yaml")]
mod yaml;
#[cfg(feature = "yaml")]
pub use yaml::YamlEncoder;

#[cfg(feature = "protobuf")]
mod protobuf;
#[cfg(feature = "protobuf")]
pub use protobuf::ProtobufEncoder;

use std::fmt;

#[derive(Debug)]
pub struct EncodeError(pub String);

impl fmt::Display for EncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for EncodeError {}

pub trait Encoder<T> {
    fn encode(&self, value: &T) -> Result<Vec<u8>, EncodeError>;
    fn decode(&self, data: &[u8]) -> Result<T, EncodeError>;
}
