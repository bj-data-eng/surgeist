//! The bounded CSS Syntax 3 section 4.3 tokenizer over preprocessed scalars.
//!
//! This cursor is the tokenizer for input that needs bounded, caller-admitted
//! work or did not arrive as `&str`. Every scalar it reads has already been
//! preprocessed (section 3.3, [`InputCursor`](crate::input::InputCursor)), so
//! U+0000, U+000D and U+000C never arrive; the lexical predicates it uses
//! describe preprocessed input.
//!
//! Each input observation, grammar transition, decoded name/digit append and
//! token append is distinct. All variable storage belongs to the caller.

use super::name::{NameCursor, Step as NameStep};
use super::number::{NumberCursor, NumberLayout, NumberPart, NumberStep};
use crate::input::Scalar;
use crate::lexical::{
    is_ident_code_point, is_non_printable, is_whitespace, valid_escape, would_start_ident,
    would_start_number,
};

/// A half-open interval `start..end`.
///
/// In [`RawToken::source`] it counts the original input's units, as a
/// [`Scalar`] does. Everywhere else in a [`RawTokenKind`] it counts slots of
/// the caller's flat record bank: the [`Record`]s the cursor asked the host to
/// store, in the order it asked.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    /// The first unit or slot.
    pub start: usize,
    /// One past the last unit or slot.
    pub end: usize,
}

/// Which name-like token a [`RawTokenKind::Name`] is.
///
/// The set is closed: it mirrors the name-like tokens of Syntax 3 section 4.1,
/// and a new edition would be a deliberate breaking change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NameKind {
    /// An `<ident-token>`.
    Ident,
    /// A `<function-token>`: an ident followed by U+0028 LEFT PARENTHESIS,
    /// which the token includes.
    Function,
    /// A `<hash-token>`; `identifier` is true when its type flag is `id`,
    /// that is, when the name would start an ident sequence.
    Hash {
        /// Whether the type flag is `id` rather than `unrestricted`.
        identifier: bool,
    },
    /// An `<at-keyword-token>`.
    AtKeyword,
}

/// Which numeric token a [`RawTokenKind::Number`] is.
///
/// The set is closed: it mirrors the numeric tokens of Syntax 3 section 4.1,
/// and a new edition would be a deliberate breaking change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NumberKind {
    /// A `<number-token>`.
    Number,
    /// A `<percentage-token>`; its `unit` span is empty.
    Percentage,
    /// A `<dimension-token>`; its `unit` span names the decoded unit.
    Dimension,
}

/// What a [`RawToken`] is, with the record-bank spans of its decoded parts.
///
/// The decoded text of a name, string or URL is not in the token: it is the
/// run of [`Record::NameUnit`]s the host stored in the named span. The digits
/// of a number are the run of [`Record::Digit`]s in `digits`, counted by
/// `layout`; the lexer never converts them to a value.
///
/// The set is closed: it mirrors the token list of Syntax 3 section 4.1 (plus
/// the comment the cursor reports), and a new edition would be a deliberate
/// breaking change.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RawTokenKind {
    /// An ident, function, hash or at-keyword token.
    Name {
        /// Which of those it is.
        kind: NameKind,
        /// The record slots of its decoded name units.
        units: Span,
    },
    /// A number, percentage or dimension token.
    Number {
        /// Which of those it is.
        kind: NumberKind,
        /// The counts of its spelling.
        layout: NumberLayout,
        /// The record slots of its digits: mantissa digits, then exponent
        /// digits.
        digits: Span,
        /// The record slots of its decoded unit; empty unless a dimension.
        unit: Span,
    },
    /// A `<whitespace-token>`.
    Whitespace,
    /// A comment. Syntax 3 discards comments; the cursor reports them so a
    /// host can keep source positions and spellings.
    Comment {
        /// Whether the comment ended at `*/` rather than at EOF.
        closed: bool,
    },
    /// A `<string-token>`.
    String {
        /// The record slots of its decoded units.
        units: Span,
        /// Whether the closing quote was present rather than EOF.
        closed: bool,
    },
    /// A `<bad-string-token>`.
    BadString,
    /// A `<url-token>`.
    Url {
        /// The record slots of its decoded units.
        units: Span,
        /// Whether the closing U+0029 RIGHT PARENTHESIS was present rather
        /// than EOF.
        closed: bool,
    },
    /// A `<bad-url-token>`.
    BadUrl {
        /// Whether the closing U+0029 RIGHT PARENTHESIS was present rather
        /// than EOF.
        closed: bool,
    },
    /// A `<delim-token>`.
    Delim(char),
    /// A `<CDO-token>`.
    Cdo,
    /// A `<CDC-token>`.
    Cdc,
}

