use serde_json::Value;
use std::fs::File;
use std::io::{BufReader, Read};

use crate::prettyprinter::PrettyPrinter;
use actson::feeder::{BufReaderJsonFeeder, JsonFeeder};
use actson::{JsonEvent, JsonParser};

/// Test if [`BufReaderJsonFeeder`] can fully consume a file
#[test]
fn read_from_file() {
    let mut expected = Vec::new();
    {
        let mut file = File::open("tests/fixtures/pass1.txt").unwrap();
        file.read_to_end(&mut expected).unwrap();
    }

    let file = File::open("tests/fixtures/pass1.txt").unwrap();
    let reader = BufReader::with_capacity(32, file);

    let mut feeder = BufReaderJsonFeeder::new(reader);

    assert!(!feeder.has_input());
    assert!(!feeder.is_done());

    assert!(feeder.fill_buf().is_ok());

    assert!(feeder.has_input());
    assert!(!feeder.is_done());

    let mut i = 0;
    loop {
        while let Some(b) = feeder.next_input() {
            assert!(!feeder.is_done());
            assert_eq!(expected[i], b);
            i += 1;
        }

        assert!(feeder.fill_buf().is_ok());

        if feeder.is_done() {
            break;
        }
    }

    assert!(!feeder.has_input());
    assert!(feeder.is_done());
}

/// Test if [`BufReaderJsonFeeder`] can be used to parse a JSON file
#[test]
fn parse_from_file() {
    let expected;
    {
        let mut buf = Vec::new();
        let mut file = File::open("tests/fixtures/pass1.txt").unwrap();
        file.read_to_end(&mut buf).unwrap();
        expected = String::from_utf8(buf).unwrap();
    }

    let file = File::open("tests/fixtures/pass1.txt").unwrap();
    let reader = BufReader::with_capacity(32, file);

    let feeder = BufReaderJsonFeeder::new(reader);
    let mut parser = JsonParser::new(feeder);
    let mut prettyprinter = PrettyPrinter::new();

    loop {
        let mut e = parser.next_event();
        if e == JsonEvent::NeedMoreInput {
            parser.feeder.fill_buf().unwrap();
            e = parser.next_event();
        }

        assert_ne!(e, JsonEvent::Error);

        prettyprinter.on_event(e, &parser).unwrap();

        if e == JsonEvent::Eof {
            break;
        }
    }

    let actual = prettyprinter.get_result();

    let em: Value = serde_json::from_str(&expected).unwrap();
    let am: Value = serde_json::from_str(actual).unwrap();
    assert_eq!(em, am);
}
