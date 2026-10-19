#![forbid(unsafe_code)]
//! Independent authority: pinned Compositing 1 (2024-03-21), §§3.1, 3.4.2,
//! 3.4.3: sixteen blend keywords, isolation auto/isolate, three noninherited
//! longhands, normal/auto/normal initials, ordered background blend lists, and
//! background's reset of background-blend-mode. The normative production, not
//! linked WPT filenames, determines the exclusion of plus-lighter here.
//! references/compositing-1--CRD-compositing-1-20240321--0fb91119308e.md:
//! 134-142, 213-221, 425-445, 479-486, 504, 510.
//! Cascade 5 §3.1 supplies reset-only CSS-wide/importance propagation;
//! references/css-cascade-5--CR-css-cascade-5-20220113--1fe27cdb69e4.md:439.
//! Surgeist public contracts supply strict checked/reentry provenance, atomic
//! normalization, and cumulative serializer budget accounting. Contextual list
//! repetition and compositing execution are deliberately not asserted.
//!
//! Source-defined initials are compared with explicitly authored values through
//! existing property wrapper accessors.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const FAMILY: [(P, &str, &str); 3] = [
    (P::BackgroundBlendMode, "hue, normal, hue", "normal"),
    (P::Isolation, "isolate", "auto"),
    (P::MixBlendMode, "multiply", "normal"),
];
const BACKGROUND_SETTABLE: [P; 8] = [
    P::BackgroundImage,
    P::BackgroundPosition,
    P::BackgroundSize,
    P::BackgroundRepeat,
    P::BackgroundAttachment,
    P::BackgroundOrigin,
    P::BackgroundClip,
    P::BackgroundColor,
];
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
const MODES: [(&str, CssBlendMode); 16] = [
    ("normal", CssBlendMode::Normal),
    ("darken", CssBlendMode::Darken),
    ("multiply", CssBlendMode::Multiply),
    ("color-burn", CssBlendMode::ColorBurn),
    ("lighten", CssBlendMode::Lighten),
    ("screen", CssBlendMode::Screen),
    ("color-dodge", CssBlendMode::ColorDodge),
    ("overlay", CssBlendMode::Overlay),
    ("soft-light", CssBlendMode::SoftLight),
    ("hard-light", CssBlendMode::HardLight),
    ("difference", CssBlendMode::Difference),
    ("exclusion", CssBlendMode::Exclusion),
    ("hue", CssBlendMode::Hue),
    ("saturation", CssBlendMode::Saturation),
    ("color", CssBlendMode::Color),
    ("luminosity", CssBlendMode::Luminosity),
];

