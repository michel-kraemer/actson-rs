#[derive(Debug, PartialEq, Eq)]
pub enum FeedError {
    Full,
}

/// A feeder can be used to provide more input data to the
/// [`JsonParser`](crate::JsonParser).
pub trait JsonFeeder {
    /// Determine if the feeder has input data that can be parsed
    fn has_input(&self) -> bool;

    /// Check if the end of the JSON text has been reached
    fn is_done(&self) -> bool;

    /// Decode and return the next character to be parsed
    fn next_input(&mut self) -> Option<u8>;
}
