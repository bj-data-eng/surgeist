//! Tests for the serialization of CSS Syntax Module Level 3 section 9: the
//! round-trip requirement, the note that lists what it takes (the backslash
//! delim, the hash flag, the unit of a dimension, the table of tokens that need
//! a comment between them), and the escaping of identifiers, strings and urls
//! that the requirement forces.
//!
//! The expectations come from the text of section 9 and from the tokenizer's
//! own rules (4.3), never from the serializer: the table is transcribed from the
//! edition as a grid in this file, and a serialization is judged by reading it
//! with the lexer and comparing the tokens.

mod common;

use common::render;
use surgeist_css_parser::document::TreeBuilder;
use surgeist_css_parser::lexer::{Lexer, Options};
use surgeist_css_parser::limits::{DepthExcess, Limits};
use surgeist_css_parser::serialize::{self, Adjacency, Writer, needs_separator};
use surgeist_css_parser::token::HashKind;
use surgeist_css_parser::{Document, Token};

/// The rendered tokens of `text`, comments and whitespace left out: the data
/// structure section 9 says must survive a round trip.
fn tokens(text: &str) -> Vec<String> {
    let mut lexer = Lexer::new(text, &Options::default());
    let mut out = Vec::new();
    while let Some(lexeme) = lexer.next_lexeme() {
        if !lexeme.token().is_trivia() {
            out.push(render(lexeme.token()));
        }
    }
    out
}

/// The tokens of `text` with whitespace kept (consecutive whitespace is one
/// token already) and comments left out.
fn tokens_with_whitespace(text: &str) -> Vec<String> {
    let mut lexer = Lexer::new(text, &Options::default());
    let mut out = Vec::new();
    while let Some(lexeme) = lexer.next_lexeme() {
        if lexeme.token() != Token::Comment {
            out.push(render(lexeme.token()));
        }
    }
    out
}

fn escaped_identifier(value: &str) -> String {
    let mut out = String::new();
    serialize::identifier(value, &mut out).unwrap();
    out
}

fn quoted(value: &str) -> String {
    let mut out = String::new();
    serialize::string(value, &mut out).unwrap();
    out
}

fn url(value: &str) -> String {
    let mut out = String::new();
    serialize::url(value, &mut out).unwrap();
    out
}

// ---------------------------------------------------------------- identifiers

#[test]
fn an_identifier_is_written_with_the_escapes_that_read_back_as_itself() {
    for (value, written) in [
        ("", ""),
        ("a", "a"),
        ("A_b-9", "A_b-9"),
        ("a1", "a1"),
        // A digit cannot start an identifier, nor follow a leading hyphen-minus.
        ("1", "\\31 "),
        ("123", "\\31 23"),
        ("-1", "-\\31 "),
        ("-12", "-\\31 2"),
        ("--1", "--1"),
        ("-a", "-a"),
        // A lone hyphen-minus is not an identifier unescaped.
        ("-", "\\-"),
        ("--", "--"),
        // NUL is the replacement character; control code points are hexadecimal
        // escapes with their terminating space.
        ("\0", "\u{fffd}"),
        ("a\u{1}b", "a\\1 b"),
        ("\u{1f}", "\\1f "),
        ("\u{7f}", "\\7f "),
        ("a\tb", "a\\9 b"),
        ("a\nb", "a\\a b"),
        // The remaining ASCII code points are escaped behind a backslash.
        (" ", "\\ "),
        ("a b", "a\\ b"),
        ("!", "\\!"),
        ("a.b", "a\\.b"),
        ("a:b", "a\\:b"),
        ("#", "\\#"),
        ("\\", "\\\\"),
        ("a(b)", "a\\(b\\)"),
        ("\"'", "\\\"\\'"),
    ] {
        assert_eq!(escaped_identifier(value), written, "{value:?}");
    }
}

