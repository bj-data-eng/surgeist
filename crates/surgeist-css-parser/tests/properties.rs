//! Properties of the lexer over generated text.
//!
//! The case generators and the codec belong to this crate (`tests/common`);
//! `surgeist-fuzz` supplies the bounded, seeded runner. Every run uses a fixed
//! seed and a fixed number of cases, so a failure reproduces.
//!
//! - P1, totality: no text makes the lexer panic, and its lexemes tile the text
//!   with no gap or overlap, each on character boundaries.
//! - P2, input kinds: the free-running lexer over UTF-8 text, the stepwise
//!   tokenizer over the UTF-8 bytes, and the stepwise tokenizer over the UTF-16
//!   units agree on every token's kind, decoded text and extent, the UTF-16
//!   extents being the UTF-8 ones counted in units.
//! - P3, protocol against lexer: a host that refuses an append once, asks again,
//!   and answers wrongly in between gets the same tokens as the lexer, and the
//!   lexical errors that the stepwise tokenizer's flags show are the lexer's.
//! - P4, round trip (section 9): serializing the tokens, with a comment where
//!   the section's table asks for one, and lexing the result gives the same
//!   tokens, modulo whitespace. The serializer is this file's own reading of
//!   section 9 and the escaping of CSSOM 2.1; the crate has none yet.
//!
//! Section 3.2 and the UTF-16 source map are not involved: the stepwise
//! tokenizer reads the units through `InputCursor`, which reports each scalar's
//! original unit interval.

mod common;

use std::fmt::Write as _;
use std::ops::Range;
use std::panic::{AssertUnwindSafe, catch_unwind};

use common::generate::generate;
use surgeist_css_parser::Token;
use surgeist_css_parser::input::{InputCursor, InputKind, InputRequest, Scalar};
use surgeist_css_parser::lexer::protocol::{
    NameKind, NumberKind as RawNumberKind, RawTokenKind, Record, TokenCursor, TokenRequest,
};
use surgeist_css_parser::lexer::{Lexer, Options};
use surgeist_css_parser::token::{HashKind, NumberKind, Sign};
use surgeist_fuzz::engine::{PropertyConfig, run_property};
use surgeist_fuzz::generation::property::bytes;
use surgeist_fuzz::{
    BuildRequirements, CaseFormat, CaseOutcome, CaseTarget, FailureClass, FailureKey, FuzzError,
    FuzzErrorKind, Generation, GeneratorIdentity, InputEncoding, InputLimits, PropertyId, Result,
    TargetDescriptor, TargetDescriptorSpec, TargetId,
};

const CASES: u32 = 600;
const TEXT_BYTES: usize = 512;
const ENTROPY_BYTES: usize = 97;
const SEED: u64 = 0x1508_0005;

// ----------------------------------------------------------------- generators

/// A text of arbitrary code points: every ASCII byte, so NULL, carriage
/// return, form feed, quotes and backslashes meet each other, and a handful of
/// non-ASCII code points around the edges of section 4.2's ranges.
fn raw_text(entropy: &[u8]) -> String {
    const WIDE: [char; 10] = [
        '\u{e9}',
        '\u{2603}',
        '\u{1f600}',
        '\u{a0}',
        '\u{fffd}',
        '\u{fffe}',
        '\u{10ffff}',
        '\u{d7ff}',
        '\u{e000}',
        '\u{b7}',
    ];
    entropy
        .iter()
        .take(96)
        .map(|&byte| {
            if byte < 0x80 {
                char::from(byte)
            } else {
                WIDE[usize::from(byte) % WIDE.len()]
            }
        })
        .collect()
}

/// The first entropy byte picks the family: authored pieces, or raw bytes.
fn text_of(entropy: &[u8]) -> String {
    match entropy.first() {
        Some(first) if first % 2 == 1 => raw_text(&entropy[1..]),
        _ => generate(entropy.get(1..).unwrap_or(&[])),
    }
}

// ----------------------------------------------------------------- the lexer's side

/// A token as the comparison sees it: the same notation for the lexer and the
/// stepwise tokenizer, so the two are compared on kind, decoded text and extent.
#[derive(Clone, Debug, Eq, PartialEq)]
struct Seen {
    what: String,
    range: Range<usize>,
}

