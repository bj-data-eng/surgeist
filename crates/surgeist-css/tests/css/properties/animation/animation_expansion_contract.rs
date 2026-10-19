#![forbid(unsafe_code)]
//! Animations 1 WD 2023-03-02 §§3.2–3.10 defines eight noninherited
//! initials, per-item shorthand defaults, time-slot and keyword-slot priority.
//! Easing 1 CRD 2023-02-13 supplies the selected easing grammar.
//! Cascade 5 CR 2022-01-13 §3.1 supplies shorthand global/importance propagation.
//! The adopted intrinsic contribution order follows shorthand slots and current
//! specified output: duration, timing-function, delay, count, direction, fill,
//! play-state, name. Membership/defaults are independent standard oracles.
//! Existing generic owned-payload equality compares retained structure without
//! time/count origins. Original/replacement provenance is asserted separately.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const MEMBERS: [P; 8] = [
    P::AnimationDuration,
    P::AnimationTimingFunction,
    P::AnimationDelay,
    P::AnimationIterationCount,
    P::AnimationDirection,
    P::AnimationFillMode,
    P::AnimationPlayState,
    P::AnimationName,
];
const INITIALS: [&str; 8] = ["0s", "ease", "0s", "1", "normal", "none", "running", "none"];
const ORDINARY: [&str; 8] = [
    "1s, 2s",
    "ease-in, linear",
    "-250ms, 0s",
    "2.5, infinite",
    "reverse, alternate",
    "forwards, both",
    "paused, running",
    "fade, slide",
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
        panic!("one retained declaration")
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
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("available intrinsic contributions")
    else {
        panic!("completed terminal contributions")
    };
    values
}
fn names(values: &CssLonghandContributions) -> Vec<P> {
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
    assert_eq!(item.source().position(), source.position());
    assert_eq!(item.source().value_components(), source.value_components());
}
fn assert_ordinary(item: &CssLonghandContribution, source: &CssDeclaration) {
    assert_source(item, source);
    assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
    assert_eq!(
        item.ordinary_value().unwrap().property().known_property(),
        item.property()
    );
}
fn assert_payloads(values: &CssLonghandContributions, expected: [&str; 8]) {
    assert_eq!(names(values), MEMBERS);
    for (item, (property, css)) in values.items().iter().zip(MEMBERS.into_iter().zip(expected)) {
        let explicit = completed(&checked(property, css));
        assert_eq!(explicit.items().len(), 1);
        assert_eq!(
            item.ordinary_value(),
            explicit.items()[0].ordinary_value(),
            "{}: {css}",
            property.canonical_name()
        );
    }
}
fn assert_initial(index: usize) {
    let property = MEMBERS[index];
    let metadata = property.metadata().expect("intrinsic animation metadata");
    assert_eq!(metadata.grammar(), property.grammar());
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("longhand")
    };
    assert_eq!(longhand.property().known_property(), property);
    assert!(!longhand.inherited_by_default());
    let initial = longhand.initial_value();
    assert_eq!(initial.property().known_property(), property);
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("ordinary intrinsic initial")
    };
    let expected = completed(&checked(property, INITIALS[index]));
    assert_eq!(expected.items().len(), 1);
    assert_eq!(Some(initial), expected.items()[0].ordinary_value());
}

#[test]
fn supported_color_control_preserves_ordinary_initial_and_source() {
    let metadata = P::Color.metadata().unwrap();
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("color")
    };
    assert!(longhand.inherited_by_default());
    let initial = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("ordinary symbolic initial")
    };
    let CssLonghandValueRef::Color(color) = initial.view() else {
        panic!("existing color variant")
    };
    assert_eq!(color.system(), Some(CssSystemColor::CanvasText));
    let source = parsed("/*😀*/color:currentcolor!important");
    let values = completed(&source);
    assert_eq!(names(&values), [P::Color]);
    assert_ordinary(&values.items()[0], &source);
}
#[test]
fn universal_reset_control_includes_all_eight_animation_targets() {
    let source = parsed("all:revert-layer!important");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("symbolic all")
    };
    assert_eq!(reset.keyword(), CssGlobalKeyword::RevertLayer);
    assert!(reset.source().same_occurrence(&source));
    for property in MEMBERS {
        assert!(!reset.excludes(CssPropertyNameRef::Known(property)));
    }
    for property in [P::Direction, P::UnicodeBidi] {
        assert!(reset.excludes(CssPropertyNameRef::Known(property)));
    }
    let custom = CssCustomPropertyName::try_new("--animation").unwrap();
    assert!(reset.excludes(CssPropertyNameRef::Custom(&custom)));
}

