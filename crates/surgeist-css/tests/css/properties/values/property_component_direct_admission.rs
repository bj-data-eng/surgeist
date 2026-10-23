//! Independent public contracts for the common property grammar's two inputs.
//! Property stimuli and typed wrappers are the existing handwritten expectations;
//! lexical boundaries and failure ordering follow the owning public contracts.

use crate::css::support::property_expectations;

use crate::css::support::property_admission::GrammarFacts;
use property_expectations::{BoundaryOutcome, CatalogExpectation};
use surgeist_css::*;

#[test]
fn common_authored_property_records_match_typed_source_and_checked_admission() {
    for case in property_expectations::CASES {
        let Some(authored) = case.ordinary_stimulus() else {
            continue;
        };
        if case.wrapper.is_none() {
            continue;
        }
        let source = parse_property_value_text(
            authored,
            CssPropertyNameRef::Known(case.property),
            CssImportance::Normal,
        );
        assert!(source.is_clean(), "{}: {source:?}", case.name);
        let checked = parse_property_value(
            CssPropertyNameRef::Known(case.property),
            parse_component_values(authored).unwrap(),
            CssImportance::Normal,
        )
        .unwrap_or_else(|error| panic!("{}: {error:?}", case.name));
        for declaration in [source.syntax().as_ref().unwrap(), &checked] {
            let known = declaration.known().unwrap();
            case.assert_wrapper(known, known.property_value().unwrap(), authored);
        }
        if let Some(CatalogExpectation::Grammar { boundary, .. }) = &case.catalog {
            let source = parse_property_value_text(
                boundary.authored,
                CssPropertyNameRef::Known(case.property),
                CssImportance::Normal,
            );
            let checked = parse_component_values(boundary.authored).map(|values| {
                parse_property_value(
                    CssPropertyNameRef::Known(case.property),
                    values,
                    CssImportance::Normal,
                )
            });
            match boundary.outcome {
                BoundaryOutcome::Rejected => {
                    assert!(source.syntax().is_none(), "{}: {source:?}", case.name);
                    assert!(
                        checked.is_err() || checked.as_ref().unwrap().is_err(),
                        "{}: {checked:?}",
                        case.name,
                    );
                }
                BoundaryOutcome::Accepted(assertion) => {
                    assert!(source.is_clean(), "{}: {source:?}", case.name);
                    assertion(source.syntax().as_ref().unwrap().known().unwrap());
                    assertion(checked.unwrap().unwrap().known().unwrap());
                }
            }
        }
    }
}

#[test]
fn checked_math_preserves_whitespace_and_exact_decimal_spelling() {
    let source = "calc(1.0000000000000000001px + 2%)";
    let values = parse_component_values(source).unwrap();
    let original = values.clone();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Width),
        values,
        CssImportance::Normal,
    )
    .expect("a dimensional sum with authored operator whitespace");
    assert_eq!(declaration.value_components(), &original);
    let CssComponentValueRef::Function(function) = declaration.value_components().items()[0].view()
    else {
        panic!("retained authored calculation");
    };
    assert!(matches!(
        function.values().items()[0].view(),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
            if number.representation() == "1.0000000000000000001" && unit == "px"
    ));
    for source in ["calc(1px+ 2%)", "calc(1px +2%)"] {
        let values = parse_component_values(source).unwrap();
        assert!(
            parse_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::Width),
                values,
                CssImportance::Normal,
            )
            .is_err(),
            "math operators require actual authored whitespace: {source}"
        );
        assert!(
            parse_property_value_text(
                source,
                CssPropertyNameRef::Known(CssKnownProperty::Width),
                CssImportance::Normal,
            )
            .syntax()
            .is_none()
        );
    }
}

#[test]
fn checked_context_retains_property_quirks_and_page_restrictions() {
    let context = CssParserContext::new(CssParserMode::Quirks);
    let width = context
        .parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Width),
            parse_component_values("7").unwrap(),
            CssImportance::Normal,
        )
        .expect("the named width owner grants quirky unitless lengths");
    let CssKnownPropertyValueRef::Width(value) = width.known().unwrap().property_value().unwrap()
    else {
        panic!("typed width");
    };
    assert_eq!(
        value
            .value()
            .serialize_specified(Default::default())
            .unwrap(),
        "7px"
    );
    assert_eq!(width.parser_context(), context);
    assert!(
        context
            .parse_page_property_value(
                CssPropertyNameRef::Known(CssKnownProperty::MarginTop),
                parse_component_values("7").unwrap(),
                CssImportance::Normal,
            )
            .is_err(),
        "Page margins do not acquire nonzero unitless element grammar"
    );
    let recovered = parse_component_values("future(").unwrap();
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Width),
            recovered.clone(),
            CssImportance::Normal,
        )
        .is_err()
    );
    let custom = CssCustomPropertyName::try_new("--tokens").unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Custom(&custom),
        recovered.clone(),
        CssImportance::Normal,
    )
    .expect("the custom-property producer retains recovered syntax");
    assert_eq!(declaration.value_components(), &recovered);
    assert_eq!(declaration.custom().unwrap().name().as_str(), "--tokens");
}

