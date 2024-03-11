use serde::{Deserialize, Serialize};
use serde_flow::{encoder::json, flow::File, flow::FileMigrate, Flow};
use tempfile::tempdir;

#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 3, file)]
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
#[flow(variant = 1, file)]
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

#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 3, file)]
pub struct CarTest {
    pub name: String,
    pub price: String,
}
#[test]
fn test_load_and_migrate() {
    let car_v2 = CarV2 {
        brand: "BMW".to_string(),
        model: "x5".to_string(),
        price: 75000,
    };

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("user");

    car_v2
        .save_to_path::<json::Encoder>(path.as_path())
        .unwrap();

    let err_without_migrate = CarTest::load_from_path::<json::Encoder>(path.as_path());
    assert!(err_without_migrate.is_err());

    let loaded_car = Car::load_and_migrate::<json::Encoder>(path.as_path()).unwrap();
    assert_eq!(loaded_car.name.as_str(), "BMW x5");
    assert_eq!(loaded_car.price.as_str(), "$75000");
    let car = CarTest::load_from_path::<json::Encoder>(path.as_path()).unwrap();

    assert_eq!(car.name.as_str(), "BMW x5");
    assert_eq!(car.price.as_str(), "$75000");
}
