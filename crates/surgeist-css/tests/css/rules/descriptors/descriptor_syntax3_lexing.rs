//! The tokens a descriptor value is read as, following Syntax 3 (2026 CRD) §4.3.
//!
//! - §4.3.7 (consume an escaped code point) and §4.3.8 (check if two code
//!   points are a valid escape): a backslash followed by a newline is not a
//!   valid escape, so it never continues an identifier, hash, dimension unit or
//!   at-keyword; it is a delimiter token and a parse error at the backslash.
//! - §4.3.9 (check if three code points would start an ident sequence): `-`
//!   followed by `\` and a newline does not start one, so `-` is a delimiter
//!   token, `1-` is a number followed by a delimiter (not a dimension), `@-` is
//!   a delimiter followed by a delimiter (not an at-keyword), and `#-` is a hash
//!   token holding `-`.
//! - §4.3.1: `$=` is not a token; `$` and `=` are delimiter tokens.

use surgeist_css::CssParser;
use surgeist_css::diagnostic::CssErrorCode;

/// Each rule with a descriptor whose value is complete, so that the first
/// token after it is the first one its grammar rejects.
const RULES: [(&str, &str); 5] = [
    ("@font-palette-values --p", "base-palette: light"),
    ("@color-profile --c", "rendering-intent: perceptual"),
    ("@counter-style x", "system: cyclic"),
    ("@font-feature-values A", "font-display: swap"),
    ("@font-face", "font-display: swap"),
];

/// The shapes, each with the first token §4.3 reads in it.
const SHAPES: [(&str, &str, &str); 5] = [
    ("-\\\n", "Delim", "-"),
    ("-\\\r\n", "Delim", "-"),
    ("#-\\\n", "Hash", "#-"),
    ("1-\\\n", "Number", "1"),
    ("@-\\\n", "Delim", "@"),
];

#[test]
fn a_backslash_before_a_newline_never_continues_the_token_before_it() {
    for (head, declaration) in RULES {
        for (shape, kind, authored) in SHAPES {
            let source = format!("{head} {{ {declaration} {shape} }}");
            let report = CssParser::default().parse_sheet(&source);
            let shape_at = source.find(shape).unwrap();
            let backslash_at = source.find('\\').unwrap();
            let descriptor: Vec<_> = report
                .diagnostics()
                .iter()
                .filter(|diagnostic| {
                    diagnostic.error().code() == CssErrorCode::InvalidDescriptorValue
                })
                .collect();
            assert_eq!(descriptor.len(), 1, "{source:?}");
            let error = descriptor[0].error();
            assert_eq!(
                error.position().byte_offset().value(),
                shape_at,
                "{source:?}"
            );
            let token = error.encountered().expect("the rejected token");
            assert_eq!(format!("{:?}", token.kind()), kind, "{source:?}");
            assert_eq!(token.authored(), authored, "{source:?}");
            let escapes: Vec<_> = report
                .diagnostics()
                .iter()
                .filter(|diagnostic| diagnostic.error().code() == CssErrorCode::EscapeParseError)
                .map(|diagnostic| diagnostic.error().position().byte_offset().value())
                .collect();
            assert_eq!(escapes, [backslash_at], "{source:?}");
        }
    }
}

#[test]
fn a_dollar_sign_before_an_equals_sign_is_two_delimiters() {
    let source = "@font-palette-values --p { font-family: $= }";
    let report = CssParser::default().parse_sheet(source);
    let error = report
        .diagnostics()
        .iter()
        .find(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidDescriptorValue)
        .expect("the rejected descriptor")
        .error();
    assert_eq!(
        error.position().byte_offset().value(),
        source.find('$').unwrap()
    );
    let token = error.encountered().expect("the rejected token");
    assert_eq!(format!("{:?}", token.kind()), "Delim");
    assert_eq!(token.authored(), "$");
}