#[test]
fn rejected_checked_values_keep_the_responsible_token_summary() {
    for (property, text) in [
        (CssKnownProperty::Width, "not-a-width"),
        (CssKnownProperty::Color, "not-a-color"),
    ] {
        let source = parse_property_value_text(
            text,
            CssPropertyNameRef::Known(property),
            CssImportance::Normal,
        );
        assert!(source.syntax().is_none());
        let checked = parse_property_value(
            CssPropertyNameRef::Known(property),
            parse_component_values(text).unwrap(),
            CssImportance::Normal,
        )
        .expect_err("the identifier is outside the selected value grammar");
        assert!(checked.code().is_some(), "a grammar rejection: {checked:?}");
        let rejections: [&dyn GrammarFacts; 2] = [source.diagnostics()[0].error(), &checked];
        for kind in rejections {
            assert!(
                matches!(
                    kind.grammar_code(),
                    Some(CssErrorCode::InvalidPropertyValue | CssErrorCode::InvalidColorSyntax)
                ),
                "property or color grammar owns this rejection: {kind:?}"
            );
            let encountered = kind
                .responsible_token()
                .expect("the responsible actual token qualifies the grammar error");
            assert_eq!(encountered.kind(), CssTokenKind::Ident);
            assert_eq!(encountered.authored(), text);
        }
    }
}

#[test]
fn checked_error_summaries_keep_programmatic_and_escaped_mixed_origins() {
    for (component, expected) in [
        (
            CssComponentValue::try_ident("not-a-width").unwrap(),
            "not-a-width",
        ),
        (
            parse_component_values(r"\6e ot-a-width").unwrap().items()[0].clone(),
            r"\6e ot-a-width",
        ),
    ] {
        let origin = component.origin().clone();
        let unrelated = parse_component_values("/*different snapshot*/").unwrap();
        let values = CssComponentValues::try_new(
            vec![unrelated.items()[0].clone(), component],
            Default::default(),
        )
        .unwrap();
        drop(unrelated);
        let error = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Width),
            values,
            CssImportance::Normal,
        )
        .expect_err("a rejected identifier retains its own diagnostic identity");
        let detail = &error;
        assert_eq!(
            detail.code(),
            Some(CssErrorCode::InvalidPropertyValue),
            "the width grammar owns this rejection: {error:?}"
        );
        let encountered = detail.encountered().expect("responsible identifier");
        assert_eq!(encountered.kind(), CssTokenKind::Ident);
        assert_eq!(encountered.authored(), expected);
        assert_eq!(error.origin(), CssSerializedOrigin::Token(origin.clone()));
        if let CssValueOrigin::Parsed(expected) = origin {
            let CssSerializedOrigin::Token(CssValueOrigin::Parsed(actual)) = error.origin() else {
                panic!("the real identifier snapshot survives input destruction");
            };
            assert!(actual.source().same_snapshot(expected.source()));
            assert_eq!(actual.span(), expected.span());
        }
    }
}

#[test]
fn trivia_does_not_replace_the_responsible_token_in_checked_errors() {
    let spelling = "123456789012345678901234567890";
    let comment = parse_component_values("/*different snapshot*/").unwrap();
    let number = parse_component_values(spelling).unwrap();
    let expected = number.items()[0].origin().clone();
    let values = CssComponentValues::try_new(
        vec![comment.items()[0].clone(), number.items()[0].clone()],
        Default::default(),
    )
    .unwrap();
    drop(comment);
    drop(number);
    let error = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Display),
        values,
        CssImportance::Normal,
    )
    .expect_err("display requires its keyword grammar");
    let detail = &error;
    assert_eq!(
        detail.code(),
        Some(CssErrorCode::InvalidPropertyValue),
        "display grammar rejection: {error:?}"
    );
    let encountered = detail.encountered().expect("the actual numeric token");
    assert_eq!(encountered.kind(), CssTokenKind::Number);
    assert_eq!(encountered.authored(), spelling);
    assert_eq!(error.origin(), CssSerializedOrigin::Token(expected));
}