/// A token together with the original input interval it covers.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RawToken {
    /// What the token is.
    pub kind: RawTokenKind,
    /// The original input units the token covers, as in [`Scalar::start`]
    /// and [`Scalar::end`].
    pub source: Span,
}

/// One fixed value the host stores in its flat record bank.
///
/// Each payload is fixed-size. Names and digits are caller-owned, ordered,
/// flat records; the spans of a [`RawTokenKind`] index them.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Record {
    /// One decoded unit of a name, string or URL.
    NameUnit(char),
    /// One digit of a number's spelling.
    Digit {
        /// Whether it is an exponent digit rather than a mantissa digit.
        exponent: bool,
        /// Its value, 0 to 9.
        value: u8,
    },
    /// A complete token.
    Token(RawToken),
}

/// What a [`TokenCursor`] needs its host to do next.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TokenRequest {
    /// Read the scalar at `index` and pass it to
    /// [`TokenCursor::read_scalar`].
    ReadScalar {
        /// The position of the scalar in the preprocessed scalar stream.
        index: usize,
    },
    /// Admit one bounded grammar decision, then call
    /// [`TokenCursor::compute`].
    Compute,
    /// Admit storage for `record` at slot `index`, store it, then call
    /// [`TokenCursor::appended`] with `index`. A refused append is simply not
    /// acknowledged: the same request stays pending.
    Append {
        /// The slot of the record bank the record takes.
        index: usize,
        /// The record to store.
        record: Record,
    },
    /// The input is exhausted and every record was appended.
    Done,
}

/// A protocol violation by the host of a [`TokenCursor`].
///
/// Every error leaves the cursor unchanged, so the pending request remains
/// valid.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProtocolError {
    /// The call does not answer the pending [`TokenRequest`].
    WrongReply,
    /// The slot index differs from the pending append's.
    WrongIndex,
    /// A scalar is empty or does not continue the previous scalar's
    /// interval, or a consumption exceeds the cached lookahead.
    Boundary,
    /// A position, slot or digit count does not fit `usize`.
    Overflow,
    /// The cursor already finished.
    Complete,
}

#[derive(Clone, Copy, Debug)]
enum NamePurpose {
    Name(NameKind),
    Dimension { layout: NumberLayout, digits: Span },
}

