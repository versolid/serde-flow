use crate::error::SerdeFlowError;

#[cfg(feature = "bincode")]
pub mod bincode;
pub mod rkyv;

pub trait FlowEncoder {
    fn serialize<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, SerdeFlowError>;
    fn deserialize<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, SerdeFlowError>;
}

pub type DefaultSerializer = ::rkyv::ser::serializers::AllocSerializer<4096>;
pub trait FlowZeroCopyEncoder {
    fn serialize<T: ::rkyv::Archive>(value: T) -> Result<Vec<u8>, SerdeFlowError>
    where
        T: ::rkyv::Serialize<DefaultSerializer>;

    fn deserialize<'a, T>(bytes: &'a [u8]) -> Result<&'a T::Archived, SerdeFlowError>
    where
        T: ::rkyv::Archive,
        T::Archived:
            for<'b> ::rkyv::CheckBytes<::rkyv::validation::validators::DefaultValidator<'b>>;
}
