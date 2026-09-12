#![forbid(unsafe_code)]

use surgeist_css::{
    CssComponentValue, CssComponentValueRef, CssComponentValues, CssRule, CssSerializedOrigin,
    CssSupportsConditionKind, CssValueOrigin, parse_component_values, parse_sheet,
};

// Authored syntax preserves the complete opaque enclosure, including escapes,
// comments and exact numeric spelling. Coordinates address the original sheet.
#[test]
fn supports_opaque_enclosures_preserve_spelling_and_original_positions() {
    for enclosure in [r"f\75 ture(/*🦊*/ +001.2300e+02 [x])", "(/*🦊*/ 2px [x])"] {
        let prefix = "/*🦊*/\n@supports ";
        let source = format!("{prefix}{enclosure} {{}}");
        let report = parse_sheet(&source);
        assert!(report.is_clean(), "{:?}", report.diagnostics());
        let [CssRule::Supports(rule)] = report.syntax().rules() else {
            panic!("expected one supports rule");
        };
        let CssSupportsConditionKind::GeneralEnclosed(value) = rule.condition().kind() else {
            panic!("expected an opaque enclosure");
        };
        assert_eq!(value.authored(), Some(enclosure));
        assert_eq!(
            value.position().unwrap().byte_offset().value(),
            prefix.len()
        );
        assert_eq!(value.position().unwrap().line().value(), 1);
        assert_eq!(value.position().unwrap().column().value(), 10);
    }
}

// CSS Syntax closes a function at EOF. The emitted delimiter must retain that
// recovery origin rather than pretending that a closing token was authored.
#[test]
fn eof_function_serialization_retains_the_implicit_closing_origin() {
    let source = "future(/*🦊*/ +001.2300e+02";
    let values = parse_component_values(source).unwrap();
    let [component] = values.items() else {
        panic!("expected one component");
    };
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("expected a function");
    };
    let CssValueOrigin::ImplicitClosure { opening, at } = function.closing_origin() else {
        panic!("expected an EOF-implied close");
    };
    assert_eq!(opening.span().start().byte_offset().value(), 0);
    assert_eq!(at.span().start().byte_offset().value(), source.len());
    let serialized = values.serialize().unwrap();
    assert_eq!(serialized.as_css(), format!("{source})"));
    assert_eq!(
        serialized.origin_at(source.len()),
        Some(&CssSerializedOrigin::Token(
            function.closing_origin().clone()
        ))
    );
}

// A supplied wrapper around parsed children owns only its delimiters. Child
// provenance remains attached to the actual parsed input.
#[test]
fn programmatic_function_preserves_parsed_child_and_delimiter_origins() {
    let children = parse_component_values("+001.2300e+02").unwrap();
    let child_origin = children.items()[0].origin().clone();
    let component = CssComponentValue::try_function("future", children).unwrap();
    assert_eq!(component.origin(), &CssValueOrigin::Programmatic);
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("expected a function");
    };
    assert_eq!(function.values().items()[0].origin(), &child_origin);
    assert_eq!(function.closing_origin(), &CssValueOrigin::Programmatic);
    let values = CssComponentValues::try_new(vec![component]).unwrap();
    let serialized = values.serialize().unwrap();
    assert_eq!(serialized.as_css(), "future(+001.2300e+02)");
    assert_eq!(
        serialized.origin_at(7),
        Some(&CssSerializedOrigin::Token(child_origin))
    );
}

fn enclosed(source: &str) -> surgeist_css::CssGeneralEnclosed {
    let values = parse_component_values(source).unwrap();
    let [component] = values.items() else {
        panic!("expected exactly one component");
    };
    surgeist_css::CssGeneralEnclosed::try_from_component(component.clone()).unwrap()
}

#[test]
fn cloned_parsed_components_retain_the_complete_authored_slice_and_closure() {
    for source in [
        r"f\75 ture(/*🦊*/ +001.2300e+02 [x])",
        "(/*🦊*/ 2px [x])",
        "future(/*🦊*/ nested([x",
        "(2px",
        "()",
        "future()",
    ] {
        let value = enclosed(source);
        let rebuilt =
            surgeist_css::CssGeneralEnclosed::try_from_component(value.component().clone())
                .unwrap();
        assert_eq!(value.authored(), Some(source));
        assert_eq!(rebuilt.authored(), Some(source));
        assert_eq!(value, rebuilt);
        assert_eq!(value.origin(), rebuilt.origin());
        assert_eq!(value.position().unwrap().byte_offset().value(), 0);
        let serialized = value.serialize().unwrap();
        assert_eq!(
            enclosed(serialized.as_css()).serialize().unwrap().as_css(),
            serialized.as_css(),
        );
    }
    let value = enclosed("future(nested([x");
    assert_eq!(value.serialize().unwrap().as_css(), "future(nested([x]))");
    let CssComponentValueRef::Function(function) = value.component().view() else {
        panic!("expected a function");
    };
    assert!(matches!(
        function.closing_origin(),
        CssValueOrigin::ImplicitClosure { .. }
    ));
}

