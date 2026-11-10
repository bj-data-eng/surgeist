//! The non-ASCII ident code points of CSS Syntax Module Level 3 (pinned
//! edition, 1 October 2026), section 4.2, across every layer that reads them:
//! the lexer, the `Document` built from it, the stepwise tokenizer protocol a
//! metered host drives, and the serializer that must write an identifier so
//! that it reads back as itself (section 9).
//!
//! The expectations come from one table, [`ADMITTED`], whose ranges are the
//! bullets of the section 4.2 definition of "non-ASCII ident code point",
//! copied from the pinned text, and from sections 4.3.1 to 4.3.5 and 4.3.9:
//! an admitted code point starts and continues an ident sequence, and any other
//! code point at or above U+0080 ends one and is a `<delim-token>` of its own.
//! Nothing here asks the crate's predicate what an ident code point is.

mod common;

use surgeist_css_parser::document::Component;
use surgeist_css_parser::input::{InputCursor, InputKind, InputRequest, Scalar};
use surgeist_css_parser::lexer::protocol::{
    NameKind, NumericTokenKind, RawTokenKind, Record, TokenCursor, TokenRequest,
};
use surgeist_css_parser::lexer::{Lexer, Options};
use surgeist_css_parser::serialize;
use surgeist_css_parser::token::HashKind;
use surgeist_css_parser::{Document, Token};

use common::ranges::{ADMITTED, admitted};
use common::tree::parse;

/// The scalar values next to the edges of every range, on both sides, and the
/// code points the review of this rule names.
fn edge_points() -> Vec<char> {
    let mut points = vec![
        '\u{80}',
        '\u{9f}',
        '\u{a0}',
        '\u{a1}',
        '\u{b6}',
        '\u{b7}',
        '\u{b8}',
        '\u{bf}',
        '\u{c0}',
        '\u{d6}',
        '\u{d7}',
        '\u{d8}',
        '\u{f6}',
        '\u{f7}',
        '\u{f8}',
        '\u{37d}',
        '\u{37e}',
        '\u{37f}',
        '\u{1fff}',
        '\u{2000}',
        '\u{200b}',
        '\u{200c}',
        '\u{200d}',
        '\u{200e}',
        '\u{2028}',
        '\u{2029}',
        '\u{203e}',
        '\u{203f}',
        '\u{2040}',
        '\u{2041}',
        '\u{206f}',
        '\u{2070}',
        '\u{218f}',
        '\u{2190}',
        '\u{2603}',
        '\u{2bff}',
        '\u{2c00}',
        '\u{2fef}',
        '\u{2ff0}',
        '\u{3000}',
        '\u{3001}',
        '\u{d7ff}',
        '\u{e000}',
        '\u{f8ff}',
        '\u{f900}',
        '\u{fdcf}',
        '\u{fdd0}',
        '\u{fdef}',
        '\u{fdf0}',
        '\u{feff}',
        '\u{fffd}',
        '\u{fffe}',
        '\u{ffff}',
        '\u{10000}',
        '\u{1f600}',
        '\u{10ffff}',
    ];
    for &(low, high) in &ADMITTED {
        points.extend(char::from_u32(low));
        points.extend(char::from_u32(high));
        // The scalar values on either side: a surrogate is not one.
        points.extend((0..low).rev().find_map(char::from_u32));
        points.extend((high + 1..=0x10_ffff).find_map(char::from_u32));
    }
    points.sort_unstable();
    points.dedup();
    points
}

/// A token in the notation these tests share, so that one expectation serves
/// the lexer, the `Document` and the protocol.
fn delim(point: char) -> String {
    format!("Delim:U+{:04X}", u32::from(point))
}

