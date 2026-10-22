//! Existing-public-API behavior from CSS Fonts 4 WD (2026-09-07) §§2.2 and 2.7.
//! Exact typed numeric payloads follow with the functional weight model.

use crate::css::support::checked_grammar_declaration as checked;
use crate::css::support::important_declaration as declaration;
use crate::css::support::known_property;
use crate::css::support::named_property::assert_checked_value_rejected_at;
use crate::css::support::selected_grammar as grammar;
use surgeist_css::*;

#[test]
fn property_accepts_keywords_exact_numbers_and_number_math() {
    let property = CssKnownProperty::FontWeight;
    assert_eq!(grammar("FONT-WEIGHT").target_property(), property);
    for value in [
        "normal",
        "bold",
        "bolder",
        "lighter",
        "1",
        "1000",
        "350.5",
        "1e3",
        "1e0",
        "calc(350 + 50)",
        "min(350, 400)",
    ] {
        for source in [
            declaration("font-weight", value),
            checked("font-weight", value),
        ] {
            assert_eq!(source.known().unwrap().property(), property, "{value}");
            assert_eq!(source.importance(), CssImportance::Important);
        }
    }
}

#[test]
fn exact_bounds_and_wrong_numeric_domains_drop_only_the_weight_declaration() {
    for invalid in [
        "0",
        "1001",
        "0.99999999999999999999",
        "1000.00000000000000000001",
        "1e-999999",
        "1e999999",
        "50%",
        "400px",
        "normal bold",
    ] {
        let source = format!("color:red;font-weight:{invalid};color:blue");
        let report = parse_style_attribute(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid weight diagnostic: {source}: {report:?}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(
            diagnostic.error().code(),
            CssErrorCode::InvalidPropertyValue
        );
        assert_eq!(report.syntax().len(), 2, "neighbors survive: {source}");
        assert!(validate_style_attribute(&source).is_err());
    }
    for (invalid, rest) in [
        ("0", "0"),
        ("1001", "1001"),
        ("50%", "50%"),
        ("400px", "400px"),
        ("normal bold", "bold"),
    ] {
        assert_checked_value_rejected_at(known_property("font-weight"), invalid, rest);
    }
}

#[test]
fn inherited_normal_weight_has_one_intrinsic_contribution_and_pending_reentry() {
    let property = CssKnownProperty::FontWeight;
    let CssPropertyKindRef::Longhand(longhand) = grammar("font-weight")
        .metadata()
        .expect("weight metadata")
        .kind()
    else {
        panic!("font-weight is a longhand");
    };
    assert!(longhand.inherited_by_default());
    assert_eq!(longhand.property().known_property(), property);
    let initial_value = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("normal is a fixed initial");
    };
    let ordinary = declaration("font-weight", "normal");
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&ordinary).expect("weight expansion")
    else {
        panic!("one ordinary weight contribution");
    };
    let [item] = values.items() else {
        panic!("one weight contribution");
    };
    assert_eq!(item.property(), property);
    assert_eq!(item.ordinary_value(), Some(initial));
    assert!(item.source().same_occurrence(&ordinary));
    assert_eq!(item.source().importance(), CssImportance::Important);

    let symbolic = declaration("font-weight", "var(--weight)");
    let CssExpansion::Pending(pending) = expand_declaration(&symbolic).unwrap() else {
        panic!("unresolved weight remains pending");
    };
    assert!(pending.source().same_occurrence(&symbolic));
    let replacement = parse_component_values("350.5").unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("replacement contributes a longhand");
    };
    let [item] = values.items() else {
        panic!("one substituted weight contribution");
    };
    assert_eq!(item.property(), property);
    assert!(item.source().same_occurrence(&symbolic));
    assert_eq!(item.replacement_components(), Some(&replacement));
    assert!(matches!(
        pending
            .reenter(parse_component_values("0").unwrap())
            .unwrap_err()
            .kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
    assert_eq!(
        pending
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
}

#[test]
fn shorthand_distinguishes_fractional_and_symbolic_weight_from_size() {
    for value in [
        "normal 16px serif",
        "400 16px serif",
        "350.5 16px serif",
        "calc(350 + 50) 16px serif",
        "400 calc(16px + 2px) serif",
    ] {
        declaration("font", value);
        checked("font", value);
    }
    for invalid in ["0 16px serif", "350.5 bold 16px serif"] {
        let source = format!("color:red;font:{invalid};color:blue");
        let report = parse_style_attribute(&source);
        assert_eq!(
            report.syntax().len(),
            2,
            "bad shorthand drops locally: {source}"
        );
        let [diagnostic] = report.diagnostics() else {
            panic!("one shorthand diagnostic: {source}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    }
}
