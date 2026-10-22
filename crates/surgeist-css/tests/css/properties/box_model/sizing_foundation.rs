//! Authored sizing values from Sizing 3 (2026-09-04) §§3.1–3.2, Sizing 4
//! (2026-09-04) §3.2, and its Values 5 (2024-11-11) §10 dependency.

use crate::css::support::property_admission::Responsible::At;
use crate::css::support::property_admission::assert_declaration_rejected as rejected;
use crate::css::support::{
    checked_grammar_declaration as checked, important_declaration as declaration,
    selected_grammar as grammar,
};
use surgeist_css::*;

const PREFERRED: &[&str] = &["width", "height", "inline-size", "block-size"];
const MINIMUM: &[&str] = &[
    "min-width",
    "min-height",
    "min-inline-size",
    "min-block-size",
];
const MAXIMUM: &[&str] = &[
    "max-width",
    "max-height",
    "max-inline-size",
    "max-block-size",
];

fn names() -> impl Iterator<Item = &'static str> {
    PREFERRED.iter().chain(MINIMUM).chain(MAXIMUM).copied()
}

fn accepted(name: &str, value: &str) {
    let property = grammar(name).target_property();
    for declaration in [declaration(name, value), checked(name, value)] {
        assert_eq!(declaration.known().unwrap().property(), property);
        assert_eq!(declaration.importance(), CssImportance::Important);
        assert_eq!(
            declaration.value_components().serialize().unwrap().as_css(),
            value
        );
    }
}

fn one_contribution(declaration: &CssDeclaration) -> CssLonghandValue {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(declaration).unwrap()
    else {
        panic!("one longhand contribution")
    };
    let [item] = values.items() else {
        panic!("exactly one longhand contribution")
    };
    assert_eq!(item.property(), declaration.known().unwrap().property());
    assert!(item.source().same_occurrence(declaration));
    assert_eq!(item.source().importance(), CssImportance::Important);
    assert!(item.replacement_components().is_none());
    item.ordinary_value().unwrap().clone()
}

#[test]
fn twelve_sizing_names_keep_distinct_physical_and_logical_identities() {
    assert_eq!(names().count(), 12);
    for name in names() {
        let grammar = grammar(name);
        assert_eq!(grammar.name(), name);
        assert_eq!(grammar.target_property().canonical_name(), name);
        let support = property_support_metadata(name).expect("selected support metadata");
        assert_eq!(support.property(), grammar.target_property());
        assert_eq!(support.feature().status(), CssSupportStatus::Complete);
        assert_eq!(
            support.feature().source().id().as_str(),
            "I-SIZING3-20260904"
        );
        assert_eq!(
            support.feature().source().url(),
            Some("https://www.w3.org/TR/2026/WD-css-sizing-3-20260904/")
        );
        assert_eq!(support.feature().production(), format!("#propdef-{name}"));
        accepted(name, "1px");
    }
    for (physical, logical) in [
        ("width", "inline-size"),
        ("height", "block-size"),
        ("min-width", "min-inline-size"),
        ("min-height", "min-block-size"),
        ("max-width", "max-inline-size"),
        ("max-height", "max-block-size"),
    ] {
        assert_ne!(
            grammar(physical).target_property(),
            grammar(logical).target_property(),
            "writing-mode mapping must retain both authored identities"
        );
    }
}

#[test]
fn preferred_and_minimum_accept_auto_but_maximum_accepts_none() {
    for name in PREFERRED.iter().chain(MINIMUM).copied() {
        accepted(name, "auto");
        rejected(name, "none", Some("none"));
    }
    for &name in MAXIMUM {
        accepted(name, "none");
        rejected(name, "auto", Some("auto"));
    }
}

#[test]
fn all_twelve_accept_shared_intrinsic_keywords_and_sizing4_extensions() {
    for name in names() {
        for value in [
            "stretch",
            "contain",
            "min-content",
            "max-content",
            "fit-content",
            "fit-content(1px)",
            "fit-content(2%)",
            "fit-content(calc(1px + 2%))",
        ] {
            accepted(name, value);
        }
        for (value, token) in [
            ("fit-content()", Some(")")),
            ("fit-content(-1px)", Some("-1px")),
            ("fit-content(-1e-999%)", Some("-1e-999%")),
        ] {
            rejected(name, value, token);
        }
    }
}

#[test]
fn nonnegative_length_percentage_rejects_negative_literals_without_float_rounding() {
    for name in names() {
        for value in ["0", "1px", "1%", "1e999px", "1e-999px", "1e-999%"] {
            accepted(name, value);
        }
        for (value, token) in [
            ("-1px", Some("-1px")),
            ("-1%", Some("-1%")),
            ("-1e-999px", Some("-1e-999px")),
            ("-1e-999%", Some("-1e-999%")),
            ("1", Some("1")),
            ("1fr", Some("1fr")),
        ] {
            rejected(name, value, token);
        }
        // The range is checked after calculation; syntax must retain the math.
        accepted(name, "calc(1px - 2px)");
        accepted(name, "calc(1px + 2%)");
        rejected(name, "calc(1 + 2)", Some("calc("));
    }
}

