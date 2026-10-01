#![forbid(unsafe_code)]
//! Functional evidence for the adopted authored shadow serialization contract.
//! The newly added serializers have no executable preimplementation RED.
//! Grammar and intrinsic initials follow Backgrounds 3 CRD 2024-03-11 §6.1
//! and Filter Effects 1 WD 2018-12-18 §6.1; canonical ordering and cumulative
//! resource charges are Surgeist's specified-value contract, not computed CSS.

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};

fn declaration(css: &str) -> CssDeclaration {
    let report = parse_style_attribute(css);
    assert!(report.is_clean(), "{css}: {:?}", report.diagnostics());
    let [value] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    value.clone()
}

#[test]
fn resumed_lengths_report_the_responsible_token_and_utf16_column() {
    for declaration in [
        "box-shadow:1px 2px red 3px",
        "filter:drop-shadow(1px 2px red 3px)",
        "backdrop-filter:drop-shadow(1px 2px red 3px)",
    ] {
        let css = format!("/* 🦀 */ {declaration}; color:blue");
        let report = parse_style_attribute(&css);
        let [diagnostic] = report.diagnostics() else {
            panic!("one diagnostic")
        };
        assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
        let position = diagnostic.error().position();
        let responsible = css.find("3px").unwrap();
        assert_eq!(position.byte_offset().value(), responsible, "{css}");
        assert_eq!(position.line().value(), 0);
        assert_eq!(
            position.column().value() as usize,
            css[..responsible].encode_utf16().count()
        );
        let [sibling] = report.syntax().as_slice() else {
            panic!("valid sibling")
        };
        assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
        assert_eq!(
            validate_style_attribute(&css).unwrap_err().diagnostics(),
            report.diagnostics()
        );
    }
}

fn box_value(css: &str) -> CssBoxShadow {
    let source = declaration(&format!("box-shadow:{css}"));
    let CssKnownPropertyValueRef::BoxShadow(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("box shadow")
    };
    value.value().clone()
}

fn shadow(css: &str) -> CssShadow {
    let CssBoxShadow::Shadows(list) = box_value(css) else {
        panic!("shadow list")
    };
    let [value] = list.shadows() else {
        panic!("one shadow")
    };
    value.clone()
}

fn drop_value(css: &str) -> CssDropShadow {
    let source = declaration(&format!("filter:drop-shadow({css})"));
    let CssKnownPropertyValueRef::Filter(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("filter")
    };
    let CssFilter::Functions(functions) = value.value() else {
        panic!("function list")
    };
    let [CssFilterFunction::DropShadow(value)] = functions.functions() else {
        panic!("drop shadow")
    };
    value.clone()
}

fn length(css: &str) -> CssSpecifiedLength {
    CssSpecifiedLength::try_from_component(CssComponentValue::try_token(css).unwrap()).unwrap()
}

fn nonnegative(css: &str) -> CssSpecifiedNonNegativeLength {
    CssSpecifiedNonNegativeLength::try_from_component(CssComponentValue::try_token(css).unwrap())
        .unwrap()
}

fn assert_budget(
    expected: &str,
    input: usize,
    projection: usize,
    serialize: impl Fn(L) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    assert_eq!(
        serialize(L::new(input, projection, expected.len())).unwrap(),
        expected
    );
    assert_eq!(
        serialize(L::new(usize::MAX, usize::MAX, usize::MAX)).unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            L::new(input - 1, projection, expected.len()),
            K::InputNodeLimit,
        ),
        (
            L::new(input, projection - 1, expected.len()),
            K::ProjectionNodeLimit,
        ),
        (L::new(input, projection, expected.len() - 1), K::ByteLimit),
        (L::new(0, projection, expected.len()), K::InputNodeLimit),
        (L::new(input, 0, expected.len()), K::ProjectionNodeLimit),
        (L::new(input, projection, 0), K::ByteLimit),
    ] {
        assert_eq!(serialize(limits).unwrap_err().kind(), kind);
    }
}

