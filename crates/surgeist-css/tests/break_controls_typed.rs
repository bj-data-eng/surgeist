#![forbid(unsafe_code)]

use surgeist_css::*;

#[test]
fn checked_keyword_domains_render_every_canonical_value_with_exact_limits() {
    for (value, expected) in [
        (CssBreakBetween::Auto, "auto"),
        (CssBreakBetween::Avoid, "avoid"),
        (CssBreakBetween::AvoidPage, "avoid-page"),
        (CssBreakBetween::Page, "page"),
        (CssBreakBetween::Left, "left"),
        (CssBreakBetween::Right, "right"),
        (CssBreakBetween::Recto, "recto"),
        (CssBreakBetween::Verso, "verso"),
        (CssBreakBetween::AvoidColumn, "avoid-column"),
        (CssBreakBetween::Column, "column"),
        (CssBreakBetween::AvoidRegion, "avoid-region"),
        (CssBreakBetween::Region, "region"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        for limits in [
            CssSpecifiedValueSerializationLimits::new(0, 1, expected.len()),
            CssSpecifiedValueSerializationLimits::new(1, 0, expected.len()),
            CssSpecifiedValueSerializationLimits::new(1, 1, expected.len() - 1),
        ] {
            assert!(value.serialize_specified_with_limits(limits).is_err());
        }
    }
    for (value, expected) in [
        (CssBreakInside::Auto, "auto"),
        (CssBreakInside::Avoid, "avoid"),
        (CssBreakInside::AvoidPage, "avoid-page"),
        (CssBreakInside::AvoidColumn, "avoid-column"),
        (CssBreakInside::AvoidRegion, "avoid-region"),
    ] {
        assert_eq!(value.serialize_specified().unwrap(), expected);
        assert_eq!(
            value
                .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                    1,
                    1,
                    expected.len()
                ))
                .unwrap(),
            expected
        );
        for limits in [
            CssSpecifiedValueSerializationLimits::new(0, 1, expected.len()),
            CssSpecifiedValueSerializationLimits::new(1, 0, expected.len()),
            CssSpecifiedValueSerializationLimits::new(1, 1, expected.len() - 1),
        ] {
            assert!(value.serialize_specified_with_limits(limits).is_err());
        }
    }
}

#[test]
fn constructed_components_retain_programmatic_provenance_and_typed_mapping() {
    for (name, source, target, expected) in [
        (
            "break-before",
            "avoid-region",
            CssKnownProperty::BreakBefore,
            CssBreakBetween::AvoidRegion,
        ),
        (
            "page-break-before",
            "always",
            CssKnownProperty::BreakBefore,
            CssBreakBetween::Page,
        ),
        (
            "break-after",
            "left",
            CssKnownProperty::BreakAfter,
            CssBreakBetween::Left,
        ),
    ] {
        let grammar = CssPropertyGrammar::from_name(name).unwrap();
        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_ident(source).unwrap()])
                .unwrap();
        let declaration =
            parse_property_value_for_grammar(grammar, components, CssImportance::Normal).unwrap();
        assert!(declaration.position().is_none());
        assert!(matches!(
            declaration.value_components().items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
        assert_eq!(declaration.known().unwrap().property(), target);
        assert_eq!(declaration.known().unwrap().grammar(), grammar);
        match declaration.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::BreakBefore(value) => assert_eq!(value.current(), &expected),
            CssKnownPropertyValueRef::BreakAfter(value) => assert_eq!(value.current(), &expected),
            _ => panic!("typed between break"),
        }
    }
    let grammar = CssPropertyGrammar::from_name("page-break-inside").unwrap();
    let components =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("avoid").unwrap()]).unwrap();
    let declaration =
        parse_property_value_for_grammar(grammar, components, CssImportance::Normal).unwrap();
    assert!(matches!(
        declaration.value_components().items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
    let CssKnownPropertyValueRef::BreakInside(value) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("typed inside break")
    };
    assert_eq!(value.current(), &CssBreakInside::Avoid);
}

#[test]
fn parsed_wrappers_keep_authored_spelling_and_distinct_legacy_grammar() {
    for (name, authored, target, expected) in [
        (
            "break-before",
            "avoid-column",
            CssKnownProperty::BreakBefore,
            CssBreakBetween::AvoidColumn,
        ),
        (
            "break-after",
            "VERSO",
            CssKnownProperty::BreakAfter,
            CssBreakBetween::Verso,
        ),
        (
            "page-break-before",
            "always",
            CssKnownProperty::BreakBefore,
            CssBreakBetween::Page,
        ),
        (
            "page-break-after",
            "RECTO",
            CssKnownProperty::BreakAfter,
            CssBreakBetween::Recto,
        ),
        (
            "page-break-before",
            "r\\65 cto",
            CssKnownProperty::BreakBefore,
            CssBreakBetween::Recto,
        ),
    ] {
        let source = format!("{name}:{authored}");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
        let declaration = &report.syntax()[0];
        let known = declaration.known().unwrap();
        assert_eq!(known.property(), target);
        assert_eq!(known.grammar().name(), name);
        match known.property_value().unwrap() {
            CssKnownPropertyValueRef::BreakBefore(value) => {
                assert_eq!(value.current(), &expected);
                assert_eq!(value.as_css(), authored);
            }
            CssKnownPropertyValueRef::BreakAfter(value) => {
                assert_eq!(value.current(), &expected);
                assert_eq!(value.as_css(), authored);
            }
            _ => panic!("expected before or after"),
        }
    }
    let report = parse_style_attribute("break-inside:avoid-region;page-break-inside:avoid");
    assert!(report.is_clean());
    for (declaration, (expected, grammar)) in report.syntax().iter().zip([
        (CssBreakInside::AvoidRegion, "break-inside"),
        (CssBreakInside::Avoid, "page-break-inside"),
    ]) {
        let CssKnownPropertyValueRef::BreakInside(value) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("typed break-inside")
        };
        assert_eq!(value.current(), &expected);
        assert_eq!(declaration.known().unwrap().grammar().name(), grammar);
    }
}
