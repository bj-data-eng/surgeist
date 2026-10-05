#![forbid(unsafe_code)]
//! Transitions 1 (2026-01-08), §§2.1–2.5, defines four noninherited
//! longhands, their initials, and one list entry per shorthand item.
//! Cascade 5 (2022-01-13), §3.1, defines omission defaults, CSS-wide
//! propagation and shorthand importance.
//! https://www.w3.org/TR/2026/WD-css-transitions-1-20260108/#transition-shorthand-property
//! https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/#shorthand
//!
//! Uses callable public APIs at 0ec18c1388dbccee62e20c0386d3b3a908d37750.
//! No future borrowed longhand variants are referenced. Generic owned-value
//! equality compares projections with independently authored explicit lists.
//! Duration and delay list equality compares retained structure without source
//! origins; raw time literal/component equality includes origins. Source occurrence
//! and replacement-component provenance are asserted separately. Direct projected
//! time-child origin inspection requires borrowed variants added with implementation.
//! The standard determines members/defaults. The adopted Surgeist contribution
//! order follows shorthand slots: property, duration, timing-function, delay.
//! This intrinsic contribution order is a public contract, not a standard mandate.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const MEMBERS: [P; 4] = [
    P::TransitionProperty,
    P::TransitionDuration,
    P::TransitionTimingFunction,
    P::TransitionDelay,
];

const LONGHANDS: [(P, &str); 4] = [
    (P::TransitionProperty, "opacity, transform"),
    (P::TransitionDuration, "1s, 2s"),
    (P::TransitionTimingFunction, "ease-in, linear"),
    (P::TransitionDelay, "-250ms, 0s"),
];

const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];

fn parsed(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one retained declaration for {css}")
    };
    source.clone()
}

fn checked(property: P, css: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        property.grammar(),
        parse_component_values(css).unwrap(),
        CssImportance::Important,
    )
    .unwrap()
}

fn retained_recovered(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(!report.is_clean(), "recovered fixture: {css}");
    assert!(
        report
            .diagnostics()
            .iter()
            .any(
                |diagnostic| diagnostic.error().code() == CssErrorCode::UnexpectedEnd
                    && diagnostic.action() == CssRecoveryAction::RetainWithImplicitClosure
            )
    );
    let [source] = report.syntax().as_slice() else {
        panic!("one retained recovered declaration for {css}")
    };
    assert!(source.known().unwrap().property_value().is_some());
    source.clone()
}

fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("intrinsic contributions must be available")
    else {
        panic!("completed terminal contributions")
    };
    values
}

fn member_names(values: &CssLonghandContributions) -> Vec<P> {
    values
        .items()
        .iter()
        .map(CssLonghandContribution::property)
        .collect()
}

fn assert_source(item: &CssLonghandContribution, source: &CssDeclaration) {
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(
        item.source().known().unwrap().grammar(),
        source.known().unwrap().grammar()
    );
    assert_eq!(item.source().value_components(), source.value_components());
    assert_eq!(item.source().position(), source.position());
}

fn assert_ordinary(item: &CssLonghandContribution, source: &CssDeclaration) {
    assert_source(item, source);
    assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
    assert_eq!(
        item.ordinary_value().unwrap().property().known_property(),
        item.property()
    );
}

fn assert_projected_lists(values: &CssLonghandContributions, expected: [&str; 4]) {
    assert_eq!(member_names(values), MEMBERS);
    for (item, (property, css)) in values.items().iter().zip(MEMBERS.into_iter().zip(expected)) {
        let explicit = checked(property, css);
        let expected_values = completed(&explicit);
        let [expected_item] = expected_values.items() else {
            panic!("one terminal for an explicit longhand")
        };
        // CssDurationList compares CssDuration's structural equality; CssDelayList
        // also compares structural time values. Distinct parsed origins therefore
        // do not invalidate this independent content oracle. Exact raw origins
        // remain observable, and are not inferred from ordinary-value equality.
        assert_eq!(
            item.ordinary_value(),
            expected_item.ordinary_value(),
            "{}: {css}",
            property.canonical_name()
        );
    }
}

