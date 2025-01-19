use std::{
    collections::VecDeque,
    num::ParseFloatError,
    str::{from_utf8, Utf8Error},
};

use crate::{feeder::JsonFeeder, options::JsonParserOptions, JsonEvent};
use btoi::ParseIntegerError;
use num_traits::{CheckedAdd, CheckedMul, CheckedSub, FromPrimitive, Zero};
use thiserror::Error;

const __: i8 = -1; // the universal error code

// Characters are mapped into these 31 character classes. This allows for
// a significant reduction in the size of the state transition table.
const C_SPACE: i8 = 0; // space
const C_WHITE: i8 = 1; // other whitespace
const C_LCURB: i8 = 2; // {
const C_RCURB: i8 = 3; // }
const C_LSQRB: i8 = 4; // [
const C_RSQRB: i8 = 5; // ]
const C_COLON: i8 = 6; // :
const C_COMMA: i8 = 7; // ,
const C_QUOTE: i8 = 8; // "
const C_BACKS: i8 = 9; // \
const C_SLASH: i8 = 10; // /
const C_PLUS: i8 = 11; // +
const C_MINUS: i8 = 12; // -
const C_POINT: i8 = 13; // .
const C_ZERO: i8 = 14; // 0
const C_DIGIT: i8 = 15; // 123456789
const C_LOW_A: i8 = 16; // a
const C_LOW_B: i8 = 17; // b
const C_LOW_C: i8 = 18; // c
const C_LOW_D: i8 = 19; // d
const C_LOW_E: i8 = 20; // e
const C_LOW_F: i8 = 21; // f
const C_LOW_L: i8 = 22; // l
const C_LOW_N: i8 = 23; // n
const C_LOW_R: i8 = 24; // r
const C_LOW_S: i8 = 25; // s
const C_LOW_T: i8 = 26; // t
const C_LOW_U: i8 = 27; // u
const C_ABCDF: i8 = 28; // ABCDF
const C_E: i8 = 29; // E
const C_ETC: i8 = 30; // everything else

/// This array maps the 128 ASCII characters into character classes.
/// The remaining Unicode characters should be mapped to C_ETC.
/// Non-whitespace control characters are errors.
#[rustfmt::skip]
const ASCII_CLASS: [i8; 128] = [
    __,      __,      __,      __,      __,      __,      __,      __,
    __,      C_WHITE, C_WHITE, __,      __,      C_WHITE, __,      __,
    __,      __,      __,      __,      __,      __,      __,      __,
    __,      __,      __,      __,      __,      __,      __,      __,

    C_SPACE, C_ETC,   C_QUOTE, C_ETC,   C_ETC,   C_ETC,   C_ETC,   C_ETC,
    C_ETC,   C_ETC,   C_ETC,   C_PLUS,  C_COMMA, C_MINUS, C_POINT, C_SLASH,
    C_ZERO,  C_DIGIT, C_DIGIT, C_DIGIT, C_DIGIT, C_DIGIT, C_DIGIT, C_DIGIT,
    C_DIGIT, C_DIGIT, C_COLON, C_ETC,   C_ETC,   C_ETC,   C_ETC,   C_ETC,

    C_ETC,   C_ABCDF, C_ABCDF, C_ABCDF, C_ABCDF, C_E,     C_ABCDF, C_ETC,
    C_ETC,   C_ETC,   C_ETC,   C_ETC,   C_ETC,   C_ETC,   C_ETC,   C_ETC,
    C_ETC,   C_ETC,   C_ETC,   C_ETC,   C_ETC,   C_ETC,   C_ETC,   C_ETC,
    C_ETC,   C_ETC,   C_ETC,   C_LSQRB, C_BACKS, C_RSQRB, C_ETC,   C_ETC,

    C_ETC,   C_LOW_A, C_LOW_B, C_LOW_C, C_LOW_D, C_LOW_E, C_LOW_F, C_ETC,
    C_ETC,   C_ETC,   C_ETC,   C_ETC,   C_LOW_L, C_ETC,   C_LOW_N, C_ETC,
    C_ETC,   C_ETC,   C_LOW_R, C_LOW_S, C_LOW_T, C_LOW_U, C_ETC,   C_ETC,
    C_ETC,   C_ETC,   C_ETC,   C_LCURB, C_ETC,   C_RCURB, C_ETC,   C_ETC
];

