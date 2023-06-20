use std::fs;

use actson::{
    feeder::{DefaultJsonFeeder, JsonFeeder},
    JsonEvent, JsonParser,
};
use criterion::{criterion_group, criterion_main, Criterion};
use serde_json::{Map, Number, Value};

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

fn to_value(event: &JsonEvent, parser: &JsonParser) -> Option<Value> {
    match event {
        JsonEvent::ValueString => Some(Value::String(parser.current_string().unwrap())),

        JsonEvent::ValueInt => {
            if let Ok(i) = parser.current_i32() {
                Some(Value::Number(Number::from(i)))
            } else {
                Some(Value::Number(Number::from(parser.current_i64().unwrap())))
            }
        }

        JsonEvent::ValueDouble => Some(Value::Number(
            Number::from_f64(parser.current_f64().unwrap()).unwrap(),
        )),
        JsonEvent::ValueTrue => Some(Value::Bool(true)),
        JsonEvent::ValueFalse => Some(Value::Bool(false)),
        JsonEvent::ValueNull => Some(Value::Null),

        _ => None,
    }
}

fn actson_parse(json_bytes: &[u8]) {
    let mut feeder = DefaultJsonFeeder::new();
    let mut parser = JsonParser::new();

    let mut stack = vec![];
    let mut current_val = Value::Array(vec![]);
    let mut current_key = None;

    let mut i: usize = 0;
    loop {
        // feed as many bytes as possible to the parser
        let mut event = parser.next_event(&mut feeder);
        while event == JsonEvent::NeedMoreInput {
            i += feeder.feed_bytes(&json_bytes[i..]);
            if i == json_bytes.len() {
                feeder.done();
            }
            event = parser.next_event(&mut feeder);
        }

        match event {
            JsonEvent::Error => panic!("Parser error"),
            JsonEvent::NeedMoreInput => panic!("Should never happen"),

            JsonEvent::StartObject | JsonEvent::StartArray => {
                stack.push((current_key, current_val));
                current_key = None;
                current_val = if event == JsonEvent::StartObject {
                    Value::Object(Map::new())
                } else {
                    Value::Array(vec![])
                }
            }

            JsonEvent::EndObject | JsonEvent::EndArray => {
                let c = current_val;
                let t = stack.pop().unwrap();
                current_val = t.1;
                if let Some(m) = current_val.as_object_mut() {
                    m.insert(t.0.unwrap(), c);
                } else if let Some(a) = current_val.as_array_mut() {
                    a.push(c);
                }
            }

            JsonEvent::FieldName => current_key = Some(parser.current_string().unwrap()),

            JsonEvent::ValueString
            | JsonEvent::ValueInt
            | JsonEvent::ValueDouble
            | JsonEvent::ValueTrue
            | JsonEvent::ValueFalse
            | JsonEvent::ValueNull => {
                let v = to_value(&event, &parser).unwrap();
                if let Some(m) = current_val.as_object_mut() {
                    m.insert(current_key.unwrap(), v);
                    current_key = None
                } else if let Some(a) = current_val.as_array_mut() {
                    a.push(v);
                }
            }

            JsonEvent::Eof => break,
        }
    }
}

fn actson_benchmark(c: &mut Criterion) {
    let json = fs::read_to_string("tests/fixtures/pass1.txt").unwrap();
    let json_bytes = json.as_bytes();

    let json_large = make_large(&json);
    let json_large_bytes = json_large.as_bytes();

    c.bench_function("actson", |b| {
        b.iter(|| {
            actson_parse(json_bytes);
        })
    });

    c.bench_function("actson_large", |b| {
        b.iter(|| {
            actson_parse(json_large_bytes);
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
