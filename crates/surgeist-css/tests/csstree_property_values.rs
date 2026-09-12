#![forbid(unsafe_code)]
//! Property grammar admissions derive from original CSS inputs and their mapped
//! Width, Color and BackgroundImage grammars, independently of captured output.
//! Literal expectations use numeric/color/URL semantics rather than serialization.
use surgeist_css::{
    CssCalcLength, CssCalculationType, CssErrorCode, CssImageLayer, CssImportance,
    CssKnownProperty, CssKnownPropertyValueRef, CssLength, CssPropertyNameRef, CssRecoveryAction,
    CssTokenKind, CssValueOrigin, ErrorKind, parse_property_value_text,
};

#[derive(Clone, Copy)]
enum Expected {
    Px(f32),
    Percent(f32),
    Rgba([u8; 3], u8),
    Url(&'static str),
    Calculation,
    Symbolic,
    Reject(CssErrorCode, CssTokenKind, usize, u32, u32),
}

#[test]
fn raw_property_corpus_obeys_selected_grammar_and_original_coordinates() {
    use Expected::*;
    let expectations = [
        (
            "value/Dimension.json#/IE hack",
            CssKnownProperty::Width,
            Reject(
                CssErrorCode::InvalidPropertyValue,
                CssTokenKind::Dimension,
                0,
                0,
                0,
            ),
        ),
        (
            "value/Dimension.json#/dimension #0",
            CssKnownProperty::Width,
            Px(10.0),
        ),
        (
            "value/Dimension.json#/dimension #1",
            CssKnownProperty::Width,
            Px(0.1),
        ),
        (
            "value/Dimension.json#/dimension #2",
            CssKnownProperty::Width,
            Px(12.34),
        ),
        (
            "value/Dimension.json#/dimension #3",
            CssKnownProperty::Width,
            Px(0.0),
        ),
        (
            "value/HexColor.json#/basic/0",
            CssKnownProperty::Color,
            Rgba([17, 0, 0], 255),
        ),
        (
            "value/HexColor.json#/basic/1",
            CssKnownProperty::Color,
            Rgba([18, 58, 188], 255),
        ),
        (
            "value/HexColor.json#/basic/2",
            CssKnownProperty::Color,
            Rgba([171, 193, 35], 255),
        ),
        (
            "value/HexColor.json#/basic/3",
            CssKnownProperty::Color,
            Rgba([161, 178, 195], 212),
        ),
        (
            "value/HexColor.json#/basic/4",
            CssKnownProperty::Color,
            Rgba([170, 187, 204], 255),
        ),
        (
            "value/HexColor.json#/edge cases/0",
            CssKnownProperty::Color,
            Reject(
                CssErrorCode::InvalidColorSyntax,
                CssTokenKind::Hash,
                0,
                0,
                0,
            ),
        ),
        (
            "value/HexColor.json#/edge cases/1",
            CssKnownProperty::Color,
            Reject(
                CssErrorCode::InvalidColorSyntax,
                CssTokenKind::IdHash,
                0,
                0,
                0,
            ),
        ),
        (
            "value/HexColor.json#/edge cases/10",
            CssKnownProperty::Color,
            Reject(
                CssErrorCode::InvalidColorSyntax,
                CssTokenKind::IdHash,
                0,
                0,
                0,
            ),
        ),
        (
            "value/HexColor.json#/edge cases/11",
            CssKnownProperty::Color,
            Reject(
                CssErrorCode::InvalidColorSyntax,
                CssTokenKind::Hash,
                0,
                0,
                0,
            ),
        ),
        (
            "value/HexColor.json#/edge cases/2",
            CssKnownProperty::Color,
            Reject(
                CssErrorCode::InvalidColorSyntax,
                CssTokenKind::Hash,
                0,
                0,
                0,
            ),
        ),
        (
            "value/HexColor.json#/edge cases/3",
            CssKnownProperty::Color,
            Reject(
                CssErrorCode::InvalidColorSyntax,
                CssTokenKind::IdHash,
                0,
                0,
                0,
            ),
        ),
        (
            "value/HexColor.json#/edge cases/4",
            CssKnownProperty::Color,
            Reject(
                CssErrorCode::InvalidColorSyntax,
                CssTokenKind::Hash,
                0,
                0,
                0,
            ),
        ),
        (
            "value/HexColor.json#/edge cases/5",
            CssKnownProperty::Color,
            Reject(
                CssErrorCode::InvalidColorSyntax,
                CssTokenKind::Hash,
                0,
                0,
                0,
            ),
        ),
        (
            "value/HexColor.json#/edge cases/6",
            CssKnownProperty::Color,
            Reject(
                CssErrorCode::InvalidColorSyntax,
                CssTokenKind::Hash,
                0,
                0,
                0,
            ),
        ),
        (
            "value/HexColor.json#/edge cases/7",
            CssKnownProperty::Color,
            Reject(
                CssErrorCode::InvalidColorSyntax,
                CssTokenKind::Hash,
                0,
                0,
                0,
            ),
        ),
        (
            "value/HexColor.json#/edge cases/8",
            CssKnownProperty::Color,
            Reject(
                CssErrorCode::InvalidColorSyntax,
                CssTokenKind::IdHash,
                0,
                0,
                0,
            ),
        ),
        (
            "value/HexColor.json#/edge cases/9",
            CssKnownProperty::Color,
            Rgba([30, 87, 233], 255),
        ),
        (
            "value/HexColor.json#/error/0",
            CssKnownProperty::Color,
            Reject(
                CssErrorCode::InvalidColorSyntax,
                CssTokenKind::Delim,
                0,
                0,
                0,
            ),
        ),
        (
            "value/HexColor.json#/error/1",
            CssKnownProperty::Color,
            Reject(
                CssErrorCode::InvalidColorSyntax,
                CssTokenKind::Delim,
                0,
                0,
                0,
            ),
        ),
        (
            "value/HexColor.json#/error/2",
            CssKnownProperty::Color,
            Reject(
                CssErrorCode::InvalidColorSyntax,
                CssTokenKind::Delim,
                0,
                0,
                0,
            ),
        ),
        (
            "value/Percentage.json#/percentage.0",
            CssKnownProperty::Width,
            Percent(10.0),
        ),
        (
            "value/Percentage.json#/percentage.1",
            CssKnownProperty::Width,
            Percent(0.1),
        ),
        (
            "value/Percentage.json#/percentage.2",
            CssKnownProperty::Width,
            Percent(12.34),
        ),
        (
            "value/Percentage.json#/percentage.3",
            CssKnownProperty::Width,
            Reject(
                CssErrorCode::InvalidPropertyValue,
                CssTokenKind::Percentage,
                0,
                0,
                0,
            ),
        ),
        (
            "value/Url.json#/error/0",
            CssKnownProperty::BackgroundImage,
            Reject(
                CssErrorCode::InvalidPropertyValue,
                CssTokenKind::BadUrl,
                0,
                0,
                0,
            ),
        ),
        (
            // Bad-URL consumption ends at the first `)`. The remaining root
            // closer is rejected by the boundary guard before value grammar.
            "value/Url.json#/error/1",
            CssKnownProperty::BackgroundImage,
            Reject(
                CssErrorCode::UnexpectedToken,
                CssTokenKind::CloseParenthesis,
                12,
                0,
                12,
            ),
        ),
        (
            "value/Url.json#/error/2",
            CssKnownProperty::BackgroundImage,
            Reject(
                CssErrorCode::InvalidPropertyValue,
                CssTokenKind::BadUrl,
                0,
                0,
                0,
            ),
        ),
        (
            "value/Url.json#/uri escaping",
            CssKnownProperty::BackgroundImage,
            Url("1 (2).png"),
        ),
        (
            "value/Url.json#/uri with parentheses",
            CssKnownProperty::BackgroundImage,
            Url("p(1).png"),
        ),
        (
            "value/Url.json#/uri with parentheses in string",
            CssKnownProperty::BackgroundImage,
            Url("p (1).png"),
        ),
        (
            "value/Url.json#/uri with special symbols",
            CssKnownProperty::BackgroundImage,
            Url("p{1};.png"),
        ),
        (
            "value/Url.json#/uri.0",
            CssKnownProperty::BackgroundImage,
            Url("http://test.com"),
        ),
        (
            "value/Url.json#/uri.1",
            CssKnownProperty::BackgroundImage,
            Url("http://test.com"),
        ),
        (
            "value/Url.json#/uri.c.1",
            CssKnownProperty::BackgroundImage,
            Url("/*test*/http://test.com/*test*/"),
        ),
        (
            "value/Url.json#/uri.s.0",
            CssKnownProperty::BackgroundImage,
            Url("http://test.com"),
        ),
        (
            "value/Url.json#/uri.s.1",
            CssKnownProperty::BackgroundImage,
            Url("http://test.com"),
        ),
        (
            "value/function/calc.json#/basic",
            CssKnownProperty::Width,
            Reject(
                CssErrorCode::InvalidPropertyValue,
                CssTokenKind::Number,
                5,
                0,
                5,
            ),
        ),
        (
            "value/function/calc.json#/custom property",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/calc.json#/nested",
            CssKnownProperty::Width,
            Calculation,
        ),
        (
            "value/function/calc.json#/with dimension and percentage",
            CssKnownProperty::Width,
            Calculation,
        ),
        (
            "value/function/var.json#/basic",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/complex balanced",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/empty fallback",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/empty fallback (parseCustomProperty:true)",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/error/0",
            CssKnownProperty::Width,
            Reject(
                CssErrorCode::InvalidPropertyValue,
                CssTokenKind::Delim,
                4,
                0,
                4,
            ),
        ),
        (
            "value/function/var.json#/error/1",
            CssKnownProperty::Width,
            Reject(
                CssErrorCode::InvalidPropertyValue,
                CssTokenKind::Delim,
                12,
                0,
                12,
            ),
        ),
        (
            "value/function/var.json#/error/2",
            CssKnownProperty::Width,
            Reject(
                CssErrorCode::InvalidPropertyValue,
                CssTokenKind::Semicolon,
                15,
                0,
                15,
            ),
        ),
        (
            "value/function/var.json#/falback is {}-block",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/only comments (parseCustomProperty:true)",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/should be a error here (since unmatched tokens are disallowed), but ok for now",
            CssKnownProperty::Width,
            Reject(
                CssErrorCode::InvalidPropertyValue,
                CssTokenKind::CloseSquareBracket,
                14,
                0,
                14,
            ),
        ),
        (
            "value/function/var.json#/should ignore everything inside falback's block",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/should parse a fallback value when parseCustomProperty is true",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/should preserve single spaces",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/should preserve single spaces (parseCustomProperty:true)",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/spaces and comments",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/spaces and comments #2 (parseCustomProperty:true)",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/spaces and comments #3 (parseCustomProperty:true)",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/spaces and comments #4 (parseCustomProperty:true)",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/spaces and comments (parseCustomProperty:true)",
            CssKnownProperty::Width,
            Symbolic,
        ),
        (
            "value/function/var.json#/unclosed var() is not an error",
            CssKnownProperty::Width,
            Reject(
                CssErrorCode::InvalidPropertyValue,
                CssTokenKind::CloseParenthesis,
                13,
                0,
                13,
            ),
        ),
        (
            "value/function/var.json#/with fallback",
            CssKnownProperty::Width,
            Symbolic,
        ),
    ];
    let fixtures = [
        include_str!("corpus/csstree/expectations/value/Dimension.json"),
        include_str!("corpus/csstree/expectations/value/HexColor.json"),
        include_str!("corpus/csstree/expectations/value/Percentage.json"),
        include_str!("corpus/csstree/expectations/value/Url.json"),
        include_str!("corpus/csstree/expectations/value/function/calc.json"),
        include_str!("corpus/csstree/expectations/value/function/var.json"),
    ];
    let cases: Vec<serde_json::Value> = fixtures
        .iter()
        .flat_map(|source| {
            serde_json::from_str::<serde_json::Value>(source).unwrap()["cases"]
                .as_array()
                .unwrap()
                .clone()
        })
        .collect();
    assert_eq!(cases.len(), expectations.len());
    let classes: serde_json::Value =
        serde_json::from_str(include_str!("csstree/expected-classes.json")).unwrap();
    for (id, property, expected) in expectations {
        let matching: Vec<_> = cases.iter().filter(|case| case["id"] == id).collect();
        assert_eq!(matching.len(), 1, "one original input for {id}");
        let input = matching[0]["input"].as_str().unwrap();
        let report = parse_property_value_text(
            input,
            CssPropertyNameRef::Known(property),
            CssImportance::Normal,
        );
        let clean = !matches!(expected, Reject(..));
        assert_eq!(report.is_clean(), clean, "{id}: {report:?}");
        assert_eq!(report.syntax().is_some(), clean, "{id}: {report:?}");
        let property_name = match property {
            CssKnownProperty::Width => "width",
            CssKnownProperty::Color => "color",
            CssKnownProperty::BackgroundImage => "background-image",
            _ => panic!("only independently mapped property grammars"),
        };
        let mut expected_class = serde_json::json!({
            "kind": if clean { "clean" } else { "strict_rejected" },
            "retained_syntax": {
                "extractor": { "kind": "known_declaration", "index": 0, "property": property_name },
                "predicate": { "relation": if clean { "nonempty" } else { "empty" } }
            }
        });
        if let Reject(code, ..) = expected {
            let code = match code {
                CssErrorCode::InvalidPropertyValue => "invalid_property_value",
                CssErrorCode::UnexpectedToken => "unexpected_token",
                CssErrorCode::InvalidColorSyntax => "invalid_color_syntax",
                _ => panic!("only independently specified diagnostic categories"),
            };
            expected_class["diagnostics"] = serde_json::json!([{
                "code": code, "action": "reject_input", "payload_relation": "intersects"
            }]);
        }
        let matching_classes: Vec<_> = classes["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|record| record["id"] == id)
            .collect();
        assert_eq!(matching_classes.len(), 1, "one expected class for {id}");
        assert_eq!(matching_classes[0]["class"], expected_class, "{id}");
        if let Reject(code, token_kind, byte, line, column) = expected {
            assert_eq!(report.diagnostics().len(), 1, "{id}: {report:?}");
            let diagnostic = &report.diagnostics()[0];
            assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput, "{id}");
            assert_eq!(diagnostic.error().code(), code, "{id}");
            let position = diagnostic.error().position();
            assert_eq!(position.byte_offset().value(), byte, "{id}");
            assert_eq!(position.line().value(), line, "{id}");
            assert_eq!(position.column().value(), column, "{id}");
            assert_eq!(diagnostic.span().start().byte_offset().value(), 0, "{id}");
            assert_eq!(
                diagnostic.span().end().byte_offset().value(),
                input.len(),
                "{id}"
            );
            let encountered = match diagnostic.error().kind() {
                ErrorKind::InvalidPropertyValue(error) => {
                    assert_eq!(error.property(), property, "{id}");
                    error.encountered()
                }
                ErrorKind::InvalidColorSyntax(error) => error.encountered(),
                ErrorKind::UnexpectedToken(error) => Some(error.encountered()),
                _ => panic!("expected property, color or boundary diagnostic: {id}"),
            };
            assert_eq!(encountered.unwrap().kind(), token_kind, "{id}");
            continue;
        }
        assert!(report.diagnostics().is_empty(), "{id}");
        let declaration = report.syntax().as_ref().unwrap();
        assert_eq!(
            declaration.property_name(),
            CssPropertyNameRef::Known(property),
            "{id}"
        );
        assert_eq!(declaration.importance(), CssImportance::Normal, "{id}");
        assert!(declaration.position().is_none(), "{id}");
        assert!(declaration.parsed_name().is_none(), "{id}");
        let origin = declaration.parsed_value().unwrap();
        assert_eq!(origin.source().as_str(), input, "{id}");
        assert_eq!(origin.span().start().byte_offset().value(), 0, "{id}");
        assert_eq!(
            origin.span().end().byte_offset().value(),
            input.len(),
            "{id}"
        );
        for component in declaration.value_components().items() {
            let CssValueOrigin::Parsed(token_origin) = component.origin() else {
                panic!("raw component must retain source: {id}");
            };
            assert!(token_origin.source().same_snapshot(origin.source()), "{id}");
        }
        let known = declaration.known().unwrap();
        match expected {
            Symbolic => {
                assert!(known.property_value().is_none(), "{id}");
                assert_eq!(
                    known.substitution_dependent().unwrap().as_css(),
                    input,
                    "{id}"
                );
            }
            Px(expected_number) | Percent(expected_number) => {
                let CssKnownPropertyValueRef::Width(value) = known.property_value().unwrap() else {
                    panic!("expected width: {id}");
                };
                let actual = match (
                    value.i01_subset().unwrap(),
                    property,
                    matches!(expected, Percent(_)),
                ) {
                    (CssLength::Px(value), CssKnownProperty::Width, false) => value.value(),
                    (CssLength::Percent(value), CssKnownProperty::Width, true) => value.value(),
                    _ => panic!("expected selected literal width unit: {id}"),
                };
                assert!(
                    (actual - expected_number).abs()
                        <= f32::EPSILON * expected_number.abs().max(1.0),
                    "{id}: {actual} != {expected_number}"
                );
            }
            Rgba(rgb, alpha) => {
                let CssKnownPropertyValueRef::Color(value) = known.property_value().unwrap() else {
                    panic!("expected color: {id}");
                };
                let color = value.i01_subset().unwrap().as_rgba().unwrap();
                assert_eq!([color.red(), color.green(), color.blue()], rgb, "{id}");
                assert!(
                    (color.alpha() - f32::from(alpha) / 255.0).abs() <= f32::EPSILON,
                    "{id}"
                );
            }
            Url(expected) => {
                let CssKnownPropertyValueRef::BackgroundImage(value) =
                    known.property_value().unwrap()
                else {
                    panic!("expected background image: {id}");
                };
                let [CssImageLayer::Url(url)] = value.i01_subset().unwrap().layers() else {
                    panic!("expected one URL layer: {id}");
                };
                assert_eq!(url.as_str(), expected, "{id}");
                assert!(url.modifiers().is_empty(), "{id}");
            }
            Calculation => {
                let CssKnownPropertyValueRef::Width(value) = known.property_value().unwrap() else {
                    panic!("expected width: {id}");
                };
                let CssLength::Calc(CssCalcLength::Typed(calculation)) =
                    value.i01_subset().unwrap()
                else {
                    panic!("expected retained typed calculation: {id}");
                };
                assert_eq!(
                    calculation.result_type(),
                    CssCalculationType::LengthPercentage,
                    "{id}"
                );
            }
            Reject(..) => unreachable!("handled above"),
        }
    }
}
