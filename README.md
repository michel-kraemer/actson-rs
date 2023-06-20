# Actson [![Actions Status](https://github.com/michel-kraemer/actson-rs/workflows/Rust/badge.svg)](https://github.com/michel-kraemer/actson-rs/actions) [![MIT license](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE) [![Latest Version](https://img.shields.io/crates/v/actson.svg)](https://crates.io/crates/actson) [![Documentation](https://img.shields.io/docsrs/actson/0.1.0)](https://docs.rs/actson/0.1.0/actson/)

Actson is a reactive JSON parser (sometimes referred to as non-blocking or
asynchronous). It is event-based and can be used in asynchronous code (for example in combination with [Tokio](https://tokio.rs/)).

## Why another JSON parser?

* **Non-blocking.** Other JSON parsers use blocking I/O. If you want to develop a reactive application you should use
  non-blocking I/O (see the [Reactive Manifesto](http://www.reactivemanifesto.org/)).
* **Big Data.** Most parsers read the full JSON text into memory to map it to
  a struct, for example. Actson can handle arbitrarily large JSON text. It is
  event-based and can be used for streaming.
* **GeoRocket.** Actson was primarily developed for the [GeoJSON](http://geojson.org/) support support in [GeoRocket](http://georocket.io),
  a high-performance reactive data store for geospatial files.

## Usage

```rust
use actson::{JsonParser, JsonEvent};
use actson::feeder::{DefaultJsonFeeder, JsonFeeder};

let json = r#"{"name": "Elvis"}"#.as_bytes();

let mut feeder = DefaultJsonFeeder::new();
let mut parser = JsonParser::new();
let mut i: usize = 0;
loop {
    // feed as many bytes as possible to the parser
    let mut event = parser.next_event(&mut feeder);
    while event == JsonEvent::NeedMoreInput {
        i += feeder.feed_bytes(&json[i..]);
        if i == json.len() {
            feeder.done();
        }
        event = parser.next_event(&mut feeder);
    }

    // do something useful with `event`

    if event == JsonEvent::Error {
       // do proper error handling here!
       panic!("Error while parsing JSON");
    }

    if event == JsonEvent::Eof {
        break;
    }
}
```

## Other languages

Besides this implementation in Rust here, there is a [Java implementation](https://github.com/michel-kraemer/actson).

## Acknowledgments

The event-based parser code and the JSON files used for testing are largely
based on the file [JSON_checker.c](http://www.json.org/JSON_checker/) and
the JSON test suite from [JSON.org](http://www.json.org/) originally released
under [this license](LICENSE_JSON_checker) (basically MIT license).

## License

Actson is released under the **MIT license**. See the
[LICENSE](LICENSE) file for more information.
