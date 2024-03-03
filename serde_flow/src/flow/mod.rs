use serde::{de::DeserializeOwned, Serialize};

use crate::encoder::FlowEncoder;
use crate::error::SerdeFlowError;
use std::{future::Future, path::Path, pin::Pin};

pub type FlowResult<T> = Result<T, SerdeFlowError>;
pub type AsyncResult<T> = Pin<Box<dyn Future<Output = FlowResult<T>>>>;

pub trait FileFlowRunner<T: Serialize + DeserializeOwned> {
    fn load_from_path<E: FlowEncoder>(path: &Path) -> FlowResult<T>;
    fn save_to_path<E: FlowEncoder>(&self, path: &Path) -> FlowResult<()>;
}

pub trait FileFlowMigrateRunner<T: Serialize + DeserializeOwned + FileFlowRunner<T>> {
    fn load_and_migrate<E: FlowEncoder>(path: &Path) -> FlowResult<T>;
    fn migrate<E: FlowEncoder>(path: &Path) -> FlowResult<()>;
}

pub trait FileFlowAsyncRunner<T> {
    fn load_from_path<E: FlowEncoder>(path: &Path) -> AsyncResult<T>;
    fn save_to_path<E: FlowEncoder>(&self, path: &Path) -> AsyncResult<T>;
}

pub trait FileFlowAsyncMigrateRunner<T: FileFlowAsyncRunner<T>> {
    fn load_and_migrate<E: FlowEncoder>(path: &Path) -> FlowResult<T>;
    fn migrate<E: FlowEncoder>(path: &Path) -> AsyncResult<()>;
}

pub trait BytesFlowRunner<T> {
    fn encode_from_bytes(&self) -> FlowResult<T>;
    fn decode_from_bytes(bytes: &[u8]) -> FlowResult<T>;
}