fn digits(representation: &str) -> usize {
    representation.bytes().filter(u8::is_ascii_digit).count()
}

fn number_seen(name: &str, number: surgeist_css_parser::token::Number<'_>, unit: &str) -> String {
    format!(
        "{name}(negative={},integer={},digits={},unit={unit:?})",
        number.sign() == Some(Sign::Minus),
        number.kind() == NumberKind::Integer,
        digits(number.representation())
    )
}

fn lexer_seen(text: &str) -> Vec<Seen> {
    let mut lexer = Lexer::new(text, &Options::default());
    let mut seen = Vec::new();
    while let Some(lexeme) = lexer.next_lexeme() {
        let what = match lexeme.token() {
            Token::Ident(value) => format!("Ident({value:?})"),
            Token::Function(value) => format!("Function({value:?})"),
            Token::AtKeyword(value) => format!("AtKeyword({value:?})"),
            Token::Hash { value, kind } => {
                format!("Hash({value:?},id={})", kind == HashKind::Id)
            }
            Token::String(value) => {
                format!("String({value:?},closed={})", !lexeme.is_unterminated())
            }
            Token::Url(value) => format!("Url({value:?},closed={})", !lexeme.is_unterminated()),
            Token::BadString => "BadString".into(),
            Token::BadUrl => format!("BadUrl(closed={})", !lexeme.is_unterminated()),
            Token::Number(number) => number_seen("Number", number, ""),
            Token::Percentage(number) => number_seen("Percentage", number, ""),
            Token::Dimension { number, unit } => number_seen("Dimension", number, unit),
            Token::Whitespace => "Whitespace".into(),
            Token::Comment => format!("Comment(closed={})", !lexeme.is_unterminated()),
            Token::Delim(point) => format!("Delim({point:?})"),
            Token::Colon => "Delim(':')".into(),
            Token::Semicolon => "Delim(';')".into(),
            Token::Comma => "Delim(',')".into(),
            Token::OpenSquare => "Delim('[')".into(),
            Token::CloseSquare => "Delim(']')".into(),
            Token::OpenParen => "Delim('(')".into(),
            Token::CloseParen => "Delim(')')".into(),
            Token::OpenCurly => "Delim('{')".into(),
            Token::CloseCurly => "Delim('}')".into(),
            Token::Cdo => "Cdo".into(),
            Token::Cdc => "Cdc".into(),
            Token::UnicodeRange(_) => "UnicodeRange".into(),
        };
        seen.push(Seen {
            what,
            range: lexeme.range().start.value()..lexeme.range().end.value(),
        });
    }
    seen
}

// ----------------------------------------------------------------- the protocol's side

/// Preprocesses `units` through the input cursor, as a host must.
fn preprocess(kind: InputKind, units: &[u16]) -> Option<Vec<Scalar>> {
    let mut cursor = InputCursor::new(kind, units.len());
    let mut scalars = Vec::new();
    loop {
        match cursor.request() {
            InputRequest::Read { index } => cursor.read(units[index]).ok()?,
            InputRequest::HeldUnit => cursor.process_held().ok()?,
            InputRequest::Emit(scalar) => {
                scalars.push(scalar);
                cursor.emitted().ok()?;
            }
            InputRequest::End => {
                if cursor.end().ok()? {
                    return Some(scalars);
                }
            }
        }
    }
}

/// How a host misbehaves while driving the cursor.
#[derive(Clone, Copy)]
enum Host {
    /// Admits everything at once.
    Admitting,
    /// Refuses the append with this ordinal once, asks again, and answers each
    /// request wrongly in between; the cursor must not change.
    RefusingAppend(usize),
}