#[test]
fn a_non_ascii_code_point_is_written_as_itself_exactly_when_it_is_an_ident_code_point() {
    // The round-trip rule: what the tokenizer reads as part of a name is
    // written as it is, and everything else is escaped. The predicate is the
    // tokenizer's, so the answer follows it.
    for point in [
        '\u{b7}',
        '\u{c0}',
        '\u{d7}',
        '\u{e9}',
        '\u{a0}',
        '\u{2603}',
        '\u{200c}',
        '\u{fffd}',
        '\u{fffe}',
        '\u{ffff}',
        '\u{10000}',
        '\u{1f600}',
        '\u{10ffff}',
        '\u{80}',
        '\u{9f}',
    ] {
        let value = format!("a{point}b");
        let written = escaped_identifier(&value);
        if surgeist_css_parser::lexical::is_ident_code_point(point) {
            assert_eq!(written, value, "U+{:04X}", u32::from(point));
        } else {
            assert_eq!(
                written,
                format!("a\\{:x} b", u32::from(point)),
                "U+{:04X}",
                u32::from(point)
            );
        }
    }
}

/// The value of the one ident token of `text`, if it is exactly one.
fn only_ident(text: &str) -> Option<String> {
    let mut lexer = Lexer::new(text, &Options::default());
    let first = lexer.next_lexeme()?;
    let Token::Ident(value) = first.token() else {
        return None;
    };
    let value = value.to_owned();
    if first.range().end.value() != text.len() {
        return None;
    }
    lexer.next_lexeme().is_none().then_some(value)
}

#[test]
fn every_ascii_code_point_in_every_position_of_an_identifier_reads_back_as_the_same_identifier() {
    for code in 1_u8..=0x7f {
        let point = char::from(code);
        for value in [
            format!("{point}"),
            format!("a{point}"),
            format!("{point}a"),
            format!("a{point}a"),
            format!("-{point}"),
            format!("-{point}a"),
            format!("--{point}"),
            format!("{point}{point}"),
        ] {
            let written = escaped_identifier(&value);
            assert_eq!(
                only_ident(&written).as_deref(),
                Some(value.as_str()),
                "{value:?} written as {written:?}"
            );
        }
    }
}

#[test]
fn every_non_ascii_code_point_in_an_identifier_reads_back_as_the_same_identifier() {
    let mut checked = 0_u32;
    for code in (0x80_u32..=0xffff).chain((0x10000..=0x10ffff).step_by(13)) {
        let Some(point) = char::from_u32(code) else {
            continue;
        };
        for value in [format!("{point}"), format!("a{point}"), format!("-{point}")] {
            let written = escaped_identifier(&value);
            assert_eq!(
                only_ident(&written).as_deref(),
                Some(value.as_str()),
                "{value:?} written as {written:?}"
            );
            checked += 1;
        }
    }
    assert!(checked > 190_000, "{checked}");
}

// ---------------------------------------------------------------- strings

#[test]
fn a_string_is_written_in_double_quotes_with_the_escapes_that_read_back_as_itself() {
    for (value, written) in [
        ("", "\"\""),
        ("abc", "\"abc\""),
        ("a\"b", "\"a\\\"b\""),
        ("a\\b", "\"a\\\\b\""),
        ("a'b", "\"a'b\""),
        ("a\nb", "\"a\\a b\""),
        ("a\tb", "\"a\\9 b\""),
        ("\u{1}", "\"\\1 \""),
        ("\u{7f}", "\"\\7f \""),
        ("\0", "\"\u{fffd}\""),
        ("\u{e9}\u{2603}", "\"\u{e9}\u{2603}\""),
    ] {
        assert_eq!(quoted(value), written, "{value:?}");
    }
}

#[test]
fn every_ascii_code_point_in_a_string_reads_back_as_the_same_string() {
    for code in 1_u8..=0x7f {
        let value = format!("a{}b", char::from(code));
        let written = quoted(&value);
        let mut lexer = Lexer::new(&written, &Options::default());
        let lexeme = lexer.next_lexeme().unwrap();
        assert_eq!(lexeme.token(), Token::String(&value), "{written:?}");
        assert_eq!(lexeme.range().end.value(), written.len());
        assert!(lexer.next_lexeme().is_none());
    }
}

// ---------------------------------------------------------------- urls

