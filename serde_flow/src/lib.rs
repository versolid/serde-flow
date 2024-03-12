//! # Serde Flow - Migration Framework
//!
//! `serde_flow` is a Rust library that simplifies managing changes in serialized data formats during software development, enabling seamless file migration and maintaining version compatibility.
//!
//! ## Key Features
//!
//! - Versioning of serialize/deserialize entities
//! - Migration of serialized bytes
//! - Async migration
//! - Zerocopy deserialization
//! - Data Integrity Verification
//!
//! ## Example
//!
//! ```rust
//! use serde_flow::{Migration, Versioned};
//!
//! #[derive(Versioned)]
//! struct MyStruct {
//!     #[version(1)]
//!     field1: String,
//!
//!     #[version(2)]
//!     field2: u32,
//! }
//!
//! // Migration from version 1 to version 2
//! impl Migration for MyStruct {
//!     fn migrate_from_previous_version(previous: Self) -> Self {
//!         // Example migration logic
//!         MyStruct {
//!             field1: previous.field1,
//!             field2: 0, // Default value for new field in version 2
//!         }
//!     }
//! }
//!
//! fn main() {
//!     let serialized_data = /* load serialized data */;
//!     
//!     // Deserialize with versioning
//!     let deserialized: MyStruct = serde_flow::deserialize(serialized_data).unwrap();
//!     
//!     // Migration to latest version
//!     let migrated: MyStruct = deserialized.migrate_to_latest_version();
//!     
//!     // Use migrated data
//!     println!("{:?}", migrated);
//! }
//! ```
pub mod encoder;
pub mod error;
pub mod flow;

extern crate serde_flow_derive;
pub use serde_flow_derive::Flow;
