Serde Flow - Migration Framework
==================================

[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/versolid/serde-flow/ci.yml?branch=main&style=for-the-badge" height="20">](https://github.com/versolid/serde-flow/actions?query=branch%3Amain)

**serde_flow is a Rust library that simplifies managing changes to serialized data formats during software development, enabling seamless file migration and maintaining version compatibility by supporting backward compatibility through versioning of serialized data.**

## Features
1. **Versioning of serialize/deserialize entities** 
2. **Migration of serialized bytes** 
3. **Async migration** 
4. **Zerocopy deserialization** 
5. **Data Integrity Verification**

## Main Concepts
Serde Flow primarily consists of three major components:
1. `#[derive(Flow)]`: To utilize Serde Flow, you must annotate your class with `serde_flow::Flow`. This annotation serves as a signal to the library that the class is eligible for data migration.
2. `#[flow(variant = N)]`: Utilize this annotation to specify the version of the entity. Simply replace N with a `u16` number that represents the version. This helps in managing different versions of your data structures efficiently.
    - ``variant = N`` - defines version of the struct with number(u16) N
    - ``file(option1, option2)`` or ``file`` - implements loading from file
        - ``blocking`` - (default) - normal blocking IO loading and deserialization
        - ``nonblocking`` - async IO loading and deserialization (it's possible to use blockin and nonblocking at the same time)
        - ``verify_write`` - verifies writted data by calculating checksum
    - ``zerocopy`` - Uses rkyv to perfome zerocopy deserialization.
3. `#[variants(StructA, StructB, ...)]` (*Optional*): This annotation is optional but highly recommended for comprehensive data migration management. Here, you list the structs that are essential for migrating into the struct highlighted with this annotation. *To ensure, you need to implement `From<VariantStruct>` for all structs listed in `#[variants(..)]`*.

## 🛠️ Getting Started
```toml
[dependencies]
serde_flow = "1.1.0"
```
Imagine you have a `User` struct that has evolved over time through versions `UserV1` -> `UserV2` -> `User` (current), while the previous versions `UserV1` and `UserV2` still exist elsewhere. To manage this scenario effectively, follow these steps:
1. **Versioning:** Start by setting proper versioning from the beginning. The initial creation of the user should be annotated with `#[flow(variant = 1)]`.
2. **Incremental Versioning:** As you iterate and create subsequent versions, ensure to increment the version in the annotations, such as `#[flow(variant = 2)]` for the next version.
3. **Migration Preparation:** When you're ready to migrate to a new version, add the `#[variants(UserV1, UserV2)]` annotation to the main User struct. It's essential to include all previous variants that you intend to migrate from.
4. **Implementation Scope:** Variants must be implemented in the same file as the main variant to ensure proper management and accessibility during migration processes.

By adhering to these guidelines, you can effectively manage the evolution of your data structures while ensuring seamless migration across versions.

### Setup - add attributes to your structs
```rust
use serde_flow::{Flow};
use serde::{Deserialize, Serialize};

// The last (variant 3) of the User
#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 3, file)]
#[variants(UserV1, UserV2)]
pub struct User {
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
}

// previous variant
#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 2)]
pub struct UserV2 {
    pub first_name: String,
    pub last_name: String,
}

// the first variant of the User entity
#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 1)]
pub struct UserV1 {
    pub name: String,
}

// Migration from UserV1 and UserV2 for User
impl From<UserV1> for User { /* migration */ }
impl From<UserV2> for User { /* migration */ }
```

### Usage
```rust
use serde_flow::{encoder::bincode, flow::File, flow::FileMigrate, Flow};

// create an old struct UserV2
let user_v2 = UserV2 {
    name: "John Adam Doe".to_string(),
};

// serialize to the disk
user_v2
    .save_to_path::<bincode::Encoder>(&Path::new("/path/to/user"))
    .unwrap();

// deserialize as User
let user = User::load_from_path::<bincode::Encoder>(path.as_path()).unwrap();

// Migrate and load (loads, migrates, saves, and returns new entity)
let user User::load_and_migrate::<bincode::Encoder>(path.as_path()).unwrap();

// Just migrate (loads, migrates, and saves new entity)
User::migrate::<bincode::Encoder>(path.as_path()).unwrap();
```

## 📜 License
Serde-flow is open-source software, freely available under the MIT License.