/// The state codes.
const GO: i8 = 0; // start
const OK: i8 = 1; // ok
const OB: i8 = 2; // object
const KE: i8 = 3; // key
const CO: i8 = 4; // colon
const VA: i8 = 5; // value
const AR: i8 = 6; // array
const ST: i8 = 7; // string
const ES: i8 = 8; // escape
const U1: i8 = 9; // u1
const U2: i8 = 10; // u2
const U3: i8 = 11; // u3
const U4: i8 = 12; // u4
const MI: i8 = 13; // minus
const ZE: i8 = 14; // zero
const IN: i8 = 15; // integer
const F0: i8 = 16; // frac0
const FR: i8 = 17; // fraction
const E1: i8 = 18; // e
const E2: i8 = 19; // ex
const E3: i8 = 20; // exp
const T1: i8 = 21; // tr
const T2: i8 = 22; // tru
const T3: i8 = 23; // true
const F1: i8 = 24; // fa
const F2: i8 = 25; // fal
const F3: i8 = 26; // fals
const F4: i8 = 27; // false
const N1: i8 = 28; // nu
const N2: i8 = 29; // nul
const N3: i8 = 30; // null
const RC: i8 = 99; // recover if in streaming mode, error otherwise

/// The state transition table takes the current state and the current symbol,
/// and returns either a new state or an action. An action is represented as a
/// negative number. A JSON text is accepted if at the end of the text the
/// state is OK and if the mode is MODE_DONE.
#[rustfmt::skip]
const STATE_TRANSITION_TABLE: [i8; 992] = [
/*               white                                      1-9                                   ABCDF  etc
             space |  {  }  [  ]  :  ,  "  \  /  +  -  .  0  |  a  b  c  d  e  f  l  n  r  s  t  u  |  E  | pad */
/*start  GO*/  GO,GO,-6,__,-5,__,__,__,ST,__,__,__,MI,__,ZE,IN,__,__,__,__,__,F1,__,N1,__,__,T1,__,__,__,__,__,
/*ok     OK*/  OK,OK,RC,-8,RC,-7,__,-3,RC,__,__,__,RC,__,RC,RC,__,__,__,__,__,RC,__,RC,__,__,RC,__,__,__,__,__,
/*object OB*/  OB,OB,__,-9,__,__,__,__,ST,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,
/*key    KE*/  KE,KE,__,__,__,__,__,__,ST,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,
/*colon  CO*/  CO,CO,__,__,__,__,-2,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,
/*value  VA*/  VA,VA,-6,__,-5,__,__,__,ST,__,__,__,MI,__,ZE,IN,__,__,__,__,__,F1,__,N1,__,__,T1,__,__,__,__,__,
/*array  AR*/  AR,AR,-6,__,-5,-7,__,__,ST,__,__,__,MI,__,ZE,IN,__,__,__,__,__,F1,__,N1,__,__,T1,__,__,__,__,__,
/*string ST*/  ST,__,ST,ST,ST,ST,ST,ST,-4,ES,ST,ST,ST,ST,ST,ST,ST,ST,ST,ST,ST,ST,ST,ST,ST,ST,ST,ST,ST,ST,ST,__,
/*escape ES*/  __,__,__,__,__,__,__,__,ST,ST,ST,__,__,__,__,__,__,ST,__,__,__,ST,__,ST,ST,__,ST,U1,__,__,__,__,
/*u1     U1*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,U2,U2,U2,U2,U2,U2,U2,U2,__,__,__,__,__,__,U2,U2,__,__,
/*u2     U2*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,U3,U3,U3,U3,U3,U3,U3,U3,__,__,__,__,__,__,U3,U3,__,__,
/*u3     U3*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,U4,U4,U4,U4,U4,U4,U4,U4,__,__,__,__,__,__,U4,U4,__,__,
/*u4     U4*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,ST,ST,ST,ST,ST,ST,ST,ST,__,__,__,__,__,__,ST,ST,__,__,
/*minus  MI*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,ZE,IN,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,
/*zero   ZE*/  OK,OK,RC,-8,RC,-7,__,-3,RC,__,__,__,__,F0,__,__,__,__,__,__,E1,RC,__,RC,__,__,RC,__,__,E1,__,__,
/*int    IN*/  OK,OK,RC,-8,RC,-7,__,-3,RC,__,__,__,__,F0,IN,IN,__,__,__,__,E1,RC,__,RC,__,__,RC,__,__,E1,__,__,
/*frac0  F0*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,FR,FR,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,
/*frac   FR*/  OK,OK,RC,-8,RC,-7,__,-3,RC,__,__,__,__,__,FR,FR,__,__,__,__,E1,RC,__,RC,__,__,RC,__,__,E1,__,__,
/*e      E1*/  __,__,__,__,__,__,__,__,__,__,__,E2,E2,__,E3,E3,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,
/*ex     E2*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,E3,E3,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,
/*exp    E3*/  OK,OK,RC,-8,RC,-7,__,-3,RC,__,__,__,__,__,E3,E3,__,__,__,__,__,RC,__,RC,__,__,RC,__,__,__,__,__,
/*tr     T1*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,T2,__,__,__,__,__,__,__,
/*tru    T2*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,T3,__,__,__,__,
/*true   T3*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,OK,__,__,__,__,__,__,__,__,__,__,__,
/*fa     F1*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,F2,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,
/*fal    F2*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,F3,__,__,__,__,__,__,__,__,__,
/*fals   F3*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,F4,__,__,__,__,__,__,
/*false  F4*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,OK,__,__,__,__,__,__,__,__,__,__,__,
/*nu     N1*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,N2,__,__,__,__,
/*nul    N2*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,N3,__,__,__,__,__,__,__,__,__,
/*null   N3*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,OK,__,__,__,__,__,__,__,__,__,
];

/// These modes can be pushed on the stack.
const MODE_ARRAY: i8 = 0;
const MODE_DONE: i8 = 1;
const MODE_KEY: i8 = 2;
const MODE_OBJECT: i8 = 3;

/// An error that can happen when reading the current value as a string
#[derive(Error, Debug)]
#[error("invalid string: {0}")]
pub struct InvalidStringValueError(#[from] Utf8Error);

/// An error that can happen when trying to parse the current value to an integer
#[derive(Error, Debug)]
#[error("invalid integer: {0}")]
pub struct InvalidIntValueError(#[from] ParseIntegerError);

/// An error that can happen when trying to parse the current value to a float
#[derive(Error, Debug)]
pub enum InvalidFloatValueError {
    #[error("unable to convert current value to string: {0}")]
    String(#[from] InvalidStringValueError),

    #[error("unable to parse current value to float: {0}")]
    Float(#[from] ParseFloatError),
}

/// An error that can happen during parsing
#[derive(Error, Debug, Clone, Copy)]
pub enum ParserError {
    /// The JSON text contains an illegal byte (e.g. a non-whitespace control
    /// character)
    #[error("JSON text contains an illegal byte: `{0}'")]
    IllegalInput(u8),

    /// The parsed text is not valid JSON
    #[error("syntax error: the parsed text is not valid JSON")]
    SyntaxError,

    /// There is nothing more to parse. The feeder is done and does not provide
    /// more input. Either the JSON text ended prematurely or
    /// [`JsonParser::next_event()`](crate::JsonParser::next_event()) was called
    /// too many times (i.e. after the end of a valid JSON text was reached).
    #[error("nothing more to parse")]
    NoMoreInput,
}

/// A non-blocking, event-based JSON parser.
pub struct JsonParser<T> {
    pub feeder: T,

    /// The stack containing the current modes
    stack: VecDeque<i8>,

    /// The maximum number of modes on the stack
    depth: usize,

    /// `true` if streaming mode is enabled, which means that the parser can
    /// handle a stream of multiple JSON values
    streaming: bool,

    /// The current state
    state: i8,

    /// Collects all characters if the current state is ST (String),
    /// IN (Integer), FR (Fraction) or the like
    current_buffer: Vec<u8>,

    /// The first event returned by [`Self::parse()`]
    event1: JsonEvent,

    /// The second event returned by [`Self::parse()`]
    event2: JsonEvent,

    /// Tracks the number of bytes that have been processed
    parsed_bytes: usize,

    /// A character that has been put back to be parsed at the next call
    /// of [`Self::next_event()`]
    putback_character: Option<u8>,

    /// Tracks if a UTF-16 high surrogate has been encountered
    high_surrogate_pair: bool,
}

impl<T> JsonParser<T>
where
    T: JsonFeeder,
{
    /// Create a new JSON parser using the given [`JsonFeeder`]
    pub fn new(feeder: T) -> Self {
        JsonParser {
            feeder,
            stack: VecDeque::from([MODE_DONE]),
            depth: 2048,
            streaming: false,
            state: GO,
            current_buffer: vec![],
            event1: JsonEvent::NeedMoreInput,
            event2: JsonEvent::NeedMoreInput,
            parsed_bytes: 0,
            putback_character: None,
            high_surrogate_pair: false,
        }
    }

    /// Create a new JSON parser using the given [`JsonFeeder`] and with a
    /// defined maximum stack depth
    #[deprecated(since = "1.1.0", note = "use `new_with_options` instead")]
    pub fn new_with_max_depth(feeder: T, max_depth: usize) -> Self {
        JsonParser {
            feeder,
            stack: VecDeque::from([MODE_DONE]),
            depth: max_depth,
            streaming: false,
            state: GO,
            current_buffer: vec![],
            event1: JsonEvent::NeedMoreInput,
            event2: JsonEvent::NeedMoreInput,
            parsed_bytes: 0,
            putback_character: None,
            high_surrogate_pair: false,
        }
    }

    /// Create a new JSON parser using the given [`JsonFeeder`] and
    /// [`JsonParserOptions`]
    pub fn new_with_options(feeder: T, options: JsonParserOptions) -> Self {
        JsonParser {
            feeder,
            stack: VecDeque::from([MODE_DONE]),
            depth: options.max_depth,
            streaming: options.streaming,
            state: GO,
            current_buffer: vec![],
            event1: JsonEvent::NeedMoreInput,
            event2: JsonEvent::NeedMoreInput,
            parsed_bytes: 0,
            putback_character: None,
            high_surrogate_pair: false,
        }
    }

    /// Push to the stack. Return `false` if the maximum stack depth has been
    /// exceeded.
    fn push(&mut self, mode: i8) -> bool {
        if self.stack.len() >= self.depth {
            return false;
        }
        self.stack.push_back(mode);
        true
    }

    /// Pop the stack, assuring that the current mode matches the expectation.
    /// Return `false` if there is underflow or if the modes mismatch.
    fn pop(&mut self, mode: i8) -> bool {
        if self.stack.is_empty() || *self.stack.back().unwrap() != mode {
            return false;
        }
        self.stack.pop_back();
        true
    }

    /// Get the next input character either from [`Self::putback_character`] or
    /// from [`Self::feeder`]
    fn get_next_input(&mut self) -> Option<u8> {
        self.putback_character
            .take()
            .or_else(|| self.feeder.next_input())
    }

    /// Put back the given character to be parsed at the next call of
    /// [`Self::next_event()`]
    fn put_back(&mut self, c: u8) {
        assert!(
            self.putback_character.is_none(),
            "Only one character can be put back"
        );
        self.putback_character = Some(c);
        self.parsed_bytes -= 1;
    }

    /// Call this method to proceed parsing the JSON text and to get the next
    /// event. The method returns [`Some(JsonEvent::NeedMoreInput)`](JsonEvent::NeedMoreInput)
    /// if it needs more input data from the feeder or `None` if the end of the
    /// JSON text has been reached.
    pub fn next_event(&mut self) -> Result<Option<JsonEvent>, ParserError> {
        while self.event1 == JsonEvent::NeedMoreInput {
            if let Some(b) = self.get_next_input() {
                self.parsed_bytes += 1;
                if self.state == ST && (32..=127).contains(&b) && b != b'\\' && b != b'"' {
                    // shortcut
                    self.current_buffer.push(b);
                } else {
                    self.parse(b)?;
                }
            } else {
                if self.feeder.is_done() {
                    if self.state != OK {
                        let r = self.state_to_event();
                        if r != JsonEvent::NeedMoreInput {
                            self.state = OK;
                            return Ok(Some(r));
                        }
                    }
                    return if self.state == OK && self.pop(MODE_DONE) {
                        Ok(None)
                    } else {
                        Err(ParserError::NoMoreInput)
                    };
                }
                return Ok(Some(JsonEvent::NeedMoreInput));
            }
        }

        let r = self.event1;
        self.event1 = self.event2;
        self.event2 = JsonEvent::NeedMoreInput;

        Ok(Some(r))
    }

    /// This function is called for each character (or partial character) in the
    /// JSON text. It will set [`self::event1`] and [`self::event2`] accordingly.
    /// As a precondition, these fields should have a value of [`JsonEvent::NeedMoreInput`].
    fn parse(&mut self, next_char: u8) -> Result<(), ParserError> {
        // determine the character's class.
        let next_class;
        if next_char >= 128 {
            next_class = C_ETC;
        } else {
            next_class = ASCII_CLASS[next_char as usize];
            if next_class <= __ {
                return Err(ParserError::IllegalInput(next_char));
            }
        }

        // Get the next state from the state transition table.
        let mut next_state =
            STATE_TRANSITION_TABLE[((self.state as usize) << 5) + next_class as usize];

        // Try to recover if in streaming mode.
        if next_state == RC {
            if self.streaming && self.stack.len() == 1 && *self.stack.back().unwrap() == MODE_DONE {
                // Streaming is enabled and we're in a state where we can handle
                // another JSON value.
                if self.state == OK {
                    // The previous value has been converted to an event. Try
                    // again to get the next state but start from the GO state.
                    next_state = STATE_TRANSITION_TABLE[((GO as usize) << 5) + next_class as usize];
                } else {
                    // Switch to the OK state to convert the current value into
                    // an event. Put back the character so it will be parsed again.
                    next_state = OK;
                    self.put_back(next_char);
                }
            } else {
                // Streaming is not enabled or we're not on the top level. This
                // is a syntax error.
                next_state = __;
            }
        }

        if next_state >= 0 {
            if (ST..=E3).contains(&next_state) {
                // According to 'STATE_TRANSITION_TABLE', we don't need to check
                // for "state <= E3". There is no way we can get here without
                // 'state' being less than or equal to E3.
                // if state >= ST && state <= E3 {
                if self.state >= ST {
                    if self.state == ES {
                        match next_char {
                            b'\\' => {
                                self.current_buffer.pop();
                                self.current_buffer.push(0x5C);
                                next_state = ST;
                            }
                            b'n' => {
                                self.current_buffer.pop();
                                self.current_buffer.push(0x0A);
                                next_state = ST;
                            }
                            b'r' => {
                                self.current_buffer.pop();
                                self.current_buffer.push(0x0D);
                                next_state = ST;
                            }
                            b't' => {
                                self.current_buffer.pop();
                                self.current_buffer.push(0x09);
                                next_state = ST;
                            }
                            b'b' => {
                                self.current_buffer.pop();
                                self.current_buffer.push(0x08);
                                next_state = ST;
                            }
                            b'f' => {
                                self.current_buffer.pop();
                                self.current_buffer.push(0x0C);
                                next_state = ST;
                            }
                            b'/' => {
                                self.current_buffer.pop();
                                self.current_buffer.push(0x2F);
                                next_state = ST;
                            }
                            b'"' => {
                                self.current_buffer.pop();
                                self.current_buffer.push(0x22);
                                next_state = ST;
                            }
                            _ => {
                                self.current_buffer.push(next_char);
                            }
                        }
                    } else if self.state == U4 {
                        self.current_buffer.push(next_char);

                        // last 6 bytes in the buffer will now be the escaped unicode in the form
                        // \uXXXX

                        // this is a UTF-8 encoded version of the unicode code point
                        if self.current_buffer.len() < 6 {
                            return Err(ParserError::SyntaxError);
                        }

                        let unicode_in_utf8 =
                            from_utf8(&self.current_buffer[self.current_buffer.len() - 4..])
                                .map_err(|_| ParserError::SyntaxError)?;

                        // convert the UTF-8 encoded unicode code point to a u32
                        let unicode = u32::from_str_radix(unicode_in_utf8, 16)
                            .map_err(|_| ParserError::SyntaxError)?;

                        // UTF-16 high pair
                        if (0xD800..=0xDBFF).contains(&unicode) {
                            if self.high_surrogate_pair {
                                return Err(ParserError::SyntaxError);
                            }

                            self.high_surrogate_pair = true;
                        }
                        // UTF-16 low pair
                        else if (0xDC00..=0xDFFF).contains(&unicode) {
                            if !self.high_surrogate_pair {
                                return Err(ParserError::SyntaxError);
                            }

                            self.high_surrogate_pair = false;
                            // UTF-16 surrogate pair detected
                            // combine the high and low surrogate pairs to get the unicode character
                            // this will be the last 12 characters in the buffer
                            // \uXXXX\uXXXX
                            //   |  |  |  |
                            //   high  low

                            if self.current_buffer.len() < 12 {
                                return Err(ParserError::SyntaxError);
                            }

                            // create the high code point
                            let high_code_point = u16::from_str_radix(
                                from_utf8(
                                    &self.current_buffer[self.current_buffer.len() - 10
                                        ..self.current_buffer.len() - 6],
                                )
                                .map_err(|_| ParserError::SyntaxError)?,
                                16,
                            )
                            .map_err(|_| ParserError::SyntaxError)?;

                            // create the low code point
                            let low_code_point = u16::from_str_radix(
                                from_utf8(&self.current_buffer[self.current_buffer.len() - 4..])
                                    .map_err(|_| ParserError::SyntaxError)?,
                                16,
                            )
                            .map_err(|_| ParserError::SyntaxError)?;

                            let char = char::decode_utf16(
                                [high_code_point, low_code_point].iter().cloned(),
                            )
                            .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
                            .collect::<String>();

                            // remove last 12 bytes and insert new
                            self.current_buffer.truncate(self.current_buffer.len() - 12);
                            self.current_buffer.extend_from_slice(char.as_bytes());
                        } else {
                            // convert the u32 to a char
                            let unicode_char =
                                char::from_u32(unicode).ok_or(ParserError::SyntaxError)?;

                            // regular case
                            // convert the char to a String and get the u8 bytes
                            let unicode_as_string = unicode_char.to_string();

                            // remove the last 6 bytes from the buffer
                            self.current_buffer.truncate(self.current_buffer.len() - 6);

                            // add the UTF-8 encoded unicode code point to the buffer
                            self.current_buffer
                                .extend_from_slice(unicode_as_string.as_bytes());
                        }
                    } else {
                        self.current_buffer.push(next_char);
                    }
                } else {
                    self.current_buffer.clear();
                    if next_state != ST {
                        self.current_buffer.push(next_char);
                    }
                }
            } else if next_state == OK {
                // end of token identified, convert state to result
                self.event1 = self.state_to_event();
            }

            // Change the state.
            self.state = next_state;
        } else {
            // Or perform one of the actions.
            self.perform_action(next_state)?;
        }

        Ok(())
    }

    /// Perform an action that changes the parser state
    fn perform_action(&mut self, action: i8) -> Result<(), ParserError> {
        match action {
            // empty }
            -9 => {
                if !self.pop(MODE_KEY) {
                    return Err(ParserError::SyntaxError);
                }
                self.state = OK;
                self.event1 = JsonEvent::EndObject;
            }

            // }
            -8 => {
                if !self.pop(MODE_OBJECT) {
                    return Err(ParserError::SyntaxError);
                }
                match self.state_to_event() {
                    JsonEvent::NeedMoreInput => self.event1 = JsonEvent::EndObject,
                    e => {
                        self.event1 = e;
                        self.event2 = JsonEvent::EndObject;
                    }
                }
                self.state = OK;
            }

            // ]
            -7 => {
                if !self.pop(MODE_ARRAY) {
                    return Err(ParserError::SyntaxError);
                }
                match self.state_to_event() {
                    JsonEvent::NeedMoreInput => self.event1 = JsonEvent::EndArray,
                    e => {
                        self.event1 = e;
                        self.event2 = JsonEvent::EndArray;
                    }
                }
                self.state = OK;
            }

            // {
            -6 => {
                if !self.push(MODE_KEY) {
                    return Err(ParserError::SyntaxError);
                }
                self.state = OB;
                self.event1 = JsonEvent::StartObject;
            }

            // [
            -5 => {
                if !self.push(MODE_ARRAY) {
                    return Err(ParserError::SyntaxError);
                }
                self.state = AR;
                self.event1 = JsonEvent::StartArray;
            }

            // "
            -4 => {
                if *self.stack.back().unwrap() == MODE_KEY {
                    self.state = CO;
                    self.event1 = JsonEvent::FieldName;
                } else {
                    self.state = OK;
                    self.event1 = JsonEvent::ValueString;
                }
            }

            // ,
            -3 => {
                match *self.stack.back().unwrap() {
                    MODE_OBJECT => {
                        // A comma causes a flip from object mode to key mode.
                        if !self.pop(MODE_OBJECT) || !self.push(MODE_KEY) {
                            return Err(ParserError::SyntaxError);
                        }
                        self.event1 = self.state_to_event();
                        self.state = KE;
                    }

                    MODE_ARRAY => {
                        self.event1 = self.state_to_event();
                        self.state = VA;
                    }

                    _ => {
                        return Err(ParserError::SyntaxError);
                    }
                }
            }

            // :
            -2 => {
                // A colon causes a flip from key mode to object mode.
                if !self.pop(MODE_KEY) || !self.push(MODE_OBJECT) {
                    return Err(ParserError::SyntaxError);
                }
                self.state = VA;
            }

            // Bad action.
            _ => {
                return Err(ParserError::SyntaxError);
            }
        }

        Ok(())
    }

    /// Converts the current parser state to a JSON event. Returns the JSON
    /// event or [`JsonEvent::NeedMoreInput`] if the current state does
    /// not produce a JSON event
    fn state_to_event(&self) -> JsonEvent {
        match self.state {
            IN | ZE => JsonEvent::ValueInt,
            FR..=E3 => JsonEvent::ValueFloat,
            T3 => JsonEvent::ValueTrue,
            F4 => JsonEvent::ValueFalse,
            N3 => JsonEvent::ValueNull,
            _ => JsonEvent::NeedMoreInput,
        }
    }

    /// Get the value of the string that has just been parsed. Call this
    /// function after you've received [`JsonEvent::FieldName`](JsonEvent#variant.FieldName)
    /// or [`JsonEvent::ValueString`](JsonEvent#variant.ValueString).
    pub fn current_str(&self) -> Result<&str, InvalidStringValueError> {
        Ok(from_utf8(&self.current_buffer)?)
    }

    /// Get the value of the integer that has just been parsed. Call this
    /// function after you've received [`JsonEvent::ValueInt`](JsonEvent#variant.ValueInt).
    pub fn current_int<I>(&self) -> Result<I, InvalidIntValueError>
    where
        I: FromPrimitive + Zero + CheckedAdd + CheckedSub + CheckedMul,
    {
        Ok(btoi::btoi(&self.current_buffer)?)
    }

    /// Get the value of the float that has just been parsed. Call this
    /// function after you've received [`JsonEvent::ValueFloat`](JsonEvent#variant.ValueFloat).
    pub fn current_float(&self) -> Result<f64, InvalidFloatValueError> {
        Ok(self.current_str()?.parse()?)
    }

    /// Return the number of bytes parsed so far
    pub fn parsed_bytes(&self) -> usize {
        self.parsed_bytes
    }
}
