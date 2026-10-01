#![forbid(unsafe_code)]
//! Exact ordinary angle literals: Values 4 WD 2024-03-12 #angles,
//! #zero-value and #numeric-data-types. Consumer policies come from Transforms
//! 1 CR 2019-02-14, Transforms 2 WD 2021-11-09, FE1 WD 2018-12-18 and
//! Images 3 CRD 2023-12-18. Specified decimal expectations use exact coefficient
//! shifts. Calculation simplification is preserved; exact math evaluation and
//! its binary64 range/precision gap are outside this literal foundation.
//! Transform/orientation have no public specified serializer at this source
//! basis: existing aggregate identity expresses their literal fidelity instead.

use surgeist_css::*;

fn declaration(property: CssKnownProperty, value: &str) -> CssDeclaration {
    let source = format!("{}:{value}!important", property.canonical_name());
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    assert_eq!(
        validate_style_attribute(&source),
        Ok(report.syntax().clone())
    );
    let [declaration] = report.syntax().as_slice() else {
        panic!("one declaration")
    };
    declaration.clone()
}

fn filter(source: &CssDeclaration) -> &CssFilter {
    match source.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Filter(value) => value.value(),
        CssKnownPropertyValueRef::BackdropFilter(value) => value.value(),
        _ => panic!("filter"),
    }
}
fn transform(source: &CssDeclaration) -> &CssTransform {
    let CssKnownPropertyValueRef::Transform(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("transform")
    };
    value.value()
}
fn orientation(source: &CssDeclaration) -> &CssImageOrientation {
    let CssKnownPropertyValueRef::ImageOrientation(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("orientation")
    };
    value.orientation()
}
fn gradient(source: &CssDeclaration) -> &CssGradient {
    let CssKnownPropertyValueRef::BackgroundImage(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("background image")
    };
    let [CssImageValue::Gradient(gradient)] = value.images().images() else {
        panic!("one gradient")
    };
    gradient
}
fn direction(source: &CssDeclaration) -> &CssLinearGradientDirection {
    let CssGradient::Linear(value) = gradient(source) else {
        panic!("linear gradient")
    };
    value.direction().unwrap()
}
fn checked(property: CssKnownProperty, value: &str) -> CssDeclaration {
    let components = parse_component_values(value).unwrap();
    let source = parse_property_value(
        CssPropertyNameRef::Known(property),
        components.clone(),
        CssImportance::Important,
    )
    .unwrap();
    assert_eq!(source.value_components(), &components);
    source
}

fn permissive_consumers(angle: &str) -> Vec<(CssKnownProperty, String)> {
    use CssKnownProperty as P;
    let mut values: Vec<_> = [
        "rotate", "rotateX", "rotateY", "rotateZ", "skewX", "skewY", "skew",
    ]
    .into_iter()
    .map(|function| (P::Transform, format!("{function}({angle})")))
    .collect();
    values.extend([
        (P::Transform, format!("rotate3d(1, 0, 0, {angle})")),
        (P::Transform, format!("skew({angle}, {angle})")),
        (P::Filter, format!("hue-rotate({angle})")),
        (P::BackdropFilter, format!("hue-rotate({angle})")),
        (
            P::BackgroundImage,
            format!("linear-gradient({angle}, red, blue)"),
        ),
        (
            P::BackgroundImage,
            format!("repeating-linear-gradient({angle}, red, blue)"),
        ),
    ]);
    values
}

#[test]
fn precise_filter_and_backdrop_literals_serialize_exactly_in_all_four_units() {
    for property in [CssKnownProperty::Filter, CssKnownProperty::BackdropFilter] {
        for unit in ["deg", "grad", "rad", "turn"] {
            for (number, expected) in [
                ("0.10000000000000000001", "0.10000000000000000001"),
                ("-1.234567890123456789", "-1.234567890123456789"),
                ("+1.2300e2", "123"),
            ] {
                let authored = format!("hue-rotate({number}{unit})");
                let expected = format!("hue-rotate({expected}{unit})");
                let source = declaration(property, &authored);
                assert_eq!(filter(&source).serialize_specified().unwrap(), expected);
                assert_eq!(
                    filter(&checked(property, &authored))
                        .serialize_specified()
                        .unwrap(),
                    expected
                );
            }
        }
    }
}