fn parsed(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [source] = report.syntax().as_slice() else {
        panic!("one declaration")
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
        expand_declaration(source).expect("intrinsic terminal contributions")
    else {
        panic!("completed longhands")
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
fn background_names() -> Vec<P> {
    BACKGROUND_SETTABLE
        .into_iter()
        .chain([P::BackgroundBlendMode])
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
fn metadata_contract(property: P, initial_css: &str) {
    let metadata = property.metadata().expect("intrinsic metadata");
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
    let explicit = completed(&checked(property, initial_css));
    assert_eq!(Some(initial), explicit.items()[0].ordinary_value());
}
fn ordinary_contract(property: P, css: &str) {
    let authored = parsed(&format!(
        "/*😀*/{}:{css}!important",
        property.canonical_name()
    ));
    let components = parse_component_values(css).unwrap();
    let before = components.clone();
    let constructed = parse_property_value(
        CssPropertyNameRef::Known(property),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(components, before);
    assert!(authored.position().is_some());
    assert!(constructed.position().is_none());
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
fn globals_contract(property: P) {
    for (css, keyword) in GLOBALS {
        for source in [
            parsed(&format!("{}:{css}!important", property.canonical_name())),
            checked(property, css),
        ] {
            let values = completed(&source);
            assert_eq!(names(&values), [property]);
            let item = &values.items()[0];
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert!(item.ordinary_value().is_none());
            assert_source(item, &source);
            assert!(item.replacement_components().is_none());
        }
    }
}
fn pending_contract(property: P, valid_css: &str, invalid: &[&str]) {
    for pending_css in ["var(--mode)", "env(mode)", "attr(data-mode)"] {
        let source = parsed(&format!(
            "/*😀*/{}:{pending_css}!important",
            property.canonical_name()
        ));
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending longhand")
        };
        assert!(handle.source().same_occurrence(&source));
        for bad in invalid {
            let components = parse_component_values(bad).unwrap();
            let expected = parse_property_value_for_grammar(
                property.grammar(),
                components.clone(),
                CssImportance::Normal,
            )
            .unwrap_err();
            let error = handle.reenter(components).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
                panic!("original grammar failure")
            };
            assert_eq!(actual, &expected);
            assert!(handle.source().same_occurrence(&source));
        }
        for residual in ["var(--again)", "env(again)", "attr(data-again)"] {
            assert_eq!(
                handle
                    .reenter(parse_component_values(residual).unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
        }
        let replacement = parse_component_values(valid_css).unwrap();
        let before = replacement.clone();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("retry")
            };
            assert_eq!(names(&values), [property]);
            let item = &values.items()[0];
            assert_ordinary(item, &source);
            assert_eq!(item.replacement_components(), Some(&replacement));
            assert_eq!(
                item.ordinary_value(),
                completed(&checked(property, valid_css)).items()[0].ordinary_value()
            );
        }
        assert_eq!(replacement, before);
        for (css, global) in GLOBALS {
            let replacement = parse_component_values(css).unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("global retry")
            };
            assert_eq!(names(&values), [property]);
            assert_eq!(
                values.items()[0].value(),
                CssContributionValueRef::Global(global)
            );
            assert_source(&values.items()[0], &source);
            assert_eq!(
                values.items()[0].replacement_components(),
                Some(&replacement)
            );
        }
    }
}

macro_rules! family_contracts {
    ($metadata:ident, $ordinary:ident, $globals:ident, $pending:ident, $property:expr, $css:literal, $initial:literal, [$($bad:literal),+]) => {
        #[test] fn $metadata() { metadata_contract($property, $initial); }
        #[test] fn $ordinary() { ordinary_contract($property, $css); }
        #[test] fn $globals() { globals_contract($property); }
        #[test] fn $pending() { pending_contract($property, $css, &[$($bad),+]); }
    };
}
family_contracts!(
    background_blend_initial_is_singleton_normal_noninherited,
    background_blend_projects_one_ordered_list_with_original_occurrence,
    background_blend_globals_remain_symbolic,
    background_blend_pending_reentry_is_reusable_and_grammar_specific,
    P::BackgroundBlendMode,
    "hue, normal, hue",
    "normal",
    [
        "",
        "normal multiply",
        "normal,",
        "normal,,screen",
        "plus-lighter",
        "normal!important",
        "normal;color:red"
    ]
);
family_contracts!(
    isolation_initial_is_auto_noninherited,
    isolation_projects_one_typed_terminal_with_original_occurrence,
    isolation_globals_remain_symbolic,
    isolation_pending_reentry_is_reusable_and_grammar_specific,
    P::Isolation,
    "isolate",
    "auto",
    [
        "",
        "none",
        "auto isolate",
        "auto,isolate",
        "auto!important",
        "auto;color:red"
    ]
);
family_contracts!(
    mix_blend_initial_is_normal_noninherited,
    mix_blend_projects_one_typed_terminal_with_original_occurrence,
    mix_blend_globals_remain_symbolic,
    mix_blend_pending_reentry_is_reusable_and_grammar_specific,
    P::MixBlendMode,
    "multiply",
    "normal",
    [
        "",
        "normal multiply",
        "normal,screen",
        "plus-lighter",
        "normal!important",
        "normal;color:red"
    ]
);

#[test]
fn sixteen_selected_modes_have_parsed_checked_and_keyword_identity() {
    for (css, expected) in MODES {
        assert_eq!(CssBlendMode::from_keyword(css), Some(expected));
        assert_eq!(
            CssBlendMode::from_keyword(&css.to_ascii_uppercase()),
            Some(expected)
        );
        assert_eq!(expected.as_css_str(), css);
        assert_eq!(expected.serialize_specified().unwrap(), css);
        for source in [
            parsed(&format!("mix-blend-mode:{}", css.to_ascii_uppercase())),
            checked(P::MixBlendMode, css),
        ] {
            let CssKnownPropertyValueRef::MixBlendMode(wrapper) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("typed scalar")
            };
            assert_eq!(*wrapper.mode(), expected);
        }
        for source in [
            parsed(&format!(
                "background-blend-mode:{}",
                css.to_ascii_uppercase()
            )),
            checked(P::BackgroundBlendMode, css),
        ] {
            let CssKnownPropertyValueRef::BackgroundBlendMode(wrapper) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("typed list")
            };
            assert_eq!(wrapper.modes().modes(), &[expected]);
        }
    }
    for (css, expected) in [
        ("auto", CssIsolation::Auto),
        ("isolate", CssIsolation::Isolate),
    ] {
        for source in [
            parsed(&format!("isolation:{}", css.to_ascii_uppercase())),
            checked(P::Isolation, css),
        ] {
            let CssKnownPropertyValueRef::Isolation(wrapper) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("typed isolation")
            };
            assert_eq!(*wrapper.isolation(), expected);
        }
        assert_eq!(expected.serialize_specified().unwrap(), css);
    }
}

