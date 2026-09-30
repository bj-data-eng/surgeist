#![forbid(unsafe_code)]

//! Direct typed and bounded specified-value evidence for Containment 2 §4.

use surgeist_css::*;

const PROPERTY: CssKnownProperty = CssKnownProperty::ContentVisibility;

fn parsed(value: &str) -> CssDeclaration {
    let source = format!("content-visibility:{value}!important");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [declaration] = report.syntax().as_slice() else {
        panic!("one parsed declaration")
    };
    declaration.clone()
}

fn terminal(source: &CssDeclaration) -> CssLonghandContribution {
    let CssExpansion::Contributions(CssContributions::Longhands(values)) =
        expand_declaration(source).unwrap()
    else {
        panic!("one ordinary longhand")
    };
    let [item] = values.items() else {
        panic!("one terminal")
    };
    item.clone()
}

#[test]
fn intrinsic_visible_and_each_authored_keyword_have_exact_typed_terminal_values() {
    let CssPropertyKindRef::Longhand(metadata) = PROPERTY.metadata().unwrap().kind() else {
        panic!("content-visibility longhand")
    };
    assert_eq!(metadata.property().known_property(), PROPERTY);
    assert!(!metadata.inherited_by_default());
    let initial_value = metadata.initial_value();
    let CssInitialValueRef::Value(initial) = initial_value.view() else {
        panic!("fixed intrinsic initial")
    };
    let CssLonghandValueRef::ContentVisibility(initial) = initial.view() else {
        panic!("content-visibility initial")
    };
    assert_eq!(initial, &CssContentVisibility::Visible);

    for (authored, expected) in [
        ("VISIBLE", CssContentVisibility::Visible),
        ("hidden", CssContentVisibility::Hidden),
        ("auto", CssContentVisibility::Auto),
    ] {
        let source = parsed(authored);
        let CssKnownPropertyValueRef::ContentVisibility(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("content-visibility wrapper")
        };
        assert_eq!(value.value(), &expected);
        assert_eq!(value.value(), &expected);
        assert_eq!(value.as_css(), authored);

        let item = terminal(&source);
        assert_eq!(item.property(), PROPERTY);
        let CssLonghandValueRef::ContentVisibility(actual) = item.ordinary_value().unwrap().view()
        else {
            panic!("content-visibility terminal value")
        };
        assert_eq!(actual, &expected);
        assert!(item.source().same_occurrence(&source));
        assert_eq!(item.source().importance(), CssImportance::Important);
    }
}

#[test]
fn programmatic_identifier_and_parsed_identifier_retain_distinct_origins() {
    let token = CssComponentValue::try_ident("auto").unwrap();
    assert!(matches!(token.origin(), CssValueOrigin::Programmatic));
    let constructed = parse_property_value(
        CssPropertyNameRef::Known(PROPERTY),
        CssComponentValues::try_new(vec![token]).unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert!(matches!(
        constructed.value_components().items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
    let parsed = parsed("auto");
    assert!(matches!(
        parsed.value_components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    for source in [&constructed, &parsed] {
        let CssKnownPropertyValueRef::ContentVisibility(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("typed current value")
        };
        assert_eq!(value.value(), &CssContentVisibility::Auto);
        let item = terminal(source);
        let CssLonghandValueRef::ContentVisibility(actual) = item.ordinary_value().unwrap().view()
        else {
            panic!("typed terminal value")
        };
        assert_eq!(actual, &CssContentVisibility::Auto);
    }
    assert_eq!(constructed.importance(), CssImportance::Normal);
    assert_eq!(parsed.importance(), CssImportance::Important);
}

#[test]
fn canonical_keywords_obey_exact_node_and_byte_limits_without_mutating_authored_text() {
    for (value, expected) in [
        (CssContentVisibility::Visible, "visible"),
        (CssContentVisibility::Hidden, "hidden"),
        (CssContentVisibility::Auto, "auto"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    expected.len(),
                ))
                .unwrap(),
            expected
        );
        for (limits, error) in [
            (
                CssSpecifiedValueSerializationLimits::new(0, 1, expected.len()),
                CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 0, expected.len()),
                CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, expected.len() - 1),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
            (
                CssSpecifiedValueSerializationLimits::new(1, 1, 0),
                CssSpecifiedValueSerializationErrorKind::ByteLimit,
            ),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                error
            );
            assert_eq!(value.serialize_specified().unwrap(), expected);
        }
        let authored = format!("content-visibility:{}!important", expected.to_uppercase());
        let report = parse_style_attribute(&authored);
        assert!(report.is_clean());
        let CssKnownPropertyValueRef::ContentVisibility(wrapper) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("typed authored value")
        };
        assert_eq!(wrapper.value(), &value);
        assert_eq!(wrapper.as_css(), expected.to_uppercase());
        assert_eq!(wrapper.value().serialize_specified().unwrap(), expected);
        assert_eq!(
            wrapper
                .value()
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 0))
                .unwrap_err()
                .kind(),
            CssSpecifiedValueSerializationErrorKind::ByteLimit
        );
        assert_eq!(wrapper.as_css(), expected.to_uppercase());
    }
}
