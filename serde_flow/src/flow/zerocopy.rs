use std::path::Path;

use super::FlowResult;

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
pub struct FlowIdArchived {
    pub flow_id: u16,
}

pub trait FileFlowRunner<T>
where
    T: rkyv::Archive + rkyv::Serialize<crate::encoder::zerocopy::DefaultSerializer>,
    T::Archived: for<'b> rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'b>>,
{
    fn load_from_path(path: &Path) -> FlowResult<T>;
    fn save_to_path(&self, path: &Path) -> FlowResult<()>;
}
