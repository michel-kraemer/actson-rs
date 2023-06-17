use std::{
    error::Error,
    num::{ParseFloatError, ParseIntError},
    string::FromUtf8Error,
};

use crate::{event::JsonEvent, feeder::JsonFeeder};

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

/// The state transition table takes the current state and the current symbol,
/// and returns either a new state or an action. An action is represented as a
/// negative number. A JSON text is accepted if at the end of the text the
/// state is OK and if the mode is MODE_DONE.
#[rustfmt::skip]
const STATE_TRANSITION_TABLE: [i8; 992] = [
/*               white                                      1-9                                   ABCDF  etc
             space |  {  }  [  ]  :  ,  "  \  /  +  -  .  0  |  a  b  c  d  e  f  l  n  r  s  t  u  |  E  | pad */
/*start  GO*/  GO,GO,-6,__,-5,__,__,__,ST,__,__,__,MI,__,ZE,IN,__,__,__,__,__,F1,__,N1,__,__,T1,__,__,__,__,__,
/*ok     OK*/  OK,OK,__,-8,__,-7,__,-3,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,
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
/*zero   ZE*/  OK,OK,__,-8,__,-7,__,-3,__,__,__,__,__,F0,__,__,__,__,__,__,E1,__,__,__,__,__,__,__,__,E1,__,__,
/*int    IN*/  OK,OK,__,-8,__,-7,__,-3,__,__,__,__,__,F0,IN,IN,__,__,__,__,E1,__,__,__,__,__,__,__,__,E1,__,__,
/*frac0  F0*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,FR,FR,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,
/*frac   FR*/  OK,OK,__,-8,__,-7,__,-3,__,__,__,__,__,__,FR,FR,__,__,__,__,E1,__,__,__,__,__,__,__,__,E1,__,__,
/*e      E1*/  __,__,__,__,__,__,__,__,__,__,__,E2,E2,__,E3,E3,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,
/*ex     E2*/  __,__,__,__,__,__,__,__,__,__,__,__,__,__,E3,E3,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,
/*exp    E3*/  OK,OK,__,-8,__,-7,__,-3,__,__,__,__,__,__,E3,E3,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,__,
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

/// A non-blocking, event-based JSON parser.
///
/// # Example
///
/// ```
/// use actson::{JsonParser, JsonEvent};
/// use actson::feeder::{DefaultJsonFeeder, JsonFeeder};
///
/// let json = r#"{"name": "Elvis"}"#;
///
/// let mut feeder = DefaultJsonFeeder::new();
/// let mut parser = JsonParser::new();
/// let mut i: usize = 0;
/// loop {
///     // feed as many bytes as possible to the parser
///     let mut event = parser.next_event(&mut feeder);
///     while matches!(event, JsonEvent::NeedMoreInput) {
///         i += feeder.feed_bytes(&json.as_bytes()[i..]);
///         if i == json.len() {
///             feeder.done();
///         }
///         event = parser.next_event(&mut feeder);
///     }
///
///     // do something useful with `event`
///
///     if matches!(event, JsonEvent::Error) {
///        // do proper error handling here!
///        panic!("Error while parsing JSON");
///     }
///
///     if matches!(event, JsonEvent::Eof) {
///         break;
///     }
/// }
/// ```
pub struct JsonParser {
    /// The stack containing the current modes
    stack: Vec<i8>,

    /// The maximum number of modes on the stack
    depth: usize,

    /// The current state
    state: i8,

    /// Collects all characters if the current state is ST (String),
    /// IN (Integer), FR (Fraction) or the like
    current_buffer: Vec<u8>,

    /// The first event returned by [`Self::parse()`]
    event1: JsonEvent,

    /// The second event returned by [`Self::parse()`]
    event2: JsonEvent,
}

impl JsonParser {
    pub fn new() -> Self {
        JsonParser {
            stack: vec![MODE_DONE],
            depth: 2048,
            state: GO,
            current_buffer: vec![],
            event1: JsonEvent::NeedMoreInput,
            event2: JsonEvent::NeedMoreInput,
        }
    }

    pub fn new_with_max_depth(max_depth: usize) -> Self {
        JsonParser {
            stack: vec![MODE_DONE],
            depth: max_depth,
            state: GO,
            current_buffer: vec![],
            event1: JsonEvent::NeedMoreInput,
            event2: JsonEvent::NeedMoreInput,
        }
    }

    fn push(&mut self, mode: i8) -> bool {
        if self.stack.len() >= self.depth {
            return false;
        }
        self.stack.push(mode);
        true
    }

    /// Pop the stack, assuring that the current mode matches the expectation.
    /// Returns `false` if there is underflow or if the modes mismatch.
    fn pop(&mut self, mode: i8) -> bool {
        if self.stack.is_empty() || *self.stack.last().unwrap() != mode {
            return false;
        }
        self.stack.pop();
        true
    }

    /// Call this method to proceed parsing the JSON text and to get the next
    /// event. The method returns [`JsonEvent::NeedMoreInput`] if it needs
    /// more input data from the given feeder.
    pub fn next_event(&mut self, feeder: &mut impl JsonFeeder) -> JsonEvent {
        while matches!(self.event1, JsonEvent::NeedMoreInput) {
            if let Some(b) = feeder.next_input() {
                self.parse(b);
            } else {
                if feeder.is_done() {
                    if self.state != OK {
                        let r = self.state_to_event();
                        if !matches!(r, JsonEvent::NeedMoreInput) {
                            self.state = OK;
                            return r;
                        }
                    }
                    return if self.state == OK && self.pop(MODE_DONE) {
                        JsonEvent::Eof
                    } else {
                        JsonEvent::Error
                    };
                }
                return JsonEvent::NeedMoreInput;
            }
        }

        let r = self.event1;
        if !matches!(self.event1, JsonEvent::Error) {
            self.event1 = self.event2;
            self.event2 = JsonEvent::NeedMoreInput;
        }

        r
    }

    /// This function is called for each character (or partial character) in the
    /// JSON text. It will set [`self::event1`] and [`self::event2`] accordingly.
    /// As a precondition, these fields should have a value of [`JsonEvent::NeedMoreInput`].
    fn parse(&mut self, next_char: u8) {
        // determine the character's class.
        let next_class: i8;
        if next_char >= 128 {
            next_class = C_ETC;
        } else {
            next_class = ASCII_CLASS[next_char as usize];
            if next_class <= __ {
                self.event1 = JsonEvent::Error;
                return;
            }
        }

        // Get the next state from the state transition table.
        let next_state = STATE_TRANSITION_TABLE[((self.state as usize) << 5) + next_class as usize];
        if next_state >= 0 {
            if (ST..=E3).contains(&next_state) {
                // According to 'STATE_TRANSITION_TABLE', we don't need to check
                // for "state <= E3". There is no way we can get here without
                // 'state' being less than or equal to E3.
                // if state >= ST && state <= E3 {
                if self.state >= ST {
                    self.current_buffer.push(next_char);
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
            self.perform_action(next_state);
        }
    }

    /// Perform an action that changes the parser state
    fn perform_action(&mut self, action: i8) {
        match action {
            // empty }
            -9 => {
                if !self.pop(MODE_KEY) {
                    self.event1 = JsonEvent::Error;
                    return;
                }
                self.state = OK;
                self.event1 = JsonEvent::EndObject;
            }

            // }
            -8 => {
                if !self.pop(MODE_OBJECT) {
                    self.event1 = JsonEvent::Error;
                    return;
                }
                self.event1 = self.state_to_event();
                if matches!(self.event1, JsonEvent::NeedMoreInput) {
                    self.event1 = JsonEvent::EndObject;
                } else {
                    self.event2 = JsonEvent::EndObject;
                }
                self.state = OK;
            }

            // ]
            -7 => {
                if !self.pop(MODE_ARRAY) {
                    self.event1 = JsonEvent::Error;
                    return;
                }
                self.event1 = self.state_to_event();
                if matches!(self.event1, JsonEvent::NeedMoreInput) {
                    self.event1 = JsonEvent::EndArray;
                } else {
                    self.event2 = JsonEvent::EndArray;
                }
                self.state = OK;
            }

            // {
            -6 => {
                if !self.push(MODE_KEY) {
                    self.event1 = JsonEvent::Error;
                    return;
                }
                self.state = OB;
                self.event1 = JsonEvent::StartObject;
            }

            // [
            -5 => {
                if !self.push(MODE_ARRAY) {
                    self.event1 = JsonEvent::Error;
                    return;
                }
                self.state = AR;
                self.event1 = JsonEvent::StartArray;
            }

            // "
            -4 => {
                if *self.stack.last().unwrap() == MODE_KEY {
                    self.state = CO;
                    self.event1 = JsonEvent::FieldName;
                } else {
                    self.state = OK;
                    self.event1 = JsonEvent::ValueString;
                }
            }

            // ,
            -3 => {
                match *self.stack.last().unwrap() {
                    MODE_OBJECT => {
                        // A comma causes a flip from object mode to key mode.
                        if !self.pop(MODE_OBJECT) || !self.push(MODE_KEY) {
                            self.event1 = JsonEvent::Error;
                            return;
                        }
                        self.event1 = self.state_to_event();
                        self.state = KE;
                    }

                    MODE_ARRAY => {
                        self.event1 = self.state_to_event();
                        self.state = VA;
                    }

                    _ => {
                        self.event1 = JsonEvent::Error;
                    }
                }
            }

            // :
            -2 => {
                // A colon causes a flip from key mode to object mode.
                if !self.pop(MODE_KEY) || !self.push(MODE_OBJECT) {
                    self.event1 = JsonEvent::Error;
                    return;
                }
                self.state = VA;
            }

            // Bad action.
            _ => {
                self.event1 = JsonEvent::Error;
            }
        }
    }

    /// Converts the current parser state to a JSON event. Returns the JSON
    /// event or [`JsonEvent::NeedMoreInput`] if the current state does
    /// not produce a JSON event
    fn state_to_event(&self) -> JsonEvent {
        match self.state {
            IN | ZE => JsonEvent::ValueInt,
            FR..=E3 => JsonEvent::ValueDouble,
            T3 => JsonEvent::ValueTrue,
            F4 => JsonEvent::ValueFalse,
            N3 => JsonEvent::ValueNull,
            _ => JsonEvent::NeedMoreInput,
        }
    }

    pub fn current_string(&self) -> Result<String, FromUtf8Error> {
        String::from_utf8(self.current_buffer.clone())
    }

    pub fn current_i32(&self) -> Result<i32, Box<dyn Error>> {
        let s = self.current_string()?;
        s.parse().map_err(|e: ParseIntError| e.into())
    }

    pub fn current_i64(&self) -> Result<i64, Box<dyn Error>> {
        let s = self.current_string()?;
        s.parse().map_err(|e: ParseIntError| e.into())
    }

    pub fn current_f64(&self) -> Result<f64, Box<dyn Error>> {
        let s = self.current_string()?;
        s.parse().map_err(|e: ParseFloatError| e.into())
    }
}

impl Default for JsonParser {
    fn default() -> Self {
        Self::new()
    }
}
