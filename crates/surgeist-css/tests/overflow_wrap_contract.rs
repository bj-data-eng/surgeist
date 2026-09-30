#![forbid(unsafe_code)]

//! Typed authored overflow wrapping contracts from CSS Text 4 §6.4 (2026-08-14).

use surgeist_css::*;

fn parsed(name: &str, value: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{name}:{value}!important"));
    assert!(
        report.is_clean(),
        "{name}:{value}: {:?}",
        report.diagnostics()
    );
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    declaration.clone()
}

fn checked(name: &str, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        CssPropertyGrammar::from_name(name).expect("known spelling"),
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .expect("checked authored value")
}

fn exact_value(source: &CssDeclaration) -> CssOverflowWrap {
    let CssKnownPropertyValueRef::OverflowWrap(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed overflow-wrap wrapper")
    };
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
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
        assert_eq!(value.serialize_specified().unwrap(), spelling);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    spelling.len()
                ))
                .unwrap(),
            spelling
        );
        for (limits, expected) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, spelling.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, spelling.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, spelling.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
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
        expand_declaration(&parsed("all", "initial")).unwrap()
    else {
        panic!("symbolic all reset")
    };
    assert!(!reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::OverflowWrap)));

    let word_break = property_support_metadata("word-break").expect("shared Text 3 source");
    assert_eq!(word_break.feature().source().id().as_str(), "S-TEXT3");
    assert_eq!(word_break.feature().status(), CssSupportStatus::Partial);
}
