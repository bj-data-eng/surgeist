#![forbid(unsafe_code)]

//! Existing-public-API contracts from CSS Fonts 4 WD (2026-09-07) §§2.4, 2.7.
//! Typed angle and authored-value contracts follow with the functional model.

use surgeist_css::*;

fn grammar() -> CssPropertyGrammar {
    CssPropertyGrammar::from_name("font-style").expect("font-style grammar")
}

fn declaration(name: &str, value: &str) -> CssDeclaration {
    let source = format!("{name}:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration: {source}");
    };
    declaration.clone()
}

fn checked(value: &str) -> CssDeclaration {
    parse_property_value_for_grammar(
        grammar(),
        parse_component_values(value).expect("component stream"),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked font-style:{value}: {error:?}"))
}

#[test]
fn normal_and_italic_remain_valid_style_controls() {
    for value in ["normal", "italic", "oblique"] {
        let parsed = declaration("font-style", value);
        assert_eq!(
            parsed.known().unwrap().property(),
            CssKnownProperty::FontStyle
        );
        assert_eq!(parsed.importance(), CssImportance::Important);
        assert_eq!(
            checked(value).known().unwrap().property(),
            CssKnownProperty::FontStyle
        );
    }
}

#[test]
fn property_accepts_direction_keywords_bounded_angles_and_symbolic_angle_math() {
    for value in [
        "left",
        "right",
        "oblique 90deg",
        "oblique -90deg",
        "oblique 100grad",
        "oblique .25turn",
        "oblique 1.5707963267948966rad",
        "oblique calc(100deg)",
    ] {
        for source in [declaration("font-style", value), checked(value)] {
            assert_eq!(
                source.known().unwrap().property(),
                CssKnownProperty::FontStyle
            );
            assert_eq!(source.importance(), CssImportance::Important);
        }
    }
}

#[test]
fn exact_outside_angles_and_wrong_domains_drop_only_the_style_declaration() {
    for invalid in [
        "oblique 90.00000000000000000001deg",
        "oblique 100.00000000000000000001grad",
        "oblique .25000000000000000001turn",
        "oblique 1.5707963267948968rad",
        "oblique 0",
        "oblique 1px",
        "oblique 1%",
        "oblique calc(16px)",
        "oblique 10deg 20deg",
        "left right",
    ] {
        let source = format!("color:red;font-style:{invalid};color:blue");
        let report = parse_style_attribute(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid style diagnostic: {source}: {report:?}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_ne!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
        assert_eq!(
            report.syntax().len(),
            2,
            "valid neighbors survive: {source}"
        );
        assert!(validate_style_attribute(&source).is_err());
        assert!(
            parse_property_value_for_grammar(
                grammar(),
                parse_component_values(invalid).unwrap(),
                CssImportance::Normal,
            )
            .is_err(),
            "checked construction accepted {invalid}"
        );
    }
}

#[test]
fn inherited_normal_style_has_one_longhand_contribution_and_strict_reentry() {
    let property = CssKnownProperty::FontStyle;
    let CssPropertyKindRef::Longhand(longhand) = grammar().metadata().unwrap().kind() else {
        panic!("font-style is a longhand");
    };
    assert!(longhand.inherited_by_default());
    assert_eq!(longhand.property().known_property(), property);
    let initial_value = longhand.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("normal is the fixed initial value");
    };
    let normal = declaration("font-style", "normal");
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&normal).expect("intrinsic style expansion")
    else {
        panic!("one style contribution");
    };
    let [item] = values.items() else {
        panic!("one style contribution");
    };
    assert_eq!(item.property(), property);
    assert_eq!(item.ordinary_value(), Some(initial));
    assert!(item.source().same_occurrence(&normal));
    assert_eq!(item.source().importance(), CssImportance::Important);

    for (spelling, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
    ] {
        let source = declaration("font-style", spelling);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("symbolic global contributes once");
        };
        let [item] = values.items() else {
            panic!("one global contribution");
        };
        assert_eq!(item.property(), property);
        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
        assert!(item.source().same_occurrence(&source));
    }

    let source = declaration("font-style", "var(--slant)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("unresolved style remains pending");
    };
    assert_eq!(
        pending
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    assert!(matches!(
        pending
            .reenter(parse_component_values("oblique 0").unwrap())
            .unwrap_err()
            .kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
    let replacement = parse_component_values("oblique 90deg").unwrap();
    let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap() else {
        panic!("strict replacement contributes a style");
    };
    let [item] = values.items() else {
        panic!("one substituted style contribution");
    };
    assert_eq!(item.property(), property);
    assert!(item.source().same_occurrence(&source));
    assert_eq!(item.replacement_components(), Some(&replacement));
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&declaration("all", "initial")).unwrap()
    else {
        panic!("all contributes its universal reset");
    };
    assert!(!reset.excludes(CssPropertyNameRef::Known(property)));
}

#[test]
fn shorthand_consumes_angle_math_but_leaves_size_math_for_font_size() {
    for value in [
        "italic 16px serif",
        "oblique 16px serif",
        "oblique 90deg 16px serif",
        "oblique calc(10deg) 16px serif",
        "oblique calc(16px) serif",
    ] {
        declaration("font", value);
    }
    for invalid in [
        "oblique 90.00000000000000000001deg 16px serif",
        "oblique calc(10deg) italic 16px serif",
    ] {
        let source = format!("color:red;font:{invalid};color:blue");
        let report = parse_style_attribute(&source);
        let [diagnostic] = report.diagnostics() else {
            panic!("one invalid shorthand diagnostic: {source}: {report:?}");
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        assert_eq!(report.syntax().len(), 2);
    }
}
