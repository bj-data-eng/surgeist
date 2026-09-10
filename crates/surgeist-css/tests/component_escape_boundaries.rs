//! CSS Syntax 3 section 4.3.7 consumes one whitespace after a hex escape,
//! including a six-digit escape. Section 10 requires token-preserving output.
//! https://www.w3.org/TR/2021/CRD-css-syntax-3-20211224/

use surgeist_css::{
    CssComponentValue, CssComponentValueErrorKind, CssComponentValueRef, CssComponentValues,
    CssSerializedOrigin, CssValueOrigin, CssValueTokenRef, parse_component_values,
};

fn joined(left: &str, right: &str) -> CssComponentValues {
    let mut items = parse_component_values(left).unwrap().items().to_vec();
    items.extend_from_slice(parse_component_values(right).unwrap().items());
    CssComponentValues::try_new(items).unwrap()
}

fn assert_left(value: &surgeist_css::CssComponentValue, spelling: &str) {
    match (spelling, value.view()) {
        (r"\31" | r"\000031", CssComponentValueRef::Token(CssValueTokenRef::Ident("1")))
        | (r"@\31", CssComponentValueRef::Token(CssValueTokenRef::AtKeyword("1")))
        | (r"#\31", CssComponentValueRef::Token(CssValueTokenRef::Hash { value: "1", .. })) => {}
        (
            r"1\70",
            CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit: "p" }),
        ) => assert_eq!(number.representation(), "1"),
        (_, other) => panic!("wrong token after serializing {spelling}: {other:?}"),
    }
}

#[test]
fn hex_escape_endings_preserve_separately_originating_whitespace() {
    for spelling in [r"\31", r"\000031", r"@\31", r"#\31", r"1\70"] {
        for whitespace in [" ", "\t", "\n", "\r", "\r\n", "\u{c}"] {
            let values = joined(spelling, whitespace);
            let output = values.serialize().unwrap();
            assert_eq!(output.as_css(), format!("{spelling}/**/{whitespace}"));
            let reparsed = parse_component_values(output.as_css()).unwrap();
            let [left, separator, right] = reparsed.items() else {
                panic!("retain the left token, separator and whitespace: {output:?}");
            };
            assert_left(left, spelling);
            assert!(matches!(
                separator.view(),
                CssComponentValueRef::Comment("")
            ));
            assert!(matches!(right.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Whitespace(actual))
                    if actual == whitespace));
        }
    }
}

#[test]
fn escape_boundary_separators_keep_both_origins_and_count_toward_limits() {
    let values = joined(r"\31", " ");
    let output = values.serialize().unwrap();
    let left = values.items()[0].origin();
    let right = values.items()[1].origin();
    for offset in 3..7 {
        assert!(matches!(output.origin_at(offset),
            Some(CssSerializedOrigin::Separator { before, after })
                if before == left && after == right));
    }
    assert!(matches!(output.origin_at(7),
        Some(CssSerializedOrigin::Token(origin)) if origin == right));
    let failure = values.serialize_with_limit(7).unwrap_err();
    assert_eq!(failure.kind(), CssComponentValueErrorKind::ByteLimit);
    assert_eq!(failure.origin(), right);
    assert_eq!(
        values.serialize_with_limit(8).unwrap().as_css(),
        r"\31/**/ "
    );
}

#[test]
fn checked_tokens_use_the_same_escape_boundary_without_parsed_origins() {
    let values = CssComponentValues::try_new(vec![
        CssComponentValue::try_token(r"\31").unwrap(),
        CssComponentValue::try_token(" ").unwrap(),
    ])
    .unwrap();
    let output = values.serialize().unwrap();
    assert_eq!(output.as_css(), r"\31/**/ ");
    assert!(matches!(
        output.origin_at(3),
        Some(CssSerializedOrigin::Separator {
            before: CssValueOrigin::Programmatic,
            after: CssValueOrigin::Programmatic,
        })
    ));
}

#[test]
fn terminated_or_escaped_backslashes_do_not_create_extra_separators() {
    for spelling in [r"\31 ", r"\000031 ", r"\\31", "plain"] {
        let values = joined(spelling, " ");
        let output = values.serialize().unwrap();
        assert_eq!(output.as_css(), format!("{spelling} "));
        let reparsed = parse_component_values(output.as_css()).unwrap();
        assert_eq!(reparsed.items().len(), 2, "{output:?}");
        assert!(matches!(
            reparsed.items()[1].view(),
            CssComponentValueRef::Token(CssValueTokenRef::Whitespace(" "))
        ));
    }
}
