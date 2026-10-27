//! Functional authored `quotes` API for CSS Generated Content 3 §2.4.1:
//! https://www.w3.org/TR/2025/WD-css-content-3-20251204/#propdef-quotes
//! Locale selection for `auto` and parent resolution for `match-parent` occur later.

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::normal_declaration;
use surgeist_css::property::{
    CssInitialValueRef, CssKnownProperty, CssKnownPropertyValueRef, CssLonghandValueRef,
    CssPropertyKindRef,
};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::content::{CssContentString, CssQuotePair, CssQuotePairList, CssQuotes};

fn pair(open: &str, close: &str) -> CssQuotePair {
    CssQuotePair::new(
        CssContentString::try_new(open).unwrap(),
        CssContentString::try_new(close).unwrap(),
    )
}

fn list(pairs: Vec<CssQuotePair>) -> CssQuotePairList {
    CssQuotePairList::try_new(pairs).expect("at least one pair")
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
    assert_eq!(
        first.serialize_specified(Default::default()).unwrap(),
        "\"«,\" \"»\""
    );
    assert_eq!(
        second.serialize_specified(Default::default()).unwrap(),
        "\"\" \"\""
    );
    assert_eq!(
        CssQuotes::Pairs(values)
            .serialize_specified(Default::default())
            .unwrap(),
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
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            expected
        );
        assert_eq!(
            value
                .serialize_specified(CssSerializationLimits::new(1, 1, expected.len(),))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (
                CssSerializationLimits::new(0, 1, expected.len()),
                SerializationFailure::InputNodeLimit,
            ),
            (
                CssSerializationLimits::new(1, 0, expected.len()),
                SerializationFailure::ProjectionNodeLimit,
            ),
            (
                CssSerializationLimits::new(1, 1, expected.len() - 1),
                SerializationFailure::ByteLimit,
            ),
        ] {
            assert_eq!(
                serialization_failure(&value.serialize_specified(limits).unwrap_err()),
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
        let parsed = normal_declaration("quotes", spelling);
        let CssKnownPropertyValueRef::Quotes(wrapper) =
            parsed.known().unwrap().property_value().unwrap()
        else {
            panic!("decoded quotes keyword")
        };
        assert_eq!(wrapper.as_css(), spelling);
        assert_eq!(wrapper.value(), &expected_value);
        assert_eq!(
            wrapper
                .value()
                .serialize_specified(Default::default())
                .unwrap(),
            canonical
        );
    }
}

#[test]
fn string_escaping_is_canonical_and_preserves_decoded_unicode_and_controls() {
    let escaped = pair("a\"b", "c\\d");
    assert_eq!(
        escaped.serialize_specified(Default::default()).unwrap(),
        "\"a\\\"b\" \"c\\\\d\""
    );
    let controls = pair("x\ny", "\u{1}é");
    assert_eq!(
        controls.serialize_specified(Default::default()).unwrap(),
        r#""x\a y" "\1 é""#
    );
    let unicode = pair("«", "»");
    assert_eq!(
        unicode.serialize_specified(Default::default()).unwrap(),
        "\"«\" \"»\""
    );
    assert_eq!(
        unicode
            .serialize_specified(Default::default())
            .unwrap()
            .len(),
        9
    );
    for (value, expected) in [
        (&escaped, "\"a\\\"b\" \"c\\\\d\""),
        (&unicode, "\"«\" \"»\""),
    ] {
        let bytes = expected.len();
        assert_eq!(
            value
                .serialize_specified(CssSerializationLimits::new(3, 3, bytes,))
                .unwrap(),
            expected
        );
        assert!(
            value
                .serialize_specified(CssSerializationLimits::new(3, 3, bytes - 1,))
                .unwrap_err()
                .is_byte_limit()
        );
    }

    let parsed = normal_declaration("quotes", r#""a\"b" "c\\d""#);
    let CssKnownPropertyValueRef::Quotes(wrapper) =
        parsed.known().unwrap().property_value().unwrap()
    else {
        panic!("parsed quotes")
    };
    let CssQuotes::Pairs(parsed_pairs) = wrapper.value() else {
        panic!("parsed pair")
    };
    assert_eq!(parsed_pairs.pairs(), &[escaped]);
    assert_eq!(wrapper.as_css(), r#""a\"b" "c\\d""#);
}

#[test]
fn parsed_and_programmatic_pairs_share_typed_value_and_initial_is_auto() {
    let programmed = CssQuotes::Pairs(list(vec![pair("<", ">"), pair("[", "]")]));
    let parsed = normal_declaration("quotes", "\"<\" \">\" \"[\" \"]\"");
    let CssKnownPropertyValueRef::Quotes(wrapper) =
        parsed.known().unwrap().property_value().unwrap()
    else {
        panic!("parsed quotes wrapper")
    };
    assert_eq!(wrapper.value(), &programmed);
    assert_eq!(
        programmed.serialize_specified(Default::default()).unwrap(),
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
    assert_eq!(
        first.serialize_specified(Default::default()).unwrap(),
        "\"a\" \"b\""
    );
    assert_eq!(
        pairs.serialize_specified(Default::default()).unwrap(),
        "\"a\" \"b\" \"c\" \"d\""
    );
    assert_eq!(
        quoted.serialize_specified(Default::default()).unwrap(),
        "\"a\" \"b\" \"c\" \"d\""
    );
    assert_eq!(
        first
            .serialize_specified(CssSerializationLimits::new(3, 3, 7))
            .unwrap(),
        "\"a\" \"b\""
    );
    let expected = "\"a\" \"b\" \"c\" \"d\"";
    assert_eq!(
        quoted
            .serialize_specified(CssSerializationLimits::new(5, 5, expected.len(),))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSerializationLimits::new(4, 5, expected.len()),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(5, 4, expected.len()),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(5, 5, expected.len() - 1),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&quoted.serialize_specified(limits).unwrap_err()),
            kind
        );
        assert_eq!(
            serialization_failure(&pairs.serialize_specified(limits).unwrap_err()),
            kind
        );
    }
    assert_eq!(
        quoted.serialize_specified(Default::default()).unwrap(),
        expected
    );
    assert_eq!(pairs.pairs()[0], first);
}