#[test]
fn construction_preserves_omissions_and_rejects_spread_without_blur_and_empty_lists() {
    let value = CssShadow::try_new(false, length("1px"), length("-2px"), None, None, None).unwrap();
    assert!(!value.inset());
    assert!(value.blur_radius().is_none());
    assert!(value.spread_radius().is_none());
    assert!(value.color().is_none());
    assert_eq!(value.offset_x().origin(), &CssValueOrigin::Programmatic);
    assert_eq!(value.serialize_specified().unwrap(), "1px -2px");
    assert!(
        CssShadow::try_new(
            false,
            length("1px"),
            length("2px"),
            None,
            Some(length("3px")),
            None
        )
        .is_none()
    );
    assert!(CssBoxShadowList::try_new(vec![]).is_none());
    for invalid in ["-1px", "-1e-999px", "1%", "auto"] {
        assert!(
            CssSpecifiedNonNegativeLength::try_from_component(
                CssComponentValue::try_token(invalid).unwrap()
            )
            .is_err()
        );
    }
    for invalid in ["1%", "auto", "1"] {
        assert!(
            CssSpecifiedLength::try_from_component(CssComponentValue::try_token(invalid).unwrap())
                .is_err()
        );
    }
    let drop = CssDropShadow::new(length("1px"), length("-2px"), None, None);
    assert!(drop.standard_deviation().is_none());
    assert_eq!(drop.serialize_specified().unwrap(), "drop-shadow(1px -2px)");
}

#[test]
fn explicit_zero_currentcolor_and_programmatic_components_remain_explicit() {
    let value = CssShadow::try_new(
        true,
        length("0px"),
        length("-0px"),
        Some(nonnegative("0px")),
        Some(length("0px")),
        Some(CssColor::current_color()),
    )
    .unwrap();
    assert_eq!(
        value.serialize_specified().unwrap(),
        "currentcolor 0px 0px 0px 0px inset"
    );
    assert_eq!(
        value.blur_radius().unwrap().origin(),
        &CssValueOrigin::Programmatic
    );
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, .. }) =
        value.offset_y().literal_component().unwrap().view()
    else {
        panic!("literal")
    };
    assert_eq!(number.representation(), "-0");
    let drop = CssDropShadow::new(
        length("0px"),
        length("0px"),
        Some(nonnegative("0px")),
        Some(CssColor::current_color()),
    );
    assert!(drop.standard_deviation().is_some());
    assert_eq!(
        drop.serialize_specified().unwrap(),
        "drop-shadow(currentcolor 0px 0px 0px)"
    );
    assert_ne!(
        drop,
        CssDropShadow::new(
            length("0px"),
            length("0px"),
            None,
            Some(CssColor::current_color())
        )
    );
}

