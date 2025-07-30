use std::cmp::min;
use std::collections::VecDeque;

use thiserror::Error;

use crate::reset::Reset;

use super::JsonFeeder;

#[derive(Error, Debug)]
pub enum PushError {
    #[error("feeder is full")]
    Full,
}

/// A feeder that can be used to provide more input data to the
/// [`JsonParser`](crate::JsonParser) in a push-based manner. The caller has
/// to take care to only push as much data as the parser can process at the
/// time. Use [`is_full()`](Self::is_full()) to determine if the parser accepts
/// more data. Then, call [`push_byte()`](Self::push_byte()) or
/// [`push_bytes()`](Self::push_bytes()) until there is no more data to push or
/// until [`is_full()`](Self::is_full()) returns `true`. Next, call
/// [`JsonParser::next_event()`](crate::JsonParser::next_event()) until it
/// returns [`JsonEvent::NeedMoreInput`](crate::JsonEvent::NeedMoreInput).
/// Repeat pushing and parsing until all input data has been consumed. Finally,
/// call [`done()`](Self::done()) to indicate the end of the JSON text.
pub struct PushJsonFeeder {
    input: VecDeque<u8>,
    done: bool,
}

impl PushJsonFeeder {
    /// Create a new push-based feeder
    pub fn new() -> Self {
        PushJsonFeeder {
            input: VecDeque::with_capacity(1024),
            done: false,
        }
    }

    /// Provide more data to the [`JsonParser`](crate::JsonParser). Should only
    /// be called if [`is_full()`](Self::is_full()) returns `false`.
    pub fn push_byte(&mut self, b: u8) -> Result<(), PushError> {
        if self.is_full() {
            return Err(PushError::Full);
        }
        self.input.push_back(b);
        Ok(())
    }

    /// Provide more data to the [`JsonParser`](crate::JsonParser). The method
    /// will consume as many bytes from the input buffer as possible, either
    /// until all bytes have been consumed or until the feeder is full
    /// (see [`is_full()`](Self::is_full())). The method will return the number
    /// of bytes consumed (which can be 0 if the parser does not accept more
    /// input at the moment).
    pub fn push_bytes(&mut self, buf: &[u8]) -> usize {
        let n = min(buf.len(), self.input.capacity() - self.input.len());
        self.input.extend(buf.iter().take(n));
        n
    }

    /// Checks if the parser accepts more input at the moment. If it doesn't,
    /// you have to call [`JsonParser::next_event()`](crate::JsonParser::next_event())
    /// until it returns [`JsonEvent::NeedMoreInput`](crate::JsonEvent::NeedMoreInput).
    /// Only then, new input can be provided to the parser.
    pub fn is_full(&self) -> bool {
        self.input.len() == self.input.capacity()
    }

    /// Call this method to indicate that the end of the JSON text has been
    /// reached and that there is no more input to parse.
    pub fn done(&mut self) {
        self.done = true;
    }
}

impl Default for PushJsonFeeder {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonFeeder for PushJsonFeeder {
    fn has_input(&self) -> bool {
        !self.input.is_empty()
    }

    fn is_done(&self) -> bool {
        self.done && !self.has_input()
    }

    fn next_input(&mut self) -> Option<u8> {
        self.input.pop_front()
    }
}

impl Reset for PushJsonFeeder {
    /// Reset the feeder to the state it was in when it was constructed
    fn reset(&mut self) {
        self.input.clear();
        self.done = false;
    }
}

#[cfg(test)]
mod test {
    use std::collections::VecDeque;

    use crate::feeder::{JsonFeeder, PushError, PushJsonFeeder};

    /// Test if the feeder is empty at the beginning
    #[test]
    fn empty_at_beginning() {
        let feeder = PushJsonFeeder::new();
        assert!(!feeder.has_input());
        assert!(!feeder.is_full());
        assert!(!feeder.is_done());
    }

    // Test that [`JsonFeeder::has_input()`] returns `true` after feeding a byte
    #[test]
    fn has_input() {
        let mut feeder = PushJsonFeeder::new();
        feeder.push_byte(b'a').unwrap();
        assert!(feeder.has_input());
    }

