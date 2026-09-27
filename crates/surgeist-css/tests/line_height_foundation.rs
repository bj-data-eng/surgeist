#![forbid(unsafe_code)]

//! Existing-public-API behavior from CSS 2.1 line-height and Fonts 4 §2.7.
//! Exact typed payload contracts follow with the functional model.

use surgeist_css::*;

fn grammar(name: &str) -> CssPropertyGrammar {
    CssPropertyGrammar::from_name(name).unwrap_or_else(|| panic!("known grammar: {name}"))
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
        parse_component_values(value).expect("component stream"),
        CssImportance::Important,
    )
    .unwrap_or_else(|error| panic!("checked {name}:{value}: {error:?}"))
}

fn rejected_locally(name: &str, value: &str) {
    let source = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid declaration diagnostic: {source}");
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

#[test]
fn established_number_length_percentage_and_font_slash_forms_remain_valid() {
    let property = CssKnownProperty::LineHeight;
    assert_eq!(grammar("LINE-HEIGHT").target_property(), property);
    for value in [
        "normal",
        "0",
        "-0",
        "1.2",
        "120%",
        "1em",
        "0px",
        "-0px",
        "-0%",
        "calc(-1)",
        "calc(-1px)",
        "calc(1em + 10%)",
    ] {
        for source in [
            declaration("line-height", value),
            checked("line-height", value),
        ] {
            assert_eq!(source.known().unwrap().property(), property, "{value}");
            assert_eq!(source.importance(), CssImportance::Important);
        }
    }
    for value in [
        "16px/normal serif",
        "16px/0 serif",
        "16px/1.2 serif",
        "16px/120% serif",
        "16px/1em serif",
        "16px/calc(-1) serif",
    ] {
        assert_eq!(
            declaration("font", value).known().unwrap().property(),
            CssKnownProperty::Font
        );
    }
    for invalid in ["auto", "1deg", "normal 1.2", "1px 2px"] {
        rejected_locally("line-height", invalid);
    }
    rejected_locally("line-height", "math");
    rejected_locally("font", "16px/math serif");
    let zero = declaration("line-height", "0");
    let CssKnownPropertyValueRef::LineHeight(value) =
        zero.known().unwrap().property_value().unwrap()
    else {
        panic!("unitless zero is line-height");
    };
    assert!(matches!(value.line_height(), CssLineHeight::Number(_)));
}

#[test]
fn tiny_negative_number_is_rejected_before_float_underflow() {
    for invalid in ["-1e-999", "-1e-50", "-0.01"] {
        rejected_locally("line-height", invalid);
    }
}

#[test]
fn tiny_negative_length_and_percentage_are_rejected_before_float_underflow() {
    for invalid in ["-1e-999px", "-1e-999%", "-1e-50px", "-1e-50%"] {
        rejected_locally("line-height", invalid);
    }
}

#[test]
fn huge_and_tiny_positive_ordinary_values_remain_authored_values() {
    for value in [
        "1e999", "1e999px", "1e999%", "1e-999", "1e-999px", "1e-999%",
    ] {
        for source in [
            declaration("line-height", value),
            checked("line-height", value),
        ] {
            assert_eq!(
                source.known().unwrap().property(),
                CssKnownProperty::LineHeight,
                "{value}"
            );
        }
    }
}

#[test]
fn font_slash_uses_the_same_exact_ordinary_line_height_boundaries() {
    for value in [
        "16px/1e999 serif",
        "16px/1e999px serif",
        "16px/1e999% serif",
        "16px/calc(-1px) serif",
    ] {
        assert_eq!(
            declaration("font", value).known().unwrap().property(),
            CssKnownProperty::Font
        );
    }
    for invalid in [
        "16px/-1e-999 serif",
        "16px/-1e-999px serif",
        "16px/-1e-999% serif",
        "16px/1deg serif",
        "16px/1.2",
        "16px/1.2 serif/normal",
    ] {
        rejected_locally("font", invalid);
    }
}

#[test]
fn inherited_normal_has_one_contribution_and_strict_pending_reentry() {
    let property = CssKnownProperty::LineHeight;
    let CssPropertyKindRef::Longhand(longhand) = grammar("line-height").metadata().unwrap().kind()
    else {
        panic!("line-height is a longhand");
    };
    assert!(longhand.inherited_by_default());
    assert_eq!(longhand.property().known_property(), property);
    let initial = longhand.initial_value();
    let CssInitialValueRef::Value(initial_value) = initial.view() else {
        panic!("normal is the fixed initial");
    };
    let normal = declaration("line-height", "normal");
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(&normal).expect("line-height intrinsic expansion")
    else {
        panic!("one line-height contribution");
    };
    let [item] = values.items() else {
        panic!("one line-height contribution")
    };
    assert_eq!(item.property(), property);
    assert_eq!(item.ordinary_value(), Some(initial_value));
    assert!(item.source().same_occurrence(&normal));
    assert_eq!(item.source().importance(), CssImportance::Important);

    for (spelling, keyword) in [
        ("initial", CssGlobalKeyword::Initial),
        ("inherit", CssGlobalKeyword::Inherit),
        ("unset", CssGlobalKeyword::Unset),
        ("revert", CssGlobalKeyword::Revert),
        ("revert-layer", CssGlobalKeyword::RevertLayer),
    ] {
        let source = declaration("line-height", spelling);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("one global contribution")
        };
        let [item] = values.items() else {
            panic!("one global contribution")
        };
        assert_eq!(item.property(), property);
        assert_eq!(item.value(), CssContributionValueRef::Global(keyword));
        assert!(item.source().same_occurrence(&source));
    }

    let source = declaration("line-height", "var(--leading)");
    let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
        panic!("unresolved line-height remains pending");
    };
    assert!(pending.source().same_occurrence(&source));
    assert_eq!(
        pending
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    assert!(matches!(
        pending
            .reenter(parse_component_values("-1e-999").unwrap())
            .unwrap_err()
            .kind(),
        CssExpansionErrorKind::InvalidReplacement(_)
    ));
    for replacement_text in ["normal", "0", "1.2", "120%", "1em", "calc(-1)"] {
        let replacement = parse_component_values(replacement_text).unwrap();
        let CssContributions::Longhands(values) = pending.reenter(replacement.clone()).unwrap()
        else {
            panic!("strict replacement contributes a longhand");
        };
        let [item] = values.items() else {
            panic!("one substituted contribution")
        };
        assert_eq!(item.property(), property);
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.replacement_components(), Some(&replacement));
    }
    let CssExpansion::Contributions(CssContributions::UniversalReset(reset)) =
        expand_declaration(&declaration("all", "initial")).unwrap()
    else {
        panic!("all contributes a universal reset")
    };
    assert!(!reset.excludes(CssPropertyNameRef::Known(property)));
}
