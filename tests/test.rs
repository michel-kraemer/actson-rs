use actson::feeder::{DefaultJsonFeeder, JsonFeeder};
use actson::{JsonEvent, JsonParser};

mod prettyprinter;

#[test]
fn simple_object() {
    let json = r#"{"name": "Elvis", "age": 42}"#;

    let mut prettyprinter = prettyprinter::PrettyPrinter::new();
    let mut feeder = DefaultJsonFeeder::new();
    let mut parser = JsonParser::new();
    let mut i: usize = 0;
    loop {
        // feed as many bytes as possible to the parser
        let mut e = parser.next_event(&mut feeder);
        while matches!(e, JsonEvent::NeedMoreInput) {
            i += feeder.feed_bytes(&json.as_bytes()[i..]);
            if i == json.len() {
                feeder.done();
            }
            e = parser.next_event(&mut feeder);
        }

        assert!(!matches!(e, JsonEvent::Error));

        prettyprinter.on_event(e, &parser).unwrap();
        println!("{:?}", e);

        if matches!(e, JsonEvent::Eof) {
            break;
        }
    }

    println!("{}", prettyprinter.get_result());
}
