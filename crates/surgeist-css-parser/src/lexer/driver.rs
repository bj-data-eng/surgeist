//! The free-running lexer: the section 4.3 machine over `&str`, without a meter.

use super::lexeme::{Lexeme, LexicalError, LexicalErrorKind, Options};
use super::machine::{
    self, Act, Fault, Lookahead, NameEndKind, NamePurpose, Position, StartKind, State, Step,
    SuffixEnd, name_end_kind, next_url_match, start_kind, suffix_end,
};
use super::protocol::name::NameCursor;
use super::protocol::{
    NameKind, NumberCursor, NumberKind as RawNumberKind, NumberLayout, NumberPart, NumberStep,
    RawTokenKind, Record, Span,
};
use crate::input::Scalar;
use crate::lexical::is_ident_code_point;
use crate::token::{HashKind, Number, NumberKind};
use crate::unicode_range::CodePointRange;
use crate::{Offset, Token};

/// The ASCII code points that continue an ident sequence, indexed by byte.
const ASCII_IDENT: [bool; 128] = {
    let mut table = [false; 128];
    let mut byte = 0;
    while byte < 128 {
        table[byte] = is_ident_code_point(byte as u8 as char);
        byte += 1;
    }
    table
};

/// Whether a byte of the text is whitespace after section 3.3: a space, a tab,
/// a line feed, and the carriage return and form feed that become line feeds.
const fn is_space_byte(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | 0x0c)
}

/// The scalars after the consumed text, decoded as the machine asks for them.
///
/// Section 3.3 is applied here, as each scalar is read: a carriage return, a
/// form feed or a carriage return and line feed pair is one U+000A scalar, a
/// NULL is U+FFFD, and every scalar keeps the interval of original bytes it
/// came from.
#[derive(Debug)]
struct Window<'s> {
    text: &'s str,
    /// The original byte after the last decoded scalar.
    next: usize,
    slots: [Option<Scalar>; 4],
    filled: u8,
}

impl<'s> Window<'s> {
    const fn new(text: &'s str) -> Self {
        Self {
            text,
            next: 0,
            slots: [None; 4],
            filled: 0,
        }
    }

    /// Forgets the decoded scalars; the next ones are read from `position`.
    #[inline]
    fn reset(&mut self, position: usize) {
        self.next = position;
        self.filled = 0;
    }

    #[inline]
    fn decode(&self, at: usize) -> Option<Scalar> {
        let bytes = self.text.as_bytes();
        let byte = *bytes.get(at)?;
        Some(if byte < 0x80 {
            match byte {
                0 => Scalar {
                    value: '\u{fffd}',
                    start: at,
                    end: at + 1,
                },
                b'\r' => Scalar {
                    value: '\n',
                    start: at,
                    end: if bytes.get(at + 1) == Some(&b'\n') {
                        at + 2
                    } else {
                        at + 1
                    },
                },
                0x0c => Scalar {
                    value: '\n',
                    start: at,
                    end: at + 1,
                },
                _ => Scalar {
                    value: char::from(byte),
                    start: at,
                    end: at + 1,
                },
            }
        } else {
            let value = self.text.get(at..)?.chars().next()?;
            Scalar {
                value,
                start: at,
                end: at + value.len_utf8(),
            }
        })
    }

    /// Drops the first `count` scalars, which the machine consumed.
    fn shift(&mut self, count: u8) {
        let count = usize::from(count);
        self.slots = [
            self.slots.get(count).copied().flatten(),
            self.slots.get(count + 1).copied().flatten(),
            self.slots.get(count + 2).copied().flatten(),
            self.slots.get(count + 3).copied().flatten(),
        ];
        self.filled = self.filled.saturating_sub(count as u8);
    }
}

impl Lookahead for Window<'_> {
    #[inline]
    fn scalar(&mut self, index: usize) -> Option<Scalar> {
        if index >= 4 {
            return None;
        }
        while usize::from(self.filled) <= index {
            let scalar = self.decode(self.next);
            if let Some(scalar) = scalar {
                self.next = scalar.end;
            }
            self.slots[usize::from(self.filled)] = scalar;
            self.filled += 1;
        }
        self.slots[index]
    }
}

