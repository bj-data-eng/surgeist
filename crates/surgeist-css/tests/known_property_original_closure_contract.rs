#![forbid(unsafe_code)]
//! Checked known properties retain strict original-component closure.
//! Canonical identities and samples come from independently authored records.

mod common;
#[path = "common/property_expectations.rs"]
mod property_expectations;

use property_expectations::{MetadataExpectation, PropertyExpectation};
use std::collections::HashSet;
use surgeist_css::*;

const GLOBALS: [(&str, CssGlobalKeyword); 5] = [
    ("initial", CssGlobalKeyword::Initial),
    ("inherit", CssGlobalKeyword::Inherit),
    ("unset", CssGlobalKeyword::Unset),
    ("revert", CssGlobalKeyword::Revert),
    ("revert-layer", CssGlobalKeyword::RevertLayer),
];
const IMPORTANCE: [CssImportance; 2] = [CssImportance::Normal, CssImportance::Important];
const PENDING: [&str; 3] = ["var(--known)", "env(known)", "attr(data-known)"];
const OPEN_PENDING: [&str; 3] = ["var(--known", "env(known", "attr(data-known"];

fn fixtures() -> &'static [PropertyExpectation] {
    let cases = property_expectations::CASES;
    assert!(
        !cases.is_empty(),
        "independent record table must be exercised"
    );
    let mut identities = HashSet::new();
    for case in cases {
        assert!(
            identities.insert(case.property),
            "duplicate fixture {}",
            case.name
        );
        assert_eq!(CssKnownProperty::from_name(case.name), Some(case.property));
    }
    cases
}