/// Runs the stepwise tokenizer to the end over `scalars`; returns its records,
/// or `None` if the cursor reported a protocol error that a correct host cannot
/// cause.
fn run_protocol(scalars: &[Scalar], host: Host) -> Option<Vec<Record>> {
    let mut cursor = TokenCursor::new(scalars.len(), 0);
    let mut records = Vec::new();
    let mut appends = 0;
    loop {
        match cursor.request() {
            TokenRequest::ReadScalar { index } => cursor.read_scalar(index, scalars[index]).ok()?,
            TokenRequest::Compute => cursor.compute().ok()?,
            TokenRequest::Append { index, record } => {
                if matches!(host, Host::RefusingAppend(ordinal) if appends == ordinal) {
                    let pending = cursor.request();
                    // A refused append is not acknowledged: every wrong reply
                    // is an error and the request does not change.
                    if cursor.compute().is_ok()
                        || cursor.appended(index + 1).is_ok()
                        || cursor.read_scalar(0, scalars.first().copied()?).is_ok()
                        || cursor.request() != pending
                    {
                        return None;
                    }
                }
                appends += 1;
                if index != records.len() {
                    return None;
                }
                records.push(record);
                cursor.appended(index).ok()?;
            }
            TokenRequest::Done => return Some(records),
        }
    }
}

fn decoded(records: &[Record], span: surgeist_css_parser::lexer::protocol::Span) -> String {
    records
        .get(span.start..span.end)
        .unwrap_or(&[])
        .iter()
        .filter_map(|record| match record {
            Record::NameUnit(unit) => Some(*unit),
            _ => None,
        })
        .collect()
}

/// The protocol's tokens in the lexer's notation, with extents as units counted
/// by `unit_offset` from the original byte offsets they would have.
fn protocol_seen(records: &[Record]) -> Vec<Seen> {
    let mut seen = Vec::new();
    for record in records {
        let Record::Token(token) = record else {
            continue;
        };
        let what = match token.kind {
            RawTokenKind::Name { kind, units } => {
                let value = decoded(records, units);
                match kind {
                    NameKind::Ident => format!("Ident({value:?})"),
                    NameKind::Function => format!("Function({value:?})"),
                    NameKind::AtKeyword => format!("AtKeyword({value:?})"),
                    NameKind::Hash { identifier } => format!("Hash({value:?},id={identifier})"),
                }
            }
            RawTokenKind::Number {
                kind, layout, unit, ..
            } => {
                let name = match kind {
                    RawNumberKind::Number => "Number",
                    RawNumberKind::Percentage => "Percentage",
                    RawNumberKind::Dimension => "Dimension",
                };
                format!(
                    "{name}(negative={},integer={},digits={},unit={:?})",
                    layout.negative,
                    layout.fraction_digits == 0 && layout.exponent_digits == 0,
                    layout.mantissa_digits + layout.exponent_digits,
                    decoded(records, unit)
                )
            }
            RawTokenKind::Whitespace => "Whitespace".into(),
            RawTokenKind::Comment { closed } => format!("Comment(closed={closed})"),
            RawTokenKind::String { units, closed } => {
                format!("String({:?},closed={closed})", decoded(records, units))
            }
            RawTokenKind::BadString => "BadString".into(),
            RawTokenKind::Url { units, closed } => {
                format!("Url({:?},closed={closed})", decoded(records, units))
            }
            RawTokenKind::BadUrl { closed } => format!("BadUrl(closed={closed})"),
            RawTokenKind::Delim(point) => format!("Delim({point:?})"),
            RawTokenKind::Cdo => "Cdo".into(),
            RawTokenKind::Cdc => "Cdc".into(),
        };
        seen.push(Seen {
            what,
            range: token.source.start..token.source.end,
        });
    }
    seen
}

/// The lexical errors that the stepwise tokenizer's closed flags show: an
/// unterminated comment, string or url, and a bad string. The other kinds are
/// causes the stepwise tokenizer does not report.
fn flagged_errors(records: &[Record]) -> Vec<(&'static str, Range<usize>)> {
    records
        .iter()
        .filter_map(|record| {
            let Record::Token(token) = record else {
                return None;
            };
            let kind = match token.kind {
                RawTokenKind::Comment { closed: false } => "unterminated_comment",
                RawTokenKind::String { closed: false, .. } => "unterminated_string",
                RawTokenKind::BadString => "bad_string",
                RawTokenKind::Url { closed: false, .. } => "unterminated_url",
                _ => return None,
            };
            Some((kind, token.source.start..token.source.end))
        })
        .collect()
}

