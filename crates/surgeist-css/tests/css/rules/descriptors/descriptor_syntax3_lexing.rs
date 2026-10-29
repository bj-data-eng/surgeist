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
const RULES: [(&str, &str); 4] = [
    ("@font-palette-values --p", "base-palette: light"),
    ("@color-profile --c", "rendering-intent: perceptual"),
    ("@counter-style x", "system: cyclic"),
    ("@font-feature-values A", "font-display: swap"),
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