fn assert_metadata(property: P) {
    let metadata = property.metadata().expect("intrinsic longhand metadata");
    assert_eq!(metadata.grammar(), property.grammar());
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("timing property is a longhand")
    };
    assert_eq!(longhand.property().known_property(), property);
    assert!(!longhand.inherited_by_default());
    let initial = longhand.initial_value();
    assert_eq!(initial.property().known_property(), property);
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("context-independent intrinsic initial")
    };
    assert_eq!(initial.property().known_property(), property);
}

#[test]
fn supported_color_control_preserves_symbolic_initial_and_occurrence() {
    let CssPropertyKindRef::Longhand(metadata) = P::Color.metadata().unwrap().kind() else {
        panic!("color longhand")
    };
    assert!(metadata.inherited_by_default());
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("symbolic ordinary color initial")
    };
    let CssLonghandValueRef::Color(color) = initial.view() else {
        panic!("existing color variant")
    };
    assert_eq!(color.system(), Some(CssSystemColor::CanvasText));
    let source = parsed("/*😀*/color:currentcolor!important");
    let values = completed(&source);
    let [item] = values.items() else {
        panic!("one color terminal")
    };
    assert_eq!(item.property(), P::Color);
    assert_ordinary(item, &source);
    assert!(item.replacement_components().is_none());
}

#[test]
fn supported_background_control_expands_omissions_without_losing_importance() {
    let source = parsed("background:red!important");
    let values = completed(&source);
    let expected = [
        P::BackgroundImage,
        P::BackgroundPosition,
        P::BackgroundSize,
        P::BackgroundRepeat,
        P::BackgroundAttachment,
        P::BackgroundOrigin,
        P::BackgroundClip,
        P::BackgroundColor,
    ];
    assert_eq!(member_names(&values), expected);
    for item in values.items() {
        assert_ordinary(item, &source);
        assert!(item.replacement_components().is_none());
    }
    let red = completed(&checked(P::BackgroundColor, "red"));
    assert_eq!(
        values.items()[7].ordinary_value(),
        red.items()[0].ordinary_value()
    );
}

#[test]
fn transition_property_metadata_is_noninherited_and_initial_identity_is_coupled() {
    assert_metadata(P::TransitionProperty);
}

#[test]
fn transition_duration_metadata_is_noninherited_and_initial_identity_is_coupled() {
    assert_metadata(P::TransitionDuration);
}

#[test]
fn transition_easing_metadata_is_noninherited_and_initial_identity_is_coupled() {
    assert_metadata(P::TransitionTimingFunction);
}

#[test]
fn transition_delay_metadata_is_noninherited_and_initial_identity_is_coupled() {
    assert_metadata(P::TransitionDelay);
}

#[test]
fn transition_metadata_lists_four_settable_terminals_without_extra_resets() {
    let metadata = P::Transition
        .metadata()
        .expect("transition shorthand metadata");
    assert_eq!(metadata.grammar(), P::Transition.grammar());
    let CssPropertyKindRef::Shorthand(shorthand) = metadata.kind() else {
        panic!("transition is a shorthand")
    };
    let names = |members: &[CssLonghandProperty]| {
        members
            .iter()
            .map(|p| p.known_property())
            .collect::<Vec<_>>()
    };
    assert_eq!(names(shorthand.members()), MEMBERS);
    assert_eq!(names(shorthand.settable_members()), MEMBERS);
    assert!(shorthand.reset_only_members().is_empty());
    assert!(!shorthand.is_legacy());
}

#[test]
fn parsed_and_checked_longhands_expand_once_with_original_components() {
    for (property, css) in LONGHANDS {
        let authored = parsed(&format!(
            "/*😀*/{}:{css}!important",
            property.canonical_name()
        ));
        let constructed = checked(property, css);
        assert!(authored.position().is_some());
        assert!(constructed.position().is_none());
        assert!(!authored.same_occurrence(&constructed));
        for source in [&authored, &constructed] {
            let values = completed(source);
            assert_eq!(member_names(&values), [property]);
            let item = &values.items()[0];
            assert_ordinary(item, source);
            assert!(item.replacement_components().is_none());
        }
        assert_eq!(
            completed(&authored).items()[0].ordinary_value(),
            completed(&constructed).items()[0].ordinary_value()
        );
    }
}

