//! The one state machine of CSS Syntax Module Level 3 section 4.3.
//!
//! [`step`] is the only implementation of the tokenizer's decisions, and
//! [`start_kind`], [`name_end_kind`] and [`suffix_end`] are the decisions it
//! shares with a driver that wants to take them without a step. It reads the
//! next scalars through a [`Lookahead`], looks at its [`State`], and says what
//! to do in an [`Act`]; it changes nothing itself. Two drivers apply the acts:
//!
//! - the stepwise [`TokenCursor`](super::protocol::TokenCursor), which turns
//!   each act into a host request and keeps the cursor unchanged until the
//!   host acknowledges it; and
//! - the free-running [`Lexer`](super::Lexer), which applies each act at once
//!   and, where a run of scalars would be consumed by an unbroken series of the
//!   same transitions, consumes the whole run in one go.
//!
//! Both read preprocessed scalars (section 3.3). A [`Fault`] reports, beside
//! the act, that the transition is one the specification marks as a parse
//! error and that no token records on its own; it is not part of the machine's
//! state.

use super::protocol::name::{NameCursor, Step as NameStep};
use super::protocol::{
    NameKind, NumberCursor, NumberKind, NumberLayout, NumberPart, NumberStep, ProtocolError,
    RawToken, RawTokenKind, Record, Span,
};
use crate::input::Scalar;
use crate::lexical::{
    is_ident_code_point, is_ident_start, is_non_printable, is_whitespace, valid_escape,
    would_start_ident, would_start_number, would_start_unicode_range,
};

/// What the machine is in the middle of.
#[derive(Clone, Copy, Debug)]
pub(crate) enum NamePurpose {
    Name(NameKind),
    Dimension { layout: NumberLayout, digits: Span },
}

/// Which token an escape belongs to, and where it returns.
#[derive(Clone, Copy, Debug)]
pub(crate) enum EscapeTarget {
    String { quote: char, units_start: usize },
    Url { units_start: usize },
    BadUrl,
}

impl EscapeTarget {
    const fn continuation(self) -> State {
        match self {
            Self::String { quote, units_start } => State::String { quote, units_start },
            Self::Url { units_start } => State::Url { units_start },
            Self::BadUrl => State::BadUrl,
        }
    }
}

/// The machine's registers between two steps.
#[derive(Clone, Copy, Debug)]
pub(crate) enum State {
    Start,
    Whitespace,
    Comment,
    Name {
        cursor: NameCursor,
        purpose: NamePurpose,
        units_start: usize,
        url_match: u8,
    },
    Number {
        cursor: NumberCursor,
        digits_start: usize,
    },
    Suffix {
        layout: NumberLayout,
        digits: Span,
    },
    String {
        quote: char,
        units_start: usize,
    },
    UrlStart {
        name: Span,
        opening_end: usize,
    },
    QuotedUrlWhitespace {
        source: Span,
    },
    Url {
        units_start: usize,
    },
    UrlWhitespace {
        units_start: usize,
    },
    BadUrl,
    Escape {
        target: EscapeTarget,
    },
    Hex {
        target: EscapeTarget,
        value: u32,
        digits: u8,
    },
    HexEmit {
        target: EscapeTarget,
        value: u32,
    },
    Done,
}

/// The positions a step needs from its driver.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Position {
    /// The slot the next appended record takes.
    pub(crate) next_slot: usize,
    /// One past the last consumed original unit.
    pub(crate) consumed_end: usize,
    /// The first original unit of the token in progress.
    pub(crate) token_start: usize,
}

/// The scalars at and after the next input code point, nearest first.
///
/// Index 0 is the next input code point. `None` is the EOF code point.
pub(crate) trait Lookahead {
    fn scalar(&mut self, index: usize) -> Option<Scalar>;

    fn point(&mut self, index: usize) -> Option<char> {
        self.scalar(index).map(|scalar| scalar.value)
    }
}

/// What a transition does.
#[derive(Clone, Copy, Debug)]
pub(crate) enum Act {
    /// The input is exhausted.
    Finish,
    /// Consume `consume` scalars and take `state`.
    Move { state: State, consume: u8 },
    /// Store `record`, then consume `consume` scalars and take `state`.
    Emit {
        record: Record,
        state: State,
        consume: u8,
    },
    /// Consume a unicode-range token (section 4.3.14) whose first code point
    /// is the next input code point. Only returned when the driver allows
    /// unicode ranges.
    UnicodeRange,
}

