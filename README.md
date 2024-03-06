# Serde Flow
The `serde_flow` is a Rust library that helps manage changes in serialized data formats during software development. As data structures evolve over time, the library makes it easy to migrate files and maintain compatibility across different versions of your application's data structures, similar to how database migration tools handle schema evolution.

# Usage
```rust

// the main file
#[derive(Serialize, Deserialize, FileFlow, FlowVariant)]
#[variant(3)] // version of the main file
#[migrations(UserV1, UserV2)]
pub struct User {
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
}

// previous variant
#[derive(Serialize, Deserialize, FileFlow, FlowVariant)]
#[variant(2)]
pub struct UserV1 {
    pub first_name: String,
    pub last_name: String,
}

// the first version of the User entity
#[derive(Serialize, Deserialize, FileFlow, FlowVariant)]
#[variant(1)]
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
