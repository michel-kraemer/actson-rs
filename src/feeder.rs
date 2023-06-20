use ringbuffer::{AllocRingBuffer, RingBuffer, RingBufferRead, RingBufferWrite};

#[derive(Debug, PartialEq, Eq)]
pub enum FeedError {
    Full,
}

/// A feeder can be used to provide more input data to the
/// [`JsonParser`](crate::JsonParser). The caller has to take care to only
/// feed as much data as the parser can process at the time. Use
/// [`is_full()`](Self::is_full()) to determine if the parser accepts
/// more data. Then, call [`feed_byte()`](Self::feed_byte()) or
/// [`feed_bytes()`](Self::feed_bytes()) until there is no more data to feed or
/// until [`is_full()`](Self::is_full()) returns `true`. Next, call
/// [`JsonParser::next_event()`](crate::JsonParser::next_event()) until it
/// returns [`JsonEvent::NeedMoreInput`](crate::JsonEvent::NeedMoreInput).
/// Repeat feeding and parsing until all input data has been consumed. Finally,
/// call [`done()`](Self::done()) to indicate the end of the JSON text.
pub trait JsonFeeder {
    /// Provide more data to the [`JsonParser`](crate::JsonParser). Should only
    /// be called if [`is_full()`](Self::is_full()) returns `false`.
    fn feed_byte(&mut self, b: u8) -> Result<(), FeedError>;

    /// Provide more data to the [`JsonParser`](crate::JsonParser). The method
    /// will consume as many bytes from the input buffer as possible, either
    /// until all bytes have been consumed or until the feeder is full
    /// (see [`is_full()`](Self::is_full())). The method will return the number
    /// of bytes consumed (which can be 0 if the parser does not accept more
    /// input at the moment).
    fn feed_bytes(&mut self, buf: &[u8]) -> usize;

    /// Checks if the parser accepts more input at the moment. If it doesn't,
    /// you have to call [`JsonParser::next_event()`](crate::JsonParser::next_event())
    /// until it returns [`JsonEvent::NeedMoreInput`](crate::JsonEvent::NeedMoreInput).
    /// Only then, new input can be provided to the parser.
    fn is_full(&self) -> bool;

    /// Call this method to indicate that the end of the JSON text has been
    /// reached and that there is no more input to parse.
    fn done(&mut self);

    /// Determine if the feeder has input data that can be parsed
    fn has_input(&self) -> bool;

    /// Check if the end of the JSON text has been reached
    fn is_done(&self) -> bool;

    /// Decode and return the next character to be parsed
    fn next_input(&mut self) -> Option<u8>;
}

pub struct DefaultJsonFeeder {
    input: AllocRingBuffer<u8>,
    done: bool,
}

impl DefaultJsonFeeder {
    pub fn new() -> Self {
        DefaultJsonFeeder {
            input: AllocRingBuffer::with_capacity(1024),
            done: false,
        }
    }
}

impl Default for DefaultJsonFeeder {
    fn default() -> Self {
        Self::new()
    }
}

impl JsonFeeder for DefaultJsonFeeder {
    fn feed_byte(&mut self, b: u8) -> Result<(), FeedError> {
        if self.is_full() {
            return Err(FeedError::Full);
        }
        self.input.push(b);
        Ok(())
    }

    fn feed_bytes(&mut self, buf: &[u8]) -> usize {
        let mut result: usize = 0;
        while result < buf.len() && !self.input.is_full() {
            self.input.push(buf[result]);
            result += 1;
        }
        result
    }

    fn is_full(&self) -> bool {
        self.input.is_full()
    }

    fn done(&mut self) {
        self.done = true;
    }

    fn has_input(&self) -> bool {
        !self.input.is_empty()
    }

    fn is_done(&self) -> bool {
        self.done && !self.has_input()
    }

    fn next_input(&mut self) -> Option<u8> {
        self.input.dequeue()
    }
}

#[cfg(test)]
mod test {
    use ringbuffer::AllocRingBuffer;

    use crate::feeder::{DefaultJsonFeeder, JsonFeeder};

