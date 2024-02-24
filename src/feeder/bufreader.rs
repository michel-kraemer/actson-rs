use std::io::{BufRead, BufReader, Read};

use super::{FillError, JsonFeeder};

/// A [`JsonFeeder`] that reads from a [`BufReader`].
pub struct BufReaderJsonFeeder<T> {
    reader: BufReader<T>,
    filled: bool,
    pos: usize,
}

impl<T> BufReaderJsonFeeder<T>
where
    T: Read,
{
    pub fn new(reader: BufReader<T>) -> Self {
        BufReaderJsonFeeder {
            reader,
            filled: false,
            pos: 0,
        }
    }

    /// Fill the feeder's internal buffer
    pub fn fill_buf(&mut self) -> Result<(), FillError> {
        self.reader.consume(self.pos);
        self.reader.fill_buf()?;
        self.filled = true;
        self.pos = 0;
        Ok(())
    }
}

impl<T> JsonFeeder for BufReaderJsonFeeder<T>
where
    T: Read,
{
    fn has_input(&self) -> bool {
        self.pos < self.reader.buffer().len()
    }

    fn is_done(&self) -> bool {
        self.filled && self.reader.buffer().is_empty()
    }

    fn next_input(&mut self) -> Option<u8> {
        let buf = self.reader.buffer();
        if self.pos < buf.len() {
            let r = Some(buf[self.pos]);
            self.pos += 1;
            r
        } else {
            None
        }
    }
}
