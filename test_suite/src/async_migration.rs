use serde::{Deserialize, Serialize};
use serde_flow::error::SerdeFlowError;
use serde_flow::{encoder::bincode, flow::FileAsync, flow::FileMigrateAsync, Flow};
use tempfile::tempdir;

#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 3, file(nonblocking))]
#[variants(CarV1, CarV2)]
pub struct Car {
    pub name: String,
    pub price: String,
}

#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 2)]
pub struct CarV1 {
    pub brand: String,
    pub model: String,
    pub price: String,
}

#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 1, file(nonblocking))]
pub struct CarV2 {
    pub brand: String,
    pub model: String,
    pub price: u32,
}

impl From<CarV1> for Car {
    fn from(value: CarV1) -> Self {
        Car {
            name: format!("{} {}", value.brand, value.model),
            price: value.price,
        }
    }
}

impl From<CarV2> for Car {
    fn from(value: CarV2) -> Self {
        Car {
            name: format!("{} {}", value.brand, value.model),
            price: format!("${}", value.price),
        }
    }
}

#[tokio::test]
async fn test_save_to_path() {
    let car_v2 = CarV2 {
        brand: "BMW".to_string(),
        model: "x3".to_string(),
        price: 45000,
    };

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("car");

    let result: std::result::Result<(), _> = car_v2
        .save_to_path_async::<bincode::Encoder>(path.as_path())
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_load_from_path() -> Result<(), SerdeFlowError> {
    let car_v2 = CarV2 {
        brand: "BMW".to_string(),
        model: "x3".to_string(),
        price: 45000,
    };

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("car");

    car_v2
        .save_to_path_async::<bincode::Encoder>(path.as_path())
        .await?;

    let car = Car::load_from_path_async::<bincode::Encoder>(path.as_path()).await?;

    assert_eq!(car.name, "BMW x3".to_string());
    assert_eq!(car.price, "$45000".to_string());
    Ok(())
}

#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 4, file(nonblocking))]
pub struct CarNoMigration {
    pub name: String,
    pub price: String,
}

#[tokio::test]
async fn test_load_from_path_variant_not_found() -> Result<(), SerdeFlowError> {
    let car_v2 = CarV2 {
        brand: "BMW".to_string(),
        model: "x3".to_string(),
        price: 45000,
    };

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("car");

    car_v2
        .save_to_path_async::<bincode::Encoder>(path.as_path())
        .await?;

    let result = CarNoMigration::load_from_path_async::<bincode::Encoder>(path.as_path()).await;
    let Err(SerdeFlowError::VariantNotFound) = result else {
        panic!("load_from_path no variant, must return VariantNotFound");
    };
    Ok(())
}

#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 4, file(nonblocking))]
#[variants(CarV1)]
pub struct CarWithMigration {
    pub name: String,
    pub price: String,
}

impl From<CarV1> for CarWithMigration {
    fn from(value: CarV1) -> Self {
        CarWithMigration {
            name: format!("{} {}", value.brand, value.model),
            price: value.price,
        }
    }
}

#[tokio::test]
async fn test_load_from_path_insufficient_variants() -> Result<(), SerdeFlowError> {
    let car_v2 = CarV2 {
        brand: "BMW".to_string(),
        model: "x3".to_string(),
        price: 45000,
    };

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("car");

    car_v2
        .save_to_path_async::<bincode::Encoder>(path.as_path())
        .await?;

    let result = CarWithMigration::load_from_path_async::<bincode::Encoder>(path.as_path()).await;
    let Err(SerdeFlowError::VariantNotFound) = result else {
        panic!("load_from_path no variant, must return VariantNotFound");
    };
    Ok(())
}

#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 3, file(nonblocking))]
pub struct CarTest {
    pub name: String,
    pub price: String,
}

#[tokio::test]
async fn test_migrate() -> Result<(), SerdeFlowError> {
    let car_v2 = CarV2 {
        brand: "BMW".to_string(),
        model: "x3".to_string(),
        price: 45000,
    };

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("car");

    car_v2
        .save_to_path_async::<bincode::Encoder>(path.as_path())
        .await?;

    // There is no migration for CarTest, loading unmigrated entity will cause an error
    let result = CarTest::load_from_path_async::<bincode::Encoder>(path.as_path()).await;
    assert!(result.is_err());

    // migrate with entity with migrations capabilities
    Car::migrate_async::<bincode::Encoder>(path.as_path()).await?;
    // load with enitty, that doesn't contain migration capabilities
    let car = CarTest::load_from_path_async::<bincode::Encoder>(path.as_path()).await?;

    assert_eq!(car.name, "BMW x3".to_string());
    assert_eq!(car.price, "$45000".to_string());
    Ok(())
}

#[tokio::test]
async fn test_load_and_migrate() -> Result<(), SerdeFlowError> {
    let car_v2 = CarV2 {
        brand: "BMW".to_string(),
        model: "x3".to_string(),
        price: 45000,
    };

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("car");

    car_v2
        .save_to_path_async::<bincode::Encoder>(path.as_path())
        .await?;

    // There is no migration for CarTest, loading unmigrated entity will cause an error
    let result = CarTest::load_from_path_async::<bincode::Encoder>(path.as_path()).await;
    assert!(result.is_err());

    let migrated_car = Car::load_and_migrate_async::<bincode::Encoder>(path.as_path()).await?;
    assert_eq!(migrated_car.name, "BMW x3".to_string());
    assert_eq!(migrated_car.price, "$45000".to_string());

    // load with enitty, that doesn't contain migration capabilities
    let car = CarTest::load_from_path_async::<bincode::Encoder>(path.as_path()).await?;

    assert_eq!(car.name, "BMW x3".to_string());
    assert_eq!(car.price, "$45000".to_string());
    Ok(())
}

#[tokio::test]
async fn test_load_from_file_not_found() -> Result<(), SerdeFlowError> {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("not_found");

    let result = Car::load_from_path_async::<bincode::Encoder>(path.as_path()).await;
    let Err(SerdeFlowError::FileNotFound) = result else {
        panic!("load_from_path without file, must return FileNotFound");
    };
    Ok(())
}

#[tokio::test]
async fn test_load_from_file_format_invalid() -> Result<(), SerdeFlowError> {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("zero");
    std::fs::write(path.as_path(), Vec::new());

    let result = Car::load_from_path_async::<bincode::Encoder>(path.as_path()).await;
    let Err(SerdeFlowError::FormatInvalid) = result else {
        panic!("load_from_path with empty file, must return FormatInvalid");
    };
    Ok(())
}

#[tokio::test]
async fn test_migration_not_found() -> Result<(), SerdeFlowError> {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("not_found");

    let result = Car::migrate_async::<bincode::Encoder>(path.as_path()).await;
    let Err(SerdeFlowError::FileNotFound) = result else {
        panic!("Migrate without file, must return FileNotFound");
    };
    Ok(())
}

#[tokio::test]
async fn test_migration_format_invalid() -> Result<(), SerdeFlowError> {
    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("not_found");
    std::fs::write(path.as_path(), Vec::new());

    let result = Car::migrate_async::<bincode::Encoder>(path.as_path()).await;
    let Err(SerdeFlowError::FormatInvalid) = result else {
        panic!("Migrate with empty file, must return FormatInvalid");
    };
    Ok(())
}