#[test]
fn duration_metadata_has_one_zero_seconds_noninherited_initial() {
    assert_initial(0);
}
#[test]
fn easing_metadata_has_one_ease_noninherited_initial() {
    assert_initial(1);
}
#[test]
fn delay_metadata_has_one_zero_seconds_noninherited_initial() {
    assert_initial(2);
}
#[test]
fn count_metadata_has_one_noninherited_initial() {
    assert_initial(3);
}
#[test]
fn direction_metadata_has_one_normal_noninherited_initial() {
    assert_initial(4);
}
#[test]
fn fill_metadata_has_one_none_noninherited_initial() {
    assert_initial(5);
}
#[test]
fn play_state_metadata_has_one_running_noninherited_initial() {
    assert_initial(6);
}
#[test]
fn name_metadata_has_one_none_noninherited_initial() {
    assert_initial(7);
}
#[test]
fn shorthand_metadata_sets_exactly_eight_ordered_members_without_extra_resets() {
    let metadata = P::Animation
        .metadata()
        .expect("animation shorthand metadata");
    assert_eq!(metadata.grammar(), P::Animation.grammar());
    let CssPropertyKindRef::Shorthand(shorthand) = metadata.kind() else {
        panic!("shorthand")
    };
    let known = |members: &[CssLonghandProperty]| {
        members
            .iter()
            .map(|property| property.known_property())
            .collect::<Vec<_>>()
    };
    assert_eq!(known(shorthand.members()), MEMBERS);
    assert_eq!(known(shorthand.settable_members()), MEMBERS);
    assert!(shorthand.reset_only_members().is_empty());
    assert!(!shorthand.is_legacy());
    for index in 0..8 {
        assert_initial(index);
    }
}

#[test]
fn every_longhand_expands_parsed_and_checked_values_without_changing_occurrence() {
    for (property, css) in MEMBERS.into_iter().zip(ORDINARY) {
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
            assert_eq!(names(&values), [property]);
            assert_ordinary(&values.items()[0], source);
            assert!(values.items()[0].replacement_components().is_none());
        }
        assert_eq!(
            completed(&authored).items()[0].ordinary_value(),
            completed(&constructed).items()[0].ordinary_value()
        );
    }
}

#[test]
fn explicit_shorthand_projects_eight_lists_with_fractional_count_and_negative_delay() {
    let css = "fade 1s ease-in -250ms 2.5 reverse forwards paused, slide 2s linear 0s infinite alternate both running";
    for source in [
        parsed(&format!("animation:{css}!important")),
        checked(P::Animation, css),
    ] {
        let values = completed(&source);
        assert_payloads(&values, ORDINARY);
        for item in values.items() {
            assert_ordinary(item, &source);
            assert!(item.replacement_components().is_none());
        }
    }
}

#[test]
fn sparse_items_fill_individual_slots_and_preserve_keyword_priority() {
    for (css, expected) in [
        (
            "fade 1s, slide ease-in",
            [
                "1s, 0s",
                "ease, ease-in",
                "0s, 0s",
                "1, 1",
                "normal, normal",
                "none, none",
                "running, running",
                "fade, slide",
            ],
        ),
        (
            "2s linear -250ms 2.5 reverse both paused fade, none",
            [
                "2s, 0s",
                "linear, ease",
                "-250ms, 0s",
                "2.5, 1",
                "reverse, normal",
                "both, none",
                "paused, running",
                "fade, none",
            ],
        ),
        (
            "3s none backwards",
            [
                "3s",
                "ease",
                "0s",
                "1",
                "normal",
                "none",
                "running",
                "backwards",
            ],
        ),
    ] {
        let source = checked(P::Animation, css);
        let values = completed(&source);
        assert_payloads(&values, expected);
        for item in values.items() {
            assert_ordinary(item, &source);
        }
    }
}