/// The two-character matchers of attribute selectors are not tokens: each is a
/// delimiter followed by `=` (§4.3.1), so a value that holds one is rejected at
/// its first delimiter and nothing aborts.
#[test]
fn a_matcher_in_a_font_face_value_is_two_delimiters() {
    for operator in ["$=", "*=", "^=", "|=", "~="] {
        for declaration in [
            "font-weight: OP",
            "font-style: OP",
            "font-display: swap OP",
            "src: url(x) OP",
            "src: local(a) OP",
            "font-family: a OP",
            "font-feature-settings: OP",
            "font-variation-settings: 'wght' 1 OP",
            "font-language-override: OP",
            "ascent-override: OP",
            "unicode-range: U+25 OP",
        ] {
            let declaration = declaration.replace("OP", operator);
            let source = format!("@font-face {{ {declaration} }}");
            let report = CssParser::default().parse_sheet(&source);
            let rejected: Vec<_> = report
                .diagnostics()
                .iter()
                .filter(|diagnostic| {
                    diagnostic.error().code() == CssErrorCode::InvalidDescriptorValue
                })
                .collect();
            assert_eq!(rejected.len(), 1, "{source:?}");
            let error = rejected[0].error();
            let first = source.find(operator).unwrap();
            let token = error.encountered().expect("the rejected token");
            assert_eq!(format!("{:?}", token.kind()), "Delim", "{source:?}");
            assert_eq!(token.authored(), &operator[..1], "{source:?}");
            assert_eq!(error.position().byte_offset().value(), first, "{source:?}");
        }
    }
}

/// The same holds for the descriptors of `@page`.
#[test]
fn a_matcher_in_a_page_value_is_two_delimiters() {
    for operator in ["$=", "*=", "^=", "|=", "~="] {
        for declaration in ["size: OP", "bleed: OP", "bleed: 1px OP", "marks: OP"] {
            let declaration = declaration.replace("OP", operator);
            let source = format!("@page {{ {declaration} }}");
            let report = CssParser::default().parse_sheet(&source);
            let rejected: Vec<_> = report
                .diagnostics()
                .iter()
                .filter(|diagnostic| {
                    diagnostic.error().code() == CssErrorCode::InvalidDescriptorValue
                })
                .collect();
            assert_eq!(rejected.len(), 1, "{source:?}");
            let error = rejected[0].error();
            let token = error.encountered().expect("the rejected token");
            assert_eq!(format!("{:?}", token.kind()), "Delim", "{source:?}");
            assert_eq!(token.authored(), &operator[..1], "{source:?}");
            assert_eq!(
                error.position().byte_offset().value(),
                source.find(operator).unwrap(),
                "{source:?}"
            );
        }
    }
}

/// A rejected matcher is named by its first delimiter where it stands: after a
/// multi-byte comment on a line that a CRLF began.
#[test]
fn a_matcher_in_a_descriptor_names_its_first_delimiter_after_a_multibyte_comment_and_crlf() {
    for operator in ["$=", "*=", "^=", "|=", "~="] {
        let source = format!("@font-face {{\r\n/*😀*/ font-weight: {operator} }}");
        let report = CssParser::default().parse_sheet(&source);
        let rejected: Vec<_> = report
            .diagnostics()
            .iter()
            .filter(|diagnostic| diagnostic.error().code() == CssErrorCode::InvalidDescriptorValue)
            .collect();
        let [diagnostic] = rejected.as_slice() else {
            panic!("one rejection: {source:?}");
        };
        let error = diagnostic.error();
        let token = error.encountered().expect("the rejected token");
        assert_eq!(format!("{:?}", token.kind()), "Delim", "{source:?}");
        assert_eq!(token.authored(), &operator[..1], "{source:?}");
        let position = error.position();
        assert_eq!(
            position.byte_offset().value(),
            source.find(operator).unwrap(),
            "{source:?}"
        );
        assert_eq!(position.line().value(), 1, "{source:?}");
        // The comment is six UTF-16 units: `/*`, a surrogate pair, `*/`.
        assert_eq!(
            position.column().value(),
            "/*😀*/ font-weight: ".encode_utf16().count() as u32,
            "{source:?}"
        );
    }
}
