#![forbid(unsafe_code)]

//! Grid 2 (2025-03-26) §7.6 specifies both implicit track-size longhands as
//! noninherited, initially `auto`, repeat-free `<track-size>+` values.
//! https://www.w3.org/TR/2025/CRD-css-grid-2-20250326/#auto-tracks
//! This RED exercises only the existing public parser and lifecycle APIs.

use surgeist_css::*;

const PROPERTIES: [(&str, CssKnownProperty); 2] = [
    ("grid-auto-rows", CssKnownProperty::GridAutoRows),
    ("grid-auto-columns", CssKnownProperty::GridAutoColumns),
];

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).expect("known implicit Grid longhand")
}

fn parsed(name: &str, value: &str, importance: CssImportance) -> CssDeclaration {
    let important = if importance == CssImportance::Important {
        "!important"
    } else {
        ""
    };
    let source = format!("{name}:{value}{important}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn checked(name: &str, value: &str, importance: CssImportance) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(name),
        parse_component_values(value).unwrap(),
        importance,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"))
}

fn terminal(source: &CssDeclaration, property: CssKnownProperty) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("intrinsic implicit-track expansion")
    else {
        panic!("one completed implicit-track longhand")
    };
    let [item] = values.items() else {
        panic!("one implicit-track contribution")
    };
    assert_eq!(item.property(), property);
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    values
}

#[test]
fn existing_repeat_free_lists_parse_as_nonempty_typed_track_sizes() {
    for (name, property) in PROPERTIES {
        for source in [
            parsed(
                name,
                "auto 10px minmax(10px, 1fr) fit-content(20%)",
                CssImportance::Normal,
            ),
            checked(
                name,
                "auto 10px minmax(10px, 1fr) fit-content(20%)",
                CssImportance::Normal,
            ),
        ] {
            assert_eq!(source.known().unwrap().property(), property);
            let sizes = match source.known().unwrap().property_value().unwrap() {
                CssKnownPropertyValueRef::GridAutoRows(value) => value.value().sizes(),
                CssKnownPropertyValueRef::GridAutoColumns(value) => value.value().sizes(),
                _ => panic!("typed implicit track-size list"),
            };
            assert_eq!(sizes.len(), 4);
            assert_eq!(
                sizes[0].breadth().unwrap().kind(),
                CssGridTrackBreadthKind::Auto
            );
            assert_eq!(sizes[1].kind(), CssGridTrackSizeKind::Breadth);
            assert_eq!(sizes[2].kind(), CssGridTrackSizeKind::MinMax);
            assert_eq!(sizes[3].kind(), CssGridTrackSizeKind::FitContent);
        }
        for invalid in ["repeat(2, 10px)", "[line] 10px", "none", ""] {
            let source = format!("color:red;{name}:{invalid};color:blue");
            let report = parse_style_attribute(&source);
            assert_eq!(report.syntax().len(), 2, "{source}");
            assert_eq!(report.diagnostics().len(), 1, "{source}");
            assert_eq!(
                report.diagnostics()[0].action(),
                CssRecoveryAction::DropDeclaration
            );
            assert!(validate_style_attribute(&source).is_err());
            assert!(
                parse_property_value_for_grammar(
                    grammar(name),
                    parse_component_values(invalid).unwrap(),
                    CssImportance::Normal,
                )
                .is_err(),
                "checked accepted {name}:{invalid}"
            );
        }
    }
}

#[test]
fn ordinary_values_expand_once_with_unchanged_source_and_importance() {
    for (name, property) in PROPERTIES {
        for value in ["auto", "10px", "minmax(10px, 1fr)", "fit-content(20%)"] {
            for source in [
                parsed(name, value, CssImportance::Important),
                checked(name, value, CssImportance::Important),
            ] {
                let values = terminal(&source, property);
                let item = &values.items()[0];
                assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
                assert_eq!(
                    item.ordinary_value().unwrap().property().known_property(),
                    property
                );
                assert!(item.replacement_components().is_none());
            }
        }
    }
}

#[test]
fn metadata_uses_noninherited_auto_initial_and_all_includes_both_longhands() {
    let CssPropertyKindRef::UniversalReset(all) = CssKnownProperty::All.metadata().unwrap().kind()
    else {
        panic!("all reset metadata")
    };
    let all_source = parsed("all", "initial", CssImportance::Important);
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all_source).unwrap()
    else {
        panic!("symbolic all reset")
    };
    assert_eq!(reset.keyword(), CssGlobalKeyword::Initial);
    for (name, property) in PROPERTIES {
        let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("implicit track longhand metadata: {name}")
        };
        assert!(!longhand.inherited_by_default(), "{name}");
        let initial = longhand.initial_value();
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("ordinary auto initial: {name}")
        };
        assert_eq!(initial.property().known_property(), property);
        // The independent parser control above identifies `auto` as one Auto
        // breadth; this compares its intrinsic contribution to metadata.
        assert_eq!(
            Some(initial),
            terminal(&parsed(name, "auto", CssImportance::Normal), property).items()[0]
                .ordinary_value()
        );
        assert!(!all.excludes(CssPropertyNameRef::Known(property)));
        assert!(!reset.excludes(CssPropertyNameRef::Known(property)));
    }
}

