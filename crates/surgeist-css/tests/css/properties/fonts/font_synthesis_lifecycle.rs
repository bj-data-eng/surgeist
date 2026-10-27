//! Independent expectations: pinned Fonts 4 WD (2026-09-07) §§2.8.1–2.8.5,
//! especially the four-column shorthand expansion table. Source ownership:
//! references/css-fonts-4--WD-css-fonts-4-20260907--03626a0c8565.md.
//! https://www.w3.org/TR/2026/WD-css-fonts-4-20260907/#font-synthesis
//! These tests use existing public front doors; new payload/serialization APIs
//! belong in functional tests alongside implementation. Font execution is external.

use crate::css::support::checked_grammar_declaration as checked;
use crate::css::support::important_declaration as declaration;
use crate::css::support::known_property;
use crate::css::support::named_property::assert_checked_value_rejected_at;
use crate::css::support::named_property::longhand_contributions as longhands;
use crate::css::support::selected_grammar as grammar;
use surgeist_css::CssParser;
use surgeist_css::component::{CssComponentValues, CssSerializedOrigin, CssValueOrigin};
use surgeist_css::declaration::{CssDeclaration, CssGlobalKeyword, CssImportance};
use surgeist_css::diagnostic::{CssErrorCode, CssRecoveryAction};
use surgeist_css::normalize::CssNormalizedItem;
use surgeist_css::property::{
    CssContributionValueRef, CssContributions, CssExpansion, CssInitialValueRef,
    CssKnownDeclaredValueRef, CssKnownProperty, CssKnownPropertyValueRef, CssLonghandContributions,
    CssPropertyKindRef, CssPropertyTarget,
};

const MEMBERS: [&str; 4] = [
    "font-synthesis-weight",
    "font-synthesis-style",
    "font-synthesis-small-caps",
    "font-synthesis-position",
];
const CAPABILITIES: [&str; 4] = ["weight", "style", "small-caps", "position"];