fn shape_of(token: Token<'_>) -> String {
    match token {
        Token::Ident(value) => format!("Ident:{value}"),
        Token::Function(value) => format!("Function:{value}"),
        Token::AtKeyword(value) => format!("AtKeyword:{value}"),
        Token::Hash {
            value,
            kind: HashKind::Id,
        } => format!("HashId:{value}"),
        Token::Hash {
            value,
            kind: HashKind::Unrestricted,
        } => format!("HashUnrestricted:{value}"),
        Token::Url(value) => format!("Url:{value}"),
        Token::Delim(point) => delim(point),
        Token::Number(_) => "Number".to_owned(),
        Token::Dimension { unit, .. } => format!("Dimension:{unit}"),
        Token::Whitespace => "Whitespace".to_owned(),
        Token::Comment => "Comment".to_owned(),
        Token::OpenParen => "OpenParen".to_owned(),
        Token::CloseParen => "CloseParen".to_owned(),
        other => format!("{other:?}"),
    }
}

/// The text of `value` after one spelling, `{c}` standing for the code point.
fn at(template: &str, point: char) -> String {
    template.replace("{c}", &point.to_string())
}

/// One input, with the tokens section 4.3 gives it when `point` is admitted
/// and when it is not. The inputs place the code point where an ident sequence
/// starts and where it continues, in every token that holds one.
fn cases(point: char) -> Vec<(String, Vec<String>)> {
    let c = |template: &str| at(template, point);
    let is = admitted(point);
    let d = delim(point);
    let s = |text: &str| text.to_owned();
    let pick = |yes: Vec<String>, no: Vec<String>| if is { yes } else { no };
    vec![
        // 4.3.4: an ident-like token; 4.3.12: its name.
        (
            c("a{c}b"),
            pick(
                vec![c("Ident:a{c}b")],
                vec![s("Ident:a"), d.clone(), s("Ident:b")],
            ),
        ),
        (
            c("{c}b"),
            pick(vec![c("Ident:{c}b")], vec![d.clone(), s("Ident:b")]),
        ),
        (c("{c}"), pick(vec![c("Ident:{c}")], vec![d.clone()])),
        (
            c("a{c}"),
            pick(vec![c("Ident:a{c}")], vec![s("Ident:a"), d.clone()]),
        ),
        (
            c("{c}{c}"),
            pick(vec![c("Ident:{c}{c}")], vec![d.clone(), d.clone()]),
        ),
        // 4.3.9: a hyphen-minus then an ident-start code point starts an ident.
        (
            c("-{c}b"),
            pick(
                vec![c("Ident:-{c}b")],
                vec![s("Delim:U+002D"), d.clone(), s("Ident:b")],
            ),
        ),
        (
            c("--a{c}b"),
            pick(
                vec![c("Ident:--a{c}b")],
                vec![s("Ident:--a"), d.clone(), s("Ident:b")],
            ),
        ),
        // 4.3.4: a function; 4.3.3 style: the name is an ident sequence.
        (
            c("a{c}b("),
            pick(
                vec![c("Function:a{c}b")],
                vec![s("Ident:a"), d.clone(), s("Function:b")],
            ),
        ),
        (
            c("{c}("),
            pick(vec![c("Function:{c}")], vec![d.clone(), s("OpenParen")]),
        ),
        // 4.3.4: `url` is only the name `url`.
        (
            c("u{c}rl(x)"),
            pick(
                vec![c("Function:u{c}rl"), s("Ident:x"), s("CloseParen")],
                vec![
                    s("Ident:u"),
                    d.clone(),
                    s("Function:rl"),
                    s("Ident:x"),
                    s("CloseParen"),
                ],
            ),
        ),
        // 4.3.6: any code point is part of an unquoted url; 4.3.4 then reads
        // what follows it.
        (c("url({c}x)"), vec![c("Url:{c}x")]),
        (
            c("url(x){c}y"),
            pick(
                vec![s("Url:x"), c("Ident:{c}y")],
                vec![s("Url:x"), d.clone(), s("Ident:y")],
            ),
        ),
        // 4.3.1: an at-keyword needs an ident sequence to follow `@`.
        (
            c("@{c}b"),
            pick(
                vec![c("AtKeyword:{c}b")],
                vec![s("Delim:U+0040"), d.clone(), s("Ident:b")],
            ),
        ),
        (
            c("@a{c}b"),
            pick(
                vec![c("AtKeyword:a{c}b")],
                vec![s("AtKeyword:a"), d.clone(), s("Ident:b")],
            ),
        ),
        // 4.3.1: a hash holds ident code points; its flag is `id` when they
        // would start an ident sequence.
        (
            c("#{c}b"),
            pick(
                vec![c("HashId:{c}b")],
                vec![s("Delim:U+0023"), d.clone(), s("Ident:b")],
            ),
        ),
        (
            c("#a{c}b"),
            pick(
                vec![c("HashId:a{c}b")],
                vec![s("HashId:a"), d.clone(), s("Ident:b")],
            ),
        ),
        (
            c("#-{c}b"),
            pick(
                vec![c("HashId:-{c}b")],
                vec![s("HashUnrestricted:-"), d.clone(), s("Ident:b")],
            ),
        ),
        (
            c("#1{c}b"),
            pick(
                vec![c("HashUnrestricted:1{c}b")],
                vec![s("HashUnrestricted:1"), d.clone(), s("Ident:b")],
            ),
        ),
        // 4.3.3: a unit is an ident sequence.
        (
            c("1{c}b"),
            pick(
                vec![c("Dimension:{c}b")],
                vec![s("Number"), d.clone(), s("Ident:b")],
            ),
        ),
        (
            c("1a{c}b"),
            pick(
                vec![c("Dimension:a{c}b")],
                vec![s("Dimension:a"), d.clone(), s("Ident:b")],
            ),
        ),
        // 4.3.2: a comment is a token of its own, so it ends a name.
        (
            c("a/**/{c}b"),
            pick(
                vec![s("Ident:a"), s("Comment"), c("Ident:{c}b")],
                vec![s("Ident:a"), s("Comment"), d.clone(), s("Ident:b")],
            ),
        ),
        // 4.3.7: an escape consumes any code point that is not a newline or a
        // hex digit, so the name holds it whatever section 4.2 says.
        (c("a\\{c}b"), vec![c("Ident:a{c}b")]),
        (c("\\{c}b"), vec![c("Ident:{c}b")]),
        (c("@\\{c}b"), vec![c("AtKeyword:{c}b")]),
        (c("#\\{c}b"), vec![c("HashId:{c}b")]),
        // Whitespace is only the whitespace of 4.2.
        (
            c("a b{c}"),
            pick(
                vec![s("Ident:a"), s("Whitespace"), c("Ident:b{c}")],
                vec![s("Ident:a"), s("Whitespace"), s("Ident:b"), d],
            ),
        ),
    ]
}

