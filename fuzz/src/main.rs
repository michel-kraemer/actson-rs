use actson::{feeder::PushJsonFeeder, JsonEvent, JsonParser};

#[macro_use]
extern crate afl;

fn main() {
    fuzz!(|data: &[u8]| {
        let feeder = PushJsonFeeder::new();
        let mut parser = JsonParser::new(feeder);
        let mut i: usize = 0;
        loop {
            let mut e = parser.next_event();
            while e == JsonEvent::NeedMoreInput {
                i += parser.feeder.push_bytes(&data[i..]);
                if i == data.len() {
                    parser.feeder.done();
                }
                e = parser.next_event();
            }

            if e == JsonEvent::Eof || e == JsonEvent::Error {
                break;
            }
        }
    });
}