#[test]
fn explicit_shorthand_keeps_list_order_and_negative_delay_without_evaluation() {
    let css = "opacity 1s ease-in -250ms, transform 2s linear 0s";
    for source in [
        parsed(&format!("transition:{css}!important")),
        checked(P::Transition, css),
    ] {
        let values = completed(&source);
        assert_projected_lists(
            &values,
            [
                "opacity, transform",
                "1s, 2s",
                "ease-in, linear",
                "-250ms, 0s",
            ],
        );
        for item in values.items() {
            assert_ordinary(item, &source);
            assert!(item.replacement_components().is_none());
        }
    }
}

#[test]
fn sparse_multi_item_shorthands_fill_each_omitted_list_slot() {
    for (css, expected) in [
        (
            "opacity 1s, transform ease-in",
            ["opacity, transform", "1s, 0s", "ease, ease-in", "0s, 0s"],
        ),
        (
            "1s, transform 2s -250ms",
            ["all, transform", "1s, 2s", "ease, ease", "0s, -250ms"],
        ),
        (
            "opacity, transform",
            ["opacity, transform", "0s, 0s", "ease, ease", "0s, 0s"],
        ),
    ] {
        let source = checked(P::Transition, css);
        let values = completed(&source);
        assert_projected_lists(&values, expected);
        for item in values.items() {
            assert_ordinary(item, &source);
        }
    }
}

#[test]
fn singleton_none_retains_timing_fields_and_defaults_other_slots() {
    let source = checked(P::Transition, "none 1s -250ms");
    assert_projected_lists(&completed(&source), ["none", "1s", "ease", "-250ms"]);
}

#[test]
fn recovered_shorthand_time_projects_without_strict_reconstruction() {
    for (css, member_index, expected_css) in [
        (
            "transition:opacity calc(1s",
            1,
            "transition-duration:calc(1s",
        ),
        (
            "transition:opacity 1s calc(-250ms",
            3,
            "transition-delay:calc(-250ms",
        ),
    ] {
        let source = retained_recovered(css);
        let CssKnownPropertyValueRef::Transition(wrapper) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("retained transition model")
        };
        assert!(CssTransitionList::try_new(wrapper.transitions().values().to_vec()).is_none());
        let transition = &wrapper.transitions().values()[0];
        let time = if member_index == 1 {
            transition.duration().unwrap().time()
        } else {
            transition.delay().unwrap()
        };
        let CssComponentValueRef::Function(function) =
            time.calculation().unwrap().components().items()[0].view()
        else {
            panic!("retained time calculation")
        };
        assert!(matches!(
            function.closing_origin(),
            CssValueOrigin::ImplicitClosure { .. }
        ));
        let values = completed(&source);
        assert_eq!(member_names(&values), MEMBERS);
        for item in values.items() {
            assert_ordinary(item, &source);
            assert!(item.replacement_components().is_none());
        }
        // The recovered explicit longhand is an independent structural payload
        // oracle. This does not prove raw projected child-origin identity, which
        // must be inspected using the new borrowed variants with implementation.
        let expected = completed(&retained_recovered(expected_css));
        assert_eq!(
            values.items()[member_index].ordinary_value(),
            expected.items()[0].ordinary_value()
        );
    }
}

#[test]
fn all_five_css_wide_values_propagate_to_each_timing_longhand() {
    for property in MEMBERS {
        for (css, keyword) in GLOBALS {
            let source = checked(property, css);
            let values = completed(&source);
            assert_eq!(member_names(&values), [property]);
            let item = &values.items()[0];
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert!(item.ordinary_value().is_none());
            assert_source(item, &source);
        }
    }
}

#[test]
fn all_five_css_wide_shorthands_set_exactly_four_ordered_globals() {
    for (css, keyword) in GLOBALS {
        let source = parsed(&format!("transition:{css}!important"));
        let values = completed(&source);
        assert_eq!(member_names(&values), MEMBERS);
        for item in values.items() {
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert!(item.ordinary_value().is_none());
            assert_source(item, &source);
            assert!(item.replacement_components().is_none());
        }
    }
}

#[test]
fn universal_reset_includes_timing_targets_without_resolving_the_full_inventory() {
    let source = parsed("all:revert-layer!important");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("symbolic universal reset")
    };
    assert_eq!(reset.keyword(), CssGlobalKeyword::RevertLayer);
    assert!(reset.source().same_occurrence(&source));
    assert_eq!(reset.source().importance(), CssImportance::Important);
    for property in MEMBERS {
        assert!(!reset.excludes(CssPropertyNameRef::Known(property)));
    }
    for property in [P::Direction, P::UnicodeBidi] {
        assert!(reset.excludes(CssPropertyNameRef::Known(property)));
    }
    let custom = CssCustomPropertyName::try_new("--timing").unwrap();
    assert!(reset.excludes(CssPropertyNameRef::Custom(&custom)));
}

