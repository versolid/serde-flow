use std::{collections::HashMap, fs::OpenOptions, pin::Pin};

use memmap2::{Mmap, MmapMut};
use rkyv::{Archive, Deserialize, Serialize};
use serde_flow::encoder::zerocopy::{Encoder, Reader, ReaderMemmap};
use tempfile::tempdir;

#[derive(Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct User {
    pub first_name: String,
    pub last_name: String,
    pub amount: u16,
}

#[derive(Archive, Serialize, Deserialize)]
#[archive(check_bytes)]
pub struct UsersWithHashMap {
    pub amount: u16,
    pub values: HashMap<String, User>,
}

#[test]
fn struct_serialize_archive() {
    let user = User {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        amount: 256,
    };
    let bytes = Encoder::serialize::<User>(&user).unwrap();

    let decoder = Reader::<User>::new(bytes);
    let user_archived = decoder.archive().unwrap();

    assert_eq!(user_archived.first_name, "John".to_string());
    assert_eq!(user_archived.last_name, "Doe".to_string());
}

#[test]
fn struct_serialize_archive_memcopy() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("user");

    let user = User {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        amount: 256,
    };
    let bytes = Encoder::serialize::<User>(&user).unwrap();
    std::fs::write(path.as_path(), &bytes);

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path.as_path())
        .unwrap();
    let mmap = unsafe { MmapMut::map_mut(&file).unwrap() };

    let reader = ReaderMemmap::<User>::new(mmap);
    let user_archived = reader.archive().unwrap();

    assert_eq!(user_archived.first_name, "John".to_string());
    assert_eq!(user_archived.last_name, "Doe".to_string());
}

#[test]
fn struct_serialize_archive_memcopy_change() {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("user");

    let user = User {
        first_name: "John".to_string(),
        last_name: "Doe".to_string(),
        amount: 256,
    };
    let bytes = Encoder::serialize::<User>(&user).unwrap();
    std::fs::write(path.as_path(), &bytes);

    let mut test_map = HashMap::<String, u16>::new();
    test_map.insert("John".to_string(), 512);

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path.as_path())
        .unwrap();

    let mut mmap = unsafe { MmapMut::map_mut(&file).unwrap() };
    let mut reader_mut = ReaderMemmap::<User>::new(mmap);

    reader_mut
        .archive_mut(|user| {
            if let Some(value) = test_map.get(user.first_name.as_str()) {
                user.amount = *value;
            }
        })
        .unwrap();

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(path.as_path())
        .unwrap();

    let mut mmap = unsafe { MmapMut::map_mut(&file).unwrap() };
    let mut reader_mut = ReaderMemmap::<User>::new(mmap);
    let user_archived = reader_mut.archive().unwrap();
    assert_eq!(user_archived.first_name, "John".to_string());
    assert_eq!(user_archived.last_name, "Doe".to_string());
    assert_eq!(user_archived.amount, 512);
}

#[test]
fn struct_with_hash_map_serialize_archive() {
    let mut users = UsersWithHashMap {
        amount: 1234,
        values: HashMap::new(),
    };
    users.values.insert(
        "Somebody".to_string(),
        User {
            first_name: "John".to_string(),
            last_name: "Doe".to_string(),
            amount: 256,
        },
    );
    users.values.insert(
        "jack".to_string(),
        User {
            first_name: "Jack".to_string(),
            last_name: "Brown".to_string(),
            amount: 256,
        },
    );

    let bytes = Encoder::serialize::<UsersWithHashMap>(&users).unwrap();
    let decoder = Reader::<UsersWithHashMap>::new(bytes);
    let users_archived = decoder.archive().unwrap();

    assert_eq!(users_archived.amount, 1234);

    assert!(users_archived.values.contains_key("Somebody"));
    let somebody = users_archived.values.get("Somebody").unwrap();
    assert_eq!(somebody.first_name, "John".to_string());
    assert_eq!(somebody.last_name, "Doe".to_string());

    assert!(users_archived.values.contains_key("jack"));
    let jack = users_archived.values.get("jack").unwrap();
    assert_eq!(jack.first_name, "Jack".to_string());
    assert_eq!(jack.last_name, "Brown".to_string());
}