fn lexer_flagged_errors(text: &str) -> Vec<(&'static str, Range<usize>)> {
    let mut lexer = Lexer::new(text, &Options::default());
    while lexer.next_lexeme().is_some() {}
    lexer
        .errors()
        .iter()
        .filter_map(|error| {
            let kind = if error.is_unterminated_comment() {
                "unterminated_comment"
            } else if error.is_unterminated_string() {
                "unterminated_string"
            } else if error.is_bad_string() {
                "bad_string"
            } else if error.is_unterminated_url() {
                "unterminated_url"
            } else {
                return None;
            };
            Some((kind, error.range().start.value()..error.range().end.value()))
        })
        .collect()
}

// ----------------------------------------------------------------- the round trip (section 9)

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Edge {
    Ident,
    AtKeyword,
    Hash,
    Dimension,
    Number,
    Function,
    Url,
    BadUrl,
    Percentage,
    Cdc,
    OpenParen,
    Delim(char),
    Other,
}

fn edge(token: &Token<'_>) -> Edge {
    match token {
        Token::Ident(_) => Edge::Ident,
        Token::AtKeyword(_) => Edge::AtKeyword,
        Token::Hash { .. } => Edge::Hash,
        Token::Dimension { .. } => Edge::Dimension,
        Token::Number(_) => Edge::Number,
        Token::Function(_) => Edge::Function,
        Token::Url(_) => Edge::Url,
        Token::BadUrl => Edge::BadUrl,
        Token::Percentage(_) => Edge::Percentage,
        Token::Cdc => Edge::Cdc,
        Token::OpenParen => Edge::OpenParen,
        Token::Delim(point) => Edge::Delim(*point),
        _ => Edge::Other,
    }
}

/// Section 9's table: whether a comment must separate `first` from `second`.
fn needs_comment(first: Edge, second: Edge) -> bool {
    use Edge::{
        AtKeyword, BadUrl, Cdc, Delim, Dimension, Function, Hash, Ident, Number, OpenParen,
        Percentage, Url,
    };
    let name_like = |edge| {
        matches!(
            edge,
            Ident | Function | Url | BadUrl | Delim('-') | Number | Percentage | Dimension | Cdc
        )
    };
    match first {
        Ident => name_like(second) || second == OpenParen,
        AtKeyword | Hash | Dimension | Delim('#') | Delim('-') => name_like(second),
        Number => {
            matches!(
                second,
                Ident | Function | Url | BadUrl | Number | Percentage | Dimension | Cdc
            ) || second == Delim('%')
        }
        Delim('@') => matches!(second, Ident | Function | Url | BadUrl | Delim('-') | Cdc),
        Delim('.') | Delim('+') => matches!(second, Number | Percentage | Dimension),
        Delim('/') => second == Delim('*'),
        _ => false,
    }
}

fn hex_escape(point: char) -> String {
    format!("\\{:x} ", u32::from(point))
}

/// CSSOM 2.1, serialize an identifier.
fn identifier(value: &str) -> String {
    let mut out = String::new();
    let points: Vec<char> = value.chars().collect();
    for (index, &point) in points.iter().enumerate() {
        let first = index == 0;
        let after_hyphen = index == 1 && points[0] == '-';
        if point == '\0' {
            out.push('\u{fffd}');
        } else if ('\u{1}'..='\u{1f}').contains(&point)
            || point == '\u{7f}'
            || ((first || after_hyphen) && point.is_ascii_digit())
        {
            out.push_str(&hex_escape(point));
        } else if first && point == '-' && points.len() == 1 {
            out.push_str("\\-");
        } else if !point.is_ascii() || point == '-' || point == '_' || point.is_ascii_alphanumeric()
        {
            out.push(point);
        } else {
            out.push('\\');
            out.push(point);
        }
    }
    out
}

/// The name part of a hash: every ident code point is itself.
fn name(value: &str) -> String {
    let mut out = String::new();
    for point in value.chars() {
        if point == '\0' {
            out.push('\u{fffd}');
        } else if ('\u{1}'..='\u{1f}').contains(&point) || point == '\u{7f}' {
            out.push_str(&hex_escape(point));
        } else if !point.is_ascii() || point == '-' || point == '_' || point.is_ascii_alphanumeric()
        {
            out.push(point);
        } else {
            out.push('\\');
            out.push(point);
        }
    }
    out
}