#[derive(Clone, Copy, Debug)]
enum EscapeTarget {
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

#[derive(Clone, Copy, Debug)]
enum State {
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

#[derive(Clone, Copy, Debug)]
struct Pending {
    record: Record,
    state: State,
    consume: u8,
    next_slot: usize,
}

/// The tokenizer as a sans-IO state machine.
///
/// The host asks [`request`](Self::request) and answers each request in turn:
/// it reads preprocessed scalars, admits each bounded decision, and stores
/// each record the cursor generates. A refused append leaves the cursor
/// unchanged and resumable. The host owns every bank; the cursor holds only
/// fixed registers.
///
/// Cached lookahead is four fixed registers (CDO needs four code points).
/// It is neither a copied source nor a second lexical backing store.
#[derive(Debug)]
pub struct TokenCursor {
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
    /// Creates a cursor over `scalar_length` preprocessed scalars whose first
    /// generated record takes slot `first_slot` of the host's record bank.
    #[must_use]
    pub const fn new(scalar_length: usize, first_slot: usize) -> Self {
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

    /// What the cursor needs from its host next. Reads only fixed registers;
    /// the host admits this observation like any other action.
    #[must_use]
    pub fn request(&self) -> TokenRequest {
        if let Some(pending) = self.pending {
            return TokenRequest::Append {
                index: self.next_slot,
                record: pending.record,
            };
        }
        if matches!(self.state, State::Done) {
            return TokenRequest::Done;
        }
        if self.cached < 4 && self.read < self.length {
            return TokenRequest::ReadScalar { index: self.read };
        }
        TokenRequest::Compute
    }

    /// One actual qualified scalar slot observation, not a borrowed source slice.
    ///
    /// # Errors
    ///
    /// [`ProtocolError::WrongReply`] unless the pending request is
    /// [`TokenRequest::ReadScalar`] for `index`; [`ProtocolError::Boundary`]
    /// when the scalar is empty or does not start where the previous one
    /// ended; [`ProtocolError::Overflow`] when the count of scalars read would
    /// exceed `usize`.
    pub fn read_scalar(&mut self, index: usize, scalar: Scalar) -> Result<(), ProtocolError> {
        if self.request() != (TokenRequest::ReadScalar { index }) {
            return Err(ProtocolError::WrongReply);
        }
        if scalar.end <= scalar.start {
            return Err(ProtocolError::Boundary);
        }
        let previous_end = if self.cached == 0 {
            self.consumed_end
        } else {
            self.window[usize::from(self.cached - 1)]
                .ok_or(ProtocolError::Boundary)?
                .end
        };
        if scalar.start != previous_end {
            return Err(ProtocolError::Boundary);
        }
        let next = self.read.checked_add(1).ok_or(ProtocolError::Overflow)?;
        self.window[usize::from(self.cached)] = Some(scalar);
        self.cached += 1;
        self.read = next;
        Ok(())
    }

    /// Commit only after the actual caller push succeeded. A refused push never
    /// calls this, so the same record/index remains available for cleanup/retry.
    ///
    /// # Errors
    ///
    /// [`ProtocolError::WrongReply`] unless an append is pending;
    /// [`ProtocolError::WrongIndex`] unless `index` is the pending slot.
    pub fn appended(&mut self, index: usize) -> Result<(), ProtocolError> {
        let pending = self.pending.ok_or(ProtocolError::WrongReply)?;
        if index != self.next_slot {
            return Err(ProtocolError::WrongIndex);
        }
        self.consume(pending.consume)?;
        self.state = pending.state;
        self.next_slot = pending.next_slot;
        self.pending = None;
        Ok(())
    }

    /// One bounded grammar decision over cached lookahead. It can request one
    /// append, or perform one fixed transition; it never loops to a next token.
    ///
    /// # Errors
    ///
    /// [`ProtocolError::WrongReply`] unless the pending request is
    /// [`TokenRequest::Compute`]; [`ProtocolError::Overflow`] when a slot or
    /// digit count would exceed `usize`; [`ProtocolError::Boundary`] and
    /// [`ProtocolError::Complete`] only if the cursor's own invariants fail.
    pub fn compute(&mut self) -> Result<(), ProtocolError> {
        if self.request() != TokenRequest::Compute {
            return Err(ProtocolError::WrongReply);
        }
        let [a, b, c, d] = self.window.map(|point| point.map(|point| point.value));
        match self.state {
            State::Start => {
                self.token_start = self.window[0].map_or(self.consumed_end, |point| point.start);
                let Some(value) = a else {
                    self.state = State::Done;
                    return Ok(());
                };
                if is_whitespace(value) {
                    self.state = State::Whitespace;
                } else if [a, b] == [Some('/'), Some('*')] {
                    self.consume(2)?;
                    self.state = State::Comment;
                } else if [a, b, c, d] == [Some('<'), Some('!'), Some('-'), Some('-')] {
                    self.token(RawTokenKind::Cdo, 4)?;
                } else if [a, b, c] == [Some('-'), Some('-'), Some('>')] {
                    self.token(RawTokenKind::Cdc, 3)?;
                } else if would_start_number(a, b, c) {
                    self.state = State::Number {
                        cursor: NumberCursor::new(),
                        digits_start: self.next_slot,
                    };
                } else if would_start_ident(a, b, c) {
                    self.state = State::Name {
                        cursor: NameCursor::new(),
                        purpose: NamePurpose::Name(NameKind::Ident),
                        units_start: self.next_slot,
                        url_match: 0,
                    };
                } else if value == '#' && (b.is_some_and(is_ident_code_point) || valid_escape(b, c))
                {
                    self.consume(1)?;
                    self.state = State::Name {
                        cursor: NameCursor::new(),
                        purpose: NamePurpose::Name(NameKind::Hash {
                            identifier: would_start_ident(b, c, d),
                        }),
                        units_start: self.next_slot,
                        url_match: 0,
                    };
                } else if value == '@' && would_start_ident(b, c, d) {
                    self.consume(1)?;
                    self.state = State::Name {
                        cursor: NameCursor::new(),
                        purpose: NamePurpose::Name(NameKind::AtKeyword),
                        units_start: self.next_slot,
                        url_match: 0,
                    };
                } else if matches!(value, '\'' | '"') {
                    self.consume(1)?;
                    self.state = State::String {
                        quote: value,
                        units_start: self.next_slot,
                    };
                } else {
                    self.token(RawTokenKind::Delim(value), 1)?;
                }
            }
            State::Whitespace => {
                if a.is_some_and(is_whitespace) {
                    self.consume(1)?;
                } else {
                    self.token(RawTokenKind::Whitespace, 0)?;
                }
            }
            State::Comment => {
                if [a, b] == [Some('*'), Some('/')] {
                    self.token(RawTokenKind::Comment { closed: true }, 2)?;
                } else if a.is_none() {
                    self.token(RawTokenKind::Comment { closed: false }, 0)?;
                } else {
                    self.consume(1)?;
                }
            }
            State::Name {
                mut cursor,
                purpose,
                units_start,
                url_match,
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
                            url_match,
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
                                url_match: match (url_match, value.to_ascii_lowercase()) {
                                    (0, 'u') | (1, 'r') | (2, 'l') => url_match + 1,
                                    _ => 4,
                                },
                            },
                            count,
                        )?;
                    }
                    NameStep::Done => {
                        let units = Span {
                            start: units_start,
                            end: self.next_slot,
                        };
                        if matches!(purpose, NamePurpose::Name(NameKind::Ident))
                            && url_match == 3
                            && a == Some('(')
                        {
                            self.consume(1)?;
                            self.state = State::UrlStart {
                                name: units,
                                opening_end: self.consumed_end,
                            };
                            return Ok(());
                        }
                        let (kind, consume) = match purpose {
                            NamePurpose::Name(NameKind::Ident) if a == Some('(') => (
                                RawTokenKind::Name {
                                    kind: NameKind::Function,
                                    units,
                                },
                                1,
                            ),
                            NamePurpose::Name(kind) => (RawTokenKind::Name { kind, units }, 0),
                            NamePurpose::Dimension { layout, digits } => (
                                RawTokenKind::Number {
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
            } => match cursor
                .step([a, b, c])
                .map_err(|_| ProtocolError::Overflow)?
            {
                NumberStep::Consume(NumberPart::Mantissa(value)) => {
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
                NumberStep::Consume(NumberPart::ExponentDigit(value)) => {
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
                        RawTokenKind::Number {
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
                } else if would_start_ident(a, b, c) {
                    self.state = State::Name {
                        cursor: NameCursor::new(),
                        purpose: NamePurpose::Dimension { layout, digits },
                        units_start: self.next_slot,
                        url_match: 0,
                    };
                } else {
                    self.token(
                        RawTokenKind::Number {
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
                    RawTokenKind::String {
                        units: Span {
                            start: units_start,
                            end: self.next_slot,
                        },
                        closed: true,
                    },
                    1,
                )?,
                None => self.token(
                    RawTokenKind::String {
                        units: Span {
                            start: units_start,
                            end: self.next_slot,
                        },
                        closed: false,
                    },
                    0,
                )?,
                Some('\n') => self.token(RawTokenKind::BadString, 0)?,
                Some('\\') => {
                    self.consume(1)?;
                    self.state = State::Escape {
                        target: EscapeTarget::String { quote, units_start },
                    };
                }
                Some(value) => self.append(Record::NameUnit(value), self.state, 1)?,
            },
            State::UrlStart { name, opening_end } => {
                if a.is_some_and(is_whitespace) {
                    self.consume(1)?;
                } else if matches!(a, Some('\'' | '"')) {
                    // Recognition may cross arbitrary whitespace, but the
                    // Function's opening ends at '(' and the actual whitespace
                    // retains its own source interval. No input is replayed.
                    let next = if self.consumed_end == opening_end {
                        State::Start
                    } else {
                        State::QuotedUrlWhitespace {
                            source: Span {
                                start: opening_end,
                                end: self.consumed_end,
                            },
                        }
                    };
                    self.append(
                        Record::Token(RawToken {
                            kind: RawTokenKind::Name {
                                kind: NameKind::Function,
                                units: name,
                            },
                            source: Span {
                                start: self.token_start,
                                end: opening_end,
                            },
                        }),
                        next,
                        0,
                    )?;
                } else {
                    self.state = State::Url {
                        units_start: self.next_slot,
                    };
                }
            }
            State::QuotedUrlWhitespace { source } => {
                self.append(
                    Record::Token(RawToken {
                        kind: RawTokenKind::Whitespace,
                        source,
                    }),
                    State::Start,
                    0,
                )?;
            }
            State::Url { units_start } => match a {
                Some(')') | None => self.token(
                    RawTokenKind::Url {
                        units: Span {
                            start: units_start,
                            end: self.next_slot,
                        },
                        closed: a.is_some(),
                    },
                    u8::from(a.is_some()),
                )?,
                Some(value) if is_whitespace(value) => {
                    self.consume(1)?;
                    self.state = State::UrlWhitespace { units_start };
                }
                Some(value) if matches!(value, '"' | '\'' | '(') || is_non_printable(value) => {
                    self.consume(1)?;
                    self.state = State::BadUrl;
                }
                Some('\\') => {
                    self.consume(1)?;
                    self.state = if valid_escape(a, b) {
                        State::Escape {
                            target: EscapeTarget::Url { units_start },
                        }
                    } else {
                        State::BadUrl
                    };
                }
                Some(value) => self.append(Record::NameUnit(value), self.state, 1)?,
            },
            State::UrlWhitespace { units_start } => {
                if a.is_some_and(is_whitespace) {
                    self.consume(1)?;
                } else if matches!(a, Some(')') | None) {
                    self.token(
                        RawTokenKind::Url {
                            units: Span {
                                start: units_start,
                                end: self.next_slot,
                            },
                            closed: a.is_some(),
                        },
                        u8::from(a.is_some()),
                    )?;
                } else {
                    self.state = State::BadUrl;
                }
            }
            State::BadUrl => {
                if matches!(a, Some(')') | None) {
                    self.token(
                        RawTokenKind::BadUrl {
                            closed: a.is_some(),
                        },
                        u8::from(a.is_some()),
                    )?;
                } else if valid_escape(a, b) {
                    self.consume(1)?;
                    self.state = State::Escape {
                        target: EscapeTarget::BadUrl,
                    };
                } else {
                    self.consume(1)?;
                }
            }
            State::Escape { target } => match a {
                None if matches!(target, EscapeTarget::Url { .. }) => {
                    self.append(Record::NameUnit('\u{fffd}'), target.continuation(), 0)?;
                }
                None => {
                    self.state = target.continuation();
                }
                Some('\n') if matches!(target, EscapeTarget::String { .. }) => {
                    self.consume(1)?;
                    self.state = target.continuation();
                }
                Some(value) if value.is_ascii_hexdigit() => {
                    self.consume(1)?;
                    self.state = State::Hex {
                        target,
                        value: value.to_digit(16).expect("hex check"),
                        digits: 1,
                    };
                }
                Some(_) if matches!(target, EscapeTarget::BadUrl) => {
                    self.consume(1)?;
                    self.state = target.continuation();
                }
                Some(value) => self.append(Record::NameUnit(value), target.continuation(), 1)?,
            },
            State::Hex {
                target,
                value,
                digits,
            } => {
                if digits < 6 && a.is_some_and(|value| value.is_ascii_hexdigit()) {
                    self.consume(1)?;
                    self.state = State::Hex {
                        target,
                        value: value * 16
                            + a.and_then(|value| value.to_digit(16)).expect("hex check"),
                        digits: digits + 1,
                    };
                } else {
                    self.consume(u8::from(a.is_some_and(is_whitespace)))?;
                    self.state = State::HexEmit { target, value };
                }
            }
            State::HexEmit {
                target: EscapeTarget::BadUrl,
                ..
            } => {
                self.state = State::BadUrl;
            }
            State::HexEmit { target, value } => {
                self.append(
                    Record::NameUnit(if value == 0 {
                        '\u{fffd}'
                    } else {
                        char::from_u32(value).unwrap_or('\u{fffd}')
                    }),
                    target.continuation(),
                    0,
                )?;
            }
            State::Done => return Err(ProtocolError::Complete),
        }
        Ok(())
    }

    fn append(&mut self, record: Record, state: State, consume: u8) -> Result<(), ProtocolError> {
        if consume > self.cached {
            return Err(ProtocolError::Boundary);
        }
        let next_slot = self
            .next_slot
            .checked_add(1)
            .ok_or(ProtocolError::Overflow)?;
        self.pending = Some(Pending {
            record,
            state,
            consume,
            next_slot,
        });
        Ok(())
    }

    fn token(&mut self, kind: RawTokenKind, consume: u8) -> Result<(), ProtocolError> {
        let end = if consume == 0 {
            self.consumed_end
        } else {
            self.window[usize::from(consume - 1)]
                .ok_or(ProtocolError::Boundary)?
                .end
        };
        self.append(
            Record::Token(RawToken {
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

    fn consume(&mut self, count: u8) -> Result<(), ProtocolError> {
        if count > self.cached {
            return Err(ProtocolError::Boundary);
        }
        if count == 0 {
            return Ok(());
        }
        self.consumed_end = self.window[usize::from(count - 1)]
            .ok_or(ProtocolError::Boundary)?
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
                TokenRequest::ReadScalar { index } => {
                    cursor.read_scalar(index, points[index]).unwrap()
                }
                TokenRequest::Compute => cursor.compute().unwrap(),
                TokenRequest::Append { index, record } => {
                    assert_eq!(index, records.len());
                    records.push(record);
                    cursor.appended(index).unwrap();
                }
                TokenRequest::Done => return records,
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

    fn token_records(records: &[Record]) -> Vec<RawToken> {
        records
            .iter()
            .filter_map(|record| match record {
                Record::Token(token) => Some(*token),
                _ => None,
            })
            .collect()
    }

    // CSS Syntax 4.3.4/4.3.6/4.3.7/4.3.14 supply these literal token outcomes,
    // independently of the consumer's eventual component/fallback grammar.
    #[test]
    fn unquoted_urls_decode_escaped_names_values_and_retain_exact_source() {
        let text = "u\\72l( a\\20 b\\)c )";
        let records = tokens(text);
        let actual = token_records(&records);
        assert_eq!(actual.len(), 1);
        let RawTokenKind::Url {
            units,
            closed: true,
        } = actual[0].kind
        else {
            panic!("escaped case-insensitive url name must select unquoted URL");
        };
        assert_eq!(names(&records, units), "a b)c");
        assert_eq!(
            actual[0].source,
            Span {
                start: 0,
                end: text.len()
            }
        );
        for text in ["url()", "URL(  )", "\\75rl()"] {
            let records = tokens(text);
            let actual = token_records(&records);
            assert_eq!(actual.len(), 1);
            let RawTokenKind::Url {
                units,
                closed: true,
            } = actual[0].kind
            else {
                panic!("actual empty URL token: {text:?}");
            };
            assert_eq!(names(&records, units), "");
            assert_eq!(
                actual[0].source,
                Span {
                    start: 0,
                    end: text.len()
                }
            );
        }
    }

    #[test]
    fn url_eof_is_distinct_from_closure_and_escape_eof_emits_replacement() {
        for (text, expected) in [
            ("url(", ""),
            ("url(  ", ""),
            ("url(a", "a"),
            ("url(a \t", "a"),
            ("url(a\\", "a�"),
            ("url(\\0", "�"),
            ("url(\\d800", "�"),
            ("url(\\110000", "�"),
            ("url(\\000061b", "ab"),
        ] {
            let records = tokens(text);
            let actual = token_records(&records);
            assert_eq!(actual.len(), 1, "{text:?}");
            let RawTokenKind::Url {
                units,
                closed: false,
            } = actual[0].kind
            else {
                panic!("EOF is an unclosed URL, not a bad URL: {text:?}");
            };
            assert_eq!(names(&records, units), expected, "{text:?}");
            assert_eq!(
                actual[0].source,
                Span {
                    start: 0,
                    end: text.len()
                }
            );
        }
    }

    #[test]
    fn quoted_url_keeps_function_opening_and_actual_arbitrary_whitespace_separate() {
        let text = "url( \t \n'v')";
        let records = tokens(text);
        let actual = token_records(&records);
        assert_eq!(actual.len(), 4);
        let RawTokenKind::Name {
            kind: NameKind::Function,
            units,
        } = actual[0].kind
        else {
            panic!("quoted URL is the original Function token");
        };
        assert_eq!(names(&records, units), "url");
        assert_eq!(actual[0].source, Span { start: 0, end: 4 });
        assert_eq!(
            actual[1],
            RawToken {
                kind: RawTokenKind::Whitespace,
                source: Span { start: 4, end: 8 },
            }
        );
        let RawTokenKind::String {
            units,
            closed: true,
        } = actual[2].kind
        else {
            panic!("quoted payload is the ordinary String token");
        };
        assert_eq!(names(&records, units), "v");
        assert_eq!(actual[2].source, Span { start: 8, end: 11 });
        assert_eq!(
            actual[3],
            RawToken {
                kind: RawTokenKind::Delim(')'),
                source: Span { start: 11, end: 12 },
            }
        );

        let text = format!("url({}\"v\")", " \t\n".repeat(100));
        let actual = token_records(&tokens(&text));
        assert_eq!(actual.len(), 4);
        assert_eq!(actual[0].source, Span { start: 0, end: 4 });
        assert_eq!(
            actual[1],
            RawToken {
                kind: RawTokenKind::Whitespace,
                source: Span { start: 4, end: 304 },
            }
        );
        assert_eq!(
            actual[2].source,
            Span {
                start: 304,
                end: 307
            }
        );
        assert_eq!(
            actual[3].source,
            Span {
                start: 307,
                end: 308
            }
        );
    }

    #[test]
    fn url_name_prefixes_at_keywords_and_dimensions_do_not_select_url_consumption() {
        for text in ["urls(x)", "@url(x)", "1url(x)", "url (x)"] {
            assert!(
                !token_records(&tokens(text)).iter().any(|token| matches!(
                    token.kind,
                    RawTokenKind::Url { .. } | RawTokenKind::BadUrl { .. }
                )),
                "{text:?}"
            );
        }
        let records = tokens("url(\"v\")");
        let actual = token_records(&records);
        assert_eq!(actual.len(), 3);
        assert!(matches!(
            actual[0].kind,
            RawTokenKind::Name {
                kind: NameKind::Function,
                ..
            }
        ));
        assert_eq!(actual[0].source, Span { start: 0, end: 4 });
        assert_eq!(actual[1].source, Span { start: 4, end: 7 });
    }

    #[test]
    fn bad_url_recovery_consumes_escaped_closing_parentheses_and_retains_eof() {
        for (text, closed) in [
            ("url(a b)", true),
            ("url(a\"b)", true),
            ("url(a'b)", true),
            ("url(a(b)", true),
            ("url(a\u{7f}b)", true),
            ("url(a\\\nb)", true),
            ("url(a b", false),
            ("url(a\"\\)", false),
            ("url(a\"\\29 ", false),
        ] {
            let actual = token_records(&tokens(text));
            assert_eq!(
                actual,
                [RawToken {
                    kind: RawTokenKind::BadUrl { closed },
                    source: Span {
                        start: 0,
                        end: text.len()
                    },
                }],
                "{text:?}"
            );
        }
        for text in ["url(a\"x\\)y)tail", "url(a\"x\\29 y)tail"] {
            let records = tokens(text);
            let actual = token_records(&records);
            assert_eq!(actual.len(), 2);
            assert_eq!(
                actual[0],
                RawToken {
                    kind: RawTokenKind::BadUrl { closed: true },
                    source: Span {
                        start: 0,
                        end: text.len() - 4
                    },
                }
            );
            let RawTokenKind::Name {
                kind: NameKind::Ident,
                units,
            } = actual[1].kind
            else {
                panic!("bad URL recovery must return before the following real token");
            };
            assert_eq!(names(&records, units), "tail");
        }
    }

    #[test]
    fn url_and_quoted_whitespace_append_refusal_keeps_every_exact_pending_record() {
        for text in ["url(a\\20 b)", "url(  \"v\")", "url(a\"\\)x)", "url(a\\"] {
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
            let mut reads = 0;
            let mut done = false;
            for _ in 0..(text.len() * 32 + 32) {
                match cursor.request() {
                    TokenRequest::ReadScalar { index } => {
                        assert_eq!(index, reads);
                        cursor.read_scalar(index, points[index]).unwrap();
                        reads += 1;
                    }
                    TokenRequest::Compute => cursor.compute().unwrap(),
                    request @ TokenRequest::Append { index, record } => {
                        assert_eq!(index, records.len());
                        assert_eq!(cursor.appended(index + 1), Err(ProtocolError::WrongIndex));
                        assert_eq!(cursor.compute(), Err(ProtocolError::WrongReply));
                        assert_eq!(cursor.request(), request);
                        records.push(record);
                        cursor.appended(index).unwrap();
                    }
                    TokenRequest::Done => {
                        done = true;
                        break;
                    }
                }
            }
            assert!(
                done,
                "bounded lexical continuation did not finish: {text:?}"
            );
            assert_eq!(reads, points.len());
            assert_eq!(records, tokens(text));
        }
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
        let RawTokenKind::Name {
            kind: NameKind::Function,
            units,
        } = tokens[0].kind
        else {
            panic!("function")
        };
        assert_eq!(names(&records, units), "rgb");
        assert_eq!(tokens[0].source, Span { start: 0, end: 7 });
        let RawTokenKind::Name {
            kind: NameKind::Hash { identifier: true },
            units,
        } = tokens[2].kind
        else {
            panic!("hash")
        };
        assert_eq!(names(&records, units), "f00");
        assert_eq!(tokens[2].source, Span { start: 8, end: 12 });
        assert_eq!(tokens[4].kind, RawTokenKind::Delim(')'));
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
        let RawTokenKind::Number {
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
            NumberLayout {
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
            RawTokenKind::Number {
                kind: NumberKind::Percentage,
                ..
            }
        ));
        let RawTokenKind::Number {
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
            RawToken {
                kind: RawTokenKind::Comment { closed: true },
                source: Span { start: 0, end: 7 }
            }
        );
        let RawTokenKind::String {
            units,
            closed: true,
        } = tokens[1].kind
        else {
            panic!("string")
        };
        assert_eq!(names(&records, units), "ab");
        assert_eq!(tokens[3].kind, RawTokenKind::BadString);
        assert!(tokens.iter().any(|token| token.kind == RawTokenKind::Cdo));
        assert!(tokens.iter().any(|token| token.kind == RawTokenKind::Cdc));
        assert_eq!(
            tokens.last().unwrap().kind,
            RawTokenKind::Comment { closed: false }
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
            TokenRequest::Append {
                index: 7,
                record: Record::NameUnit('r')
            }
        );
        assert_eq!(cursor.appended(6), Err(ProtocolError::WrongIndex));
        assert_eq!(cursor.compute(), Err(ProtocolError::WrongReply));
        assert_eq!(cursor.request(), before);
        cursor.appended(7).unwrap();
        cursor.compute().unwrap();
        assert!(matches!(
            cursor.request(),
            TokenRequest::Append {
                index: 8,
                record: Record::Token(_)
            }
        ));
    }
}
