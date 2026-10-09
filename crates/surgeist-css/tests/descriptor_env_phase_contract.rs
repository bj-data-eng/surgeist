#![forbid(unsafe_code)]

//! Independent functional new-capability draft for the adopted descriptor env
//! phase contract. Env1 WD20250923 §3; Syntax3 CRD20211224 §§8.2/9.2;
//! selected Counter Styles3 §3 and Fonts4 §6.9 with adopted completed-index
//! cardinalities. No environment lookup, computed value or author store is modeled.

use surgeist_css::{
    CssComponentValue, CssComponentValueErrorKind as ComponentFailure, CssComponentValueLimits,
    CssComponentValueRef, CssComponentValues, CssCounterStyleDescriptorKind as CounterKind,
    CssCounterStyleDescriptorRef as CounterOccurrence, CssCounterStyleDescriptorValue as Counter,
    CssCounterStyleDescriptorValueRef as CounterView, CssCounterStyleValueError,
    CssCounterStyleValueErrorKind as CounterFailure, CssCounterSymbol, CssFontDisplay,
    CssFontFeatureDisplayOccurrence, CssFontFeatureDisplayValue as Display,
    CssFontFeatureDisplayValueRef as DisplayView, CssFontFeatureValue as Feature,
    CssFontFeatureValueBlock, CssFontFeatureValueDefinition, CssFontFeatureValueError,
    CssFontFeatureValueErrorKind as FontFailure, CssFontFeatureValueKind as FeatureKind,
    CssFontFeatureValueName, CssFontFeatureValueRef as FeatureView, CssFontFeatureValuesErrorKind,
    CssFontFeatureValuesItem as Item, CssNormalizedItem, CssParsedOrigin, CssRule,
    CssRuleContextKindRef, CssSerializedOrigin,
    CssSpecifiedRuleSerializationErrorKind as RuleEmitFailure,
    CssSpecifiedValueSerializationErrorKind as EmitFailure,
    CssSpecifiedValueSerializationLimits as EmitLimits, CssValueOrigin, CssValueTokenRef,
    normalize_report, parse_component_values, parse_counter_style_block,
    parse_counter_style_descriptor_value, parse_font_feature_display_value,
    parse_font_feature_value, parse_font_feature_values_block, parse_sheet,
};

fn components(source: &str) -> CssComponentValues {
    parse_component_values(source).unwrap()
}
fn counter(kind: CounterKind, source: &str) -> Counter {
    Counter::try_from_components(kind, components(source)).unwrap()
}
fn display(source: &str) -> Display {
    Display::try_from_components(components(source)).unwrap()
}
fn feature(kind: FeatureKind, source: &str) -> Feature {
    Feature::try_from_components(kind, components(source)).unwrap()
}
fn point(origin: &CssParsedOrigin, source: &str, start: usize, end: usize) {
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), end);
}
fn counter_grammar(error: CssCounterStyleValueError) {
    assert!(
        matches!(error.kind(), CounterFailure::Grammar(_)),
        "{error:?}"
    );
}
fn font_grammar(error: CssFontFeatureValueError) {
    assert!(matches!(error.kind(), FontFailure::Grammar(_)), "{error:?}");
}

const COUNTER_ORDINARY: [(CounterKind, &str); 10] = [
    (CounterKind::System, "cyclic"),
    (CounterKind::Negative, "\"-\""),
    (CounterKind::Symbols, "\"a\" \"b\""),
    (CounterKind::Prefix, "\"(\""),
    (CounterKind::Suffix, "\")\""),
    (CounterKind::Range, "1 9"),
    (CounterKind::Pad, "2 \"0\""),
    (CounterKind::Fallback, "decimal"),
    (CounterKind::AdditiveSymbols, "1 \"I\", 0 \"Z\""),
    (CounterKind::SpeakAs, "words"),
];
const FEATURE_ORDINARY: [(FeatureKind, &str, &[&str]); 7] = [
    (FeatureKind::Stylistic, "1", &["1"]),
    (FeatureKind::HistoricalForms, "1 2 3", &["1", "2", "3"]),
    (FeatureKind::Styleset, "0 100 101", &["0", "100", "101"]),
    (FeatureKind::CharacterVariant, "0 101", &["0", "101"]),
    (FeatureKind::Swash, "2", &["2"]),
    (FeatureKind::Ornaments, "3", &["3"]),
    (FeatureKind::Annotation, "4", &["4"]),
];

