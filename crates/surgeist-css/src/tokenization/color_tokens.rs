//! Flat color-bearing CSS token continuation over actual preprocessed scalars.
//!
//! Each input observation, grammar transition, decoded name/digit append and
//! token append is distinct. All variable storage belongs to the caller.

use super::bounded::Scalar;
use super::name::{NameCursor, Step as NameStep, whitespace};
use super::number::{Layout, NumberCursor, Part, Step as NumberStep, number_start};
use super::{identifier_start, name_point, valid_escape};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Span {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NameKind {
    Ident,
    Function,
    Hash { identifier: bool },
    At,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NumberKind {
    Number,
    Percentage,
    Dimension,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TokenKind {
    Name {
        kind: NameKind,
        units: Span,
    },
    Number {
        kind: NumberKind,
        layout: Layout,
        digits: Span,
        unit: Span,
    },
    Whitespace,
    Comment {
        closed: bool,
    },
    String {
        units: Span,
        closed: bool,
    },
    BadString,
    Delimiter(char),
    Cdo,
    Cdc,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) source: Span,
}

/// Each payload is fixed. Names/digits are caller-owned ordered flat records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Record {
    NameUnit(char),
    Digit { exponent: bool, value: u8 },
    Token(Token),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Request {
    ReadScalar { index: usize },
    Compute,
    Append { index: usize, record: Record },
    Done,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Error {
    WrongReply,
    WrongIndex,
    Boundary,
    Overflow,
    Complete,
}

#[derive(Clone, Copy)]
enum NamePurpose {
    Name(NameKind),
    Dimension { layout: Layout, digits: Span },
}

#[derive(Clone, Copy)]
enum State {
    Start,
    Whitespace,
    Comment,
    Name {
        cursor: NameCursor,
        purpose: NamePurpose,
        units_start: usize,
    },
    Number {
        cursor: NumberCursor,
        digits_start: usize,
    },
    Suffix {
        layout: Layout,
        digits: Span,
    },
    String {
        quote: char,
        units_start: usize,
    },
    StringEscape {
        quote: char,
        units_start: usize,
    },
    StringHex {
        quote: char,
        units_start: usize,
        value: u32,
        digits: u8,
    },
    StringHexEmit {
        quote: char,
        units_start: usize,
        value: u32,
    },
    Done,
}

#[derive(Clone, Copy)]
struct Pending {
    record: Record,
    state: State,
    consume: u8,
    next_slot: usize,
}

/// Cached lookahead is four fixed registers (CDO needs four code points).
/// It is neither a copied source nor a second lexical backing store.
pub(crate) struct TokenCursor {
    length: usize,
    read: usize,
    window: [Option<Scalar>; 4],
    cached: u8,
    consumed_end: usize,
    token_start: usize,
    next_slot: usize,
    state: State,
    pending: Option<Pending>,
}

impl TokenCursor {
    pub(crate) const fn new(scalar_length: usize, first_slot: usize) -> Self {
        Self {
            length: scalar_length,
            read: 0,
            window: [None; 4],
            cached: 0,
            consumed_end: 0,
            token_start: 0,
            next_slot: first_slot,
            state: State::Start,
            pending: None,
        }
    }

    pub(crate) fn request(&self) -> Request {
        if let Some(pending) = self.pending {
            return Request::Append {
                index: self.next_slot,
                record: pending.record,
            };
        }
        if matches!(self.state, State::Done) {
            return Request::Done;
        }
        if self.cached < 4 && self.read < self.length {
            return Request::ReadScalar { index: self.read };
        }
        Request::Compute
    }

    /// One actual qualified scalar slot observation, not a borrowed source slice.
    pub(crate) fn read_scalar(&mut self, index: usize, scalar: Scalar) -> Result<(), Error> {
        if self.request() != (Request::ReadScalar { index }) {
            return Err(Error::WrongReply);
        }
        if scalar.end <= scalar.start {
            return Err(Error::Boundary);
        }
        let previous_end = if self.cached == 0 {
            self.consumed_end
        } else {
            self.window[usize::from(self.cached - 1)]
                .ok_or(Error::Boundary)?
                .end
        };
        if scalar.start != previous_end {
            return Err(Error::Boundary);
        }
        let next = self.read.checked_add(1).ok_or(Error::Overflow)?;
        self.window[usize::from(self.cached)] = Some(scalar);
        self.cached += 1;
        self.read = next;
        Ok(())
    }

    /// Commit only after the actual caller push succeeded. A refused push never
    /// calls this, so the same record/index remains available for cleanup/retry.
    pub(crate) fn appended(&mut self, index: usize) -> Result<(), Error> {
        let pending = self.pending.ok_or(Error::WrongReply)?;
        if index != self.next_slot {
            return Err(Error::WrongIndex);
        }
        self.consume(pending.consume)?;
        self.state = pending.state;
        self.next_slot = pending.next_slot;
        self.pending = None;
        Ok(())
    }

    /// One bounded grammar decision over cached lookahead. It can request one
    /// append, or perform one fixed transition; it never loops to a next token.
    pub(crate) fn compute(&mut self) -> Result<(), Error> {
        if self.request() != Request::Compute {
            return Err(Error::WrongReply);
        }
        let [a, b, c, d] = self.window.map(|point| point.map(|point| point.value));
        match self.state {
            State::Start => {
                self.token_start = self.window[0].map_or(self.consumed_end, |point| point.start);
                let Some(value) = a else {
                    self.state = State::Done;
                    return Ok(());
                };
                if whitespace(value) {
                    self.state = State::Whitespace;
                } else if [a, b] == [Some('/'), Some('*')] {
                    self.consume(2)?;
                    self.state = State::Comment;
                } else if [a, b, c, d] == [Some('<'), Some('!'), Some('-'), Some('-')] {
                    self.token(TokenKind::Cdo, 4)?;
                } else if [a, b, c] == [Some('-'), Some('-'), Some('>')] {
                    self.token(TokenKind::Cdc, 3)?;
                } else if number_start([a, b, c]) {
                    self.state = State::Number {
                        cursor: NumberCursor::new(),
                        digits_start: self.next_slot,
                    };
                } else if identifier_start([a, b, c]) {
                    self.state = State::Name {
                        cursor: NameCursor::new(),
                        purpose: NamePurpose::Name(NameKind::Ident),
                        units_start: self.next_slot,
                    };
                } else if value == '#' && (b.is_some_and(name_point) || valid_escape(b, c)) {
                    self.consume(1)?;
                    self.state = State::Name {
                        cursor: NameCursor::new(),
                        purpose: NamePurpose::Name(NameKind::Hash {
                            identifier: identifier_start([b, c, d]),
                        }),
                        units_start: self.next_slot,
                    };
                } else if value == '@' && identifier_start([b, c, d]) {
                    self.consume(1)?;
                    self.state = State::Name {
                        cursor: NameCursor::new(),
                        purpose: NamePurpose::Name(NameKind::At),
                        units_start: self.next_slot,
                    };
                } else if matches!(value, '\'' | '"') {
                    self.consume(1)?;
                    self.state = State::String {
                        quote: value,
                        units_start: self.next_slot,
                    };
                } else {
                    self.token(TokenKind::Delimiter(value), 1)?;
                }
            }
            State::Whitespace => {
                if a.is_some_and(whitespace) {
                    self.consume(1)?;
                } else {
                    self.token(TokenKind::Whitespace, 0)?;
                }
            }
            State::Comment => {
                if [a, b] == [Some('*'), Some('/')] {
                    self.token(TokenKind::Comment { closed: true }, 2)?;
                } else if a.is_none() {
                    self.token(TokenKind::Comment { closed: false }, 0)?;
                } else {
                    self.consume(1)?;
                }
            }
            State::Name {
                mut cursor,
                purpose,
                units_start,
            } => {
                let action = cursor.action([a, b]);
                match action.step() {
                    NameStep::Consume { count } => {
                        action.complete();
                        self.consume(count)?;
                        self.state = State::Name {
                            cursor,
                            purpose,
                            units_start,
                        };
                    }
                    NameStep::Emit { value, count } => {
                        action.complete();
                        self.append(
                            Record::NameUnit(value),
                            State::Name {
                                cursor,
                                purpose,
                                units_start,
                            },
                            count,
                        )?;
                    }
                    NameStep::Done => {
                        let units = Span {
                            start: units_start,
                            end: self.next_slot,
                        };
                        let (kind, consume) = match purpose {
                            NamePurpose::Name(NameKind::Ident) if a == Some('(') => (
                                TokenKind::Name {
                                    kind: NameKind::Function,
                                    units,
                                },
                                1,
                            ),
                            NamePurpose::Name(kind) => (TokenKind::Name { kind, units }, 0),
                            NamePurpose::Dimension { layout, digits } => (
                                TokenKind::Number {
                                    kind: NumberKind::Dimension,
                                    layout,
                                    digits,
                                    unit: units,
                                },
                                0,
                            ),
                        };
                        self.token(kind, consume)?;
                    }
                }
            }
            State::Number {
                mut cursor,
                digits_start,
            } => match cursor.step([a, b, c]).map_err(|_| Error::Overflow)? {
                NumberStep::Consume(Part::Mantissa(value)) => {
                    self.append(
                        Record::Digit {
                            exponent: false,
                            value,
                        },
                        State::Number {
                            cursor,
                            digits_start,
                        },
                        1,
                    )?;
                }
                NumberStep::Consume(Part::ExponentDigit(value)) => {
                    self.append(
                        Record::Digit {
                            exponent: true,
                            value,
                        },
                        State::Number {
                            cursor,
                            digits_start,
                        },
                        1,
                    )?;
                }
                NumberStep::Consume(_) => {
                    self.consume(1)?;
                    self.state = State::Number {
                        cursor,
                        digits_start,
                    };
                }
                NumberStep::Transition => {
                    self.state = State::Number {
                        cursor,
                        digits_start,
                    };
                }
                NumberStep::Done(layout) => {
                    self.state = State::Suffix {
                        layout,
                        digits: Span {
                            start: digits_start,
                            end: self.next_slot,
                        },
                    };
                }
            },
            State::Suffix { layout, digits } => {
                if a == Some('%') {
                    self.token(
                        TokenKind::Number {
                            kind: NumberKind::Percentage,
                            layout,
                            digits,
                            unit: Span {
                                start: self.next_slot,
                                end: self.next_slot,
                            },
                        },
                        1,
                    )?;
                } else if identifier_start([a, b, c]) {
                    self.state = State::Name {
                        cursor: NameCursor::new(),
                        purpose: NamePurpose::Dimension { layout, digits },
                        units_start: self.next_slot,
                    };
                } else {
                    self.token(
                        TokenKind::Number {
                            kind: NumberKind::Number,
                            layout,
                            digits,
                            unit: Span {
                                start: self.next_slot,
                                end: self.next_slot,
                            },
                        },
                        0,
                    )?;
                }
            }
            State::String { quote, units_start } => match a {
                Some(value) if value == quote => self.token(
                    TokenKind::String {
                        units: Span {
                            start: units_start,
                            end: self.next_slot,
                        },
                        closed: true,
                    },
                    1,
                )?,
                None => self.token(
                    TokenKind::String {
                        units: Span {
                            start: units_start,
                            end: self.next_slot,
                        },
                        closed: false,
                    },
                    0,
                )?,
                Some('\n') => self.token(TokenKind::BadString, 0)?,
                Some('\\') => {
                    self.consume(1)?;
                    self.state = State::StringEscape { quote, units_start };
                }
                Some(value) => self.append(Record::NameUnit(value), self.state, 1)?,
            },
            State::StringEscape { quote, units_start } => match a {
                None => {
                    self.state = State::String { quote, units_start };
                }
                Some('\n') => {
                    self.consume(1)?;
                    self.state = State::String { quote, units_start };
                }
                Some(value) if value.is_ascii_hexdigit() => {
                    self.consume(1)?;
                    self.state = State::StringHex {
                        quote,
                        units_start,
                        value: value.to_digit(16).expect("hex check"),
                        digits: 1,
                    };
                }
                Some(value) => self.append(
                    Record::NameUnit(value),
                    State::String { quote, units_start },
                    1,
                )?,
            },
            State::StringHex {
                quote,
                units_start,
                value,
                digits,
            } => {
                if digits < 6 && a.is_some_and(|value| value.is_ascii_hexdigit()) {
                    self.consume(1)?;
                    self.state = State::StringHex {
                        quote,
                        units_start,
                        value: value * 16
                            + a.and_then(|value| value.to_digit(16)).expect("hex check"),
                        digits: digits + 1,
                    };
                } else {
                    self.consume(u8::from(a.is_some_and(whitespace)))?;
                    self.state = State::StringHexEmit {
                        quote,
                        units_start,
                        value,
                    };
                }
            }
            State::StringHexEmit {
                quote,
                units_start,
                value,
            } => {
                self.append(
                    Record::NameUnit(if value == 0 {
                        '\u{fffd}'
                    } else {
                        char::from_u32(value).unwrap_or('\u{fffd}')
                    }),
                    State::String { quote, units_start },
                    0,
                )?;
            }
            State::Done => return Err(Error::Complete),
        }
        Ok(())
    }

    fn append(&mut self, record: Record, state: State, consume: u8) -> Result<(), Error> {
        if consume > self.cached {
            return Err(Error::Boundary);
        }
        let next_slot = self.next_slot.checked_add(1).ok_or(Error::Overflow)?;
        self.pending = Some(Pending {
            record,
            state,
            consume,
            next_slot,
        });
        Ok(())
    }

    fn token(&mut self, kind: TokenKind, consume: u8) -> Result<(), Error> {
        let end = if consume == 0 {
            self.consumed_end
        } else {
            self.window[usize::from(consume - 1)]
                .ok_or(Error::Boundary)?
                .end
        };
        self.append(
            Record::Token(Token {
                kind,
                source: Span {
                    start: self.token_start,
                    end,
                },
            }),
            State::Start,
            consume,
        )
    }

    fn consume(&mut self, count: u8) -> Result<(), Error> {
        if count > self.cached {
            return Err(Error::Boundary);
        }
        if count == 0 {
            return Ok(());
        }
        self.consumed_end = self.window[usize::from(count - 1)]
            .ok_or(Error::Boundary)?
            .end;
        let count = usize::from(count);
        self.window = [
            self.window.get(count).copied().flatten(),
            self.window.get(count + 1).copied().flatten(),
            self.window.get(count + 2).copied().flatten(),
            self.window.get(count + 3).copied().flatten(),
        ];
        self.cached -= count as u8;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(text: &str) -> Vec<Record> {
        let points: Vec<_> = text
            .char_indices()
            .map(|(start, value)| Scalar {
                value,
                start,
                end: start + value.len_utf8(),
            })
            .collect();
        let mut cursor = TokenCursor::new(points.len(), 0);
        let mut records = Vec::new();
        loop {
            match cursor.request() {
                Request::ReadScalar { index } => cursor.read_scalar(index, points[index]).unwrap(),
                Request::Compute => cursor.compute().unwrap(),
                Request::Append { index, record } => {
                    assert_eq!(index, records.len());
                    records.push(record);
                    cursor.appended(index).unwrap();
                }
                Request::Done => return records,
            }
        }
    }

    fn names(records: &[Record], span: Span) -> String {
        records[span.start..span.end]
            .iter()
            .map(|record| match record {
                Record::NameUnit(value) => *value,
                _ => panic!("name unit"),
            })
            .collect()
    }

    #[test]
    fn escaped_function_names_and_hashes_have_exact_source_intervals() {
        let records = tokens("\\72 gb( #f00 )");
        let tokens: Vec<_> = records
            .iter()
            .filter_map(|record| {
                if let Record::Token(token) = record {
                    Some(*token)
                } else {
                    None
                }
            })
            .collect();
        let TokenKind::Name {
            kind: NameKind::Function,
            units,
        } = tokens[0].kind
        else {
            panic!("function")
        };
        assert_eq!(names(&records, units), "rgb");
        assert_eq!(tokens[0].source, Span { start: 0, end: 7 });
        let TokenKind::Name {
            kind: NameKind::Hash { identifier: true },
            units,
        } = tokens[2].kind
        else {
            panic!("hash")
        };
        assert_eq!(names(&records, units), "f00");
        assert_eq!(tokens[2].source, Span { start: 8, end: 12 });
        assert_eq!(tokens[4].kind, TokenKind::Delimiter(')'));
    }

    #[test]
    fn numeric_digits_are_flat_checked_and_not_converted_by_the_lexer() {
        let records = tokens("-12.30e+04deg .5% 1e+x");
        let tokens: Vec<_> = records
            .iter()
            .filter_map(|record| {
                if let Record::Token(token) = record {
                    Some(*token)
                } else {
                    None
                }
            })
            .collect();
        let TokenKind::Number {
            kind: NumberKind::Dimension,
            layout,
            digits,
            unit,
        } = tokens[0].kind
        else {
            panic!("dimension")
        };
        assert_eq!(
            layout,
            Layout {
                negative: true,
                mantissa_digits: 4,
                fraction_digits: 2,
                exponent_negative: false,
                exponent_digits: 2
            }
        );
        assert_eq!(
            records[digits.start..digits.end],
            [
                Record::Digit {
                    exponent: false,
                    value: 1
                },
                Record::Digit {
                    exponent: false,
                    value: 2
                },
                Record::Digit {
                    exponent: false,
                    value: 3
                },
                Record::Digit {
                    exponent: false,
                    value: 0
                },
                Record::Digit {
                    exponent: true,
                    value: 0
                },
                Record::Digit {
                    exponent: true,
                    value: 4
                }
            ]
        );
        assert_eq!(names(&records, unit), "deg");
        assert!(matches!(
            tokens[2].kind,
            TokenKind::Number {
                kind: NumberKind::Percentage,
                ..
            }
        ));
        let TokenKind::Number {
            kind: NumberKind::Dimension,
            layout,
            unit,
            ..
        } = tokens[4].kind
        else {
            panic!("unconsumed exponent becomes dimension")
        };
        assert_eq!(layout.exponent_digits, 0);
        assert_eq!(names(&records, unit), "e");
    }

    #[test]
    fn comments_strings_bad_strings_and_cdo_cdc_keep_boundaries() {
        let records = tokens("/* x */'a\\62 ' \"x\n<!-- -->/*");
        let tokens: Vec<_> = records
            .iter()
            .filter_map(|record| {
                if let Record::Token(token) = record {
                    Some(*token)
                } else {
                    None
                }
            })
            .collect();
        assert_eq!(
            tokens[0],
            Token {
                kind: TokenKind::Comment { closed: true },
                source: Span { start: 0, end: 7 }
            }
        );
        let TokenKind::String {
            units,
            closed: true,
        } = tokens[1].kind
        else {
            panic!("string")
        };
        assert_eq!(names(&records, units), "ab");
        assert_eq!(tokens[3].kind, TokenKind::BadString);
        assert!(tokens.iter().any(|token| token.kind == TokenKind::Cdo));
        assert!(tokens.iter().any(|token| token.kind == TokenKind::Cdc));
        assert_eq!(
            tokens.last().unwrap().kind,
            TokenKind::Comment { closed: false }
        );
    }

    #[test]
    fn actual_refused_push_preserves_generated_record_and_boundary() {
        let mut cursor = TokenCursor::new(1, 7);
        cursor
            .read_scalar(
                0,
                Scalar {
                    value: 'r',
                    start: 0,
                    end: 1,
                },
            )
            .unwrap();
        cursor.compute().unwrap();
        cursor.compute().unwrap();
        let before = cursor.request();
        assert_eq!(
            before,
            Request::Append {
                index: 7,
                record: Record::NameUnit('r')
            }
        );
        assert_eq!(cursor.appended(6), Err(Error::WrongIndex));
        assert_eq!(cursor.compute(), Err(Error::WrongReply));
        assert_eq!(cursor.request(), before);
        cursor.appended(7).unwrap();
        cursor.compute().unwrap();
        assert!(matches!(
            cursor.request(),
            Request::Append {
                index: 8,
                record: Record::Token(_)
            }
        ));
    }
}
