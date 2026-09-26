#![forbid(unsafe_code)]

//! Authored overflow from CSS Overflow 3 WD (2025-10-07) §3.1.
//! Cross-axis computed-value coupling and writing-mode mapping require later context.

use surgeist_css::*;

const LONGHANDS: [&str; 4] = [
    "overflow-x",
    "overflow-y",
    "overflow-block",
    "overflow-inline",
];
const SHORTHAND: &str = "overflow";

fn names() -> impl Iterator<Item = &'static str> {
    LONGHANDS.into_iter().chain([SHORTHAND])
}

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("selected property: {name}"))
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}")
    };
    declaration.clone()
}

fn checked(name: &str, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(name),
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"))
}

fn accepted(name: &str, value: &str) {
    for source in [declaration(name, value), checked(name, value)] {
        assert_eq!(
            source.known().unwrap().property(),
            grammar(name).target_property()
        );
        assert_eq!(source.importance(), CssImportance::Important);
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            value
        );
    }
}

fn invalid(name: &str, value: &str) {
    let source = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid-value diagnostic: {source}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(report.syntax().len(), 2, "neighbors survive: {source}");
    assert!(validate_style_attribute(&source).is_err());
    assert!(
        parse_property_value_for_grammar(
            grammar(name),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .is_err(),
        "checked construction accepted {name}:{value}"
    );
}

fn one_value(source: &CssDeclaration) -> CssLonghandValue {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one terminal contribution")
    };
    let [item] = values.items() else {
        panic!("one terminal contribution")
    };
    assert!(item.source().same_occurrence(source));
    item.ordinary_value().unwrap().clone()
}

#[test]
fn five_distinct_names_have_complete_dated_overflow3_support() {
    let mut identities = Vec::new();
    for name in names() {
        let grammar = grammar(name);
        assert_eq!(grammar.name(), name);
        let property = grammar.target_property();
        assert_eq!(property.canonical_name(), name);
        assert_eq!(
            grammar.feature_id().as_str(),
            format!(
                "{}.property.{name}",
                if ["overflow", "overflow-x", "overflow-y"].contains(&name) {
                    "baseline"
                } else {
                    "ext"
                }
            )
        );
        assert!(
            !identities.contains(&property),
            "{name} aliases another property"
        );
        identities.push(property);
        let support = property_support_metadata(name).expect("selected support metadata");
        assert_eq!(support.property(), property);
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(support.feature().source().id().as_str(), "X-OVERFLOW3");
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2025/WD-css-overflow-3-20251007/")
        );
        assert_eq!(support.feature().production(), format!("#propdef-{name}"));
    }
    assert_eq!(identities.len(), 5);
}

#[test]
fn terminals_accept_five_keywords_and_overlay_alias_with_strict_recovery() {
    for name in LONGHANDS {
        for value in ["visible", "hidden", "clip", "scroll", "auto", "overlay"] {
            accepted(name, value);
        }
        for value in ["VISIBLE", "AuTo", "OVERLAY", "vis\\69 ble"] {
            accepted(name, value);
        }
        for value in [
            "none",
            "both",
            "start",
            "visible hidden",
            "overlay auto",
            "visible / hidden",
            "1px",
            "initial clip",
        ] {
            invalid(name, value);
        }
    }
    for value in [
        "visible",
        "hidden clip",
        "scroll auto",
        "auto visible",
        "overlay",
        "visible overlay",
        "overlay clip",
    ] {
        accepted(SHORTHAND, value);
    }
    for value in [
        "none",
        "both",
        "visible hidden clip",
        "overlay auto clip",
        "visible / hidden",
        "visible 1px",
        "initial hidden",
    ] {
        invalid(SHORTHAND, value);
    }
}