#[test]
fn huge_and_tiny_literal_dimensions_are_admitted_without_machine_float_limits() {
    for (number, decimal) in [
        ("1e40", format!("1{}", "0".repeat(40))),
        ("-1e-40", format!("-0.{}1", "0".repeat(39))),
        ("1e999", format!("1{}", "0".repeat(999))),
        ("-1e-999", format!("-0.{}1", "0".repeat(998))),
    ] {
        for unit in ["deg", "grad", "rad", "turn"] {
            let angle = format!("{number}{unit}");
            for (property, value) in permissive_consumers(&angle) {
                declaration(property, &value);
                checked(property, &value);
            }
            declaration(CssKnownProperty::ImageOrientation, &format!("{angle} flip"));
            checked(CssKnownProperty::ImageOrientation, &angle);
            let filter_value =
                declaration(CssKnownProperty::Filter, &format!("hue-rotate({angle})"));
            assert_eq!(
                filter(&filter_value).serialize_specified().unwrap(),
                format!("hue-rotate({decimal}{unit})")
            );
        }
    }
}

#[test]
fn transforms_distinguish_adjacent_exact_literal_coefficients_that_round_to_one_float() {
    for function in [
        "rotate", "rotateX", "rotateY", "rotateZ", "skewX", "skewY", "skew",
    ] {
        let first = checked(
            CssKnownProperty::Transform,
            &format!("{function}(0.10000000000000000001deg)"),
        );
        let second = checked(
            CssKnownProperty::Transform,
            &format!("{function}(0.10000000000000000002deg)"),
        );
        assert_ne!(
            transform(&first),
            transform(&second),
            "{function}: distinct coefficients"
        );
    }
}

#[test]
fn image_orientation_distinguishes_adjacent_exact_literal_coefficients() {
    for suffix in ["", " flip"] {
        let first = checked(
            CssKnownProperty::ImageOrientation,
            &format!("0.10000000000000000001deg{suffix}"),
        );
        let second = checked(
            CssKnownProperty::ImageOrientation,
            &format!("0.10000000000000000002deg{suffix}"),
        );
        assert_ne!(orientation(&first), orientation(&second));
    }
}

#[test]
fn identical_literal_aggregates_ignore_source_location_changes() {
    let first = declaration(CssKnownProperty::Transform, "rotate(0.1deg)");
    let second_report = parse_style_attribute("/*😀*/ transform:rotate(0.1deg)!important");
    assert!(second_report.is_clean());
    assert_eq!(transform(&first), transform(&second_report.syntax()[0]));
    let first = declaration(CssKnownProperty::ImageOrientation, "0.1deg flip");
    let second_report = parse_style_attribute("/*😀*/ image-orientation:0.1deg flip!important");
    assert!(second_report.is_clean());
    assert_eq!(orientation(&first), orientation(&second_report.syntax()[0]));
}

#[test]
fn permitted_consumers_accept_exact_number_zero_with_signed_and_large_exponent_spelling() {
    for zero in [
        "0",
        "-0",
        "+0.0",
        "-0e999999999999999999",
        "+0e-999999999999999999",
    ] {
        for (property, value) in permissive_consumers(zero) {
            declaration(property, &value);
            checked(property, &value);
        }
        let source = declaration(CssKnownProperty::Filter, &format!("hue-rotate({zero})"));
        assert_eq!(
            filter(&source).serialize_specified().unwrap(),
            "hue-rotate(0)"
        );
        let source = declaration(
            CssKnownProperty::BackgroundImage,
            &format!("linear-gradient({zero}, red, blue)"),
        );
        assert_eq!(
            gradient(&source).serialize_specified().unwrap(),
            "linear-gradient(0, red, blue)"
        );
    }
}

fn reject(property: CssKnownProperty, value: &str) {
    let prefix = "/*😀*/ ";
    let unit = format!("{}:{value};", property.canonical_name());
    let source = format!("{prefix}{unit}color:blue");
    let report = parse_style_attribute(&source);
    let [sibling] = report.syntax().as_slice() else {
        panic!("only color survives {source}: {:?}", report.diagnostics())
    };
    assert_eq!(sibling.known().unwrap().property(), CssKnownProperty::Color);
    let [diagnostic] = report.diagnostics() else {
        panic!("one invalid angle diagnostic: {source}")
    };
    assert_eq!(diagnostic.action(), CssRecoveryAction::DropDeclaration);
    let end = prefix.len() + unit.len();
    assert_eq!(
        diagnostic.span().start().byte_offset().value(),
        prefix.len()
    );
    assert_eq!(diagnostic.span().end().byte_offset().value(), end);
    assert_eq!(
        diagnostic.span().start().column().value() as usize,
        prefix.encode_utf16().count()
    );
    assert_eq!(
        diagnostic.span().end().column().value() as usize,
        source[..end].encode_utf16().count()
    );
    assert_eq!(
        validate_style_attribute(&source).unwrap_err().diagnostics(),
        report.diagnostics()
    );
    assert!(
        parse_property_value(
            CssPropertyNameRef::Known(property),
            parse_component_values(value).unwrap(),
            CssImportance::Normal
        )
        .is_err()
    );
}

