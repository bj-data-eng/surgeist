#![forbid(unsafe_code)]
//! Functional new-API evidence, independent of the preimplementation authored RED.
//! Color 5 WD20260908 §6 owns grammar/ink identity; the shared structural
//! ceiling counts every function, including numeric, color and image ancestors.
use surgeist_css::*;

fn number(text: &str, programmatic: bool) -> CssColorComponent {
    let component = if programmatic {
        CssComponentValue::try_number(text).unwrap()
    } else {
        parse_component_values(text).unwrap().items()[0].clone()
    };
    let literal = CssColorNumberLiteral::try_from_component(component.clone()).unwrap();
    assert_eq!(literal.numeric().representation(), text);
    assert_eq!(literal.origin(), component.origin());
    CssColorComponent::Number(literal)
}
fn percentage(text: &str) -> CssColorComponent {
    CssColorComponent::Percentage(
        CssColorPercentageLiteral::try_from_component(
            parse_component_values(text).unwrap().items()[0].clone(),
        )
        .unwrap(),
    )
}
fn calculation(depth: usize, percent: bool) -> CssColorComponent {
    let scalar = if percent { "50%" } else { "0.5" };
    let raw = parse_component_values(&format!(
        "{}{scalar}{}",
        "calc(".repeat(depth),
        ")".repeat(depth)
    ))
    .unwrap();
    assert_eq!(raw.nesting_depth() as usize, depth);
    if percent {
        CssColorComponent::PercentageCalculation(
            CssPercentageCalculation::try_from_components(raw).unwrap(),
        )
    } else {
        CssColorComponent::NumberCalculation(
            CssNumberCalculation::try_from_components(raw).unwrap(),
        )
    }
}
fn numeric_payload(
    depth: usize,
    slot: usize,
) -> Result<CssDeviceCmykColor, CssColorConstructionError> {
    let mut channels = std::array::from_fn(|_| CssColorComponent::None);
    let mut alpha = None;
    if slot == 4 {
        alpha = Some(calculation(depth, true));
    } else {
        channels[slot] = calculation(depth, false);
    }
    CssDeviceCmykColor::try_new(CssColorSyntax::Modern, channels, alpha)
}
fn named(text: &str) -> CssColor {
    CssColor::from_named(CssNamedColor::try_new(text).unwrap())
}

#[test]
fn private_payload_accessors_retain_order_syntax_alpha_and_every_scalar_origin() {
    for programmatic in [false, true] {
        let channels = [
            number("-.2", programmatic),
            percentage("140%"),
            CssColorComponent::None,
            number("1.000000000000000000001", programmatic),
        ];
        let alpha = percentage("150%");
        let payload = CssDeviceCmykColor::try_new(
            CssColorSyntax::Modern,
            channels.clone(),
            Some(alpha.clone()),
        )
        .unwrap();
        assert_eq!(payload.syntax(), CssColorSyntax::Modern);
        assert_eq!(payload.channels(), &channels);
        assert_eq!(payload.alpha(), Some(&alpha));
        let value = CssColor::from_device_cmyk(payload.clone());
        assert_eq!(value.device_cmyk_value(), Some(&payload));
        assert_eq!(value.kind_name(), "device-cmyk");
        assert_eq!(
            value.absolute_eligibility(),
            CssAbsoluteColorEligibility::Contextual(CssAbsoluteColorExclusion::DeviceCmyk)
        );
        assert!(value.rgb_value().is_none());
        assert!(named("black").device_cmyk_value().is_none());
        let before = value.clone();
        for _ in 0..3 {
            assert_eq!(
                value.to_specified_css().unwrap(),
                "device-cmyk(-0.2 1.4 none 1.000000000000000000001)"
            );
            assert_eq!(value, before);
            assert_eq!(value.device_cmyk_value().unwrap().channels(), &channels);
            assert_eq!(value.device_cmyk_value().unwrap().alpha(), Some(&alpha));
        }
        let CssColorComponent::Number(first) = &channels[0] else {
            panic!("first literal")
        };
        if programmatic {
            assert_eq!(first.origin(), &CssValueOrigin::Programmatic);
        } else {
            let CssValueOrigin::Parsed(origin) = first.origin() else {
                panic!("parsed scalar")
            };
            assert_eq!(origin.source().as_str(), "-.2");
            assert_eq!(origin.span().start().byte_offset().value(), 0);
            assert_eq!(origin.span().end().byte_offset().value(), 3);
        }
    }
}

