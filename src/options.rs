/// Options for [`JsonParser`](super::JsonParser). Use [`JsonParserOptionsBuilder`]
/// to create instances of this struct.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct JsonParserOptions {
    /// The maximum stack depth
    pub(super) max_depth: usize,

    /// `true` if streaming mode should be enabled, which means that the parser
    /// will be able to handle a stream of multiple JSON values
    pub(super) streaming: bool,
}

/// A builder for [`JsonParserOptions`]
///
/// ```rust
/// use actson::feeder::PushJsonFeeder;
/// use actson::options::JsonParserOptionsBuilder;
/// use actson::JsonParser;
///
/// let feeder = PushJsonFeeder::new();
/// let mut parser = JsonParser::new_with_options(
///     feeder,
///     JsonParserOptionsBuilder::default()
///         .with_max_depth(16)
///         .build(),
/// );
/// ```
#[derive(Default, Clone, Copy, Debug, PartialEq, Eq)]
pub struct JsonParserOptionsBuilder {
    options: JsonParserOptions,
}

impl Default for JsonParserOptions {
    /// Returns default JSON parser options
    fn default() -> Self {
        Self {
            max_depth: 2048,
            streaming: false,
        }
    }
}

impl JsonParserOptions {
    /// Returns the maximum stack depth
    pub fn max_depth(&self) -> usize {
        self.max_depth
    }

    /// Returns `true` if streaming mode should be enabled, which means that
    /// the parser will be able to handle a stream of multiple JSON values
    pub fn streaming(&self) -> bool {
        self.streaming
    }
}

impl JsonParserOptionsBuilder {
    /// Set the maximum stack depth
    pub fn with_max_depth(mut self, max_depth: usize) -> Self {
        self.options.max_depth = max_depth;
        self
    }

    /// Enable streaming mode, which means that the parser will be able to
    /// handle a stream of multiple JSON values. Values must be clearly
    /// separable. They must either be self-delineating values (i.e. arrays,
    /// objects, strings) or keywords (i.e. `true`, `false`, `null`), or they
    /// must be separated either by white space, at least one self-delineating
    /// value, or at least one keyword.
    ///
    /// ## Example streams
    ///
    /// `1 2 3 4 5`
    ///
    /// `[1,2,3][4,5,6]{"key": "value"} 7 8 9`
    ///
    /// `"a""b"[1, 2, 3] {"key": "value"}`
    ///
    /// ## Example:
    ///
    /// ```rust
    /// use actson::feeder::SliceJsonFeeder;
    /// use actson::options::JsonParserOptionsBuilder;
    /// use actson::{JsonEvent, JsonParser};
    ///
    /// let json = r#"1 2""{"key":"value"}
    /// ["a","b"]4true"#.as_bytes();
    ///
    /// let feeder = SliceJsonFeeder::new(json);
    /// let mut parser = JsonParser::new_with_options(
    ///     feeder,
    ///     JsonParserOptionsBuilder::default()
    ///         .with_streaming(true)
    ///         .build(),
    /// );
    ///
    /// let mut events = Vec::new();
    /// while let Some(e) = parser.next_event().unwrap() {
    ///     events.push(e);
    /// }
    ///
    /// assert_eq!(events, vec![
    ///     JsonEvent::ValueInt,
    ///     JsonEvent::ValueInt,
    ///     JsonEvent::ValueString,
    ///     JsonEvent::StartObject,
    ///     JsonEvent::FieldName,
    ///     JsonEvent::ValueString,
    ///     JsonEvent::EndObject,
    ///     JsonEvent::StartArray,
    ///     JsonEvent::ValueString,
    ///     JsonEvent::ValueString,
    ///     JsonEvent::EndArray,
    ///     JsonEvent::ValueInt,
    ///     JsonEvent::ValueTrue,
    /// ]);
    /// ```
    pub fn with_streaming(mut self, streaming: bool) -> Self {
        self.options.streaming = streaming;
        self
    }

    /// Create a new [`JsonParserOptions`] object
    pub fn build(self) -> JsonParserOptions {
        self.options
    }
}
