use std::cell::RefCell;

use crate::error::SerdeFlowError;
use rkyv::{ser::Serializer, Archive, Deserialize, Serialize};

pub type DefaultSerializer = rkyv::ser::serializers::AllocSerializer<4096>;

pub struct Encoder;

impl Encoder {
    /// Serializes the provided value into a byte vector.
    ///
    /// # Errors
    ///
    /// Returns a `SerdeFlowError::EncodingFailed` if the encoding process fails.
    ///
    /// ```
    pub fn serialize<T: Archive>(value: &T) -> Result<Vec<u8>, crate::error::SerdeFlowError>
    where
        T: Serialize<DefaultSerializer>,
    {
        let mut serializer = DefaultSerializer::default();
        let _ = serializer
            .serialize_value(value)
            .map_err(|_| SerdeFlowError::EncodingFailed)?;
        let bytes = serializer.into_serializer().into_inner().into_vec();
        Ok(bytes)
    }
}

pub struct Reader<'a, T: rkyv::Archive> {
    bytes: Box<[u8]>,
    archived: RefCell<Option<&'a rkyv::Archived<T>>>,
}

impl<'a, T: rkyv::Archive> Reader<'a, T>
where
    T: rkyv::Archive,
    T::Archived: for<'b> rkyv::CheckBytes<rkyv::validation::validators::DefaultValidator<'b>>,
{
    #[must_use]
    pub fn new(bytes: Vec<u8>) -> Self {
        Self {
            bytes: bytes.into_boxed_slice(),
            archived: RefCell::new(None),
        }
    }

    /// Deserializes the archived data into the original type `T`.
    ///
    /// # Errors
    ///
    /// Returns a `SerdeFlowError::ParsingFailed` if deserialization fails due to incorrect format or insufficient variants.
    ///
    /// ```
    pub fn deserialize(&'a self) -> Result<T, SerdeFlowError>
    where
        rkyv::Archived<T>: Deserialize<T, rkyv::Infallible>,
    {
        let archived = self.archive()?;
        archived
            .deserialize(&mut rkyv::Infallible)
            .map_err(|_| SerdeFlowError::ParsingFailed)
    }

    /// Archives the original data into its archived form.
    ///
    /// # Errors
    ///
    /// Returns a `SerdeFlowError::ParsingFailed` if parsing fails due to incorrect format.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_flow::SerdeFlowError;
    /// use std::result::Result;
    ///
    /// fn example_method() -> Result<(), SerdeFlowError> {
    ///     // Archive the original data
    ///     let result: Result<&'a ArchivedType, SerdeFlowError> = original_data.archive();
    ///     Ok(())
    /// }
    /// ```
    pub fn archive(&'a self) -> Result<&'a T::Archived, SerdeFlowError> {
        let borrow = self.archived.borrow();
        if borrow.is_some() {
            return borrow.ok_or(SerdeFlowError::Undefined);
        }
        drop(borrow);

        let archive: &'a T::Archived = rkyv::check_archived_root::<T>(&self.bytes)
            .map_err(|_| SerdeFlowError::ParsingFailed)?;
        self.archived.replace(Some(archive));

        let borrow = self.archived.borrow();
        borrow.ok_or(SerdeFlowError::Undefined)
    }
}