#[test]
fn background_list_keeps_decoded_case_duplicates_and_constructed_order() {
    let source = checked(P::BackgroundBlendMode, r"h\75 e, NORMAL, hue");
    let CssKnownPropertyValueRef::BackgroundBlendMode(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("list")
    };
    let expected = [CssBlendMode::Hue, CssBlendMode::Normal, CssBlendMode::Hue];
    assert_eq!(wrapper.modes().modes(), &expected);
    assert_eq!(
        wrapper.modes().serialize_specified().unwrap(),
        "hue, normal, hue"
    );
    assert_eq!(
        CssBlendModeList::try_new(expected.to_vec())
            .unwrap()
            .modes(),
        &expected
    );
    assert!(CssBlendModeList::try_new(Vec::new()).is_none());
}

#[test]
fn invalid_compositing_values_recover_atomically_and_keep_color_control() {
    for (property, values) in [
        (
            P::BackgroundBlendMode,
            &[
                "normal multiply",
                "normal,",
                ",normal",
                "normal,,screen",
                "plus-lighter",
            ][..],
        ),
        (P::Isolation, &["none", "auto isolate", "auto,isolate"][..]),
        (
            P::MixBlendMode,
            &["normal multiply", "normal,screen", "plus-lighter"][..],
        ),
    ] {
        for value in values {
            let css = format!(
                "/*😀*/{}:{value};color:currentcolor",
                property.canonical_name()
            );
            let report = parse_style_attribute(&css);
            let [source] = report.syntax().as_slice() else {
                panic!("only adjacent color retained")
            };
            assert_eq!(source.known().unwrap().property(), P::Color);
            let [diagnostic] = report.diagnostics() else {
                panic!("one atomic diagnostic")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            assert_eq!(
                diagnostic.error().code(),
                CssErrorCode::InvalidPropertyValue
            );
            let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
                panic!("property error")
            };
            assert_eq!(detail.property(), property);
            assert_eq!(
                validate_style_attribute(&css).unwrap_err().diagnostics(),
                report.diagnostics()
            );
            assert!(
                parse_property_value(
                    CssPropertyNameRef::Known(property),
                    parse_component_values(value).unwrap(),
                    CssImportance::Normal
                )
                .is_err()
            );
        }
    }
}

fn recovered_origin(components: &CssComponentValues) -> CssValueOrigin {
    let serialized = components.serialize().unwrap();
    (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("original implicit closure in recovered comment")
}
fn closure_contract(property: P, css: &str) {
    for value in [
        format!("{css}/*"),
        "initial/*".into(),
        "var(--mode)/*".into(),
    ] {
        let components = parse_component_values(&value).unwrap();
        let before = components.clone();
        let origin = recovered_origin(&components);
        for result in [
            parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                CssImportance::Important,
            ),
            parse_property_value_for_grammar(
                property.grammar(),
                components.clone(),
                CssImportance::Important,
            ),
        ] {
            let error = result.expect_err("checked values cannot repair original delimiters");
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
            ));
            assert_eq!(
                error.origin(),
                &CssSerializedOrigin::End(Some(origin.clone()))
            );
        }
        assert_eq!(components, before);
    }
}
#[test]
fn background_blend_recovered_comments_are_rejected_by_both_checked_fronts() {
    closure_contract(P::BackgroundBlendMode, "hue, normal");
}
#[test]
fn isolation_recovered_comments_are_rejected_by_both_checked_fronts() {
    closure_contract(P::Isolation, "isolate");
}
#[test]
fn mix_blend_recovered_comments_are_rejected_by_both_checked_fronts() {
    closure_contract(P::MixBlendMode, "multiply");
}