fn ordinary_counter(value: &Counter, kind: CounterKind, specified: &str) {
    assert_eq!(value.kind(), kind);
    assert!(matches!(
        (kind, value.view()),
        (CounterKind::System, CounterView::System(_))
            | (CounterKind::Negative, CounterView::Negative(_))
            | (CounterKind::Symbols, CounterView::Symbols(_))
            | (CounterKind::Prefix, CounterView::Prefix(_))
            | (CounterKind::Suffix, CounterView::Suffix(_))
            | (CounterKind::Range, CounterView::Range(_))
            | (CounterKind::Pad, CounterView::Pad(_))
            | (CounterKind::Fallback, CounterView::Fallback(_))
            | (
                CounterKind::AdditiveSymbols,
                CounterView::AdditiveSymbols(_)
            )
            | (CounterKind::SpeakAs, CounterView::SpeakAs(_))
    ));
    assert_eq!(value.to_specified_css().unwrap(), specified);
}
fn ordinary_feature(value: &Feature, kind: FeatureKind, digits: &[&str]) {
    assert_eq!(value.kind(), kind);
    let FeatureView::Indexes(indexes) = value.view() else {
        panic!("completed indexes")
    };
    assert_eq!(
        indexes
            .iter()
            .map(|n| n.as_decimal_str())
            .collect::<Vec<_>>(),
        digits
    );
}

#[test]
fn all_counter_ordinary_views_construct_and_pending_reentry_keeps_the_kind() {
    for (kind, source) in COUNTER_ORDINARY {
        let supplied = components(source);
        let value = Counter::try_from_components(kind, supplied.clone()).unwrap();
        ordinary_counter(&value, kind, source);
        assert_eq!(value.components(), &supplied);
        assert!(value.origin().is_none());
        let pending = counter(kind, "env(choice)");
        assert!(matches!(pending.view(), CounterView::Pending(_)));
        let completed = pending
            .reparse_after_substitution(supplied.clone())
            .unwrap();
        ordinary_counter(&completed, kind, source);
        assert!(completed.origin().is_none());
        assert_eq!(completed.components(), &supplied);
        assert!(matches!(pending.view(), CounterView::Pending(_)));
    }
}

#[test]
fn all_five_display_keywords_construct_and_reenter_as_ordinary_without_a_default() {
    for (source, expected) in [
        ("auto", CssFontDisplay::Auto),
        ("block", CssFontDisplay::Block),
        ("swap", CssFontDisplay::Swap),
        ("fallback", CssFontDisplay::Fallback),
        ("optional", CssFontDisplay::Optional),
    ] {
        let supplied = components(source);
        let ordinary = Display::try_from_components(supplied.clone()).unwrap();
        assert!(matches!(ordinary.view(), DisplayView::Ordinary(actual) if actual == expected));
        assert!(ordinary.origin().is_none());
        assert_eq!(ordinary.components(), &supplied);
        assert_eq!(ordinary.to_specified_css().unwrap(), source);
        let pending = display("env(choice)");
        let completed = pending.reparse_after_substitution(supplied).unwrap();
        assert!(matches!(completed.view(), DisplayView::Ordinary(actual) if actual == expected));
        assert!(completed.origin().is_none());
        assert!(matches!(pending.view(), DisplayView::Pending(_)));
    }
}

#[test]
fn seven_completed_feature_kinds_and_pending_reentry_keep_their_selected_indexes() {
    for (kind, source, digits) in FEATURE_ORDINARY {
        let supplied = components(source);
        let value = Feature::try_from_components(kind, supplied.clone()).unwrap();
        ordinary_feature(&value, kind, digits);
        assert!(value.origin().is_none());
        assert_eq!(value.components(), &supplied);
        assert_eq!(value.to_specified_css().unwrap(), source);
        let pending = feature(kind, "env(choice)");
        let completed = pending.reparse_after_substitution(supplied).unwrap();
        ordinary_feature(&completed, kind, digits);
        assert!(completed.origin().is_none());
        assert!(matches!(pending.view(), FeatureView::Pending(_)));
    }
}

#[test]
fn all_pending_views_retain_global_name_indices_fallback_and_supplied_components() {
    let source = "env(--Theme 2, red, blue)";
    let supplied = components(source);
    for (kind, _) in COUNTER_ORDINARY {
        let value = Counter::try_from_components(kind, supplied.clone()).unwrap();
        assert!(matches!(value.view(), CounterView::Pending(_)));
        assert_eq!(value.kind(), kind);
        assert_eq!(value.components(), &supplied);
        assert!(value.origin().is_none());
        assert_eq!(value.to_specified_css().unwrap(), source);
    }
    let value = Display::try_from_components(supplied.clone()).unwrap();
    assert!(matches!(value.view(), DisplayView::Pending(_)));
    assert_eq!(value.components(), &supplied);
    assert!(value.origin().is_none());
    for (kind, _, _) in FEATURE_ORDINARY {
        let value = Feature::try_from_components(kind, supplied.clone()).unwrap();
        assert!(matches!(value.view(), FeatureView::Pending(_)));
        assert_eq!(value.kind(), kind);
        assert_eq!(value.components(), &supplied);
        assert!(value.origin().is_none());
    }
    let CssComponentValueRef::Function(env) = supplied.items()[0].view() else {
        panic!("env")
    };
    assert!(matches!(
        env.values().items()[0].view(),
        CssComponentValueRef::Token(CssValueTokenRef::Ident("--Theme"))
    ));
    assert!(env.values().items().iter().any(|v| matches!(v.view(), CssComponentValueRef::Token(CssValueTokenRef::Number(n)) if n.representation() == "2")));
}