/// How much of `url` the units spelled so far match, after `run` is appended.
fn advance_url_match(mut matched: u8, run: &str) -> u8 {
    for point in run.chars() {
        if matched >= 4 {
            break;
        }
        matched = next_url_match(matched, point);
    }
    matched
}

/// How the decoded text of the token in progress is held.
#[derive(Clone, Copy, Debug)]
enum Text {
    /// No unit yet.
    Empty,
    /// The units are exactly the source bytes `start..end`.
    Source { start: usize, end: usize },
    /// The units are in the scratch pool.
    Pool,
}

/// What a finished token is, without the spans of the machine's record bank
/// that it no longer needs.
#[derive(Clone, Copy, Debug)]
enum Kind {
    Name(NameKind, Span),
    Number {
        kind: RawNumberKind,
        integer: bool,
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
    Url {
        units: Span,
        closed: bool,
    },
    BadUrl {
        closed: bool,
    },
    Delim(char),
    Cdo,
    Cdc,
    UnicodeRange(CodePointRange),
}

impl Kind {
    fn of(raw: RawTokenKind) -> Self {
        match raw {
            RawTokenKind::Name { kind, units } => Self::Name(kind, units),
            RawTokenKind::Number {
                kind, layout, unit, ..
            } => Self::Number {
                kind,
                integer: layout.fraction_digits == 0 && layout.exponent_digits == 0,
                unit,
            },
            RawTokenKind::Whitespace => Self::Whitespace,
            RawTokenKind::Comment { closed } => Self::Comment { closed },
            RawTokenKind::String { units, closed } => Self::String { units, closed },
            RawTokenKind::BadString => Self::BadString,
            RawTokenKind::Url { units, closed } => Self::Url { units, closed },
            RawTokenKind::BadUrl { closed } => Self::BadUrl { closed },
            RawTokenKind::Delim(point) => Self::Delim(point),
            RawTokenKind::Cdo => Self::Cdo,
            RawTokenKind::Cdc => Self::Cdc,
        }
    }
}

/// The token a step finished and the source it covers.
#[derive(Clone, Copy, Debug)]
struct Finished {
    kind: Kind,
    source: Span,
}

/// A free-running, unmetered lexer over text (Run granularity).
///
/// It is the same section 4.3 state machine that drives the stepwise
/// [`TokenCursor`](super::protocol::TokenCursor), applying each transition at
/// once instead of through a host's requests, and consuming a whole run of
/// ordinary scalars (a name, whitespace, the inside of a comment, string or
/// url, a number's digits) where the transitions would consume them one by one
/// with the same result. It applies section 3.3 as it reads, so the text need
/// not be preprocessed and offsets are original bytes.
///
/// Tokenizing never fails: every text has a token stream, and the places where
/// the specification calls the text a parse error are reported through
/// [`errors`](Self::errors).
///
/// ```
/// use surgeist_css_parser::lexer::{Lexer, Options};
/// use surgeist_css_parser::Token;
///
/// let mut lexer = Lexer::new("a{b:1px}", &Options::default());
/// let mut kinds = Vec::new();
/// while let Some(lexeme) = lexer.next_lexeme() {
///     kinds.push(lexeme.token().kind());
/// }
/// assert_eq!(kinds.len(), 6);
/// assert!(lexer.errors().is_empty());
/// ```
#[derive(Debug)]
pub struct Lexer<'s> {
    text: &'s str,
    window: Window<'s>,
    /// One past the last consumed original byte.
    pos: usize,
    token_start: usize,
    state: State,
    ranges_allowed: bool,
    origin: usize,
    held: Text,
    scratch: String,
    number_end: usize,
    bad_url: bool,
    finished: Option<Finished>,
    errors: Vec<LexicalError>,
}

impl<'s> Lexer<'s> {
    /// Creates a lexer over `text`.
    #[must_use]
    pub fn new(text: &'s str, options: &Options) -> Self {
        Self {
            text,
            window: Window::new(text),
            pos: 0,
            token_start: 0,
            state: State::Start,
            ranges_allowed: options.allows_unicode_ranges(),
            origin: options.origin_offset().value(),
            held: Text::Empty,
            scratch: String::new(),
            number_end: 0,
            bad_url: false,
            finished: None,
            errors: Vec::new(),
        }
    }