#[test]
fn complete_comments_globals_and_pending_components_remain_admitted() {
    for (property, css, _) in FAMILY {
        for value in [
            format!("{css}/**/"),
            "initial/**/".into(),
            "var(--mode)/**/".into(),
        ] {
            let components = parse_component_values(&value).unwrap();
            for source in [
                parse_property_value(
                    CssPropertyNameRef::Known(property),
                    components.clone(),
                    CssImportance::Important,
                )
                .unwrap(),
                parse_property_value_for_grammar(
                    property.grammar(),
                    components.clone(),
                    CssImportance::Important,
                )
                .unwrap(),
            ] {
                assert_eq!(source.value_components(), &components);
                assert_eq!(source.known().unwrap().grammar(), property.grammar());
                assert_eq!(source.importance(), CssImportance::Important);
            }
        }
    }
}

#[test]
fn recovered_reentry_fails_at_original_closure_then_valid_retry_preserves_source() {
    for (property, css, _) in FAMILY {
        let source = checked(property, "var(--mode)");
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for value in [format!("{css}/*"), "initial/*".into()] {
            let components = parse_component_values(&value).unwrap();
            let origin = recovered_origin(&components);
            let error = handle.reenter(components).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("strict closure failure")
            };
            assert!(matches!(
                error.kind(),
                CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
            ));
            assert_eq!(error.origin(), &CssSerializedOrigin::End(Some(origin)));
            let replacement = parse_component_values(css).unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("retry")
            };
            assert_eq!(names(&values), [property]);
            assert_ordinary(&values.items()[0], &source);
            assert_eq!(
                values.items()[0].replacement_components(),
                Some(&replacement)
            );
        }
    }
}

#[test]
fn background_metadata_preserves_eight_settable_members_and_one_reset_only_target() {
    let CssPropertyKindRef::Shorthand(metadata) = P::Background.metadata().unwrap().kind() else {
        panic!("background shorthand")
    };
    let properties = |members: &[CssLonghandProperty]| {
        members
            .iter()
            .map(|p| p.known_property())
            .collect::<Vec<_>>()
    };
    assert_eq!(properties(metadata.settable_members()), BACKGROUND_SETTABLE);
    assert_eq!(
        properties(metadata.reset_only_members()),
        [P::BackgroundBlendMode]
    );
    assert_eq!(properties(metadata.members()), background_names());
}