#[test]
fn every_timing_longhand_reenters_atomically_and_keeps_the_original_grammar() {
    for (property, css) in LONGHANDS {
        let source = checked(property, "var(--timing)");
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("whole-value pending longhand")
        };
        assert!(handle.source().same_occurrence(&source));
        let replacement = parse_component_values(css).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed longhand replacement")
            };
            assert_eq!(member_names(&values), [property]);
            let item = &values.items()[0];
            assert_ordinary(item, &source);
            assert_eq!(item.replacement_components(), Some(&replacement));
            let explicit = completed(&checked(property, css));
            assert_eq!(item.ordinary_value(), explicit.items()[0].ordinary_value());
        }
    }
}

#[test]
fn shorthand_reentry_is_reusable_after_grammar_and_residual_substitution_failures() {
    let source = parsed("/*😀*/transition:var(--timing)!important");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("whole-value pending shorthand")
    };
    let replacement = parse_component_values("opacity 1s, transform ease-in").unwrap();
    for invalid in [
        "",
        "opacity -1s",
        "opacity 1s 2s 3s",
        "opacity 1s!important",
        "opacity 1s;color:red",
    ] {
        let components = parse_component_values(invalid).unwrap();
        let expected = parse_property_value_for_grammar(
            P::Transition.grammar(),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap_err();
        let error = handle.reenter(components).unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
            panic!("original-grammar failure")
        };
        assert_eq!(actual, &expected);
        assert!(handle.source().same_occurrence(&source));
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("valid retry")
        };
        assert_projected_lists(
            &values,
            ["opacity, transform", "1s, 0s", "ease, ease-in", "0s, 0s"],
        );
    }
    for residual in [
        "var(--again)",
        "env(timing)",
        "attr(timing)",
        "opacity calc(var(--again) + 1s)",
    ] {
        assert_eq!(
            handle
                .reenter(parse_component_values(residual).unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("valid retry after residual")
        };
        assert_eq!(member_names(&values), MEMBERS);
    }
    for _ in 0..2 {
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("completed shorthand")
        };
        for item in values.items() {
            assert_ordinary(item, &source);
            assert_eq!(item.replacement_components(), Some(&replacement));
            for (actual, expected) in item
                .replacement_components()
                .unwrap()
                .items()
                .iter()
                .zip(replacement.items())
            {
                assert_eq!(actual.origin(), expected.origin());
                if let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(expected)) =
                    (actual.origin(), expected.origin())
                {
                    assert!(actual.source().same_snapshot(expected.source()));
                }
            }
        }
    }
    let CssContributions::Longhands(values) = handle
        .reenter(parse_component_values("unset").unwrap())
        .unwrap()
    else {
        panic!("global replacement")
    };
    assert_eq!(member_names(&values), MEMBERS);
    for item in values.items() {
        assert_eq!(
            item.value(),
            CssContributionValueRef::Global(CssGlobalKeyword::Unset)
        );
    }
}