/// A transition the specification calls a parse error that no token records.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Fault {
    /// A U+005C REVERSE SOLIDUS that does not start a valid escape (section
    /// 4.3.1).
    EscapedNewline,
    /// An escape whose backslash is the last code point (section 4.3.7).
    EscapedEof,
    /// A url token that turns into a bad url (section 4.3.6) for a reason the
    /// specification marks as a parse error.
    BadUrl,
}

/// One transition.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Step {
    pub(crate) act: Act,
    pub(crate) fault: Option<Fault>,
}

impl Step {
    const fn of(act: Act) -> Self {
        Self { act, fault: None }
    }

    const fn with(act: Act, fault: Fault) -> Self {
        Self {
            act,
            fault: Some(fault),
        }
    }
}

const fn name_state(purpose: NamePurpose, units_start: usize) -> State {
    State::Name {
        cursor: NameCursor::new(),
        purpose,
        units_start,
        url_match: 0,
    }
}

fn token<L: Lookahead>(
    look: &mut L,
    at: Position,
    kind: RawTokenKind,
    consume: u8,
) -> Result<Act, ProtocolError> {
    let end = if consume == 0 {
        at.consumed_end
    } else {
        look.scalar(usize::from(consume - 1))
            .ok_or(ProtocolError::BOUNDARY)?
            .end
    };
    Ok(Act::Emit {
        record: Record::Token(RawToken {
            kind,
            source: Span {
                start: at.token_start,
                end,
            },
        }),
        state: State::Start,
        consume,
    })
}

pub(crate) const fn plain_number(
    layout: NumberLayout,
    digits: Span,
    unit: Span,
    kind: NumberKind,
) -> RawTokenKind {
    RawTokenKind::Number {
        kind,
        layout,
        digits,
        unit,
    }
}

/// How an ident sequence ends, by whether a left parenthesis follows it
/// (sections 4.3.1, 4.3.3 and 4.3.4).
#[derive(Clone, Copy, Debug)]
pub(crate) enum NameEnd {
    /// The name is `url` and a parenthesis follows: consume it and read the
    /// url's content.
    Url,
    /// The token the name makes, and how many scalars after the name (the
    /// parenthesis of a function) it takes.
    Token { kind: RawTokenKind, consume: u8 },
}

/// How much of `url` the units spelled so far match after one more unit
/// (section 4.3.4 selects url consumption for an ASCII case-insensitive match):
/// 0 to 3 letters of it, or 4 once they cannot be it.
pub(crate) const fn next_url_match(matched: u8, unit: char) -> u8 {
    match (matched, unit.to_ascii_lowercase()) {
        (0, 'u') | (1, 'r') | (2, 'l') => matched + 1,
        _ => 4,
    }
}

/// The decision of 4.3.4 when an ident sequence ends, without the spans.
#[derive(Clone, Copy, Debug)]
pub(crate) enum NameEndKind {
    /// The name is `url` and a parenthesis follows.
    Url,
    /// A function token: the name and its parenthesis.
    Function,
    /// The name-like token of this kind, without a following scalar.
    Plain(NameKind),
    /// A dimension's unit.
    Dimension,
}

pub(crate) const fn name_end_kind(
    purpose: &NamePurpose,
    url_match: u8,
    paren: bool,
) -> NameEndKind {
    match purpose {
        NamePurpose::Name(NameKind::Ident) if paren && url_match == 3 => NameEndKind::Url,
        NamePurpose::Name(NameKind::Ident) if paren => NameEndKind::Function,
        NamePurpose::Name(kind) => NameEndKind::Plain(*kind),
        NamePurpose::Dimension { .. } => NameEndKind::Dimension,
    }
}

/// The decision of 4.3.4 when an ident sequence ends. `units` is the span of its
/// decoded units and `url_match` how much of `url` they spell.
pub(crate) const fn name_end(
    purpose: NamePurpose,
    units: Span,
    url_match: u8,
    paren: bool,
) -> NameEnd {
    match name_end_kind(&purpose, url_match, paren) {
        NameEndKind::Url => NameEnd::Url,
        NameEndKind::Function => NameEnd::Token {
            kind: RawTokenKind::Name {
                kind: NameKind::Function,
                units,
            },
            consume: 1,
        },
        NameEndKind::Plain(kind) => NameEnd::Token {
            kind: RawTokenKind::Name { kind, units },
            consume: 0,
        },
        NameEndKind::Dimension => match purpose {
            NamePurpose::Dimension { layout, digits } => NameEnd::Token {
                kind: plain_number(layout, digits, units, NumberKind::Dimension),
                consume: 0,
            },
            NamePurpose::Name(kind) => NameEnd::Token {
                kind: RawTokenKind::Name { kind, units },
                consume: 0,
            },
        },
    }
}

