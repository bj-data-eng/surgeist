#![forbid(unsafe_code)]

//! Public checked declaration construction and original-source provenance.
//!
//! Margin/padding expectations follow the selected CSS Box 3 grammar:
//! https://www.w3.org/TR/2024/REC-css-box-3-20240411/#margin-shorthand
//! https://www.w3.org/TR/2024/REC-css-box-3-20240411/#padding-properties
//! Whole-value keywords and pending authored values follow Cascade 5 / Variables 1:
//! https://www.w3.org/TR/2022/CR-css-cascade-5-20220113/#all-shorthand
//! https://www.w3.org/TR/2022/CR-css-variables-1-20220616/#using-variables
//! Source identity, coordinates and clean-report equality are Surgeist contracts.

use surgeist_css::{
    CssAuthoredDeclarationValue, CssComponentValue, CssComponentValueRef, CssComponentValues,
    CssCustomPropertyName, CssDeclaration, CssEdges, CssGlobalKeyword, CssImportance,
    CssKnownProperty, CssKnownPropertyValueRef, CssLength, CssParsedOrigin, CssPropertyNameRef,
    CssPropertyValueErrorKind, CssRule, CssSerializedOrigin, CssValueOrigin, CssValueTokenRef,
    ErrorKind, parse_component_values, parse_property_value, parse_sheet, parse_style_attribute,
    validate_sheet,
};

fn known_value(property: CssKnownProperty, value: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        parse_component_values(value).expect("component syntax"),
        CssImportance::Normal,
    )
    .expect("property grammar")
}

fn margin(declaration: &CssDeclaration) -> &CssEdges {
    let Some(CssKnownPropertyValueRef::Margin(value)) =
        declaration.known().unwrap().property_value()
    else {
        panic!("expected a typed margin value");
    };
    value
        .i01_subset()
        .expect("these explicit lengths are in the I01 domain")
}

fn assert_edges(actual: &CssEdges, expected: [f32; 4]) {
    assert_eq!(actual.top, CssLength::try_px(expected[0]).unwrap());
    assert_eq!(actual.right, CssLength::try_px(expected[1]).unwrap());
    assert_eq!(actual.bottom, CssLength::try_px(expected[2]).unwrap());
    assert_eq!(actual.left, CssLength::try_px(expected[3]).unwrap());
}

fn parsed(origin: &CssValueOrigin) -> &CssParsedOrigin {
    let CssValueOrigin::Parsed(origin) = origin else {
        panic!("expected an original parsed token, received {origin:?}");
    };
    origin
}

fn responsible(origin: &CssSerializedOrigin) -> &CssValueOrigin {
    match origin {
        CssSerializedOrigin::Token(origin) | CssSerializedOrigin::End(Some(origin)) => origin,
        other => panic!("expected a responsible token or its EOF cursor, received {other:?}"),
    }
}

fn assert_source_span(origin: &CssParsedOrigin, source: &str, expected: &str) {
    assert_eq!(origin.source().as_str(), source);
    let start = origin.span().start().byte_offset().value();
    let end = origin.span().end().byte_offset().value();
    assert_eq!(&source[start..end], expected);
    // All fixtures using this helper are one line. Count UTF-16 independently
    // from the UTF-8 byte offset, including supplementary Unicode scalars.
    assert_eq!(origin.span().start().line().value(), 0);
    assert_eq!(
        origin.span().start().column().value() as usize,
        source[..start].encode_utf16().count()
    );
    assert_eq!(
        origin.span().end().column().value() as usize,
        source[..end].encode_utf16().count()
    );
}