// ------------------------------------------------------------------ the lexer

fn lexer_shapes(text: &str) -> Vec<String> {
    let mut lexer = Lexer::new(text, &Options::default());
    let mut shapes = Vec::new();
    while let Some(lexeme) = lexer.next_lexeme() {
        shapes.push(shape_of(lexeme.token()));
    }
    assert!(
        lexer.errors().is_empty(),
        "{text:?} holds a lexical error: {:?}",
        lexer.errors()
    );
    shapes
}

fn assert_every_case(mut tokenize: impl FnMut(&str) -> Vec<String>) {
    let mut failures = Vec::new();
    for point in edge_points() {
        for (text, expected) in cases(point) {
            let found = tokenize(&text);
            if found != expected {
                failures.push(format!(
                    "U+{:04X} in {text:?}: expected {expected:?}, found {found:?}",
                    u32::from(point)
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} cases fail:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

#[test]
fn the_lexer_ends_an_ident_at_an_excluded_code_point_and_continues_it_at_an_admitted_one() {
    assert_every_case(lexer_shapes);
}

#[test]
fn the_lexer_forms_an_ident_from_every_scalar_value_exactly_when_section_4_2_admits_it() {
    // Every scalar value from U+0080, at the start of an ident and inside one.
    for code in 0x80_u32..=0x10_ffff {
        let Some(point) = char::from_u32(code) else {
            continue;
        };
        let inside = format!("a{point}b");
        let start = format!("{point}b");
        let (inside_expected, start_expected) = if admitted(point) {
            (
                vec![format!("Ident:a{point}b")],
                vec![format!("Ident:{point}b")],
            )
        } else {
            (
                vec!["Ident:a".to_owned(), delim(point), "Ident:b".to_owned()],
                vec![delim(point), "Ident:b".to_owned()],
            )
        };
        assert_eq!(
            lexer_shapes(&inside),
            inside_expected,
            "U+{code:04X} inside"
        );
        assert_eq!(
            lexer_shapes(&start),
            start_expected,
            "U+{code:04X} at the start"
        );
    }
}

#[test]
fn an_ascii_code_point_is_unchanged_by_the_non_ascii_rule() {
    // Letters, digits, the low line and the hyphen-minus still form idents.
    assert_eq!(lexer_shapes("a-Z_09"), ["Ident:a-Z_09"]);
    assert_eq!(lexer_shapes("_9"), ["Ident:_9"]);
    // U+007F is a non-printable code point and an ASCII code point that is not
    // an ident code point: it is a delim.
    assert_eq!(
        lexer_shapes("a\u{7f}b"),
        ["Ident:a", delim('\u{7f}').as_str(), "Ident:b"]
    );
}

// ---------------------------------------------------------------- the document

fn document_shapes(text: &str) -> Vec<String> {
    let document: Document<'_> = parse(text);
    document
        .list(document.root_list())
        .iter()
        .map(|&node| match document.component(node) {
            Component::Token { token, .. } => shape_of(token),
            Component::Function { .. } | Component::Block { .. } => {
                format!("Group:{}", shape_of(document.token(node)))
            }
            _ => "Other".to_owned(),
        })
        .collect()
}

#[test]
fn a_document_holds_the_tokens_the_lexer_reads_across_every_range_edge() {
    // A function and an unclosed group are components of their own; the cases
    // whose tokens close a group are compared by their flat reading instead.
    let mut failures = Vec::new();
    for point in edge_points() {
        for (text, expected) in cases(point) {
            if expected
                .iter()
                .any(|shape| shape.starts_with("Function:") || shape.ends_with("Paren"))
            {
                continue;
            }
            let found = document_shapes(&text);
            if found != expected {
                failures.push(format!(
                    "U+{:04X} in {text:?}: expected {expected:?}, found {found:?}",
                    u32::from(point)
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn a_document_nests_a_function_whose_name_holds_an_admitted_code_point() {
    for point in edge_points() {
        let text = format!("a{point}b(x)");
        let document = parse(&text);
        let root = document.list(document.root_list());
        if admitted(point) {
            assert_eq!(root.len(), 1, "U+{:04X}", u32::from(point));
            assert!(matches!(
                document.component(root[0]),
                Component::Function { .. }
            ));
            assert_eq!(
                shape_of(document.token(root[0])),
                format!("Function:a{point}b")
            );
        } else {
            // `a`, the delim, then `b(x)` as a function.
            assert_eq!(root.len(), 3, "U+{:04X}", u32::from(point));
            assert!(matches!(
                document.component(root[2]),
                Component::Function { .. }
            ));
        }
        // A name that is only the code point: a function when it is admitted,
        // and otherwise a delim followed by a parenthesis block.
        let text = format!("{point}(x)");
        let document = parse(&text);
        let root = document.list(document.root_list());
        if admitted(point) {
            assert_eq!(root.len(), 1, "U+{:04X}", u32::from(point));
            assert!(matches!(
                document.component(root[0]),
                Component::Function { .. }
            ));
        } else {
            assert_eq!(root.len(), 2, "U+{:04X}", u32::from(point));
            assert!(matches!(
                document.component(root[1]),
                Component::Block { .. }
            ));
        }
    }
}

// ----------------------------------------------------------- the protocol

/// Preprocesses UTF-8 `bytes` the way a metered host must before tokenizing.
fn preprocess(bytes: &[u8]) -> Vec<Scalar> {
    let units: Vec<u16> = bytes.iter().map(|&byte| u16::from(byte)).collect();
    let mut cursor = InputCursor::new(InputKind::Utf8, units.len());
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

fn protocol_shapes(text: &str) -> Vec<String> {
    let scalars = preprocess(text.as_bytes());
    let mut cursor = TokenCursor::new(scalars.len(), 0);
    let mut records = Vec::new();
    loop {
        match cursor.request() {
            TokenRequest::ReadScalar { index } => {
                cursor.read_scalar(index, scalars[index]).unwrap();
            }
            TokenRequest::Compute => cursor.compute().unwrap(),
            TokenRequest::Append { index, record } => {
                assert_eq!(index, records.len());
                records.push(record);
                cursor.appended(index).unwrap();
            }
            TokenRequest::Done => break,
        }
    }
    let name = |span: surgeist_css_parser::lexer::protocol::Span| -> String {
        records[span.start..span.end]
            .iter()
            .map(|record| match record {
                Record::NameUnit(unit) => *unit,
                other => panic!("expected a name unit, found {other:?}"),
            })
            .collect()
    };
    records
        .iter()
        .filter_map(|record| {
            let Record::Token(token) = record else {
                return None;
            };
            Some(match token.kind {
                RawTokenKind::Name { kind, units } => match kind {
                    NameKind::Ident => format!("Ident:{}", name(units)),
                    NameKind::Function => format!("Function:{}", name(units)),
                    NameKind::AtKeyword => format!("AtKeyword:{}", name(units)),
                    NameKind::Hash { identifier: true } => format!("HashId:{}", name(units)),
                    NameKind::Hash { identifier: false } => {
                        format!("HashUnrestricted:{}", name(units))
                    }
                },
                RawTokenKind::Number {
                    kind: NumericTokenKind::Dimension,
                    unit,
                    ..
                } => format!("Dimension:{}", name(unit)),
                RawTokenKind::Number {
                    kind: NumericTokenKind::Number,
                    ..
                } => "Number".to_owned(),
                RawTokenKind::Url { units, .. } => format!("Url:{}", name(units)),
                RawTokenKind::Delim(point) => delim(point),
                RawTokenKind::Whitespace => "Whitespace".to_owned(),
                RawTokenKind::Comment { .. } => "Comment".to_owned(),
                other => format!("{other:?}"),
            })
        })
        .collect()
}

#[test]
fn the_stepwise_tokenizer_reads_the_same_tokens_across_every_range_edge() {
    // `(` and `)` are tokens of their own in the lexer and in this
    // notation: the protocol reports them as `{:?}` of their kind, so the
    // cases that hold one are compared on the tokens that precede it.
    let mut failures = Vec::new();
    for point in edge_points() {
        for (text, expected) in cases(point) {
            if expected
                .iter()
                .any(|shape| shape == "OpenParen" || shape == "CloseParen")
            {
                continue;
            }
            let found = protocol_shapes(&text);
            if found != expected {
                failures.push(format!(
                    "U+{:04X} in {text:?}: expected {expected:?}, found {found:?}",
                    u32::from(point)
                ));
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn the_stepwise_tokenizer_reads_the_lexers_tokens_for_every_scalar_value() {
    for code in 0x80_u32..=0x10_ffff {
        let Some(point) = char::from_u32(code) else {
            continue;
        };
        let text = format!("a{point}b");
        assert_eq!(protocol_shapes(&text), lexer_shapes(&text), "U+{code:04X}");
    }
}

// -------------------------------------------------------------- serialization

fn identifier(value: &str) -> String {
    let mut out = String::new();
    serialize::identifier(value, &mut out).unwrap();
    out
}

/// The value of the one ident token of `text`, when it is exactly one.
fn only_ident(text: &str) -> Option<String> {
    let mut lexer = Lexer::new(text, &Options::default());
    let first = lexer.next_lexeme()?;
    let Token::Ident(value) = first.token() else {
        return None;
    };
    let value = value.to_owned();
    (first.range().end.value() == text.len() && lexer.next_lexeme().is_none()).then_some(value)
}

#[test]
fn an_identifier_escapes_a_code_point_exactly_when_section_4_2_excludes_it() {
    // Section 9: the output reads back as the same identifier. An admitted
    // code point is written as itself; an excluded one only as an escape.
    for point in edge_points() {
        for value in [
            format!("a{point}b"),
            format!("{point}b"),
            format!("-{point}b"),
            format!("a{point}"),
        ] {
            let written = identifier(&value);
            assert_eq!(
                only_ident(&written).as_deref(),
                Some(value.as_str()),
                "{value:?} written as {written:?}"
            );
            if admitted(point) {
                assert_eq!(written, value, "U+{:04X}", u32::from(point));
            } else {
                assert!(
                    !written.contains(point),
                    "U+{:04X} written raw in {written:?}",
                    u32::from(point)
                );
                assert!(
                    written.contains(&format!("\\{:x} ", u32::from(point))),
                    "U+{:04X} not escaped in {written:?}",
                    u32::from(point)
                );
            }
        }
    }
}

#[test]
fn an_identifier_of_any_scalar_value_reads_back_as_itself() {
    // P4 over all of Unicode, the excluded ranges included.
    for code in 0x80_u32..=0x10_ffff {
        let Some(point) = char::from_u32(code) else {
            continue;
        };
        for value in [format!("a{point}"), format!("{point}")] {
            let written = identifier(&value);
            assert_eq!(
                only_ident(&written).as_deref(),
                Some(value.as_str()),
                "U+{code:04X}: {value:?} written as {written:?}"
            );
        }
    }
}

#[test]
fn the_named_code_points_are_written_as_escapes() {
    // CSSOM 2.1 would write these raw; the round trip of section 9 does not.
    for (point, written) in [
        ('\u{d7}', "\\d7 "),
        ('\u{a0}', "\\a0 "),
        ('\u{2603}', "\\2603 "),
        ('\u{fffe}', "\\fffe "),
        ('\u{80}', "\\80 "),
        ('\u{2028}', "\\2028 "),
        ('\u{e000}', "\\e000 "),
    ] {
        assert_eq!(identifier(&point.to_string()), written, "{point:?}");
        assert_eq!(
            identifier(&format!("a{point}b")),
            format!("a{written}b"),
            "{point:?}"
        );
    }
    // Admitted code points stay as they are.
    for point in ['\u{b7}', '\u{e9}', '\u{200c}', '\u{fffd}', '\u{1f600}'] {
        assert_eq!(identifier(&point.to_string()), point.to_string());
    }
}

#[test]
fn a_token_with_an_excluded_code_point_in_its_name_is_written_so_that_it_reads_back() {
    // Every name-bearing token: the value holds the code point (read through
    // an escape, which any code point may follow), is written back with its
    // Display and read again.
    for point in edge_points() {
        for template in [
            "a\\{c}b", "\\{c}b", "@\\{c}b", "#\\{c}b", "#1\\{c}b", "1\\{c}b", "a\\{c}b(",
        ] {
            let text = at(template, point);
            let mut lexer = Lexer::new(&text, &Options::default());
            let first = lexer.next_lexeme().expect("one token").token();
            let (token, written) = (format!("{first:?}"), first.to_string());
            assert!(lexer.next_lexeme().is_none(), "{text:?} is one token");
            let mut again = Lexer::new(&written, &Options::default());
            let reread = format!("{:?}", again.next_lexeme().expect("one token").token());
            assert!(
                again.next_lexeme().is_none(),
                "{text:?} written as {written:?} is one token"
            );
            assert_eq!(reread, token, "{text:?} written as {written:?}");
            if !admitted(point) {
                assert!(
                    !written.contains(point),
                    "U+{:04X} written raw in {written:?}",
                    u32::from(point)
                );
            }
        }
    }
}
