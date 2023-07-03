use crate::feeder::JsonFeeder;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};

/// A [`JsonFeeder`] that reads from an asynchronous [`BufReader`].
pub struct AsyncBufReaderJsonFeeder<T> {
    reader: BufReader<T>,
    filled: bool,
    pos: usize,
}

impl<T> AsyncBufReaderJsonFeeder<T>
where
    T: AsyncRead + Unpin,
{
    pub fn new(reader: BufReader<T>) -> Self {
        AsyncBufReaderJsonFeeder {
            reader,
            filled: false,
            pos: 0,
        }
    }

    pub async fn fill_buf(&mut self) -> Result<(), std::io::Error> {
        self.reader.consume(self.pos);
        self.reader.fill_buf().await?;
        self.filled = true;
        self.pos = 0;
        Ok(())
    }
}

impl<T> JsonFeeder for AsyncBufReaderJsonFeeder<T>
where
    T: AsyncRead + Unpin,
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
