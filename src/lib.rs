//! # Actson
//!
//! A non-blocking, event-based JSON parser.
//!
//! ```
//! use actson::{JsonParser, JsonEvent};
//! use actson::feeder::{DefaultJsonFeeder, JsonFeeder};
//!
//! let json = r#"{"name": "Elvis"}"#.as_bytes();
//!
//! let mut feeder = DefaultJsonFeeder::new();
//! let mut parser = JsonParser::new();
//! let mut i: usize = 0;
//! loop {
//!     // feed as many bytes as possible to the parser
//!     let mut event = parser.next_event(&mut feeder);
//!     while event == JsonEvent::NeedMoreInput {
//!         i += feeder.feed_bytes(&json[i..]);
//!         if i == json.len() {
//!             feeder.done();
//!         }
//!         event = parser.next_event(&mut feeder);
//!     }
//!
//!     // do something useful with `event`
//!
//!     if event == JsonEvent::Error {
//!        // do proper error handling here!
//!        panic!("Error while parsing JSON");
//!     }
//!
//!     if event == JsonEvent::Eof {
//!         break;
//!     }
//! }
//! ```
mod event;
pub mod feeder;
mod parser;

pub use event::JsonEvent;
pub use parser::JsonParser;
