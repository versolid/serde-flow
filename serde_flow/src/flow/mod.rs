use serde::{de::DeserializeOwned, Deserialize, Serialize};

use crate::encoder::FlowEncoder;
use crate::error::SerdeFlowError;
use std::{future::Future, path::Path, pin::Pin};

#[cfg(feature = "zerocopy")]
pub mod zerocopy;

pub type FlowResult<T> = Result<T, SerdeFlowError>;
pub type AsyncResult<T> = Pin<Box<dyn Future<Output = FlowResult<T>>>>;

pub trait File<T: Serialize + DeserializeOwned> {
    fn load_from_path<E: FlowEncoder>(path: &Path) -> FlowResult<T>;
    fn save_to_path<E: FlowEncoder>(&self, path: &Path) -> FlowResult<()>;
}

pub trait FileMigrate<T: Serialize + DeserializeOwned + File<T>> {
    fn load_and_migrate<E: FlowEncoder>(path: &Path) -> FlowResult<T>;
    fn migrate<E: FlowEncoder>(path: &Path) -> FlowResult<()>;
}

pub trait FileAsync<T> {
    fn load_from_path_async<E: FlowEncoder>(path: &Path) -> AsyncResult<T>;
    fn save_to_path_async<E: FlowEncoder>(&self, path: &Path) -> AsyncResult<T>;
}

pub trait FileMigrateAsync<T: FileAsync<T>> {
    fn load_and_migrate_async<E: FlowEncoder>(path: &Path) -> AsyncResult<T>;
    fn migrate_async<E: FlowEncoder>(path: &Path) -> AsyncResult<()>;
}

pub trait Bytes<T> {
    fn encode(&self) -> FlowResult<T>;
    fn decode(bytes: &[u8]) -> FlowResult<T>;
}

#[cfg(feature = "serde")]
#[derive(Serialize, Deserialize)]
pub struct FlowId {
    pub flow_id: u16,
}
