use actson::{
    feeder::{DefaultJsonFeeder, JsonFeeder},
    JsonEvent, JsonParser,
};

#[macro_use]
extern crate afl;

fn main() {
    fuzz!(|data: &[u8]| {
        let mut feeder = DefaultJsonFeeder::new();
        let mut parser = JsonParser::new();
        let mut i: usize = 0;
        loop {
            let mut e = parser.next_event(&mut feeder);
            while matches!(e, JsonEvent::NeedMoreInput) {
                i += feeder.feed_bytes(&data[i..]);
                if i == data.len() {
                    feeder.done();
                }
                e = parser.next_event(&mut feeder);
            }

            if matches!(e, JsonEvent::Eof) || matches!(e, JsonEvent::Error) {
                break;
            }
        }
    });
}
