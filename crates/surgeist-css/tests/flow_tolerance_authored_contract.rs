#![forbid(unsafe_code)]
//! Authored flow-tolerance grammar and complete intrinsic declaration lifecycle.
//!
//! Independent source oracles: pinned Grid3 2026-01-21 #placement-tolerance
//! lines 791-799 and 815-837 (normal | signed length-percentage | infinite,
//! initial normal, non-inherited); Values4 2024-03-12 §§10.1-10.9 and §10.13
//! (function arities/types, exceptional arithmetic, specified simplification
//! and percentage-before-dimension/alphabetic unit sorting); Syntax3 recovery
//! versus checked original closure; Cascade5 CSS-wide symbolic values.
//! Exact scalar/declaration resource costs are existing public contracts:
//! flow_tolerance_serialization.rs:135-181 and declaration_serialization.rs:205-210.
//! Whole-sheet selector/rule graph work accounting remains with existing graph
//! owners; the existing CssSheet boundary supplies cumulative sibling bytes.

use surgeist_css::CssKnownProperty as P;
use surgeist_css::*;

const PROP: P = P::FlowTolerance;
const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];

fn parsed(text: &str) -> CssDeclaration {
    let css = format!("/*😀*/flow-tolerance:{text}!important");
    let report = parse_style_attribute(&css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    assert!(validate_style_attribute(&css).is_ok());
    let [source] = report.syntax().as_slice() else {
        panic!("one occurrence")
    };
    assert_eq!(source.known().unwrap().property(), PROP);
    assert_eq!(source.known().unwrap().grammar(), PROP.grammar());
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_some());
    assert_eq!(source.parsed_name().unwrap().source().as_str(), css);
    source.clone()
}
fn checked(text: &str, grammar: bool) -> CssDeclaration {
    let components = parse_component_values(text).unwrap();
    let before = components.clone();
    let source = if grammar {
        parse_property_value_for_grammar(
            PROP.grammar(),
            components.clone(),
            CssImportance::Important,
        )
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(PROP),
            components.clone(),
            CssImportance::Important,
        )
    }
    .unwrap_or_else(|error| panic!("{text}: {error:?}"));
    assert_eq!(components, before);
    assert_eq!(source.value_components(), &components);
    assert_eq!(source.importance(), CssImportance::Important);
    assert!(source.position().is_none());
    assert!(source.parsed_name().is_none());
    assert!(source.parsed_value().is_none());
    assert_eq!(source.known().unwrap().grammar(), PROP.grammar());
    source
}
fn fronts(text: &str) -> [CssDeclaration; 3] {
    [parsed(text), checked(text, false), checked(text, true)]
}
fn value(source: &CssDeclaration) -> &CssFlowTolerance {
    let CssKnownPropertyValueRef::FlowTolerance(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("typed flow tolerance")
    };
    wrapper.value()
}
fn constructed(text: &str) -> CssFlowTolerance {
    match text {
        "normal" => CssFlowTolerance::normal(),
        "infinite" => CssFlowTolerance::infinite(),
        _ => {
            let components = parse_component_values(text).unwrap();
            let scalar = if text.contains('(') {
                CssSpecifiedLengthPercentage::try_from_calculation(
                    CssLengthPercentageCalculation::try_from_components(components).unwrap(),
                )
                .unwrap()
            } else {
                CssSpecifiedLengthPercentage::try_from_component(components.items()[0].clone())
                    .unwrap()
            };
            CssFlowTolerance::length_percentage(scalar)
        }
    }
}
fn completed(source: &CssDeclaration) -> CssLonghandContributions {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one complete terminal")
    };
    assert_eq!(values.items().len(), 1);
    assert_eq!(values.items()[0].property(), PROP);
    assert_source(&values.items()[0], source);
    values
}
fn assert_source(item: &CssLonghandContribution, source: &CssDeclaration) {
    assert!(item.source().same_occurrence(source));
    assert_eq!(item.source().importance(), source.importance());
    assert_eq!(item.source().position(), source.position());
    assert_eq!(item.source().value_components(), source.value_components());
    assert_eq!(item.source().known().unwrap().grammar(), PROP.grammar());
}
fn ordinary(values: &CssLonghandContributions) -> &CssFlowTolerance {
    let CssContributionValueRef::Ordinary(CssLonghandValueRef::FlowTolerance(v)) =
        values.items()[0].value()
    else {
        panic!("ordinary terminal")
    };
    v
}
fn accepted(input: &str, expected: &str) {
    for source in fronts(input) {
        let before = source.clone();
        assert_eq!(
            value(&source).serialize_specified().unwrap(),
            expected,
            "{input}"
        );
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("flow-tolerance: {expected} !important;")
        );
        let values = completed(&source);
        assert_eq!(ordinary(&values), value(&source));
        assert!(values.items()[0].replacement_components().is_none());
        assert_eq!(source, before);
        checked(expected, false);
        checked(expected, true);
    }
    assert_eq!(constructed(input).serialize_specified().unwrap(), expected);
}
fn invalid(input: &str) {
    let css = format!("color:red;flow-tolerance:{input};color:blue");
    let report = parse_style_attribute(&css);
    assert_eq!(report.syntax().len(), 2, "{input}");
    assert!(
        report
            .syntax()
            .iter()
            .all(|v| v.known().unwrap().property() == P::Color)
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one atomic grammar failure: {input}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
        panic!("property-specific failure")
    };
    assert_eq!(detail.property(), PROP);
    assert_eq!(
        validate_style_attribute(&css).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    let components = parse_component_values(input).unwrap();
    let before = components.clone();
    for result in [
        parse_property_value(
            CssPropertyNameRef::Known(PROP),
            components.clone(),
            CssImportance::Normal,
        ),
        parse_property_value_for_grammar(PROP.grammar(), components.clone(), CssImportance::Normal),
    ] {
        assert!(
            matches!(
                result.unwrap_err().kind(),
                CssPropertyValueErrorKind::Grammar(_)
            ),
            "{input}"
        );
    }
    assert_eq!(components, before);
}

#[test]
fn intrinsic_metadata_has_one_noninherited_normal_initial() {
    let feature = feature_metadata("ext.property.flow-tolerance").unwrap();
    assert_eq!(feature.source().id().as_str(), "X-GRID3-20260121");
    assert_eq!(feature.status(), CssSupportStatus::Complete);
    assert_eq!(feature.supported_subset(), None);
    assert_eq!(feature.unsupported_remainder(), None);
    assert_eq!(feature.recognized_unsupported_code(), None);
    assert_eq!(P::from_name("flow-tolerance"), Some(PROP));
    assert_eq!(PROP.canonical_name(), "flow-tolerance");
    for obsolete in ["grid-flow-tolerance", "item-tolerance"] {
        assert!(P::from_name(obsolete).is_none());
    }
    let CssPropertyKindRef::Longhand(metadata) = PROP.metadata().unwrap().kind() else {
        panic!("intrinsic longhand")
    };
    assert_eq!(metadata.property().known_property(), PROP);
    assert!(!metadata.inherited_by_default());
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial.view() else {
        panic!("initial value")
    };
    let CssLonghandValueRef::FlowTolerance(initial) = initial.view() else {
        panic!("typed normal initial")
    };
    assert_eq!(initial, &CssFlowTolerance::normal());
    assert_eq!(CssFlowTolerance::default(), CssFlowTolerance::normal());
}
#[test]
fn signed_literals_keywords_and_canonical_sorting_are_preserved() {
    for (input, expected) in [
        ("normal", "normal"),
        ("infinite", "infinite"),
        ("+02.500EM", "2.5em"),
        ("-25.000%", "-25%"),
        ("-0", "0"),
        ("-0px", "0px"),
        ("-0%", "0%"),
        (r"2\50 X", "2px"),
        ("calc(2px + 3px)", "calc(5px)"),
        ("calc(3px + 2em)", "calc(2em + 3px)"),
        ("calc(-2px - 3%)", "calc(-3% - 2px)"),
        ("calc(2 * 3em)", "calc(6em)"),
        ("max(-2%, -3%)", "max(-2%, -3%)"),
        ("min(1px, 2em)", "min(1px, 2em)"),
        ("max(1px, 2%)", "max(1px, 2%)"),
        ("clamp(1px, 2em, 3px)", "clamp(1px, 2em, 3px)"),
    ] {
        accepted(input, expected);
    }
    for (input, expected) in [("NoRmAl", "normal"), (r"\69 nfinite", "infinite")] {
        for source in fronts(input) {
            assert_eq!(value(&source).serialize_specified().unwrap(), expected);
        }
    }
}
#[test]
fn exact_literal_payloads_and_programmatic_origins_survive_projection() {
    let component = CssComponentValue::try_dimension("-0.5", "em").unwrap();
    let components = CssComponentValues::try_new(vec![component.clone()]).unwrap();
    let source = parse_property_value(
        CssPropertyNameRef::Known(PROP),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    let CssFlowToleranceRef::LengthPercentage(scalar) = value(&source).as_ref() else {
        panic!("signed payload")
    };
    assert_eq!(scalar.literal_component(), Some(&component));
    assert_eq!(component.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(source.value_components(), &components);
    assert!(source.position().is_none());
    let values = completed(&source);
    assert_eq!(ordinary(&values), value(&source));
    for input in ["-1e999px", "-1e-999%"] {
        let expected = if input.ends_with("px") {
            format!("-1{}px", "0".repeat(999))
        } else {
            "0%".into()
        };
        for source in fronts(input) {
            let CssFlowToleranceRef::LengthPercentage(scalar) = value(&source).as_ref() else {
                panic!("exact literal")
            };
            match scalar.literal_component().unwrap().view() {
                CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) => {
                    assert_eq!(number.representation(), "-1e999");
                    assert_eq!(unit, "px");
                }
                CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) => {
                    assert_eq!(number.representation(), "-1e-999")
                }
                _ => panic!("original exact numeric token"),
            }
            assert_eq!(value(&source).serialize_specified().unwrap(), expected);
        }
    }
}
#[test]
fn all_twenty_one_math_functions_enter_through_length_percentage_composition() {
    // Elementary identities and specified-stage calc wrappers are independent
    // oracles. Inverse-angle results are divided by angles before length scaling.
    for (input, expected) in [
        ("calc(1px + 2px)", "calc(3px)"),
        ("min(-2px, 3px)", "calc(-2px)"),
        ("max(-2px, 3px)", "calc(3px)"),
        ("clamp(-2px, 1px, 3px)", "calc(1px)"),
        ("round(2.5px, 1px)", "calc(3px)"),
        ("mod(-18px, 5px)", "calc(2px)"),
        ("rem(-18px, 5px)", "calc(-3px)"),
        ("calc(sin(90deg) * 1px)", "calc(1px)"),
        ("calc(cos(0) * 1px)", "calc(1px)"),
        ("calc(tan(0turn) * 1px)", "calc(0px)"),
        ("calc(asin(1) / 90deg * 1px)", "calc(1px)"),
        ("calc(acos(1) / 1deg * 1px)", "calc(0px)"),
        ("calc(atan(infinity) / 90deg * 1px)", "calc(1px)"),
        ("calc(atan2(1, 0) / 90deg * 1px)", "calc(1px)"),
        ("calc(pow(2, 3) * 1px)", "calc(8px)"),
        ("calc(sqrt(4) * 1px)", "calc(2px)"),
        ("hypot(3px, 4px)", "calc(5px)"),
        ("calc(log(1, 2) * 1px)", "calc(0px)"),
        ("calc(exp(0) * 1px)", "calc(1px)"),
        ("abs(-2px)", "calc(2px)"),
        ("calc(sign(-2px) * 1px)", "calc(-1px)"),
    ] {
        accepted(input, expected);
        for source in fronts(input) {
            let CssFlowToleranceRef::LengthPercentage(scalar) = value(&source).as_ref() else {
                panic!("LP math result")
            };
            let calculation = scalar.calculation().unwrap();
            assert_eq!(calculation.components(), source.value_components());
            assert_eq!(calculation.result_type(), CssCalculationType::Length);
        }
    }
}
#[test]
fn mixed_percentage_math_unbounded_clamp_and_exception_domains_are_authored() {
    for input in [
        "min(1px, 2%)",
        "clamp(none, 2%, 3px)",
        "hypot(3px, 4%)",
        "round(2%, 1%)",
        "mod(2%, 1%)",
        "rem(2%, 1%)",
        "abs(-2%)",
        "calc(10% / 10% * 1px)",
    ] {
        let constructed = constructed(input);
        let before = constructed.clone();
        let output = constructed.serialize_specified().unwrap();
        // Mixed-basis admission and canonical idempotence are controls; output
        // here is not an independently asserted canonical golden. The separate
        // 21-function table and signed sorting table supply explicit goldens.
        for source in fronts(input) {
            assert_eq!(value(&source), &constructed);
            assert_eq!(ordinary(&completed(&source)), value(&source));
            let CssFlowToleranceRef::LengthPercentage(scalar) = value(&source).as_ref() else {
                panic!("mixed LP calculation")
            };
            assert_eq!(
                scalar.calculation().unwrap().components(),
                source.value_components()
            );
        }
        assert_eq!(constructed, before);
        assert_eq!(
            value(&checked(&output, true))
                .serialize_specified()
                .unwrap(),
            output
        );
    }
    accepted("clamp(none, 2px, none)", "calc(2px)");
    // Values4 exceptional arithmetic and dimension serialization (§10.13)
    // retain an explicit multiplication by a unit; contextual censoring is later.
    for (input, expected) in [
        ("calc(sqrt(-1) * 1px)", "calc(NaN * 1px)"),
        ("calc(log(0) * 1px)", "calc(-infinity * 1px)"),
        ("calc(asin(2) / 1deg * 1px)", "calc(NaN * 1px)"),
        ("calc(pow(NaN, 0) * 1px)", "calc(NaN * 1px)"),
    ] {
        accepted(input, expected);
    }
}
#[test]
fn invalid_grammar_math_arity_types_and_sum_separators_drop_only_the_occurrence() {
    for input in [
        "auto",
        "none",
        "-1",
        "1fr",
        "1deg",
        "1s",
        "normal infinite",
        "1px 2px",
        "calc(1)",
        "sin(90deg)",
        "asin(1)",
        "calc()",
        "calc(1px, 2px)",
        "calc(1px + 1s)",
        "min()",
        "max()",
        "clamp(1px, 2px)",
        "clamp(1px, none, 3px)",
        "round()",
        "round(sideways, 1px, 2px)",
        "mod(1px)",
        "rem(1px, 2px, 3px)",
        "calc(sin(1px) * 1px)",
        "calc(cos(1s) * 1px)",
        "calc(tan(1%) * 1px)",
        "calc(asin(1deg) / 1deg * 1px)",
        "calc(acos(1%) / 1deg * 1px)",
        "calc(atan(1px) / 1deg * 1px)",
        "calc(atan2(1px, 1s) / 1deg * 1px)",
        "calc(pow(1px, 2) * 1px)",
        "calc(sqrt(1px) * 1px)",
        "calc(log(1px) * 1px)",
        "calc(exp(1px) * 1px)",
        "hypot(1px, 1s)",
        "abs(1px, 2px)",
        "calc(sign() * 1px)",
        "calc(1px/**/+/**/2px)",
        "calc(1px +/**/2px)",
    ] {
        invalid(input);
    }
    accepted("calc(1px /**/+ /**/2px)", "calc(3px)");
}
#[test]
fn checked_failures_map_to_the_original_responsible_annotation_token() {
    let text = "/*😀*/1px!important";
    let components = parse_component_values(text).unwrap();
    let serialized = components.serialize().unwrap();
    let expected = serialized
        .origin_at(serialized.as_css().find('!').unwrap())
        .unwrap()
        .clone();
    let before = components.clone();
    for result in [
        parse_property_value(
            CssPropertyNameRef::Known(PROP),
            components.clone(),
            CssImportance::Normal,
        ),
        parse_property_value_for_grammar(PROP.grammar(), components.clone(), CssImportance::Normal),
    ] {
        let error = result.unwrap_err();
        assert!(matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
        assert_eq!(error.origin(), &expected);
    }
    assert_eq!(components, before);
}
#[test]
fn all_globals_remain_symbolic_one_terminal_with_original_provenance() {
    for (text, keyword) in GLOBALS {
        for source in fronts(text) {
            assert_eq!(source.known().unwrap().global(), Some(keyword));
            let values = completed(&source);
            assert!(
                matches!(values.items()[0].value(), CssContributionValueRef::Global(v) if v == keyword)
            );
            assert!(values.items()[0].replacement_components().is_none());
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("flow-tolerance: {text} !important;")
            );
        }
    }
}
#[test]
fn every_substitution_front_reenters_all_globals_and_ordinary_values_with_exact_origins() {
    for text in [
        "var(--threshold, -1px)",
        "env(threshold)",
        "attr(data-threshold *)",
    ] {
        for source in fronts(text) {
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("whole pending handle")
            };
            assert!(handle.source().same_occurrence(&source));
            assert_eq!(handle.source().importance(), CssImportance::Important);
            for replacement_text in ["-25%", "normal", "infinite", "calc(-2px - 3%)"] {
                let replacement = parse_component_values(replacement_text).unwrap();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("one ordinary reentry")
                };
                assert_eq!(values.items().len(), 1);
                assert_eq!(values.items()[0].property(), PROP);
                assert_source(&values.items()[0], &source);
                assert_eq!(ordinary(&values), &constructed(replacement_text));
                assert_eq!(
                    values.items()[0].replacement_components(),
                    Some(&replacement)
                );
                let CssFlowToleranceRef::LengthPercentage(scalar) = ordinary(&values).as_ref()
                else {
                    continue;
                };
                if let Some(calculation) = scalar.calculation() {
                    assert_eq!(calculation.components(), &replacement);
                }
                for (actual, supplied) in values.items()[0]
                    .replacement_components()
                    .unwrap()
                    .items()
                    .iter()
                    .zip(replacement.items())
                {
                    assert_eq!(actual.origin(), supplied.origin());
                    let (CssValueOrigin::Parsed(actual), CssValueOrigin::Parsed(supplied)) =
                        (actual.origin(), supplied.origin())
                    else {
                        panic!("replacement parsed source")
                    };
                    assert!(actual.source().same_snapshot(supplied.source()));
                    assert_eq!(actual.source().as_str(), replacement_text);
                }
            }
            for (replacement_text, keyword) in GLOBALS {
                let replacement = parse_component_values(replacement_text).unwrap();
                let CssContributions::Longhands(values) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("global reentry")
                };
                assert_eq!(values.items().len(), 1);
                assert_eq!(values.items()[0].property(), PROP);
                assert_source(&values.items()[0], &source);
                assert!(
                    matches!(values.items()[0].value(), CssContributionValueRef::Global(v) if v == keyword)
                );
                assert_eq!(
                    values.items()[0].replacement_components(),
                    Some(&replacement)
                );
            }
        }
    }
}
#[test]
fn residual_and_invalid_replacements_are_atomic_reusable_and_keep_pending_source() {
    for text in [
        "var(--threshold)",
        "env(threshold)",
        "attr(data-threshold *)",
    ] {
        let source = checked(text, true);
        let before = source.clone();
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for replacement in [
            "var(--again)",
            "env(again)",
            "attr(data-again *)",
            "calc(var(--again) - 2px)",
            r"\76 ar(--again)",
        ] {
            for _ in 0..2 {
                assert!(matches!(
                    handle
                        .reenter(parse_component_values(replacement).unwrap())
                        .unwrap_err()
                        .kind(),
                    CssExpansionErrorKind::ResidualSubstitution
                ));
            }
        }
        for replacement in [
            "auto",
            "normal infinite",
            "-1",
            "1px!important",
            "1px;color:red",
            "calc(1px + 1s)",
        ] {
            for _ in 0..2 {
                let error = handle
                    .reenter(parse_component_values(replacement).unwrap())
                    .unwrap_err();
                let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                    panic!("strict grammar reentry")
                };
                assert!(matches!(
                    error.kind(),
                    CssPropertyValueErrorKind::Grammar(_)
                ));
            }
        }
        assert!(handle.source().same_occurrence(&source));
        assert_eq!(source, before);
        let CssContributions::Longhands(values) = handle
            .reenter(parse_component_values("-2px").unwrap())
            .unwrap()
        else {
            panic!("successful retry")
        };
        assert_source(&values.items()[0], &source);
        assert_eq!(ordinary(&values), &constructed("-2px"));
        assert!(source.known().unwrap().substitution_dependent().is_some());
    }
}

