//! Existing-public-API behavior from CSS 2.1 line-height and Fonts 4 §2.7.
//! Exact typed payload contracts follow with the functional model.

use crate::css::support::checked_grammar_declaration as checked;
use crate::css::support::important_declaration as declaration;
use crate::css::support::known_property;
use crate::css::support::named_property::assert_checked_value_rejected_at;
use crate::css::support::selected_grammar as grammar;
use surgeist_css::*;

fn rejected_locally<'a>(name: &str, value: &str, rest: impl Into<Option<&'a str>>) {
    let rest = rest.into();
    let source = format!("color:red;{name}:{value};color:blue");
    let report = parse_style_attribute(&source);
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid declaration diagnostic: {source}");
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    assert_eq!(
        diagnostic.error().code(),
        CssErrorCode::InvalidPropertyValue
    );
    assert_eq!(report.syntax().len(), 2, "neighbors survive: {source}");
    assert!(validate_style_attribute(&source).is_err());
    assert_checked_value_rejected_at(known_property(name), value, rest);
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
    for (invalid, rest) in [
        ("auto", "auto"),
        ("1deg", "1deg"),
        ("normal 1.2", "1.2"),
        ("1px 2px", "2px"),
    ] {
        rejected_locally("line-height", invalid, rest);
    }
    rejected_locally("line-height", "math", "math");
    rejected_locally("font", "16px/math serif", "math serif");
    let zero = declaration("line-height", "0");
    let CssKnownPropertyValueRef::LineHeight(value) =
        zero.known().unwrap().property_value().unwrap()
    else {
        panic!("unitless zero is line-height");
    };
    assert!(matches!(value.value(), CssLineHeight::Number(_)));
}

#[test]
fn tiny_negative_number_is_rejected_before_float_underflow() {
    for (invalid, rest) in [
        ("-1e-999", "-1e-999"),
        ("-1e-50", "-1e-50"),
        ("-0.01", "-0.01"),
    ] {
        rejected_locally("line-height", invalid, rest);
    }
}

#[test]
fn tiny_negative_length_and_percentage_are_rejected_before_float_underflow() {
    for (invalid, rest) in [
        ("-1e-999px", "-1e-999px"),
        ("-1e-999%", "-1e-999%"),
        ("-1e-50px", "-1e-50px"),
        ("-1e-50%", "-1e-50%"),
    ] {
        rejected_locally("line-height", invalid, rest);
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
    // Fonts 4 §2.7: the family is missing, so the value ends early and its
    // last authored token is responsible.
    rejected_locally("font", "16px/1.2", "1.2");
    for (invalid, rest) in [
        ("16px/-1e-999 serif", "-1e-999 serif"),
        ("16px/-1e-999px serif", "-1e-999px serif"),
        ("16px/-1e-999% serif", "-1e-999% serif"),
        ("16px/1deg serif", "1deg serif"),
        ("16px/1.2 serif/normal", "/normal"),
    ] {
        rejected_locally("font", invalid, rest);
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
    assert!(
        pending
            .reenter(parse_component_values("var(--again)").unwrap())
            .unwrap_err()
            .is_residual_substitution()
    );
    assert!(
        pending
            .reenter(parse_component_values("-1e-999").unwrap())
            .unwrap_err()
            .replacement_error()
            .is_some()
    );
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
