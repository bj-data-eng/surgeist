#![forbid(unsafe_code)]
//! Display3 2026-06-05 §3 defines order:<integer>, initial 0, noninherited.
//! Values4 2024-03-12 §§5/5.2 define integer spelling; finite implementation
//! ranges are permitted. Exact ordinary transport and truthful I01 projection
//! are adopted Surgeist product contracts, not an arbitrary-precision CSS claim.
use surgeist_css::*;

const OUTSIDE_I32: &[&str] = &[
    "2147483648",
    "-2147483649",
    "+0002147483648",
    "-0002147483649",
    "1234567890123456789012345678901234567890",
    "-1234567890123456789012345678901234567890",
];

fn parsed(property: CssKnownProperty, text: &str) -> CssDeclaration {
    let name = match property {
        CssKnownProperty::Order => "order",
        CssKnownProperty::ZIndex => "z-index",
        _ => panic!("selected integer property"),
    };
    let report = parse_style_attribute(&format!("{name}:{text}!important"));
    assert!(
        report.is_clean(),
        "{name}:{text}: {:?}",
        report.diagnostics()
    );
    let [declaration] = report.syntax().as_slice() else {
        panic!("one ordinary declaration")
    };
    declaration.clone()
}

fn frozen_integer(declaration: &CssDeclaration) -> Option<i32> {
    match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Order(value) => match value.i01_subset() {
            Some(CssOrder::Integer(value)) => Some(*value),
            None => None,
            _ => panic!("unexpected future I01 order"),
        },
        CssKnownPropertyValueRef::ZIndex(value) => match value.i01_subset() {
            Some(CssZIndex::Integer(value)) => Some(*value),
            None => None,
            _ => panic!("ordinary integer, not auto"),
        },
        _ => panic!("selected integer wrapper"),
    }
}

fn assert_authored_integer(declaration: &CssDeclaration, text: &str) {
    let raw = match declaration.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Order(value) => value.as_css(),
        CssKnownPropertyValueRef::ZIndex(value) => value.as_css(),
        _ => panic!("selected integer wrapper"),
    };
    assert_eq!(raw, text);
    assert_eq!(
        declaration.value_components().serialize().unwrap().as_css(),
        text
    );
    let [component] = declaration.value_components().items() else {
        panic!("one integer token")
    };
    let CssComponentValueRef::Token(CssValueTokenRef::Number(number)) = component.view() else {
        panic!("number token")
    };
    assert_eq!(number.kind(), CssNumericTokenKind::Integer);
    assert_eq!(number.representation(), text);
}

fn assert_parsed_outside_i32(property: CssKnownProperty) {
    for text in OUTSIDE_I32 {
        let declaration = parsed(property, text);
        assert_authored_integer(&declaration, text);
        assert_eq!(declaration.importance(), CssImportance::Important);
        let CssValueOrigin::Parsed(origin) = declaration.value_components().items()[0].origin()
        else {
            panic!("original parsed token")
        };
        assert!(
            origin
                .source()
                .same_snapshot(declaration.parsed_value().unwrap().source())
        );
        // Independent contract: none of these mathematical integers fits i32.
        // A saturated boundary is not an exact frozen representation.
        assert_eq!(frozen_integer(&declaration), None, "{property:?}: {text}");
    }
}

fn assert_checked_outside_i32(property: CssKnownProperty) {
    for text in OUTSIDE_I32 {
        let programmatic =
            CssComponentValues::try_new(vec![CssComponentValue::try_number(text).unwrap()])
                .unwrap();
        assert!(matches!(
            programmatic.items()[0].origin(),
            CssValueOrigin::Programmatic
        ));
        let retained_parsed = parse_component_values(text).unwrap();
        assert!(matches!(
            retained_parsed.items()[0].origin(),
            CssValueOrigin::Parsed(_)
        ));
        for components in [programmatic, retained_parsed] {
            let declaration = parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                CssImportance::Normal,
            )
            .expect("checked finite integer admission");
            assert_authored_integer(&declaration, text);
            assert_eq!(declaration.value_components(), &components);
            assert_eq!(
                declaration.value_components().items()[0].origin(),
                components.items()[0].origin()
            );
            assert_eq!(declaration.importance(), CssImportance::Normal);
            assert!(declaration.position().is_none());
            assert!(declaration.parsed_name().is_none());
            assert!(declaration.parsed_value().is_none());
            assert_eq!(frozen_integer(&declaration), None, "{property:?}: {text}");
        }
    }
}

#[test]
fn parsed_order_refuses_inexact_i01_boundary_substitution() {
    assert_parsed_outside_i32(CssKnownProperty::Order);
}

#[test]
fn parsed_z_index_refuses_inexact_i01_boundary_substitution() {
    assert_parsed_outside_i32(CssKnownProperty::ZIndex);
}

#[test]
fn checked_order_refuses_inexact_i01_boundary_substitution() {
    assert_checked_outside_i32(CssKnownProperty::Order);
}

#[test]
fn checked_z_index_refuses_inexact_i01_boundary_substitution() {
    assert_checked_outside_i32(CssKnownProperty::ZIndex);
}