fn implicit_origin(components: &CssComponentValues) -> CssValueOrigin {
    for value in components.items() {
        let closing = match value.view() {
            CssComponentValueRef::Function(v) => Some(v.closing_origin()),
            CssComponentValueRef::Block(v) => Some(v.closing_origin()),
            _ => None,
        };
        if let Some(origin @ CssValueOrigin::ImplicitClosure { .. }) = closing {
            return origin.clone();
        }
    }
    let serialized = components.serialize().unwrap();
    (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(origin @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(origin.clone())
            }
            _ => None,
        })
        .expect("retained implicit closure mapping")
}
fn assert_closure(error: &CssPropertyValueParseError, origin: &CssValueOrigin) {
    assert!(matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::End(Some(origin.clone()))
    );
}
fn assert_original_eof(origin: &CssValueOrigin, text: &str) {
    let CssValueOrigin::ImplicitClosure { opening, at } = origin else {
        panic!("implicit EOF")
    };
    assert_eq!(opening.source().as_str(), text);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(at.span().start().byte_offset().value(), text.len());
    assert_eq!(at.span().end().byte_offset().value(), text.len());
}
#[test]
fn checked_original_comments_and_functions_cannot_be_repaired_before_grammar() {
    for text in [
        "-2px/*",
        "normal/*",
        "infinite/*",
        "initial/*",
        "var(--threshold)/*",
        "env(threshold)/*",
        "attr(data-threshold *)/*",
        "calc(-2px - 3%",
        "var(--threshold",
        "env(threshold",
        "attr(data-threshold *",
    ] {
        let components = parse_component_values(text).unwrap();
        let before = components.clone();
        let origin = implicit_origin(&components);
        assert_original_eof(&origin, text);
        for result in [
            parse_property_value(
                CssPropertyNameRef::Known(PROP),
                components.clone(),
                CssImportance::Important,
            ),
            parse_property_value_for_grammar(
                PROP.grammar(),
                components.clone(),
                CssImportance::Important,
            ),
        ] {
            assert_closure(&result.unwrap_err(), &origin);
        }
        assert_eq!(components, before);
    }
    for text in [
        "-2px/**/",
        "normal/**/",
        "infinite/**/",
        "initial/**/",
        "var(--threshold)/**/",
        "env(threshold)/**/",
        "attr(data-threshold *)/**/",
        "calc(-2px - 3%)",
    ] {
        checked(text, false);
        checked(text, true);
    }
    for (keyword, _) in GLOBALS {
        let text = format!("{keyword}/*");
        let components = parse_component_values(&text).unwrap();
        let origin = implicit_origin(&components);
        assert_original_eof(&origin, &text);
        for result in [
            parse_property_value(
                CssPropertyNameRef::Known(PROP),
                components.clone(),
                CssImportance::Normal,
            ),
            parse_property_value_for_grammar(
                PROP.grammar(),
                components.clone(),
                CssImportance::Normal,
            ),
        ] {
            assert_closure(&result.unwrap_err(), &origin);
        }
        checked(&format!("{keyword}/**/"), false);
        checked(&format!("{keyword}/**/"), true);
    }
}
#[test]
fn strict_reentry_rejects_original_closure_and_can_retry_matched_replacements() {
    for pending in [
        "var(--threshold)",
        "env(threshold)",
        "attr(data-threshold *)",
    ] {
        let source = checked(pending, true);
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending")
        };
        for text in [
            "-2px/*",
            "normal/*",
            "infinite/*",
            "initial/*",
            "calc(-2px - 3%",
        ] {
            let components = parse_component_values(text).unwrap();
            let origin = implicit_origin(&components);
            assert_original_eof(&origin, text);
            for _ in 0..2 {
                let error = handle.reenter(components.clone()).unwrap_err();
                let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                    panic!("original closure rejection")
                };
                assert_closure(error, &origin);
            }
            assert!(handle.source().same_occurrence(&source));
        }
        // Residual detection has priority over reentry grammar even at repaired EOF.
        for text in ["var(--again", "env(again", "attr(data-again *"] {
            assert!(matches!(
                handle
                    .reenter(parse_component_values(text).unwrap())
                    .unwrap_err()
                    .kind(),
                CssExpansionErrorKind::ResidualSubstitution
            ));
        }
        for text in ["-2px/**/", "initial/**/", "calc(-2px - 3%)"] {
            assert!(
                handle
                    .reenter(parse_component_values(text).unwrap())
                    .is_ok()
            );
        }
    }
}
#[test]
fn browser_recovery_retains_occurrences_and_diagnostics_through_normalization() {
    for text in [
        "-2px/*",
        "initial/*",
        "var(--threshold)/*",
        "calc(-2px - 3%",
        "var(--threshold",
    ] {
        let css = format!(".a{{flow-tolerance:{text}");
        let report = parse_sheet(&css);
        assert!(!report.is_clean());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        );
        assert!(validate_sheet(&css).is_err());
        let before = report.clone();
        let exact = CssNormalizationLimits::try_new(0, 1, 1, 1).unwrap();
        let normalized = normalize_report_with_limits(&report, exact).unwrap();
        assert_eq!(normalized.diagnostics(), report.diagnostics());
        let declarations: Vec<_> = normalized
            .syntax()
            .items()
            .iter()
            .filter_map(|item| match item {
                CssNormalizedItem::Declaration(v) => Some(v),
                _ => None,
            })
            .collect();
        let [item] = declarations.as_slice() else {
            panic!("retained recovered source")
        };
        assert_eq!(item.order(), 0);
        assert_eq!(item.source().parsed_name().unwrap().source().as_str(), css);
        match item.expansion() {
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_eq!(values.items().len(), 1);
                assert_source(&values.items()[0], item.source());
            }
            CssExpansion::Pending(handle) => {
                assert!(handle.source().same_occurrence(item.source()))
            }
            _ => panic!("flow recovery expansion"),
        }
        assert!(
            normalize_report_with_limits(
                &report,
                CssNormalizationLimits::try_new(0, 1, 1, 0).unwrap()
            )
            .is_err()
        );
        assert_eq!(report, before);
        assert!(normalize_report_with_limits(&report, exact).is_ok());
    }
}
#[test]
fn normalization_counts_every_ordinary_global_and_whole_pending_occurrence_once() {
    let css = "@media screen{.a{flow-tolerance:normal!important;flow-tolerance:-25%;flow-tolerance:infinite;flow-tolerance:var(--threshold);flow-tolerance:unset;flow-tolerance:calc(-2px - 3%)}}";
    let report = parse_sheet(css);
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 2, 6, 6).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 6);
    for (index, item) in declarations.iter().enumerate() {
        assert_eq!(item.order(), index);
        assert_eq!(item.source().known().unwrap().property(), PROP);
        assert_eq!(
            item.source().importance(),
            if index == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        assert!(
            item.selector_context()
                .same_context(declarations[0].selector_context())
        );
        assert!(
            item.rule_context()
                .same_context(declarations[0].rule_context())
        );
        if index > 0 {
            assert!(
                !item
                    .source()
                    .same_occurrence(declarations[index - 1].source())
            );
        }
        match item.expansion() {
            CssExpansion::Contributions(CssContributions::Longhands(values)) => {
                assert_eq!(values.items().len(), 1);
                assert_eq!(values.items()[0].property(), PROP);
                assert_source(&values.items()[0], item.source());
                if index == 4 {
                    assert!(matches!(
                        values.items()[0].value(),
                        CssContributionValueRef::Global(CssGlobalKeyword::Unset)
                    ));
                } else {
                    assert_eq!(
                        ordinary(values),
                        &constructed(
                            ["normal", "-25%", "infinite", "", "", "calc(-2px - 3%)"][index]
                        )
                    );
                }
            }
            CssExpansion::Pending(handle) => {
                assert_eq!(index, 3);
                assert!(handle.source().same_occurrence(item.source()));
            }
            _ => panic!("one terminal or whole pending"),
        }
    }
    for (limits, resource) in [
        (
            CssNormalizationLimits::try_new(1, 2, 6, 5).unwrap(),
            CssNormalizationResource::Contributions,
        ),
        (
            CssNormalizationLimits::try_new(1, 2, 5, 6).unwrap(),
            CssNormalizationResource::Declarations,
        ),
    ] {
        for _ in 0..2 {
            let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
            assert_eq!(
                error.kind(),
                &CssNormalizationErrorKind::LimitExceeded { resource, limit: 5 }
            );
            assert_eq!(error.declaration_order(), Some(5));
            assert!(
                error
                    .declaration()
                    .unwrap()
                    .same_occurrence(declarations[5].source())
            );
        }
    }
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
}
#[test]
fn primitive_and_declaration_resources_have_exact_independent_costs_and_atomic_retry() {
    use CssSpecifiedValueSerializationErrorKind as Kind;
    use CssSpecifiedValueSerializationLimits as Limits;
    for (input, expected, inputs, projections) in [
        ("normal", "normal", 1, 1),
        ("infinite", "infinite", 1, 1),
        ("-2px", "-2px", 1, 1),
        ("calc(1px + 2em)", "calc(2em + 1px)", 4, 5),
    ] {
        let source = checked(input, true);
        let before = source.clone();
        let declaration = format!("flow-tolerance: {expected} !important;");
        assert_eq!(
            value(&source)
                .serialize_specified_with_limits(Limits::new(inputs, projections, expected.len()))
                .unwrap(),
            expected
        );
        assert_eq!(
            source
                .to_specified_css_with_limits(Limits::new(
                    inputs + 2,
                    projections + 2,
                    declaration.len()
                ))
                .unwrap(),
            declaration
        );
        for (primitive, aggregate, kind) in [
            (
                Limits::new(inputs - 1, projections, expected.len()),
                Limits::new(inputs + 1, projections + 2, declaration.len()),
                Kind::InputNodeLimit,
            ),
            (
                Limits::new(inputs, projections - 1, expected.len()),
                Limits::new(inputs + 2, projections + 1, declaration.len()),
                Kind::ProjectionNodeLimit,
            ),
            (
                Limits::new(inputs, projections, expected.len() - 1),
                Limits::new(inputs + 2, projections + 2, declaration.len() - 1),
                Kind::ByteLimit,
            ),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    value(&source)
                        .serialize_specified_with_limits(primitive)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(
                    source
                        .to_specified_css_with_limits(aggregate)
                        .unwrap_err()
                        .kind(),
                    kind
                );
                assert_eq!(source, before);
            }
        }
        assert_eq!(source.to_specified_css().unwrap(), declaration);
    }
}
#[test]
fn sheet_siblings_share_canonical_byte_budget_with_atomic_reusable_failure() {
    use CssSpecifiedValueSerializationLimits as Limits;
    let report =
        parse_sheet(".a{flow-tolerance:normal!important}.b{flow-tolerance:calc(1px + 2em)}");
    assert!(report.is_clean());
    let sheet = report.syntax();
    let before = sheet.clone();
    let expected =
        ".a { flow-tolerance: normal !important; }\n.b { flow-tolerance: calc(2em + 1px); }";
    let exact = Limits::new(usize::MAX, usize::MAX, expected.len());
    let short = Limits::new(usize::MAX, usize::MAX, expected.len() - 1);
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
    assert!(
        sheet
            .rules()
            .iter()
            .all(|r| r.to_specified_css_with_limits(short).is_ok())
    );
    for _ in 0..2 {
        let error = sheet.to_specified_css_with_limits(short).unwrap_err();
        assert_eq!(
            error.kind(),
            CssSpecifiedRuleSerializationErrorKind::Resource(
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            )
        );
        assert_eq!(error.rule_index(), Some(1));
        assert_eq!(sheet, &before);
    }
    assert_eq!(sheet.to_specified_css_with_limits(exact).unwrap(), expected);
}
#[test]
fn supported_color_control_keeps_checked_expansion_and_provider_contract() {
    let components = parse_component_values("currentcolor").unwrap();
    for source in [
        parse_property_value(
            CssPropertyNameRef::Known(P::Color),
            components.clone(),
            CssImportance::Important,
        )
        .unwrap(),
        parse_property_value_for_grammar(P::Color.grammar(), components, CssImportance::Important)
            .unwrap(),
    ] {
        assert_eq!(
            source.to_specified_css().unwrap(),
            "color: currentcolor !important;"
        );
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("color control")
        };
        assert_eq!(values.items().len(), 1);
        assert_eq!(values.items()[0].property(), P::Color);
        assert!(values.items()[0].source().same_occurrence(&source));
        assert!(matches!(
            values.items()[0].value(),
            CssContributionValueRef::Ordinary(CssLonghandValueRef::Color(_))
        ));
    }
}