#[test]
fn a_url_is_written_unquoted_with_the_escapes_that_read_back_as_itself() {
    for (value, written) in [
        ("", "url()"),
        ("a", "url(a)"),
        ("a b", "url(a\\20 b)"),
        ("a\"b", "url(a\\\"b)"),
        ("a'b", "url(a\\'b)"),
        ("a(b)", "url(a\\(b\\))"),
        ("a\\b", "url(a\\\\b)"),
        ("a\nb", "url(a\\a b)"),
        ("\u{1}", "url(\\1 )"),
        ("\u{7f}", "url(\\7f )"),
        ("\0", "url(\u{fffd})"),
        ("http://x/y?z=1#f", "url(http://x/y?z=1#f)"),
    ] {
        assert_eq!(url(value), written, "{value:?}");
    }
}

#[test]
fn every_ascii_code_point_in_a_url_reads_back_as_the_same_url() {
    for code in 1_u8..=0x7f {
        for value in [
            format!("a{}b", char::from(code)),
            format!("{}", char::from(code)),
        ] {
            let written = url(&value);
            let mut lexer = Lexer::new(&written, &Options::default());
            let lexeme = lexer.next_lexeme().unwrap();
            assert_eq!(lexeme.token(), Token::Url(&value), "{written:?}");
            assert_eq!(lexeme.range().end.value(), written.len());
            assert!(lexer.next_lexeme().is_none());
        }
    }
}

// ---------------------------------------------------------------- one token

/// The canonical serialization of the one token of `text`.
fn written(text: &str) -> String {
    let mut lexer = Lexer::new(text, &Options::default());
    let token = lexer.next_lexeme().unwrap().token();
    token.to_string()
}

#[test]
fn a_token_is_written_from_its_value() {
    for (text, expected) in [
        ("a", "a"),
        ("\\61 b", "ab"),
        ("\"x\"", "\"x\""),
        ("'x'", "\"x\""),
        ("url( a )", "url(a)"),
        ("url(\"a\"", "url("), // the function token of a quoted url
        ("1.50", "1.50"),
        ("+.5E-3", "+.5E-3"),
        ("10px", "10px"),
        ("10PX", "10PX"),
        ("50%", "50%"),
        ("#id", "#id"),
        ("@media", "@media"),
        ("f(", "f("),
        (";", ";"),
        (":", ":"),
        (",", ","),
        ("[", "["),
        ("(", "("),
        ("{", "{"),
        ("]", "]"),
        (")", ")"),
        ("}", "}"),
        ("<!--", "<!--"),
        ("-->", "-->"),
        ("~", "~"),
        (" \n\t ", " "),
    ] {
        assert_eq!(written(text), expected, "{text:?}");
    }
}

#[test]
fn a_hash_with_the_id_flag_needs_more_escaping_than_an_unrestricted_one() {
    // Section 9's second note. `#1a` is unrestricted: its value would not start
    // an identifier, and is written as it is. `#\31 a` has the same value with
    // the id flag (the escape is a valid escape, so it would start one), and
    // keeps the escape.
    let mut lexer = Lexer::new("#1a #\\31 a", &Options::default());
    let unrestricted = lexer.next_lexeme().unwrap().token();
    assert_eq!(
        unrestricted,
        Token::Hash {
            value: "1a",
            kind: HashKind::Unrestricted
        }
    );
    assert_eq!(unrestricted.to_string(), "#1a");
    let _ = lexer.next_lexeme(); // the whitespace
    let id = lexer.next_lexeme().unwrap().token();
    assert_eq!(
        id,
        Token::Hash {
            value: "1a",
            kind: HashKind::Id
        }
    );
    assert_eq!(id.to_string(), "#\\31 a");
}

/// Writes every code point of `unit` as a hexadecimal escape, so the text is a
/// valid unit whatever its first code point.
fn hex_all(unit: &str) -> String {
    unit.chars()
        .map(|point| format!("\\{:x} ", u32::from(point)))
        .collect()
}

#[test]
fn the_unit_of_a_dimension_is_escaped_where_it_would_read_as_an_exponent() {
    // Section 9's third note, and 4.3.13: after a number, `e` or `E`, an
    // optional sign and a digit are part of the number.
    for (text, expected) in [
        ("1\\65 3", "1\\65 3"),   // unit `e3`
        ("1\\45 3", "1\\45 3"),   // unit `E3`
        ("1\\65 -3", "1\\65 -3"), // unit `e-3`
        ("1\\65 x", "1ex"),       // unit `ex`: no exponent, so no escape
        ("1e\\+5", "1e\\+5"),     // unit `e+5`: the plus is escaped already
        ("1\\65 -x", "1e-x"),     // unit `e-x`
        ("1\\31 ", "1\\31 "),     // a unit that starts with a digit
        ("1\\2d ", "1\\-"),       // the unit `-`
        ("1\\2d 1", "1-\\31 "),   // the unit `-1`
    ] {
        assert_eq!(written(text), expected, "{text:?}");
    }
}