#[test]
fn nonzero_underflow_numbers_never_enter_the_angle_zero_exception() {
    for number in [
        "1e-999",
        "-1e-999",
        "0.000000000000000000000000000000000000000000000000001",
    ] {
        for (property, value) in permissive_consumers(number) {
            reject(property, &value);
        }
    }
}

#[test]
fn strict_image_orientation_rejects_number_zero_but_accepts_zero_dimensions() {
    for zero in ["0", "-0", "+0.0", "0e999", "0 flip", "flip -0"] {
        reject(CssKnownProperty::ImageOrientation, zero);
    }
    for dimension in ["0deg", "-0grad", "+0e999rad", "0turn flip"] {
        declaration(CssKnownProperty::ImageOrientation, dimension);
        checked(CssKnownProperty::ImageOrientation, dimension);
    }
}

#[test]
fn wrong_domains_nonzero_numbers_and_number_root_calculations_remain_invalid() {
    for angle in [
        "1",
        "-2",
        "1px",
        "1%",
        "calc(0)",
        "calc(1px)",
        "calc(1deg + 1px)",
    ] {
        for (property, value) in permissive_consumers(angle) {
            reject(property, &value);
        }
        reject(CssKnownProperty::ImageOrientation, angle);
    }
}

#[test]
fn gradient_omits_only_exact_positive_default_directions_and_keeps_neighbors() {
    for function in ["linear-gradient", "repeating-linear-gradient"] {
        for default in [
            "180deg",
            "+1.80e2deg",
            "200grad",
            "+2e2grad",
            ".5turn",
            "+5e-1turn",
        ] {
            let source = declaration(
                CssKnownProperty::BackgroundImage,
                &format!("{function}({default}, red, blue)"),
            );
            assert_eq!(
                gradient(&source).serialize_specified().unwrap(),
                format!("{function}(red, blue)")
            );
        }
        for angle in [
            "179.99999999999999999deg",
            "180.00000000000000001deg",
            "199.99999999999999999grad",
            "200.00000000000000001grad",
            "0.49999999999999999turn",
            "0.50000000000000001turn",
            "540deg",
            "-180deg",
            "180rad",
            "0",
            "calc(180deg)",
        ] {
            let source = declaration(
                CssKnownProperty::BackgroundImage,
                &format!("{function}({angle}, red, blue)"),
            );
            assert_eq!(
                gradient(&source).serialize_specified().unwrap(),
                format!("{function}({angle}, red, blue)")
            );
        }
    }
}