#[test]
fn raw_root_delimiters_keep_precedence_over_unadmitted_bad_url_tokens() {
    for (source, responsible, offset, kind) in [
        ("1px!url(a b)", "!", 3, CssTokenKind::Delim),
        ("url(a b);", ";", 8, CssTokenKind::Semicolon),
    ] {
        let report = parse_property_value_text(
            source,
            CssPropertyNameRef::Known(CssKnownProperty::Width),
            CssImportance::Normal,
        );
        assert!(report.syntax().is_none());
        let [diagnostic] = report.diagnostics() else {
            panic!("one raw input failure: {report:?}");
        };
        let detail = diagnostic.error();
        assert_eq!(
            detail.code(),
            CssErrorCode::UnexpectedToken,
            "root boundary owns rejection: {diagnostic:?}"
        );
        assert_eq!(detail.encountered().unwrap().kind(), kind);
        assert_eq!(detail.encountered().unwrap().authored(), responsible);
        assert_eq!(diagnostic.error().position().byte_offset().value(), offset);
        assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput);
        assert_eq!(diagnostic.span().start().byte_offset().value(), 0);
        assert_eq!(diagnostic.span().end().byte_offset().value(), source.len());
    }
}

#[test]
fn source_bad_url_keeps_property_error_context_and_later_independent_declaration() {
    let source = "width:1px url(a b);color:red";
    let report = parse_style_attribute(source);
    let [declaration] = &report.syntax()[..] else {
        panic!("only the following color declaration survives: {report:?}");
    };
    assert_eq!(
        declaration.known().unwrap().property(),
        CssKnownProperty::Color
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid width declaration: {report:?}");
    };
    let detail = diagnostic.error();
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidPropertyValue,
        "the property grammar owns lexical rejection: {diagnostic:?}"
    );
    assert_eq!(detail.known_property().unwrap(), CssKnownProperty::Width);
    assert_eq!(detail.encountered().unwrap().kind(), CssTokenKind::BadUrl);
    assert_eq!(diagnostic.error().position().byte_offset().value(), 10);
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
}

#[test]
fn source_name_and_keyframe_policy_rejections_precede_payload_grammar() {
    let report = parse_style_attribute("unknown:url(a b);color:red");
    assert_eq!(report.syntax().len(), 1);
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one unknown property: {report:?}");
    };
    assert_eq!(diagnostic.error().code(), CssErrorCode::UnknownProperty);
    assert_eq!(diagnostic.error().position().byte_offset().value(), 0);
    let report = parse_keyframe_declaration_block("{animation-name:url(a b);color:red}");
    let block = report.syntax().as_ref().expect("retained keyframe body");
    assert_eq!(block.body().len(), 1);
    assert_eq!(
        block.body()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    let [diagnostic] = report.diagnostics() else {
        panic!("one forbidden keyframe property: {report:?}");
    };
    let detail = diagnostic.error();
    assert_eq!(
        detail.code(),
        CssErrorCode::InvalidPropertyValue,
        "keyframe property policy owns rejection: {diagnostic:?}"
    );
    assert_eq!(
        detail.known_property().unwrap(),
        CssKnownProperty::AnimationName
    );
    // Source diagnostics qualify an inferred property rejection with the real
    // name token; the later malformed value does not acquire ownership.
    let encountered = detail.encountered().expect("responsible property name");
    assert_eq!(encountered.kind(), CssTokenKind::Ident);
    assert_eq!(encountered.authored(), "animation-name");
    assert_eq!(diagnostic.error().position().byte_offset().value(), 1);
}

#[test]
fn separately_owned_tokens_keep_their_lexical_boundaries_and_original_snapshots() {
    for (first, second) in [("1", "px"), ("au", "to")] {
        let (values, first_origin) = {
            let first = parse_component_values(first).unwrap();
            let second = parse_component_values(second).unwrap();
            let first_origin = first.items()[0].origin().clone();
            (
                CssComponentValues::try_new(
                    vec![first.items()[0].clone(), second.items()[0].clone()],
                    Default::default(),
                )
                .unwrap(),
                first_origin,
            )
        };
        let error = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Width),
            values,
            CssImportance::Normal,
        )
        .expect_err("two independently supplied tokens must not become 1px or auto");
        assert!(
            (error.code() == Some(CssErrorCode::InvalidPropertyValue)
                && error.known_property().unwrap() == CssKnownProperty::Width)
        );
        assert_eq!(error.origin(), CssSerializedOrigin::Token(first_origin));
    }
    let values = {
        let left = parse_component_values("1px").unwrap();
        let right = parse_component_values("2px").unwrap();
        CssComponentValues::try_new(
            vec![left.items()[0].clone(), right.items()[0].clone()],
            Default::default(),
        )
        .unwrap()
    };
    let before = values.clone();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Margin),
        values,
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(declaration.value_components(), &before);
    let CssKnownPropertyValueRef::Margin(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed margin shorthand");
    };
    for (actual, expected) in value
        .value()
        .assigned_values()
        .iter()
        .zip(["1px", "2px", "1px", "2px"])
    {
        let CssMarginValue::LengthPercentage(actual) = actual else {
            panic!("explicit length margin");
        };
        assert_eq!(
            actual.serialize_specified(Default::default()).unwrap(),
            expected
        );
    }
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert!(declaration.parsed_value().is_none());
}