#[test]
fn a_dimension_reads_back_as_itself_whatever_its_unit() {
    for unit in [
        "e3", "E3", "e-3", "e-30", "ex", "e+5", "e-x", "e", "E", "1", "-", "--", "-1", "--1",
        "\u{e9}", "x1", "px", "e\u{1}", "e-", "e5e5",
    ] {
        let source = format!("1{}", hex_all(unit));
        let mut lexer = Lexer::new(&source, &Options::default());
        let lexeme = lexer.next_lexeme().unwrap();
        let Token::Dimension { unit: read, .. } = lexeme.token() else {
            panic!("{source:?} is a dimension");
        };
        assert_eq!(read, unit, "{source:?}");
        let text = lexeme.token().to_string();
        assert_eq!(
            tokens(&text),
            tokens(&source),
            "{unit:?} written as {text:?}"
        );
        assert_eq!(tokens(&text).len(), 1, "{unit:?} written as {text:?}");
    }
}

#[test]
fn a_delim_of_a_backslash_is_a_backslash_and_a_newline() {
    // Section 9's first note. The tokenizer emits the delim only before a
    // newline that starts a whitespace token.
    assert_eq!(Token::Delim('\\').to_string(), "\\\n");
    assert_eq!(
        tokens_with_whitespace("\\\n"),
        ["Delim('\\\\')", "Whitespace"]
    );
}

#[test]
fn a_bad_string_and_a_bad_url_are_written_as_text_that_reads_as_one_again() {
    assert_eq!(
        tokens_with_whitespace(&Token::BadString.to_string()),
        ["BadString", "Whitespace"]
    );
    assert_eq!(tokens(&Token::BadUrl.to_string()), ["BadUrl"]);
    // And they carry no value to lose: the lexer's own are the same tokens.
    assert_eq!(tokens("\"a\n"), ["BadString"]);
    assert_eq!(tokens("url(a b)"), ["BadUrl"]);
}

#[test]
fn a_unicode_range_is_written_as_hexadecimal_bounds_and_reads_back_when_allowed() {
    let options = Options::default().unicode_ranges(true);
    for text in [
        "U+26",
        "U+400-4FF",
        "U+0-10FFFF",
        "U+110000-110001",
        "U+20-10",
        "U+4??",
    ] {
        let mut lexer = Lexer::new(text, &options);
        let token = lexer.next_lexeme().unwrap().token();
        assert!(matches!(token, Token::UnicodeRange(_)), "{text}");
        let written = token.to_string();
        let mut again = Lexer::new(&written, &options);
        let reread = again.next_lexeme().unwrap().token();
        assert_eq!(reread, token, "{text} written as {written}");
        assert!(again.next_lexeme().is_none());
    }
    assert_eq!(
        Token::UnicodeRange(surgeist_css_parser::unicode_range::CodePointRange::new(
            0x26, 0x26
        ))
        .to_string(),
        "U+26"
    );
    assert_eq!(
        Token::UnicodeRange(surgeist_css_parser::unicode_range::CodePointRange::new(
            0x400, 0x4ff
        ))
        .to_string(),
        "U+400-4FF"
    );
}

// ---------------------------------------------------------------- section 9's table

/// The table of section 9, transcribed as a grid: the rows are the first
/// token, the columns the second, an `x` is a ✗ (a comment is required) and a
/// dot an empty cell.
const COLUMNS: [(&str, Adjacency, &str); 12] = [
    ("ident", Adjacency::Ident, "a"),
    ("function", Adjacency::Function, "f("),
    ("url", Adjacency::Url, "url(u)"),
    ("bad url", Adjacency::BadUrl, "url(a b)"),
    ("-", Adjacency::Delim('-'), "-"),
    ("number", Adjacency::Number, "1"),
    ("percentage", Adjacency::Percentage, "1%"),
    ("dimension", Adjacency::Dimension, "1px"),
    ("CDC", Adjacency::Cdc, "-->"),
    ("(", Adjacency::OpenParen, "("),
    ("*", Adjacency::Delim('*'), "*"),
    ("%", Adjacency::Delim('%'), "%"),
];

