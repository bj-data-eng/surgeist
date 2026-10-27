//! Typed authored overflow wrapping contracts from CSS Text 4 §6.4 (2026-08-14).

use crate::css::support::checked_grammar_declaration as checked;
use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::important_declaration as parsed;
use surgeist_css::conformance::{CssSupportStatus, property_support_metadata};
use surgeist_css::declaration::{CssDeclaration, CssImportance};
use surgeist_css::property::{
    CssContributions, CssExpansion, CssInitialValueRef, CssKnownProperty, CssKnownPropertyValueRef,
    CssLonghandValueRef, CssPropertyKindRef, CssPropertyNameRef,
};
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::text::CssOverflowWrap;

fn exact_value(source: &CssDeclaration) -> CssOverflowWrap {
    let CssKnownPropertyValueRef::OverflowWrap(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed overflow-wrap wrapper")
    };
    let CssExpansion::Contributions(CssContributions::Longhands(values)) = source.expand().unwrap()
    else {
        panic!("terminal expansion")
    };
    let [item] = values.items() else {
        panic!("one canonical contribution")
    };
    assert_eq!(item.property(), CssKnownProperty::OverflowWrap);
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), CssImportance::Important);
    let CssLonghandValueRef::OverflowWrap(expanded) = item.ordinary_value().unwrap().view() else {
        panic!("typed longhand value")
    };
    assert_eq!(expanded, wrapper.value());
    *expanded
}

#[test]
fn parsed_checked_and_expanded_values_retain_three_distinct_keywords() {
    for (spelling, expected) in [
        ("normal", CssOverflowWrap::Normal),
        ("break-word", CssOverflowWrap::BreakWord),
        ("anywhere", CssOverflowWrap::Anywhere),
    ] {
        for name in ["overflow-wrap", "word-wrap"] {
            assert_eq!(exact_value(&parsed(name, spelling)), expected);
            assert_eq!(exact_value(&checked(name, spelling)), expected);
        }
    }
    let CssPropertyKindRef::Longhand(longhand) =
        CssKnownProperty::OverflowWrap.metadata().unwrap().kind()
    else {
        panic!("terminal metadata")
    };
    let initial_value = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("fixed initial")
    };
    assert_eq!(
        initial.view(),
        CssLonghandValueRef::OverflowWrap(&CssOverflowWrap::Normal)
    );
}

#[test]
fn direct_keyword_serialization_is_canonical_and_budgeted() {
    for (value, spelling) in [
        (CssOverflowWrap::Normal, "normal"),
        (CssOverflowWrap::BreakWord, "break-word"),
        (CssOverflowWrap::Anywhere, "anywhere"),
    ] {
        assert_eq!(
            value.serialize_specified(Default::default()).unwrap(),
            spelling
        );
        assert_eq!(
            value
                .serialize_specified(CssSerializationLimits::new(1, 1, spelling.len()))
                .unwrap(),
            spelling
        );
        for (limits, expected) in [
            (
                CssSerializationLimits::new(0, 1, spelling.len()),
                SerializationFailure::InputNodeLimit,
            ),
            (
                CssSerializationLimits::new(1, 0, spelling.len()),
                SerializationFailure::ProjectionNodeLimit,
            ),
            (
                CssSerializationLimits::new(1, 1, spelling.len() - 1),
                SerializationFailure::ByteLimit,
            ),
        ] {
            assert_eq!(
                serialization_failure(&value.serialize_specified(limits).unwrap_err()),
                expected
            );
        }
    }
}

#[test]
fn authored_alias_origin_and_all_reset_use_the_canonical_identity() {
    let source = parsed("WoRd-WrAp", "anywhere");
    let origin = source.parsed_name().expect("parsed alias origin");
    let span = origin.span();
    let raw = origin.source().as_str();
    assert_eq!(
        &raw[span.start().byte_offset().value()..span.end().byte_offset().value()],
        "WoRd-WrAp"
    );
    assert_eq!(
        source.known().unwrap().property(),
        CssKnownProperty::OverflowWrap
    );
    assert_eq!(exact_value(&source), CssOverflowWrap::Anywhere);

    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        parsed("all", "initial").expand().unwrap()
    else {
        panic!("symbolic all reset")
    };
    assert!(!reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::OverflowWrap)));

    let word_break = property_support_metadata("word-break").expect("selected Text 4 source");
    assert_eq!(word_break.feature().source().id().as_str(), "X-TEXT4");
    assert_eq!(word_break.feature().status(), CssSupportStatus::Complete);
}
