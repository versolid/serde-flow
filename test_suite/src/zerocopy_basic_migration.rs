use rkyv::{Archive, Deserialize, Serialize};
use serde_flow::flow::zerocopy::File;
use serde_flow_derive::Flow;
use tempfile::tempdir;

#[derive(Archive, Serialize, Deserialize, Flow)]
#[archive(check_bytes)]
#[flow(variant = 3, file, zerocopy)]
#[variants(CarV1, CarV2)]
pub struct Car {
    pub name: String,
    pub price: String,
}

#[derive(Archive, Serialize, Deserialize, Flow)]
#[archive(check_bytes)]
#[flow(variant = 2, zerocopy)]
pub struct CarV1 {
    pub brand: String,
    pub model: String,
    pub price: String,
}

#[derive(Archive, Serialize, Deserialize, Flow)]
#[archive(check_bytes)]
#[flow(variant = 1, file, zerocopy)]
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

#[test]
fn test_v2_load_from_path() {
    let car_v2 = CarV2 {
        brand: "BMW".to_string(),
        model: "x3".to_string(),
        price: 45000,
    };

    let temp_dir = tempdir().unwrap();
    let path = temp_dir.path().to_path_buf().join("car");

    car_v2.save_to_path(path.as_path()).unwrap();

    let car = Car::load_from_path(path.as_path()).unwrap();
    let car_archived = car.archive().unwrap();

    assert_eq!(car_archived.name, "BMW x3".to_string());
    assert_eq!(car_archived.price, "$45000".to_string());
}