fn string(value: &str) -> String {
    let mut out = String::from("\"");
    for point in value.chars() {
        match point {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            point if point == '\n' || point.is_control() => out.push_str(&hex_escape(point)),
            point => out.push(point),
        }
    }
    out.push('"');
    out
}

fn url(value: &str) -> String {
    let mut out = String::from("url(");
    for point in value.chars() {
        match point {
            '"' | '\'' | '(' | ')' | '\\' | ' ' => {
                out.push('\\');
                out.push(point);
            }
            point if point.is_control() => out.push_str(&hex_escape(point)),
            point => out.push(point),
        }
    }
    out.push(')');
    out
}

/// A dimension's unit may read as the exponent of the number before it.
fn unit(value: &str) -> String {
    let escaped = identifier(value);
    let mut points = value.chars();
    match (points.next(), points.next()) {
        (Some(first @ ('e' | 'E')), Some(second))
            if second.is_ascii_digit() || second == '-' || second == '+' =>
        {
            format!("{}{}", hex_escape(first), name(&value[1..]))
        }
        _ => escaped,
    }
}

/// Writes one token so that it lexes back to itself. Bad tokens have no value to
/// write, so they are not written.
fn write_token(token: &Token<'_>) -> Option<String> {
    Some(match token {
        Token::Ident(value) => identifier(value),
        Token::Function(value) => format!("{}(", identifier(value)),
        Token::AtKeyword(value) => format!("@{}", identifier(value)),
        Token::Hash { value, kind } => format!(
            "#{}",
            if *kind == HashKind::Id {
                identifier(value)
            } else {
                name(value)
            }
        ),
        Token::String(value) => string(value),
        Token::Url(value) => url(value),
        Token::Delim('\\') => "\\\n".to_owned(),
        Token::Delim(point) => point.to_string(),
        Token::Number(number) => number.representation().to_owned(),
        Token::Percentage(number) => format!("{}%", number.representation()),
        Token::Dimension {
            number,
            unit: value,
        } => {
            format!("{}{}", number.representation(), unit(value))
        }
        Token::Colon => ":".to_owned(),
        Token::Semicolon => ";".to_owned(),
        Token::Comma => ",".to_owned(),
        Token::OpenSquare => "[".to_owned(),
        Token::OpenParen => "(".to_owned(),
        Token::OpenCurly => "{".to_owned(),
        Token::CloseSquare => "]".to_owned(),
        Token::CloseParen => ")".to_owned(),
        Token::CloseCurly => "}".to_owned(),
        Token::Cdo => "<!--".to_owned(),
        Token::Cdc => "-->".to_owned(),
        Token::UnicodeRange(_)
        | Token::BadString
        | Token::BadUrl
        | Token::Whitespace
        | Token::Comment => return None,
    })
}

/// The tokens of `text` as owned renderings with whitespace and comments
/// removed, which is what the round trip preserves.
fn tokens_of(text: &str) -> Vec<String> {
    let mut lexer = Lexer::new(text, &Options::default());
    let mut out = Vec::new();
    while let Some(lexeme) = lexer.next_lexeme() {
        if !lexeme.token().is_trivia() {
            out.push(common::render(lexeme.token()));
        }
    }
    out
}

/// Serializes the tokens of `text`, then lexes the result and compares. The
/// lexer lends each token only until the next call, so each is written out
/// before the next is asked for. Text with a bad string or a bad url is not
/// serialized: they have no value to write.
fn round_trip(text: &str) -> std::result::Result<(), String> {
    let mut lexer = Lexer::new(text, &Options::default());
    let mut out = String::new();
    let mut previous: Option<Edge> = None;
    while let Some(lexeme) = lexer.next_lexeme() {
        let token = lexeme.token();
        match token {
            Token::BadString | Token::BadUrl => return Ok(()),
            Token::Comment => continue,
            Token::Whitespace => {
                if !out.is_empty() && !out.ends_with(' ') {
                    out.push(' ');
                }
                previous = None;
                continue;
            }
            _ => {}
        }
        let current = edge(&token);
        if previous.is_some_and(|before| needs_comment(before, current)) {
            out.push_str("/**/");
        }
        let Some(written) = write_token(&token) else {
            return Ok(());
        };
        out.push_str(&written);
        previous = Some(current);
    }
    let (before, after) = (tokens_of(text), tokens_of(&out));
    if before == after {
        Ok(())
    } else {
        let mut detail = String::new();
        let _ = write!(
            detail,
            "{text:?} serialized as {out:?}: {before:?} against {after:?}"
        );
        Err(detail)
    }
}