#[test]
fn box_serialization_has_independent_canonical_text_and_stable_reparsing() {
    for (authored, expected) in [
        ("none", "none"),
        ("inset 1px -2px RED", "red 1px -2px inset"),
        ("1px 2px 0 -4px currentcolor", "currentcolor 1px 2px 0 -4px"),
        (
            "1px 2px, inset blue 3px 4px 0",
            "1px 2px, blue 3px 4px 0 inset",
        ),
        ("calc(1px + 2em) -2px", "calc(2em + 1px) -2px"),
    ] {
        let value = box_value(authored);
        assert_eq!(value.serialize_specified().unwrap(), expected);
        let reparsed = box_value(expected);
        assert_eq!(reparsed.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn drop_serialization_emits_full_function_and_preserves_optional_deviation() {
    for (authored, expected) in [
        ("1px 2px", "drop-shadow(1px 2px)"),
        ("-1px 2px 0 red", "drop-shadow(red -1px 2px 0)"),
        (
            "calc(1px + 2em) -2px 3px",
            "drop-shadow(calc(2em + 1px) -2px 3px)",
        ),
    ] {
        let value = drop_value(authored);
        assert_eq!(value.serialize_specified().unwrap(), expected);
        let inner = expected
            .strip_prefix("drop-shadow(")
            .unwrap()
            .strip_suffix(')')
            .unwrap();
        let reparsed = drop_value(inner);
        assert_eq!(reparsed.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn symbolic_colors_and_math_use_existing_child_providers_without_resolution() {
    for (authored, expected) in [
        (
            "color-mix(in srgb, red, blue) 1px 2px",
            "color-mix(in srgb, red, blue) 1px 2px",
        ),
        ("rgb(from red r g b) 1px 2px", "rgb(from red r g b) 1px 2px"),
        ("1px 2px calc(1px + 2em)", "1px 2px calc(2em + 1px)"),
        ("1px 2px 0 calc(1px + 2em)", "1px 2px 0 calc(2em + 1px)"),
    ] {
        assert_eq!(shadow(authored).serialize_specified().unwrap(), expected);
    }
    let value = drop_value("rgb(from red r g b) 1px 2px calc(1px + 2em)");
    assert_eq!(
        value.serialize_specified().unwrap(),
        "drop-shadow(rgb(from red r g b) 1px 2px calc(2em + 1px))"
    );
}

#[test]
fn serialization_failure_and_success_leave_exact_parsed_origins_unchanged() {
    let css = "box-shadow:inset 1e999px -1e-999px 0 -2px red!important";
    let source = declaration(css);
    let CssKnownPropertyValueRef::BoxShadow(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("box")
    };
    let CssBoxShadow::Shadows(list) = wrapper.value() else {
        panic!("list")
    };
    let value = &list.shadows()[0];
    let before = value.clone();
    let origin = value.offset_x().origin().clone();
    let CssValueOrigin::Parsed(parsed) = &origin else {
        panic!("parsed origin")
    };
    assert_eq!(parsed.source().as_str(), css);
    assert_eq!(
        &css[parsed.span().start().byte_offset().value()
            ..parsed.span().end().byte_offset().value()],
        "1e999px"
    );
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(100, 100, 1))
            .unwrap_err()
            .kind(),
        K::ByteLimit
    );
    assert_eq!(value, &before);
    assert_eq!(value.offset_x().origin(), &origin);
    let huge = format!("1{}px", "0".repeat(999));
    assert_eq!(
        value.serialize_specified().unwrap(),
        format!("red {huge} 0px 0 -2px inset")
    );
    assert_eq!(value.offset_x().origin(), &origin);
    let drop = drop_value("1e999px -1e-999px 0 red");
    let before = drop.clone();
    let origin = drop.standard_deviation().unwrap().origin().clone();
    assert_eq!(
        drop.serialize_specified_with_limits(L::new(0, 100, 100))
            .unwrap_err()
            .kind(),
        K::InputNodeLimit
    );
    assert_eq!(drop, before);
    assert_eq!(drop.standard_deviation().unwrap().origin(), &origin);
}

#[test]
fn literal_resource_limits_count_aggregate_present_children_and_inset() {
    // Each aggregate, literal, named color and inset keyword costs one input
    // and one projection node. Separators cost bytes only.
    for (css, expected, nodes) in [
        ("1px 2px", "1px 2px", 3),
        ("inset 1px 2px", "1px 2px inset", 4),
        (
            "red 1px 2px 3px -4px inset",
            "red 1px 2px 3px -4px inset",
            7,
        ),
        ("0 0 0 0 currentcolor", "currentcolor 0 0 0 0", 6),
    ] {
        let value = shadow(css);
        let before = value.clone();
        assert_budget(expected, nodes, nodes, |limits| {
            value.serialize_specified_with_limits(limits)
        });
        assert_eq!(value, before);
    }
    assert_budget("none", 1, 1, |limits| {
        CssBoxShadow::None.serialize_specified_with_limits(limits)
    });
    for (css, expected, nodes) in [
        ("1px 2px", "drop-shadow(1px 2px)", 3),
        ("red 1px 2px 3px", "drop-shadow(red 1px 2px 3px)", 5),
    ] {
        let value = drop_value(css);
        assert_budget(expected, nodes, nodes, |limits| {
            value.serialize_specified_with_limits(limits)
        });
    }
}

#[test]
fn list_and_box_enum_share_one_budget_and_retain_list_order() {
    let list = CssBoxShadowList::try_new(vec![shadow("1px 2px"), shadow("3px 4px")]).unwrap();
    // One list plus two aggregates plus four literal children: seven nodes.
    assert_budget("1px 2px, 3px 4px", 7, 7, |limits| {
        list.serialize_specified_with_limits(limits)
    });
    assert_eq!(list.serialize_specified().unwrap(), "1px 2px, 3px 4px");
    let value = CssBoxShadow::Shadows(list.clone());
    assert_budget("1px 2px, 3px 4px", 7, 7, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    assert_eq!(value, CssBoxShadow::Shadows(list));
}

#[test]
fn sibling_math_arenas_never_refund_cumulative_projection_work() {
    // Each calc sum owns four input nodes (function, sum, two literals) and
    // five projected nodes (two literals, two terms, canonical sum).
    let one = shadow("calc(1px + 2em) 2px");
    assert_budget("calc(2em + 1px) 2px", 6, 7, |limits| {
        one.serialize_specified_with_limits(limits)
    });
    let two = shadow("calc(1px + 2em) calc(1px + 2em)");
    assert_budget("calc(2em + 1px) calc(2em + 1px)", 9, 11, |limits| {
        two.serialize_specified_with_limits(limits)
    });
    let list = CssBoxShadowList::try_new(vec![one.clone(), one]).unwrap();
    assert_budget(
        "calc(2em + 1px) 2px, calc(2em + 1px) 2px",
        13,
        15,
        |limits| list.serialize_specified_with_limits(limits),
    );
    let drop = drop_value("calc(1px + 2em) calc(1px + 2em)");
    assert_budget(
        "drop-shadow(calc(2em + 1px) calc(2em + 1px))",
        9,
        11,
        |limits| drop.serialize_specified_with_limits(limits),
    );
}

#[test]
fn checked_programmatic_math_and_extreme_decimals_keep_exact_meaning() {
    let children = CssComponentValues::try_new(
        ["1px", " ", "+", " ", "2em"]
            .into_iter()
            .map(|token| CssComponentValue::try_token(token).unwrap())
            .collect(),
    )
    .unwrap();
    let calc = CssLengthCalculation::try_from_components(
        CssComponentValues::try_new(vec![
            CssComponentValue::try_function("calc", children).unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
    let offset = CssSpecifiedLength::try_from_calculation(calc.clone()).unwrap();
    let blur = CssSpecifiedNonNegativeLength::try_from_calculation(calc).unwrap();
    let shadow = CssShadow::try_new(
        false,
        offset.clone(),
        length("2px"),
        Some(blur.clone()),
        None,
        None,
    )
    .unwrap();
    assert_eq!(shadow.offset_x().origin(), &CssValueOrigin::Programmatic);
    assert_eq!(
        shadow.serialize_specified().unwrap(),
        "calc(2em + 1px) 2px calc(2em + 1px)"
    );
    let drop = CssDropShadow::new(offset, length("2px"), Some(blur), None);
    assert_eq!(
        drop.serialize_specified().unwrap(),
        "drop-shadow(calc(2em + 1px) 2px calc(2em + 1px))"
    );
    // Huge integer expansion is exact; six-place text rounds tiny offsets to zero.
    let huge = format!("1{}px", "0".repeat(400));
    let expected = format!("{huge} 0px");
    let value = CssShadow::try_new(
        false,
        length("1e400px"),
        length("-1e-400px"),
        None,
        None,
        None,
    )
    .unwrap();
    assert_budget(&expected, 3, 3, |limits| {
        value.serialize_specified_with_limits(limits)
    });
    let drop = CssDropShadow::new(length("1e400px"), length("-1e-400px"), None, None);
    assert_budget(&format!("drop-shadow({expected})"), 3, 3, |limits| {
        drop.serialize_specified_with_limits(limits)
    });
    assert_eq!(
        box_value(&expected).serialize_specified().unwrap(),
        expected
    );
}

#[test]
fn typed_initial_and_ordinary_borrowed_variant_have_box_shadow_values() {
    let CssPropertyKindRef::Longhand(metadata) =
        CssKnownProperty::BoxShadow.metadata().unwrap().kind()
    else {
        panic!("longhand")
    };
    let initial = metadata.initial_value();
    let CssInitialValueRef::Value(value) = initial.view() else {
        panic!("ordinary initial")
    };
    assert!(matches!(
        value.view(),
        CssLonghandValueRef::BoxShadow(CssBoxShadow::None)
    ));
    for css in ["box-shadow:none", "box-shadow:red 1px 2px!important"] {
        let source = declaration(css);
        let CssKnownPropertyValueRef::BoxShadow(wrapper) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("box")
        };
        let CssExpansion::Contributions(CssContributions::Longhands(values)) =
            expand_declaration(&source).unwrap()
        else {
            panic!("ordinary")
        };
        let [value] = values.items() else {
            panic!("one contribution")
        };
        let CssLonghandValueRef::BoxShadow(ordinary) = value.ordinary_value().unwrap().view()
        else {
            panic!("box contribution")
        };
        assert_eq!(ordinary, wrapper.value());
    }
}

#[test]
fn mixed_normalization_preserves_order_provenance_and_one_unit_per_shadow() {
    let css = ".a{box-shadow:red 1px 2px!important;color:blue;box-shadow:inherit}.b{box-shadow:var(--shadow)}";
    let report = parse_sheet(css);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let values: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(values.len(), 4);
    for (index, value) in values.iter().enumerate() {
        assert_eq!(value.order(), index);
        assert_eq!(
            value.source().known().unwrap().property(),
            if index == 1 {
                CssKnownProperty::Color
            } else {
                CssKnownProperty::BoxShadow
            }
        );
        match value.expansion() {
            CssExpansion::Contributions(CssContributions::Longhands(items)) => {
                let [item] = items.items() else {
                    panic!("one terminal")
                };
                assert!(item.source().same_occurrence(value.source()));
                if index == 2 {
                    assert_eq!(
                        item.value(),
                        CssContributionValueRef::Global(CssGlobalKeyword::Inherit)
                    );
                }
                if index == 0 {
                    let CssLonghandValueRef::BoxShadow(shadow) =
                        item.ordinary_value().unwrap().view()
                    else {
                        panic!("box")
                    };
                    assert_eq!(shadow.serialize_specified().unwrap(), "red 1px 2px");
                    assert_eq!(item.source().importance(), CssImportance::Important);
                }
            }
            CssExpansion::Pending(handle) => {
                assert_eq!(index, 3);
                assert!(handle.source().same_occurrence(value.source()));
            }
            _ => panic!("owned normalized declaration"),
        }
    }
    for (limit, order) in [(2, 2), (3, 3)] {
        let limits = CssNormalizationLimits::try_new(256, usize::MAX, usize::MAX, limit).unwrap();
        let error = normalize_sheet_with_limits(report.syntax(), limits).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNormalizationErrorKind::LimitExceeded {
                resource: CssNormalizationResource::Contributions,
                limit
            }
        );
        assert_eq!(error.declaration_order(), Some(order));
        assert!(
            error
                .declaration()
                .unwrap()
                .same_occurrence(values[order].source())
        );
    }
}
