pub mod encoder;
pub mod error;
pub mod flow;

extern crate serde_flow_derive;
pub use serde_flow_derive::{FileFlow, FileFlowZeroCopy, FlowVariant};
