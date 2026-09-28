#![forbid(unsafe_code)]

//! Authored grid-auto-flow contracts from Grid 2 (2025-03-26) §7.7 and
//! Grid 3 (2026-01-21) §2.3, with the recorded WebKit normal/dense choice.
//! Canonical serialization of new semantic states needs the future current
//! value API; this RED uses only existing public parsing and expansion APIs.

use surgeist_css::*;

fn grammar() -> CssPropertyGrammar {
    CssPropertyGrammar::from_name("grid-auto-flow").expect("known grid longhand")
}

fn parsed(value: &str) -> CssDeclaration {
    let source = format!("grid-auto-flow:{value} !important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one grid-auto-flow declaration: {source}")
    };
    declaration.clone()
}

fn checked(value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(),
        parse_component_values(value).unwrap(),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked grid-auto-flow {value}: {error:?}"))
}

fn terminal(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).expect("intrinsic grid-auto-flow expansion")
    else {
        panic!("one completed longhand expansion")
    };
    let [item] = values.items() else {
        panic!("one terminal grid-auto-flow contribution")
    };
    assert_eq!(item.property(), CssKnownProperty::GridAutoFlow);
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    values
}

#[test]
fn existing_explicit_axis_spelling_remains_a_parsing_control() {
    for (text, axis, dense) in [
        ("row", CssGridAutoFlowAxis::Row, false),
        ("row dense", CssGridAutoFlowAxis::Row, true),
        ("column", CssGridAutoFlowAxis::Column, false),
        ("column dense", CssGridAutoFlowAxis::Column, true),
    ] {
        for source in [parsed(text), checked(text)] {
            let CssKnownPropertyValueRef::GridAutoFlow(value) =
                source.known().unwrap().property_value().unwrap()
            else {
                panic!("grid-auto-flow typed wrapper")
            };
            let flow = value.i01_subset().expect("explicit axis I01 payload");
            assert_eq!(flow.axis(), axis);
            assert_eq!(flow.dense(), dense);
        }
    }
}

#[test]
fn six_authored_states_and_both_axis_dense_orders_are_accepted() {
    for value in [
        "normal",
        "dense",
        "row",
        "row dense",
        "column",
        "column dense",
        "dense row",
        "dense column",
    ] {
        for source in [parsed(value), checked(value)] {
            let values = terminal(&source);
            assert!(matches!(
                values.items()[0].value(),
                CssContributionValueRef::Ordinary(_)
            ));
            assert_eq!(
                values.items()[0]
                    .ordinary_value()
                    .unwrap()
                    .property()
                    .known_property(),
                CssKnownProperty::GridAutoFlow
            );
            assert!(values.items()[0].replacement_components().is_none());
        }
    }
}

#[test]
fn keyword_matching_decodes_escapes_and_ignores_ascii_case() {
    for value in [
        "NoRmAl",
        "DENSE",
        "R\\6f W",
        "c\\6f lumn",
        "DENSE rOw",
        "CoLuMn d\\65 nse",
    ] {
        for source in [parsed(value), checked(value)] {
            terminal(&source);
        }
    }
}

#[test]
fn normal_cannot_combine_with_dense_and_duplicate_or_conflicting_keywords_reject() {
    for value in [
        "normal dense",
        "dense normal",
        "normal row",
        "row normal",
        "row row",
        "column column",
        "dense dense",
        "normal normal",
        "row column",
        "column row",
        "row dense dense",
        "dense column row",
    ] {
        let source = format!("color:red;grid-auto-flow:{value};color:blue");
        let report = parse_style_attribute(&source);
        assert_eq!(report.syntax().len(), 2, "{source}");
        assert_eq!(report.diagnostics().len(), 1, "{source}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration,
            "{source}"
        );
        assert!(validate_style_attribute(&source).is_err(), "{source}");
        assert!(
            parse_property_value_for_grammar(
                grammar(),
                parse_component_values(value).unwrap(),
                CssImportance::Normal,
            )
            .is_err(),
            "checked accepted {value}"
        );
    }
}

#[test]
fn metadata_has_noninherited_normal_initial_and_all_includes_longhand() {
    let CssPropertyKindRef::Longhand(longhand) = grammar().metadata().unwrap().kind() else {
        panic!("grid-auto-flow longhand metadata")
    };
    assert!(!longhand.inherited_by_default());
    let initial_value = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("ordinary normal initial")
    };
    assert_eq!(
        initial.property().known_property(),
        CssKnownProperty::GridAutoFlow
    );
    // Equality here checks metadata transport; the new current value API must
    // independently assert Normal rather than deriving the oracle from parsing.
    assert_eq!(
        Some(initial),
        terminal(&parsed("normal")).items()[0].ordinary_value()
    );

    let CssPropertyKindRef::UniversalReset(all) = CssKnownProperty::All.metadata().unwrap().kind()
    else {
        panic!("all metadata")
    };
    assert!(!all.excludes(CssPropertyNameRef::Known(CssKnownProperty::GridAutoFlow)));
    let source = parsed("normal");
    assert_eq!(source.importance(), CssImportance::Important);
    let all_source = parse_style_attribute("all:initial!important");
    assert!(all_source.is_clean());
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&all_source.syntax()[0]).unwrap()
    else {
        panic!("symbolic all reset")
    };
    assert_eq!(reset.keyword(), CssGlobalKeyword::Initial);
    assert!(!reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::GridAutoFlow)));
}

#[test]
fn globals_are_terminal_and_pending_reentry_is_strict_repeatable_and_provenanced() {
    for (value, expected) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = parsed(value);
        let values = terminal(&source);
        let item = &values.items()[0];
        assert_eq!(item.value(), CssContributionValueRef::Global(expected));
        assert!(item.ordinary_value().is_none());
    }

    let source = parsed("var(--flow)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("symbolic grid-auto-flow pending substitution")
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
    for value in [
        "normal dense",
        "row column",
        "dense dense",
        "row 1",
        "inherit row",
    ] {
        assert!(
            matches!(
                pending
                    .reenter(parse_component_values(value).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ),
            "{value}"
        );
    }
    for value in ["dense", "normal", "dense column", "inherit"] {
        let replacement = parse_component_values(value).unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("reentry produces terminal longhand")
            };
            let [item] = values.items() else {
                panic!("one replacement contribution")
            };
            assert_eq!(item.property(), CssKnownProperty::GridAutoFlow);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
        }
    }
}

#[test]
fn normalization_keeps_order_importance_and_original_occurrences() {
    let sheet = ".p{color:red;grid-auto-flow:dense!important;grid-auto-flow:column;opacity:1}";
    let report = parse_sheet(sheet);
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
    assert_eq!(values.len(), 4);
    for (index, value) in values.iter().enumerate() {
        assert_eq!(value.order(), index);
        assert!(value.source().same_occurrence(&originals[index]));
    }
    for (index, importance) in [(1, CssImportance::Important), (2, CssImportance::Normal)] {
        let value = values[index];
        assert_eq!(value.source().importance(), importance);
        let CssExpansion::Contributions(CssContributions::Longhands(items)) = value.expansion()
        else {
            panic!("grid-auto-flow longhand expansion")
        };
        assert_eq!(items.items().len(), 1);
        assert_eq!(items.items()[0].property(), CssKnownProperty::GridAutoFlow);
        assert!(items.items()[0].source().same_occurrence(&originals[index]));
    }
}
