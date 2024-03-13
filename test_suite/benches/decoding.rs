use std::{fs::OpenOptions, path::Path};

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use memmap2::MmapMut;
use serde_flow::{
    encoder::{bincode, zerocopy::ReaderMemmap, FlowEncoder},
    error::SerdeFlowError,
};
use tempfile::tempdir;

#[derive(rkyv::Archive, rkyv::Serialize, rkyv::Deserialize)]
#[archive(check_bytes)]
struct PaymentRkyv {
    pub number1: u16,
    pub number2: u64,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct PaymentSerde {
    pub number1: u16,
    pub number2: u64,
}

fn bench_encoding(c: &mut Criterion) {
    let temp_dir = tempdir().unwrap();
    let rkyv_path = temp_dir.path().to_path_buf().join("rkyv");
    let serde_path = temp_dir.path().to_path_buf().join("serde");

    let serde_object = PaymentSerde {
        number1: 1234,
        number2: 56789,
    };

    let rkyv_object = PaymentRkyv {
        number1: 1234,
        number2: 56789,
    };

    let rkyv_bytes = serde_flow::encoder::zerocopy::Encoder::serialize(&rkyv_object).unwrap();
    let serde_bytes = serde_flow::encoder::bincode::Encoder::serialize(&serde_object).unwrap();

    std::fs::write(rkyv_path.as_path(), rkyv_bytes).unwrap();
    std::fs::write(serde_path.as_path(), serde_bytes).unwrap();

    let mut group = c.benchmark_group("Serializing");
    group.bench_function("rkyv mmap", |b| {
        b.iter(|| {
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(rkyv_path.as_path())
                .unwrap();

            let mmap = black_box(unsafe { MmapMut::map_mut(&file).unwrap() });
            let mut reader_mut = black_box(ReaderMemmap::<PaymentRkyv>::new(mmap));
            reader_mut
                .archive_mut(|payment| {
                    payment.number1 = 2;
                    payment.number2 = 2;
                })
                .unwrap();
        });
    });

    // group.bench_function("rkyv deserialize", |b| {
    //     b.iter(|| {
    //         let response = ObjectTopRkyv::load_from_path(black_box(rkyv_path.as_path())).unwrap();
    //         black_box(response.deserialize().unwrap());
    //     });
    // });

    group.bench_function("bincode", |b| {
        b.iter(|| {
            let bytes = black_box(std::fs::read(serde_path.as_path()).unwrap());
            let mut payment = black_box(
                serde_flow::encoder::bincode::Encoder::deserialize::<PaymentSerde>(&bytes).unwrap(),
            );
            payment.number1 = 2;
            payment.number2 = 2;
            let bytes = serde_flow::encoder::bincode::Encoder::serialize(&payment).unwrap();
            std::fs::write(serde_path.as_path(), bytes).unwrap();
        });
    });
    group.finish();
}

criterion_group!(benches, bench_encoding);
criterion_main!(benches);