#[test]
fn terminal_initials_and_old_i01_projection_preserve_authored_overflow() {
    for name in LONGHANDS {
        let property = grammar(name).target_property();
        let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a terminal longhand")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(!longhand.inherited_by_default());
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), property);
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("{name} has a fixed initial")
        };
        assert_eq!(value, &one_value(&declaration(name, "visible")));

        for keyword in ["visible", "hidden", "clip", "scroll", "auto", "overlay"] {
            let source = declaration(name, keyword);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("{name}:{keyword} has one ordinary contribution")
            };
            let [item] = values.items() else {
                panic!("one terminal contribution")
            };
            assert_eq!(item.property(), property);
            assert!(item.ordinary_value().is_some());
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
        }
    }
    for name in ["overflow-x", "overflow-y"] {
        for (old, expected) in [
            ("visible", CssOverflow::Visible),
            ("hidden", CssOverflow::Hidden),
            ("clip", CssOverflow::Clip),
            ("scroll", CssOverflow::Scroll),
        ] {
            let source = declaration(name, old);
            let subset = match source.known().unwrap().property_value().unwrap() {
                CssKnownPropertyValueRef::OverflowX(value) => value.i01_subset(),
                CssKnownPropertyValueRef::OverflowY(value) => value.i01_subset(),
                _ => panic!("{name} has its existing property wrapper"),
            };
            assert_eq!(subset, Some(&expected));
        }
        for keyword in ["auto", "overlay"] {
            let source = declaration(name, keyword);
            let subset = match source.known().unwrap().property_value().unwrap() {
                CssKnownPropertyValueRef::OverflowX(value) => value.i01_subset(),
                CssKnownPropertyValueRef::OverflowY(value) => value.i01_subset(),
                _ => panic!("{name} has its existing property wrapper"),
            };
            assert!(subset.is_none(), "{name}:{keyword} is outside I01");
        }
    }
}

#[test]
fn shorthand_sets_only_x_then_y_and_preserves_uncomputed_axis_values() {
    let CssPropertyKindRef::Shorthand(shorthand) = grammar(SHORTHAND).metadata().unwrap().kind()
    else {
        panic!("overflow is a shorthand")
    };
    assert!(!shorthand.is_legacy());
    assert!(shorthand.reset_only_members().is_empty());
    assert_eq!(
        shorthand
            .settable_members()
            .iter()
            .map(|member| member.known_property().canonical_name())
            .collect::<Vec<_>>(),
        ["overflow-x", "overflow-y"]
    );
    for (authored, x, y) in [
        ("visible", "visible", "visible"),
        ("hidden clip", "hidden", "clip"),
        ("visible hidden", "visible", "hidden"),
        ("clip scroll", "clip", "scroll"),
        ("auto visible", "auto", "visible"),
        ("overlay", "auto", "auto"),
        ("visible overlay", "visible", "auto"),
    ] {
        let source = declaration(SHORTHAND, authored);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("overflow has two terminal contributions")
        };
        let [x_item, y_item] = values.items() else {
            panic!("exactly x then y")
        };
        for (item, name, value) in [(x_item, "overflow-x", x), (y_item, "overflow-y", y)] {
            assert_eq!(item.property(), grammar(name).target_property());
            assert_eq!(
                item.ordinary_value(),
                Some(&one_value(&declaration(name, value)))
            );
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert!(item.replacement_components().is_none());
        }
    }
    for (authored, expected) in [
        (
            "visible",
            CssOverflowI01PropertyValue::Single(CssOverflow::Visible),
        ),
        (
            "hidden",
            CssOverflowI01PropertyValue::Single(CssOverflow::Hidden),
        ),
        (
            "clip",
            CssOverflowI01PropertyValue::Single(CssOverflow::Clip),
        ),
        (
            "scroll",
            CssOverflowI01PropertyValue::Single(CssOverflow::Scroll),
        ),
        (
            "hidden clip",
            CssOverflowI01PropertyValue::Pair(CssOverflowAxes {
                x: CssOverflow::Hidden,
                y: CssOverflow::Clip,
            }),
        ),
    ] {
        let source = declaration(SHORTHAND, authored);
        let CssKnownPropertyValueRef::Overflow(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("existing overflow wrapper")
        };
        assert_eq!(value.i01_subset(), Some(&expected));
    }
    for authored in ["auto", "overlay", "visible auto"] {
        let source = declaration(SHORTHAND, authored);
        let CssKnownPropertyValueRef::Overflow(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("existing overflow wrapper")
        };
        assert!(value.i01_subset().is_none());
    }
    assert_eq!(
        one_value(&declaration("overflow-x", "overlay")),
        one_value(&declaration("overflow-x", "auto")),
        "overlay aliases auto before authored-value expansion"
    );
    assert_eq!(
        declaration("overflow-x", "overlay")
            .value_components()
            .serialize()
            .unwrap()
            .as_css(),
        "overlay",
        "the parsed declaration retains its source spelling"
    );
}

