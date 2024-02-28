use serde_json::{Map, Number, Value};
use thiserror::Error;

use crate::feeder::{JsonFeeder, SliceJsonFeeder};
use crate::parser::{
    InvalidFloatValueError, InvalidIntValueError, InvalidStringValueError, ParserError,
};
use crate::{JsonEvent, JsonParser};

/// An error that can happen when parsing JSON to a Serde [`Value`]
#[derive(Error, Debug)]
pub enum IntoSerdeValueError {
    #[error("{0}")]
    Parse(#[from] ParserError),

    #[error("{0}")]
    InvalidStringValue(#[from] InvalidStringValueError),

    #[error("{0}")]
    InvalidIntValue(#[from] InvalidIntValueError),

    #[error("{0}")]
    InvalidFloatValue(#[from] InvalidFloatValueError),

    #[error("not a JSON number: {0}")]
    IllegalJsonNumber(f64),
}

fn to_value<T>(event: &JsonEvent, parser: &JsonParser<T>) -> Result<Value, IntoSerdeValueError>
where
    T: JsonFeeder,
{
    Ok(match event {
        JsonEvent::ValueString => Value::String(parser.current_str()?.to_string()),
        JsonEvent::ValueInt => Value::Number(Number::from(parser.current_int::<i64>()?)),
        JsonEvent::ValueFloat => {
            let f = parser.current_float()?;
            let n = Number::from_f64(f).ok_or(IntoSerdeValueError::IllegalJsonNumber(f))?;
            Value::Number(n)
        }
        JsonEvent::ValueTrue => Value::Bool(true),
        JsonEvent::ValueFalse => Value::Bool(false),
        JsonEvent::ValueNull => Value::Null,
        _ => unreachable!("this function will only be called for valid events"),
    })
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
pub fn from_slice(v: &[u8]) -> Result<Value, IntoSerdeValueError> {
    let feeder = SliceJsonFeeder::new(v);
    let mut parser = JsonParser::new(feeder);

    let mut stack = vec![];
    let mut result = None;
    let mut current_key = None;

    while let Some(event) = parser.next_event()? {
        match event {
            JsonEvent::NeedMoreInput => {}

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

            JsonEvent::FieldName => current_key = Some(parser.current_str()?.to_string()),

            JsonEvent::ValueString
            | JsonEvent::ValueInt
            | JsonEvent::ValueFloat
            | JsonEvent::ValueTrue
            | JsonEvent::ValueFalse
            | JsonEvent::ValueNull => {
                if let Some((_, top)) = stack.last_mut() {
                    let v = to_value(&event, &parser)?;
                    if let Some(m) = top.as_object_mut() {
                        m.insert(current_key.unwrap(), v);
                        current_key = None
                    } else if let Some(a) = top.as_array_mut() {
                        a.push(v);
                    }
                } else if result.is_none() {
                    let v = to_value(&event, &parser)?;
                    result = Some(v);
                } else {
                    return Err(IntoSerdeValueError::Parse(ParserError::SyntaxError));
                }
            }
        }
    }

    result.ok_or(IntoSerdeValueError::Parse(ParserError::NoMoreInput))
}

#[cfg(test)]
mod test {
    use crate::{
        parser::ParserError,
        serde_json::{from_slice, IntoSerdeValueError},
    };
    use serde_json::{from_slice as serde_from_slice, Value};

    /// Test that a top-level string value can be parsed
    #[test]
    fn top_level_string() {
        let json = r#""Elvis""#.as_bytes();
        assert_eq!(
            serde_from_slice::<Value>(json).unwrap(),
            from_slice(json).unwrap()
        );
    }

    /// Test that a top-level int value can be parsed
    #[test]
    fn top_level_int() {
        let json = r#"5"#.as_bytes();
        assert_eq!(
            serde_from_slice::<Value>(json).unwrap(),
            from_slice(json).unwrap()
        );
    }

    /// Test that a top-level float value can be parsed
    #[test]
    fn top_level_float() {
        let json = r#"-5.0"#.as_bytes();
        assert_eq!(
            serde_from_slice::<Value>(json).unwrap(),
            from_slice(json).unwrap()
        );
    }

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

    /// Test that a premature end of input is reported correctly
    #[test]
    fn premature_end_of_input() {
        let json = r#"{"name":"#.as_bytes();
        assert!(matches!(
            from_slice(json),
            Err(IntoSerdeValueError::Parse(ParserError::NoMoreInput))
        ));
    }

    /// Test that a syntax error is reported correctly
    #[test]
    fn syntax_error() {
        let json = r#"{"name"}"#.as_bytes();
        assert!(matches!(
            from_slice(json),
            Err(IntoSerdeValueError::Parse(ParserError::SyntaxError))
        ));
    }
}