fn checked_construction() {
    let components = CssComponentValues::try_new(vec![
        CssComponentValue::try_dimension("1", "px").unwrap(),
        CssComponentValue::try_token(" ").unwrap(),
        CssComponentValue::try_dimension("2", "px").unwrap(),
        CssComponentValue::try_token(" ").unwrap(),
        CssComponentValue::try_dimension("3", "px").unwrap(),
    ])
    .unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Margin),
        components,
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(
        declaration.known().unwrap().property(),
        CssKnownProperty::Margin
    );
    assert_edges(margin(&declaration), [1.0, 2.0, 3.0, 2.0]);
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert_eq!(declaration.position(), None);
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    assert!(
        declaration
            .value_components()
            .items()
            .iter()
            .all(|item| matches!(item.origin(), CssValueOrigin::Programmatic))
    );
    assert!(matches!(declaration.value_components().items()[0].view(),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
        if number.representation() == "1" && unit == "px"));

    let negative =
        CssComponentValues::try_new(vec![CssComponentValue::try_dimension("-1", "px").unwrap()])
            .unwrap();
    let error = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Padding),
        negative,
        CssImportance::Normal,
    )
    .expect_err("negative padding is not valid property syntax");
    let CssPropertyValueErrorKind::Grammar(ErrorKind::InvalidPropertyValue(detail)) = error.kind()
    else {
        panic!("expected a structured padding grammar error: {error:?}");
    };
    assert_eq!(detail.property(), CssKnownProperty::Padding);
    assert!(matches!(
        responsible(error.origin()),
        CssValueOrigin::Programmatic
    ));
    println!("checked construction: ok");
}

fn declared_value_branches() {
    let ordinary = known_value(CssKnownProperty::Margin, "4px");
    assert_edges(margin(&ordinary), [4.0; 4]);
    let global = known_value(CssKnownProperty::All, "revert-layer");
    assert_eq!(global.known().unwrap().property(), CssKnownProperty::All);
    assert_eq!(
        global.known().unwrap().global(),
        Some(CssGlobalKeyword::RevertLayer)
    );
    assert!(global.known().unwrap().property_value().is_none());
    let pending = known_value(CssKnownProperty::Margin, "var(--space, 2px)");
    assert!(pending.known().unwrap().property_value().is_none());
    assert_eq!(
        pending
            .known()
            .unwrap()
            .substitution_dependent()
            .unwrap()
            .as_css(),
        "var(--space, 2px)"
    );

    let name = CssCustomPropertyName::try_new("--Theme").unwrap();
    let custom = parse_property_value(
        CssPropertyNameRef::Custom(&name),
        parse_component_values("unresolved(2px)").unwrap(),
        CssImportance::Important,
    )
    .unwrap();
    assert!(custom.known().is_none());
    assert_eq!(custom.custom().unwrap().name().as_str(), "--Theme");
    assert_eq!(
        custom.custom().unwrap().value().value().unwrap().as_css(),
        "unresolved(2px)"
    );
    assert_eq!(custom.importance(), CssImportance::Important);
    assert_eq!(custom.position(), None);
    let custom_global = parse_property_value(
        CssPropertyNameRef::Custom(&name),
        parse_component_values("initial").unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert_eq!(
        custom_global.custom().unwrap().value().global(),
        Some(CssGlobalKeyword::Initial)
    );
    let empty = parse_property_value(
        CssPropertyNameRef::Custom(&name),
        CssComponentValues::try_new(Vec::new()).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert!(empty.custom().unwrap().value().value().unwrap().is_empty());
    assert!(empty.value_components().items().is_empty());
    assert_eq!(empty.position(), None);
    assert!(empty.parsed_name().is_none());
    assert!(empty.parsed_value().is_none());
    println!("declared value branches: ok");
}

fn value_boundaries_and_error_origins() {
    for (value, invalid_token) in [
        ("1px!important", "!"),
        ("var(--space)!important", "!"),
        ("1px;padding:2px", ";"),
        ("var(--space);padding:2px", ";"),
        ("1px blue", "blue"),
    ] {
        let error = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Margin),
            parse_component_values(value).unwrap(),
            CssImportance::Normal,
        )
        .expect_err("one value cannot smuggle an annotation, declaration or extra grammar token");
        assert!(matches!(
            error.kind(),
            CssPropertyValueErrorKind::Grammar(_)
        ));
        assert_source_span(parsed(responsible(error.origin())), value, invalid_token);
    }

    let left = parse_component_values("1px").unwrap();
    let right = parse_component_values("2px").unwrap();
    let left_source = parsed(left.items()[0].origin()).source().clone();
    let right_source = parsed(right.items()[0].origin()).source().clone();
    let mixed = CssComponentValues::try_new(vec![
        left.items()[0].clone(),
        CssComponentValue::try_token(" ").unwrap(),
        right.items()[0].clone(),
    ])
    .unwrap();
    let declaration = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Margin),
        mixed,
        CssImportance::Normal,
    )
    .unwrap();
    assert_edges(margin(&declaration), [1.0, 2.0, 1.0, 2.0]);
    let items = declaration.value_components().items();
    assert!(
        parsed(items[0].origin())
            .source()
            .same_snapshot(&left_source)
    );
    assert!(matches!(items[1].origin(), CssValueOrigin::Programmatic));
    assert!(
        parsed(items[2].origin())
            .source()
            .same_snapshot(&right_source)
    );
    assert!(!left_source.same_snapshot(&right_source));
    assert_eq!(declaration.position(), None);
    println!("value boundaries and origins: ok");
}