// ----------------------------------------------------------------- the target

fn property(name: &str) -> PropertyId {
    PropertyId::new(format!("surgeist-css-parser::{name}")).expect("owner-qualified property")
}

const TOTALITY: &str = "totality-and-tiling";
const INPUT_KINDS: &str = "input-kinds-agree";
const PROTOCOL: &str = "protocol-equals-lexer";
const ROUND_TRIP: &str = "round-trip";
const PROPERTIES: [&str; 4] = [TOTALITY, INPUT_KINDS, PROTOCOL, ROUND_TRIP];

struct Target {
    descriptor: TargetDescriptor,
}

impl Target {
    fn new() -> Self {
        let descriptor = TargetDescriptor::new(TargetDescriptorSpec {
            id: TargetId::new("surgeist-css-parser::lexer").unwrap(),
            owner: "surgeist-css-parser".into(),
            binary: "surgeist-css-parser-lexer-property".into(),
            properties: PROPERTIES.map(property).to_vec(),
            source_references: vec![
                "https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#tokenizer-algorithms".into(),
                "https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#input-preprocessing".into(),
                "https://www.w3.org/TR/2026/CRD-css-syntax-3-20261001/#serialization".into(),
            ],
            format: CaseFormat::new("surgeist-css-parser::source-text", 1).unwrap(),
            generator_identity: GeneratorIdentity::new("surgeist-css-parser::pieces-and-raw", 1)
                .unwrap(),
            encoding: InputEncoding::RawBytes,
            limits: InputLimits::new(TEXT_BYTES as u64, 0, 0, 0, 0).unwrap(),
            requirements: BuildRequirements::new(vec![], vec![]).unwrap(),
            min_evaluated: u64::from(CASES),
            max_generation_rejections: 0,
        })
        .unwrap();
        Self { descriptor }
    }
}

fn failure(name: &str, detail: String) -> Result<CaseOutcome> {
    let mut detail = detail.replace(char::is_control, " ");
    detail.truncate(2048);
    CaseOutcome::failure(
        FailureKey::new(property(name), FailureClass::PropertyViolation, None)?,
        detail.trim().to_owned(),
    )
}

fn utf16_offset(text: &str, byte: usize) -> usize {
    text.get(..byte)
        .map_or(0, |prefix| prefix.encode_utf16().count())
}

/// P1.
fn check_totality(text: &str) -> std::result::Result<(), String> {
    let seen = catch_unwind(AssertUnwindSafe(|| lexer_seen(text)))
        .map_err(|_| format!("the lexer panicked on {text:?}"))?;
    let mut at = 0;
    for token in &seen {
        if token.range.start != at || token.range.end <= token.range.start {
            return Err(format!("{text:?}: a gap or empty lexeme at {at}: {seen:?}"));
        }
        if !text.is_char_boundary(token.range.start) || !text.is_char_boundary(token.range.end) {
            return Err(format!("{text:?}: a lexeme splits a character: {token:?}"));
        }
        at = token.range.end;
    }
    if at != text.len() {
        return Err(format!("{text:?}: lexemes end at {at}, not {}", text.len()));
    }
    let mut lexer = Lexer::new(text, &Options::default());
    while lexer.next_lexeme().is_some() {}
    for error in lexer.errors() {
        let range = error.range();
        if range.start > range.end || range.end.value() > text.len() {
            return Err(format!("{text:?}: an error outside the text: {error:?}"));
        }
    }
    Ok(())
}

