use serde_json::{Map, Number, Value};

use crate::feeder::{JsonFeeder, PushJsonFeeder};
use crate::{JsonEvent, JsonParser};

#[derive(Debug, Clone)]
pub struct ParserError;

fn to_value<T>(event: &JsonEvent, parser: &JsonParser<T>) -> Option<Value>
where
    T: JsonFeeder,
{
    match event {
        JsonEvent::ValueString => Some(Value::String(parser.current_string().unwrap())),

        JsonEvent::ValueInt => {
            if let Ok(i) = parser.current_i32() {
                Some(Value::Number(Number::from(i)))
            } else {
                Some(Value::Number(Number::from(parser.current_i64().unwrap())))
            }
        }

        JsonEvent::ValueDouble => Some(Value::Number(
            Number::from_f64(parser.current_f64().unwrap()).unwrap(),
        )),

        JsonEvent::ValueTrue => Some(Value::Bool(true)),
        JsonEvent::ValueFalse => Some(Value::Bool(false)),
        JsonEvent::ValueNull => Some(Value::Null),

        _ => None,
    }
}

/// Parse a byte slice into a Serde JSON [Value]
///
/// ```
/// use serde_json::json;
/// use actson::serde_json::from_slice;
///
/// let json = r#"{"name": "Elvis"}"#.as_bytes();
/// let expected = json!({
///     "name": "Elvis"
/// });
/// let actual = from_slice(&json).unwrap();
/// assert_eq!(expected, actual);
/// ```
pub fn from_slice(v: &[u8]) -> Result<Value, ParserError> {
    let mut feeder = PushJsonFeeder::new();
    let mut parser = JsonParser::new(&mut feeder);

    let mut stack = vec![];
    let mut result = None;
    let mut current_key = None;

    let mut i: usize = 0;
    loop {
        // feed as many bytes as possible to the parser
        let mut event = parser.next_event();
        while event == JsonEvent::NeedMoreInput {
            i += parser.feeder.push_bytes(&v[i..]);
            if i == v.len() {
                parser.feeder.done();
            }
            event = parser.next_event();
        }

        match event {
            JsonEvent::Error => return Err(ParserError),
            JsonEvent::NeedMoreInput => panic!("Should never happen"),

            JsonEvent::StartObject | JsonEvent::StartArray => {
                let v = if event == JsonEvent::StartObject {
                    Value::Object(Map::new())
                } else {
                    Value::Array(vec![])
                };
                stack.push((current_key, v));
                current_key = None;
            }

            JsonEvent::EndObject | JsonEvent::EndArray => {
                let v = stack.pop().unwrap();
                if let Some((_, top)) = stack.last_mut() {
                    if let Some(m) = top.as_object_mut() {
                        m.insert(v.0.unwrap(), v.1);
                    } else if let Some(a) = top.as_array_mut() {
                        a.push(v.1);
                    }
                } else {
                    result = Some(v.1);
                }
            }

            JsonEvent::FieldName => current_key = Some(parser.current_string().unwrap()),

            JsonEvent::ValueString
            | JsonEvent::ValueInt
            | JsonEvent::ValueDouble
            | JsonEvent::ValueTrue
            | JsonEvent::ValueFalse
            | JsonEvent::ValueNull => {
                if let Some((_, top)) = stack.last_mut() {
                    let v = to_value(&event, &parser).unwrap();
                    if let Some(m) = top.as_object_mut() {
                        m.insert(current_key.unwrap(), v);
                        current_key = None
                    } else if let Some(a) = top.as_array_mut() {
                        a.push(v);
                    }
                } else {
                    return Err(ParserError);
                }
            }

            JsonEvent::Eof => break,
        }
    }

    result.ok_or(ParserError)
}

#[cfg(test)]
mod test {
    use crate::serde_json::from_slice;
    use serde_json::{from_slice as serde_from_slice, Value};

    /// Test that an empty object is parsed correctly
    #[test]
    fn empty_object() {
        let json = r#"{}"#.as_bytes();
        assert_eq!(
            serde_from_slice::<Value>(json).unwrap(),
            from_slice(json).unwrap()
        );
    }

    /// Test that a simple object is parsed correctly
    #[test]
    fn simple_object() {
        let json = r#"{"name": "Elvis"}"#.as_bytes();
        assert_eq!(
            serde_from_slice::<Value>(json).unwrap(),
            from_slice(json).unwrap()
        );
    }

    /// Test that an empty array is parsed correctly
    #[test]
    fn empty_array() {
        let json = r#"[]"#.as_bytes();
        assert_eq!(
            serde_from_slice::<Value>(json).unwrap(),
            from_slice(json).unwrap()
        );
    }

    /// Test that a simple array is parsed correctly
    #[test]
    fn simple_array() {
        let json = r#"["Elvis", "Max"]"#.as_bytes();
        assert_eq!(
            serde_from_slice::<Value>(json).unwrap(),
            from_slice(json).unwrap()
        );
    }

    /// Test that an array with mixed values is parsed correctly
    #[test]
    fn mixed_array() {
        let json = r#"["Elvis", 132, "Max", 80.67]"#.as_bytes();
        assert_eq!(
            serde_from_slice::<Value>(json).unwrap(),
            from_slice(json).unwrap()
        );
    }

    /// Test that embedded objects is parsed correctly
    #[test]
    fn embedded_objects() {
        let json = r#"{
            "name": "Elvis",
            "address": {"street": "Graceland", "city": "Memphis"},
            "albums": [
                "Elvis Presley",
                "Elvis",
                "Elvis' Christmas Album",
                "Elvis Is Back!",
                {
                    "title": "His Hand in Mine",
                    "year": 1960
                },
                "... any many others :)"
            ]
        }"#
        .as_bytes();
        assert_eq!(
            serde_from_slice::<Value>(json).unwrap(),
            from_slice(json).unwrap()
        );
    }
}
