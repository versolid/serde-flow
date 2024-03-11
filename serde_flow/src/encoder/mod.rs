use crate::error::SerdeFlowError;
use crc::{Crc, CRC_32_ISCSI};

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

pub const CASTAGNOLI: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);
