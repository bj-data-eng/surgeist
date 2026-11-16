//! Authored overflow controls from CSS Overflow 3 WD (2025-10-07) §§3.2, 3.4, 4.2, 5.1.
//! The clip margin's visual boxes use CSS Box 4 WD (2024-08-04) §2.3, and its
//! signed length offset follows the retained Overflow 3 Editor's Draft capture
//! (2026-08-13) §3.2.

use crate::css::support::named_property::accepted_by_attribute_and_grammar as accepted;
use crate::css::support::named_property::invalid_named_value;
use crate::css::support::named_property::invalid_named_value_at as invalid;
use crate::css::support::named_property::sole_longhand_contribution;
use crate::css::support::{
    checked_grammar_declaration as checked, important_declaration as declaration,
    selected_grammar as grammar,
};
use surgeist_css::CssParser;
use surgeist_css::component::CssComponentValues;
use surgeist_css::conformance::{CssSupportStatus, property_support_metadata};
use surgeist_css::declaration::{CssDeclaration, CssGlobalKeyword, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::normalize::CssNormalizedItem;
use surgeist_css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssInitialValueRef,
    CssKnownPropertyValueRef, CssLonghandValue, CssPropertyKindRef, CssPropertyNameRef,
};
use surgeist_css::values::text::CssTextOverflow;

const NAMES: [&str; 4] = [
    "overflow-clip-margin",
    "scroll-behavior",
    "scrollbar-gutter",
    "text-overflow",
];