fn background_reset_contract(source: &CssDeclaration) {
    let values = completed(source);
    assert_eq!(names(&values), background_names());
    let reset = values.items().last().unwrap();
    assert_eq!(reset.property(), P::BackgroundBlendMode);
    assert_ordinary(reset, source);
    // Exactly one normal value, not one normal per authored background layer.
    let explicit = completed(&checked(P::BackgroundBlendMode, "normal"));
    assert_eq!(reset.ordinary_value(), explicit.items()[0].ordinary_value());
    for item in values.items() {
        assert_source(item, source);
    }
}
#[test]
fn background_ordinary_expansion_resets_blending_without_repeating_initial_per_layer() {
    for source in [
        parsed("/*😀*/background:none,none red!important"),
        checked(P::Background, "none, none red"),
    ] {
        background_reset_contract(&source);
        assert!(
            completed(&source)
                .items()
                .iter()
                .all(|item| item.replacement_components().is_none())
        );
    }
}
#[test]
fn background_css_wide_values_propagate_to_reset_only_blending_target() {
    for (css, global) in GLOBALS {
        let source = checked(P::Background, css);
        let values = completed(&source);
        assert_eq!(names(&values), background_names());
        for item in values.items() {
            assert_eq!(item.value(), CssContributionValueRef::Global(global));
            assert_source(item, &source);
        }
    }
}
#[test]
fn background_pending_reentry_keeps_reset_source_importance_and_replacement() {
    let source = parsed("/*😀*/background:var(--background)!important");
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("pending background")
    };
    let replacement = parse_component_values("none, none red").unwrap();
    for _ in 0..2 {
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("completed background")
        };
        assert_eq!(names(&values), background_names());
        assert_eq!(
            values.items()[8].ordinary_value(),
            completed(&checked(P::BackgroundBlendMode, "normal")).items()[0].ordinary_value()
        );
        for item in values.items() {
            assert_source(item, &source);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
    for (css, keyword) in GLOBALS {
        let replacement = parse_component_values(css).unwrap();
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("global background retry")
        };
        assert_eq!(names(&values), background_names());
        for item in values.items() {
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert_source(item, &source);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn ordered_normalization_keeps_authored_lists_and_charges_background_reset_terminal() {
    let css = ".a{color:currentcolor;background-blend-mode:hue,normal,hue!important;isolation:isolate;mix-blend-mode:multiply;background:red!important}.b{background-blend-mode:var(--mode);mix-blend-mode:unset}";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let before = report.clone();
    // Four ordinary singletons + nine-member background + pending + global = 15.
    let exact = CssNormalizationLimits::try_new(0, 2, 7, 15).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 7);
    for (index, (item, property)) in declarations
        .iter()
        .zip([
            P::Color,
            P::BackgroundBlendMode,
            P::Isolation,
            P::MixBlendMode,
            P::Background,
            P::BackgroundBlendMode,
            P::MixBlendMode,
        ])
        .enumerate()
    {
        assert_eq!(item.order(), index);
        assert_eq!(item.source().known().unwrap().property(), property);
    }
    let CssExpansion::Contributions(CssContributions::Longhands(authored)) =
        declarations[1].expansion()
    else {
        panic!("authored list")
    };
    assert_eq!(names(authored), [P::BackgroundBlendMode]);
    assert_eq!(
        authored.items()[0].ordinary_value(),
        completed(&checked(P::BackgroundBlendMode, "hue, normal, hue")).items()[0].ordinary_value()
    );
    assert_ordinary(&authored.items()[0], declarations[1].source());
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
        css.find("background-blend-mode:").unwrap()
    );
    let CssExpansion::Contributions(CssContributions::Longhands(background)) =
        declarations[4].expansion()
    else {
        panic!("normalized background")
    };
    assert_eq!(names(background), background_names());
    assert_eq!(
        background.items()[8].ordinary_value(),
        completed(&checked(P::BackgroundBlendMode, "normal")).items()[0].ordinary_value()
    );
    for item in background.items() {
        assert_source(item, declarations[4].source());
    }
    assert!(matches!(
        declarations[5].expansion(),
        CssExpansion::Pending(_)
    ));
    let CssExpansion::Contributions(CssContributions::Longhands(global)) =
        declarations[6].expansion()
    else {
        panic!("global")
    };
    assert_eq!(
        global.items()[0].value(),
        CssContributionValueRef::Global(CssGlobalKeyword::Unset)
    );
    for (limits, resource, limit) in [
        (
            CssNormalizationLimits::try_new(0, 2, 7, 14).unwrap(),
            CssNormalizationResource::Contributions,
            14,
        ),
        (
            CssNormalizationLimits::try_new(0, 2, 6, 15).unwrap(),
            CssNormalizationResource::Declarations,
            6,
        ),
    ] {
        let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded { resource, limit }
        );
        assert_eq!(error.declaration_order(), Some(6));
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(declarations[6].source())
        );
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}

#[test]
fn background_requires_nine_normalization_members_and_recovers_after_eight_budget_failure() {
    for value in ["red", "revert-layer"] {
        let report = parse_sheet(&format!(".a{{background:{value}!important}}"));
        assert!(report.is_clean());
        let exact = CssNormalizationLimits::try_new(0, 1, 1, 9).unwrap();
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
            CssNormalizationLimits::try_new(0, 1, 1, 8).unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded {
                resource: CssNormalizationResource::Contributions,
                limit: 8
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
}

#[test]
fn recovered_browser_comments_keep_diagnostics_through_report_normalization() {
    for (property, css, _) in FAMILY {
        let report = parse_sheet(&format!(".a{{{}:{css}/*", property.canonical_name()));
        assert!(!report.is_clean());
        let before = report.clone();
        let exact = CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap();
        let normalized = normalize_report_with_limits(&report, exact).unwrap();
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let declarations = normalized
            .syntax()
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(value) => Some(value),
                _ => None,
            })
            .collect::<Vec<_>>();
        let [item] = declarations.as_slice() else {
            panic!("one retained browser declaration")
        };
        assert_eq!(item.source().known().unwrap().property(), property);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
        else {
            panic!("ordinary recovered value")
        };
        assert_eq!(names(values), [property]);
        assert_ordinary(&values.items()[0], item.source());
        let error = normalize_report_with_limits(
            &report,
            CssNormalizationLimits::try_new(0, 1, 1, 0).unwrap(),
        )
        .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded {
                resource: CssNormalizationResource::Contributions,
                limit: 0
            }
        );
        assert!(error.declaration().unwrap().same_occurrence(item.source()));
        assert_eq!(report, before);
        assert!(normalize_report_with_limits(&report, exact).is_ok());
    }
}