#[test]
fn all_five_css_wide_values_propagate_to_every_longhand() {
    for property in MEMBERS {
        for (css, keyword) in GLOBALS {
            let source = checked(property, css);
            let values = completed(&source);
            assert_eq!(names(&values), [property]);
            let item = &values.items()[0];
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert!(item.ordinary_value().is_none());
            assert_source(item, &source);
        }
    }
}
#[test]
fn all_five_css_wide_shorthands_set_eight_ordered_globals() {
    for (css, keyword) in GLOBALS {
        let source = parsed(&format!("animation:{css}!important"));
        let values = completed(&source);
        assert_eq!(names(&values), MEMBERS);
        for item in values.items() {
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert!(item.ordinary_value().is_none());
            assert_source(item, &source);
        }
    }
}

#[test]
fn all_nine_pending_grammars_reenter_repeatedly_with_original_source_and_replacement() {
    for (property, css) in MEMBERS
        .into_iter()
        .zip(ORDINARY)
        .chain([(P::Animation, "fade 1s, slide ease-in")])
    {
        let source = parsed(&format!(
            "/*😀*/{}:var(--animation)!important",
            property.canonical_name()
        ));
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("whole-value pending")
        };
        assert!(handle.source().same_occurrence(&source));
        let replacement = parse_component_values(css).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("terminal reentry")
            };
            if property == P::Animation {
                assert_payloads(
                    &values,
                    [
                        "1s, 0s",
                        "ease, ease-in",
                        "0s, 0s",
                        "1, 1",
                        "normal, normal",
                        "none, none",
                        "running, running",
                        "fade, slide",
                    ],
                );
            } else {
                assert_eq!(names(&values), [property]);
                let explicit = completed(&checked(property, css));
                assert_eq!(
                    values.items()[0].ordinary_value(),
                    explicit.items()[0].ordinary_value()
                );
            }
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
    }
}

#[test]
fn shorthand_pending_handle_survives_grammar_recovery_and_residual_failures() {
    let source = checked(P::Animation, "var(--animation)");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending shorthand")
    };
    let replacement = parse_component_values("fade 1s, slide ease-in").unwrap();
    for invalid in [
        "",
        "fade -1s",
        "fade 1s 2s 3s",
        "fade slide",
        "fade 1s!important",
        "fade 1s;color:red",
        "fade calc(1s",
        "fade 1s calc(-250ms",
    ] {
        let components = parse_component_values(invalid).unwrap();
        let expected = parse_property_value_for_grammar(
            P::Animation.grammar(),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap_err();
        let error = handle.reenter(components).unwrap_err();
        let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
            panic!("strict original grammar")
        };
        assert_eq!(actual, &expected);
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("valid retry")
        };
        assert_eq!(names(&values), MEMBERS);
    }
    for residual in [
        "var(--again)",
        "env(animation)",
        "attr(animation)",
        "fade calc(var(--again) + 1s)",
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
            panic!("retry after residual")
        };
        assert_eq!(names(&values), MEMBERS);
    }
    let CssContributions::Longhands(values) = handle
        .reenter(parse_component_values("unset").unwrap())
        .unwrap()
    else {
        panic!("global replacement")
    };
    assert_eq!(names(&values), MEMBERS);
    for item in values.items() {
        assert_source(item, &source);
        assert_eq!(
            item.value(),
            CssContributionValueRef::Global(CssGlobalKeyword::Unset)
        );
    }
}