#[test]
fn calc_size_basis_and_size_keyword_follow_the_values5_function_grammar() {
    for name in PREFERRED.iter().chain(MINIMUM).copied() {
        accepted(name, "calc-size(auto, size)");
    }
    for &name in MAXIMUM {
        accepted(name, "calc-size(min-content, size)");
        rejected(name, "calc-size(auto, size)", Some("auto"));
        rejected(name, "calc-size(calc-size(auto, 1px), 1px)", Some("auto"));
    }
    for name in names() {
        for value in [
            "calc-size(any, 1px)",
            "calc-size(any, 0px)",
            "calc-size(any, 0%)",
            "calc-size(any, -1px)",
            "calc-size(0px, size + 1px)",
            "calc-size(min-content, size + 1px)",
            "calc-size(min-content, min(size, 1px))",
            "calc-size(fit-content, size)",
            "calc-size(calc-size(min-content, size), size + 1px)",
        ] {
            accepted(name, value);
        }
        for (value, token) in [
            ("calc-size(any, size)", At("size")),
            ("calc-size(any, 0)", At("0")),
            ("calc-size(0, size)", At("0")),
            ("calc-size(any, size * 0)", At("size")),
            ("calc-size(size, 1px)", At("size")),
            ("calc-size(none, 1px)", At("none")),
            ("calc-size(fit-content(1px), size)", At("fit-content(")),
            ("calc-size(min-content)", At(")")),
            ("calc-size(min-content, 1)", At("1")),
            (
                "calc-size(min-content, calc-size(min-content, size))",
                At("calc-size("),
            ),
            ("calc-size(any, 1px / 1px)", At("calc-size(")),
            ("calc-size(any, 1px * 1px)", At("calc-size(")),
            ("calc(size + 1px)", At("size")),
            ("calc(calc-size(min-content, size) + 1px)", At("calc-size(")),
        ] {
            rejected(name, value, token);
        }
    }
}

#[test]
fn all_twelve_have_noninherited_fixed_initials_and_one_ordinary_contribution() {
    for name in names() {
        let property = grammar(name).target_property();
        let CssPropertyKindRef::Longhand(longhand) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("{name} is a terminal longhand")
        };
        assert_eq!(longhand.property().known_property(), property);
        assert!(!longhand.inherited_by_default(), "{name}");
        let initial = longhand.initial_value();
        assert_eq!(initial.property().known_property(), property);
        let CssInitialValueRef::Value(value) = initial.view() else {
            panic!("{name} has a fixed authored initial")
        };
        let expected = if MAXIMUM.contains(&name) {
            "none"
        } else {
            "auto"
        };
        assert_eq!(
            value,
            &one_contribution(&declaration(name, expected)),
            "{name}"
        );

        let ordinary = declaration(name, "1px");
        assert_eq!(
            one_contribution(&ordinary).property().known_property(),
            property
        );
        assert_eq!(
            one_contribution(&checked(name, "1px"))
                .property()
                .known_property(),
            property
        );
    }
}

#[test]
fn css_wide_keywords_remain_whole_values_and_pending_reentry_is_strict() {
    for name in names() {
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
                panic!("symbolic global for {name}")
            };
            let [item] = values.items() else {
                panic!("one symbolic global for {name}")
            };
            assert_eq!(item.property(), grammar(name).target_property());
            assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
            assert!(item.source().same_occurrence(&source));
        }
        rejected(name, "initial 1px", Some("1px"));

        let source = declaration(name, "var(--size)");
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending substitution for {name}")
        };
        assert!(handle.source().same_occurrence(&source));
        let invalid_value = if MAXIMUM.contains(&name) {
            "auto"
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
        let replacement = parse_component_values("1px").unwrap();
        let CssContributions::Longhands(values) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("one substituted longhand for {name}")
        };
        let [item] = values.items() else {
            panic!("one substituted contribution for {name}")
        };
        assert_eq!(item.property(), grammar(name).target_property());
        assert!(item.ordinary_value().is_some());
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
}

#[test]
fn normalization_retains_order_and_distinct_logical_sizing_properties() {
    let report = parse_sheet(
        ".a{width:1px;inline-size:2px;min-inline-size:auto;max-width:none;max-inline-size:3px}",
    );
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let properties = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => {
                Some(value.source().known().unwrap().property())
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        properties,
        [
            "width",
            "inline-size",
            "min-inline-size",
            "max-width",
            "max-inline-size"
        ]
        .map(|name| grammar(name).target_property())
    );

    let pending = parse_sheet(".a{inline-size:var(--size)}");
    assert!(pending.is_clean());
    let normalized = normalize_sheet(pending.syntax()).unwrap();
    assert!(normalized.items().iter().any(|item| matches!(
        item,
        CssNormalizedItem::Declaration(value) if matches!(value.expansion(), CssExpansion::Pending(_))
    )));
}
