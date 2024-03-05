use thiserror::Error;

#[derive(Error, Debug)]
pub enum SerdeFlowError {
    #[error("Variant not found for object")]
    VariantNotFound,
    #[error("File not found for object")]
    FileNotFound,
    #[error("Encoding failed")]
    EncodingFailed,
    #[error("Failed to parse, incorrect format or not enough variants")]
    ParsingFailed,
    #[error("Invalid format")]
    FormatInvalid,
    #[error(transparent)]
    IOError(#[from] std::io::Error),
}
