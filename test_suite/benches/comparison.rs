use std::collections::HashMap;

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use serde_flow::{
    encoder::bincode,
    flow::{zerocopy::File as ZeroFile, File},
};
use tempfile::tempdir;

#[derive(serde::Serialize, serde::Deserialize, serde_flow::Flow)]
#[flow(variant = 1, file)]
pub struct ObjectTopSerde {
    pub field1: String,
    pub field2: String,
    pub values: HashMap<String, ObjectSerde>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct ObjectSerde {
    pub field: String,
    pub number1: u32,
    pub number2: u32,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, serde_flow::Flow)]
#[archive(check_bytes)]
#[flow(variant = 1, file, zerocopy)]
pub struct ObjectTopRkyv {
    pub field1: String,
    pub field2: String,
    pub values: HashMap<String, ObjectRkyv>,
}

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize, Clone)]
#[archive(check_bytes)]
pub struct ObjectRkyv {
    pub field: String,
    pub number1: u32,
    pub number2: u32,
}

fn create_serde() -> ObjectTopSerde {
    let mut values = HashMap::new();
    for i in 0..2000 {
        values.insert(
            format!("Id{i}"),
            ObjectSerde {
                field: format!("{i}Some value could be long here {i}"),
                number1: i * 10,
                number2: i * 20,
            },
        );
    }

    ObjectTopSerde {
        field1: "Some field".to_string(),
        field2: "Some another field".to_string(),
        values,
    }
}

fn create_rkyv() -> ObjectTopRkyv {
    let mut values = HashMap::new();
    for i in 0..2000 {
        values.insert(
            format!("Id{i}"),
            ObjectRkyv {
                field: format!("{i}Some value could be long here {i}"),
                number1: i * 10,
                number2: i * 20,
            },
        );
    }

    ObjectTopRkyv {
        field1: "Some field".to_string(),
        field2: "Some another field".to_string(),
        values,
    }
}

fn bench_serialization(c: &mut Criterion) {
    let rkyv_file = create_rkyv();
    let serde_file = create_serde();

    let temp_dir = tempdir().unwrap();
    let rkyv_path = temp_dir.path().to_path_buf().join("rkyv");
    let serde_path = temp_dir.path().to_path_buf().join("serde");

    rkyv_file.save_to_path(rkyv_path.as_path()).unwrap();
    serde_file
        .save_to_path::<bincode::Encoder>(serde_path.as_path())
        .unwrap();

    let mut group = c.benchmark_group("Deserialize");
    group.bench_function("rkyv archive", |b| {
        b.iter(|| {
            let response = ObjectTopRkyv::load_from_path(black_box(rkyv_path.as_path())).unwrap();
            black_box(response.archive().unwrap());
        });
    });

    group.bench_function("rkyv deserialize", |b| {
        b.iter(|| {
            let response = ObjectTopRkyv::load_from_path(black_box(rkyv_path.as_path())).unwrap();
            black_box(response.deserialize().unwrap());
        });
    });

    group.bench_function("bincode deserialize", |b| {
        b.iter(|| {
            black_box(
                ObjectTopSerde::load_from_path::<bincode::Encoder>(black_box(serde_path.as_path()))
                    .unwrap(),
            );
        });
    });
    group.finish();
}

criterion_group!(benches, bench_serialization);
criterion_main!(benches);