#[test]
fn raw_windows_are_optional_whole_values_and_function_origins_are_delimiters() {
    fn check(origin: &CssParsedOrigin, supplied: &CssComponentValues) {
        point(origin, "env(x)", 0, 6);
        let [component] = supplied.items() else {
            panic!("one env")
        };
        let CssComponentValueRef::Function(env) = component.view() else {
            panic!("env")
        };
        let CssValueOrigin::Parsed(opening) = component.origin() else {
            panic!("opener")
        };
        point(opening, "env(x)", 0, 4);
        let CssValueOrigin::Parsed(closing) = env.closing_origin() else {
            panic!("closer")
        };
        point(closing, "env(x)", 5, 6);
        assert!(origin.source().same_snapshot(opening.source()));
        assert!(origin.source().same_snapshot(closing.source()));
    }
    let raw = parse_counter_style_descriptor_value("env(x)", CounterKind::Prefix);
    assert!(raw.is_clean());
    let value = raw.syntax().as_ref().unwrap();
    assert!(matches!(value.view(), CounterView::Pending(_)));
    check(value.origin().unwrap(), value.components());
    let raw = parse_font_feature_display_value("env(x)");
    assert!(raw.is_clean());
    let value = raw.syntax().as_ref().unwrap();
    assert!(matches!(value.view(), DisplayView::Pending(_)));
    check(value.origin().unwrap(), value.components());
    let raw = parse_font_feature_value("env(x)", FeatureKind::Styleset);
    assert!(raw.is_clean());
    let value = raw.syntax().as_ref().unwrap();
    assert!(matches!(value.view(), FeatureView::Pending(_)));
    check(value.origin().unwrap(), value.components());
    assert!(counter(CounterKind::Prefix, "env(x)").origin().is_none());
    assert!(display("env(x)").origin().is_none());
    assert!(feature(FeatureKind::Styleset, "env(x)").origin().is_none());
}

#[test]
fn mixed_pending_components_and_programmatic_delimiters_keep_truthful_origins() {
    let mut mixed = components("env(x) ").items().to_vec();
    mixed.push(CssComponentValue::try_ident("later").unwrap());
    let mixed = CssComponentValues::try_new(mixed).unwrap();
    let a = Counter::try_from_components(CounterKind::Prefix, mixed.clone()).unwrap();
    let b = Display::try_from_components(mixed.clone()).unwrap();
    let c = Feature::try_from_components(FeatureKind::Styleset, mixed.clone()).unwrap();
    assert!(matches!(a.view(), CounterView::Pending(_)));
    assert!(matches!(b.view(), DisplayView::Pending(_)));
    assert!(matches!(c.view(), FeatureView::Pending(_)));
    assert_eq!(a.components(), &mixed);
    assert_eq!(b.components(), &mixed);
    assert_eq!(c.components(), &mixed);
    assert!(a.origin().is_none() && b.origin().is_none() && c.origin().is_none());
    assert!(matches!(
        mixed.items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert_eq!(
        mixed.items().last().unwrap().origin(),
        &CssValueOrigin::Programmatic
    );
    let input = CssComponentValues::try_new(vec![
        CssComponentValue::try_function("env", components("x")).unwrap(),
    ])
    .unwrap();
    let value = Display::try_from_components(input.clone()).unwrap();
    assert!(matches!(value.view(), DisplayView::Pending(_)));
    assert!(value.origin().is_none());
    assert_eq!(value.components(), &input);
    let CssComponentValueRef::Function(env) = value.components().items()[0].view() else {
        panic!("env")
    };
    assert_eq!(
        value.components().items()[0].origin(),
        &CssValueOrigin::Programmatic
    );
    assert_eq!(env.closing_origin(), &CssValueOrigin::Programmatic);
    let CssValueOrigin::Parsed(argument) = env.values().items()[0].origin() else {
        panic!("supplied parsed child")
    };
    point(argument, "x", 0, 1);
}

#[test]
fn programmatic_ordinary_values_and_mixed_index_replacements_do_not_fabricate_ranges() {
    let supplied =
        CssComponentValues::try_new(vec![CssComponentValue::try_number("+0002").unwrap()]).unwrap();
    let value = Feature::try_from_components(FeatureKind::Swash, supplied.clone()).unwrap();
    ordinary_feature(&value, FeatureKind::Swash, &["2"]);
    assert!(value.origin().is_none());
    assert_eq!(value.components(), &supplied);
    let FeatureView::Indexes(indexes) = value.view() else {
        panic!("indexes")
    };
    assert!(indexes[0].origin().is_none());
    assert_eq!(
        value.components().items()[0].origin(),
        &CssValueOrigin::Programmatic
    );
    assert!(
        matches!(value.components().items()[0].view(), CssComponentValueRef::Token(CssValueTokenRef::Number(n)) if n.representation() == "+0002")
    );
    let value = Display::try_from_components(
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("swap").unwrap()]).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        value.view(),
        DisplayView::Ordinary(CssFontDisplay::Swap)
    ));
    assert!(value.origin().is_none());
    let mut supplied = components("2 ").items().to_vec();
    supplied.push(CssComponentValue::try_string("0").unwrap());
    let supplied = CssComponentValues::try_new(supplied).unwrap();
    let value = Counter::try_from_components(CounterKind::Pad, supplied.clone()).unwrap();
    assert!(value.origin().is_none());
    assert_eq!(value.components(), &supplied);
    let CounterView::Pad(pad) = value.view() else {
        panic!("checked pad")
    };
    let surgeist_css::CssIntegerValue::Literal(minimum) = pad.minimum_length() else {
        panic!("ordinary literal pad length")
    };
    let CssValueOrigin::Parsed(number) = minimum.origin() else {
        panic!("actual parsed integer")
    };
    point(number, "2 ", 0, 1);
    assert!(matches!(pad.symbol(), CssCounterSymbol::String(value) if value.as_str() == "0"));
    assert_eq!(
        value.components().items().last().unwrap().origin(),
        &CssValueOrigin::Programmatic
    );
    let mut supplied = components("1 ").items().to_vec();
    supplied.extend_from_slice(components("2").items());
    let supplied = CssComponentValues::try_new(supplied).unwrap();
    let pending = feature(FeatureKind::Styleset, "env(x)");
    let completed = pending
        .reparse_after_substitution(supplied.clone())
        .unwrap();
    ordinary_feature(&completed, FeatureKind::Styleset, &["1", "2"]);
    assert!(completed.origin().is_none());
    assert_eq!(completed.components(), &supplied);
    let FeatureView::Indexes(indexes) = completed.view() else {
        panic!("indexes")
    };
    let first = indexes[0].origin().unwrap();
    let second = indexes[1].origin().unwrap();
    point(first, "1 ", 0, 1);
    point(second, "2", 0, 1);
    assert!(!first.source().same_snapshot(second.source()));
}

