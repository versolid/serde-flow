use serde::{Deserialize, Serialize};
use serde_flow::{
    encoder::bincode, encoder::json, flow::File, flow::FileMigrate, FileFlow, FlowVariant,
};
use tempfile::tempdir;

#[derive(Serialize, Deserialize, FileFlow, FlowVariant)]
#[variant(3)]
#[migrations(UserV1, UserV2)]
pub struct User {
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
}

#[derive(Serialize, Deserialize, FileFlow, FlowVariant)]
#[variant(2)]
pub struct UserV1 {
    pub first_name: String,
    pub last_name: String,
}

#[derive(Serialize, Deserialize, FileFlow, FlowVariant)]
#[variant(1)]
pub struct UserV2 {
    pub name: String,
}

impl From<UserV1> for User {
    fn from(value: UserV1) -> Self {
        let names: Vec<String> = value
            .last_name
            .split(' ')
            .filter(|v| !v.is_empty())
            .map(std::string::ToString::to_string)
            .collect();

        if names.len() == 2 {
            let middle_name = names
                .first()
                .map(std::string::ToString::to_string)
                .unwrap_or_default();

            let last_name = names
                .get(1)
                .map(std::string::ToString::to_string)
                .unwrap_or_default();
            return Self {
                first_name: value.first_name,
                middle_name,
                last_name,
            };
        }
        Self {
            first_name: value.first_name,
            middle_name: String::default(),
            last_name: value.last_name,
        }
    }
}

impl From<UserV2> for User {
    fn from(value: UserV2) -> Self {
        let names: Vec<String> = value
            .name
            .split(' ')
            .filter(|v| !v.is_empty())
            .map(std::string::ToString::to_string)
            .collect();

        let first_name = names
            .first()
            .map(std::string::ToString::to_string)
            .unwrap_or_default();

        let middle_name = names
            .get(1)
            .map(std::string::ToString::to_string)
            .unwrap_or_default();

        let last_name = names
            .get(2)
            .map(std::string::ToString::to_string)
            .unwrap_or_default();
        Self {
            first_name,
            middle_name,
            last_name,
        }
    }
}

#[test]
fn test_v2_load_from_path() {
    let user_v2 = UserV2 {
        name: "John Adam Doe".to_string(),
    };

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("user");

    user_v2
        .save_to_path::<bincode::Encoder>(path.as_path())
        .unwrap();

    let user = User::load_from_path::<bincode::Encoder>(path.as_path()).unwrap();
    assert_eq!(user.first_name.as_str(), "John");
    assert_eq!(user.middle_name.as_str(), "Adam");
    assert_eq!(user.last_name.as_str(), "Doe");
}

#[test]
fn test_v1_load_from_path() {
    let user_v1 = UserV1 {
        first_name: "John".to_string(),
        last_name: "Adam Doe".to_string(),
    };

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("user");

    user_v1
        .save_to_path::<bincode::Encoder>(path.as_path())
        .unwrap();

    let user = User::load_from_path::<bincode::Encoder>(path.as_path()).unwrap();
    assert_eq!(user.first_name.as_str(), "John");
    assert_eq!(user.middle_name.as_str(), "Adam");
    assert_eq!(user.last_name.as_str(), "Doe");
}

#[derive(Serialize, Deserialize, FileFlow, FlowVariant)]
#[variant(3)]
pub struct UserTestV3 {
    pub first_name: String,
    pub middle_name: String,
    pub last_name: String,
}

#[test]
fn test_v2_migrate() {
    let user_v2 = UserV2 {
        name: "John Adam Doe".to_string(),
    };

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("user");

    user_v2
        .save_to_path::<bincode::Encoder>(path.as_path())
        .unwrap();

    let err_without_migrate = UserTestV3::load_from_path::<bincode::Encoder>(path.as_path());
    assert!(err_without_migrate.is_err());

    User::migrate::<bincode::Encoder>(path.as_path()).unwrap();
    let user = UserTestV3::load_from_path::<bincode::Encoder>(path.as_path()).unwrap();

    assert_eq!(user.first_name.as_str(), "John");
    assert_eq!(user.middle_name.as_str(), "Adam");
    assert_eq!(user.last_name.as_str(), "Doe");
}

#[test]
fn test_v2_load_and_migrate() {
    let user_v2 = UserV2 {
        name: "John Adam Doe".to_string(),
    };

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("user");

    user_v2
        .save_to_path::<bincode::Encoder>(path.as_path())
        .unwrap();

    let err_without_migrate = UserTestV3::load_from_path::<bincode::Encoder>(path.as_path());
    assert!(err_without_migrate.is_err());

    let loaded_user = User::load_and_migrate::<bincode::Encoder>(path.as_path()).unwrap();
    assert_eq!(loaded_user.first_name.as_str(), "John");
    assert_eq!(loaded_user.middle_name.as_str(), "Adam");
    assert_eq!(loaded_user.last_name.as_str(), "Doe");
    let user = UserTestV3::load_from_path::<bincode::Encoder>(path.as_path()).unwrap();

    assert_eq!(user.first_name.as_str(), "John");
    assert_eq!(user.middle_name.as_str(), "Adam");
    assert_eq!(user.last_name.as_str(), "Doe");
}

#[test]
fn test_v2_load_and_migrate_json() {
    let user_v2 = UserV2 {
        name: "John Adam Doe".to_string(),
    };

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("user");

    user_v2
        .save_to_path::<json::Encoder>(path.as_path())
        .unwrap();

    let err_without_migrate = UserTestV3::load_from_path::<json::Encoder>(path.as_path());
    assert!(err_without_migrate.is_err());

    let loaded_user = User::load_and_migrate::<json::Encoder>(path.as_path()).unwrap();
    assert_eq!(loaded_user.first_name.as_str(), "John");
    assert_eq!(loaded_user.middle_name.as_str(), "Adam");
    assert_eq!(loaded_user.last_name.as_str(), "Doe");
    let user = UserTestV3::load_from_path::<json::Encoder>(path.as_path()).unwrap();

    assert_eq!(user.first_name.as_str(), "John");
    assert_eq!(user.middle_name.as_str(), "Adam");
    assert_eq!(user.last_name.as_str(), "Doe");
}
