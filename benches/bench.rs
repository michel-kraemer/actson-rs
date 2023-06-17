use std::fs;

use actson::{feeder::JsonFeeder, JsonEvent};
use criterion::{criterion_group, criterion_main, Criterion};

fn actson_benchmark(c: &mut Criterion) {
    let json = fs::read_to_string("tests/fixtures/pass1.txt").unwrap();
    let json_bytes = json.as_bytes();
    c.bench_function("actson", |b| {
        b.iter(|| {
            let mut feeder = actson::feeder::DefaultJsonFeeder::new();
            let mut parser = actson::JsonParser::new();

            let mut i: usize = 0;
            loop {
                // feed as many bytes as possible to the parser
                let mut event = parser.next_event(&mut feeder);
                while matches!(event, JsonEvent::NeedMoreInput) {
                    i += feeder.feed_bytes(&json_bytes[i..]);
                    if i == json.len() {
                        feeder.done();
                    }
                    event = parser.next_event(&mut feeder);
                }

                if matches!(event, JsonEvent::Eof) {
                    break;
                }
            }
        })
    });
}

fn serde_benchmark(c: &mut Criterion) {
    let json = fs::read_to_string("tests/fixtures/pass1.txt").unwrap();
    c.bench_function("serde", |b| {
        b.iter(|| {
            let _: serde_json::Value = serde_json::from_str(&json).unwrap();
        })
    });
}

criterion_group!(benches, actson_benchmark, serde_benchmark);
criterion_main!(benches);
