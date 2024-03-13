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
//! use serde_flow::flow::{File, FileMigrate}
//!
//! #[derive(Serialize, Deserialize, Flow)]
//! #[flow(variant = 1, file)]
//! struct MyStruct {
//!     // Your struct fields here
//!     field: String
//! }
//! 
//! let object = MyStruct { field: "Something".to_string() };
//! 
//! // save your object to path
//! object.save_to_file::<bincode::Encoder>(path)?;
//! // load your object from the path
//! let object = MyStruct::load_from_file::<bincode::Encoder>(path)?;
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
//! use serde_flow::flow::{Bytes}
//!
//! #[derive(Serialize, Deserialize, Flow)]
//! #[flow(variant = 1, bytes)]
//! struct MyStruct {
//!     // Your struct fields here
//!     field: String
//! }
//! 
//! 
//! let object = MyStruct { field: "Something".to_string() };
//! // encode the object into bytes
//! let bytes = object.encode::<bincode::Encoder>()?;
//! // decode the object from bytes
//! let object = MyStruct::decode::<bincode::Encoder>(&bytes)?;
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
//! #[flow(variant = 1, file(verify_write)]
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
//! use serde_flow::{encoder::bincode, flow::File, flow::FileMigrate, Flow};
//!
//! let user = User { /* ... */ };
//! user.save_to_path::<bincode::Encoder>(path.as_path())?;
//! // loading without updating stored User
//! let user = User::load_from_path::<bincode::Encoder>(path.as_path())?;
//! // loading with updating stored User
//! let user = User::load_and_migrate::<bincode::Encoder>(path.as_path())?;
//! // only migrating stored User
//! User::migrate::<bincode::Encoder>(path.as_path())?;
//! ```
//!
//! ### Nonblocking
//!
//! ```rust
//! use serde_flow::{encoder::bincode, flow::FileAsync, flow::FileMigrateAsync, Flow};
//!
//! let user = User { /* ... */ };
//! user.save_to_path_async::<bincode::Encoder>(path.as_path()).await?;
//! let user = User::load_from_path_async::<bincode::Encoder>(path.as_path()).await?;
//! let user = User::load_and_migrate::<bincode::Encoder>(path.as_path())?;
//! User::migrate::<bincode::Encoder>(path.as_path())?;
//! ```
//!
//! ### Zerocopy
//!
//! This function makes a ``Reader<T>`` that can read information from files. Also, if you're using zero-copy, the ``load_from_path`` method updates the saved file automatically when it reads the information. The ``save_to_path`` method is the save.
//!
//! ```rust
//! use serde_flow::{flow::zerocopy::{File, FileMigrate}, Flow};
//!
//! // Reader<User>
//! let user = User::load_from_path::<bincode::Encoder>(path.as_path()).await;
//! ```
//!
//! #### Reader
//!
//! With the ``Reader<T>`` trait, you can do two things: map exact bytes of the loaded file into immutable object (called "archive") or decode and copy information from a loaded file (called "deserialize"). The ``archive`` method uses zero-copy, while ``deserialize`` doesn't use it.
//!
//! ```rust
//! let user_reader = User::load_from_path(path.as_path()).unwrap();
//! let user_archived = user_reader.archive().unwrap();
//!
//! assert_eq!(user_archived.name, "John Doe".to_string());
//! ```
//!
//! ### Zerocopy Non-blocking
//!
//! ```rust
//! use serde_flow::{flow::zerocopy::{FileAsync, FileMigrateAsync}, Flow};
//! let user_reader = User::load_from_path_async(path.as_path()).await?;
//! ```
pub mod encoder;
pub mod error;
pub mod flow;

extern crate serde_flow_derive;
pub use serde_flow_derive::Flow;