const ROWS: [(&str, Adjacency, &str, &str); 11] = [
    ("ident", Adjacency::Ident, "a", "xxxxxxxxxx.."),
    ("at-keyword", Adjacency::AtKeyword, "@k", "xxxxxxxxx..."),
    ("hash", Adjacency::Hash, "#h", "xxxxxxxxx..."),
    ("dimension", Adjacency::Dimension, "1px", "xxxxxxxxx..."),
    ("#", Adjacency::Delim('#'), "#", "xxxxxxxxx..."),
    ("-", Adjacency::Delim('-'), "-", "xxxxxxxxx..."),
    ("number", Adjacency::Number, "1", "xxxx.xxxx..x"),
    ("@", Adjacency::Delim('@'), "@", "xxxxx...x..."),
    (".", Adjacency::Delim('.'), ".", ".....xxx...."),
    ("+", Adjacency::Delim('+'), "+", ".....xxx...."),
    ("/", Adjacency::Delim('/'), "/", "..........x."),
];

#[test]
fn needs_separator_is_the_table_of_section_9_cell_by_cell() {
    for (row_name, row, _, cells) in ROWS {
        assert_eq!(cells.len(), COLUMNS.len(), "{row_name}");
        for ((column_name, column, _), cell) in COLUMNS.into_iter().zip(cells.chars()) {
            assert_eq!(
                needs_separator(row, column),
                cell == 'x',
                "{row_name} then {column_name}"
            );
        }
    }
}

#[test]
fn a_pair_the_table_does_not_name_never_needs_a_comment() {
    let universe = [
        Adjacency::Ident,
        Adjacency::AtKeyword,
        Adjacency::Hash,
        Adjacency::Dimension,
        Adjacency::Number,
        Adjacency::Function,
        Adjacency::Url,
        Adjacency::BadUrl,
        Adjacency::Percentage,
        Adjacency::Cdc,
        Adjacency::OpenParen,
        Adjacency::Other,
        Adjacency::Delim('#'),
        Adjacency::Delim('-'),
        Adjacency::Delim('@'),
        Adjacency::Delim('.'),
        Adjacency::Delim('+'),
        Adjacency::Delim('/'),
        Adjacency::Delim('*'),
        Adjacency::Delim('%'),
        Adjacency::Delim('<'),
        Adjacency::Delim('!'),
        Adjacency::Delim('\\'),
        Adjacency::Delim('='),
    ];
    let row_of = |adjacency: Adjacency| ROWS.iter().position(|row| row.1 == adjacency);
    let column_of = |adjacency: Adjacency| COLUMNS.iter().position(|column| column.1 == adjacency);
    for first in universe {
        for second in universe {
            let expected = match (row_of(first), column_of(second)) {
                (Some(row), Some(column)) => ROWS[row].3.as_bytes()[column] == b'x',
                _ => false,
            };
            assert_eq!(
                needs_separator(first, second),
                expected,
                "{first:?} then {second:?}"
            );
        }
    }
}

#[test]
fn where_the_table_is_empty_the_tokens_run_together_as_the_same_two_tokens() {
    // A blank cell is a pair whose text, written with nothing between, reads as
    // the two tokens (for the representative tokens of the row and the column).
    for (row_name, _, row_text, cells) in ROWS {
        for ((column_name, _, column_text), cell) in COLUMNS.into_iter().zip(cells.chars()) {
            if cell == 'x' {
                continue;
            }
            let glued = format!("{row_text}{column_text}");
            let mut expected = tokens(row_text);
            expected.extend(tokens(column_text));
            assert_eq!(
                tokens(&glued),
                expected,
                "{row_name} then {column_name}: {glued:?}"
            );
        }
    }
}

