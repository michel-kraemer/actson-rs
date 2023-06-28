use std::io::{BufRead, BufReader, Read};

use super::JsonFeeder;

/// A [`JsonFeeder`] that reads from a [`BufReader`].
pub struct BufReaderJsonFeeder<'a, T>
where
    T: Read,
{
    reader: &'a mut BufReader<T>,
    filled: bool,
    pos: usize,
}

impl<'a, T> BufReaderJsonFeeder<'a, T>
where
    T: Read,
{
    pub fn new(reader: &'a mut BufReader<T>) -> Self {
        BufReaderJsonFeeder {
            reader,
            filled: false,
            pos: 0,
        }
    }

    pub fn fill_buf(&mut self) -> Result<(), std::io::Error> {
        self.reader.consume(self.pos);
        self.reader.fill_buf()?;
        self.filled = true;
        self.pos = 0;
        Ok(())
    }
}

impl<'a, T> JsonFeeder for BufReaderJsonFeeder<'a, T>
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