#[test]
fn exact_i32_boundaries_signs_and_leading_zeros_keep_frozen_views() {
    for property in [CssKnownProperty::Order, CssKnownProperty::ZIndex] {
        for (text, expected) in [
            ("2147483647", i32::MAX),
            ("-2147483648", i32::MIN),
            ("+0002147483647", i32::MAX),
            ("-0002147483648", i32::MIN),
            ("0", 0),
            ("+0", 0),
            ("-0000", 0),
            ("+0000000000000000000000000002", 2),
            ("-0000000000000000000000000002", -2),
        ] {
            let declaration = parsed(property, text);
            assert_authored_integer(&declaration, text);
            assert_eq!(frozen_integer(&declaration), Some(expected));
            let components =
                CssComponentValues::try_new(vec![CssComponentValue::try_number(text).unwrap()])
                    .unwrap();
            let checked = parse_property_value(
                CssPropertyNameRef::Known(property),
                components.clone(),
                CssImportance::Normal,
            )
            .unwrap();
            assert_authored_integer(&checked, text);
            assert_eq!(checked.value_components(), &components);
            assert_eq!(frozen_integer(&checked), Some(expected));
        }
    }
}

#[test]
fn z_index_auto_and_integer_math_keep_existing_typed_boundaries() {
    for text in ["auto", "AUTO", r"a\75 to"] {
        let parsed = parsed(CssKnownProperty::ZIndex, text);
        let components =
            CssComponentValues::try_new(vec![CssComponentValue::try_token(text).unwrap()]).unwrap();
        let checked = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::ZIndex),
            components.clone(),
            CssImportance::Normal,
        )
        .unwrap();
        assert_eq!(checked.value_components(), &components);
        for declaration in [parsed, checked] {
            let CssKnownPropertyValueRef::ZIndex(value) =
                declaration.known().unwrap().property_value().unwrap()
            else {
                panic!("z-index wrapper")
            };
            assert_eq!(value.as_css(), text);
            assert_eq!(value.value(), &CssZIndexValue::Auto);
            assert_eq!(value.i01_subset(), Some(&CssZIndex::Auto));
        }
    }
    for property in [CssKnownProperty::Order, CssKnownProperty::ZIndex] {
        for text in ["calc(1.5)", "calc(-1.5)", "calc(2px / 1px)"] {
            let declaration = parsed(property, text);
            let calculation = match declaration.known().unwrap().property_value().unwrap() {
                CssKnownPropertyValueRef::Order(value) => match value.value() {
                    CssIntegerValue::Calculation(value) => value,
                    _ => panic!("unrounded integer calculation"),
                },
                CssKnownPropertyValueRef::ZIndex(value) => match value.value() {
                    CssZIndexValue::Integer(CssIntegerValue::Calculation(value)) => value,
                    _ => panic!("unrounded integer calculation"),
                },
                _ => panic!("integer property"),
            };
            assert_eq!(calculation.result_type(), CssCalculationType::Number);
            assert_eq!(frozen_integer(&declaration), None);
        }
    }
}

#[test]
fn order_has_noninherited_longhand_metadata_and_ordinary_initial_shape() {
    let metadata = CssKnownProperty::Order
        .metadata()
        .expect("Order intrinsic metadata");
    let CssPropertyKindRef::Longhand(longhand) = metadata.kind() else {
        panic!("Order is one longhand")
    };
    assert!(!longhand.inherited_by_default());
    let initial = longhand.initial_value();
    assert_eq!(initial.property().known_property(), CssKnownProperty::Order);
    assert!(matches!(initial.view(), CssInitialValueRef::Value(_)));
    // Exact initial Literal(0) payload needs the functional new longhand branch.
    // An ordinary initial shape alone does not prove its numeric value.
}

#[test]
fn already_admitted_order_literals_and_math_expand_once_with_source_identity() {
    for text in ["0", "-2", "calc(1.5)", "calc(-1.5)"] {
        let declaration = parsed(CssKnownProperty::Order, text);
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&declaration).expect("Order intrinsic expansion")
        else {
            panic!("ordinary longhand contributions")
        };
        let [value] = values.items() else {
            panic!("exactly one Order contribution")
        };
        assert_eq!(value.property(), CssKnownProperty::Order);
        assert!(matches!(
            value.value(),
            CssContributionValueRef::Ordinary(_)
        ));
        assert_eq!(
            value.ordinary_value().unwrap().property().known_property(),
            CssKnownProperty::Order
        );
        assert!(value.source().same_occurrence(&declaration));
        assert_eq!(value.source().importance(), CssImportance::Important);
        assert!(value.replacement_components().is_none());
    }
}

#[test]
fn noninteger_ordinary_tokens_and_wrong_numeric_roots_remain_invalid() {
    for (property, name) in [
        (CssKnownProperty::Order, "order"),
        (CssKnownProperty::ZIndex, "z-index"),
    ] {
        for text in [
            "1.0",
            "1e2",
            "1%",
            "1px",
            "",
            "1 2",
            "1,2",
            "calc(1px)",
            "calc(1%)",
            "inherit 1",
        ] {
            let report = parse_style_attribute(&format!("color:red;{name}:{text};color:blue"));
            let [diagnostic] = report.diagnostics() else {
                panic!("one invalid integer declaration: {name}:{text}")
            };
            assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
            assert_eq!(report.syntax().len(), 2);
            let components = parse_component_values(text).unwrap();
            assert!(
                parse_property_value(
                    CssPropertyNameRef::Known(property),
                    components,
                    CssImportance::Normal
                )
                .is_err(),
                "{name}:{text}"
            );
        }
    }
    let report = parse_style_attribute("order:auto");
    assert!(!report.is_clean());
    assert!(report.syntax().is_empty());
}
