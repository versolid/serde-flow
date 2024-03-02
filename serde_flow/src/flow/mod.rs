use serde::{de::DeserializeOwned, Serialize};

use crate::encoder::FlowEncoder;
use crate::error::SerdeFlowError;
use std::{future::Future, path::Path, pin::Pin};

pub type FlowResult<T> = Result<T, SerdeFlowError>;
pub type AsyncResult<T> = Pin<Box<dyn Future<Output = Result<T, std::io::Error>>>>;

pub trait FileFlow<T: Serialize + DeserializeOwned> {
    fn load_from_path<E: FlowEncoder>(path: &Path) -> FlowResult<T>;
    fn save_on_path<E: FlowEncoder>(&self, path: &Path) -> FlowResult<()>;
}

pub trait FileFlowMigrate<T: Serialize + DeserializeOwned + FileFlow<T>> {
    fn load_and_migrate<E: FlowEncoder>(path: &Path) -> FlowResult<T>;
    fn migrate<E: FlowEncoder>(path: &Path) -> FlowResult<()>;
}

pub trait FileFlowAsync<T> {
    fn load_from_path<E: FlowEncoder>(path: &Path) -> AsyncResult<T>;
    fn save_on_path<E: FlowEncoder>(&self, path: &Path) -> AsyncResult<T>;
}

pub trait FileFlowAsyncMigrate<T: FileFlowAsync<T>> {
    fn load_and_migrate<E: FlowEncoder>(path: &Path) -> Result<T, std::io::Error>;
    fn migrate<E: FlowEncoder>(path: &Path) -> AsyncResult<()>;
}

pub trait FlowBytes<T> {
    fn encode_from_bytes(&self) -> Result<T, std::io::Error>;
    fn decode_from_bytes(bytes: &[u8]) -> Result<T, std::io::Error>;
}