#[test]
fn strict_checked_admission_and_reentry_reject_original_implicit_closures() {
    for source in ["env(x", "env(x, {fallback", "env(x, \"fallback", "env(x)/*"] {
        let supplied = components(source);
        let a = Counter::try_from_components(CounterKind::Prefix, supplied.clone()).unwrap_err();
        let b = Display::try_from_components(supplied.clone()).unwrap_err();
        let c = Feature::try_from_components(FeatureKind::Styleset, supplied).unwrap_err();
        assert_eq!(a.kind(), &CounterFailure::RecoveredComponent);
        assert_eq!(b.kind(), &FontFailure::RecoveredComponent);
        assert_eq!(c.kind(), &FontFailure::RecoveredComponent);
        for origin in [a.origin(), b.origin(), c.origin()] {
            assert!(matches!(
                origin,
                CssSerializedOrigin::Token(CssValueOrigin::ImplicitClosure { .. })
            ));
        }
    }
    let a = counter(CounterKind::Prefix, "env(x)")
        .reparse_after_substitution(components("\"x"))
        .unwrap_err();
    let b = display("env(x)")
        .reparse_after_substitution(components("swap/*"))
        .unwrap_err();
    let c = feature(FeatureKind::Styleset, "env(x)")
        .reparse_after_substitution(components("1/*"))
        .unwrap_err();
    assert_eq!(a.kind(), &CounterFailure::RecoveredComponent);
    assert_eq!(b.kind(), &FontFailure::RecoveredComponent);
    assert_eq!(c.kind(), &FontFailure::RecoveredComponent);
    for origin in [a.origin(), b.origin(), c.origin()] {
        assert!(matches!(
            origin,
            CssSerializedOrigin::Token(CssValueOrigin::ImplicitClosure { .. })
        ));
    }
}

#[test]
fn checked_limits_are_cumulative_and_fail_before_pending_admission() {
    let supplied = components("env(x)");
    for (limits, expected) in [
        (
            CssComponentValueLimits::try_new(0, usize::MAX, usize::MAX).unwrap(),
            ComponentFailure::NestingLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, 0, usize::MAX).unwrap(),
            ComponentFailure::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, usize::MAX, 1).unwrap(),
            ComponentFailure::ByteLimit,
        ),
    ] {
        // The existing component owner is a secondary control for the adopted
        // conservation relation: the family wrapper must retain its exact error
        // origin, rather than manufacture source for a resource failure.
        let component_error =
            CssComponentValues::try_new_with_limits(supplied.items().to_vec(), limits).unwrap_err();
        assert_eq!(component_error.kind(), expected);
        let a =
            Counter::try_from_components_with_limits(CounterKind::Prefix, supplied.clone(), limits)
                .unwrap_err();
        let b = Display::try_from_components_with_limits(supplied.clone(), limits).unwrap_err();
        let c = Feature::try_from_components_with_limits(
            FeatureKind::Styleset,
            supplied.clone(),
            limits,
        )
        .unwrap_err();
        assert_eq!(a.kind(), &CounterFailure::Component(expected));
        assert_eq!(b.kind(), &FontFailure::Component(expected));
        assert_eq!(c.kind(), &FontFailure::Component(expected));
        for origin in [a.origin(), b.origin(), c.origin()] {
            assert_eq!(
                origin,
                &CssSerializedOrigin::Token(component_error.origin().clone())
            );
        }
    }
    // Each leaf fits independently, but all three components must share a budget.
    let supplied = components("1 2");
    let error = Feature::try_from_components_with_limits(
        FeatureKind::Styleset,
        supplied.clone(),
        CssComponentValueLimits::try_new(256, 2, usize::MAX).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &FontFailure::Component(ComponentFailure::ComponentLimit)
    );
    assert!(Feature::try_from_components(FeatureKind::Styleset, supplied).is_ok());
}

