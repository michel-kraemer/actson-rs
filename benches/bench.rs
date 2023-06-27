use std::fs;

use criterion::{criterion_group, criterion_main, Criterion};
use serde_json::Value;

use actson::serde_json::from_slice;

fn make_large(json: &str) -> String {
    let mut large = String::from("{");
    for i in 0..10000 {
        if large.len() > 1 {
            large.push(',');
        }
        large.push_str(&format!(r#""{}":"#, i));
        large.push_str(json);
    }
    large.push('}');
    large
}

fn actson_benchmark(c: &mut Criterion) {
    let json = fs::read_to_string("tests/fixtures/pass1.txt").unwrap();
    let json_bytes = json.as_bytes();

    let json_large = make_large(&json);
    let json_large_bytes = json_large.as_bytes();

    c.bench_function("actson", |b| {
        b.iter(|| {
            from_slice(json_bytes).unwrap();
        })
    });

    c.bench_function("actson_large", |b| {
        b.iter(|| {
            from_slice(json_large_bytes).unwrap();
        })
    });

    c.bench_function("serde", |b| {
        b.iter(|| {
            let _: Value = serde_json::from_str(&json).unwrap();
        })
    });

    c.bench_function("serde_large", |b| {
        b.iter(|| {
            let _: Value = serde_json::from_str(&json_large).unwrap();
        })
    });
}

criterion_group!(benches, actson_benchmark);
criterion_main!(benches);
