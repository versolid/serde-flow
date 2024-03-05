use crate::error::SerdeFlowError;

#[cfg(feature = "bincode")]
pub mod bincode;
#[cfg(feature = "json")]
pub mod json;
#[cfg(feature = "zerocopy")]
pub mod zerocopy;

pub trait FlowEncoder {
    fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, SerdeFlowError>;
    fn deserialize<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, SerdeFlowError>;
}