#[test]
fn recovered_duration_and_delay_project_without_forging_checked_closure() {
    for (css, index, expected_css) in [
        (
            "animation:fade 1s, slide calc(2s",
            0,
            "animation-duration:1s, calc(2s",
        ),
        (
            "animation:fade 1s, slide 2s calc(-250ms",
            2,
            "animation-delay:0s, calc(-250ms",
        ),
    ] {
        let report = parse_style_attribute(css);
        assert!(!report.is_clean());
        let [source] = report.syntax().as_slice() else {
            panic!("retained recovered shorthand")
        };
        let CssKnownPropertyValueRef::Animation(wrapper) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("animation recovery")
        };
        assert_eq!(wrapper.animations().values().len(), 2);
        assert!(CssAnimationList::try_new(wrapper.animations().values().to_vec()).is_none());
        let values = completed(source);
        assert_eq!(names(&values), MEMBERS);
        for item in values.items() {
            assert_ordinary(item, source);
        }
        let expected_report = parse_style_attribute(expected_css);
        assert!(!expected_report.is_clean());
        let [expected_source] = expected_report.syntax().as_slice() else {
            panic!("retained recovered explicit longhand")
        };
        let expected = completed(expected_source);
        assert_eq!(
            values.items()[index].ordinary_value(),
            expected.items()[0].ordinary_value()
        );
    }
}

#[test]
fn ordered_normalization_preserves_groups_and_fixed_eight_member_budget() {
    let css = ".a{color:red;animation:fade 1s,slide ease-in!important;animation-delay:-250ms}.b{animation:var(--animation);all:unset;--animation:fade 1s}";
    let report = parse_sheet(css);
    assert!(report.is_clean());
    let before = report.clone();
    // Four other completed/pending/custom/reset occurrences plus Color: five;
    // the two-item shorthand contributes exactly eight members, totaling thirteen.
    let exact = CssNormalizationLimits::try_new(0, 2, 6, 13).unwrap();
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
    for (order, declaration) in declarations.iter().enumerate() {
        assert_eq!(declaration.order(), order);
    }
    for (declaration, property) in declarations[..5].iter().zip([
        P::Color,
        P::Animation,
        P::AnimationDelay,
        P::Animation,
        P::All,
    ]) {
        assert_eq!(declaration.source().known().unwrap().property(), property);
    }
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        declarations[1].expansion()
    else {
        panic!("normalized shorthand")
    };
    assert_payloads(
        values,
        [
            "1s, 0s",
            "ease, ease-in",
            "0s, 0s",
            "1, 1",
            "normal, normal",
            "none, none",
            "running, running",
            "fade, slide",
        ],
    );
    assert_eq!(
        declarations[1].source().importance(),
        CssImportance::Important
    );
    for item in values.items() {
        assert_ordinary(item, declarations[1].source());
    }
    assert!(matches!(
        declarations[3].expansion(),
        CssExpansion::Pending(_)
    ));
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
        CssNormalizationLimits::try_new(0, 2, 6, 12).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 12
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
fn sparse_and_recovered_shorthands_normalize_at_eight_and_fail_atomically_at_seven() {
    for css in [
        ".a{animation:fade 1s,slide ease-in!important}",
        ".a{animation:fade 1s,slide calc(2s",
    ] {
        let report = parse_sheet(css);
        let before = report.clone();
        let exact = CssNormalizationLimits::try_new(0, 1, 1, 8).unwrap();
        let normalized = normalize_report_with_limits(&report, exact).unwrap();
        assert_eq!(normalized.is_clean(), report.is_clean());
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let declaration = normalized
            .syntax()
            .items()
            .iter()
            .find_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .unwrap();
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            declaration.expansion()
        else {
            panic!("normalized eight members")
        };
        assert_eq!(names(values), MEMBERS);
        for item in values.items() {
            assert_ordinary(item, declaration.source());
        }
        let error = normalize_report_with_limits(
            &report,
            CssNormalizationLimits::try_new(0, 1, 1, 7).unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded {
                resource: CssNormalizationResource::Contributions,
                limit: 7
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
