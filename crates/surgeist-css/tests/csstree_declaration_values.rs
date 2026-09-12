#![forbid(unsafe_code)]
//! Selected Syntax 3, Variables 1 and Filter Effects 1 declaration admission.
//! Expectations derive from the original input, independently of the corpus oracle.
use surgeist_css::{CssErrorCode, CssRecoveryAction, parse_declaration};

#[test]
fn raw_declaration_corpus_obeys_selected_grammar() {
    let expectations = [
        (
            "declaration/Declaration.json#/declaration property with # hack",
            Some("unexpected_token"),
        ),
        (
            "declaration/Declaration.json#/declaration property with $ hack",
            Some("unexpected_token"),
        ),
        (
            "declaration/Declaration.json#/declaration property with & hack",
            Some("unexpected_token"),
        ),
        (
            "declaration/Declaration.json#/declaration property with * hack",
            Some("unexpected_token"),
        ),
        (
            "declaration/Declaration.json#/declaration property with + hack",
            Some("unexpected_token"),
        ),
        (
            "declaration/Declaration.json#/declaration property with _ hack",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/declaration property with ~1 hack",
            Some("unexpected_token"),
        ),
        (
            "declaration/Declaration.json#/declaration property with ~1~1 hack",
            Some("unexpected_token"),
        ),
        (
            "declaration/Declaration.json#/declaration.0",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/declaration.1",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/declaration.c.0",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/declaration.c.1",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/declaration.c.2",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/declaration.c.3",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/declaration.s.0",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/declaration.s.1",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/declaration.s.2",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/declaration.s.3",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/error/0",
            Some("unexpected_token"),
        ),
        (
            "declaration/Declaration.json#/error/1",
            Some("unexpected_token"),
        ),
        (
            "declaration/Declaration.json#/error/2",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/shouldn't include !important into a value when parseValue is false",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/shouldn't include !important into a value when parseValue is false (brackets)",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/shouldn't include !important into a value when parseValue is false (function)",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/shouldn't include !important into a value when parseValue is false (w~1o whitespaces)",
            Some("unknown_property"),
        ),
        (
            "declaration/Declaration.json#/shouldn't parse a value when parseValue is false",
            Some("unknown_property"),
        ),
        (
            "declaration/Important.json#/!ie hack/0",
            Some("unknown_property"),
        ),
        (
            "declaration/Important.json#/!ie hack/1",
            Some("unknown_property"),
        ),
        (
            "declaration/Important.json#/\\9 hack",
            Some("unknown_property"),
        ),
        (
            "declaration/Important.json#/basic/0",
            Some("unknown_property"),
        ),
        (
            "declaration/Important.json#/basic/1",
            Some("unknown_property"),
        ),
        (
            "declaration/Important.json#/basic/2",
            Some("unknown_property"),
        ),
        (
            "declaration/Important.json#/divided by comment",
            Some("unknown_property"),
        ),
        (
            "declaration/Important.json#/divided by spaces",
            Some("unknown_property"),
        ),
        (
            "declaration/Important.json#/divided by spaces and comments",
            Some("unknown_property"),
        ),
        ("declaration/custom-property.json#/basic", None),
        ("declaration/custom-property.json#/complex balanced", None),
        (
            "declaration/custom-property.json#/double dash is a valid property name",
            Some("unknown_property"),
        ),
        ("declaration/custom-property.json#/empty", None),
        (
            "declaration/custom-property.json#/empty (parseCustomProperty:true)",
            None,
        ),
        (
            "declaration/custom-property.json#/empty with important",
            None,
        ),
        (
            "declaration/custom-property.json#/empty with important (parseCustomProperty:true)",
            None,
        ),
        (
            "declaration/custom-property.json#/error/0",
            Some("invalid_declaration_annotation"),
        ),
        ("declaration/custom-property.json#/only a comment", None),
        (
            "declaration/custom-property.json#/only a comment (parseCustomProperty:true)",
            None,
        ),
        (
            "declaration/custom-property.json#/should ignore everything inside block",
            None,
        ),
        (
            "declaration/custom-property.json#/should parse a custom property value when parseCustomProperty is true",
            None,
        ),
        (
            "declaration/custom-property.json#/should preserve single spaces",
            None,
        ),
        (
            "declaration/custom-property.json#/should preserve single spaces (parseCustomProperty:true)",
            None,
        ),
        (
            "declaration/custom-property.json#/should preserve single spaces with important",
            None,
        ),
        (
            "declaration/custom-property.json#/should preserve single spaces with important (parseCustomProperty:true)",
            None,
        ),
        (
            "declaration/custom-property.json#/spaces and comments",
            None,
        ),
        (
            "declaration/custom-property.json#/spaces and comments #2 (parseCustomProperty:true)",
            None,
        ),
        (
            "declaration/custom-property.json#/spaces and comments #3 (parseCustomProperty:true)",
            None,
        ),
        (
            "declaration/custom-property.json#/spaces and comments (parseCustomProperty:true)",
            None,
        ),
        ("declaration/custom-property.json#/value is {}-block", None),
        (
            "declaration/custom-property.json#/value should be parsed as balanced Raw/0",
            Some("unexpected_token"),
        ),
        (
            "declaration/custom-property.json#/value should be parsed as balanced Raw/1",
            Some("unexpected_token"),
        ),
        (
            "declaration/custom-property.json#/whitespace beetween comments (parseCustomProperty:true)",
            None,
        ),
        ("declaration/custom-property.json#/with important", None),
        (
            "declaration/custom-property.json#/with parentheses and important",
            None,
        ),
        (
            "declaration/filter.json#/alpha()",
            Some("invalid_property_value"),
        ),
        (
            "declaration/filter.json#/alpha() case insensetive",
            Some("invalid_property_value"),
        ),
        (
            "declaration/filter.json#/chroma()",
            Some("invalid_property_value"),
        ),
        (
            "declaration/filter.json#/dropshadow()",
            Some("invalid_property_value"),
        ),
        (
            "declaration/filter.json#/filter.0",
            Some("invalid_property_value"),
        ),
        (
            "declaration/filter.json#/filter.1",
            Some("invalid_property_value"),
        ),
        (
            "declaration/filter.json#/filter.2",
            Some("unexpected_token"),
        ),
        (
            "declaration/filter.json#/filter.3",
            Some("unknown_property"),
        ),
        (
            "declaration/filter.json#/filter.4",
            Some("unknown_property"),
        ),
        (
            "declaration/filter.json#/filter.5",
            Some("unknown_property"),
        ),
        (
            "declaration/filter.json#/filter.c.0",
            Some("invalid_property_value"),
        ),
        (
            "declaration/filter.json#/filter.c.1",
            Some("invalid_property_value"),
        ),
        (
            "declaration/filter.json#/filter.s.0",
            Some("invalid_property_value"),
        ),
        (
            "declaration/filter.json#/filter.s.1",
            Some("invalid_property_value"),
        ),
        (
            "declaration/filter.json#/filter.s.2",
            Some("invalid_property_value"),
        ),
        (
            "declaration/filter.json#/unclosed functions should not be invalid per spec, not sure it's true for IE (need to be check and fail may be)",
            Some("invalid_property_value"),
        ),
    ];
    let fixtures = [
        include_str!("corpus/csstree/expectations/declaration/Declaration.json"),
        include_str!("corpus/csstree/expectations/declaration/Important.json"),
        include_str!("corpus/csstree/expectations/declaration/custom-property.json"),
        include_str!("corpus/csstree/expectations/declaration/filter.json"),
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
    for (id, code) in expectations {
        let clean = code.is_none();
        let matching: Vec<_> = cases.iter().filter(|case| case["id"] == id).collect();
        assert_eq!(matching.len(), 1, "one original input for {id}");
        let input = matching[0]["input"].as_str().unwrap();
        let report = parse_declaration(input);
        assert_eq!(report.is_clean(), clean, "{id}: {report:?}");
        assert_eq!(report.syntax().is_some(), clean, "{id}: {report:?}");
        let mut expected_class = serde_json::json!({
            "kind": if clean {"clean"} else {"strict_rejected"},
            "retained_syntax": {
                "extractor": {"kind": "style_declarations"},
                "predicate": {"relation": if clean {"nonempty"} else {"empty"}}
            }
        });
        if clean {
            let declaration = report.syntax().as_ref().unwrap();
            assert!(declaration.custom().is_some(), "{id}");
            assert!(report.diagnostics().is_empty(), "{id}");
        } else {
            let code = code.unwrap();
            let expected_code = match code {
                "unexpected_token" => CssErrorCode::UnexpectedToken,
                "unknown_property" => CssErrorCode::UnknownProperty,
                "invalid_property_value" => CssErrorCode::InvalidPropertyValue,
                "invalid_declaration_annotation" => CssErrorCode::InvalidDeclarationAnnotation,
                _ => panic!("unknown expected diagnostic category"),
            };
            expected_class["diagnostics"] = serde_json::json!([{
                "code": code, "action": "reject_input", "payload_relation": "intersects"
            }]);
            assert_eq!(report.diagnostics().len(), 1, "{id}");
            for diagnostic in report.diagnostics() {
                assert_eq!(diagnostic.error().code(), expected_code, "{id}");
                assert!(
                    diagnostic.error().position().byte_offset().value() < input.len(),
                    "{id}"
                );
                assert_eq!(diagnostic.action(), CssRecoveryAction::RejectInput, "{id}");
                assert_eq!(diagnostic.span().start().byte_offset().value(), 0, "{id}");
                assert_eq!(
                    diagnostic.span().end().byte_offset().value(),
                    input.len(),
                    "{id}"
                );
            }
        }
        let matching: Vec<_> = classes["records"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["id"] == id)
            .collect();
        assert_eq!(matching.len(), 1, "one expected class for {id}");
        assert_eq!(matching[0]["class"], expected_class, "{id}");
    }
}
