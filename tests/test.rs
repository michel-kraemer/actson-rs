use actson::feeder::{DefaultJsonFeeder, JsonFeeder};
use actson::{JsonEvent, JsonParser};

mod prettyprinter;

#[test]
fn simple_object() {
    let json = r#"{"name": "Elvis", "age": 42}"#;

    let mut prettyprinter = prettyprinter::PrettyPrinter::new();
    let mut feeder = DefaultJsonFeeder::new();
    let mut parser = JsonParser::new();
    feeder.feed_bytes(json.as_bytes());
    feeder.done();
    loop {
        let e = parser.next_event(&mut feeder);
        prettyprinter.on_event(e, &parser).unwrap();
        if matches!(e, JsonEvent::Eof) || matches!(e, JsonEvent::Error) {
            break;
        }
    }

    println!("{}", prettyprinter.get_result());
}
