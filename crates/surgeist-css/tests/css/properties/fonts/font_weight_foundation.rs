#![forbid(unsafe_code)]

//! Existing-public-API behavior from CSS Fonts 4 WD (2026-09-07) §§2.2 and 2.7.
//! Exact typed numeric payloads follow with the functional weight model.

use surgeist_css::*;

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).expect("font grammar")
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one retained declaration: {source}");
    };
    declaration.clone()
}

fn checked(name: &str, value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(name),
        parse_component_values(value).expect("valid component stream"),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"))
}

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
        assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
        assert_eq!(report.syntax().len(), 2, "neighbors survive: {source}");
        assert!(validate_style_attribute(&source).is_err());
    }
    for invalid in ["0", "1001", "50%", "400px", "normal bold"] {
        assert!(
            parse_property_value_for_grammar(
                grammar("font-weight"),
                parse_component_values(invalid).unwrap(),
                CssImportance::Normal,
            )
            .is_err(),
            "checked construction accepted {invalid}"
        );
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
fn symbolic_global_weights_contribute_once_without_losing_importance() {
    for (spelling, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration("font-weight", spelling);
        checked("font-weight", spelling);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).expect("weight global expansion")
        else {
            panic!("one symbolic longhand contribution");
        };
        let [item] = values.items() else {
            panic!("one weight contribution");
        };
        assert_eq!(item.property(), CssKnownProperty::FontWeight);
        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&declaration("all", "initial")).unwrap()
    else {
        panic!("all contributes its universal reset");
    };
    assert!(!reset.excludes(CssPropertyNameRef::Known(CssKnownProperty::FontWeight)));
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
