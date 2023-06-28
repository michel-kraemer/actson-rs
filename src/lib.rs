//! # Actson
//!
//! A non-blocking, event-based JSON parser.
//!
//! ## Examples
//!
//! ### Push-based parsing
//!
//! Push-based parsing is the most flexible way of using Actson. Push new bytes
//! into a [`PushJsonFeeder`](crate::feeder::PushJsonFeeder) and then let the
//! parser consume them until it returns [`JsonEvent::NeedMoreInput`]. Repeat
//! this process until you receive [`JsonEvent::Eof`] or [`JsonEvent::Error`].
//!
//! This approach is very low-level but gives you the freedom to provide new
//! bytes to the parser whenever they are available and to generate new JSON
//! events whenever you need them.
//!
//! ```
//! use actson::{JsonParser, JsonEvent};
//! use actson::feeder::{PushJsonFeeder, JsonFeeder};
//!
//! let json = r#"{"name": "Elvis"}"#.as_bytes();
//!
//! let mut feeder = PushJsonFeeder::new();
//! let mut parser = JsonParser::new(&mut feeder);
//! let mut i: usize = 0;
//! loop {
//!     // feed as many bytes as possible to the parser
//!     let mut event = parser.next_event();
//!     while event == JsonEvent::NeedMoreInput {
//!         i += parser.feeder.push_bytes(&json[i..]);
//!         if i == json.len() {
//!             parser.feeder.done();
//!         }
//!         event = parser.next_event();
//!     }
//!
//!     // do something useful with `event`
//!     // match event {
//!     //     ...
//!     // }
//!
//!     assert_ne!(event, JsonEvent::Error);
//!
//!     if event == JsonEvent::Eof {
//!         break;
//!     }
//! }
//! ```
//!
//! ### Parsing from a `BufReader`
//!
//! [`BufReaderJsonFeeder`](crate::feeder::BufReaderJsonFeeder) allows you to
//! feed the parser from a [`BufReader`](std::io::BufReader). This is useful if
//! you want to parse JSON from a file or a network connection.
//!
//! ```
//! use actson::{JsonParser, JsonEvent};
//!
//! use std::fs::File;
//! use std::io::BufReader;
//!
//! let file = File::open("tests/fixtures/pass1.txt").unwrap();
//! let mut reader = BufReader::new(file);
//!
//! let mut feeder = actson::feeder::BufReaderJsonFeeder::new(&mut reader);
//! let mut parser = JsonParser::new(&mut feeder);
//! loop {
//!     let mut event = parser.next_event();
//!     if event == JsonEvent::NeedMoreInput {
//!         parser.feeder.fill_buf().unwrap();
//!         event = parser.next_event();
//!     }
//!
//!     // do something useful with `event`
//!     // match event {
//!     //     ...
//!     // }
//!
//!     assert_ne!(event, JsonEvent::Error);
//!
//!     if event == JsonEvent::Eof {
//!         break;
//!     }
//! }
//! ```
//!
//! ### Parsing a slice of bytes
//!
//! For convenience, [`SliceJsonFeeder`](crate::feeder::SliceJsonFeeder) allows
//! you to feed the parser from a slice of bytes.
//!
//! ```
//! use actson::{JsonParser, JsonEvent};
//! use actson::feeder::SliceJsonFeeder;
//!
//! let json = r#"{"name": "Elvis"}"#.as_bytes();
//!
//! let mut feeder = SliceJsonFeeder::new(json);
//! let mut parser = JsonParser::new(&mut feeder);
//! loop {
//!     let event = parser.next_event();
//!
//!     // do something useful with `event`
//!     // match event {
//!     //     ...
//!     // }
//!
//!     assert_ne!(event, JsonEvent::Error);
//!
//!     if event == JsonEvent::Eof {
//!         break;
//!     }
//! }
//! ```
//!
//! ### Parsing into a Serde JSON Value
//!
//! For testing and compatibility reasons, Actson is able to parse a byte slice
//! into a [Serde JSON](https://github.com/serde-rs/json) Value.
//!
//! Heads up: You need to enable the `serde_json` feature for this.
//!
//! ```
//! use actson::serde_json::from_slice;
//!
//! let json = r#"{"name": "Elvis"}"#.as_bytes();
//! let value = from_slice(json).unwrap();
//!
//! assert!(value.is_object());
//! assert_eq!(value["name"], "Elvis");
//! ```
//!
//! However, if you find yourself doing this, you probably don't need the
//! reactive features of Actson and your data seems to completely fit into
//! memory. In this case, you're most likely better off using Serde JSON
//! directly.
mod event;
pub mod feeder;
mod parser;

#[cfg(feature = "serde_json")]
pub mod serde_json;

pub use event::JsonEvent;
pub use parser::JsonParser;
