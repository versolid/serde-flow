use std::path::Path;

use super::FlowResult;
use crate::encoder::zerocopy::{DefaultSerializer, Reader};

pub trait File<T>
where
    T: rkyv::Archive + rkyv::Serialize<crate::encoder::zerocopy::DefaultSerializer>,
    T::Archived: for<'b> rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'b>>,
{
    fn from_path(path: &Path) -> FlowResult<Reader<T>>;
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