fn assert_member_values(values: &CssLonghandContributions, source: &CssDeclaration, mask: u8) {
    assert_eq!(values.items().len(), 4);
    for (index, (item, name)) in values.items().iter().zip(MEMBERS).enumerate() {
        assert_eq!(item.property().canonical_name(), name);
        // Fonts 4's table: included capability -> auto, omitted -> none.
        let keyword = if mask & (1 << index) != 0 {
            "auto"
        } else {
            "none"
        };
        let expected = longhands(&checked(name, keyword));
        assert_eq!(
            item.ordinary_value(),
            expected.items()[0].ordinary_value(),
            "{name}"
        );
        assert!(item.source().same_occurrence(source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
}

#[test]
fn historical_weight_style_none_parsing_and_checked_values_remain_controls() {
    for text in ["none", "weight", "style", "weight style", "STYLE WEIGHT"] {
        let parsed = declaration("font-synthesis", text);
        let constructed = checked("font-synthesis", text);
        assert_eq!(
            parsed.known().unwrap().property(),
            CssKnownProperty::FontSynthesis
        );
        let CssKnownPropertyValueRef::FontSynthesis(left) =
            parsed.known().unwrap().property_value().unwrap()
        else {
            panic!("synthesis")
        };
        let CssKnownPropertyValueRef::FontSynthesis(right) =
            constructed.known().unwrap().property_value().unwrap()
        else {
            panic!("synthesis")
        };
        assert_eq!(left.value(), right.value());
        assert_eq!(left.as_css(), text);
        assert_eq!(constructed.importance(), CssImportance::Important);
    }
}

#[test]
fn historical_invalid_shorthand_recovery_and_typed_checked_errors_remain_controls() {
    for (text, rest) in [
        ("weight weight", "weight"),
        ("style style", "style"),
        ("none weight", "weight"),
        ("weight none", "none"),
        ("weight, style", ", style"),
        ("auto", "auto"),
        ("oblique-only", "oblique-only"),
    ] {
        let source = format!("color:red;font-synthesis:{text};color:blue");
        let report = CssParser::default().parse_block_contents(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one diagnostic: {source}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(report.syntax().len(), 2);
        assert!(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .is_err()
        );
        assert_checked_value_rejected_at(CssKnownProperty::FontSynthesis, text, rest);
    }
}

#[test]
fn shorthand_global_and_pending_parse_admission_remains_a_control() {
    for text in ["initial", "inherit", "unset", "revert", "revert-layer"] {
        let source = declaration("font-synthesis", text);
        assert!(matches!(
            source.known().unwrap().declared_value(),
            CssKnownDeclaredValueRef::Global(_)
        ));
    }
    for text in ["var(--synthesis)", "env(synthesis)"] {
        let source = declaration("font-synthesis", text);
        assert!(matches!(
            source.known().unwrap().declared_value(),
            CssKnownDeclaredValueRef::SubstitutionDependent(_)
        ));
        let constructed = checked("font-synthesis", text);
        assert_eq!(
            source
                .value_components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            text
        );
        assert_eq!(
            constructed
                .value_components()
                .serialize_tokens(Default::default())
                .unwrap()
                .as_css(),
            text
        );
    }
}

#[test]
fn four_capabilities_and_representative_permutations_are_admitted() {
    // Singletons establish new terminals; reversed pairs and two full orders
    // establish || order independence without redundant exhaustive permutations.
    for text in [
        "small-caps",
        "position",
        "position small-caps",
        "small-caps weight",
        "position style",
        "weight style small-caps position",
        "POSITION SMALL-CAPS STYLE WEIGHT",
    ] {
        let source = declaration("font-synthesis", text);
        assert!(source.known().unwrap().property_value().is_some());
        assert!(
            CssParser::default()
                .parse_block_contents(format!("font-synthesis:{text}"))
                .into_validation_result()
                .is_ok()
        );
        assert!(
            checked("font-synthesis", text)
                .known()
                .unwrap()
                .property_value()
                .is_some()
        );
    }
    for (invalid, rest) in [
        ("small-caps small-caps", "small-caps"),
        ("position position", "position"),
        ("none position", "position"),
        ("small-caps auto", "auto"),
        ("oblique-only weight", "oblique-only weight"),
    ] {
        let report = CssParser::default()
            .parse_block_contents(format!("font-synthesis:{invalid};color:red"));
        assert_eq!(report.syntax().len(), 1, "{invalid}");
        let [diagnostic] = report.diagnostics() else {
            panic!("one shorthand grammar error")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_checked_value_rejected_at(known_property("font-synthesis"), invalid, rest);
    }
}

#[test]
fn four_longhands_admit_only_their_selected_keyword_grammars_with_exact_origins() {
    for name in MEMBERS {
        let keywords: &[&str] = if name == "font-synthesis-style" {
            &["auto", "none", "OBLIQUE-ONLY"]
        } else {
            &["auto", "none"]
        };
        for keyword in keywords {
            let text = format!("{name}:{keyword}!important");
            let report = CssParser::default().parse_block_contents(&text);
            assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
            let source = &report.syntax()[0];
            assert_eq!(source.known().unwrap().property().canonical_name(), name);
            assert_eq!(source.importance(), CssImportance::Important);
            assert_eq!(source.position().unwrap().byte_offset().value(), 0);
            let origin = source.parsed_value().unwrap();
            assert_eq!(origin.span().start().byte_offset().value(), name.len() + 1);
            assert_eq!(
                origin.span().end().byte_offset().value(),
                name.len() + 1 + keyword.len()
            );
            assert!(source.known().unwrap().property_value().is_some());
        }
    }
}

#[test]
fn checked_longhand_admission_preserves_component_origins_and_typed_rejection() {
    for name in MEMBERS {
        let components = CssComponentValues::parse(&CssParser::default(), "auto").unwrap();
        let before = components.clone();
        let source = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Grammar(grammar(name)),
            components,
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(source.value_components(), &before);
        assert!(source.position().is_none());
        assert_eq!(source.importance(), CssImportance::Important);
        let error = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Grammar(grammar(name)),
            CssComponentValues::parse(&CssParser::default(), "auto none").unwrap(),
            CssImportance::Normal,
        )
        .unwrap_err();
        assert!(error.code().is_some());
        let CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin)) = error.origin() else {
            panic!("original none token")
        };
        assert_eq!(origin.span().start().byte_offset().value(), 5);
        assert_eq!(origin.span().end().byte_offset().value(), 9);
    }
}

#[test]
fn longhand_grammar_errors_recover_locally_as_known_property_failures() {
    for name in MEMBERS {
        let invalid: &[&str] = if name == "font-synthesis-style" {
            &["weight", "auto none", "normal", "1", "auto,none"]
        } else {
            &["oblique-only", "auto none", "normal", "1", "auto,none"]
        };
        for text in invalid {
            let report = CssParser::default()
                .parse_block_contents(format!("color:red;{name}:{text};color:blue"));
            assert_eq!(report.syntax().len(), 2, "{name}:{text}");
            let [diagnostic] = report.diagnostics() else {
                panic!("one grammar error")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            assert_ne!(
                diagnostic.error().code(),
                CssErrorCode::UnknownProperty,
                "{name}:{text}"
            );
        }
    }
}

#[test]
fn shorthand_metadata_has_exactly_four_settable_members_and_no_reset_only_members() {
    let CssPropertyKindRef::Shorthand(metadata) =
        grammar("font-synthesis").metadata().unwrap().kind()
    else {
        panic!("shorthand")
    };
    assert_eq!(
        metadata
            .settable_members()
            .iter()
            .map(|property| property.known_property().canonical_name())
            .collect::<Vec<_>>(),
        MEMBERS
    );
    assert!(metadata.reset_only_members().is_empty());
}

#[test]
fn each_longhand_is_inherited_and_has_intrinsic_auto_initial() {
    for name in MEMBERS {
        let CssPropertyKindRef::Longhand(metadata) = grammar(name).metadata().unwrap().kind()
        else {
            panic!("longhand")
        };
        assert!(metadata.inherited_by_default());
        assert_eq!(metadata.property().known_property().canonical_name(), name);
        let initial = metadata.initial_value();
        let CssInitialValueRef::Value(initial) = initial.view() else {
            panic!("fixed auto initial")
        };
        let source = declaration(name, "auto");
        let values = longhands(&source);
        assert_eq!(values.items().len(), 1);
        assert_eq!(values.items()[0].ordinary_value(), Some(initial));
        assert!(values.items()[0].source().same_occurrence(&source));
    }
}

#[test]
fn complete_shorthand_table_projects_selected_auto_and_omitted_none() {
    // Sixteen distinct semantic combinations, not permutations of the same set.
    for mask in 0u8..16 {
        let text = if mask == 0 {
            "none".to_owned()
        } else {
            CAPABILITIES
                .iter()
                .enumerate()
                .filter_map(|(index, name)| (mask & (1 << index) != 0).then_some(*name))
                .collect::<Vec<_>>()
                .join(" ")
        };
        let source = declaration("font-synthesis", &text);
        let before = source.clone();
        let values = longhands(&source);
        assert_member_values(&values, &source, mask);
        assert!(
            values
                .items()
                .iter()
                .all(|item| item.replacement_components().is_none())
        );
        assert!(source.same_occurrence(&before));
        assert_eq!(source.value_components(), before.value_components());
    }
}

#[test]
fn pending_reentry_is_strict_and_preserves_source_and_replacement_provenance() {
    for name in std::iter::once("font-synthesis").chain(MEMBERS) {
        for symbolic in ["var(--synthesis)", "env(synthesis)"] {
            let source = declaration(name, symbolic);
            let before = source.clone();
            let CssExpansion::Pending(pending) = source.expand().unwrap() else {
                panic!("pending")
            };
            assert!(pending.source().same_occurrence(&source));
            let text = if name == "font-synthesis" {
                "position weight"
            } else if name == "font-synthesis-style" {
                "oblique-only"
            } else {
                "none"
            };
            let replacement = CssComponentValues::parse(&CssParser::default(), text).unwrap();
            let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("completed")
            };
            if name == "font-synthesis" {
                assert_member_values(&values, &source, 0b1001);
            } else {
                assert_eq!(
                    values.items()[0].ordinary_value(),
                    longhands(&checked(name, text)).items()[0].ordinary_value()
                );
            }
            for item in values.items() {
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().importance(), CssImportance::Important);
                assert_eq!(item.replacement_components(), Some(&replacement));
            }
            let CssContributions::Longhands(globals) = pending
                .reenter(CssComponentValues::parse(&CssParser::default(), "initial").unwrap())
                .unwrap()
            else {
                panic!("global replacement")
            };
            assert_eq!(
                globals.items().len(),
                if name == "font-synthesis" { 4 } else { 1 }
            );
            assert!(globals.items().iter().all(|item| item.value()
                == CssContributionValueRef::Global(CssGlobalKeyword::Initial)
                && item.source().same_occurrence(&source)));
            for invalid in ["auto none", "auto!important", "none;"] {
                assert!(
                    pending
                        .reenter(CssComponentValues::parse(&CssParser::default(), invalid).unwrap())
                        .unwrap_err()
                        .replacement_error()
                        .is_some(),
                    "{name}: {invalid}"
                );
            }
            assert!(
                pending
                    .reenter(
                        CssComponentValues::parse(&CssParser::default(), "var(--again)").unwrap()
                    )
                    .unwrap_err()
                    .is_residual_substitution()
            );
            assert!(source.same_occurrence(&before));
            assert_eq!(source.value_components(), before.value_components());
        }
    }
}

#[test]
fn normalization_preserves_repeated_shorthand_order_importance_and_occurrence() {
    let text = ".a{font-synthesis:weight!important;color:red;font-synthesis:none;font-synthesis:style!important}";
    let report = CssParser::default().parse_sheet(text);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
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
    for (index, (value, (property, authored, importance))) in declarations
        .iter()
        .zip([
            ("font-synthesis", "weight", CssImportance::Important),
            ("color", "red", CssImportance::Normal),
            ("font-synthesis", "none", CssImportance::Normal),
            ("font-synthesis", "style", CssImportance::Important),
        ])
        .enumerate()
    {
        assert_eq!(value.order(), index);
        assert_eq!(
            value.source().known().unwrap().property().canonical_name(),
            property
        );
        assert_eq!(value.source().importance(), importance);
        let origin = value.source().parsed_value().unwrap();
        let offset = text.find(&format!("{property}:{authored}")).unwrap() + property.len() + 1;
        assert_eq!(origin.span().start().byte_offset().value(), offset);
        assert_eq!(
            origin.span().end().byte_offset().value(),
            offset + authored.len()
        );
        if property == "font-synthesis" {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) =
                value.expansion()
            else {
                panic!("expanded synthesis")
            };
            assert_eq!(values.items().len(), 4);
            assert!(
                values
                    .items()
                    .iter()
                    .all(|item| item.source().same_occurrence(value.source()))
            );
        }
    }
}
