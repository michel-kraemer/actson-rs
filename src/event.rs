/// All possible JSON events returned by {@link JsonParser#nextEvent()}
#[derive(Copy, Clone, Debug)]
pub enum JsonEvent {
    /// The JSON text contains a syntax error.
    Error = -1,

    /// The JSON parser needs more input before the next event can be returned.
    /// Invoke the parser's feeder to give it more input.
    NeedMoreInput = 0,

    /// The start of a JSON object.
    StartObject = 1,

    /// The end of a JSON object.
    EndObject = 2,

    /// The start of a JSON array.
    StartArray = 3,

    /// The end of a JSON array.
    EndArray = 4,

    /// A field name. Call {@link JsonParser#getCurrentString()}
    /// to get the name.
    FieldName = 5,

    /// A string value. Call {@link JsonParser#getCurrentString()}
    /// to get the value.
    ValueString = 6,

    /// An integer value. Call {@link JsonParser#getCurrentInt()}
    /// to get the value.
    ValueInt = 7,

    /// A double value. Call {@link JsonParser#getCurrentDouble()}
    /// to get the value.
    ValueDouble = 8,

    /// The boolean value `true`.
    ValueTrue = 9,

    /// The boolean value `false`.
    ValueFalse = 10,

    /// A `null` value.
    ValueNull = 11,

    /// The end of the JSON text
    Eof = 99,
}