#[test]
fn escaped_case_varied_angle_unit_and_precise_coefficient_keep_original_token_origin() {
    let source = "/*😀*/ filter:hue-rotate(+1.23000000000000000001D\\45 G)!important";
    let report = parse_style_attribute(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let component = &report.syntax()[0].value_components().items()[0];
    let CssComponentValueRef::Function(function) = component.view() else {
        panic!("filter function")
    };
    let angle = &function.values().items()[0];
    let CssComponentValueRef::Token(CssValueTokenRef::Dimension { number, unit }) = angle.view()
    else {
        panic!("dimension")
    };
    assert_eq!(number.representation(), "+1.23000000000000000001");
    assert_eq!(unit, "DEG");
    let CssValueOrigin::Parsed(origin) = angle.origin() else {
        panic!("original token origin")
    };
    let start = source.find('+').unwrap();
    let end = source.find(')').unwrap();
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(origin.span().start().byte_offset().value(), start);
    assert_eq!(origin.span().end().byte_offset().value(), end);
    assert_eq!(
        origin.span().start().column().value() as usize,
        source[..start].encode_utf16().count()
    );
    assert_eq!(
        filter(&report.syntax()[0]).serialize_specified().unwrap(),
        "hue-rotate(1.23000000000000000001deg)"
    );
}

fn located(property: CssKnownProperty, value: &str) -> (CssDeclaration, CssDeclaration) {
    let first = declaration(property, value);
    let report = parse_style_attribute(&format!(
        "/*😀*/ {}:{value}!important",
        property.canonical_name()
    ));
    assert!(report.is_clean());
    (first, report.syntax()[0].clone())
}

#[test]
fn transform_calculation_aggregate_equality_ignores_original_source_coordinates() {
    let (first, second) = located(CssKnownProperty::Transform, "rotate(calc(25deg + 5deg))");
    assert!(
        transform(&first) == transform(&second),
        "transform aggregate must ignore calculation origins"
    );
}
#[test]
fn filter_calculation_aggregate_equality_ignores_original_source_coordinates() {
    for property in [CssKnownProperty::Filter, CssKnownProperty::BackdropFilter] {
        let (first, second) = located(property, "hue-rotate(calc(25deg + 5deg))");
        assert!(
            filter(&first) == filter(&second),
            "{property:?} aggregate must ignore calculation origins"
        );
    }
}
#[test]
fn gradient_direction_calculation_aggregate_equality_ignores_original_source_coordinates() {
    let (first, second) = located(
        CssKnownProperty::BackgroundImage,
        "linear-gradient(calc(25deg + 5deg), red, blue)",
    );
    assert!(
        direction(&first) == direction(&second),
        "gradient direction must ignore calculation origins"
    );
}
#[test]
fn image_orientation_calculation_aggregate_equality_ignores_original_source_coordinates() {
    let (first, second) = located(
        CssKnownProperty::ImageOrientation,
        "calc(25deg + 5deg) flip",
    );
    assert!(
        orientation(&first) == orientation(&second),
        "orientation aggregate must ignore calculation origins"
    );
}

#[test]
fn raw_angle_math_and_color_still_compare_original_provenance() {
    let first = CssAngleCalculation::try_from_components(
        parse_component_values("calc(25deg + 5deg)").unwrap(),
    )
    .unwrap();
    let second = CssAngleCalculation::try_from_components(
        parse_component_values("  calc(25deg + 5deg)").unwrap(),
    )
    .unwrap();
    assert_ne!(first, second);
    assert_ne!(first.origin(), second.origin());
    let (first, second) = located(CssKnownProperty::Color, "hsl(25deg 20% 30%)");
    let CssKnownPropertyValueRef::Color(first) = first.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    let CssKnownPropertyValueRef::Color(second) = second.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    assert_ne!(first.value(), second.value());
}

#[test]
fn existing_angle_calculation_simplification_remains_a_positive_control() {
    let source = declaration(CssKnownProperty::Filter, "hue-rotate(calc(25deg + 5deg))");
    assert_eq!(
        filter(&source).serialize_specified().unwrap(),
        "hue-rotate(calc(30deg))"
    );
    let source = declaration(
        CssKnownProperty::BackgroundImage,
        "linear-gradient(calc(180deg), red, blue)",
    );
    assert_eq!(
        gradient(&source).serialize_specified().unwrap(),
        "linear-gradient(calc(180deg), red, blue)"
    );
}

#[test]
fn ordinary_recovered_angle_math_is_retained_but_checked_construction_is_strict() {
    for (property, incomplete) in [
        (CssKnownProperty::Transform, "rotate(calc(25deg + 5deg"),
        (CssKnownProperty::Filter, "hue-rotate(calc(25deg + 5deg"),
        (
            CssKnownProperty::BackdropFilter,
            "hue-rotate(calc(25deg + 5deg",
        ),
        (CssKnownProperty::ImageOrientation, "calc(25deg + 5deg"),
    ] {
        let source = format!("{}:{incomplete}", property.canonical_name());
        let report = parse_style_attribute(&source);
        assert_eq!(
            report.syntax().len(),
            1,
            "ordinary recovered angle is retained: {source}: {:?}",
            report.diagnostics()
        );
        assert!(!report.is_clean());
        assert!(validate_style_attribute(&source).is_err());
        assert!(
            report
                .diagnostics()
                .iter()
                .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
        );
        let replacement = parse_component_values(incomplete).unwrap();
        let Err(_) = parse_property_value(
            CssPropertyNameRef::Known(property),
            replacement.clone(),
            CssImportance::Normal,
        ) else {
            panic!("checked recovered angle must be rejected: {source}")
        };
    }
}

#[test]
fn pending_angle_reentry_retains_exact_literals_importance_and_replacement_sources() {
    for property in [CssKnownProperty::Filter, CssKnownProperty::BackdropFilter] {
        let source = declaration(property, "var(--angle)");
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending filter")
        };
        assert_eq!(
            pending
                .reenter(parse_component_values("hue-rotate(var(--again))").unwrap())
                .unwrap_err()
                .kind(),
            &CssExpansionErrorKind::ResidualSubstitution
        );
        for invalid in [
            "hue-rotate(1e-999)",
            "hue-rotate(calc(0))",
            "hue-rotate(1px)",
        ] {
            let Err(error) = pending.reenter(parse_component_values(invalid).unwrap()) else {
                panic!("invalid replacement: {invalid}")
            };
            assert!(matches!(
                error.kind(),
                CssExpansionErrorKind::InvalidReplacement(_)
            ));
        }
        let replacement = parse_component_values("hue-rotate(0.10000000000000000001deg)").unwrap();
        for _ in 0..2 {
            let CssContributions::Longhands(items) = pending.reenter(replacement.clone()).unwrap()
            else {
                panic!("ordinary terminal")
            };
            let [item] = items.items() else {
                panic!("one filter terminal")
            };
            assert_eq!(item.property(), property);
            assert!(item.source().same_occurrence(&source));
            assert_eq!(item.source().importance(), CssImportance::Important);
            assert_eq!(item.replacement_components(), Some(&replacement));
            let value = match item.ordinary_value().unwrap().view() {
                CssLonghandValueRef::Filter(value) | CssLonghandValueRef::BackdropFilter(value) => {
                    value
                }
                _ => panic!("filter terminal"),
            };
            assert_eq!(
                value.serialize_specified().unwrap(),
                "hue-rotate(0.10000000000000000001deg)"
            );
        }
    }
}

