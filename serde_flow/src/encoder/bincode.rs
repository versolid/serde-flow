use serde::{de::DeserializeOwned, Serialize};

use crate::error::SerdeFlowError;

use super::FlowEncoder;

pub struct Encoder;
impl FlowEncoder for Encoder {
    fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, SerdeFlowError> {
        let bytes = bincode::serialize(value).map_err(|_| SerdeFlowError::ParsingFailed)?;
        Ok(bytes)
    }
    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, SerdeFlowError> {
        let object: T = bincode::deserialize(bytes).map_err(|_| SerdeFlowError::ParsingFailed)?;
        Ok(object)
    }
}
