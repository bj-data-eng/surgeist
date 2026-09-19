#![forbid(unsafe_code)]
//! Narrow exact-authored fidelity regression; does not settle the legacy math grammar.
use surgeist_css::*;

const NEGATIVE: &[&str] = &["1e-100deg", "-1e-100deg", "90.000001deg", "89.999999deg"];

fn grammar() -> CssPropertyGrammar {
    CssPropertyGrammar::from_name("glyph-orientation-vertical").unwrap()
}

#[test]
fn parsed_near_terminal_dimensions_are_rejected_with_neighbor_recovery() {
    for text in NEGATIVE {
        let source = format!("glyph-orientation-vertical:{text};text-orientation:mixed");
        let report = parse_style_attribute(&source);
        assert!(
            !report.is_clean(),
            "{text} must not round into a legacy terminal"
        );
        assert_eq!(report.diagnostics().len(), 1);
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDeclaration
        );
        let [remaining] = report.syntax().as_slice() else {
            panic!("one valid neighbor: {text}")
        };
        let CssKnownPropertyValueRef::TextOrientation(value) =
            remaining.known().unwrap().property_value().unwrap()
        else {
            panic!("text orientation")
        };
        assert_eq!(value.orientation(), &CssTextOrientation::Mixed);
        assert!(validate_style_attribute(&source).is_err());
    }
}

#[test]
fn checked_near_terminal_dimensions_are_rejected_for_both_origins() {
    for text in NEGATIVE {
        let parsed = parse_component_values(text).unwrap();
        let programmatic =
            CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()]).unwrap();
        assert!(matches!(
            programmatic.items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
        for components in [parsed, programmatic] {
            assert!(
                parse_property_value_for_grammar(grammar(), components, CssImportance::Important)
                    .is_err(),
                "{text}"
            );
        }
    }
}

#[test]
fn existing_exact_terminals_keep_mapping_spelling_and_origins() {
    for (text, expected) in [
        ("auto", CssTextOrientation::Mixed),
        ("AUTO", CssTextOrientation::Mixed),
        ("0", CssTextOrientation::Upright),
        ("-0", CssTextOrientation::Upright),
        ("+00090", CssTextOrientation::Sideways),
        ("0deg", CssTextOrientation::Upright),
        ("-0.0DEG", CssTextOrientation::Upright),
        ("0e2deg", CssTextOrientation::Upright),
        ("90.000deg", CssTextOrientation::Sideways),
        ("9e1deg", CssTextOrientation::Sideways),
        (r"90d\65g", CssTextOrientation::Sideways),
    ] {
        let report = parse_style_attribute(&format!("glyph-orientation-vertical:{text}!important"));
        assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
        let parsed = report.syntax()[0].clone();
        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()]).unwrap();
        let checked = parse_property_value_for_grammar(
            grammar(),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(checked.value_components(), &components);
        assert!(matches!(
            checked.value_components().items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
        assert!(matches!(
            parsed.value_components().items()[0].origin(),
            CssValueOrigin::Parsed(_)
        ));
        for declaration in [parsed, checked] {
            assert_eq!(declaration.importance(), CssImportance::Important);
            let known = declaration.known().unwrap();
            assert_eq!(known.grammar(), grammar());
            let CssKnownPropertyValueRef::TextOrientation(value) = known.property_value().unwrap()
            else {
                panic!("text orientation")
            };
            assert_eq!(value.orientation(), &expected);
            assert_eq!(value.as_css(), text);
        }
    }
}

#[test]
fn unchanged_deferred_syntax_boundary_is_not_expanded_by_fidelity_fix() {
    // Characterizes the retained boundary only; no claim these settle the CSSWG issue.
    for text in [
        "0.0",
        "90.0",
        "9e1",
        "calc(90deg)",
        "calc(90)",
        "0rad",
        "0turn",
        "90grad",
    ] {
        let components = parse_component_values(text).unwrap();
        assert!(
            parse_property_value_for_grammar(grammar(), components, CssImportance::Normal).is_err(),
            "{text}"
        );
    }
}
