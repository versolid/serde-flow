use std::path::{Path, PathBuf};

use super::{AsyncResult, FlowResult};
use crate::encoder::zerocopy::Reader;

pub trait File<T>
where
    T: rkyv::Archive + rkyv::Serialize<crate::encoder::zerocopy::DefaultSerializer>,
    T::Archived: for<'b> rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'b>>,
{
    fn load_from_path(path: &Path) -> FlowResult<Reader<T>>;
    fn save_to_path(&self, path: &Path) -> FlowResult<()>;
}

pub trait FileMigrate<T>
where
    T: rkyv::Archive + rkyv::Serialize<crate::encoder::zerocopy::DefaultSerializer>,
    T::Archived: for<'b> rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'b>>,
{
    fn load_and_migrate(path: &Path) -> FlowResult<Reader<T>>;
    fn migrate(path: &Path) -> FlowResult<()>;
}

pub trait FileAsync<T>
where
    T: rkyv::Archive + rkyv::Serialize<crate::encoder::zerocopy::DefaultSerializer>,
    T::Archived: for<'b> rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'b>>,
{
    fn load_from_path_async<'a>(path: PathBuf) -> AsyncResult<'a, Reader<'a, T>>;
    fn save_to_path_async(&self, path: PathBuf) -> AsyncResult<()>;
}

pub trait FileMigrateAsync<T>
where
    T: rkyv::Archive + rkyv::Serialize<crate::encoder::zerocopy::DefaultSerializer>,
    T::Archived: for<'b> rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'b>>,
{
    fn load_and_migrate_async(path: &Path) -> AsyncResult<Reader<T>>;
    fn migrate_async(path: &Path) -> AsyncResult<()>;
}

pub trait Bytes<T>
where
    T: rkyv::Archive + rkyv::Serialize<crate::encoder::zerocopy::DefaultSerializer>,
    T::Archived: for<'b> rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'b>>,
{
    fn encode(&self) -> FlowResult<Vec<u8>>;
    fn decode(bytes: Vec<u8>) -> FlowResult<Reader<'static, T>>;
}