/// P2.
fn check_input_kinds(text: &str) -> std::result::Result<(), String> {
    let lexer = lexer_seen(text);
    let bytes: Vec<u16> = text.bytes().map(u16::from).collect();
    let units: Vec<u16> = text.encode_utf16().collect();
    let utf8 = preprocess(InputKind::Utf8, &bytes).ok_or("the UTF-8 input cursor failed")?;
    let utf16 = preprocess(InputKind::Utf16, &units).ok_or("the UTF-16 input cursor failed")?;
    let by_bytes =
        protocol_seen(&run_protocol(&utf8, Host::Admitting).ok_or("the protocol failed")?);
    if by_bytes != lexer {
        return Err(format!(
            "{text:?}: stepwise over bytes {by_bytes:?} against lexer {lexer:?}"
        ));
    }
    let by_units =
        protocol_seen(&run_protocol(&utf16, Host::Admitting).ok_or("the protocol failed")?);
    let expected: Vec<Seen> = lexer
        .iter()
        .map(|seen| Seen {
            what: seen.what.clone(),
            range: utf16_offset(text, seen.range.start)..utf16_offset(text, seen.range.end),
        })
        .collect();
    if by_units != expected {
        return Err(format!(
            "{text:?}: stepwise over units {by_units:?} against lexer in units {expected:?}"
        ));
    }
    Ok(())
}

/// P3.
fn check_protocol(text: &str, ordinal: usize) -> std::result::Result<(), String> {
    let bytes: Vec<u16> = text.bytes().map(u16::from).collect();
    let scalars = preprocess(InputKind::Utf8, &bytes).ok_or("the UTF-8 input cursor failed")?;
    let plain = run_protocol(&scalars, Host::Admitting).ok_or("the protocol failed")?;
    let appends = plain.len();
    let refused = run_protocol(&scalars, Host::RefusingAppend(ordinal % appends.max(1)))
        .ok_or("a refused append changed the cursor")?;
    if plain != refused {
        return Err(format!("{text:?}: a refused append changed the output"));
    }
    let flagged = flagged_errors(&plain);
    let lexed = lexer_flagged_errors(text);
    if flagged != lexed {
        return Err(format!(
            "{text:?}: the stepwise flags show {flagged:?}, the lexer reports {lexed:?}"
        ));
    }
    Ok(())
}

impl CaseTarget for Target {
    type Case = String;

    fn descriptor(&self) -> &TargetDescriptor {
        &self.descriptor
    }

    fn decode(&self, input: &[u8]) -> Result<String> {
        String::from_utf8(input.to_vec())
            .map_err(|_| FuzzError::new(FuzzErrorKind::InvalidCase, "decode css text", "UTF-8"))
    }

    fn encode(&self, case: &String) -> Result<Vec<u8>> {
        Ok(case.as_bytes().to_vec())
    }

    fn generate(&self, input: &[u8]) -> Result<Generation<String>> {
        Ok(Generation::Generated(text_of(input)))
    }

    fn evaluate(&self, text: &String) -> Result<CaseOutcome> {
        if let Err(detail) = check_totality(text) {
            return failure(TOTALITY, detail);
        }
        if let Err(detail) = check_input_kinds(text) {
            return failure(INPUT_KINDS, detail);
        }
        if let Err(detail) = check_protocol(text, text.len()) {
            return failure(PROTOCOL, detail);
        }
        if let Err(detail) = round_trip(text) {
            return failure(ROUND_TRIP, detail);
        }
        CaseOutcome::evaluated(PROPERTIES.map(property).to_vec())
    }
}

#[test]
fn generated_text_is_lexed_totally_consistently_across_input_kinds_and_round_trips() {
    let target = Target::new();
    let report = run_property(
        &target,
        &bytes(ENTROPY_BYTES, target.descriptor().limits()),
        PropertyConfig {
            cases: CASES,
            max_rejections: 0,
            max_shrink_iterations: 96,
            seed: SEED,
        },
        &mut |_| Ok(()),
    )
    .expect("property run completes with sufficient coverage");
    if let Some(failure) = report.failure {
        panic!(
            "{:?}: {}\ntext: {:?}",
            failure.outcome.failure_key(),
            failure.outcome.detail().unwrap_or_default(),
            String::from_utf8_lossy(&failure.canonical)
        );
    }
    assert!(report.counts.evaluated() >= u64::from(CASES));
}

