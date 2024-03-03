use super::FlowZeroCopyEncoder;
use crate::error::SerdeFlowError;
use rkyv::{ser::Serializer, Archive, Serialize};

pub struct Encoder;

impl FlowZeroCopyEncoder for Encoder {
    fn serialize<T: Archive>(value: T) -> Result<Vec<u8>, crate::error::SerdeFlowError>
    where
        T: Serialize<super::DefaultSerializer>,
    {
        let mut serializer = super::DefaultSerializer::default();
        let _ = serializer.serialize_value(&value).unwrap();
        let bytes = serializer.into_serializer().into_inner().into_vec();
        Ok(bytes)
    }

    fn deserialize<'a, T>(bytes: &'a [u8]) -> Result<&'a T::Archived, SerdeFlowError>
    where
        T: ::rkyv::Archive,
        T::Archived:
            for<'b> ::rkyv::CheckBytes<::rkyv::validation::validators::DefaultValidator<'b>>,
    {
        Ok(rkyv::check_archived_root::<T>(bytes).unwrap())
    }
}
