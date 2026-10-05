#![forbid(unsafe_code)]
//! CSS Syntax 3 §4.3.6 leaves the first non-whitespace input for the bad-URL
//! remnants algorithm. Section 4.3.14 consumes valid escapes before recognizing
//! an unescaped closing parenthesis; the first escape cannot be discarded.

use surgeist_css::{CssComponentValueErrorKind, CssValueOrigin, parse_component_values};

fn assert_bad_url_range(source: &str, end: usize) {
    let error = parse_component_values(source).expect_err("invalid trailing URL content");
    assert_eq!(error.kind(), CssComponentValueErrorKind::BadUrl);
    let CssValueOrigin::Parsed(origin) = error.origin() else {
        panic!("bad URL retains its genuine authored source");
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    assert_eq!(origin.span().end().byte_offset().value(), end);
}

#[test]
fn first_post_whitespace_escape_keeps_the_escaped_closer_inside_bad_url_remnants() {
    let source = r"url(a \)still)";
    assert_bad_url_range(source, source.len());
}

#[test]
fn first_post_whitespace_backslash_pair_keeps_the_literal_closer_as_the_boundary() {
    let source = r"url(a \\)tail";
    assert_bad_url_range(source, 9);
}

#[test]
fn preceding_nonmember_and_quote_controls_keep_their_unescaped_final_boundary() {
    for source in [r"url(a b\)still)", "url(a \"x)"] {
        assert_bad_url_range(source, source.len());
    }
}