#[test]
fn css_wide_all_and_pending_reentry_keep_axes_and_occurrence() {
    let all_source = declaration("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all_source).unwrap()
    else {
        panic!("symbolic all reset")
    };
    for name in names() {
        assert!(!reset.excludes(CssPropertyNameRef::Known(grammar(name).target_property())));
        invalid(name, "initial clip");
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(name, text);
            checked(name, text);
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("{name} has symbolic global contributions")
            };
            assert_eq!(values.items().len(), if name == SHORTHAND { 2 } else { 1 });
            for item in values.items() {
                assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
            }
        }
        for pending in ["var(--overflow)", "env(--overflow)"] {
            let source = declaration(name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("{name} keeps substitution pending")
            };
            assert!(handle.source().same_occurrence(&source));
            for bad in ["none", "visible hidden clip", "initial clip"] {
                assert!(matches!(
                    handle
                        .reenter(parse_component_values(bad).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::InvalidReplacement(_)
                ));
            }
            assert_eq!(
                handle
                    .reenter(parse_component_values("var(--again)").unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            let replacement = parse_component_values("overlay").unwrap();
            let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("{name} reenters to ordinary contributions")
            };
            let expected_names: &[&str] = if name == SHORTHAND {
                &["overflow-x", "overflow-y"]
            } else {
                &[name]
            };
            assert_eq!(values.items().len(), expected_names.len());
            for (item, expected_name) in values.items().iter().zip(expected_names) {
                assert_eq!(item.property(), grammar(expected_name).target_property());
                assert_eq!(
                    item.ordinary_value(),
                    Some(&one_value(&declaration(expected_name, "auto")))
                );
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
            let CssContributions::Longhands(globals) = handle
                .reenter(parse_component_values("inherit").unwrap())
                .unwrap()
            else {
                panic!("{name} reenters to symbolic global contributions")
            };
            assert_eq!(globals.items().len(), expected_names.len());
            for item in globals.items() {
                assert_eq!(
                    item.value(),
                    CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
                );
                assert!(item.source().same_occurrence(&source));
            }
        }
    }
}

#[test]
fn normalization_preserves_order_and_logical_names_after_invalid_recovery() {
    let sheet = ".a{overflow-block:overlay;overflow-x:clip;overflow:visible hidden;overflow-inline:none;overflow-inline:auto;overflow-y:scroll}";
    let report = parse_sheet(sheet);
    let [diagnostic] = report.diagnostics() else {
        panic!(
            "one invalid inline overflow diagnostic: {:?}",
            report.diagnostics()
        )
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let declarations = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(declarations.len(), 5);
    for (index, (name, authored)) in [
        ("overflow-block", "overlay"),
        ("overflow-x", "clip"),
        ("overflow", "visible hidden"),
        ("overflow-inline", "auto"),
        ("overflow-y", "scroll"),
    ]
    .into_iter()
    .enumerate()
    {
        let item = declarations[index];
        assert_eq!(item.order(), index);
        assert_eq!(
            item.source().known().unwrap().property(),
            grammar(name).target_property()
        );
        assert_eq!(
            item.source()
                .value_components()
                .serialize()
                .unwrap()
                .as_css(),
            authored
        );
        assert!(matches!(
            item.expansion(),
            CssExpansion::Contributions(CssContributions::Longhands(_))
        ));
    }
}