#[test]
fn the_properties_detect_a_wrong_lexer_result() {
    // The checks must be able to fail. Two sequences of tokens that differ in a
    // decoded text or an extent are different, and a serialization that drops a
    // required comment does not round-trip.
    let ours = lexer_seen("a(b) 1px");
    let mut changed = ours.clone();
    changed[0].what = "Function(\"z\")".into();
    assert_ne!(ours, changed);
    let mut moved = ours.clone();
    moved[1].range = 3..4;
    assert_ne!(ours, moved);
    // `1` then `px` written without the comment of section 9's table lexes as
    // one dimension.
    assert_eq!(tokens_of("1/**/px").len(), 2);
    assert_eq!(tokens_of("1px").len(), 1);
    assert!(round_trip("1/**/px").is_ok());
    assert!(needs_comment(Edge::Number, Edge::Ident));
    assert!(!needs_comment(Edge::Ident, Edge::Delim('.')));
}

#[test]
fn raw_and_authored_families_both_reach_hard_inputs() {
    // The raw family reaches NULL, carriage return and form feed; the authored
    // family reaches every escape of block punctuation.
    let raw = text_of(&[1, 0, 13, 12, 92, 10, 34]);
    assert!(raw.contains('\0') && raw.contains('\r') && raw.contains('\u{c}'));
    let mut seen_escape = false;
    for seed in 0..64_u8 {
        let text = text_of(&[
            0,
            seed,
            seed.wrapping_mul(7),
            seed.wrapping_mul(13),
            seed ^ 0x55,
        ]);
        seen_escape |= ["\\{", "\\}", "\\(", "\\)", "\\[", "\\]"]
            .iter()
            .any(|escape| text.contains(escape));
    }
    assert!(seen_escape);
}

/// Every text of up to three symbols over an alphabet that holds one code point
/// of each class section 4.3.1 distinguishes, of four over a subset, and of up
/// to five over a small set of multi-code-point symbols (`url(`, an escape with
/// its terminator, an escaped parenthesis, a comment opener, a unicode-range
/// opener), is lexed as the stepwise tokenizer reads it, with the same lexical
/// errors. This is the check that no run of the lexer decides a token
/// differently from the machine's steps, in the cases where a run starts, ends
/// or is cut short.
#[test]
fn every_short_text_is_lexed_as_the_stepwise_tokenizer_reads_it() {
    const WIDE: [&str; 26] = [
        "a", "u", "U", "e", "1", ".", "+", "-", "%", "\"", "'", "\\", "\n", "\r", "\0", "(", ")",
        "#", "@", "/", "*", "<", "!", " ", "\u{e9}", "_",
    ];
    const NARROW: [&str; 13] = [
        "a", "1", ".", "-", "\\", "\n", "\"", "(", ")", "#", "/", "*", "e",
    ];
    const SYMBOLS: [&str; 10] = [
        "url(", "\\31 ", "\\)", "/*", "U+", "a", "1", " ", "\"", "\n",
    ];
    fn texts(alphabet: &[&str], length: usize, mut visit: impl FnMut(&str)) {
        let mut indexes = vec![0_usize; length];
        loop {
            let text: String = indexes.iter().map(|&index| alphabet[index]).collect();
            visit(&text);
            let mut place = 0;
            while place < length {
                indexes[place] += 1;
                if indexes[place] < alphabet.len() {
                    break;
                }
                indexes[place] = 0;
                place += 1;
            }
            if place == length {
                return;
            }
        }
    }
    let mut checked = 0_usize;
    let mut check = |text: &str| {
        checked += 1;
        if let Err(detail) = check_input_kinds(text).and_then(|()| check_protocol(text, 0)) {
            panic!("{detail}");
        }
    };
    for length in 1..=3 {
        texts(&WIDE, length, &mut check);
    }
    texts(&NARROW, 4, &mut check);
    for length in 1..=5 {
        texts(&SYMBOLS, length, &mut check);
    }
    assert!(checked > 150_000, "{checked} texts checked");
}