#[test]
fn reentry_limits_are_atomic_and_failed_attempts_leave_the_receiver_reusable() {
    let a = counter(CounterKind::Prefix, "env(x)");
    let b = display("env(x)");
    let c = feature(FeatureKind::Styleset, "env(x)");
    for (limits, expected) in [
        (
            CssComponentValueLimits::try_new(256, 0, usize::MAX).unwrap(),
            ComponentFailure::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, usize::MAX, 0).unwrap(),
            ComponentFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            a.reparse_after_substitution_with_limits(components("\"x\""), limits)
                .unwrap_err()
                .kind(),
            &CounterFailure::Component(expected)
        );
        assert_eq!(
            b.reparse_after_substitution_with_limits(components("swap"), limits)
                .unwrap_err()
                .kind(),
            &FontFailure::Component(expected)
        );
        assert_eq!(
            c.reparse_after_substitution_with_limits(components("1"), limits)
                .unwrap_err()
                .kind(),
            &FontFailure::Component(expected)
        );
    }
    let leaf_only = CssComponentValueLimits::try_new(0, usize::MAX, usize::MAX).unwrap();
    assert!(
        a.reparse_after_substitution_with_limits(components("\"x\""), leaf_only)
            .is_ok()
    );
    assert!(
        b.reparse_after_substitution_with_limits(components("swap"), leaf_only)
            .is_ok()
    );
    assert!(
        c.reparse_after_substitution_with_limits(components("1"), leaf_only)
            .is_ok()
    );
    assert!(matches!(a.view(), CounterView::Pending(_)));
    assert!(matches!(b.view(), DisplayView::Pending(_)));
    assert!(matches!(c.view(), FeatureView::Pending(_)));
}

#[test]
fn ordinary_receivers_reject_reentry_before_replacement_limits_without_a_false_origin() {
    let none = CssComponentValueLimits::try_new(0, 0, 0).unwrap();
    let a = counter(CounterKind::Prefix, "\"x\"")
        .reparse_after_substitution_with_limits(components("env(x)"), none)
        .unwrap_err();
    let b = display("swap")
        .reparse_after_substitution_with_limits(components("env(x)"), none)
        .unwrap_err();
    let c = feature(FeatureKind::Styleset, "1")
        .reparse_after_substitution_with_limits(components("env(x)"), none)
        .unwrap_err();
    assert_eq!(a.kind(), &CounterFailure::NotPending);
    assert_eq!(b.kind(), &FontFailure::NotPending);
    assert_eq!(c.kind(), &FontFailure::NotPending);
    for origin in [a.origin(), b.origin(), c.origin()] {
        assert_eq!(origin, &CssSerializedOrigin::End(None));
    }
}

#[test]
fn nested_residual_var_and_env_reject_before_ordinary_grammar_with_opener_origins() {
    for source in ["f(var(--x))", "f(env(x))", "f(ENV(x))"] {
        let a = counter(CounterKind::Prefix, "env(x)")
            .reparse_after_substitution(components(source))
            .unwrap_err();
        let b = display("env(x)")
            .reparse_after_substitution(components(source))
            .unwrap_err();
        let c = feature(FeatureKind::Styleset, "env(x)")
            .reparse_after_substitution(components(source))
            .unwrap_err();
        assert_eq!(a.kind(), &CounterFailure::ResidualSubstitution);
        assert_eq!(b.kind(), &FontFailure::ResidualSubstitution);
        assert_eq!(c.kind(), &FontFailure::ResidualSubstitution);
        for origin in [a.origin(), b.origin(), c.origin()] {
            let CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) = origin else {
                panic!("actual residual opener")
            };
            point(origin, source, 2, 6);
        }
    }
    let replacement = CssComponentValues::try_new(vec![
        CssComponentValue::try_function(
            "f",
            CssComponentValues::try_new(vec![
                CssComponentValue::try_function("var", components("--x")).unwrap(),
            ])
            .unwrap(),
        )
        .unwrap(),
    ])
    .unwrap();
    let error = display("env(x)")
        .reparse_after_substitution(replacement)
        .unwrap_err();
    assert_eq!(error.kind(), &FontFailure::ResidualSubstitution);
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );
    // attr is not a universal residual-family classifier in these descriptors;
    // the selected ordinary display/index grammar owns its rejection.
    font_grammar(
        display("env(x)")
            .reparse_after_substitution(components("attr(x)"))
            .unwrap_err(),
    );
    font_grammar(
        feature(FeatureKind::Styleset, "env(x)")
            .reparse_after_substitution(components("attr(x)"))
            .unwrap_err(),
    );
}