fn finish(failures: Vec<String>) {
    assert!(
        failures.is_empty(),
        "{} individual failures:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// Obtain a PUBLIC original witness, then check its coordinates independently.
// We never derive the expectation from the property error or a private helper.
fn comment_witness(values: &CssComponentValues, source: &str) -> CssValueOrigin {
    let serialized = values.serialize().expect("fixture component serialization");
    let origin = (0..serialized.as_css().len())
        .find_map(|offset| match serialized.origin_at(offset) {
            Some(CssSerializedOrigin::Token(value @ CssValueOrigin::ImplicitClosure { .. })) => {
                Some(value.clone())
            }
            _ => None,
        })
        .expect("fixture has original comment closure");
    let start = source
        .rfind("/*unfinished")
        .expect("independent comment opening");
    assert_original(&origin, source, start, source.len());
    origin
}

fn function_witness(values: &CssComponentValues, source: &str) -> CssValueOrigin {
    let function = values
        .items()
        .iter()
        .find_map(|item| match item.view() {
            CssComponentValueRef::Function(value) => Some(value),
            _ => None,
        })
        .expect("fixture root function");
    let origin = function.closing_origin().clone();
    let start = source
        .find("var(")
        .or_else(|| source.find("env("))
        .or_else(|| source.find("attr("))
        .or_else(|| source.find("future("))
        .expect("independent function opening");
    let end = start + source[start..].find('(').unwrap() + 1;
    assert_original(&origin, source, start, end);
    origin
}

fn assert_original(origin: &CssValueOrigin, source: &str, start: usize, end: usize) {
    let CssValueOrigin::ImplicitClosure { opening, at } = origin else {
        panic!("actual original implicit origin: {origin:?}")
    };
    assert_eq!(opening.source().as_str(), source);
    assert_eq!(at.source().as_str(), source);
    assert!(opening.source().same_snapshot(at.source()));
    assert_eq!(opening.span().start().byte_offset().value(), start);
    assert_eq!(opening.span().end().byte_offset().value(), end);
    assert_eq!(opening.span().start().line().value(), 0);
    assert_eq!(opening.span().end().line().value(), 0);
    assert_eq!(
        opening.span().start().column().value() as usize,
        source[..start].encode_utf16().count()
    );
    assert_eq!(
        opening.span().end().column().value() as usize,
        source[..end].encode_utf16().count()
    );
    assert_eq!(at.span().start(), at.span().end());
    assert_eq!(at.span().start().byte_offset().value(), source.len());
    assert_eq!(at.span().start().line().value(), 0);
    assert_eq!(
        at.span().start().column().value() as usize,
        source.encode_utf16().count()
    );
}

fn expected_error(error: &CssPropertyValueParseError, witness: &CssValueOrigin) -> bool {
    matches!(
        error.kind(),
        CssPropertyValueErrorKind::Grammar(ErrorKind::UnexpectedEnd(_))
    ) && error.origin() == &CssSerializedOrigin::End(Some(witness.clone()))
}

fn reject_fronts(
    property: CssKnownProperty,
    values: &CssComponentValues,
    witness: &CssValueOrigin,
    label: &str,
    failures: &mut Vec<String>,
) {
    let before = values.clone();
    for importance in IMPORTANCE {
        for repeat in 0..2 {
            for (front, result) in [
                (
                    "known name",
                    parse_property_value(
                        CssPropertyNameRef::Known(property),
                        values.clone(),
                        importance,
                    ),
                ),
                (
                    "canonical grammar",
                    parse_property_value_for_grammar(
                        property.grammar(),
                        values.clone(),
                        importance,
                    ),
                ),
            ] {
                if !matches!(result, Err(ref error) if expected_error(error, witness)) {
                    failures.push(format!("{property:?}/{label}/{front}/{importance:?}/retry{repeat}: expected typed original EOF {witness:?}; actual {result:?}"));
                }
                assert_eq!(values, &before, "immutable input {label}");
            }
        }
    }
}

fn assert_constructed(
    declaration: &CssDeclaration,
    property: CssKnownProperty,
    grammar: CssPropertyGrammar,
    values: &CssComponentValues,
    importance: CssImportance,
) {
    let known = declaration.known().expect("known declaration");
    assert_eq!(known.property(), property);
    assert_eq!(known.grammar(), grammar);
    assert_eq!(declaration.value_components(), values);
    assert_eq!(declaration.importance(), importance);
    assert_eq!(declaration.position(), None);
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    for (actual, original) in declaration
        .value_components()
        .items()
        .iter()
        .zip(values.items())
    {
        assert_eq!(actual.origin(), original.origin());
        if let (CssValueOrigin::Parsed(a), CssValueOrigin::Parsed(b)) =
            (actual.origin(), original.origin())
        {
            assert!(a.source().same_snapshot(b.source()));
        }
    }
}

fn assert_global_retry(
    completed: CssContributions,
    source: &CssDeclaration,
    replacement: &CssComponentValues,
    keyword: CssGlobalKeyword,
) {
    match completed {
        CssContributions::Longhands(values) => {
            assert!(!values.items().is_empty());
            for item in values.items() {
                assert!(item.source().same_occurrence(source));
                assert_eq!(item.source().importance(), source.importance());
                assert_eq!(
                    item.source().known().unwrap().grammar(),
                    source.known().unwrap().grammar()
                );
                assert_eq!(item.replacement_components(), Some(replacement));
                assert!(
                    matches!(item.value(), CssContributionValueRef::Global(actual) if actual == keyword)
                );
            }
        }
        CssContributions::UniversalReset(reset) => {
            assert_eq!(source.known().unwrap().property(), CssKnownProperty::All);
            assert_eq!(reset.keyword(), keyword);
            assert!(reset.source().same_occurrence(source));
            assert_eq!(reset.source().importance(), source.importance());
            assert_eq!(reset.replacement_components(), Some(replacement));
        }
        other => panic!("known completed global contribution: {other:?}"),
    }
}

#[test]
fn every_independent_record_rejects_original_global_comment_closure_at_both_fronts() {
    let mut failures = Vec::new();
    for case in fixtures() {
        for (keyword, _) in GLOBALS {
            let source = format!("/*😀*/{keyword}/*unfinished");
            let values = parse_component_values(&source).unwrap();
            let witness = comment_witness(&values, &source);
            reject_fronts(case.property, &values, &witness, &source, &mut failures);
        }
    }
    finish(failures);
}

#[test]
fn every_independent_record_rejects_pending_original_function_and_comment_closure() {
    let mut failures = Vec::new();
    for case in fixtures() {
        for pending in OPEN_PENDING {
            let source = format!("/*😀*/{pending}");
            let values = parse_component_values(&source).unwrap();
            let witness = function_witness(&values, &source);
            reject_fronts(case.property, &values, &witness, &source, &mut failures);
        }
        for pending in PENDING {
            let source = format!("/*😀*/{pending}/*unfinished");
            let values = parse_component_values(&source).unwrap();
            let witness = comment_witness(&values, &source);
            reject_fronts(case.property, &values, &witness, &source, &mut failures);
        }
    }
    finish(failures);
}

#[test]
fn every_record_matched_closed_globals_and_programmatic_globals_preserve_identity() {
    for case in fixtures() {
        for (keyword, expected) in GLOBALS {
            let parsed = parse_component_values(&format!("/*😀*/{keyword}/**/")).unwrap();
            let programmatic =
                CssComponentValues::try_new(vec![CssComponentValue::try_ident(keyword).unwrap()])
                    .unwrap();
            assert!(
                programmatic
                    .items()
                    .iter()
                    .all(|item| matches!(item.origin(), CssValueOrigin::Programmatic))
            );
            for values in [parsed, programmatic] {
                for importance in IMPORTANCE {
                    for declaration in [
                        parse_property_value(
                            CssPropertyNameRef::Known(case.property),
                            values.clone(),
                            importance,
                        )
                        .unwrap(),
                        parse_property_value_for_grammar(
                            case.property.grammar(),
                            values.clone(),
                            importance,
                        )
                        .unwrap(),
                    ] {
                        assert_constructed(
                            &declaration,
                            case.property,
                            case.property.grammar(),
                            &values,
                            importance,
                        );
                        assert_eq!(declaration.known().unwrap().global(), Some(expected));
                    }
                }
            }
        }
    }
}

#[test]
fn every_record_matched_closed_pending_and_programmatic_pending_preserve_original_components() {
    for case in fixtures() {
        let programmatic = CssComponentValues::try_new(vec![
            CssComponentValue::try_function(
                "var",
                CssComponentValues::try_new(vec![CssComponentValue::try_ident("--known").unwrap()])
                    .unwrap(),
            )
            .unwrap(),
        ])
        .unwrap();
        for values in PENDING
            .into_iter()
            .map(|input| parse_component_values(&format!("/*😀*/{input}/**/")).unwrap())
            .chain(std::iter::once(programmatic))
        {
            for importance in IMPORTANCE {
                for declaration in [
                    parse_property_value(
                        CssPropertyNameRef::Known(case.property),
                        values.clone(),
                        importance,
                    )
                    .unwrap(),
                    parse_property_value_for_grammar(
                        case.property.grammar(),
                        values.clone(),
                        importance,
                    )
                    .unwrap(),
                ] {
                    assert_constructed(
                        &declaration,
                        case.property,
                        case.property.grammar(),
                        &values,
                        importance,
                    );
                    assert!(
                        declaration
                            .known()
                            .unwrap()
                            .substitution_dependent()
                            .is_some()
                    );
                }
            }
        }
    }
}

#[test]
fn available_record_reentry_rejects_original_closure_is_residual_first_and_retries_atomically() {
    let mut failures = Vec::new();
    for case in fixtures() {
        // Availability is an INDEPENDENT fixture fact; unsupported setup is not RED.
        if matches!(case.metadata, MetadataExpectation::Unavailable) {
            continue;
        }
        for importance in IMPORTANCE {
            let source = parse_property_value(
                CssPropertyNameRef::Known(case.property),
                parse_component_values("var(--known)").unwrap(),
                importance,
            )
            .unwrap();
            let before_source = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!(
                    "{} independent available fixture needs pending handle",
                    case.name
                )
            };
            for (keyword, _) in GLOBALS {
                let input = format!("/*😀*/{keyword}/*unfinished");
                let replacement = parse_component_values(&input).unwrap();
                let before = replacement.clone();
                let witness = comment_witness(&replacement, &input);
                for repeat in 0..2 {
                    let actual = handle.reenter(replacement.clone());
                    if !matches!(actual, Err(ref error) if matches!(error.kind(), CssExpansionErrorKind::InvalidReplacement(detail) if expected_error(detail, &witness)))
                    {
                        failures.push(format!("{}/{input}/reentry/{importance:?}/retry{repeat}: expected exact original EOF {witness:?}; actual {actual:?}", case.name));
                    }
                    assert_eq!(replacement, before);
                    assert_eq!(source, before_source);
                    assert!(handle.source().same_occurrence(&source));
                }
            }
            let replacement = parse_component_values("/*😀*/future(value").unwrap();
            let witness = function_witness(&replacement, "/*😀*/future(value");
            let actual = handle.reenter(replacement.clone());
            if !matches!(actual, Err(ref error) if matches!(error.kind(), CssExpansionErrorKind::InvalidReplacement(detail) if expected_error(detail, &witness)))
            {
                failures.push(format!("{}/future/reentry/{importance:?}: expected exact original EOF; actual {actual:?}", case.name));
            }
            for residual in [
                "var(--again",
                "env(again",
                "attr(data-again",
                "[future(var(--again",
            ] {
                let replacement = parse_component_values(residual).unwrap();
                let before = replacement.clone();
                assert_eq!(
                    handle.reenter(replacement.clone()).unwrap_err().kind(),
                    &CssExpansionErrorKind::ResidualSubstitution,
                    "{}/{residual}",
                    case.name
                );
                assert_eq!(replacement, before);
                assert_eq!(source, before_source);
            }
            for (keyword, expected) in GLOBALS {
                let replacement = parse_component_values(&format!("/*😀*/{keyword}/**/")).unwrap();
                for _ in 0..2 {
                    assert_global_retry(
                        handle.reenter(replacement.clone()).unwrap(),
                        &source,
                        &replacement,
                        expected,
                    );
                    assert_eq!(source, before_source);
                    assert!(handle.source().same_occurrence(&source));
                }
            }
        }
    }
    finish(failures);
}

#[test]
fn every_record_browser_recovery_keeps_unclean_known_global_and_valid_preceding_sibling() {
    for case in fixtures() {
        for (keyword, expected) in GLOBALS {
            let input = format!("color:red;{}:{keyword}/*unfinished", case.name);
            let report = parse_style_attribute(&input);
            assert!(!report.is_clean(), "{input}");
            assert_eq!(
                validate_style_attribute(&input).unwrap_err().diagnostics(),
                report.diagnostics()
            );
            let [sibling, recovered] = report.syntax().as_slice() else {
                panic!("retain valid sibling and recovered known occurrence: {input}: {report:?}")
            };
            assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
            assert_eq!(recovered.known().unwrap().property(), case.property);
            assert_eq!(recovered.known().unwrap().global(), Some(expected));
            let witness = comment_witness(recovered.value_components(), &input);
            assert!(matches!(witness, CssValueOrigin::ImplicitClosure { .. }));
            let clean = format!("color:red;{}:{keyword}/**/", case.name);
            assert!(
                parse_style_attribute(&clean).is_clean(),
                "matched complete browser control {clean}"
            );
        }
    }
}

#[test]
fn existing_record_ordinary_samples_remain_callable_and_typed_without_new_property_inventory() {
    for case in fixtures() {
        if case.wrapper.is_none() {
            // All's explicit dispatch `block` is negative; no guessed ordinary sample.
            continue;
        }
        let Some(authored) = case.ordinary_stimulus() else {
            continue;
        };
        let values = parse_component_values(authored).unwrap();
        for importance in IMPORTANCE {
            for declaration in [
                parse_property_value(
                    CssPropertyNameRef::Known(case.property),
                    values.clone(),
                    importance,
                )
                .unwrap(),
                parse_property_value_for_grammar(
                    case.property.grammar(),
                    values.clone(),
                    importance,
                )
                .unwrap(),
            ] {
                assert_constructed(
                    &declaration,
                    case.property,
                    case.property.grammar(),
                    &values,
                    importance,
                );
                let known = declaration.known().unwrap();
                case.assert_wrapper(known, known.property_value().unwrap(), authored);
            }
        }
    }
}

#[test]
fn independently_recorded_name_equivalent_aliases_share_strict_grammar_without_changing_target() {
    let mut failures = Vec::new();
    for case in fixtures() {
        for alias in case.aliases {
            let grammar = CssPropertyGrammar::from_name(alias).unwrap();
            assert_eq!(
                grammar,
                case.property.grammar(),
                "independent equivalent alias {alias}"
            );
            let values = parse_component_values("inherit/*unfinished").unwrap();
            let witness = comment_witness(&values, "inherit/*unfinished");
            let actual =
                parse_property_value_for_grammar(grammar, values.clone(), CssImportance::Important);
            if !matches!(actual, Err(ref error) if expected_error(error, &witness)) {
                failures.push(format!(
                    "{}/{alias}: expected original EOF; actual {actual:?}",
                    case.name
                ));
            }
            let closed = parse_component_values("inherit/**/").unwrap();
            let declaration =
                parse_property_value_for_grammar(grammar, closed.clone(), CssImportance::Important)
                    .unwrap();
            assert_constructed(
                &declaration,
                case.property,
                grammar,
                &closed,
                CssImportance::Important,
            );
            assert_eq!(
                declaration.known().unwrap().global(),
                Some(CssGlobalKeyword::Inherit)
            );
            let pending_values = parse_component_values("var(--known").unwrap();
            let pending_witness = function_witness(&pending_values, "var(--known");
            let actual =
                parse_property_value_for_grammar(grammar, pending_values, CssImportance::Important);
            if !matches!(actual, Err(ref error) if expected_error(error, &pending_witness)) {
                failures.push(format!(
                    "{}/{alias}/pending: expected original EOF; actual {actual:?}",
                    case.name
                ));
            }
            if !matches!(case.metadata, MetadataExpectation::Unavailable) {
                let source = parse_property_value_for_grammar(
                    grammar,
                    parse_component_values("var(--known)").unwrap(),
                    CssImportance::Important,
                )
                .unwrap();
                let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                    panic!("available alias pending")
                };
                let actual = handle.reenter(values.clone());
                if !matches!(actual, Err(ref error) if matches!(error.kind(), CssExpansionErrorKind::InvalidReplacement(detail) if expected_error(detail, &witness)))
                {
                    failures.push(format!(
                        "{}/{alias}/reentry: expected original EOF; actual {actual:?}",
                        case.name
                    ));
                }
                assert_eq!(
                    handle
                        .reenter(parse_component_values("var(--again").unwrap())
                        .unwrap_err()
                        .kind(),
                    &CssExpansionErrorKind::ResidualSubstitution
                );
                assert_global_retry(
                    handle.reenter(closed.clone()).unwrap(),
                    &source,
                    &closed,
                    CssGlobalKeyword::Inherit,
                );
            }
        }
    }
    finish(failures);
}