#[test]
fn normalization_retains_exact_angle_literals_in_surviving_source_order() {
    let source = ".a{filter:hue-rotate(0.10000000000000000001deg)!important;image-orientation:0;backdrop-filter:hue-rotate(1e-40turn);filter:var(--angle)}";
    let report = parse_sheet(source);
    assert_eq!(
        report.diagnostics().len(),
        1,
        "strict image orientation zero: {:?}",
        report.diagnostics()
    );
    let normalized = normalize_sheet(report.syntax()).unwrap();
    let items: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(value) => Some(value),
            _ => None,
        })
        .collect();
    assert_eq!(items.len(), 3);
    for (order, (item, authored)) in items
        .iter()
        .zip([
            "hue-rotate(0.10000000000000000001deg)",
            "hue-rotate(1e-40turn)",
            "var(--angle)",
        ])
        .enumerate()
    {
        assert_eq!(item.order(), order);
        let property = if order == 1 {
            CssKnownProperty::BackdropFilter
        } else {
            CssKnownProperty::Filter
        };
        assert_eq!(
            item.source().position().unwrap().byte_offset().value(),
            source.find(authored).unwrap() - property.canonical_name().len() - 1
        );
        assert_eq!(
            item.source().importance(),
            if order == 0 {
                CssImportance::Important
            } else {
                CssImportance::Normal
            }
        );
        if order == 2 {
            let CssExpansion::Pending(pending) = item.expansion() else {
                panic!("pending terminal")
            };
            assert!(pending.source().same_occurrence(item.source()));
        } else {
            let CssExpansion::Contributions(CssContributions::Longhands(values)) = item.expansion()
            else {
                panic!("ordinary terminal")
            };
            let [value] = values.items() else {
                panic!("one terminal")
            };
            assert!(value.source().same_occurrence(item.source()));
            assert!(value.replacement_components().is_none());
            let filter = match value.ordinary_value().unwrap().view() {
                CssLonghandValueRef::Filter(value) | CssLonghandValueRef::BackdropFilter(value) => {
                    value
                }
                _ => panic!("filter terminal"),
            };
            let expected = if order == 0 {
                "hue-rotate(0.10000000000000000001deg)".to_owned()
            } else {
                format!("hue-rotate(0.{}1turn)", "0".repeat(39))
            };
            assert_eq!(filter.serialize_specified().unwrap(), expected);
        }
    }
}

