#![forbid(unsafe_code)]
//! Existing admission boundaries for the represented Masking 1 layer fields.
//! Masking 1 CRD 2021-08-05 §7.9 uses || between image, position[/size]
//! and repeat; any group order is valid, but slash/size belongs to position.
//! https://www.w3.org/TR/2021/CRD-css-masking-1-20210805/#the-mask
//! No future serializer API or additional mask fields are required by these tests.

use surgeist_css::*;

const ORDERS: &[&str] = &[
    "url(mask.png) 3.000e1px top / contain no-repeat",
    "url(mask.png) no-repeat 3.000e1px top / contain",
    "3.000e1px top / contain url(mask.png) no-repeat",
    "3.000e1px top / contain no-repeat url(mask.png)",
    "no-repeat url(mask.png) 3.000e1px top / contain",
    "no-repeat 3.000e1px top / contain url(mask.png)",
];
const PREFIXES: &[&str] = &[
    "center / 10px no-repeat url(mask.png)",
    "center / auto no-repeat url(mask.png)",
    "center url(mask.png) no-repeat",
    "space round url(mask.png) center / contain",
    "repeat-x center url(mask.png)",
];
const INVALID: &[&str] = &[
    "no-repeat no-repeat no-repeat",
    "repeat-x repeat-y",
    "url(a) url(b)",
    "none url(a)",
    "center center center",
    "center / contain / cover",
    "/ contain",
    "center / -1px",
    "center / 10px 20px 30px",
    "no-repeat url(a),",
    "no-repeat url(a), , none",
    "no-repeat url(a) nonsense",
];

fn mask(declaration: &CssDeclaration) -> &CssMaskList {
    let known = declaration.known().unwrap();
    assert_eq!(known.property(), CssKnownProperty::Mask);
    assert!(known.global().is_none());
    assert!(known.substitution_dependent().is_none());
    let CssKnownPropertyValueRef::Mask(wrapper) = known.property_value().unwrap() else {
        panic!("typed mask value")
    };
    wrapper.value()
}

fn position(actual: CssSourcePosition, source: &str, offset: usize) {
    assert_eq!(actual.byte_offset().value(), offset);
    assert_eq!(actual.line().value(), 0);
    assert_eq!(
        actual.column().value() as usize,
        source[..offset].encode_utf16().count()
    );
}

fn exact_layer(declaration: &CssDeclaration, source: &str) {
    let value = mask(declaration);
    let [layer] = value.layers() else {
        panic!("one layer")
    };
    let physical = layer.position().unwrap();
    let CssHorizontalPosition::Offset(offset) = physical.horizontal() else {
        panic!("offset")
    };
    assert!(matches!(offset.literal_component().unwrap().view(),
        CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit })
        if number.representation() == "3.000e1" && unit == "px"));
    assert!(matches!(physical.vertical(), CssVerticalPosition::Top));
    let CssValueOrigin::Parsed(origin) = offset.origin() else {
        panic!("parsed offset")
    };
    assert_eq!(origin.source().as_str(), source);
    let start = source.find("3.000e1px").unwrap();
    position(origin.span().start(), source, start);
    position(origin.span().end(), source, start + "3.000e1px".len());
    // Use the retained exact position, then independently assert all other
    // represented fields, including those lacking individual public getters.
    let expected = CssMaskList::try_new(vec![
        CssMaskLayer::try_new(
            Some(CssImageValue::Url(CssUrl::new("mask.png"))),
            Some(physical.clone()),
            Some(CssBackgroundSize::Contain),
            Some(CssBackgroundRepeat::Axes {
                x: CssBackgroundRepeatStyle::NoRepeat,
                y: CssBackgroundRepeatStyle::NoRepeat,
            }),
        )
        .unwrap(),
    ])
    .unwrap();
    assert_eq!(*value, expected);
}

fn checked(value: &str, grammar: bool) -> CssDeclaration {
    let components = parse_component_values(value).unwrap();
    let result = if grammar {
        parse_property_value_for_grammar(
            CssKnownProperty::Mask.grammar(),
            components.clone(),
            CssImportance::Important,
        )
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Mask),
            components.clone(),
            CssImportance::Important,
        )
    };
    let declaration = result.unwrap_or_else(|error| panic!("{value}: {error:?}"));
    assert_eq!(declaration.importance(), CssImportance::Important);
    assert!(declaration.position().is_none());
    assert!(declaration.parsed_name().is_none());
    assert!(declaration.parsed_value().is_none());
    assert_eq!(declaration.value_components(), &components);
    mask(&declaration);
    declaration
}