#[test]
fn where_the_table_has_a_cross_the_tokens_run_together_as_other_tokens_and_a_comment_separates_them()
 {
    // Not every cross is a pair that merges for every token (`@` and `-` merge
    // only before an identifier), so the claim is checked in both directions
    // where it holds: the writer puts a comment in every cross, the output
    // reads back as the two tokens, and for the pairs below the glued text does
    // not.
    let merging = [
        ("ident", "a", "ident", "a"),
        ("ident", "a", "function", "f("),
        ("ident", "a", "url", "url(u)"),
        ("ident", "a", "bad url", "url(a b)"),
        ("ident", "a", "-", "-"),
        ("ident", "a", "number", "1"),
        ("ident", "a", "percentage", "1%"),
        ("ident", "a", "dimension", "1px"),
        ("ident", "a", "CDC", "-->"),
        ("ident", "a", "(", "("),
        ("at-keyword", "@k", "ident", "a"),
        ("at-keyword", "@k", "-", "-"),
        ("hash", "#h", "number", "1"),
        ("hash", "#h", "CDC", "-->"),
        ("dimension", "1px", "ident", "a"),
        ("dimension", "1px", "number", "1"),
        ("dimension", "1px", "-", "-"),
        ("#", "#", "ident", "a"),
        ("#", "#", "number", "1"),
        ("#", "#", "-", "-"),
        ("-", "-", "ident", "a"),
        ("-", "-", "-", "-"),
        ("-", "-", "number", "1"),
        ("-", "-", "function", "f("),
        ("-", "-", "CDC", "-->"),
        ("number", "1", "ident", "a"),
        ("number", "1", "number", "1"),
        ("number", "1", "percentage", "1%"),
        ("number", "1", "dimension", "1px"),
        ("number", "1", "%", "%"),
        ("number", "1", "CDC", "-->"),
        ("@", "@", "ident", "a"),
        ("@", "@", "function", "f("),
        ("@", "@", "CDC", "-->"),
        (".", ".", "number", "1"),
        (".", ".", "percentage", "1%"),
        (".", ".", "dimension", "1px"),
        ("+", "+", "number", "1"),
        ("+", "+", "percentage", "1%"),
        ("+", "+", "dimension", "1px"),
        ("/", "/", "*", "*"),
    ];
    for (row_name, row_text, column_name, column_text) in merging {
        let glued = format!("{row_text}{column_text}");
        let mut apart = tokens(row_text);
        apart.extend(tokens(column_text));
        assert_ne!(
            tokens(&glued),
            apart,
            "{row_name} then {column_name}: {glued:?}"
        );
        // The table asks for a comment, and one separates them.
        let row = ROWS.iter().find(|row| row.0 == row_name).unwrap().1;
        let column = COLUMNS
            .iter()
            .find(|column| column.0 == column_name)
            .unwrap()
            .1;
        assert!(
            needs_separator(row, column),
            "{row_name} then {column_name}"
        );
        let with_comment = format!("{row_text}/**/{column_text}");
        assert_eq!(tokens(&with_comment), apart, "{with_comment:?}");
    }
}

#[test]
fn the_writer_puts_a_comment_in_every_cross_of_the_table_and_nowhere_else() {
    for (row_name, row, row_text, cells) in ROWS {
        for ((column_name, column, column_text), cell) in COLUMNS.into_iter().zip(cells.chars()) {
            let mut first = Lexer::new(row_text, &Options::default());
            let first = first.next_lexeme().unwrap().token();
            let mut second = Lexer::new(column_text, &Options::default());
            let second = second.next_lexeme().unwrap().token();
            assert_eq!(Adjacency::from(&first), row, "{row_name}");
            assert_eq!(Adjacency::from(&second), column, "{column_name}");
            let mut writer = Writer::new(String::new());
            writer.token(first).unwrap();
            writer.token(second).unwrap();
            let out = writer.into_inner();
            assert_eq!(
                out.contains("/**/"),
                cell == 'x',
                "{row_name} then {column_name}: {out:?}"
            );
            let mut apart = tokens(row_text);
            apart.extend(tokens(column_text));
            assert_eq!(
                tokens(&out),
                apart,
                "{row_name} then {column_name}: {out:?}"
            );
        }
    }
}