#[test]
fn supplied_enclosures_have_no_fabricated_authored_span_even_with_parsed_children() {
    let children = parse_component_values("1e+003").unwrap();
    let expected_origin = children.items()[0].origin().clone();
    for value in [
        surgeist_css::CssGeneralEnclosed::try_function("future", children.clone()).unwrap(),
        surgeist_css::CssGeneralEnclosed::try_parenthesized(children).unwrap(),
    ] {
        assert_eq!(value.authored(), None);
        assert_eq!(value.position(), None);
        assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
        let serialized = value.serialize().unwrap();
        let offset = serialized.as_css().find("1e+003").unwrap();
        assert_eq!(
            serialized.origin_at(offset),
            Some(&CssSerializedOrigin::Token(expected_origin.clone()))
        );
        assert_eq!(
            serialized.origin_at(serialized.as_css().len() - 1),
            Some(&CssSerializedOrigin::Token(CssValueOrigin::Programmatic))
        );
    }
    let empty = CssComponentValues::try_new(vec![]).unwrap();
    assert_eq!(
        surgeist_css::CssGeneralEnclosed::try_function("future", empty.clone())
            .unwrap()
            .serialize()
            .unwrap()
            .as_css(),
        "future()"
    );
    assert_eq!(
        surgeist_css::CssGeneralEnclosed::try_parenthesized(empty)
            .unwrap()
            .serialize()
            .unwrap()
            .as_css(),
        "()"
    );
}

#[test]
fn wrong_outer_components_report_the_supplied_origin() {
    for source in ["x", "[x]", "{x}", "/*x*/", " "] {
        let values = parse_component_values(source).unwrap();
        let component = values.items()[0].clone();
        let origin = component.origin().clone();
        let error = surgeist_css::CssGeneralEnclosed::try_from_component(component).unwrap_err();
        assert!(matches!(
            error,
            surgeist_css::CssGeneralEnclosedError::WrongOuterComponent { .. }
        ));
        assert_eq!(error.origin(), &origin);
    }
    let error = surgeist_css::CssGeneralEnclosed::try_from_component(
        CssComponentValue::try_ident("x").unwrap(),
    )
    .unwrap_err();
    assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
}

#[test]
fn constructor_errors_preserve_component_syntax_and_depth_failures() {
    let empty = CssComponentValues::try_new(vec![]).unwrap();
    let error = surgeist_css::CssGeneralEnclosed::try_function("url", empty).unwrap_err();
    let surgeist_css::CssGeneralEnclosedError::Component(component) = error else {
        panic!("expected a component construction error");
    };
    assert_eq!(
        component.kind(),
        surgeist_css::CssComponentValueErrorKind::InvalidFunction
    );
    assert_eq!(component.origin(), &CssValueOrigin::Programmatic);
    let mut children =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("x").unwrap()]).unwrap();
    for _ in 0..256 {
        let block = CssComponentValue::try_block(surgeist_css::CssBlockKind::Parenthesis, children)
            .unwrap();
        children = CssComponentValues::try_new(vec![block]).unwrap();
    }
    let error = surgeist_css::CssGeneralEnclosed::try_parenthesized(children).unwrap_err();
    let surgeist_css::CssGeneralEnclosedError::Component(component) = error else {
        panic!("expected the component depth limit");
    };
    assert_eq!(
        component.kind(),
        surgeist_css::CssComponentValueErrorKind::NestingLimit
    );
    assert_eq!(component.origin(), &CssValueOrigin::Programmatic);
}

#[test]
fn equality_compares_spelling_and_source_text_but_not_snapshot_identity() {
    let first = enclosed("future(01.00000000000000000000000001)");
    let second = enclosed("future(01.00000000000000000000000001)");
    assert_eq!(first, second);
    let (CssValueOrigin::Parsed(left), CssValueOrigin::Parsed(right)) =
        (first.origin(), second.origin())
    else {
        panic!("expected parsed origins");
    };
    assert!(!left.source().same_snapshot(right.source()));
    assert_ne!(first, enclosed("future(01.00000000000000000000000002)"));
    assert_ne!(enclosed("future(1)"), enclosed("future(1"));
    let supplied = surgeist_css::CssGeneralEnclosed::try_function(
        "future",
        CssComponentValues::try_new(vec![
            CssComponentValue::try_token("01.00000000000000000000000001").unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        first.serialize().unwrap().as_css(),
        supplied.serialize().unwrap().as_css()
    );
    assert_ne!(first, supplied);
    // Parsed numeric components and supplied tokens differ even though both
    // spell the exact same number; programmatic token parsing leaks no origin.
    assert_ne!(
        parse_component_values("1").unwrap().items()[0],
        CssComponentValue::try_token("1").unwrap()
    );
}

#[test]
fn supports_fallback_collects_only_its_own_enclosure_and_shares_the_sheet_snapshot() {
    let source = "/*🦊*/\n@supports future(1) and (2px) and selector(undeclared|x) {}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Supports(rule)] = report.syntax().rules() else {
        panic!("expected supports");
    };
    let CssSupportsConditionKind::And(list) = rule.condition().kind() else {
        panic!("expected three operands");
    };
    assert_eq!(list.conditions().len(), 3);
    let mut snapshots = Vec::new();
    for (condition, expected) in
        list.conditions()
            .iter()
            .zip(["future(1)", "(2px)", "selector(undeclared|x)"])
    {
        let CssSupportsConditionKind::GeneralEnclosed(value) = condition.kind() else {
            panic!("expected an opaque operand");
        };
        assert_eq!(value.authored(), Some(expected));
        assert_eq!(value.serialize().unwrap().as_css(), expected);
        let CssValueOrigin::Parsed(origin) = value.origin() else {
            panic!("expected original parsed provenance");
        };
        assert_eq!(origin.source().as_str(), source);
        assert_eq!(
            origin.span().start().byte_offset().value(),
            source.find(expected).unwrap()
        );
        snapshots.push(origin.source());
        assert_eq!(
            surgeist_css::CssGeneralEnclosed::try_from_component(value.component().clone())
                .unwrap(),
            *value
        );
    }
    assert!(snapshots[0].same_snapshot(snapshots[1]));
    assert!(snapshots[1].same_snapshot(snapshots[2]));
}