#[test]
fn grammar_errors_keep_parsed_or_programmatic_responsible_tokens_and_empty_has_no_token() {
    let a = Counter::try_from_components(CounterKind::Prefix, components("1")).unwrap_err();
    let b = Display::try_from_components(components("banana")).unwrap_err();
    let c = Feature::try_from_components(FeatureKind::Styleset, components("1.0")).unwrap_err();
    assert!(matches!(a.kind(), CounterFailure::Grammar(_)));
    assert!(matches!(b.kind(), FontFailure::Grammar(_)));
    assert!(matches!(c.kind(), FontFailure::Grammar(_)));
    for (origin, source, end) in [
        (a.origin(), "1", 1),
        (b.origin(), "banana", 6),
        (c.origin(), "1.0", 3),
    ] {
        let CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) = origin else {
            panic!("responsible parsed token")
        };
        point(origin, source, 0, end);
    }
    let a = Counter::try_from_components(
        CounterKind::Prefix,
        CssComponentValues::try_new(vec![CssComponentValue::try_number("1").unwrap()]).unwrap(),
    )
    .unwrap_err();
    let b = Display::try_from_components(
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("banana").unwrap()]).unwrap(),
    )
    .unwrap_err();
    let c = Feature::try_from_components(
        FeatureKind::Styleset,
        CssComponentValues::try_new(vec![CssComponentValue::try_string("x").unwrap()]).unwrap(),
    )
    .unwrap_err();
    for origin in [a.origin(), b.origin(), c.origin()] {
        assert_eq!(
            origin,
            &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
        );
    }
    counter_grammar(a);
    font_grammar(b);
    font_grammar(c);
    let a = Counter::try_from_components(CounterKind::Prefix, components("")).unwrap_err();
    let b = Display::try_from_components(components("")).unwrap_err();
    let c = Feature::try_from_components(FeatureKind::Styleset, components("")).unwrap_err();
    for origin in [a.origin(), b.origin(), c.origin()] {
        assert_eq!(origin, &CssSerializedOrigin::End(None));
    }
    counter_grammar(a);
    font_grammar(b);
    font_grammar(c);
}

#[test]
fn invalid_or_empty_replacements_and_cardinality_fail_without_consuming_the_pending_phase() {
    let raw = parse_counter_style_descriptor_value("env(x)", CounterKind::Prefix);
    let a = raw.syntax().as_ref().unwrap();
    let before = a.clone();
    for source in ["", "1", "\"x\" \"y\""] {
        counter_grammar(
            a.reparse_after_substitution(components(source))
                .unwrap_err(),
        );
    }
    for source in ["\"x\"", "\"y\""] {
        let completed = a.reparse_after_substitution(components(source)).unwrap();
        ordinary_counter(&completed, CounterKind::Prefix, source);
        assert!(completed.origin().is_none());
        assert_eq!(a, &before);
    }
    assert!(a.origin().is_some());
    let b = display("env(x)");
    let before = b.clone();
    for source in ["", "banana", "swap block"] {
        font_grammar(
            b.reparse_after_substitution(components(source))
                .unwrap_err(),
        );
    }
    assert!(matches!(
        b.reparse_after_substitution(components("swap"))
            .unwrap()
            .view(),
        DisplayView::Ordinary(CssFontDisplay::Swap)
    ));
    assert!(matches!(
        b.reparse_after_substitution(components("block"))
            .unwrap()
            .view(),
        DisplayView::Ordinary(CssFontDisplay::Block)
    ));
    assert_eq!(b, before);
    let c = feature(FeatureKind::CharacterVariant, "env(x)");
    let before = c.clone();
    for source in ["", "1.0", "-1", "1 2 3"] {
        font_grammar(
            c.reparse_after_substitution(components(source))
                .unwrap_err(),
        );
    }
    ordinary_feature(
        &c.reparse_after_substitution(components("1 2")).unwrap(),
        FeatureKind::CharacterVariant,
        &["1", "2"],
    );
    ordinary_feature(
        &c.reparse_after_substitution(components("3")).unwrap(),
        FeatureKind::CharacterVariant,
        &["3"],
    );
    assert_eq!(c, before);
}