    /// Test that [`JsonFeeder::is_full()`] returns `true` if the buffer is
    /// actually full
    #[test]
    fn is_full() {
        let mut feeder = PushJsonFeeder {
            input: VecDeque::with_capacity(16),
            done: false,
        };
        for i in 0..16 {
            assert!(!feeder.is_full());
            feeder.push_byte(b'a' + i).unwrap();
        }
        assert!(feeder.is_full());
    }

    /// Test if the feeder accepts a byte array
    #[test]
    fn feed_buf() {
        let mut feeder = PushJsonFeeder {
            input: VecDeque::with_capacity(16),
            done: false,
        };
        let buf = "abcd".as_bytes();

        assert!(!feeder.is_full());
        assert!(!feeder.has_input());

        feeder.push_bytes(buf);

        assert!(!feeder.is_full());
        assert!(feeder.has_input());

        assert_eq!(feeder.next_input(), Some(b'a'));
        assert_eq!(feeder.next_input(), Some(b'b'));
        assert_eq!(feeder.next_input(), Some(b'c'));
        assert_eq!(feeder.next_input(), Some(b'd'));
        assert!(!feeder.is_full());
        assert!(!feeder.has_input());

        feeder.push_bytes(buf);
        assert!(!feeder.is_full());
        feeder.push_bytes(buf);
        assert!(!feeder.is_full());
        feeder.push_bytes(buf);
        assert!(!feeder.is_full());
        feeder.push_bytes(buf);
        assert!(feeder.is_full());
    }

    /// Test that [`JsonFeeder::is_done()`] returns `true` if [`JsonFeeder::done()`]
    /// has been called and the input has been fully consumed
    #[test]
    fn is_done() {
        let mut feeder = PushJsonFeeder::new();
        assert!(!feeder.is_done());
        feeder.push_byte(b'a').unwrap();
        assert!(!feeder.is_done());
        feeder.done();
        assert!(!feeder.is_done());
        feeder.next_input();
        assert!(feeder.is_done());
    }

    /// Test that the feeder returns an error if it is full
    #[test]
    fn too_full() {
        let mut feeder = PushJsonFeeder {
            input: VecDeque::with_capacity(16),
            done: false,
        };
        for i in 0..16 {
            feeder.push_byte(b'a' + i).unwrap();
        }
        assert!(feeder.is_full());
        assert!(matches!(feeder.push_byte(b'z'), Err(PushError::Full)));
    }

    /// Test if the feeder returns the correct input
    fn assert_buf_eq(expected: &[u8], feeder: &mut PushJsonFeeder) {
        let mut i = 0;
        let mut j = 0;
        while i < expected.len() {
            while !feeder.is_full() && i < expected.len() {
                feeder.push_byte(expected[i]).unwrap();
                i += 1;
            }
            while feeder.has_input() {
                assert_eq!(feeder.next_input(), Some(expected[j]));
                j += 1;
            }
        }
        assert_eq!(j, expected.len());
        assert!(!feeder.has_input());
        assert!(!feeder.is_full());
    }

    /// Test if a short string can be decoded correctly
    #[test]
    fn short_string() {
        let mut feeder = PushJsonFeeder {
            input: VecDeque::with_capacity(16),
            done: false,
        };
        assert_buf_eq(b"abcdef", &mut feeder);
    }

    /// Test if a long string (longer than the feeder's buffer size) can be
    /// decoded correctly
    #[test]
    fn long_string() {
        let mut feeder = PushJsonFeeder {
            input: VecDeque::with_capacity(16),
            done: false,
        };
        assert_buf_eq(b"abcdefghijklmnopqrstuvwxyz", &mut feeder);
    }

    /// Test if a very long string (much longer than the feeder's buffer size)
    /// can be decoded correctly
    #[test]
    fn very_long_string() {
        let mut feeder = PushJsonFeeder {
            input: VecDeque::with_capacity(16),
            done: false,
        };
        assert_buf_eq(
            b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
            &mut feeder,
        );
    }
}
