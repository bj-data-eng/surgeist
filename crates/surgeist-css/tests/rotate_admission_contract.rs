#![forbid(unsafe_code)]
//! Existing public boundaries for the authored rotate grammar.
//! Independent authority: selected Transforms2 2021-11-09 §5 (rotate grammar),
//! Values3 2024-03-22 §6.1 (angle units; bare zero is not generally an angle),
//! Values3 §8.1 (typed calc), and Values4 2024-03-12 mathematical typing.
//! No assumptions about a future rotate payload, normalization or output API.
use surgeist_css::*;

const KEYWORD_AXES: &[&str] = &[
    "x 30deg",
    "30deg x",
    "y -0.25turn",
    "-0.25turn y",
    "z 1rad",
    "1rad z",
];
const VECTOR_AXES: &[&str] = &[
    "1 2 3 30deg",
    "30deg 1 2 3",
    "-1 0 0 30deg",
    "30deg -1 0 0",
    "0 0 0 30deg",
    "30deg 0 0 0",
];
const TYPED_MATH: &[&str] = &[
    "calc(15deg + 15deg)",
    "x calc(15deg + 15deg)",
    "calc(15deg + 15deg) x",
    "calc(1 + 1) 0 0 calc(15deg + 15deg)",
    "calc(15deg + 15deg) calc(1 + 1) 0 0",
];
const CONTROLS: &[&str] = &["none", "0deg", "45deg", "-0.25turn", "1rad", "100grad"];
const INVALID: &[&str] = &[
    "x",
    "1 2 3",
    "x y 30deg",
    "30deg x y",
    "x 1 2 3 30deg",
    "1 2 3 x 30deg",
    "1 2 30deg",
    "30deg 1 2",
    "1 2 3 4 30deg",
    "1 30deg 2 3",
    "30deg 60deg",
    "x 30deg 60deg",
    "none 30deg",
    "30deg none",
    "30px",
    "x 30px",
    "1% 2 3 30deg",
    "1px 2 3 30deg",
    "1, 2, 3, 30deg",
    "x calc(1 + 1)",
    "calc(15deg + 1px)",
    "30deg garbage",
];

fn grammar() -> CssPropertyGrammar {
    CssKnownProperty::Rotate.grammar()
}

fn assert_rotate_occurrence(declaration: &CssDeclaration, value: &str) {
    let known = declaration
        .known()
        .expect("retained known rotate declaration");
    assert_eq!(known.property(), CssKnownProperty::Rotate, "{value}");
    assert_eq!(known.grammar(), grammar(), "{value}");
    assert!(
        matches!(
            known.property_value(),
            Some(CssKnownPropertyValueRef::Rotate(_))
        ),
        "{value}"
    );
    assert!(known.global().is_none(), "{value}");
    assert!(known.substitution_dependent().is_none(), "{value}");
    assert_eq!(
        declaration.importance(),
        CssImportance::Important,
        "{value}"
    );
}

fn parsed_accepts(value: &str) {
    let source = format!("color:red;rotate:{value}!important;width:1px");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source:?}: {:?}", report.diagnostics());
    let [before, rotate, after] = report.syntax().as_slice() else {
        panic!(
            "all three occurrences retained: {source:?}: {:?}",
            report.syntax()
        );
    };
    assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
    assert_rotate_occurrence(rotate, value);
    assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
}

fn checked_accepts(value: &str) {
    let components = parse_component_values(value).expect("well-formed component input");
    let by_property = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Rotate),
        components.clone(),
        CssImportance::Important,
    );
    let by_grammar =
        parse_property_value_for_grammar(grammar(), components.clone(), CssImportance::Important);
    let by_property =
        by_property.unwrap_or_else(|error| panic!("property admission of {value:?}: {error:?}"));
    let by_grammar =
        by_grammar.unwrap_or_else(|error| panic!("owned grammar reentry of {value:?}: {error:?}"));
    for declaration in [&by_property, &by_grammar] {
        assert_rotate_occurrence(declaration, value);
        assert!(declaration.position().is_none());
        assert!(declaration.parsed_value().is_none());
        assert_eq!(declaration.value_components(), &components, "{value}");
    }
}

fn parsed_rejects(value: &str) {
    let source = format!("color:red;rotate:{value}!important;width:1px");
    let report = parse_style_attribute(&source);
    assert!(
        !report.is_clean(),
        "invalid rotate must produce a diagnostic: {source}"
    );
    let [before, after] = report.syntax().as_slice() else {
        panic!(
            "invalid rotate dropped and neighboring occurrences retained: {source}: {:?}",
            report.syntax()
        );
    };
    assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
    assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
    assert!(
        report.diagnostics().iter().any(|diagnostic| {
            diagnostic.error().code() == CssErrorCode::InvalidPropertyValue
                && diagnostic.action() == CssRecoveryAction::DropDeclaration
        }),
        "{source}: {:?}",
        report.diagnostics()
    );
}

fn checked_rejects(value: &str) {
    let components = parse_component_values(value).expect("well-formed component input");
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Rotate),
            components.clone(),
            CssImportance::Important,
        )
        .is_err(),
        "property admission must reject all of {value:?}"
    );
    assert!(
        parse_property_value_for_grammar(grammar(), components, CssImportance::Important,).is_err(),
        "owned grammar reentry must reject all of {value:?}"
    );
}

#[test]
fn none_and_ordinary_angle_controls_remain_retained_through_all_front_doors() {
    for value in CONTROLS {
        parsed_accepts(value);
        checked_accepts(value);
    }
}
#[test]
fn parser_retains_keyword_axis_on_either_side_of_angle() {
    for value in KEYWORD_AXES {
        parsed_accepts(value);
    }
}
#[test]
fn public_property_and_owned_grammar_admit_keyword_axis_on_either_side_of_angle() {
    for value in KEYWORD_AXES {
        checked_accepts(value);
    }
}
#[test]
fn parser_retains_three_number_axis_before_or_after_angle() {
    for value in VECTOR_AXES {
        parsed_accepts(value);
    }
}
#[test]
fn public_property_and_owned_grammar_admit_three_number_axis_before_or_after_angle() {
    for value in VECTOR_AXES {
        checked_accepts(value);
    }
}
#[test]
fn parser_retains_angle_math_and_number_axis_math_in_their_typed_grammar_domains() {
    for value in TYPED_MATH {
        parsed_accepts(value);
    }
}
#[test]
fn public_property_and_owned_grammar_admit_angle_math_and_number_axis_math() {
    for value in TYPED_MATH {
        checked_accepts(value);
    }
}
#[test]
fn whole_value_grammar_rejects_partial_axes_wrong_domains_and_trailing_tokens() {
    for value in INVALID {
        parsed_rejects(value);
        checked_rejects(value);
    }
}
#[test]
fn individual_rotate_requires_an_angle_unit_for_ordinary_zero() {
    // Values3 §6.1 defines angles as dimensions and excludes general bare-zero
    // admission. Transforms2 §5 supplies no zero alternative; §12's explicit
    // exception governs transform functions, not this individual property.
    for value in ["0", "-0", "+0", "x 0", "0 x", "1 2 3 0", "0 1 2 3"] {
        parsed_rejects(value);
        checked_rejects(value);
    }
}
