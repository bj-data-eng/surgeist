#![forbid(unsafe_code)]

//! Authored overflow controls from CSS Overflow 3 WD (2025-10-07) §§3.2, 3.4, 4.2, 5.1.
//! The clip margin's visual boxes use CSS Box 4 WD (2024-08-04) §2.3.

use surgeist_css::*;

const NAMES: [&str; 4] = [
    "overflow-clip-margin",
    "scroll-behavior",
    "scrollbar-gutter",
    "text-overflow",
];

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
    assert_eq!(item.property(), source.known().unwrap().property());
    assert!(item.source().same_occurrence(source));
    item.ordinary_value().unwrap().clone()
}

#[test]
fn four_distinct_terminals_have_complete_overflow3_support_and_fixed_initials() {
    let mut identities = Vec::new();
    for (name, initial_text) in [
        ("overflow-clip-margin", "0px"),
        ("scroll-behavior", "auto"),
        ("scrollbar-gutter", "auto"),
        ("text-overflow", "clip"),
    ] {
        let grammar = grammar(name);
        let property = grammar.target_property();
        assert_eq!(grammar.name(), name);
        assert_eq!(property.canonical_name(), name);
        assert_eq!(
            grammar.feature_id().as_str(),
            format!(
                "{}.property.{name}",
                if name == "text-overflow" {
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
        let CssPropertyKindRef::Longhand(longhand) = grammar.metadata().unwrap().kind() else {
            panic!("{name} is a terminal longhand")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(!longhand.inherited_by_default());
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), property);
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("{name} has a fixed initial")
        };
        assert_eq!(
            value,
            &one_value(&declaration(name, initial_text)),
            "{name}"
        );
    }
    assert_eq!(identities.len(), 4);
}

#[test]
fn clip_margin_accepts_one_visual_box_and_one_nonnegative_length_in_either_order() {
    let name = "overflow-clip-margin";
    for value in [
        "0px",
        "0",
        "-0px",
        "1px",
        "1e999px",
        "1e-999px",
        "content-box",
        "padding-box",
        "border-box",
        "content-box 2px",
        "2px content-box",
        "padding-box 0",
        "1e999px border-box",
        "calc(1px + 2px)",
        "content-box calc(-1px)",
    ] {
        accepted(name, value);
    }
    for value in [
        "",
        "margin-box",
        "fill-box",
        "stroke-box",
        "view-box",
        "1%",
        "content-box 1%",
        "-1px",
        "-1e-999px",
        "content-box -1px",
        "1",
        "1fr",
        "content-box padding-box",
        "1px 2px",
        "content-box 1px border-box",
        "initial 1px",
    ] {
        invalid(name, value);
    }
}

#[test]
fn three_keyword_controls_accept_their_exact_selected_domains() {
    for value in ["auto", "smooth", "SMOOTH"] {
        accepted("scroll-behavior", value);
    }
    for value in [
        "stable",
        "auto",
        "stable both-edges",
        "both-edges stable",
        "BOTH-EDGES STABLE",
    ] {
        accepted("scrollbar-gutter", value);
    }
    for value in ["clip", "ellipsis", "ELLIPSIS"] {
        accepted("text-overflow", value);
    }
    for value in ["instant", "smooth auto", "scroll", "1px", "initial smooth"] {
        invalid("scroll-behavior", value);
    }
    for value in [
        "both-edges",
        "auto both-edges",
        "stable auto",
        "stable both-edges both-edges",
        "left",
        "1px",
    ] {
        invalid("scrollbar-gutter", value);
    }
    for value in [
        "none",
        "fade",
        "clip ellipsis",
        "ellipsis clip",
        "'…'",
        "\"…\"",
        "initial ellipsis",
    ] {
        invalid("text-overflow", value);
    }
    for (text, expected) in [
        ("clip", CssTextOverflow::Clip),
        ("ellipsis", CssTextOverflow::Ellipsis),
    ] {
        let source = declaration("text-overflow", text);
        let CssKnownPropertyValueRef::TextOverflow(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("existing text-overflow property wrapper")
        };
        assert_eq!(value.value(), &expected);
    }
}

#[test]
fn four_controls_expand_to_one_authored_ordinary_contribution() {
    for (name, values) in [
        (
            "overflow-clip-margin",
            &["content-box", "2px border-box"][..],
        ),
        ("scroll-behavior", &["auto", "smooth"][..]),
        ("scrollbar-gutter", &["auto", "both-edges stable"][..]),
        ("text-overflow", &["clip", "ellipsis"][..]),
    ] {
        for value in values {
            for source in [declaration(name, value), checked(name, value)] {
                let CssExpansion::Contributions(CssContributions::Longhands(items)) =
                    expand_declaration(&source).unwrap()
                else {
                    panic!("{name}:{value} has one ordinary contribution")
                };
                let [item] = items.items() else {
                    panic!("one ordinary contribution")
                };
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
    }
}

#[test]
fn css_wide_all_and_pending_reentry_preserve_property_and_source() {
    let all_source = declaration("all", "initial");
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all_source).unwrap()
    else {
        panic!("symbolic all reset")
    };
    for name in NAMES {
        assert!(!reset.excludes(CssPropertyNameRef::Known(grammar(name).target_property())));
        for (text, keyword) in [
            ("initial", CssGlobalKeyword::Initial),
            ("inherit", CssGlobalKeyword::Inherit),
            ("unset", CssGlobalKeyword::Unset),
            ("revert", CssGlobalKeyword::Revert),
            ("revert-layer", CssGlobalKeyword::RevertLayer),
        ] {
            let source = declaration(name, text);
            checked(name, text);
            let CssExpansion::Contributions(CssContributions::Longhands(items)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("{name} has one symbolic global")
            };
            let [item] = items.items() else {
                panic!("one symbolic global contribution")
            };
            assert_eq!(item.property(), grammar(name).target_property());
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
        }
        for pending in ["var(--control)", "env(--control)"] {
            let source = declaration(name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("{name} keeps substitution pending")
            };
            assert!(handle.source().same_occurrence(&source));
            let invalid_value = if name == "overflow-clip-margin" {
                "margin-box"
            } else {
                "none"
            };
            assert!(matches!(
                handle
                    .reenter(parse_component_values(invalid_value).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
            assert_eq!(
                handle
                    .reenter(parse_component_values("var(--again)").unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            let replacement_text = match name {
                "overflow-clip-margin" => "border-box 2px",
                "scroll-behavior" => "smooth",
                "scrollbar-gutter" => "stable both-edges",
                "text-overflow" => "ellipsis",
                _ => unreachable!(),
            };
            let replacement = parse_component_values(replacement_text).unwrap();
            let CssContributions::Longhands(items) = handle.reenter(replacement.clone()).unwrap()
            else {
                panic!("{name} reenters to one ordinary contribution")
            };
            let [item] = items.items() else {
                panic!("one substituted contribution")
            };
            assert_eq!(item.property(), grammar(name).target_property());
            assert_eq!(
                item.ordinary_value(),
                Some(&one_value(&declaration(name, replacement_text)))
            );
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            let CssContributions::Longhands(globals) = handle
                .reenter(parse_component_values("inherit").unwrap())
                .unwrap()
            else {
                panic!("{name} reenters to one symbolic global")
            };
            assert_eq!(
                globals.items()[0].value(),
                CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
            );
            assert!(globals.items()[0].source().same_occurrence(&source));
        }
    }
}

#[test]
fn normalization_preserves_authored_control_order_after_invalid_recovery() {
    let sheet = ".a{overflow-clip-margin:2px content-box;scroll-behavior:smooth;scrollbar-gutter:both-edges;scrollbar-gutter:stable both-edges;text-overflow:ellipsis}";
    let report = parse_sheet(sheet);
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid gutter diagnostic: {:?}", report.diagnostics())
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
    assert_eq!(declarations.len(), 4);
    for (index, (name, authored)) in [
        ("overflow-clip-margin", "2px content-box"),
        ("scroll-behavior", "smooth"),
        ("scrollbar-gutter", "stable both-edges"),
        ("text-overflow", "ellipsis"),
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
