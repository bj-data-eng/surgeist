#![forbid(unsafe_code)]

//! Functional authored `quotes` API for CSS Generated Content 3 §2.4.1:
//! https://www.w3.org/TR/2025/WD-css-content-3-20251204/#propdef-quotes
//! Locale selection for `auto` and parent resolution for `match-parent` occur later.

use surgeist_css::*;

fn pair(open: &str, close: &str) -> CssQuotePair {
    CssQuotePair::new(
        CssContentString::try_new(open).unwrap(),
        CssContentString::try_new(close).unwrap(),
    )
}

fn list(pairs: Vec<CssQuotePair>) -> CssQuotePairList {
    CssQuotePairList::try_new(pairs).expect("at least one pair")
}

fn authored(value: &str) -> CssDeclaration {
    let source = format!("quotes:{value}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one quotes declaration")
    };
    declaration.clone()
}

#[test]
fn checked_pairs_preserve_order_and_empty_strings_but_reject_empty_list_and_nul() {
    assert!(CssQuotePairList::try_new(Vec::new()).is_none());
    assert!(CssContentString::try_new("bad\0value").is_none());
    let first = pair("«,", "»");
    let second = pair("", "");
    let values = list(vec![first.clone(), second.clone()]);
    assert_eq!(values.pairs().len(), 2);
    assert_eq!(values.pairs()[0].open().as_str(), "«,");
    assert_eq!(values.pairs()[0].close().as_str(), "»");
    assert_eq!(values.pairs()[1].open().as_str(), "");
    assert_eq!(values.pairs()[1].close().as_str(), "");
    assert_eq!(first.serialize_specified().unwrap(), "\"«,\" \"»\"");
    assert_eq!(second.serialize_specified().unwrap(), "\"\" \"\"");
    assert_eq!(
        CssQuotes::Pairs(values).serialize_specified().unwrap(),
        "\"«,\" \"»\" \"\" \"\""
    );
}

#[test]
fn all_symbolic_keywords_serialize_without_context() {
    for (value, expected) in [
        (CssQuotes::Auto, "auto"),
        (CssQuotes::None, "none"),
        (CssQuotes::MatchParent, "match-parent"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    expected.len(),
                ))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, expected.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
    }
}

#[test]
fn decoded_keyword_spellings_keep_authored_text_and_serialize_canonically() {
    for (spelling, expected_value, canonical) in [
        ("AUTO", CssQuotes::Auto, "auto"),
        (r#"m\61 tch-parent"#, CssQuotes::MatchParent, "match-parent"),
        (r#"\6e one"#, CssQuotes::None, "none"),
    ] {
        let parsed = authored(spelling);
        let CssKnownPropertyValueRef::Quotes(wrapper) =
            parsed.known().unwrap().property_value().unwrap()
        else {
            panic!("decoded quotes keyword")
        };
        assert_eq!(wrapper.as_css(), spelling);
        assert_eq!(wrapper.quotes(), &expected_value);
        assert_eq!(wrapper.quotes().serialize_specified().unwrap(), canonical);
    }
}

#[test]
fn string_escaping_is_canonical_and_preserves_decoded_unicode_and_controls() {
    let escaped = pair("a\"b", "c\\d");
    assert_eq!(
        escaped.serialize_specified().unwrap(),
        "\"a\\\"b\" \"c\\\\d\""
    );
    let controls = pair("x\ny", "\u{1}é");
    assert_eq!(controls.serialize_specified().unwrap(), r#""x\a y" "\1 é""#);
    let unicode = pair("«", "»");
    assert_eq!(unicode.serialize_specified().unwrap(), "\"«\" \"»\"");
    assert_eq!(unicode.serialize_specified().unwrap().len(), 9);
    for (value, expected) in [
        (&escaped, "\"a\\\"b\" \"c\\\\d\""),
        (&unicode, "\"«\" \"»\""),
    ] {
        let bytes = expected.len();
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    3, 3, bytes,
                ))
                .unwrap(),
            expected
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    3,
                    3,
                    bytes - 1,
                ))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
    }

    let parsed = authored(r#""a\"b" "c\\d""#);
    let CssKnownPropertyValueRef::Quotes(wrapper) =
        parsed.known().unwrap().property_value().unwrap()
    else {
        panic!("parsed quotes")
    };
    let CssQuotes::Pairs(parsed_pairs) = wrapper.quotes() else {
        panic!("parsed pair")
    };
    assert_eq!(parsed_pairs.pairs(), &[escaped]);
    assert_eq!(wrapper.as_css(), r#""a\"b" "c\\d""#);
}

#[test]
fn parsed_and_programmatic_pairs_share_typed_value_and_initial_is_auto() {
    let programmed = CssQuotes::Pairs(list(vec![pair("<", ">"), pair("[", "]")]));
    let parsed = authored("\"<\" \">\" \"[\" \"]\"");
    let CssKnownPropertyValueRef::Quotes(wrapper) =
        parsed.known().unwrap().property_value().unwrap()
    else {
        panic!("parsed quotes wrapper")
    };
    assert_eq!(wrapper.quotes(), &programmed);
    assert_eq!(
        programmed.serialize_specified().unwrap(),
        "\"<\" \">\" \"[\" \"]\""
    );
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::Quotes.metadata().unwrap().kind()
    else {
        panic!("quotes longhand")
    };
    assert!(metadata.inherited_by_default());
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("ordinary auto initial")
    };
    let CssLonghandValueRef::Quotes(value) = initial.view() else {
        panic!("typed quotes initial")
    };
    assert_eq!(value, &CssQuotes::Auto);
}

#[test]
fn pair_list_and_enum_share_cumulative_input_projection_and_byte_budgets() {
    let first = pair("a", "b");
    let pairs = list(vec![first.clone(), pair("c", "d")]);
    let quoted = CssQuotes::Pairs(pairs.clone());
    assert_eq!(first.serialize_specified().unwrap(), "\"a\" \"b\"");
    assert_eq!(
        pairs.serialize_specified().unwrap(),
        "\"a\" \"b\" \"c\" \"d\""
    );
    assert_eq!(
        quoted.serialize_specified().unwrap(),
        "\"a\" \"b\" \"c\" \"d\""
    );
    assert_eq!(
        first
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 7))
            .unwrap(),
        "\"a\" \"b\""
    );
    let expected = "\"a\" \"b\" \"c\" \"d\"";
    assert_eq!(
        quoted
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                5,
                5,
                expected.len(),
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(4, 5, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 4, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 5, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            quoted
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(
            pairs
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(quoted.serialize_specified().unwrap(), expected);
    assert_eq!(pairs.pairs()[0], first);
}