#[test]
fn parsed_permutations_retain_all_fields_exact_origins_importance_and_siblings() {
    for value in ORDERS {
        let source = format!("/*😀*/color:red; mask:{value}!important; width:1px");
        let report = parse_style_attribute(&source);
        assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
        let [before, declaration, after] = report.syntax().as_slice() else {
            panic!("three declarations")
        };
        assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
        assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
        assert_eq!(declaration.importance(), CssImportance::Important);
        position(
            declaration.position().unwrap(),
            &source,
            source.find("mask:").unwrap(),
        );
        let region = declaration.parsed_value().unwrap();
        assert_eq!(region.source().as_str(), source);
        let span = region.span();
        assert_eq!(
            source[span.start().byte_offset().value()..span.end().byte_offset().value()].trim(),
            *value
        );
        exact_layer(declaration, &source);
    }
}

#[test]
fn validator_admits_all_six_component_group_orders() {
    for value in ORDERS {
        assert!(
            validate_style_attribute(&format!("mask:{value}!important; color:blue")).is_ok(),
            "{value}"
        );
    }
}

#[test]
fn checked_property_admits_permutations_with_supplied_components_and_origins() {
    for value in ORDERS {
        exact_layer(&checked(value, false), value);
    }
}

#[test]
fn checked_grammar_admits_permutations_with_supplied_components_and_origins() {
    for value in ORDERS {
        exact_layer(&checked(value, true), value);
    }
}

#[test]
fn optional_second_repeat_size_and_position_stop_at_following_groups() {
    for value in PREFIXES {
        let report = parse_style_attribute(&format!("mask:{value}"));
        assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
        assert_eq!(mask(&report.syntax()[0]).layers().len(), 1);
        assert!(validate_style_attribute(&format!("mask:{value}")).is_ok());
        checked(value, false);
        checked(value, true);
    }
}

#[test]
fn layer_boundaries_and_existing_complete_orders_remain_admitted() {
    for value in [
        "none",
        "no-repeat",
        "url(mask.png) center / contain no-repeat",
        "no-repeat, none",
        "none, url(mask.png) center / contain no-repeat",
    ] {
        let report = parse_style_attribute(&format!("mask:{value}"));
        assert!(report.is_clean(), "{value}: {:?}", report.diagnostics());
        let count = if value.contains(',') { 2 } else { 1 };
        assert_eq!(mask(&report.syntax()[0]).layers().len(), count);
        assert!(validate_style_attribute(&format!("mask:{value}")).is_ok());
        assert_eq!(mask(&checked(value, false)).layers().len(), count);
        assert_eq!(mask(&checked(value, true)).layers().len(), count);
    }
}

#[test]
fn malformed_groups_drop_only_local_declaration_with_exact_recovery_coordinates() {
    for value in INVALID {
        let source = format!("/*😀*/color:red; mask:{value}!important; width:1px");
        let report = parse_style_attribute(&source);
        let [before, after] = report.syntax().as_slice() else {
            panic!("only siblings survive: {value}")
        };
        assert_eq!(before.known().unwrap().property(), CssKnownProperty::Color);
        assert_eq!(after.known().unwrap().property(), CssKnownProperty::Width);
        let [diagnostic] = report.diagnostics() else {
            panic!("one diagnostic: {value}")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let ErrorKind::InvalidPropertyValue(detail) = diagnostic.error().kind() else {
            panic!("typed property error")
        };
        assert_eq!(detail.property(), CssKnownProperty::Mask);
        let start = source.find("mask:").unwrap();
        let end = start + source[start..].find(';').unwrap() + 1;
        position(diagnostic.span().start(), &source, start);
        position(diagnostic.span().end(), &source, end);
    }
}

#[test]
fn validator_rejects_duplicate_groups_invalid_sizes_and_empty_layers() {
    for value in INVALID {
        assert!(
            validate_style_attribute(&format!("mask:{value}; color:blue")).is_err(),
            "{value}"
        );
    }
}

fn checked_rejects(value: &str, grammar: bool) {
    let components = parse_component_values(value).unwrap();
    let result = if grammar {
        parse_property_value_for_grammar(
            CssKnownProperty::Mask.grammar(),
            components,
            CssImportance::Normal,
        )
    } else {
        parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Mask),
            components,
            CssImportance::Normal,
        )
    };
    let error = result.expect_err("malformed mask layer");
    let CssPropertyValueErrorKind::Grammar(ErrorKind::InvalidPropertyValue(detail)) = error.kind()
    else {
        panic!("typed grammar error: {error:?}")
    };
    assert_eq!(detail.property(), CssKnownProperty::Mask);
}

#[test]
fn checked_property_rejects_duplicate_groups_invalid_sizes_and_empty_layers() {
    for value in INVALID {
        checked_rejects(value, false);
    }
}

#[test]
fn checked_grammar_rejects_duplicate_groups_invalid_sizes_and_empty_layers() {
    for value in INVALID {
        checked_rejects(value, true);
    }
}