fn legacy(
    grammar_name: &str,
    expected_property: CssKnownProperty,
    ordinary: &str,
    failures: &mut Vec<String>,
) {
    // These four specialized vectors already exist in the metadata and Page-break
    // owners. This is not another general property inventory or a schema oracle.
    let case = property_expectations::find(expected_property)
        .expect("legacy owner's independent canonical record");
    let grammar = CssPropertyGrammar::from_name(grammar_name).unwrap();
    assert_eq!(grammar.target_property(), case.property);
    assert_ne!(grammar, case.property.grammar());
    let values = parse_component_values(&format!("{ordinary}/**/")).unwrap();
    let built = parse_property_value_for_grammar(grammar, values.clone(), CssImportance::Important)
        .unwrap();
    assert_constructed(
        &built,
        case.property,
        grammar,
        &values,
        CssImportance::Important,
    );
    let pending_values = parse_component_values("var(--known)").unwrap();
    let source =
        parse_property_value_for_grammar(grammar, pending_values.clone(), CssImportance::Important)
            .unwrap();
    assert_constructed(
        &source,
        case.property,
        grammar,
        &pending_values,
        CssImportance::Important,
    );
    let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
        panic!("legacy pending")
    };
    for input in ["inherit/*unfinished", "var(--known"] {
        let values = parse_component_values(input).unwrap();
        let witness = if input.contains("/*unfinished") {
            comment_witness(&values, input)
        } else {
            function_witness(&values, input)
        };
        let actual =
            parse_property_value_for_grammar(grammar, values.clone(), CssImportance::Important);
        if !matches!(actual, Err(ref error) if expected_error(error, &witness)) {
            failures.push(format!(
                "{grammar_name}/{input}/checked: expected original EOF; actual {actual:?}"
            ));
        }
    }
    let values = parse_component_values("inherit/*unfinished").unwrap();
    let witness = comment_witness(&values, "inherit/*unfinished");
    let actual = handle.reenter(values.clone());
    if !matches!(actual, Err(ref error) if matches!(error.kind(), CssExpansionErrorKind::InvalidReplacement(detail) if expected_error(detail, &witness)))
    {
        failures.push(format!(
            "{grammar_name}/reentry: expected original EOF; actual {actual:?}"
        ));
    }
    assert_eq!(
        handle
            .reenter(parse_component_values("var(--again").unwrap())
            .unwrap_err()
            .kind(),
        &CssExpansionErrorKind::ResidualSubstitution
    );
    for _ in 0..2 {
        let replacement = parse_component_values("inherit/**/").unwrap();
        assert_global_retry(
            handle.reenter(replacement.clone()).unwrap(),
            &source,
            &replacement,
            CssGlobalKeyword::Inherit,
        );
    }
    assert!(handle.source().same_occurrence(&source));
    assert_eq!(handle.source().known().unwrap().grammar(), grammar);
}