#[test]
fn legacy_number_and_number_calculation_payloads_are_checked_without_clamping() {
    let channels = [
        number("-2", true),
        calculation(1, false),
        number("4", false),
        number("5", true),
    ];
    let payload =
        CssDeviceCmykColor::try_new(CssColorSyntax::Legacy, channels.clone(), None).unwrap();
    assert_eq!(payload.syntax(), CssColorSyntax::Legacy);
    assert_eq!(payload.channels(), &channels);
    assert_eq!(payload.alpha(), None);
    assert_eq!(
        CssColor::from_device_cmyk(payload)
            .to_specified_css()
            .unwrap(),
        "device-cmyk(-2 calc(0.5) 4 5)"
    );
    for slot in 0..4 {
        for invalid in [
            CssColorComponent::None,
            percentage("10%"),
            calculation(1, true),
        ] {
            let mut channels = std::array::from_fn(|_| number("0", true));
            channels[slot] = invalid;
            assert_eq!(
                CssDeviceCmykColor::try_new(CssColorSyntax::Legacy, channels, None).unwrap_err(),
                CssColorConstructionError::InvalidSyntax
            );
        }
    }
    for alpha in [
        CssColorComponent::None,
        number(".5", true),
        percentage("50%"),
        calculation(1, false),
        calculation(1, true),
    ] {
        assert_eq!(
            CssDeviceCmykColor::try_new(
                CssColorSyntax::Legacy,
                std::array::from_fn(|_| number("0", true)),
                Some(alpha)
            )
            .unwrap_err(),
            CssColorConstructionError::InvalidSyntax
        );
    }
}

#[test]
fn modern_calculation_domains_and_omission_missing_alpha_are_distinct() {
    let channels = [
        calculation(1, false),
        calculation(1, true),
        percentage("50%"),
        CssColorComponent::None,
    ];
    let omitted =
        CssDeviceCmykColor::try_new(CssColorSyntax::Modern, channels.clone(), None).unwrap();
    let missing = CssDeviceCmykColor::try_new(
        CssColorSyntax::Modern,
        channels.clone(),
        Some(CssColorComponent::None),
    )
    .unwrap();
    assert_ne!(omitted, missing);
    assert!(omitted.alpha().is_none());
    assert_eq!(missing.alpha(), Some(&CssColorComponent::None));
    let CssColorComponent::NumberCalculation(number) = &omitted.channels()[0] else {
        panic!("typed number")
    };
    let CssColorComponent::PercentageCalculation(percent) = &omitted.channels()[1] else {
        panic!("typed percentage")
    };
    assert_eq!(number.result_type(), CssCalculationType::Number);
    assert_eq!(percent.result_type(), CssCalculationType::Percentage);
    assert!(matches!(
        number.expression().origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert_eq!(
        CssColor::from_device_cmyk(omitted)
            .to_specified_css()
            .unwrap(),
        "device-cmyk(calc(0.5) calc(0.5) 0.5 none)"
    );
    assert_eq!(
        CssColor::from_device_cmyk(missing)
            .to_specified_css()
            .unwrap(),
        "device-cmyk(calc(0.5) calc(0.5) 0.5 none / none)"
    );
}

#[test]
fn each_numeric_channel_and_alpha_obeys_255_child_plus_one_function_depth() {
    for slot in 0..5 {
        assert!(numeric_payload(255, slot).is_ok());
        assert_eq!(
            numeric_payload(256, slot).unwrap_err(),
            CssColorConstructionError::NestingLimit
        );
        let deepest = CssColor::from_device_cmyk(numeric_payload(255, slot).unwrap());
        assert_eq!(
            CssContrastColor::try_new(deepest.clone()).unwrap_err(),
            CssColorConstructionError::NestingLimit
        );
        for (light, dark) in [
            (deepest.clone(), named("blue")),
            (named("blue"), deepest.clone()),
        ] {
            assert_eq!(
                CssLightDarkColor::try_new(light, dark).unwrap_err(),
                CssColorConstructionError::NestingLimit
            );
        }
        assert_eq!(
            CssAlphaColor::try_new(deepest.clone(), None).unwrap_err(),
            CssColorConstructionError::NestingLimit
        );
        assert_eq!(
            CssColorMix::try_new(None, vec![CssColorMixComponent::new(deepest.clone(), None)])
                .unwrap_err(),
            CssColorMixConstructionError::NestingLimit
        );
        let channels = ["r", "g", "b"].map(|channel| {
            CssRelativeColorExpression::try_from_components(
                parse_component_values(channel).unwrap(),
                CssRelativeColorEnvironment::Rgb,
                CssRelativeColorResultDomain::NumberPercentage,
            )
            .unwrap()
        });
        assert_eq!(
            CssRelativeColor::try_new(CssRelativeColorFunction::Rgb, deepest, channels, None)
                .unwrap_err(),
            CssColorConstructionError::NestingLimit
        );
        let inner = CssColor::from_device_cmyk(numeric_payload(254, slot).unwrap());
        let wrapped =
            CssColor::from_contrast_color(CssContrastColor::try_new(inner.clone()).unwrap());
        assert_eq!(
            CssContrastColor::try_new(wrapped).unwrap_err(),
            CssColorConstructionError::NestingLimit
        );
        let stops = CssColorStopList::try_new(vec![
            CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(inner, None))),
            CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
                named("blue"),
                None,
            ))),
        ])
        .unwrap();
        let image =
            CssImageValue::Gradient(CssGradient::Linear(CssLinearGradient::new(None, stops)));
        assert_eq!(
            CssLightDarkImage::try_new(image, CssImageValue::None).unwrap_err(),
            CssImageConstructionError::NestingLimit
        );
    }
}