#[test]
fn css_wide_values_expand_terminally_and_pending_reentry_is_strict() {
    for (name, property) in PROPERTIES {
        for (spelling, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            for source in [
                parsed(name, spelling, CssImportance::Important),
                checked(name, spelling, CssImportance::Important),
            ] {
                let values = terminal(&source, property);
                assert_eq!(
                    values.items()[0].value(),
                    CssContributionValueRef::Global(keyword)
                );
                assert!(values.items()[0].ordinary_value().is_none());
            }
        }
        let source = parsed(name, "var(--tracks)", CssImportance::Important);
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("implicit track substitution remains pending")
        };
        assert!(pending.source().same_occurrence(&source));
        assert_eq!(pending.source().importance(), CssImportance::Important);
        assert_eq!(
            pending
                .reenter(parse_component_values("var(--again)").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        for invalid in ["repeat(2, 10px)", "[line] 10px", "none", "auto / 10px"] {
            assert!(
                matches!(
                    pending
                        .reenter(parse_component_values(invalid).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::InvalidReplacement(_)
                ),
                "{name}:{invalid}"
            );
        }
        for valid in ["auto 10px", "minmax(10px, 1fr)", "inherit"] {
            let replacement = parse_component_values(valid).unwrap();
            for _ in 0..2 {
                let CssContributions::Longhands(values) =
                    pending.reenter(replacement.clone()).unwrap()
                else {
                    panic!("one reentered longhand")
                };
                let [item] = values.items() else {
                    panic!("one reentered contribution")
                };
                assert_eq!(item.property(), property);
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
                if valid == "inherit" {
                    assert_eq!(
                        item.value(),
                        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
                    );
                    assert!(item.ordinary_value().is_none());
                } else {
                    assert!(matches!(item.value(), CssContributionValueRef::Ordinary(_)));
                    assert_eq!(
                        item.ordinary_value().unwrap().property().known_property(),
                        property
                    );
                }
            }
        }
    }
}

#[test]
fn normalization_preserves_both_properties_order_and_occurrences() {
    let report = parse_sheet(concat!(
        ".p{color:red;grid-auto-rows:auto!important;grid-auto-columns:10px;",
        "grid-auto-rows:minmax(10px,1fr);opacity:1}"
    ));
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssRule::Style(rule) = &report.syntax().rules()[0] else {
        panic!("one style rule")
    };
    let originals = rule.declarations();
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 5);
    for (index, value) in values.iter().enumerate() {
        assert_eq!(value.order(), index);
        assert!(value.source().same_occurrence(&originals[index]));
    }
    for (index, property, importance) in [
        (1, CssKnownProperty::GridAutoRows, CssImportance::Important),
        (2, CssKnownProperty::GridAutoColumns, CssImportance::Normal),
        (3, CssKnownProperty::GridAutoRows, CssImportance::Normal),
    ] {
        let CssExpansion::Contributions(CssContributions::Longhands(items)) =
            values[index].expansion()
        else {
            panic!("terminal implicit-track contribution")
        };
        let [item] = items.items() else {
            panic!("one contribution")
        };
        assert_eq!(item.property(), property);
        assert!(item.source().same_occurrence(&originals[index]));
        assert_eq!(item.source().importance(), importance);
    }
}

#[test]
fn normalization_reports_exact_rule_declaration_and_contribution_limits() {
    for (name, _) in PROPERTIES {
        let report = parse_sheet(&format!(".p{{{name}:auto}}"));
        assert!(report.is_clean());
        let original = report.syntax().clone();
        for (limits, resource) in [
            (
                CssNormalizationLimits::try_new(1, 0, 1, 1).unwrap(),
                CssNormalizationResource::Rules,
            ),
            (
                CssNormalizationLimits::try_new(1, 1, 0, 1).unwrap(),
                CssNormalizationResource::Declarations,
            ),
            (
                CssNormalizationLimits::try_new(1, 1, 1, 0).unwrap(),
                CssNormalizationResource::Contributions,
            ),
        ] {
            let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded { resource, limit: 0 }
            );
            assert_eq!(report.syntax(), &original);
        }
        assert!(
            normalize_sheet_with_limits(
                report.syntax(),
                CssNormalizationLimits::try_new(1, 1, 1, 1).unwrap()
            )
            .is_ok()
        );
    }
}