#[test]
fn recovered_replacement_is_rejected_without_consuming_pending_handle() {
    let source = parsed("transition:var(--timing)!important");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending transition")
    };
    for css in ["opacity calc(1s", "opacity 1s calc(-250ms"] {
        let recovered = parse_component_values(css).unwrap();
        let expected = parse_property_value_for_grammar(
            P::Transition.grammar(),
            recovered.clone(),
            CssImportance::Normal,
        )
        .unwrap_err();
        let error = handle.reenter(recovered).unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
            panic!("strict original-grammar admission")
        };
        assert_eq!(actual, &expected);
        assert!(handle.source().same_occurrence(&source));
        let replacement = parse_component_values("opacity 1s, transform ease-in").unwrap();
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("valid retry after recovered replacement")
        };
        assert_projected_lists(
            &values,
            ["opacity, transform", "1s, 0s", "ease, ease-in", "0s, 0s"],
        );
        for item in values.items() {
            assert_ordinary(item, &source);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn ordered_normalization_preserves_timing_groups_and_exact_contribution_budget() {
    let css = ".a{color:red;transition:opacity 1s,transform ease-in!important;transition-delay:-250ms}.b{transition:var(--timing);all:unset;--timing:opacity 1s}";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    // Completed shorthand costs four members regardless of its two authored
    // items. Each longhand/custom/universal/pending declaration costs one.
    let exact = CssNormalizationLimits::try_new(0, 2, 6, 9).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 6);
    for (index, item) in declarations.iter().enumerate() {
        assert_eq!(item.order(), index);
    }
    for (item, property) in declarations[..5].iter().zip([
        P::Color,
        P::Transition,
        P::TransitionDelay,
        P::Transition,
        P::All,
    ]) {
        assert_eq!(item.source().known().unwrap().property(), property);
    }
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        declarations[1].expansion()
    else {
        panic!("normalized shorthand")
    };
    assert_projected_lists(
        values,
        ["opacity, transform", "1s, 0s", "ease, ease-in", "0s, 0s"],
    );
    for item in values.items() {
        assert_ordinary(item, declarations[1].source());
    }
    assert_eq!(
        declarations[1].source().importance(),
        CssImportance::Important
    );
    assert_eq!(
        declarations[1]
            .source()
            .position()
            .unwrap()
            .byte_offset()
            .value(),
        css.find("transition:").unwrap()
    );
    let CssExpansion::Pending(handle) = declarations[3].expansion() else {
        panic!("normalized pending shorthand")
    };
    assert!(handle.source().same_occurrence(declarations[3].source()));
    assert!(matches!(
        declarations[4].expansion(),
        CssExpansion::Contributions(CssContributions::UniversalReset(_))
    ));
    assert!(matches!(
        declarations[5].expansion(),
        CssExpansion::Contributions(CssContributions::Custom(_))
    ));
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 2, 6, 8).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 8
        }
    );
    assert_eq!(error.declaration_order(), Some(5));
    assert!(
        error
            .declaration()
            .unwrap()
            .same_occurrence(declarations[5].source())
    );
    assert_eq!(report.diagnostics(), before.diagnostics());
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}

#[test]
fn a_sparse_shorthand_admits_four_members_and_fails_atomically_at_three() {
    let report = parse_sheet(".a{transition:opacity 1s,transform ease-in!important}");
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let exact = CssNormalizationLimits::try_new(0, 1, 1, 4).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let source = normalized
        .items()
        .iter()
        .find_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value.source()),
            _ => None,
        })
        .unwrap();
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(0, 1, 1, 3).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 3
        }
    );
    assert_eq!(error.declaration_order(), Some(0));
    assert!(error.declaration().unwrap().same_occurrence(source));
    assert_eq!(
        error.declaration().unwrap().importance(),
        CssImportance::Important
    );
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}

#[test]
fn recovered_time_normalization_preserves_diagnostics_and_contribution_budget() {
    for css in [
        ".a{transition:opacity calc(1s",
        ".a{transition:opacity 1s calc(-250ms",
    ] {
        let report = parse_sheet(css);
        assert!(!report.is_clean());
        let before = report.clone();
        let exact = CssNormalizationLimits::try_new(0, 1, 1, 4).unwrap();
        let normalized = normalize_report_with_limits(&report, exact).unwrap();
        assert!(!normalized.is_clean());
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let declarations: Vec<_> = normalized
            .syntax()
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect();
        let [declaration] = declarations.as_slice() else {
            panic!("one retained transition occurrence")
        };
        assert_eq!(declaration.order(), 0);
        assert_eq!(
            declaration.source().known().unwrap().property(),
            P::Transition
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declaration.expansion()
        else {
            panic!("ordinary recovered transition contributions")
        };
        assert_eq!(member_names(values), MEMBERS);
        for item in values.items() {
            assert_ordinary(item, declaration.source());
        }
        let error = normalize_report_with_limits(
            &report,
            CssNormalizationLimits::try_new(0, 1, 1, 3).unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded {
                resource: CssNormalizationResource::Contributions,
                limit: 3
            }
        );
        assert_eq!(error.declaration_order(), Some(0));
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(declaration.source())
        );
        assert_eq!(report.diagnostics(), before.diagnostics());
        assert!(normalize_report_with_limits(&report, exact).is_ok());
    }
}
