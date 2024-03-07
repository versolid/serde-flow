use serde::{Deserialize, Serialize};
use serde_flow::{encoder::bincode, flow::FileAsync, flow::FileMigrateAsync, Flow};
use tempfile::tempdir;

#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 3, file(nonbloking))]
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
#[flow(variant = 1, file(nonbloking))]
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
async fn test_load_from_path() {
    let car_v2 = CarV2 {
        brand: "BMW".to_string(),
        model: "x3".to_string(),
        price: 45000,
    };

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("car");

    car_v2
        .save_to_path_async::<bincode::Encoder>(path.as_path())
        .await
        .unwrap();

    let car = Car::load_from_path_async::<bincode::Encoder>(path.as_path())
        .await
        .unwrap();

    assert_eq!(car.name, "BMW x3".to_string());
    assert_eq!(car.price, "$45000".to_string());
}