    /// Returns the next token, or `None` at the end of the text.
    ///
    /// The lexeme borrows the lexer: the decoded text of an escaped name or
    /// string lives in the lexer's scratch space until the next call.
    pub fn next_lexeme(&mut self) -> Option<Lexeme<'_>> {
        if !self.advance() {
            return None;
        }
        self.lexeme()
    }

    /// Returns the lexical errors met so far, in the order they were met.
    #[must_use]
    pub fn errors(&self) -> &[LexicalError] {
        &self.errors
    }

    // ----- text of the token in progress

    fn text_length(&self) -> usize {
        match self.held {
            Text::Empty => 0,
            Text::Source { start, end } => end - start,
            Text::Pool => self.scratch.len(),
        }
    }

    /// Moves the held units into the pool so that a unit that is not source
    /// text can follow them.
    fn pool(&mut self) {
        if let Text::Source { start, end } = self.held {
            self.scratch
                .push_str(self.text.get(start..end).unwrap_or(""));
        }
        self.held = Text::Pool;
    }

    fn push_unit(&mut self, unit: char) {
        self.pool();
        self.scratch.push(unit);
    }

    /// Appends the source bytes `from..to` as units.
    fn push_source(&mut self, from: usize, to: usize) {
        match self.held {
            Text::Empty => {
                self.held = Text::Source {
                    start: from,
                    end: to,
                }
            }
            Text::Source { start, end } if end == from => {
                self.held = Text::Source { start, end: to };
            }
            _ => {
                self.pool();
                self.scratch.push_str(self.text.get(from..to).unwrap_or(""));
            }
        }
    }

    fn held_slice(&self, span: Span) -> &str {
        match self.held {
            Text::Empty => "",
            Text::Source { start, .. } => self
                .text
                .get(start + span.start..start + span.end)
                .unwrap_or(""),
            Text::Pool => self.scratch.get(span.start..span.end).unwrap_or(""),
        }
    }

    // ----- driving

    fn advance(&mut self) -> bool {
        self.finished = None;
        self.held = Text::Empty;
        self.scratch.clear();
        self.bad_url = false;
        loop {
            if matches!(self.state, State::Done) {
                return false;
            }
            if matches!(self.state, State::Start) {
                self.token_start = self.pos;
            }
            match self.run() {
                Run::Finished => return true,
                Run::Progress => continue,
                Run::Nothing => {}
            }
            let at = Position {
                next_slot: self.text_length(),
                consumed_end: self.pos,
                token_start: self.token_start,
            };
            let step = machine::step(self.state, &mut self.window, at, self.ranges_allowed);
            let Ok(step) = step else {
                // The machine's own invariants cannot fail on text; if they
                // ever did, stop rather than loop.
                self.state = State::Done;
                return false;
            };
            if self.apply(step) {
                return true;
            }
        }
    }

    /// Applies a transition. Returns whether it finished a token.
    fn apply(&mut self, step: Step) -> bool {
        match step.act {
            Act::Finish => self.state = State::Done,
            Act::Move { state, consume } => {
                self.consume(consume);
                if matches!(state, State::Suffix { .. }) {
                    self.number_end = self.pos;
                }
                self.state = state;
            }
            Act::Emit {
                record,
                state,
                consume,
            } => {
                match record {
                    Record::NameUnit(unit) => self.push_unit(unit),
                    Record::Digit { .. } => {}
                    Record::Token(raw) => {
                        self.consume(consume);
                        self.state = state;
                        self.note_fault(step.fault, Some(raw.source));
                        self.finish(Finished {
                            kind: Kind::of(raw.kind),
                            source: raw.source,
                        });
                        return true;
                    }
                }
                self.consume(consume);
                self.state = state;
            }
            Act::UnicodeRange => {
                let finished = self.consume_unicode_range();
                self.state = State::Start;
                self.finish(finished);
                return true;
            }
        }
        self.note_fault(step.fault, None);
        false
    }

    fn consume(&mut self, count: u8) {
        if count == 0 {
            return;
        }
        if let Some(scalar) = self.window.scalar(usize::from(count) - 1) {
            self.pos = scalar.end;
        }
        self.window.shift(count);
    }

    fn note_fault(&mut self, fault: Option<Fault>, token: Option<Span>) {
        match fault {
            None => {}
            Some(Fault::BadUrl) => self.bad_url = true,
            Some(Fault::EscapedNewline) => {
                if let Some(source) = token {
                    self.record(LexicalErrorKind::EscapedNewline, source.start, source.end);
                }
            }
            Some(Fault::EscapedEof) => {
                self.record(
                    LexicalErrorKind::EscapedEof,
                    self.pos.saturating_sub(1),
                    self.pos,
                );
            }
        }
    }

    fn record(&mut self, kind: LexicalErrorKind, start: usize, end: usize) {
        let range = self.offset(start)..self.offset(end);
        self.errors.push(LexicalError::new(kind, range));
    }

    fn offset(&self, position: usize) -> Offset {
        Offset::new(self.origin.saturating_add(position))
    }

    /// Records the token and the errors that its ending shows.
    #[inline]
    fn finish(&mut self, finished: Finished) {
        self.report(finished);
        self.finished = Some(finished);
    }

    /// Reports the lexical error that the way a token ended shows, if any.
    fn report(&mut self, finished: Finished) {
        let (start, end) = (finished.source.start, finished.source.end);
        match finished.kind {
            Kind::Comment { closed: false } => {
                self.record(LexicalErrorKind::UnterminatedComment, start, end);
            }
            Kind::String { closed: false, .. } => {
                self.record(LexicalErrorKind::UnterminatedString, start, end);
            }
            Kind::BadString => self.record(LexicalErrorKind::BadString, start, end),
            Kind::Url { closed: false, .. } => {
                self.record(LexicalErrorKind::UnterminatedUrl, start, end);
            }
            Kind::BadUrl { .. } if self.bad_url => {
                self.record(LexicalErrorKind::BadUrl, start, end);
            }
            _ => {}
        }
    }

    // ----- runs
    //
    // A run consumes scalars that an unbroken series of the machine's own
    // transitions would consume one at a time, and stops at the first scalar
    // whose transition is not that series' repetition: a backslash, a NULL,
    // a newline in a string, an ending delimiter that needs a decision. The
    // decisions at a run's end are the machine's, through the shared
    // `name_end` and `suffix_end` and the cursors' own steps; a run never
    // decides a token alone that the machine would decide differently, which
    // the stepwise property tests check on generated and raw text.

    /// Finishes a token at `end` and returns to the start state. The caller
    /// reports any lexical error the token's ending shows.
    #[inline]
    fn complete(&mut self, kind: Kind, end: usize) -> Run {
        self.pos = end;
        self.window.reset(end);
        self.state = State::Start;
        self.finished = Some(Finished {
            kind,
            source: Span {
                start: self.token_start,
                end,
            },
        });
        Run::Finished
    }

    /// Finishes a token that reached the end of the text without its closing
    /// delimiter and reports it.
    #[cold]
    fn complete_unterminated(&mut self, kind: Kind, end: usize) -> Run {
        let run = self.complete(kind, end);
        if let Some(finished) = self.finished {
            self.report(finished);
        }
        run
    }

    /// Consumes a run of scalars, or finishes a token that a run decides.
    fn run(&mut self) -> Run {
        match self.state {
            State::Start => self.start_run(),
            State::Whitespace => {
                let end = self.skip_while(is_space_byte);
                self.complete(Kind::Whitespace, end)
            }
            State::Comment => self.run_comment(),
            State::Name {
                cursor,
                purpose,
                units_start,
                url_match,
            } if cursor.is_plain() => self.run_name(cursor, purpose, units_start, url_match, false),
            State::String { quote, units_start } => self.run_string(quote, units_start),
            State::Url { units_start } => self.run_url(units_start),
            State::Number {
                cursor,
                digits_start,
            } => self.run_number(cursor, digits_start),
            _ => Run::Nothing,
        }
    }

    fn skip_while(&self, keep: impl Fn(u8) -> bool) -> usize {
        let bytes = self.text.as_bytes();
        let rest = bytes.get(self.pos..).unwrap_or(&[]);
        self.pos
            + rest
                .iter()
                .position(|&byte| !keep(byte))
                .unwrap_or(rest.len())
    }

    /// Section 4.3.2. The inside of a comment matters only at an asterisk.
    fn run_comment(&mut self) -> Run {
        let bytes = self.text.as_bytes();
        let mut at = self.pos;
        loop {
            let rest = bytes.get(at..).unwrap_or(&[]);
            match rest.iter().position(|&byte| byte == b'*') {
                None => {
                    return self
                        .complete_unterminated(Kind::Comment { closed: false }, bytes.len());
                }
                Some(offset) => {
                    at += offset;
                    if bytes.get(at + 1) == Some(&b'/') {
                        return self.complete(Kind::Comment { closed: true }, at + 2);
                    }
                    at += 1;
                }
            }
        }
    }

    /// Sections 4.3.12 and 4.3.4. Consumes the ident code points that are
    /// themselves, then decides the token if nothing needs the machine.
    /// `fresh` is whether the caller has not stored the state yet.
    fn run_name(
        &mut self,
        cursor: NameCursor,
        purpose: NamePurpose,
        units_start: usize,
        mut url_match: u8,
        fresh: bool,
    ) -> Run {
        let text = self.text;
        let bytes = text.as_bytes();
        let start = self.pos;
        let mut end = start;
        while let Some(&byte) = bytes.get(end) {
            if byte < 0x80 {
                if ASCII_IDENT[usize::from(byte)] {
                    end += 1;
                } else {
                    break;
                }
            } else {
                match text.get(end..).and_then(|rest| rest.chars().next()) {
                    Some(point) if is_ident_code_point(point) => end += point.len_utf8(),
                    _ => break,
                }
            }
        }
        if end > start {
            url_match = advance_url_match(url_match, text.get(start..end).unwrap_or(""));
            self.push_source(start, end);
            self.pos = end;
        }
        match bytes.get(end) {
            // A backslash starts an escape or ends the name, and a NULL is a
            // unit of its own: the machine takes these.
            Some(b'\\' | 0) => {
                self.state = State::Name {
                    cursor,
                    purpose,
                    units_start,
                    url_match,
                };
                self.window.reset(end);
                if end > start || fresh {
                    Run::Progress
                } else {
                    Run::Nothing
                }
            }
            next => {
                let units = Span {
                    start: units_start,
                    end: self.text_length(),
                };
                let paren = next == Some(&b'(');
                match name_end_kind(&purpose, url_match, paren) {
                    NameEndKind::Url => {
                        self.pos = end + 1;
                        self.window.reset(self.pos);
                        self.state = State::UrlStart {
                            name: units,
                            opening_end: self.pos,
                        };
                        Run::Progress
                    }
                    NameEndKind::Function => {
                        self.complete(Kind::Name(NameKind::Function, units), end + 1)
                    }
                    NameEndKind::Plain(kind) => self.complete(Kind::Name(kind, units), end),
                    NameEndKind::Dimension => {
                        let integer = match purpose {
                            NamePurpose::Dimension { layout, .. } => {
                                layout.fraction_digits == 0 && layout.exponent_digits == 0
                            }
                            NamePurpose::Name(_) => true,
                        };
                        self.complete(
                            Kind::Number {
                                kind: RawNumberKind::Dimension,
                                integer,
                                unit: units,
                            },
                            end,
                        )
                    }
                }
            }
        }
    }

    /// Section 4.3.5.
    fn run_string(&mut self, quote: char, units_start: usize) -> Run {
        let bytes = self.text.as_bytes();
        let quote = quote as u8;
        let start = self.pos;
        let end = self.skip_while(|byte| {
            !(byte == quote || matches!(byte, b'\\' | b'\n' | b'\r' | 0x0c | 0))
        });
        if end > start {
            self.push_source(start, end);
            self.pos = end;
        }
        let units = Span {
            start: units_start,
            end: self.text_length(),
        };
        match bytes.get(end) {
            Some(&byte) if byte == quote => self.complete(
                Kind::String {
                    units,
                    closed: true,
                },
                end + 1,
            ),
            None => self.complete_unterminated(
                Kind::String {
                    units,
                    closed: false,
                },
                end,
            ),
            Some(_) => {
                self.window.reset(end);
                if end > start {
                    Run::Progress
                } else {
                    Run::Nothing
                }
            }
        }
    }

    /// Section 4.3.6.
    fn run_url(&mut self, units_start: usize) -> Run {
        let bytes = self.text.as_bytes();
        let start = self.pos;
        let end = self.skip_while(|byte| {
            !matches!(byte, b')' | b'"' | b'\'' | b'(' | b'\\' | 0..=0x20 | 0x7f)
        });
        if end > start {
            self.push_source(start, end);
            self.pos = end;
        }
        let units = Span {
            start: units_start,
            end: self.text_length(),
        };
        match bytes.get(end) {
            Some(b')') => self.complete(
                Kind::Url {
                    units,
                    closed: true,
                },
                end + 1,
            ),
            None => self.complete_unterminated(
                Kind::Url {
                    units,
                    closed: false,
                },
                end,
            ),
            Some(_) => {
                self.window.reset(end);
                if end > start {
                    Run::Progress
                } else {
                    Run::Nothing
                }
            }
        }
    }

    /// Sections 4.3.13 and 4.3.3. The cursor decides every step; a run of digits
    /// is counted at once.
    fn run_number(&mut self, mut cursor: NumberCursor, digits_start: usize) -> Run {
        let bytes = self.text.as_bytes();
        let mut end = self.pos;
        loop {
            let point = |offset: usize| bytes.get(end + offset).copied().map(char::from);
            match cursor.step([point(0), point(1), point(2)]) {
                Ok(NumberStep::Consume(part)) => {
                    end += 1;
                    if matches!(part, NumberPart::Mantissa(_) | NumberPart::ExponentDigit(_)) {
                        let extra = bytes.get(end..).map_or(0, |rest| {
                            rest.iter().take_while(|byte| byte.is_ascii_digit()).count()
                        });
                        if extra > 0 {
                            if cursor.add_digits(extra).is_err() {
                                self.state = State::Done;
                                return Run::Finished;
                            }
                            end += extra;
                        }
                        // Most numbers end here: nothing that can continue one
                        // follows the digits.
                        let next = match bytes.get(end) {
                            Some(&byte) if byte < 0x80 => Some(Some(char::from(byte))),
                            None => Some(None),
                            Some(_) => None,
                        };
                        if let Some(layout) = next.and_then(|next| cursor.finish_without_more(next))
                        {
                            self.pos = end;
                            self.number_end = end;
                            let digits = Span {
                                start: digits_start,
                                end: self.text_length(),
                            };
                            return self.after_number(layout, digits);
                        }
                    }
                }
                Ok(NumberStep::Transition) => {}
                Ok(NumberStep::Done(layout)) => {
                    self.pos = end;
                    self.number_end = end;
                    let digits = Span {
                        start: digits_start,
                        end: self.text_length(),
                    };
                    return self.after_number(layout, digits);
                }
                Err(_) => {
                    self.state = State::Done;
                    return Run::Finished;
                }
            }
        }
    }

    /// Section 4.3.3: what follows the number decides its token.
    fn after_number(&mut self, layout: NumberLayout, digits: Span) -> Run {
        let end = self.pos;
        let bytes = self.text.as_bytes();
        // The three scalars after the number. Plain ASCII is itself; anything
        // else (a NULL, a carriage return, a form feed, a multi-byte code
        // point) is decoded as section 3.3 says.
        let plain = |index: usize| match bytes.get(index) {
            Some(&byte) if byte < 0x80 && !matches!(byte, 0 | b'\r' | 0x0c) => {
                Some(Some(char::from(byte)))
            }
            None => Some(None),
            Some(_) => None,
        };
        let (first, second, third) = match (plain(end), plain(end + 1), plain(end + 2)) {
            (Some(first), Some(second), Some(third)) => (first, second, third),
            _ => {
                self.window.reset(end);
                (
                    self.window.point(0),
                    self.window.point(1),
                    self.window.point(2),
                )
            }
        };
        let integer = layout.fraction_digits == 0 && layout.exponent_digits == 0;
        let next_slot = self.text_length();
        let empty = Span {
            start: next_slot,
            end: next_slot,
        };
        match suffix_end(first, second, third) {
            SuffixEnd::Percentage => self.complete(
                Kind::Number {
                    kind: RawNumberKind::Percentage,
                    integer,
                    unit: empty,
                },
                end + 1,
            ),
            SuffixEnd::Number => self.complete(
                Kind::Number {
                    kind: RawNumberKind::Number,
                    integer,
                    unit: empty,
                },
                end,
            ),
            SuffixEnd::Dimension => self.run_name(
                NameCursor::new(),
                NamePurpose::Dimension { layout, digits },
                next_slot,
                0,
                true,
            ),
        }
    }

    /// Section 4.3.1 for the first code point of a token. A byte that settles
    /// the token alone (a bracket, a brace, a colon, a semicolon, a comma,
    /// whitespace, a digit, a letter) is taken at once; any other first code
    /// point is decided by the machine's own `start_kind`, and only what needs
    /// the machine's steps (a unicode-range, a backslash that starts no
    /// escape, the end of the text) is left to it.
    fn start_run(&mut self) -> Run {
        let bytes = self.text.as_bytes();
        let Some(&byte) = bytes.get(self.pos) else {
            return Run::Nothing;
        };
        let start = self.pos;
        match byte {
            b'{' | b'}' | b'(' | b')' | b'[' | b']' | b':' | b';' | b',' => {
                self.complete(Kind::Delim(char::from(byte)), start + 1)
            }
            byte if is_space_byte(byte) => {
                let end = self.skip_while(is_space_byte);
                self.complete(Kind::Whitespace, end)
            }
            b'0'..=b'9' => self.run_number(NumberCursor::new(), 0),
            b'a'..=b'z' | b'A'..=b'Z' | b'_'
                if !(self.ranges_allowed && matches!(byte, b'u' | b'U')) =>
            {
                self.run_name(
                    NameCursor::new(),
                    NamePurpose::Name(NameKind::Ident),
                    0,
                    0,
                    true,
                )
            }
            _ => self.start_decided(),
        }
    }

    /// Decides the token from its first code point with the machine's
    /// `start_kind`, and takes the cases that need no step.
    fn start_decided(&mut self) -> Run {
        let start = self.pos;
        self.window.reset(start);
        let Some(first) = self.window.scalar(0) else {
            return Run::Nothing;
        };
        let after = first.end;
        match start_kind(first.value, &mut self.window, self.ranges_allowed) {
            StartKind::Whitespace | StartKind::Comment | StartKind::String => match first.value {
                '/' => {
                    self.pos = after + 1;
                    self.window.reset(self.pos);
                    self.state = State::Comment;
                    Run::Progress
                }
                '"' | '\'' => {
                    self.pos = after;
                    self.window.reset(after);
                    self.state = State::String {
                        quote: first.value,
                        units_start: 0,
                    };
                    Run::Progress
                }
                _ => Run::Nothing,
            },
            StartKind::Cdo => self.complete(Kind::Cdo, start + 4),
            StartKind::Cdc => self.complete(Kind::Cdc, start + 3),
            StartKind::Number => self.run_number(NumberCursor::new(), 0),
            StartKind::Ident => self.run_name(
                NameCursor::new(),
                NamePurpose::Name(NameKind::Ident),
                0,
                0,
                true,
            ),
            StartKind::Hash { identifier } => {
                self.pos = after;
                self.window.reset(after);
                self.run_name(
                    NameCursor::new(),
                    NamePurpose::Name(NameKind::Hash { identifier }),
                    0,
                    0,
                    true,
                )
            }
            StartKind::AtKeyword => {
                self.pos = after;
                self.window.reset(after);
                self.run_name(
                    NameCursor::new(),
                    NamePurpose::Name(NameKind::AtKeyword),
                    0,
                    0,
                    true,
                )
            }
            StartKind::Delim => self.complete(Kind::Delim(first.value), after),
            StartKind::UnicodeRange | StartKind::BadEscape => Run::Nothing,
        }
    }

    // ----- unicode ranges

    /// Section 4.3.14, consume a unicode-range token. The next two input code
    /// points, "U+" in either case, are discarded.
    fn consume_unicode_range(&mut self) -> Finished {
        let hex = |window: &mut Window<'_>, index: usize| {
            window
                .scalar(index)
                .and_then(|scalar| scalar.value.to_digit(16))
        };
        self.consume(2);
        let mut value: u32 = 0;
        let mut digits = 0_u32;
        while digits < 6 {
            let Some(digit) = hex(&mut self.window, 0) else {
                break;
            };
            value = value * 16 + digit;
            digits += 1;
            self.consume(1);
        }
        let mut questions = 0_u32;
        while digits + questions < 6
            && self.window.scalar(0).map(|scalar| scalar.value) == Some('?')
        {
            questions += 1;
            self.consume(1);
        }
        let range = if questions > 0 {
            let start = value << (4 * questions);
            CodePointRange::new(start, start + (1 << (4 * questions)) - 1)
        } else if self.window.scalar(0).map(|scalar| scalar.value) == Some('-')
            && hex(&mut self.window, 1).is_some()
        {
            self.consume(1);
            let mut end: u32 = 0;
            let mut count = 0;
            while count < 6 {
                let Some(digit) = hex(&mut self.window, 0) else {
                    break;
                };
                end = end * 16 + digit;
                count += 1;
                self.consume(1);
            }
            CodePointRange::new(value, end)
        } else {
            CodePointRange::new(value, value)
        };
        Finished {
            kind: Kind::UnicodeRange(range),
            source: Span {
                start: self.token_start,
                end: self.pos,
            },
        }
    }

    // ----- output

    fn lexeme(&self) -> Option<Lexeme<'_>> {
        let finished = self.finished?;
        let (token, unterminated) = self.token(finished);
        Some(Lexeme {
            token,
            range: self.offset(finished.source.start)..self.offset(finished.source.end),
            unterminated,
        })
    }

    #[inline]
    fn token(&self, finished: Finished) -> (Token<'_>, bool) {
        match finished.kind {
            Kind::Name(kind, units) => {
                let text = self.held_slice(units);
                (
                    match kind {
                        NameKind::Ident => Token::Ident(text),
                        NameKind::Function => Token::Function(text),
                        NameKind::AtKeyword => Token::AtKeyword(text),
                        NameKind::Hash { identifier } => Token::Hash {
                            value: text,
                            kind: if identifier {
                                HashKind::Id
                            } else {
                                HashKind::Unrestricted
                            },
                        },
                    },
                    false,
                )
            }
            Kind::Number {
                kind,
                integer,
                unit,
            } => {
                let representation = self
                    .text
                    .get(finished.source.start..self.number_end)
                    .unwrap_or("");
                let flag = if integer {
                    NumberKind::Integer
                } else {
                    NumberKind::Number
                };
                let number = Number::new(representation, flag);
                (
                    match kind {
                        RawNumberKind::Number => Token::Number(number),
                        RawNumberKind::Percentage => Token::Percentage(number),
                        RawNumberKind::Dimension => Token::Dimension {
                            number,
                            unit: self.held_slice(unit),
                        },
                    },
                    false,
                )
            }
            Kind::Whitespace => (Token::Whitespace, false),
            Kind::Comment { closed } => (Token::Comment, !closed),
            Kind::String { units, closed } => (Token::String(self.held_slice(units)), !closed),
            Kind::BadString => (Token::BadString, false),
            Kind::Url { units, closed } => (Token::Url(self.held_slice(units)), !closed),
            Kind::BadUrl { closed } => (Token::BadUrl, !closed),
            Kind::Delim(point) => (
                match point {
                    ':' => Token::Colon,
                    ';' => Token::Semicolon,
                    ',' => Token::Comma,
                    '[' => Token::OpenSquare,
                    ']' => Token::CloseSquare,
                    '(' => Token::OpenParen,
                    ')' => Token::CloseParen,
                    '{' => Token::OpenCurly,
                    '}' => Token::CloseCurly,
                    other => Token::Delim(other),
                },
                false,
            ),
            Kind::Cdo => (Token::Cdo, false),
            Kind::Cdc => (Token::Cdc, false),
            Kind::UnicodeRange(range) => (Token::UnicodeRange(range), false),
        }
    }
}

/// What a run did.
#[derive(Clone, Copy, Debug)]
enum Run {
    /// Nothing applied: the machine takes a step.
    Nothing,
    /// The state or position changed: look for another run.
    Progress,
    /// A token is finished.
    Finished,
}
