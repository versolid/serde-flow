//! # Serde Flow - Migration Framework
//!
//! **This Rust library helps manage changes to how data is saved in files while making it easy to move from one version of the file format to another, and keeping things working with older versions by versioning how the data is saved.**
//!
//! ## Key Features
//!
//! - Versioning of serialize/deserialize entities
//! - Migration of serialized bytes
//! - Async migration
//! - Zerocopy deserialization
//! - Data Integrity Verification
//!
//! ## Modes of Operation
//!
//! Serde-Flow has two ways of working: File and Bytes. The File mode is good for when you want to work with files directly on your computer, while the Bytes mode is better for when you're working with data in your computer's memory. You can use them both at the same time if you need to.
//!
//! - File Mode
//! - Bytes Mode
//! - Both (use them together``#[flow(variant = 1, file, bytes)]``)
//!
//! #### File Mode
//!
//! The File mode helps you work with files on your computer. It can read data from a certain place on your computer, save and load information in files automatically, and can also help update files to the newest version. To use this mode, add a special instruction called ``#[flow(file)]`` above your code. This tells Serde-Flow to treat that part of your code as working with files.
//!
//! #####  Example Usage:
//!
//! ```rust
//! use serde::{Serialize, Deserialize};
//! use serde_flow::Flow;
//! use serde_flow::encoder::bincode;
//! use serde_flow::flow::{File, FileMigrate};
//! # use tempfile::tempdir;
//!
//! #[derive(Serialize, Deserialize, Flow)]
//! #[flow(variant = 1, file)]
//! struct MyStruct {
//!     // Your struct fields here
//!     field: String
//! }
//! # fn main() {
//! # let temp_dir = tempdir().unwrap();
//! # let path_buf = temp_dir.path().to_path_buf().join("car");
//! # let path = path_buf.as_path();
//! let object = MyStruct { field: "Something".to_string() };
//!
//! // save your object to path
//! object.save_to_path::<bincode::Encoder>(path).unwrap();
//! // load your object from the path
//! let object = MyStruct::load_from_path::<bincode::Encoder>(path).unwrap();
//! # }
//! ```
//!
//! ### 2. Bytes Mode
//!
//! The Bytes mode is for when you're working with computer memory instead of files. It's good for things like sending information between computers or saving data in a special way. To use this mode, add another special instruction called #[flow(bytes)] above your code. This tells Serde-Flow to treat that part of your code as working with computer memory.
//!
//! #### Example Usage:
//!
//! ```rust
//! use serde::{Serialize, Deserialize};
//! use serde_flow::Flow;
//! use serde_flow::encoder::bincode;
//! use serde_flow::flow::{Bytes};
//!
//! #[derive(Serialize, Deserialize, Flow)]
//! #[flow(variant = 1, bytes)]
//! struct MyStruct {
//!     // Your struct fields here
//!     field: String
//! }
//! # fn main() {
//! let object = MyStruct { field: "Something".to_string() };
//! // encode the object into bytes
//! let bytes = object.encode::<bincode::Encoder>().unwrap();
//! // decode the object from bytes
//! let object = MyStruct::decode::<bincode::Encoder>(&bytes).unwrap();
//! # }
//! ```
//! # Migrations
//!
//! To use *migrations*, you need to tell the program about different ways your data can be saved (called "variants"). Migrations works well with text formats, like JSON. To do this, add a special instruction called ``[#[variants(StructA, StructB, ...)]`` and list all the ways your data can be saved.
//!
//! ## Setup ``File Mode`` for Serde serialization
//!
//! Implements basic serde struct for serializing and deserializing User with version 1.
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use serde_flow::{Flow};
//!
//! #[derive(Flow, Serialize, Deserialize)]
//! #[flow(variant = 1, file)]
//! struct User {
//!     name: String
//! }
//! ```
//!
//! ## Setup *async* ``File Mode`` for Serde serialization
//!
//! To read and write files using Serde asynchronously, add the ``nonblocking`` option to the ``file`` attribute in the flow instruction.
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use serde_flow::{Flow};
//!
//! #[derive(Flow, Serialize, Deserialize)]
//! #[flow(variant = 1, file(nonblocking))]
//! struct User {
//!     name: String
//! }
//! ```
//!
//! ## Zerocopy
//!
//! To deserialize files or bytes using SerdeFlow without copying data, use the "rkyv" library in your project. Add three special instructions called ``rkyv::Serialize``, ``rkyv::Deserialize``, and ``rkyv::Archive`` to your code. This can also work asynchronously if needed.
//!
//! ```rust
//! use rkyv::{Archive, Deserialize, Serialize};
//! use serde_flow::{Flow};
//!
//! #[derive(Flow, Archive, Serialize, Deserialize)]
//! #[flow(variant = 1, file, zerocopy)]
//! #[archive(check_bytes)]
//! struct User {
//!     name: String
//! }
//! ```
//!
//! ## Verify Write
//!
//! To make sure your files are saved correctly, you can use verification. Serde-Flow will save the information in a file and then check if the data is the same when it's loaded back. To use this, add the special instruction called ``verify_write`` to your code.
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use serde_flow::{Flow};
//!
//! #[derive(Flow, Serialize, Deserialize)]
//! #[flow(variant = 1, file(verify_write))]
//! struct User {
//!     name: String
//! }
//! ```
//!
//! ## Usage
//!
//! You have to include some imports to use migrations.
//!
//! ### Blocking
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use serde_flow::{encoder::bincode, flow::File, flow::FileMigrate, Flow};
//! use serde_flow::flow::FlowResult;
//! # use tempfile::tempdir;
//!
//! #[derive(Flow, Serialize, Deserialize)]
//! #[flow(variant = 2, file(verify_write))]
//! #[variants(UserV1)]
//! struct User {
//!     name: String
//! }
//!
//! #[derive(Flow, Serialize, Deserialize)]
//! #[flow(variant = 1, file(verify_write))]
//! struct UserV1 {
//!     value: u16
//! }
//! impl From<UserV1> for User {
//!     fn from(object: UserV1) -> User {
//!         User { name: object.value.to_string() }
//!     }
//! }
//! # fn main() -> FlowResult<()> {
//! # let temp_dir = tempdir().unwrap();
//! # let path_buf = temp_dir.path().to_path_buf().join("car");
//! # let path = path_buf.as_path();
//! // create an old user
//! let user = UserV1 { value: 123 };
//! user.save_to_path::<bincode::Encoder>(path)?;
//! // loading without updating stored User
//! let user = User::load_from_path::<bincode::Encoder>(path)?;
//! // loading with updating stored User
//! let user = User::load_and_migrate::<bincode::Encoder>(path)?;
//! // only migrating stored User
//! User::migrate::<bincode::Encoder>(path)?;
//!
//! # Ok(())
//! # }
//! ```
//!
//! ### Nonblocking
//!
//! ```rust
//! use serde::{Deserialize, Serialize};
//! use serde_flow::{encoder::bincode, flow::FileAsync, flow::FileMigrateAsync, Flow};
//! use serde_flow::flow::FlowResult;
//! # use tempfile::tempdir;
//! # #[derive(Flow, Serialize, Deserialize)]
//! # #[flow(variant = 2, file(nonblocking))]
//! # #[variants(UserV1)]
//! # struct User {
//! #    name: String
//! # }
//! # #[derive(Flow, Serialize, Deserialize)]
//! # #[flow(variant = 1, file(nonblocking))]
//! # struct UserV1 {
//! #    value: u16
//! # }
//! # impl From<UserV1> for User {
//! #    fn from(object: UserV1) -> User {
//! #        User { name: object.value.to_string() }
//! #    }
//! # }
//! # #[tokio::main]
//! # async fn main() -> FlowResult<()> {
//! # let temp_dir = tempdir().unwrap();
//! # let path_buf = temp_dir.path().to_path_buf().join("car");
//! # let path = path_buf.as_path();
//!
//! // create an old user
//! let user = UserV1 { value: 123 };
//! user.save_to_path_async::<bincode::Encoder>(path).await?;
//!
//! let user = User::load_from_path_async::<bincode::Encoder>(path).await?;
//! let user = User::load_and_migrate_async::<bincode::Encoder>(path).await?;
//! User::migrate_async::<bincode::Encoder>(path).await?;
//! # Ok(())
//! # }
//! ```
//!
//! ### Zerocopy
//!
//! This function makes a ``Reader<T>`` that can read information from files. Also, if you're using zero-copy, the ``load_from_path`` method updates the saved file automatically when it reads the information. The ``save_to_path`` method is the save.
//!
//! ```rust
//! use serde_flow::{flow::zerocopy::{File, FileMigrate}, Flow};
//! use rkyv::{Archive, Serialize, Deserialize};
//! # use tempfile::tempdir;
//!
//! #[derive(Flow, Archive, Serialize, Deserialize)]
//! #[archive(check_bytes)]
//! #[flow(variant = 1, file, zerocopy)]
//! struct User {
//!     name: String
//! }
//!
//! # fn main() {
//! # let temp_dir = tempdir().unwrap();
//! # let path_buf = temp_dir.path().to_path_buf().join("car");
//! # let path = path_buf.as_path();
//! # let user = User { name: "Jan Janssen".to_string() };
//! # user.save_to_path(path).unwrap();
//! // Reader<User>
//! let user = User::load_from_path(path).unwrap();
//! # }
//! ```
//!
//! #### Reader
//!
//! With the ``Reader<T>`` trait, you can do two things: map exact bytes of the loaded file into immutable object (called "archive") or decode and copy information from a loaded file (called "deserialize"). The ``archive`` method uses zero-copy, while ``deserialize`` doesn't use it.
//!
//! ```rust
//! use serde_flow::{flow::zerocopy::File, Flow};
//! use rkyv::{Archive, Serialize, Deserialize};
//! # use serde_flow::flow::FlowResult;
//! # use tempfile::tempdir;
//!
//! #[derive(Flow, Archive, Serialize, Deserialize)]
//! #[archive(check_bytes)]
//! #[flow(variant = 1, file, zerocopy)]
//! struct User {
//!     name: String
//! }
//! # fn main() -> FlowResult<()> {
//! # let temp_dir = tempdir().unwrap();
//! # let path_buf = temp_dir.path().to_path_buf().join("car");
//! # let path = path_buf.as_path();
//! # let user = User { name: "Jan Janssen".to_string() };
//! # let _ = user.save_to_path(path)?;
//! let user_reader = User::load_from_path(path)?;
//! let user_archived = user_reader.archive()?;
//!
//! assert_eq!(user_archived.name, "Jan Janssen".to_string());
//! # Ok(())
//! # }
//! ```
//!
//! ### Zerocopy Non-blocking
//!
//! ```rust
//! use serde_flow::{flow::zerocopy::FileAsync, Flow};
//! use rkyv::{Archive, Serialize, Deserialize};
//! # use serde_flow::flow::FlowResult;
//! # use tempfile::tempdir;
//!
//! #[derive(Flow, Archive, Serialize, Deserialize)]
//! #[archive(check_bytes)]
//! #[flow(variant = 1, file(nonblocking), zerocopy)]
//! struct User {
//!     name: String
//! }
//!
//! # #[tokio::main]
//! # async fn main() -> FlowResult<()> {
//! # let temp_dir = tempdir().unwrap();
//! # let path_buf = temp_dir.path().to_path_buf().join("car");
//! # let path = path_buf.clone();
//! # let user = User { name: "Jan Janssen".to_string() };
//! # let _ = user.save_to_path_async(path).await?;
//! # let path = path_buf.clone();
//! let user_reader = User::load_from_path_async(path).await?;
//!
//! # Ok(())
//! # }
//! ```
pub mod encoder;
pub mod error;
pub mod flow;

extern crate serde_flow_derive;
pub use serde_flow_derive::Flow;
