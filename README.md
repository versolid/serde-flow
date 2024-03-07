# Serde Flow
The `serde_flow` is a Rust library that helps manage changes in serialized data formats during software development. As data structures evolve over time, the library makes it easy to migrate files and maintain compatibility across different versions of your application's data structures, similar to how database migration tools handle schema evolution.

```toml
[dependencies]
serde_flow = "1.0.0"
```

# Basics
Serde Flow primarily consists of three major components:
1. `#[derive(Flow)]`: To utilize Serde Flow, you must annotate your class with `serde_flow::Flow`. This annotation serves as a signal to the library that the class is eligible for data migration.
2. `#[flow(variant = N)]`: Utilize this annotation to specify the version of the entity. Simply replace N with a `u16` number that represents the version. This helps in managing different versions of your data structures efficiently.
3. `#[variants(StructA, StructB, ...)]` (*Optional*): This annotation is optional but highly recommended for comprehensive data migration management. Here, you list the structs that are essential for migrating into the struct highlighted with this annotation. *To ensure, you need to implement `From<VariantStruct>` for all structs listed in `#[variants(..)]`*.

# Usage
```rust

// the main file
#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 2, file)]
#[variants(UserV1, UserV2)]
pub struct User {
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
}

// previous variant
#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 2)]
pub struct UserV1 {
    pub first_name: String,
    pub last_name: String,
}

// the first version of the User entity
#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 1)]
pub struct UserV2 {
    pub name: String,
}

// provide a proper mappings from old file to the new one
impl From<UserV1> for User {
    fn from(value: UserV1) -> Self {
        //... migration
    }
}

impl From<UserV2> for User {
    fn from(value: UserV2) -> Self {
        //... migration
    }
}
```

## Basic Serialization & Deserialization
```rust
// create an old struct
let user_v2 = UserV2 {
    name: "John Adam Doe".to_string(),
};

// store on the disk
user_v2
    .save_to_path::<bincode::Encoder>(&Path::new("/path/to/user"))
    .unwrap();

// during the loading, it will be migrated to the new User struct
let user = User::load_from_path::<bincode::Encoder>(path.as_path()).unwrap();
```
