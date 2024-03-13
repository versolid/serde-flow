use serde::{Deserialize, Serialize};
use serde_flow::{encoder::bincode, error::SerdeFlowError, flow::Bytes, Flow};

#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 3, bytes)]
#[variants(MyStructV2, MyStructV1)]
struct MyStruct {
    field: String,
}

#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 2, bytes)]
#[variants(MyStructV1)]
struct MyStructV2 {
    field: String,
    value: u32,
}

#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 1, bytes)]
struct MyStructV1 {
    field: String,
    value1: u32,
    value2: u32,
}

impl From<MyStructV1> for MyStruct {
    fn from(object: MyStructV1) -> Self {
        MyStruct {
            field: format!("Name: {}", object.value1 + object.value2),
        }
    }
}

impl From<MyStructV1> for MyStructV2 {
    fn from(object: MyStructV1) -> Self {
        MyStructV2 {
            field: "Name: ".to_string(),
            value: object.value1 + object.value2,
        }
    }
}

impl From<MyStructV2> for MyStruct {
    fn from(object: MyStructV2) -> Self {
        MyStruct {
            field: format!("Name: {}", object.value),
        }
    }
}

#[test]
pub fn decode_from_v1() {
    let mystruct1 = MyStructV1 {
        field: "Name: ".to_string(),
        value1: 10,
        value2: 20,
    };

    let bytes = mystruct1.encode::<bincode::Encoder>().unwrap();
    let mystruct3 = MyStruct::decode::<bincode::Encoder>(&bytes).unwrap();
    assert_eq!("Name: 30", mystruct3.field.as_str());
}

#[test]
pub fn decode_v2_from_v1() {
    let mystruct1 = MyStructV1 {
        field: "Name: ".to_string(),
        value1: 10,
        value2: 20,
    };

    let bytes = mystruct1.encode::<bincode::Encoder>().unwrap();
    let mystruct2 = MyStructV2::decode::<bincode::Encoder>(&bytes).unwrap();
    assert_eq!("Name: ", mystruct2.field.as_str());
    assert_eq!(30, mystruct2.value);
}

#[test]
pub fn decode_from_emtpy_returns_error() {
    let empty_bytes = Vec::<u8>::new();
    let result = MyStruct::decode::<bincode::Encoder>(&empty_bytes);
    assert!(matches!(result, Err(SerdeFlowError::FormatInvalid)))
}

#[derive(Serialize, Deserialize, Flow)]
#[flow(variant = 20, bytes)]
struct MyStructV20 {
    value: u32,
}

#[test]
pub fn decode_from_not_found_variant_returns_error() {
    let mystruct20 = MyStructV20 { value: 20 };
    let bytes = mystruct20.encode::<bincode::Encoder>().unwrap();
    let result = MyStruct::decode::<bincode::Encoder>(&bytes);
    assert!(matches!(result, Err(SerdeFlowError::VariantNotFound)))
}