#[test]
fn the_unicode_range_token_is_read_like_an_ident_on_both_sides() {
    // The table has no unicode-range token; with the lexer's option on, one
    // runs into the tokens a name does, and a question mark.
    let options = Options::default().unicode_ranges(true);
    let ranged = |text: &str| -> Vec<String> {
        let mut lexer = Lexer::new(text, &options);
        let mut out = Vec::new();
        while let Some(lexeme) = lexer.next_lexeme() {
            if lexeme.token() != Token::Comment {
                out.push(render(lexeme.token()));
            }
        }
        out
    };
    for (before, after) in [
        ("U+26", "a"),
        ("U+26", "1"),
        ("U+26", "1px"),
        ("U+26", "1%"),
        ("U+26", "?"),
        ("a", "U+26"),
        ("1", "U+26"),
        ("#", "U+26"),
        ("@", "U+26"),
        ("-", "U+26"),
        ("@k", "U+26"),
    ] {
        let glued = format!("{before}{after}");
        let mut apart = ranged(before);
        apart.extend(ranged(after));
        assert_ne!(ranged(&glued), apart, "{before} then {after}");
        let mut writer = Writer::new(String::new());
        for text in [before, after] {
            let mut lexer = Lexer::new(text, &options);
            writer.token(lexer.next_lexeme().unwrap().token()).unwrap();
        }
        let out = writer.into_inner();
        assert!(out.contains("/**/"), "{before} then {after}: {out:?}");
        assert_eq!(ranged(&out), apart, "{before} then {after}: {out:?}");
    }
    assert!(needs_separator(Adjacency::UnicodeRange, Adjacency::Ident));
    assert!(needs_separator(Adjacency::Ident, Adjacency::UnicodeRange));
    assert!(!needs_separator(
        Adjacency::UnicodeRange,
        Adjacency::Delim('*')
    ));
}

// ---------------------------------------------------------------- the writer