#[test]
fn definitions_and_display_occurrences_use_checked_values_without_manufactured_name_positions() {
    for (kind, _, _) in FEATURE_ORDINARY {
        let definition = CssFontFeatureValueDefinition::new(
            CssFontFeatureValueName::try_new("Pick").unwrap(),
            feature(kind, "env(x)"),
        );
        assert_eq!(definition.name().as_str(), "Pick");
        assert_eq!(definition.value().kind(), kind);
        assert!(matches!(definition.value().view(), FeatureView::Pending(_)));
        assert!(definition.position().is_none());
        assert!(definition.parsed_name().is_none());
        let block =
            CssFontFeatureValueBlock::try_new(kind, vec![definition.clone(), definition]).unwrap();
        assert_eq!(block.kind(), kind);
        assert_eq!(block.definitions().len(), 2);
    }
    let display = CssFontFeatureDisplayOccurrence::new(display("env(x)"));
    assert!(matches!(display.value().view(), DisplayView::Pending(_)));
    assert!(display.position().is_none());
    assert!(display.parsed_name().is_none());
    let mismatch = CssFontFeatureValueDefinition::new(
        CssFontFeatureValueName::try_new("Pick").unwrap(),
        feature(FeatureKind::Styleset, "env(x)"),
    );
    let error = CssFontFeatureValueBlock::try_new(FeatureKind::Swash, vec![mismatch]).unwrap_err();
    assert_eq!(error.kind(), CssFontFeatureValuesErrorKind::KindMismatch);
    assert_eq!(error.block(), Some(FeatureKind::Swash));
    assert_eq!(error.definition_index(), Some(0));
}

#[test]
fn parsed_body_values_keep_actual_name_windows_and_later_pending_effective_occurrences() {
    let source = "{prefix:\"a\";prefix:env(x);suffix:\"b\"}";
    let report = parse_counter_style_block(source);
    assert!(report.is_clean(), "{report:?}");
    let descriptors = report.syntax().as_ref().unwrap().body();
    let occurrences = descriptors.occurrences().collect::<Vec<_>>();
    let [
        CounterOccurrence::Prefix(first),
        CounterOccurrence::Prefix(last),
        CounterOccurrence::Suffix(_),
    ] = occurrences.as_slice()
    else {
        panic!("ordered source occurrences")
    };
    assert!(matches!(first.value().view(), CounterView::Prefix(_)));
    assert!(matches!(last.value().view(), CounterView::Pending(_)));
    assert_eq!(last.value().kind(), CounterKind::Prefix);
    let effective = descriptors.prefix().unwrap();
    assert_eq!(effective, *last);
    assert!(
        effective
            .parsed_value()
            .unwrap()
            .source()
            .same_snapshot(last.parsed_value().unwrap().source())
    );
    point(last.parsed_name().unwrap(), source, 12, 18);
    point(last.parsed_value().unwrap(), source, 19, 25);
    point(last.value().origin().unwrap(), source, 19, 25);
    assert!(
        last.parsed_value()
            .unwrap()
            .source()
            .same_snapshot(last.value().origin().unwrap().source())
    );
    let source = "{font-display:env(x);@styleset{Pick:env(y)}}";
    let report = parse_font_feature_values_block(source);
    assert!(report.is_clean(), "{report:?}");
    let [Item::FontDisplay(display), Item::Block(block)] =
        report.syntax().as_ref().unwrap().body().as_slice()
    else {
        panic!("real mixed body")
    };
    point(display.parsed_name().unwrap(), source, 1, 13);
    point(display.value().origin().unwrap(), source, 14, 20);
    assert!(matches!(display.value().view(), DisplayView::Pending(_)));
    let [definition] = block.definitions() else {
        panic!("one definition")
    };
    point(definition.parsed_name().unwrap(), source, 31, 35);
    point(definition.value().origin().unwrap(), source, 36, 42);
    assert!(matches!(definition.value().view(), FeatureView::Pending(_)));
    assert_eq!(definition.value().kind(), FeatureKind::Styleset);
}

#[test]
fn pending_system_does_not_invent_an_ordinary_system_and_extends_still_forbids_symbols() {
    let accepted = parse_counter_style_block("{system:env(x);symbols:a b}");
    assert!(accepted.is_clean(), "{accepted:?}");
    let system = accepted.syntax().as_ref().unwrap().body().system().unwrap();
    assert!(matches!(system.value().view(), CounterView::Pending(_)));
    let rejected = parse_counter_style_block("{system:extends decimal;symbols:env(x)}");
    assert!(!rejected.is_clean());
    assert!(rejected.syntax().is_none());
}

#[test]
fn value_output_uses_cumulative_byte_and_node_limits_without_mutating_phase() {
    let a = counter(CounterKind::Prefix, "env(x)");
    let b = display("env(x)");
    let c = feature(FeatureKind::Styleset, "env(x)");
    for (limits, expected) in [
        (
            EmitLimits::new(0, usize::MAX, usize::MAX),
            EmitFailure::InputNodeLimit,
        ),
        (
            EmitLimits::new(usize::MAX, 0, usize::MAX),
            EmitFailure::ProjectionNodeLimit,
        ),
        (
            EmitLimits::new(usize::MAX, usize::MAX, 5),
            EmitFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            a.to_specified_css_with_limits(limits).unwrap_err().kind(),
            expected
        );
        assert_eq!(
            b.to_specified_css_with_limits(limits).unwrap_err().kind(),
            expected
        );
        assert_eq!(
            c.to_specified_css_with_limits(limits).unwrap_err().kind(),
            expected
        );
    }
    let exact = EmitLimits::new(usize::MAX, usize::MAX, 6);
    assert_eq!(a.to_specified_css_with_limits(exact).unwrap(), "env(x)");
    assert_eq!(b.to_specified_css_with_limits(exact).unwrap(), "env(x)");
    assert_eq!(c.to_specified_css_with_limits(exact).unwrap(), "env(x)");
    assert!(matches!(a.view(), CounterView::Pending(_)));
    assert!(matches!(b.view(), DisplayView::Pending(_)));
    assert!(matches!(c.view(), FeatureView::Pending(_)));
}

