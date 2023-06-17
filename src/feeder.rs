#[derive(Debug)]
pub enum FeedError {
    Full,
    NotEnoughInputData,
}

/// A feeder can be used to provide more input data to the [`JsonParser`].
/// The caller has to take care to only feed as much data as the parser can
/// process at the time. Use [`isFull`] to determine if the parser accepts
/// more data. Then, call [`feed_byte`] or [`feed_bytes`] until there is no more
/// data to feed or until [`isFull`] returns `true`. Next, call
/// [`JsonParser.nextEvent`] until it returns [`JsonEvent::NeedMoreInput`].
/// Repeat feeding and parsing until all input data has been consumed. Finally,
/// call [`done`] to indicate the end of the JSON text.
pub trait JsonFeeder {
    /// Provide more data to the [`JsonParser`]. Should only be called if
    /// [`isFull`] returns `false`.
    fn feed_byte(&mut self, b: u8) -> Result<(), FeedError>;

    /// Provide more data to the [`JsonParser`]. The method will consume as
    /// many bytes from the input buffer as possible, either until all bytes
    /// have been consumed or until the feeder is full (see [`isFull`]).
    /// The method will return the number of bytes consumed (which can be 0 if
    /// the parser does not accept more input at the moment).
    fn feed_bytes(&mut self, buf: &[u8]) -> usize;

    /// Checks if the parser accepts more input at the moment. If it doesn't,
    /// you have to call [`nextEvent`] until it returns
    /// [`JsonEvent::NeedMoreInput`]. Only then, new input can be provided
    /// to the parser.
    fn is_full(&self) -> bool;

    /// Call this method to indicate that the end of the JSON text has been
    /// reached and that there is no more input to parse.
    fn done(&mut self);

    /// Determine if the feeder has input data that can be parsed
    fn has_input(&self) -> bool;

    /// Check if the end of the JSON text has been reached
    fn is_done(&self) -> bool;

    /// Decode and return the next character to be parsed
    fn next_input(&mut self) -> Result<u8, FeedError>;
}

pub struct DefaultJsonFeeder {
    input: Vec<u8>,
    done: bool,
}

impl DefaultJsonFeeder {
    pub fn new() -> Self {
        DefaultJsonFeeder {
            input: Vec::new(),
            done: false,
        }
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
        self.input.extend_from_slice(buf);
        buf.len()
    }

    fn is_full(&self) -> bool {
        false
    }

    fn done(&mut self) {
        self.done = true;
    }

    fn has_input(&self) -> bool {
        !self.input.is_empty()
    }

    fn is_done(&self) -> bool {
        self.done
    }

    fn next_input(&mut self) -> Result<u8, FeedError> {
        if !self.has_input() {
            return Err(FeedError::NotEnoughInputData);
        }
        Ok(self.input.remove(0))
    }
}