#[test]
fn blend_list_and_primitive_output_obey_independent_text_and_cumulative_limits() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    let list = CssBlendModeList::try_new(vec![
        CssBlendMode::Hue,
        CssBlendMode::Normal,
        CssBlendMode::Hue,
    ])
    .unwrap();
    let before = list.clone();
    let expected = "hue, normal, hue";
    // Public list contract: one aggregate plus three primitive visits.
    assert_eq!(
        list.serialize_specified_with_limits(Limits::new(4, 4, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(3, 4, expected.len()), Kind::InputNodeLimit),
        (Limits::new(4, 3, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(4, 4, expected.len() - 1), Kind::ByteLimit),
    ] {
        assert_eq!(
            list.serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(list, before);
    }
    assert_eq!(list.serialize_specified().unwrap(), expected);
    assert_eq!(
        CssBlendMode::Multiply
            .serialize_specified_with_limits(Limits::new(1, 1, 8))
            .unwrap(),
        "multiply"
    );
    assert_eq!(
        CssIsolation::Isolate
            .serialize_specified_with_limits(Limits::new(1, 1, 7))
            .unwrap(),
        "isolate"
    );
    for limits in [
        Limits::new(0, 1, 8),
        Limits::new(1, 0, 8),
        Limits::new(1, 1, 0),
    ] {
        assert!(
            CssBlendMode::Multiply
                .serialize_specified_with_limits(limits)
                .is_err()
        );
        assert!(
            CssIsolation::Isolate
                .serialize_specified_with_limits(limits)
                .is_err()
        );
    }
}

#[test]
fn declaration_output_preserves_explicit_list_importance_and_full_provider_budget() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    let source = parsed("BACKGROUND-BLEND-MODE:HUE,NORMAL,HUE!important");
    let expected = "background-blend-mode: hue, normal, hue !important;";
    let before = source.clone();
    // Declaration + name + list + three keywords, per existing public contracts.
    assert_eq!(
        source
            .to_specified_css_with_limits(Limits::new(6, 6, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(5, 6, expected.len()), Kind::InputNodeLimit),
        (Limits::new(6, 5, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(6, 6, expected.len() - 1), Kind::ByteLimit),
    ] {
        assert_eq!(
            source
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(source, before);
    }
    assert_eq!(source.to_specified_css().unwrap(), expected);
    for (property, css, _) in FAMILY {
        let source = checked(property, css);
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("{}: {css} !important;", property.canonical_name())
        );
    }
}

#[test]
fn sibling_rule_output_shares_final_byte_budget_and_preserves_original_sheet() {
    use CssSpecifiedRuleSerializationErrorKind as RuleError;
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    let report = parse_sheet(
        ".a{background-blend-mode:hue,normal,hue!important;isolation:isolate}.b{mix-blend-mode:multiply}",
    );
    assert!(report.is_clean());
    let expected = ".a { background-blend-mode: hue, normal, hue !important; isolation: isolate; }\n.b { mix-blend-mode: multiply; }";
    let sheet = report.syntax();
    let before = sheet.clone();
    assert_eq!(sheet.to_specified_css().unwrap(), expected);
    let short = Limits::new(usize::MAX, usize::MAX, expected.len() - 1);
    assert!(
        sheet
            .rules()
            .iter()
            .all(|rule| rule.to_specified_css_with_limits(short).is_ok())
    );
    let error = sheet.to_specified_css_with_limits(short).unwrap_err();
    assert_eq!(error.kind(), RuleError::Resource(Kind::ByteLimit));
    assert_eq!(error.rule_index(), Some(1));
    assert_eq!(sheet, &before);
    assert_eq!(
        sheet
            .to_specified_css_with_limits(Limits::new(usize::MAX, usize::MAX, expected.len()))
            .unwrap(),
        expected
    );
}

#[test]
fn existing_color_control_retains_symbolic_initial_occurrence_and_normalization() {
    let CssPropertyKindRef::Longhand(metadata) = P::Color.metadata().unwrap().kind() else {
        panic!("color")
    };
    assert!(metadata.inherited_by_default());
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("symbolic color")
    };
    let CssLonghandValueRef::Color(color) = initial.view() else {
        panic!("existing color borrowed variant")
    };
    assert_eq!(color.system(), Some(CssSystemColor::CanvasText));
    let source = parsed("/*😀*/color:currentcolor!important");
    let values = completed(&source);
    assert_eq!(names(&values), [P::Color]);
    assert_ordinary(&values.items()[0], &source);
    let report = parse_sheet(".a{color:currentcolor}");
    assert!(
        normalize_sheet_with_limits(
            report.syntax(),
            CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap()
        )
        .is_ok()
    );
}
