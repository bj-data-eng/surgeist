//! A host drives the stepwise input and tokenizer protocols through the
//! crate's public surface only.
//!
//! The expectations are written from CSS Syntax Module Level 3 (pinned
//! edition, 1 October 2026): section 3.3 for the preprocessing of the input
//! stream, and sections 4.2 and 4.3 for the tokens. A token's source interval
//! counts original input units, so a carriage return and line feed pair is one
//! scalar that covers two units.

use surgeist_css_parser::input::{InputCursor, InputKind, InputRequest, Scalar};
use surgeist_css_parser::lexer::protocol::{
    NameKind, NumberKind, NumberLayout, ProtocolError, RawToken, RawTokenKind, Record, Span,
    TokenCursor, TokenRequest,
};

/// Preprocesses `units` the way a host must before tokenizing.
fn preprocess(kind: InputKind, units: &[u16]) -> Vec<Scalar> {
    let mut cursor = InputCursor::new(kind, units.len());
    let mut scalars = Vec::new();
    loop {
        match cursor.request() {
            InputRequest::Read { index } => cursor.read(units[index]).unwrap(),
            InputRequest::HeldUnit => cursor.process_held().unwrap(),
            InputRequest::Emit(scalar) => {
                scalars.push(scalar);
                cursor.emitted().unwrap();
            }
            InputRequest::End => {
                if cursor.end().unwrap() {
                    return scalars;
                }
            }
        }
    }
}

/// Tokenizes preprocessed scalars, storing every record the cursor asks for.
fn tokenize(scalars: &[Scalar]) -> Vec<Record> {
    let mut cursor = TokenCursor::new(scalars.len(), 0);
    let mut records = Vec::new();
    loop {
        match cursor.request() {
            TokenRequest::ReadScalar { index } => {
                cursor.read_scalar(index, scalars[index]).unwrap();
            }
            TokenRequest::Compute => cursor.compute().unwrap(),
            TokenRequest::Append { index, record } => {
                assert_eq!(index, records.len(), "slots are assigned in order");
                records.push(record);
                cursor.appended(index).unwrap();
            }
            TokenRequest::Done => return records,
        }
    }
}

fn tokens(records: &[Record]) -> Vec<RawToken> {
    records
        .iter()
        .filter_map(|record| match record {
            Record::Token(token) => Some(*token),
            _ => None,
        })
        .collect()
}

/// The decoded text the host stored in `span`.
fn text(records: &[Record], span: Span) -> String {
    records[span.start..span.end]
        .iter()
        .map(|record| match record {
            Record::NameUnit(unit) => *unit,
            other => panic!("expected a name unit, found {other:?}"),
        })
        .collect()
}

fn utf16(text: &str) -> Vec<u16> {
    text.encode_utf16().collect()
}

#[test]
fn a_carriage_return_line_feed_pair_is_one_whitespace_covering_both_original_units() {
    // 3.3: a CR LF pair is replaced by one LF. 4.3.1: whitespace is one token.
    let records = tokenize(&preprocess(InputKind::Utf16, &utf16("a\r\nb")));
    let found = tokens(&records);
    assert_eq!(found.len(), 3);
    assert_eq!(found[0].source, Span { start: 0, end: 1 });
    assert_eq!(found[1].kind, RawTokenKind::Whitespace);
    assert_eq!(found[1].source, Span { start: 1, end: 3 });
    assert_eq!(found[2].source, Span { start: 3, end: 4 });
}

#[test]
fn a_nul_becomes_a_replacement_character_that_is_an_ident_start() {
    // 3.3 replaces U+0000 with U+FFFD, and U+FFFD is a non-ASCII code point,
    // which section 4.2 admits in an ident.
    let bytes: Vec<u16> = "\0x".bytes().map(u16::from).collect();
    let records = tokenize(&preprocess(InputKind::Utf8, &bytes));
    let found = tokens(&records);
    assert_eq!(found.len(), 1);
    let RawTokenKind::Name {
        kind: NameKind::Ident,
        units,
    } = found[0].kind
    else {
        panic!("expected an ident, found {:?}", found[0].kind);
    };
    assert_eq!(text(&records, units), "\u{fffd}x");
    assert_eq!(found[0].source, Span { start: 0, end: 2 });
}

#[test]
fn utf8_scalars_report_the_original_byte_interval() {
    // U+00E9 is two UTF-8 bytes, so "é1" has the scalars [0, 2) and [2, 3).
    let bytes: Vec<u16> = "é1".bytes().map(u16::from).collect();
    let scalars = preprocess(InputKind::Utf8, &bytes);
    assert_eq!(
        scalars,
        [
            Scalar {
                value: 'é',
                start: 0,
                end: 2
            },
            Scalar {
                value: '1',
                start: 2,
                end: 3
            },
        ]
    );
}

#[test]
fn a_dimension_keeps_its_digits_layout_and_unit_for_the_host_to_convert() {
    // 4.3.3 and 4.3.13: "1.5e3px" is a dimension whose number spelling is
    // "1.5e3" and whose unit is "px". The tokenizer reports the spelling's
    // counts and digits; it never produces a value.
    let records = tokenize(&preprocess(InputKind::Utf16, &utf16("1.5e3px")));
    let found = tokens(&records);
    assert_eq!(found.len(), 1);
    let RawTokenKind::Number {
        kind: NumberKind::Dimension,
        layout,
        digits,
        unit,
    } = found[0].kind
    else {
        panic!("expected a dimension, found {:?}", found[0].kind);
    };
    assert_eq!(
        layout,
        NumberLayout {
            negative: false,
            mantissa_digits: 2,
            fraction_digits: 1,
            exponent_negative: false,
            exponent_digits: 1,
        }
    );
    let digit_values: Vec<(bool, u8)> = records[digits.start..digits.end]
        .iter()
        .map(|record| match record {
            Record::Digit { exponent, value } => (*exponent, *value),
            other => panic!("expected a digit, found {other:?}"),
        })
        .collect();
    assert_eq!(digit_values, [(false, 1), (false, 5), (true, 3)]);
    assert_eq!(text(&records, unit), "px");
    assert_eq!(found[0].source, Span { start: 0, end: 7 });
}

#[test]
fn an_unacknowledged_append_stays_pending_and_blocks_further_work() {
    // The host-visible contract of the protocol: until the host acknowledges
    // the append (because its storage admitted the record), the same request is
    // offered again and no other action is accepted.
    let scalars = preprocess(InputKind::Utf16, &utf16("a"));
    let mut cursor = TokenCursor::new(scalars.len(), 0);
    cursor.read_scalar(0, scalars[0]).unwrap();
    while !matches!(cursor.request(), TokenRequest::Append { .. }) {
        cursor.compute().unwrap();
    }
    let pending = cursor.request();
    assert_eq!(cursor.compute(), Err(ProtocolError::WrongReply));
    assert_eq!(cursor.request(), pending);
    let TokenRequest::Append { index, .. } = pending else {
        unreachable!("the loop above stops at an append");
    };
    assert_eq!(cursor.appended(index + 1), Err(ProtocolError::WrongIndex));
    assert_eq!(cursor.request(), pending);
    cursor.appended(index).unwrap();
    assert_ne!(cursor.request(), pending);
}
