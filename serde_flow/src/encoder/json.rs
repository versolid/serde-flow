use super::FlowEncoder;
use crate::error::SerdeFlowError;
use serde::{de::DeserializeOwned, Serialize};

pub struct Encoder;
impl FlowEncoder for Encoder {
    fn serialize<T: Serialize>(value: &T) -> Result<Vec<u8>, SerdeFlowError> {
        let json_string =
            serde_json::to_string(value).map_err(|_| SerdeFlowError::EncodingFailed)?;
        println!("Json\n{json_string}");
        Ok(json_string.as_bytes().to_vec())
    }
    fn deserialize<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, SerdeFlowError> {
        let object: T = serde_json::from_slice(bytes).map_err(|_| SerdeFlowError::ParsingFailed)?;
        Ok(object)
    }
}