#[test]
fn exact_literal_provider_failures_are_atomic_and_do_not_mutate_source_or_refund_work() {
    use CssSpecifiedValueSerializationErrorKind as K;
    use CssSpecifiedValueSerializationLimits as L;
    let source = declaration(
        CssKnownProperty::Filter,
        "hue-rotate(0.10000000000000000001deg)",
    );
    let before = source.clone();
    let CssFilter::Functions(list) = filter(&source) else {
        panic!("function list")
    };
    let [function] = list.functions() else {
        panic!("one hue rotation")
    };
    let expected = "hue-rotate(0.10000000000000000001deg)";
    // One function aggregate plus its one ordinary scalar: 2/2.
    assert_eq!(
        function
            .serialize_specified_with_limits(L::new(2, 2, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (L::new(1, 2, expected.len()), K::InputNodeLimit),
        (L::new(2, 1, expected.len()), K::ProjectionNodeLimit),
        (L::new(2, 2, expected.len() - 1), K::ByteLimit),
    ] {
        assert_eq!(
            function
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(function.serialize_specified().unwrap(), expected);
    assert_eq!(source, before);
    let huge = declaration(CssKnownProperty::Filter, "hue-rotate(1e999deg)");
    let before = huge.clone();
    assert_eq!(
        filter(&huge)
            .serialize_specified_with_limits(L::new(3, 3, 64))
            .unwrap_err()
            .kind(),
        K::ByteLimit
    );
    assert_eq!(huge, before);
}

#[test]
fn suppressed_exact_gradient_default_still_consumes_its_scalar_budget_without_output_bytes() {
    use CssSpecifiedValueSerializationErrorKind as K;
    use CssSpecifiedValueSerializationLimits as L;
    let source = declaration(
        CssKnownProperty::BackgroundImage,
        "linear-gradient(+1.80e2deg, red, blue)",
    );
    let before = source.clone();
    let value = gradient(&source);
    let expected = "linear-gradient(red, blue)";
    // gradient 1 + direction 1 + scalar 1 + stop list 1 + 2*(stop 1+color 1).
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(8, 8, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (L::new(7, 8, expected.len()), K::InputNodeLimit),
        (L::new(8, 7, expected.len()), K::ProjectionNodeLimit),
        (L::new(8, 8, expected.len() - 1), K::ByteLimit),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    assert_eq!(source, before);
}

#[test]
fn pending_filter_reentry_rejects_recovered_angle_math_roots() {
    for property in [CssKnownProperty::Filter, CssKnownProperty::BackdropFilter] {
        let source = declaration(property, "var(--angle)");
        let CssExpansion::Pending(pending) = expand_declaration(&source).unwrap() else {
            panic!("pending filter")
        };
        let replacement = parse_component_values("hue-rotate(calc(25deg + 5deg").unwrap();
        let Err(error) = pending.reenter(replacement) else {
            panic!("recovered angle math must not reenter")
        };
        assert!(matches!(
            error.kind(),
            CssExpansionErrorKind::InvalidReplacement(_)
        ));
    }
}

#[test]
fn ordinary_gradient_outer_recovery_retains_its_complete_angle_calculation() {
    let css = "background-image:linear-gradient(calc(25deg + 5deg), red, blue";
    let report = parse_style_attribute(css);
    assert_eq!(report.syntax().len(), 1);
    assert!(!report.is_clean());
    assert!(validate_style_attribute(css).is_err());
    assert!(
        report
            .diagnostics()
            .iter()
            .any(|d| d.action() == CssRecoveryAction::RetainWithImplicitClosure)
    );
    assert_eq!(
        gradient(&report.syntax()[0]).serialize_specified().unwrap(),
        "linear-gradient(calc(30deg), red, blue)"
    );
}

// Functional evidence for the newly introduced checked owners.
mod checked_construction {
    use std::error::Error as _;
    use surgeist_css::*;

    fn component(text: &str) -> CssComponentValue {
        let values = parse_component_values(text).unwrap();
        let mut components = values.items().iter().filter(|value| {
            !matches!(
                value.view(),
                CssComponentValueRef::Token(CssValueTokenRef::Whitespace(_))
                    | CssComponentValueRef::Comment(_)
            )
        });
        let value = components.next().expect("one component");
        assert!(components.next().is_none());
        value.clone()
    }
    fn hue(angle: CssAngleOrZero) -> CssFilterFunction {
        CssFilterFunction::HueRotate(CssFilterHueRotate::new(angle))
    }
    fn literal(text: &str) -> CssAngleOrZero {
        CssAngleOrZero::Angle(CssAngleValue::from_literal(
            CssAngleLiteral::try_from_component(component(text)).unwrap(),
        ))
    }
    fn budget(value: &CssFilterFunction, expected: &str) {
        use CssSpecifiedValueSerializationErrorKind as K;
        use CssSpecifiedValueSerializationLimits as L;
        let before = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(L::new(2, 2, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (L::new(1, 2, expected.len()), K::InputNodeLimit),
            (L::new(2, 1, expected.len()), K::ProjectionNodeLimit),
            (L::new(2, 2, expected.len() - 1), K::ByteLimit),
        ] {
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(value, &before);
        }
    }

    #[test]
    fn exact_literal_constructors_borrow_original_tokens_units_and_origins() {
        for (text, coefficient, unit) in [
            (
                "+1.23000000000000000001DEG",
                "+1.23000000000000000001",
                CssAngleUnit::Degrees,
            ),
            (r"-.25t\75rn", "-.25", CssAngleUnit::Turns),
            ("1e999grad", "1e999", CssAngleUnit::Gradians),
            ("-1e-999rad", "-1e-999", CssAngleUnit::Radians),
        ] {
            let supplied = component(text);
            let angle = CssAngleLiteral::try_from_component(supplied.clone()).unwrap();
            assert_eq!(angle.numeric().representation(), coefficient);
            assert_eq!(angle.unit(), unit);
            assert_eq!(angle.component(), &supplied);
            assert_eq!(angle.origin(), supplied.origin());
            assert!(matches!(angle.origin(), CssValueOrigin::Parsed(_)));
        }
        for unit in [
            CssAngleUnit::Degrees,
            CssAngleUnit::Gradians,
            CssAngleUnit::Radians,
            CssAngleUnit::Turns,
        ] {
            let angle = CssAngleLiteral::try_new("+1.00000000000000000001e999", unit).unwrap();
            assert_eq!(
                angle.numeric().representation(),
                "+1.00000000000000000001e999"
            );
            assert_eq!(angle.origin(), &CssValueOrigin::Programmatic);
        }
    }

    #[test]
    fn literal_and_zero_failures_report_the_supplied_component_origin() {
        for text in ["0", "25%", "1px", "calc(1deg)", "auto"] {
            let supplied = component(text);
            let error = CssAngleLiteral::try_from_component(supplied.clone()).unwrap_err();
            assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidToken);
            assert_eq!(error.origin(), supplied.origin());
        }
        for text in ["1e-999", "-1e-999", "0deg", "0%", "calc(0)"] {
            let supplied = component(text);
            let error = CssZeroLiteral::try_from_component(supplied.clone()).unwrap_err();
            assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidToken);
            assert_eq!(error.origin(), supplied.origin());
        }
        for number in ["NaN", "infinity", "1 2", ""] {
            assert!(CssAngleLiteral::try_new(number, CssAngleUnit::Degrees).is_err());
            assert!(CssAngleCalculation::try_literal(number, CssAngleUnit::Degrees).is_err());
        }
    }

    #[test]
    fn exact_zero_keeps_signed_exponent_spelling_and_serializes_as_one_scalar() {
        for text in [
            "0",
            "-0",
            "+0.000e-999999999999999999999999999999",
            "-000e999999999999999999999999999999",
        ] {
            let supplied = component(text);
            let zero = CssZeroLiteral::try_from_component(supplied.clone()).unwrap();
            assert_eq!(zero.numeric().representation(), text);
            assert_eq!(zero.component(), &supplied);
            assert_eq!(zero.origin(), supplied.origin());
            let angle = CssAngleOrZero::Zero(zero);
            assert_eq!(angle.origin(), supplied.origin());
            budget(&hue(angle), "hue-rotate(0)");
        }
    }

    #[test]
    fn checked_calculation_transfer_normalizes_only_an_ordinary_dimension_root() {
        let supplied = component("+1.23000000000000000001TURN");
        let calculation = CssAngleCalculation::try_from_components(
            CssComponentValues::try_new(vec![supplied.clone()]).unwrap(),
        )
        .unwrap();
        let angle = CssAngleValue::try_from_calculation(calculation).unwrap();
        assert!(angle.calculation().is_none());
        assert_eq!(angle.literal().unwrap().component(), &supplied);
        assert_eq!(angle.origin(), supplied.origin());
        let ordinary = CssAngleCalculation::try_literal("1e999", CssAngleUnit::Radians).unwrap();
        assert_eq!(ordinary.result_type(), CssCalculationType::Angle);
        assert_eq!(
            CssAngleValue::try_from_calculation(ordinary)
                .unwrap()
                .literal()
                .unwrap()
                .numeric()
                .representation(),
            "1e999"
        );
        for text in ["calc(180deg)", "calc(90deg + 90deg)", "min(180deg, 200deg)"] {
            let calculation =
                CssAngleCalculation::try_from_components(parse_component_values(text).unwrap())
                    .unwrap();
            let angle = CssAngleValue::try_from_calculation(calculation.clone()).unwrap();
            assert!(angle.literal().is_none());
            assert_eq!(angle.calculation(), Some(&calculation));
            assert_eq!(angle.origin(), calculation.origin());
        }
    }

    #[test]
    fn transfer_rejects_recovered_math_at_the_original_implicit_closure() {
        let source = "/*😀*/filter:hue-rotate(calc(25deg + 5deg";
        let report = parse_style_attribute(source);
        assert_eq!(report.syntax().len(), 1);
        assert!(!report.is_clean());
        let CssKnownPropertyValueRef::Filter(filter) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("filter")
        };
        let CssFilter::Functions(functions) = filter.value() else {
            panic!("functions")
        };
        let [CssFilterFunction::HueRotate(hue)] = functions.functions() else {
            panic!("hue")
        };
        let Some(CssAngleOrZero::Angle(angle)) = hue.authored_angle() else {
            panic!("angle")
        };
        let calculation = angle.calculation().unwrap();
        let [root] = calculation.components().items() else {
            panic!("root")
        };
        let CssComponentValueRef::Function(function) = root.view() else {
            panic!("math")
        };
        let error = CssAngleValue::try_from_calculation(calculation.clone()).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssNumericConstructionErrorKind::RecoveredComponent
        );
        assert_eq!(error.origin(), Some(function.closing_origin()));
        assert!(error.path().is_none());
        assert!(error.source().is_none());
        let Some(CssValueOrigin::ImplicitClosure { opening, at }) = error.origin() else {
            panic!("implicit")
        };
        assert_eq!(opening.source().as_str(), source);
        assert_eq!(
            opening.span().start().byte_offset().value(),
            source.find("calc(").unwrap()
        );
        assert_eq!(at.span().start().byte_offset().value(), source.len());
    }

    #[test]
    fn raw_owner_equality_preserves_provenance_while_aggregates_compare_structure() {
        let first = literal("25deg");
        let second = literal(" 25deg");
        assert_ne!(first, second);
        assert_eq!(hue(first.clone()), hue(second.clone()));
        assert_eq!(
            CssTransformFunction::Rotate(first.clone()),
            CssTransformFunction::Rotate(second.clone())
        );
        assert_ne!(hue(first), hue(literal("25.0deg")));
        let first = CssZeroLiteral::try_from_component(component("-0e999")).unwrap();
        let second = CssZeroLiteral::try_from_component(component(" -0e999")).unwrap();
        assert_ne!(first, second);
        assert_eq!(
            hue(CssAngleOrZero::Zero(first)),
            hue(CssAngleOrZero::Zero(second))
        );
        assert_ne!(
            hue(literal("0deg")),
            hue(CssAngleOrZero::Zero(
                CssZeroLiteral::try_from_component(component("0")).unwrap()
            ))
        );
    }

    #[test]
    fn exact_literal_serialization_accounts_for_unit_bytes_and_cumulative_siblings() {
        for (authored, expected) in [
            (
                "+1.23000000000000000001DEG",
                "hue-rotate(1.23000000000000000001deg)",
            ),
            ("-.25turn", "hue-rotate(-0.25turn)"),
            ("-0e999rad", "hue-rotate(0rad)"),
            (
                "1e40grad",
                "hue-rotate(10000000000000000000000000000000000000000grad)",
            ),
        ] {
            budget(&hue(literal(authored)), expected);
        }
        let list =
            CssFilterFunctionList::try_new(vec![hue(literal(".5turn")), hue(literal("25grad"))])
                .unwrap();
        let expected = "hue-rotate(0.5turn) hue-rotate(25grad)";
        use CssSpecifiedValueSerializationErrorKind as K;
        use CssSpecifiedValueSerializationLimits as L;
        assert_eq!(
            list.serialize_specified_with_limits(L::new(5, 5, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (L::new(4, 5, expected.len()), K::InputNodeLimit),
            (L::new(5, 4, expected.len()), K::ProjectionNodeLimit),
            (L::new(5, 5, expected.len() - 1), K::ByteLimit),
        ] {
            assert_eq!(
                list.serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
        }
        let huge = hue(literal("1e999999999999999999999999999999deg"));
        assert_eq!(
            huge.serialize_specified_with_limits(L::new(2, 2, 64))
                .unwrap_err()
                .kind(),
            K::ByteLimit
        );
    }
}
