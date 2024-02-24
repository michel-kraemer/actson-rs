use std::error::Error;

use actson::feeder::JsonFeeder;
use actson::{JsonEvent, JsonParser};

enum Type {
    Object,
    Array,
}

/// Demonstrates how you can use the [`JsonParser`] to pretty-print
/// a JSON object or array. Note: this is not a perfect implementation of a
/// pretty-printer. The output could still be nicer.
pub struct PrettyPrinter {
    result: String,
    types: Vec<Type>,
    element_counts: Vec<i32>,
    level: i32,
}

impl PrettyPrinter {
    pub fn new() -> Self {
        PrettyPrinter {
            result: String::new(),
            types: vec![],
            element_counts: vec![],
            level: 0,
        }
    }

    fn indent(&mut self) {
        for _ in 0..self.level {
            self.result.push_str("  ");
        }
    }

    fn on_start_object(&mut self) {
        self.on_value();
        self.result.push_str("{\n");
        self.level += 1;
        self.indent();
        self.element_counts.push(0);
        self.types.push(Type::Object);
    }

    fn on_end_object(&mut self) {
        self.level -= 1;
        self.result.push('\n');
        self.indent();
        self.result.push('}');
        self.element_counts.pop();
        self.types.pop();
    }

    fn on_start_array(&mut self) {
        self.on_value();
        self.result.push_str("[\n");
        self.level += 1;
        self.indent();
        self.element_counts.push(0);
        self.types.push(Type::Array);
    }

    fn on_end_array(&mut self) {
        self.level -= 1;
        self.result.push('\n');
        self.indent();
        self.result.push(']');
        self.element_counts.pop();
        self.types.pop();
    }

    fn on_field_name(&mut self, name: &str) {
        if let Some(last) = self.element_counts.last() {
            if *last > 0 {
                self.result.push_str(",\n");
                self.indent();
            }
        }

        self.result.push('"');
        self.result.push_str(name);
        self.result.push_str("\": ");

        if let Some(last) = self.element_counts.pop() {
            self.element_counts.push(last + 1);
        }
    }

    fn on_value(&mut self) {
        if let Some(Type::Array) = self.types.last() {
            if let Some(last) = self.element_counts.pop() {
                if last > 0 {
                    self.result.push_str(", ");
                }
                self.element_counts.push(last + 1);
            }
        }
    }

    fn on_value_string(&mut self, value: &str) {
        self.on_value();
        self.result.push('"');
        self.result.push_str(value);
        self.result.push('"');
    }

    fn on_value_int<I>(&mut self, value: I)
    where
        I: ToString,
    {
        self.on_value();
        self.result.push_str(&value.to_string());
    }

    fn on_value_float(&mut self, value: f64) {
        self.on_value();
        let mut buf = dtoa::Buffer::new();
        self.result.push_str(buf.format(value));
    }

    fn on_value_bool(&mut self, value: bool) {
        self.on_value();
        self.result.push_str(&value.to_string());
    }

    fn on_value_null(&mut self) {
        self.on_value();
        self.result.push_str("null");
    }

    pub fn on_event<T>(
        &mut self,
        event: JsonEvent,
        parser: &JsonParser<T>,
    ) -> Result<(), Box<dyn Error>>
    where
        T: JsonFeeder,
    {
        match event {
            JsonEvent::NeedMoreInput => {}
            JsonEvent::StartObject => self.on_start_object(),
            JsonEvent::EndObject => self.on_end_object(),
            JsonEvent::StartArray => self.on_start_array(),
            JsonEvent::EndArray => self.on_end_array(),
            JsonEvent::FieldName => self.on_field_name(parser.current_string()?),
            JsonEvent::ValueString => self.on_value_string(parser.current_string()?),
            JsonEvent::ValueInt => self.on_value_int(parser.current_int::<i64>()?),
            JsonEvent::ValueFloat => self.on_value_float(parser.current_float()?),
            JsonEvent::ValueTrue => self.on_value_bool(true),
            JsonEvent::ValueFalse => self.on_value_bool(false),
            JsonEvent::ValueNull => self.on_value_null(),
            JsonEvent::Eof => {}
            JsonEvent::Error => return Err("Could not parse JSON".into()),
        }
        Ok(())
    }

    pub fn get_result(&self) -> &str {
        &self.result
    }
}
