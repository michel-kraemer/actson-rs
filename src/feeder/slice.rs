use super::JsonFeeder;

/// A [`JsonFeeder`] that feeds the [`JsonParser`](crate::JsonParser) from a slice of bytes
pub struct SliceJsonFeeder<'a> {
    slice: &'a [u8],
    pos: usize,
}

impl<'a> SliceJsonFeeder<'a> {
    /// Create a new feeder that wraps around the given byte slice
    pub fn new(slice: &'a [u8]) -> Self {
        SliceJsonFeeder { slice, pos: 0 }
    }
}

impl<'a> JsonFeeder for SliceJsonFeeder<'a> {
    fn has_input(&self) -> bool {
        self.pos < self.slice.len()
    }

    fn is_done(&self) -> bool {
        !self.has_input()
    }

    fn next_input(&mut self) -> Option<u8> {
        if !self.has_input() {
            None
        } else {
            let r = Some(self.slice[self.pos]);
            self.pos += 1;
            r
        }
    }
}

#[cfg(test)]
mod test {
    use crate::feeder::JsonFeeder;

    #[test]
    fn empty() {
        let feeder = super::SliceJsonFeeder::new(b"");
        assert!(!feeder.has_input());
        assert!(feeder.is_done());
    }

    #[test]
    fn consume_all() {
        let mut feeder = super::SliceJsonFeeder::new(b"Elvis");
        assert!(feeder.has_input());
        assert!(!feeder.is_done());
        assert_eq!(feeder.next_input(), Some(b'E'));
        assert_eq!(feeder.next_input(), Some(b'l'));
        assert_eq!(feeder.next_input(), Some(b'v'));
        assert_eq!(feeder.next_input(), Some(b'i'));
        assert_eq!(feeder.next_input(), Some(b's'));
        assert!(!feeder.has_input());
        assert!(feeder.is_done());
    }
}