    /// Test if the feeder is empty at the beginning
    #[test]
    fn empty_at_beginning() {
        let feeder = DefaultJsonFeeder::new();
        assert!(!feeder.has_input());
        assert!(!feeder.is_full());
        assert!(!feeder.is_done());
    }

    // Test that [`JsonFeeder::has_input()`] returns `true` after feeding a byte
    #[test]
    fn has_input() {
        let mut feeder = DefaultJsonFeeder::new();
        feeder.feed_byte(b'a').unwrap();
        assert!(feeder.has_input());
    }

    /// Test that [`JsonFeeder::is_full()`] returns `true` if the buffer is
    /// actually full
    #[test]
    fn is_full() {
        let mut feeder = DefaultJsonFeeder {
            input: AllocRingBuffer::with_capacity(16),
            done: false,
        };
        for i in 0..16 {
            assert!(!feeder.is_full());
            feeder.feed_byte(b'a' + i).unwrap();
        }
        assert!(feeder.is_full());
    }

    /// Test if the feeder accepts a byte array
    #[test]
    fn feed_buf() {
        let mut feeder = DefaultJsonFeeder {
            input: AllocRingBuffer::with_capacity(16),
            done: false,
        };
        let buf = "abcd".as_bytes();

        assert!(!feeder.is_full());
        assert!(!feeder.has_input());

        feeder.feed_bytes(buf);

        assert!(!feeder.is_full());
        assert!(feeder.has_input());

        assert_eq!(feeder.next_input(), Some(b'a'));
        assert_eq!(feeder.next_input(), Some(b'b'));
        assert_eq!(feeder.next_input(), Some(b'c'));
        assert_eq!(feeder.next_input(), Some(b'd'));
        assert!(!feeder.is_full());
        assert!(!feeder.has_input());

        feeder.feed_bytes(buf);
        assert!(!feeder.is_full());
        feeder.feed_bytes(buf);
        assert!(!feeder.is_full());
        feeder.feed_bytes(buf);
        assert!(!feeder.is_full());
        feeder.feed_bytes(buf);
        assert!(feeder.is_full());
    }

    /// Test that [`JsonFeeder::is_done()`] returns `true` if [`JsonFeeder::done()`]
    /// has been called and the input has been fully consumed
    #[test]
    fn is_done() {
        let mut feeder = DefaultJsonFeeder::new();
        assert!(!feeder.is_done());
        feeder.feed_byte(b'a').unwrap();
        assert!(!feeder.is_done());
        feeder.done();
        assert!(!feeder.is_done());
        feeder.next_input();
        assert!(feeder.is_done());
    }

    /// Test that the feeder returns an error if it is full
    #[test]
    fn too_full() {
        let mut feeder = DefaultJsonFeeder {
            input: AllocRingBuffer::with_capacity(16),
            done: false,
        };
        for i in 0..16 {
            feeder.feed_byte(b'a' + i).unwrap();
        }
        assert!(feeder.is_full());
        assert_eq!(feeder.feed_byte(b'z'), Err(crate::feeder::FeedError::Full));
    }

    /// Test if the feeder returns the correct input
    fn assert_buf_eq(expected: &[u8], feeder: &mut impl JsonFeeder) {
        let mut i = 0;
        let mut j = 0;
        while i < expected.len() {
            while !feeder.is_full() && i < expected.len() {
                feeder.feed_byte(expected[i]).unwrap();
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
        let mut feeder = DefaultJsonFeeder {
            input: AllocRingBuffer::with_capacity(16),
            done: false,
        };
        assert_buf_eq(b"abcdef", &mut feeder);
    }

    /// Test if a long string (longer than the feeder's buffer size) can be
    /// decoded correctly
    #[test]
    fn long_string() {
        let mut feeder = DefaultJsonFeeder {
            input: AllocRingBuffer::with_capacity(16),
            done: false,
        };
        assert_buf_eq(b"abcdefghijklmnopqrstuvwxyz", &mut feeder);
    }

    /// Test if a very long string (much longer than the feeder's buffer size)
    /// can be decoded correctly
    #[test]
    fn very_long_string() {
        let mut feeder = DefaultJsonFeeder {
            input: AllocRingBuffer::with_capacity(16),
            done: false,
        };
        assert_buf_eq(
            b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz",
            &mut feeder,
        );
    }
}
