//! Narrow exact-authored fidelity regression; does not settle the legacy math grammar.
use crate::css::support::selected_grammar;
use surgeist_css::*;

const NEGATIVE: &[&str] = &["1e-100deg", "-1e-100deg", "90.000001deg", "89.999999deg"];

#[test]
fn parsed_near_terminal_dimensions_are_rejected_with_neighbor_recovery() {
    for text in NEGATIVE {
        let source = format!("glyph-orientation-vertical:{text};text-orientation:mixed");
        let report = CssParser::default().parse_block_contents(&source);
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
        assert_eq!(value.value(), &CssTextOrientation::Mixed);
        assert!(
            CssParser::default()
                .parse_block_contents(&source)
                .into_validation_result()
                .is_err()
        );
    }
}

#[test]
fn checked_near_terminal_dimensions_are_rejected_for_both_origins() {
    for text in NEGATIVE {
        let parsed = CssComponentValues::parse(&CssParser::default(), text).unwrap();
        let programmatic = CssComponentValues::try_new(
            vec![CssComponentValue::try_token(text).unwrap()],
            Default::default(),
        )
        .unwrap();
        assert!(matches!(
            programmatic.items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
        for components in [parsed, programmatic] {
            assert!(
                CssDeclaration::try_from_components(
                    &CssParser::default(),
                    CssPropertyTarget::Grammar(selected_grammar("glyph-orientation-vertical")),
                    components,
                    CssImportance::Important
                )
                .is_err(),
                "{text}"
            );
        }
    }
}

#[test]
fn exact_terminals_keep_mapping_spelling_and_origins() {
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
        let report = CssParser::default()
            .parse_block_contents(format!("glyph-orientation-vertical:{text}!important"));
        assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
        let parsed = report.syntax()[0].clone();
        let components = CssComponentValues::try_new(
            vec![CssComponentValue::try_token(text).unwrap()],
            Default::default(),
        )
        .unwrap();
        let checked = CssDeclaration::try_from_components(
            &CssParser::default(),
            CssPropertyTarget::Grammar(selected_grammar("glyph-orientation-vertical")),
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
            assert_eq!(
                known.grammar(),
                selected_grammar("glyph-orientation-vertical")
            );
            let CssKnownPropertyValueRef::TextOrientation(value) = known.property_value().unwrap()
            else {
                panic!("text orientation")
            };
            assert_eq!(value.value(), &expected);
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
        let components = CssComponentValues::parse(&CssParser::default(), text).unwrap();
        assert!(
            CssDeclaration::try_from_components(
                &CssParser::default(),
                CssPropertyTarget::Grammar(selected_grammar("glyph-orientation-vertical")),
                components,
                CssImportance::Normal
            )
            .is_err(),
            "{text}"
        );
    }
}
