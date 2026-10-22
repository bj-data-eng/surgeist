//! Syntax 3 (CRD 2026-10-01) tokenizes a reverse solidus followed by a
//! newline as a `<delim-token>`, and declaration consumption removes the
//! trailing whitespace. The retained `\` then ends the value: re-tokenizing
//! `\` at EOF consumes an escape and yields an identifier, so no CSS text
//! preserves the retained tokens. That is an unserializable boundary, not a
//! resource exhaustion, whichever serializer reaches it. Pending values keep
//! their trailing whitespace, so their boundary case is pinned privately in
//! `src/pending_serialization.rs`.

use surgeist_css::*;

#[test]
fn named_supports_body_reports_an_unserializable_trailing_reverse_solidus() {
    // The tokenizer reports the bare reverse solidus as a parse error, so the
    // single-child entry declines; the inert body still retains the child.
    let (body, _) = parse_supports_test_contents("future: a \\\n;").into_parts();
    let body = body.expect("inert body");
    assert_eq!(body.children().count(), 1);
    assert_eq!(
        body.to_specified_css_with_graph_limits(CssRuleGraphSerializationLimits {
            values: CssSpecifiedValueSerializationLimits::default(),
            max_depth: 8,
        })
        .unwrap_err()
        .kind(),
        CssSpecifiedValueSerializationErrorKind::UnserializableBoundary
    );
}
