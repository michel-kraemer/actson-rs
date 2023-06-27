//! # Actson
//!
//! A non-blocking, event-based JSON parser.
//!
//! ## Examples
//!
//! ### Push-based parsing
//!
//! Push-based parsing is the most flexible way of using Actson. Push new bytes into a
//! [`PushJsonFeeder`](crate::feeder::PushJsonFeeder) and then let the parser consume these bytes
//! until it returns [`JsonEvent::NeedMoreInput`]. Repeat this process until you receive
//! [`JsonEvent::Eof`] or [`JsonEvent::Error`].
//!
//! This approach is very low-level but gives you the freedom to provide new bytes to the parser
//! whenever they are available and to generate new JSON events whenever you need them.
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
//! ### Parsing a slice of bytes
//!
//! For convenience, [`SliceJsonFeeder`](crate::feeder::SliceJsonFeeder) allows you to feed the
//! parser from a slice of bytes.
//!
//! ```
//! use actson::{JsonParser, JsonEvent};
//! use actson::feeder::{SliceJsonFeeder, JsonFeeder};
//!
//! let json = r#"{"name": "Elvis"}"#.as_bytes();
//!
//! let mut feeder = SliceJsonFeeder::new(json);
//! let mut parser = JsonParser::new(&mut feeder);
//! let mut i: usize = 0;
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
mod event;
pub mod feeder;
mod parser;

#[cfg(feature = "serde_json")]
pub mod serde_json;

pub use event::JsonEvent;
pub use parser::JsonParser;
