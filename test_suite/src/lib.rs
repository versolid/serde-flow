#[cfg(test)]
mod mod_test {
    use std::path::Path;

    // use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
    use serde::{Deserialize, Serialize};
    use serde_flow::{encoder::bincode, flow::FileFlowRunner, FileFlow, FlowVariant};

    #[derive(Serialize, Deserialize, FileFlow, FlowVariant)]
    #[variant(3)]
    #[migrations(UserV1, UserV2)]
    pub struct User {
        pub first_name: String,
        pub middle_name: String,
        pub last_name: String,
    }

    #[derive(Serialize, Deserialize, FlowVariant)]
    #[variant(1)]
    pub struct UserV1 {
        pub first_name: String,
        pub last_name: String,
    }

    #[derive(Serialize, Deserialize, FileFlow, FlowVariant)]
    #[variant(2)]
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

            let middle_name = names
                .first()
                .map(std::string::ToString::to_string)
                .unwrap_or_default();

            let last_name = names
                .get(1)
                .map(std::string::ToString::to_string)
                .unwrap_or_default();
            Self {
                first_name: value.first_name,
                middle_name,
                last_name,
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
    fn test_serialize() {
        let user_v2 = UserV2 {
            name: "John Adam Doe".to_string(),
        };

        let dir = Path::new("testdir/");
        std::fs::create_dir_all(dir).unwrap();
        let path = dir.to_path_buf().join("user");

        user_v2
            .save_to_path::<bincode::Encoder>(path.as_path())
            .unwrap();

        let user = User::load_from_path::<bincode::Encoder>(path.as_path()).unwrap();
        assert_eq!(user.first_name.as_str(), "John");
        assert_eq!(user.middle_name.as_str(), "Adam");
        assert_eq!(user.last_name.as_str(), "Doe");

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn test_serialize_rkyv() {
        // let user_v3 = UserV3 {
        //     name: "John Adam Doe".to_string(),
        // };
        //
        // let dir = Path::new("testdir/");
        // std::fs::create_dir_all(dir).unwrap();
        // let path = dir.to_path_buf().join("user");

        // let v1_bytes = rkyv::Encoder::serialize(&user_v3).unwrap();
        // std::fs::write(&path, v1_bytes).unwrap();

        // let user = User::load_from_path::<bincode::Encoder>(path.as_path()).unwrap();
        // assert_eq!(user.first_name.as_str(), "John");
        // assert_eq!(user.middle_name.as_str(), "Adam");
        // assert_eq!(user.last_name.as_str(), "Doe");
        //
        // std::fs::remove_dir_all(dir).unwrap();
    }
}