/// What follows a number (section 4.3.3).
#[derive(Clone, Copy, Debug)]
pub(crate) enum SuffixEnd {
    /// A percent sign: a percentage.
    Percentage,
    /// An ident sequence: a dimension.
    Dimension,
    /// Neither: a number.
    Number,
}

pub(crate) const fn suffix_end(
    first: Option<char>,
    second: Option<char>,
    third: Option<char>,
) -> SuffixEnd {
    if matches!(first, Some('%')) {
        SuffixEnd::Percentage
    } else if would_start_ident(first, second, third) {
        SuffixEnd::Dimension
    } else {
        SuffixEnd::Number
    }
}

/// The transition out of `state`, given the scalars ahead.
///
/// `ranges_allowed` is section 4.3.1's `unicode ranges allowed`.
pub(crate) fn step<L: Lookahead>(
    state: State,
    look: &mut L,
    at: Position,
    ranges_allowed: bool,
) -> Result<Step, ProtocolError> {
    let a = look.point(0);
    Ok(match state {
        State::Start => {
            let Some(value) = a else {
                return Ok(Step::of(Act::Finish));
            };
            return start(value, look, at, ranges_allowed);
        }
        State::Whitespace => {
            if a.is_some_and(is_whitespace) {
                Step::of(Act::Move { state, consume: 1 })
            } else {
                Step::of(token(look, at, RawTokenKind::Whitespace, 0)?)
            }
        }
        State::Comment => {
            if a == Some('*') && look.point(1) == Some('/') {
                Step::of(token(look, at, RawTokenKind::Comment { closed: true }, 2)?)
            } else if a.is_none() {
                Step::of(token(look, at, RawTokenKind::Comment { closed: false }, 0)?)
            } else {
                Step::of(Act::Move { state, consume: 1 })
            }
        }
        State::Name {
            mut cursor,
            purpose,
            units_start,
            url_match,
        } => {
            let b = look.point(1);
            let escape_at_eof = cursor.is_in_escape() && a.is_none();
            let action = cursor.action([a, b]);
            match action.step() {
                NameStep::Consume { count } => {
                    action.complete();
                    Step::of(Act::Move {
                        state: State::Name {
                            cursor,
                            purpose,
                            units_start,
                            url_match,
                        },
                        consume: count,
                    })
                }
                NameStep::Emit { value, count } => {
                    action.complete();
                    let act = Act::Emit {
                        record: Record::NameUnit(value),
                        state: State::Name {
                            cursor,
                            purpose,
                            units_start,
                            url_match: next_url_match(url_match, value),
                        },
                        consume: count,
                    };
                    if escape_at_eof {
                        Step::with(act, Fault::EscapedEof)
                    } else {
                        Step::of(act)
                    }
                }
                NameStep::Done => {
                    let units = Span {
                        start: units_start,
                        end: at.next_slot,
                    };
                    match name_end(purpose, units, url_match, a == Some('(')) {
                        NameEnd::Url => {
                            let opening_end = look.scalar(0).ok_or(ProtocolError::BOUNDARY)?.end;
                            Step::of(Act::Move {
                                state: State::UrlStart {
                                    name: units,
                                    opening_end,
                                },
                                consume: 1,
                            })
                        }
                        NameEnd::Token { kind, consume } => {
                            Step::of(token(look, at, kind, consume)?)
                        }
                    }
                }
            }
        }
        State::Number {
            mut cursor,
            digits_start,
        } => {
            let window = [a, look.point(1), look.point(2)];
            match cursor.step(window).map_err(|_| ProtocolError::OVERFLOW)? {
                NumberStep::Consume(NumberPart::Mantissa(value)) => Step::of(Act::Emit {
                    record: Record::Digit {
                        exponent: false,
                        value,
                    },
                    state: State::Number {
                        cursor,
                        digits_start,
                    },
                    consume: 1,
                }),
                NumberStep::Consume(NumberPart::ExponentDigit(value)) => Step::of(Act::Emit {
                    record: Record::Digit {
                        exponent: true,
                        value,
                    },
                    state: State::Number {
                        cursor,
                        digits_start,
                    },
                    consume: 1,
                }),
                NumberStep::Consume(_) => Step::of(Act::Move {
                    state: State::Number {
                        cursor,
                        digits_start,
                    },
                    consume: 1,
                }),
                NumberStep::Transition => Step::of(Act::Move {
                    state: State::Number {
                        cursor,
                        digits_start,
                    },
                    consume: 0,
                }),
                NumberStep::Done(layout) => Step::of(Act::Move {
                    state: State::Suffix {
                        layout,
                        digits: Span {
                            start: digits_start,
                            end: at.next_slot,
                        },
                    },
                    consume: 0,
                }),
            }
        }
        State::Suffix { layout, digits } => {
            let empty = Span {
                start: at.next_slot,
                end: at.next_slot,
            };
            match suffix_end(a, look.point(1), look.point(2)) {
                SuffixEnd::Percentage => Step::of(token(
                    look,
                    at,
                    plain_number(layout, digits, empty, NumberKind::Percentage),
                    1,
                )?),
                SuffixEnd::Dimension => Step::of(Act::Move {
                    state: name_state(NamePurpose::Dimension { layout, digits }, at.next_slot),
                    consume: 0,
                }),
                SuffixEnd::Number => Step::of(token(
                    look,
                    at,
                    plain_number(layout, digits, empty, NumberKind::Number),
                    0,
                )?),
            }
        }
        State::String { quote, units_start } => {
            let units = Span {
                start: units_start,
                end: at.next_slot,
            };
            match a {
                Some(value) if value == quote => Step::of(token(
                    look,
                    at,
                    RawTokenKind::String {
                        units,
                        closed: true,
                    },
                    1,
                )?),
                None => Step::of(token(
                    look,
                    at,
                    RawTokenKind::String {
                        units,
                        closed: false,
                    },
                    0,
                )?),
                Some('\n') => Step::of(token(look, at, RawTokenKind::BadString, 0)?),
                Some('\\') => Step::of(Act::Move {
                    state: State::Escape {
                        target: EscapeTarget::String { quote, units_start },
                    },
                    consume: 1,
                }),
                Some(value) => Step::of(Act::Emit {
                    record: Record::NameUnit(value),
                    state,
                    consume: 1,
                }),
            }
        }
        State::UrlStart { name, opening_end } => {
            if a.is_some_and(is_whitespace) {
                Step::of(Act::Move { state, consume: 1 })
            } else if matches!(a, Some('\'' | '"')) {
                // Recognition may cross arbitrary whitespace, but the
                // Function's opening ends at '(' and the actual whitespace
                // retains its own source interval. No input is replayed.
                let next = if at.consumed_end == opening_end {
                    State::Start
                } else {
                    State::QuotedUrlWhitespace {
                        source: Span {
                            start: opening_end,
                            end: at.consumed_end,
                        },
                    }
                };
                Step::of(Act::Emit {
                    record: Record::Token(RawToken {
                        kind: RawTokenKind::Name {
                            kind: NameKind::Function,
                            units: name,
                        },
                        source: Span {
                            start: at.token_start,
                            end: opening_end,
                        },
                    }),
                    state: next,
                    consume: 0,
                })
            } else {
                Step::of(Act::Move {
                    state: State::Url {
                        units_start: at.next_slot,
                    },
                    consume: 0,
                })
            }
        }
        State::QuotedUrlWhitespace { source } => Step::of(Act::Emit {
            record: Record::Token(RawToken {
                kind: RawTokenKind::Whitespace,
                source,
            }),
            state: State::Start,
            consume: 0,
        }),
        State::Url { units_start } => {
            let units = Span {
                start: units_start,
                end: at.next_slot,
            };
            match a {
                Some(')') | None => Step::of(token(
                    look,
                    at,
                    RawTokenKind::Url {
                        units,
                        closed: a.is_some(),
                    },
                    u8::from(a.is_some()),
                )?),
                Some(value) if is_whitespace(value) => Step::of(Act::Move {
                    state: State::UrlWhitespace { units_start },
                    consume: 1,
                }),
                Some(value) if matches!(value, '"' | '\'' | '(') || is_non_printable(value) => {
                    Step::with(
                        Act::Move {
                            state: State::BadUrl,
                            consume: 1,
                        },
                        Fault::BadUrl,
                    )
                }
                Some('\\') => {
                    if valid_escape(a, look.point(1)) {
                        Step::of(Act::Move {
                            state: State::Escape {
                                target: EscapeTarget::Url { units_start },
                            },
                            consume: 1,
                        })
                    } else {
                        Step::with(
                            Act::Move {
                                state: State::BadUrl,
                                consume: 1,
                            },
                            Fault::BadUrl,
                        )
                    }
                }
                Some(value) => Step::of(Act::Emit {
                    record: Record::NameUnit(value),
                    state,
                    consume: 1,
                }),
            }
        }
        State::UrlWhitespace { units_start } => {
            if a.is_some_and(is_whitespace) {
                Step::of(Act::Move { state, consume: 1 })
            } else if matches!(a, Some(')') | None) {
                Step::of(token(
                    look,
                    at,
                    RawTokenKind::Url {
                        units: Span {
                            start: units_start,
                            end: at.next_slot,
                        },
                        closed: a.is_some(),
                    },
                    u8::from(a.is_some()),
                )?)
            } else {
                // Section 4.3.6 marks no parse error for whitespace followed
                // by anything but ")" or EOF.
                Step::of(Act::Move {
                    state: State::BadUrl,
                    consume: 0,
                })
            }
        }
        State::BadUrl => {
            if matches!(a, Some(')') | None) {
                Step::of(token(
                    look,
                    at,
                    RawTokenKind::BadUrl {
                        closed: a.is_some(),
                    },
                    u8::from(a.is_some()),
                )?)
            } else if valid_escape(a, look.point(1)) {
                Step::of(Act::Move {
                    state: State::Escape {
                        target: EscapeTarget::BadUrl,
                    },
                    consume: 1,
                })
            } else {
                Step::of(Act::Move { state, consume: 1 })
            }
        }
        State::Escape { target } => match a {
            None if matches!(target, EscapeTarget::Url { .. }) => Step::with(
                Act::Emit {
                    record: Record::NameUnit('\u{fffd}'),
                    state: target.continuation(),
                    consume: 0,
                },
                Fault::EscapedEof,
            ),
            None if matches!(target, EscapeTarget::BadUrl) => Step::with(
                Act::Move {
                    state: target.continuation(),
                    consume: 0,
                },
                Fault::EscapedEof,
            ),
            // In a string a backslash before EOF is simply dropped.
            None => Step::of(Act::Move {
                state: target.continuation(),
                consume: 0,
            }),
            Some('\n') if matches!(target, EscapeTarget::String { .. }) => Step::of(Act::Move {
                state: target.continuation(),
                consume: 1,
            }),
            Some(value) if value.is_ascii_hexdigit() => Step::of(Act::Move {
                state: State::Hex {
                    target,
                    value: value.to_digit(16).ok_or(ProtocolError::BOUNDARY)?,
                    digits: 1,
                },
                consume: 1,
            }),
            Some(_) if matches!(target, EscapeTarget::BadUrl) => Step::of(Act::Move {
                state: target.continuation(),
                consume: 1,
            }),
            Some(value) => Step::of(Act::Emit {
                record: Record::NameUnit(value),
                state: target.continuation(),
                consume: 1,
            }),
        },
        State::Hex {
            target,
            value,
            digits,
        } => {
            if digits < 6 && a.is_some_and(|value| value.is_ascii_hexdigit()) {
                let digit = a
                    .and_then(|value| value.to_digit(16))
                    .ok_or(ProtocolError::BOUNDARY)?;
                Step::of(Act::Move {
                    state: State::Hex {
                        target,
                        value: value * 16 + digit,
                        digits: digits + 1,
                    },
                    consume: 1,
                })
            } else {
                Step::of(Act::Move {
                    state: State::HexEmit { target, value },
                    consume: u8::from(a.is_some_and(is_whitespace)),
                })
            }
        }
        State::HexEmit {
            target: EscapeTarget::BadUrl,
            ..
        } => Step::of(Act::Move {
            state: State::BadUrl,
            consume: 0,
        }),
        State::HexEmit { target, value } => Step::of(Act::Emit {
            record: Record::NameUnit(if value == 0 {
                '\u{fffd}'
            } else {
                char::from_u32(value).unwrap_or('\u{fffd}')
            }),
            state: target.continuation(),
            consume: 0,
        }),
        State::Done => return Err(ProtocolError::COMPLETE),
    })
}