fn parsed_declaration_coordinates() {
    let source = "/*😀*/.x{m\\61 rgin:1px 2px!important;padding:0;--Empty:;}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::Style(style)] = report.syntax().rules() else {
        panic!("one style rule")
    };
    let [margin_declaration, padding, empty] = style.declarations().as_slice() else {
        panic!("three declarations")
    };
    assert_edges(margin(margin_declaration), [1.0, 2.0, 1.0, 2.0]);
    assert_eq!(margin_declaration.importance(), CssImportance::Important);
    let name = margin_declaration.parsed_name().unwrap();
    assert_source_span(name, source, "m\\61 rgin");
    assert_eq!(name.span().start().byte_offset().value(), 11);
    assert_eq!(name.span().start().column().value(), 9);
    assert_eq!(margin_declaration.position(), Some(name.span().start()));
    assert_source_span(
        margin_declaration.parsed_value().unwrap(),
        source,
        "1px 2px",
    );
    assert_source_span(
        parsed(margin_declaration.value_components().items()[0].origin()),
        source,
        "1px",
    );
    assert_source_span(padding.parsed_name().unwrap(), source, "padding");
    let Some(CssKnownPropertyValueRef::Padding(padding_value)) =
        padding.known().unwrap().property_value()
    else {
        panic!("typed padding")
    };
    let expected_zero = CssEdges::all(CssLength::Zero);
    assert_eq!(padding_value.i01_subset(), Some(&expected_zero));
    assert_eq!(empty.custom().unwrap().name().as_str(), "--Empty");
    assert!(empty.custom().unwrap().value().value().unwrap().is_empty());
    assert!(empty.value_components().items().is_empty());
    let empty_origin = empty.parsed_value().unwrap();
    assert_source_span(empty_origin, source, "");
    assert_eq!(empty_origin.span().start(), empty_origin.span().end());
    assert_eq!(
        empty_origin.span().start().byte_offset().value(),
        source.find("--Empty:;").unwrap() + "--Empty:".len()
    );
    for declaration in [margin_declaration, padding, empty] {
        assert!(
            declaration
                .parsed_name()
                .unwrap()
                .source()
                .same_snapshot(name.source())
        );
        assert!(
            declaration
                .parsed_value()
                .unwrap()
                .source()
                .same_snapshot(name.source())
        );
    }
    assert_eq!(&validate_sheet(source).unwrap(), report.syntax());
    println!("parsed declaration coordinates: ok");
}