#[test]
fn existing_glyph_and_page_legacy_owners_keep_distinct_grammar_and_strict_original_closure() {
    let mut failures = Vec::new();
    legacy(
        "glyph-orientation-vertical",
        CssKnownProperty::TextOrientation,
        "90deg",
        &mut failures,
    );
    legacy(
        "page-break-before",
        CssKnownProperty::BreakBefore,
        "always",
        &mut failures,
    );
    legacy(
        "page-break-after",
        CssKnownProperty::BreakAfter,
        "always",
        &mut failures,
    );
    legacy(
        "page-break-inside",
        CssKnownProperty::BreakInside,
        "avoid",
        &mut failures,
    );
    finish(failures);
}

#[test]
fn mixed_programmatic_known_keyword_and_parsed_comment_keep_actual_original_snapshot() {
    let comment = parse_component_values("/*unfinished").unwrap();
    let witness = comment_witness(&comment, "/*unfinished");
    let mut items = vec![CssComponentValue::try_ident("inherit").unwrap()];
    items.extend(comment.items().iter().cloned());
    let values = CssComponentValues::try_new(items).unwrap();
    assert!(matches!(
        values.items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
    let mut failures = Vec::new();
    for case in fixtures() {
        reject_fronts(
            case.property,
            &values,
            &witness,
            "mixed standalone comment source",
            &mut failures,
        );
        if !matches!(case.metadata, MetadataExpectation::Unavailable) {
            let source = parse_property_value(
                CssPropertyNameRef::Known(case.property),
                parse_component_values("var(--known)").unwrap(),
                CssImportance::Important,
            )
            .unwrap();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("available mixed-input pending")
            };
            let actual = handle.reenter(values.clone());
            if !matches!(actual, Err(ref error) if matches!(error.kind(), CssExpansionErrorKind::InvalidReplacement(detail) if expected_error(detail, &witness)))
            {
                failures.push(format!(
                    "{}/mixed original snapshot/reentry: expected exact EOF; actual {actual:?}",
                    case.name
                ));
            }
            let complete =
                CssComponentValues::try_new(vec![CssComponentValue::try_ident("inherit").unwrap()])
                    .unwrap();
            assert_global_retry(
                handle.reenter(complete.clone()).unwrap(),
                &source,
                &complete,
                CssGlobalKeyword::Inherit,
            );
        }
    }
    finish(failures);
}

#[test]
fn programmatic_wrapper_preserves_recovered_child_original_eof_in_every_known_front() {
    let child = parse_component_values("future(value").unwrap();
    let witness = function_witness(&child, "future(value");
    let outer = CssComponentValue::try_function("outer", child.clone()).unwrap();
    let values = CssComponentValues::try_new(vec![outer]).unwrap();
    let before = values.clone();
    let mut failures = Vec::new();
    for case in fixtures() {
        reject_fronts(
            case.property,
            &values,
            &witness,
            "programmatic wrapper with recovered parsed child",
            &mut failures,
        );
        assert_eq!(values, before);
    }
    finish(failures);
}

#[test]
fn custom_name_recovery_contract_stays_separate_and_symbolic_with_empty_and_closed_controls() {
    let name = CssCustomPropertyName::try_new("--Theme").unwrap();
    for input in [
        "",
        "INITIAL",
        "var(--known)",
        "future(value)",
        "future(value",
        "inherit/*unfinished",
    ] {
        let values = parse_component_values(input).unwrap();
        let before = values.clone();
        for importance in IMPORTANCE {
            let source = parse_property_value(
                CssPropertyNameRef::Custom(&name),
                values.clone(),
                importance,
            )
            .unwrap();
            assert!(source.known().is_none());
            assert_eq!(source.custom().unwrap().name().as_str(), "--Theme");
            assert_eq!(source.importance(), importance);
            assert_eq!(source.value_components(), &values);
            assert_eq!(source.position(), None);
            assert!(source.parsed_name().is_none());
            assert!(source.parsed_value().is_none());
            let CssExpansion::Contributions(CssContributions::Custom(value)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("custom symbolic contribution")
            };
            assert!(value.source().same_occurrence(&source));
            assert_eq!(values, before);
        }
    }
    for input in ["future(value", "inherit/*unfinished"] {
        let browser = parse_style_attribute(&format!("--Theme:{input}"));
        assert!(!browser.is_clean());
        assert_eq!(browser.syntax().len(), 1);
    }
}

#[test]
fn repaired_component_output_keeps_byte_failures_atomic_and_original_closure_origin() {
    // This tests the EXISTING component budget boundary. Checked declarations have
    // no byte-limit parameter; do not claim this injects a competing checked error.
    // Keep time-before-serialization and other-known-after-serialization in source;
    // capacity/serialization errors remain owned by the existing component writer.
    let values = parse_component_values("future(1px").unwrap();
    let before = values.clone();
    let witness = function_witness(&values, "future(1px");
    let expected = "future(1px)";
    assert_eq!(expected.len(), 11);
    for _ in 0..2 {
        let error = values.serialize_with_limit(10).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::ByteLimit);
        assert_eq!(error.origin(), &witness);
        assert_eq!(values, before);
        let output = values.serialize_with_limit(11).unwrap();
        assert_eq!(output.as_css(), expected);
        assert_eq!(
            output.origin_at(10),
            Some(&CssSerializedOrigin::Token(witness.clone()))
        );
        assert_eq!(values, before);
    }
    let exact = CssComponentValueLimits::try_new(1, 2, 11).unwrap();
    let joined = CssComponentValues::try_new_with_limits(values.items().to_vec(), exact).unwrap();
    assert_eq!(joined, values);
    for (limits, kind) in [
        (
            CssComponentValueLimits::try_new(0, 2, 11).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
        (
            CssComponentValueLimits::try_new(1, 1, 11).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(1, 2, 10).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            CssComponentValues::try_new_with_limits(values.items().to_vec(), limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(values, before);
    }
    let complete = parse_component_values(expected).unwrap();
    assert_eq!(
        complete.serialize_with_limit(11).unwrap().as_css(),
        expected
    );
    assert_eq!(
        complete.serialize_with_limit(10).unwrap_err().kind(),
        CssComponentValueErrorKind::ByteLimit
    );
}