fn write(tokens: &[Token<'_>]) -> String {
    let mut writer = Writer::new(String::new());
    for token in tokens {
        writer.token(*token).unwrap();
    }
    writer.into_inner()
}

#[test]
fn the_writer_collapses_whitespace_and_a_comment_separates_what_it_stands_between() {
    use Token::{Comment, Delim, Ident, Whitespace};
    assert_eq!(write(&[Ident("a"), Ident("b")]), "a/**/b");
    assert_eq!(write(&[Ident("a"), Whitespace, Ident("b")]), "a b");
    assert_eq!(
        write(&[Ident("a"), Whitespace, Whitespace, Ident("b")]),
        "a b"
    );
    // A comment already separates the two; the writer adds no second.
    assert_eq!(write(&[Ident("a"), Comment, Ident("b")]), "a/**/b");
    assert_eq!(write(&[Whitespace, Whitespace]), " ");
    assert_eq!(write(&[Delim('/'), Whitespace, Delim('*')]), "/ *");
    assert_eq!(write(&[Delim('/'), Delim('*')]), "//**/*");
}

#[test]
fn a_less_than_a_bang_and_two_hyphens_would_read_as_a_cdo_and_are_separated() {
    use Token::{Cdc, Delim, Ident, Whitespace};
    // Section 4.3.1 reads `<!--` as a CDO-token. The table does not list the
    // pair; the round-trip requirement does.
    for third in [
        Ident("--x"),
        Ident("--"),
        Cdc,
        Token::Function("--f"),
        Token::Function("--"),
        Token::Function("---f"),
    ] {
        let out = write(&[Delim('<'), Delim('!'), third]);
        assert!(out.starts_with("<!/**/"), "{out:?}");
        assert_eq!(tokens(&out).len(), 3, "{out:?}");
    }
    // One hyphen-minus, or something between, does not.
    assert_eq!(write(&[Delim('<'), Delim('!'), Ident("-x")]), "<!-x");
    assert_eq!(write(&[Delim('<'), Delim('!'), Delim('-')]), "<!-");
    assert_eq!(
        write(&[Delim('<'), Whitespace, Delim('!'), Ident("--x")]),
        "< !--x"
    );
    assert_eq!(write(&[Delim('<'), Ident("--x")]), "<--x");
    assert_eq!(tokens("<!/**/--x").len(), 3);
}

#[test]
fn the_ident_of_two_hyphens_followed_by_a_greater_than_would_read_as_a_cdc_and_is_separated() {
    use Token::{Delim, Ident};
    // `\-->` is the ident `--` and a `>`; written plainly it is a CDC-token
    // (section 4.3.1 reads `-->` before an ident).
    assert_eq!(tokens("\\-->"), ["Ident(\"--\")", "Delim('>')"]);
    assert_eq!(tokens("-->"), ["Cdc"]);
    let out = write(&[Ident("--"), Delim('>')]);
    assert_eq!(out, "--/**/>");
    assert_eq!(tokens(&out), ["Ident(\"--\")", "Delim('>')"]);
    // A longer ident ends before the `>`; anything between also does.
    assert_eq!(write(&[Ident("--x"), Delim('>')]), "--x>");
    assert_eq!(write(&[Ident("--"), Token::Whitespace, Delim('>')]), "-- >");
    assert_eq!(write(&[Ident("--"), Delim('<')]), "--<");
}

#[test]
fn a_backslash_delim_and_a_bad_string_bring_their_newline_and_whitespace_follows_it() {
    use Token::{BadString, Delim, Whitespace};
    let out = write(&[Delim('\\'), Whitespace]);
    assert_eq!(out, "\\\n ");
    assert_eq!(
        tokens_with_whitespace(&out),
        ["Delim('\\\\')", "Whitespace"]
    );
    let out = write(&[BadString, Whitespace]);
    assert_eq!(tokens_with_whitespace(&out), ["BadString", "Whitespace"]);
}

// ---------------------------------------------------------------- component values

fn list(text: &str) -> String {
    let document: Document = Document::parse(
        text,
        &Options::default(),
        Limits::default(),
        0,
        DepthExcess::Fail,
    )
    .unwrap();
    let mut out = String::new();
    serialize::list(&document, document.root_list(), &mut out).unwrap();
    out
}

#[test]
fn component_values_are_written_with_their_groups_and_closing_tokens() {
    assert_eq!(list("a { b : f( 1px ) } "), "a { b : f( 1px ) } ");
    assert_eq!(list("f(a) [b] (c) {d}"), "f(a) [b] (c) {d}");
    assert_eq!(list(""), "");
    assert_eq!(list("a\\ b"), "a\\ b"); // an escape is written the way the serializer escapes
    // The normal form: quotes, hexadecimal escapes, no comments.
    assert_eq!(list("'x' \\61 b"), "\"x\" ab");
    assert_eq!(list("a/*x*/b"), "a/**/b");
}

#[test]
fn a_group_the_input_ended_is_written_closed_and_a_stray_closing_token_stays() {
    assert_eq!(list("f(a"), "f(a)");
    assert_eq!(list("(a [b"), "(a [b])");
    assert_eq!(list("a } b"), "a } b");
    assert_eq!(list("(a } b)"), "(a } b)");
    assert_eq!(list(") ]"), ") ]");
}

#[test]
fn a_denied_block_is_written_as_the_text_it_covered() {
    let limits = Limits::new(1, usize::MAX, usize::MAX);
    let document: Document = Document::parse(
        "a{ b{ c } d{ e } } f",
        &Options::default(),
        limits,
        0,
        DepthExcess::DenyBlock,
    )
    .unwrap();
    let mut out = String::new();
    serialize::list(&document, document.root_list(), &mut out).unwrap();
    assert_eq!(out, "a{ b{ c } d{ e } } f");
}

#[test]
fn tokens_built_without_text_are_separated_where_the_table_asks() {
    let mut builder: TreeBuilder<'_, ()> = Document::builder(Limits::default(), 0, None).unwrap();
    let root = builder.root();
    for token in [Token::Ident("a"), Token::Ident("b")] {
        builder.leaf(root, token, None, None).unwrap();
    }
    let (function, inner) = builder
        .open(root, Token::Function("f"), None, None)
        .unwrap();
    builder.leaf(inner, Token::Ident("c"), None, None).unwrap();
    builder
        .close(
            function,
            surgeist_css_parser::document::GroupEnd::Closed {
                range: None,
                payload: None,
            },
        )
        .unwrap();
    let document = builder.finish(None);
    let mut out = String::new();
    serialize::list(&document, document.root_list(), &mut out).unwrap();
    assert_eq!(out, "a/**/b/**/f(c)");
}