fn occurrence_identity_and_value_equality() {
    let first = known_value(CssKnownProperty::Margin, "1px");
    let second = known_value(CssKnownProperty::Margin, "1px");
    assert_eq!(first, second);
    assert!(!first.same_occurrence(&second));
    assert!(first.same_occurrence(&first.clone()));
    let first_source = parsed(first.value_components().items()[0].origin()).source();
    let second_source = parsed(second.value_components().items()[0].origin()).source();
    assert_eq!(first_source, second_source);
    assert!(!first_source.same_snapshot(second_source));

    let source = "margin:var(--space);margin:var(--space)";
    let first_parse = parse_style_attribute(source);
    let second_parse = parse_style_attribute(source);
    assert!(first_parse.is_clean());
    assert!(second_parse.is_clean());
    let [first, next] = first_parse.syntax().as_slice() else {
        panic!("two pending declarations")
    };
    assert!(!first.same_occurrence(next));
    assert_eq!(first.known(), next.known());
    assert_eq!(first_parse.syntax(), second_parse.syntax());
    assert!(!first.same_occurrence(&second_parse.syntax()[0]));

    fn require_eq<T: Eq>(left: &T, right: &T) {
        assert!(left == right);
    }
    let authored = CssAuthoredDeclarationValue::try_new("1px").unwrap();
    require_eq(
        &authored,
        &CssAuthoredDeclarationValue::try_new("1px").unwrap(),
    );
    assert_ne!(
        authored,
        CssAuthoredDeclarationValue::try_new("01px").unwrap()
    );
    println!("occurrence identity and value equality: ok");
}

fn collect_declarations<'a>(rules: &'a [CssRule], output: &mut Vec<&'a CssDeclaration>) {
    for rule in rules {
        match rule {
            CssRule::Style(style) => {
                output.extend(style.declarations().iter());
                collect_declarations(style.rules(), output);
            }
            CssRule::Media(media) => collect_declarations(media.rules(), output),
            other => panic!("unexpected rule in the media-only fixture: {other:?}"),
        }
    }
}

fn original_snapshot_after_structural_recovery() {
    // 65 media ancestors require structural chunking; 257 exceed the documented
    // 256-level limit. Both paths retain outer declarations from original input.
    for depth in [65, 257] {
        let source = format!(
            "/*😀*/.before{{margin:1px}}{}.inside{{margin:2px}}{}.after{{margin:3px}}",
            "@media all{".repeat(depth),
            "}".repeat(depth)
        );
        let report = parse_sheet(&source);
        assert_eq!(
            report.is_clean(),
            depth == 65,
            "depth {depth}: {:?}",
            report.diagnostics()
        );
        let mut declarations = Vec::new();
        collect_declarations(report.syntax().rules(), &mut declarations);
        let expected: &[f32] = if depth == 65 {
            &[1.0, 2.0, 3.0]
        } else {
            &[1.0, 3.0]
        };
        assert_eq!(declarations.len(), expected.len());
        let snapshot = declarations[0].parsed_name().unwrap().source();
        assert_eq!(snapshot.as_str(), source);
        for (declaration, expected) in declarations.iter().zip(expected) {
            assert_edges(margin(declaration), [*expected; 4]);
            let name = declaration.parsed_name().unwrap();
            assert_source_span(name, &source, "margin");
            assert!(name.source().same_snapshot(snapshot));
            let value = declaration.parsed_value().unwrap();
            assert_source_span(value, &source, &format!("{expected}px"));
            assert!(value.source().same_snapshot(snapshot));
            let token = parsed(declaration.value_components().items()[0].origin());
            assert_source_span(token, &source, &format!("{expected}px"));
            assert!(token.source().same_snapshot(snapshot));
        }
    }
    println!("original snapshot after structural recovery: ok");
}

fn main() {
    checked_construction();
    declared_value_branches();
    value_boundaries_and_error_origins();
    parsed_declaration_coordinates();
    occurrence_identity_and_value_equality();
    original_snapshot_after_structural_recovery();
}