#[test]
fn suppressed_pending_occurrences_consume_complete_work_even_when_only_later_values_emit() {
    // Sixty-four literal fallback identifier tokens require at least sixty-four
    // visits; a sixteen-node budget cannot omit them. The simple last-value
    // controls fit sixteen under the established ordinary writer contracts.
    let fallback = (0..64).map(|_| "x").collect::<Vec<_>>().join(" ");
    for (limits, expected) in [
        (
            EmitLimits::new(16, usize::MAX, 1_000),
            EmitFailure::InputNodeLimit,
        ),
        (
            EmitLimits::new(usize::MAX, 16, 1_000),
            EmitFailure::ProjectionNodeLimit,
        ),
    ] {
        let simple = parse_sheet("@counter-style demo{prefix:\"ok\"}");
        let [CssRule::CounterStyle(rule)] = simple.syntax().rules() else {
            panic!("counter")
        };
        assert_eq!(
            rule.descriptor_specified_css_with_limits(CounterKind::Prefix, limits)
                .unwrap(),
            "\"ok\""
        );
        let source = format!("@counter-style demo{{prefix:env(x,{fallback});prefix:\"ok\"}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{report:?}");
        let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
            panic!("counter")
        };
        assert_eq!(
            rule.descriptor_specified_css(CounterKind::Prefix).unwrap(),
            "\"ok\""
        );
        assert_eq!(
            rule.descriptor_specified_css_with_limits(CounterKind::Prefix, limits)
                .unwrap_err()
                .kind(),
            expected
        );
        let simple = parse_sheet("@font-feature-values Demo{@styleset{Pick:1}}");
        assert!(
            simple.syntax().rules()[0]
                .to_specified_css_with_limits(limits)
                .is_ok()
        );
        let source =
            format!("@font-feature-values Demo{{@styleset{{Pick:env(x,{fallback});Pick:1}}}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{report:?}");
        let rule = &report.syntax().rules()[0];
        assert_eq!(
            rule.to_specified_css().unwrap(),
            "@font-feature-values Demo { @styleset { Pick: 1; } }"
        );
        assert_eq!(
            rule.to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            RuleEmitFailure::Resource(expected)
        );
    }
}

#[test]
fn matched_root_blocks_are_declaration_value_data_under_whole_env_deferral() {
    for source in ["env(x) {}", "{ x } env(x)", "env(x, f(!important))"] {
        assert!(matches!(
            counter(CounterKind::Prefix, source).view(),
            CounterView::Pending(_)
        ));
        assert!(matches!(display(source).view(), DisplayView::Pending(_)));
        assert!(matches!(
            feature(FeatureKind::Styleset, source).view(),
            FeatureView::Pending(_)
        ));
    }
    counter_grammar(
        Counter::try_from_components(CounterKind::Prefix, components("env(x)!important"))
            .unwrap_err(),
    );
    font_grammar(Display::try_from_components(components("env(x)!important")).unwrap_err());
    font_grammar(
        Feature::try_from_components(FeatureKind::Styleset, components("env(x)!important"))
            .unwrap_err(),
    );
}

#[test]
fn normalization_keeps_pending_typed_payloads_in_their_actual_parent_context() {
    let source = "@media all{@counter-style demo{prefix:env(x)}@font-feature-values Demo{@styleset{Pick:env(x)}}}";
    let parsed = parse_sheet(source);
    assert!(parsed.is_clean(), "{parsed:?}");
    let normalized = normalize_report(&parsed).unwrap();
    let [
        CssNormalizedItem::Rule(parent),
        CssNormalizedItem::Rule(counter),
        CssNormalizedItem::Rule(font),
    ] = normalized.syntax().items()
    else {
        panic!("group and two actual children")
    };
    assert!(counter.parent().unwrap().same_context(parent));
    assert!(font.parent().unwrap().same_context(parent));
    let CssRuleContextKindRef::CounterStyle(counter) = counter.kind() else {
        panic!("counter owner")
    };
    assert!(matches!(
        counter.descriptors().prefix().unwrap().value().view(),
        CounterView::Pending(_)
    ));
    let CssRuleContextKindRef::FontFeatureValues(font) = font.kind() else {
        panic!("font owner")
    };
    let [Item::Block(block)] = font.items() else {
        panic!("styleset")
    };
    assert!(matches!(
        block.definitions()[0].value().view(),
        FeatureView::Pending(_)
    ));
}