#[test]
fn parser_payload_keeps_typed_literals_calculations_and_sources() {
    let text = "color:device-cmyk(-.2 140% calc(1 / 2) calc(50%) / calc(25%))";
    let report = parse_style_attribute(text);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let CssKnownPropertyValueRef::Color(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("color")
    };
    let payload = value.value().device_cmyk_value().unwrap();
    assert_eq!(payload.syntax(), CssColorSyntax::Modern);
    let [
        CssColorComponent::Number(c),
        CssColorComponent::Percentage(m),
        CssColorComponent::NumberCalculation(y),
        CssColorComponent::PercentageCalculation(k),
    ] = payload.channels()
    else {
        panic!("four typed ordered channels")
    };
    assert_eq!(c.numeric().representation(), "-.2");
    assert_eq!(m.numeric().representation(), "140");
    for origin in [
        c.origin(),
        m.origin(),
        y.expression().origin(),
        k.expression().origin(),
    ] {
        let CssValueOrigin::Parsed(origin) = origin else {
            panic!("parsed domain origin")
        };
        assert_eq!(origin.source().as_str(), text);
    }
    assert!(matches!(
        payload.alpha(),
        Some(CssColorComponent::PercentageCalculation(_))
    ));
}

#[test]
fn metadata_records_only_the_complete_dated_authored_production() {
    let metadata = feature_metadata("interop.value.device-cmyk").unwrap();
    assert_eq!(metadata.kind(), CssFeatureKind::Value);
    assert_eq!(metadata.status(), CssSupportStatus::Complete);
    assert_eq!(metadata.production(), "#funcdef-device-cmyk");
    assert_eq!(metadata.source().id().as_str(), "I-COLOR5-20260908");
    assert_eq!(
        metadata.source().url(),
        Some("https://www.w3.org/TR/2026/WD-css-color-5-20260908/")
    );
    assert!(metadata.supported_subset().is_none());
    assert!(metadata.unsupported_remainder().is_none());
    assert_eq!(
        feature_metadata("ext.value.color-mix")
            .unwrap()
            .source()
            .id()
            .as_str(),
        "I-COLOR5"
    );
}