/// What section 4.3.1 decides on the first code point of a token.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StartKind {
    /// Whitespace: consume as much as possible.
    Whitespace,
    /// A comment opening, "/*".
    Comment,
    /// "<!--".
    Cdo,
    /// "-->".
    Cdc,
    /// A numeric token.
    Number,
    /// An ident-like token: an ident start code point, or a valid escape.
    Ident,
    /// A hash token with this type flag; the number sign is consumed first.
    Hash { identifier: bool },
    /// An at-keyword; the commercial at is consumed first.
    AtKeyword,
    /// A string with this quote; the quote is consumed first.
    String,
    /// A unicode-range token (section 4.3.14).
    UnicodeRange,
    /// A delim token with the first code point.
    Delim,
    /// A backslash that starts no valid escape: a delim token and a parse
    /// error.
    BadEscape,
}

/// Section 4.3.1, consume a token: the decision on the first code point,
/// `value`, given the code points after it. It consumes nothing.
pub(crate) fn start_kind<L: Lookahead>(
    value: char,
    look: &mut L,
    ranges_allowed: bool,
) -> StartKind {
    let a = Some(value);
    match value {
        ' ' | '\t' | '\n' => StartKind::Whitespace,
        '/' if look.point(1) == Some('*') => StartKind::Comment,
        '<' if [look.point(1), look.point(2), look.point(3)]
            == [Some('!'), Some('-'), Some('-')] =>
        {
            StartKind::Cdo
        }
        '-' | '+' | '.' | '0'..='9' => {
            let (b, c) = (look.point(1), look.point(2));
            if value == '-' && [b, c] == [Some('-'), Some('>')] {
                StartKind::Cdc
            } else if would_start_number(a, b, c) {
                StartKind::Number
            } else if would_start_ident(a, b, c) {
                StartKind::Ident
            } else {
                StartKind::Delim
            }
        }
        'u' | 'U'
            if ranges_allowed && would_start_unicode_range(a, look.point(1), look.point(2)) =>
        {
            StartKind::UnicodeRange
        }
        '#' => {
            let (b, c, d) = (look.point(1), look.point(2), look.point(3));
            if b.is_some_and(is_ident_code_point) || valid_escape(b, c) {
                StartKind::Hash {
                    identifier: would_start_ident(b, c, d),
                }
            } else {
                StartKind::Delim
            }
        }
        '@' => {
            if would_start_ident(look.point(1), look.point(2), look.point(3)) {
                StartKind::AtKeyword
            } else {
                StartKind::Delim
            }
        }
        '\'' | '"' => StartKind::String,
        '\\' => {
            if valid_escape(a, look.point(1)) {
                StartKind::Ident
            } else {
                StartKind::BadEscape
            }
        }
        value if is_ident_start(value) => StartKind::Ident,
        _ => StartKind::Delim,
    }
}