fn one_value(source: &CssDeclaration) -> CssLonghandValue {
    sole_longhand_contribution(source)
        .ordinary_value()
        .unwrap()
        .clone()
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
fn clip_margin_accepts_one_visual_box_and_one_signed_length_in_either_order() {
    // Overflow 3 Editor's Draft (2026-08-13) section 3.2: `<visual-box> || <length>`;
    // "Negative values indicate insets", so the offset is an ordinary signed length.
    let name = "overflow-clip-margin";
    for value in [
        "0px",
        "0",
        "-0px",
        "1px",
        "-1px",
        "-0.5em",
        "1e999px",
        "1e-999px",
        "-1e-999px",
        "-1e999px",
        "content-box",
        "padding-box",
        "border-box",
        "content-box 2px",
        "2px content-box",
        "content-box -1px",
        "-1px content-box",
        "border-box -3.5px",
        "-3.5px padding-box",
        "padding-box 0",
        "padding-box -0",
        "1e999px border-box",
        "-1e999px border-box",
        "calc(1px + 2px)",
        "calc(-1px)",
        "calc(1px - 2px)",
        "calc(1px - 2em)",
        "calc(2em - 3px)",
        "content-box calc(-1px)",
        "calc(-1px) content-box",
        "border-box min(-1px, 2px)",
        "max(-1px, -2px) padding-box",
    ] {
        accepted(name, value);
    }
}

#[test]
fn clip_margin_rejects_other_boxes_percentages_unitless_numbers_and_duplicates() {
    let name = "overflow-clip-margin";
    for (value, rest) in [
        ("", ""),
        ("margin-box", "margin-box"),
        ("fill-box", "fill-box"),
        ("stroke-box", "stroke-box"),
        ("view-box", "view-box"),
        ("1%", "1%"),
        ("-1%", "-1%"),
        ("content-box 1%", "1%"),
        ("content-box -1%", "-1%"),
        ("1", "1"),
        ("-1", "-1"),
        ("content-box -1", "-1"),
        ("1fr", "1fr"),
        ("-1fr", "-1fr"),
        ("content-box padding-box", "padding-box"),
        ("1px 2px", "2px"),
        ("-1px -2px", "-2px"),
        ("-1px 2px", "2px"),
        ("content-box 1px border-box", "border-box"),
        ("-1px content-box padding-box", "padding-box"),
        ("-1px content-box -2px", "-2px"),
        ("initial 1px", "1px"),
        ("-1px initial", "initial"),
    ] {
        invalid(name, value, rest);
    }
}

#[test]
fn clip_margin_percentages_stay_rejected_in_literals_and_math() {
    // Overflow 3 section 3.2: Percentages are n/a, so a percentage never
    // enters the offset, whether alone or inside a calculation.
    let name = "overflow-clip-margin";
    for value in [
        "calc(1px + 1%)",
        "calc(-1px - 1%)",
        "content-box calc(1% - 1px)",
        "calc(-1%)",
    ] {
        invalid_named_value(name, value);
    }
}

#[test]
fn clip_margin_parsed_and_checked_declarations_hold_the_same_signed_value() {
    let name = "overflow-clip-margin";
    for value in [
        "-1px",
        "-0px",
        "-1e-999px",
        "-1.5em content-box",
        "border-box -2px",
        "calc(1px - 2px)",
        "padding-box calc(1px - 2em)",
        "min(-1px, 2px) border-box",
    ] {
        assert_eq!(
            one_value(&declaration(name, value)),
            one_value(&checked(name, value)),
            "{value}"
        );
    }
    // Sign, omission and exact tiny magnitude each distinguish specified values.
    assert_ne!(
        one_value(&declaration(name, "-1px")),
        one_value(&declaration(name, "1px"))
    );
    assert_ne!(
        one_value(&declaration(name, "-1px")),
        one_value(&declaration(name, "padding-box -1px"))
    );
    assert_ne!(
        one_value(&declaration(name, "-1e-999px")),
        one_value(&declaration(name, "0px"))
    );
}

#[test]
fn clip_margin_pending_reentry_admits_signed_replacements_and_rejects_percentages() {
    let name = "overflow-clip-margin";
    let source = declaration(name, "var(--inset)");
    let CssExpansion::Pending(handle) = source.expand().unwrap() else {
        panic!("clip margin keeps var() pending")
    };
    for replacement_text in ["-2px", "content-box -2px", "calc(1px - 2em) border-box"] {
        let replacement =
            CssComponentValues::parse(&CssParser::default(), replacement_text).unwrap();
        let CssContributions::Longhands(items) = handle.reenter(replacement.clone()).unwrap()
        else {
            panic!("{replacement_text} reenters to one ordinary contribution")
        };
        let [item] = items.items() else {
            panic!("one substituted contribution")
        };
        assert_eq!(
            item.ordinary_value(),
            Some(&one_value(&declaration(name, replacement_text))),
            "{replacement_text}"
        );
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
    for invalid_text in ["-1%", "-1", "-1px -2px", "calc(1px + 1%)"] {
        assert!(
            handle
                .reenter(CssComponentValues::parse(&CssParser::default(), invalid_text).unwrap())
                .unwrap_err()
                .replacement_error()
                .is_some(),
            "{invalid_text}"
        );
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
    for (value, rest) in [
        ("instant", "instant"),
        ("smooth auto", "auto"),
        ("scroll", "scroll"),
        ("1px", "1px"),
        ("initial smooth", "smooth"),
    ] {
        invalid("scroll-behavior", value, rest);
    }
    // Overflow 3: `both-edges` needs `stable`, and the value ends after it,
    // so the last authored token is responsible.
    invalid("scrollbar-gutter", "both-edges", "both-edges");
    for (value, rest) in [
        ("auto both-edges", "both-edges"),
        ("stable auto", "auto"),
        ("stable both-edges both-edges", "both-edges"),
        ("left", "left"),
        ("1px", "1px"),
    ] {
        invalid("scrollbar-gutter", value, rest);
    }
    for (value, rest) in [
        ("none", "none"),
        ("fade", "fade"),
        ("clip ellipsis", "ellipsis"),
        ("ellipsis clip", "clip"),
        ("'…'", "'…'"),
        ("\"…\"", r#""…""#),
        ("initial ellipsis", "ellipsis"),
    ] {
        invalid("text-overflow", value, rest);
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
                    source.expand().unwrap()
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
        all_source.expand().unwrap()
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
                source.expand().unwrap()
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
            let CssExpansion::Pending(handle) = source.expand().unwrap() else {
                panic!("{name} keeps substitution pending")
            };
            assert!(handle.source().same_occurrence(&source));
            let invalid_value = if name == "overflow-clip-margin" {
                "margin-box"
            } else {
                "none"
            };
            assert!(
                handle
                    .reenter(
                        CssComponentValues::parse(&CssParser::default(), invalid_value).unwrap()
                    )
                    .unwrap_err()
                    .replacement_error()
                    .is_some()
            );
            assert!(
                handle
                    .reenter(
                        CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap()
                    )
                    .unwrap_err()
                    .is_residual_substitution()
            );
            let replacement_text = match name {
                "overflow-clip-margin" => "border-box 2px",
                "scroll-behavior" => "smooth",
                "scrollbar-gutter" => "stable both-edges",
                "text-overflow" => "ellipsis",
                _ => unreachable!(),
            };
            let replacement =
                CssComponentValues::parse(&CssParser::default(), replacement_text).unwrap();
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
                .reenter(CssComponentValues::parse(&CssParser::default(), "inherit").unwrap())
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
    let report = CssParser::default().parse_sheet(sheet);
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid gutter diagnostic: {:?}", report.diagnostics())
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    let normalized = report.syntax().normalize(Default::default()).unwrap();
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
                .serialize_tokens(Default::default())
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
