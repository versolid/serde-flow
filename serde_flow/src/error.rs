use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerdeFlowError {
    /// Indicates that a variant for the object was not found.
    #[error("Variant not found for the object")]
    VariantNotFound,
    /// Indicates that a file for the object was not found.
    #[error("File not found the object")]
    FileNotFound,
    /// Indicates that encoding of the object failed.
    #[error("Encoding process failed")]
    EncodingFailed,
    /// Indicates that parsing of the object failed due to incorrect format or insufficient variants.
    #[error("Parsing failed due to incorrect format or insufficient variants")]
    ParsingFailed,
    /// Indicates that the format of the object is invalid.
    #[error("Invalid data format")]
    FormatInvalid,
    /// Failed to write bytes on the disk
    #[error("Failed to write bytes on the disk")]
    FailedToWrite,
    /// Indicates an undefined error.
    #[error("An undefined error occurred")]
    Undefined,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