/// Section 4.3.1 as the machine takes it.
fn start<L: Lookahead>(
    value: char,
    look: &mut L,
    at: Position,
    ranges_allowed: bool,
) -> Result<Step, ProtocolError> {
    Ok(Step::of(match start_kind(value, look, ranges_allowed) {
        StartKind::Whitespace => Act::Move {
            state: State::Whitespace,
            consume: 0,
        },
        StartKind::Comment => Act::Move {
            state: State::Comment,
            consume: 2,
        },
        StartKind::Cdo => token(look, at, RawTokenKind::Cdo, 4)?,
        StartKind::Cdc => token(look, at, RawTokenKind::Cdc, 3)?,
        StartKind::Number => Act::Move {
            state: State::Number {
                cursor: NumberCursor::new(),
                digits_start: at.next_slot,
            },
            consume: 0,
        },
        StartKind::Ident => Act::Move {
            state: name_state(NamePurpose::Name(NameKind::Ident), at.next_slot),
            consume: 0,
        },
        StartKind::Hash { identifier } => Act::Move {
            state: name_state(
                NamePurpose::Name(NameKind::Hash { identifier }),
                at.next_slot,
            ),
            consume: 1,
        },
        StartKind::AtKeyword => Act::Move {
            state: name_state(NamePurpose::Name(NameKind::AtKeyword), at.next_slot),
            consume: 1,
        },
        StartKind::String => Act::Move {
            state: State::String {
                quote: value,
                units_start: at.next_slot,
            },
            consume: 1,
        },
        StartKind::UnicodeRange => Act::UnicodeRange,
        StartKind::Delim => token(look, at, RawTokenKind::Delim(value), 1)?,
        StartKind::BadEscape => {
            // Section 4.3.1: a backslash that does not start a valid escape is
            // a parse error and a delim token.
            return Ok(Step::with(
                token(look, at, RawTokenKind::Delim(value), 1)?,
                Fault::EscapedNewline,
            ));
        }
    }))
}
