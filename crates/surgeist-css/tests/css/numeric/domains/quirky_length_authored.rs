#![forbid(unsafe_code)]
//! Values4 Appendix C: retained Number tokens have a distinct pixel interpretation.
use surgeist_css::*;

const QUIRKS: CssParserContext = CssParserContext::new(CssParserMode::Quirks);

fn declaration(name: CssKnownProperty, values: CssComponentValues) -> CssDeclaration {
    QUIRKS
        .parse_property_value_for_grammar(name.grammar(), values, CssImportance::Normal)
        .unwrap()
}

#[test]
fn contextual_length_owners_keep_original_number_tokens_and_distinguish_pixels() {
    for (text, expected) in [("+7", "7px"), ("1.0", "1px"), ("1e1", "10px")] {
        for values in [
            parse_component_values(text).unwrap(),
            CssComponentValues::try_new(vec![CssComponentValue::try_number(text).unwrap()])
                .unwrap(),
        ] {
            let original = &values.items()[0];
            let margin = declaration(CssKnownProperty::MarginTop, values.clone());
            let CssKnownPropertyValueRef::MarginTop(margin) =
                margin.known().unwrap().property_value().unwrap()
            else {
                panic!("margin")
            };
            let margin = margin.value().length_percentage().unwrap();
            assert!(margin.is_quirky_length());
            assert_eq!(margin.literal_component(), Some(original));
            assert_eq!(margin.origin(), original.origin());
            assert_eq!(margin.calculation(), None);
            assert_eq!(margin.serialize_specified().unwrap(), expected);
            assert!(CssSpecifiedLengthPercentage::try_from_component(original.clone()).is_err());

            let padding = declaration(CssKnownProperty::PaddingTop, values.clone());
            let CssKnownPropertyValueRef::PaddingTop(padding) =
                padding.known().unwrap().property_value().unwrap()
            else {
                panic!("padding")
            };
            let padding = padding.value().length_percentage();
            assert!(padding.is_quirky_length());
            assert_eq!(padding.literal_component(), Some(original));
            assert_eq!(padding.origin(), original.origin());
            assert_eq!(padding.calculation(), None);
            assert_eq!(padding.serialize_specified().unwrap(), expected);
            assert!(
                CssSpecifiedNonNegativeLengthPercentage::try_from_component(original.clone())
                    .is_err()
            );

            let border = declaration(CssKnownProperty::BorderTopWidth, values.clone());
            let CssKnownPropertyValueRef::BorderTopWidth(border) =
                border.known().unwrap().property_value().unwrap()
            else {
                panic!("border")
            };
            let CssBorderWidth::Length(border) = border.value() else {
                panic!("length")
            };
            assert!(border.is_quirky_length());
            assert_eq!(border.literal_component(), Some(original));
            assert_eq!(border.origin(), original.origin());
            assert_eq!(border.calculation(), None);
            assert_eq!(border.serialize_specified().unwrap(), expected);
            assert!(CssSpecifiedNonNegativeLength::try_from_component(original.clone()).is_err());

            // The sole function exception retains the same inner Number, even
            // for programmatic origins that carry no source position.
            let rect = CssComponentValue::try_function(
                "rect",
                CssComponentValues::try_new(vec![
                    original.clone(),
                    CssComponentValue::try_token(",").unwrap(),
                    CssComponentValue::try_ident("auto").unwrap(),
                    CssComponentValue::try_token(",").unwrap(),
                    CssComponentValue::try_ident("auto").unwrap(),
                    CssComponentValue::try_token(",").unwrap(),
                    CssComponentValue::try_ident("auto").unwrap(),
                ])
                .unwrap(),
            )
            .unwrap();
            let clip = declaration(
                CssKnownProperty::Clip,
                CssComponentValues::try_new(vec![rect]).unwrap(),
            );
            let CssKnownPropertyValueRef::Clip(clip) =
                clip.known().unwrap().property_value().unwrap()
            else {
                panic!("clip")
            };
            let CssClip::Rect(clip) = clip.clip() else {
                panic!("rectangle")
            };
            let CssClipEdge::Length(clip) = clip.top() else {
                panic!("length")
            };
            assert!(clip.is_quirky_length());
            assert_eq!(clip.literal_component(), Some(original));
            assert_eq!(clip.origin(), original.origin());
            assert_eq!(clip.calculation(), None);
            assert_eq!(clip.serialize_specified().unwrap(), expected);
            assert!(CssSpecifiedLength::try_from_component(original.clone()).is_err());
        }
    }
}

#[test]
fn exact_zero_and_explicit_dimensions_remain_ordinary_retained_literals() {
    for text in ["0", "-0", "+0.0e99", "7px"] {
        let values = parse_component_values(text).unwrap();
        let declaration = declaration(CssKnownProperty::MarginTop, values.clone());
        let CssKnownPropertyValueRef::MarginTop(margin) =
            declaration.known().unwrap().property_value().unwrap()
        else {
            panic!("margin")
        };
        let length = margin.value().length_percentage().unwrap();
        assert!(!length.is_quirky_length());
        assert_eq!(length.literal_component(), Some(&values.items()[0]));
        assert_eq!(length.origin(), values.items()[0].origin());
    }
}

#[test]
fn negative_nonzero_coefficients_retain_exact_sign_beyond_tokenizer_float_range() {
    let values = parse_component_values("-1e-1000").unwrap();
    let declaration = declaration(CssKnownProperty::MarginTop, values.clone());
    let CssKnownPropertyValueRef::MarginTop(margin) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("margin")
    };
    let length = margin.value().length_percentage().unwrap();
    assert!(length.is_quirky_length());
    assert_eq!(length.literal_component(), Some(&values.items()[0]));
    assert_eq!(length.serialize_specified().unwrap(), "0px");
    for property in [
        CssKnownProperty::PaddingTop,
        CssKnownProperty::BorderTopWidth,
        CssKnownProperty::Width,
    ] {
        assert!(
            QUIRKS
                .parse_property_value_for_grammar(
                    property.grammar(),
                    values.clone(),
                    CssImportance::Normal
                )
                .is_err()
        );
    }
}

#[test]
fn fit_content_function_arguments_do_not_inherit_the_named_property_permission() {
    for property in [
        CssKnownProperty::Width,
        CssKnownProperty::MinWidth,
        CssKnownProperty::MaxWidth,
    ] {
        for text in [
            "fit-content(7)",
            "fit-content(calc(7))",
            "calc-size(7, size)",
        ] {
            let values = parse_component_values(text).unwrap();
            assert!(
                QUIRKS
                    .parse_property_value_for_grammar(
                        property.grammar(),
                        values,
                        CssImportance::Normal
                    )
                    .is_err()
            );
        }
        assert!(
            QUIRKS
                .parse_property_value_for_grammar(
                    property.grammar(),
                    parse_component_values("fit-content(7px)").unwrap(),
                    CssImportance::Normal
                )
                .is_ok()
        );
    }
}

fn clip_length(original: &CssComponentValue) -> CssSpecifiedLength {
    let rect = CssComponentValue::try_function(
        "rect",
        CssComponentValues::try_new(vec![
            original.clone(),
            CssComponentValue::try_token(",").unwrap(),
            CssComponentValue::try_ident("auto").unwrap(),
            CssComponentValue::try_token(",").unwrap(),
            CssComponentValue::try_ident("auto").unwrap(),
            CssComponentValue::try_token(",").unwrap(),
            CssComponentValue::try_ident("auto").unwrap(),
        ])
        .unwrap(),
    )
    .unwrap();
    let declaration = declaration(
        CssKnownProperty::Clip,
        CssComponentValues::try_new(vec![rect]).unwrap(),
    );
    let CssKnownPropertyValueRef::Clip(clip) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("clip")
    };
    let CssClip::Rect(rect) = clip.clip() else {
        panic!("rectangle")
    };
    let CssClipEdge::Length(value) = rect.top() else {
        panic!("length")
    };
    value.clone()
}

#[test]
fn checked_pixel_lengths_compose_without_granting_number_syntax_to_composite_properties() {
    for values in [
        parse_component_values("+1e1").unwrap(),
        CssComponentValues::try_new(vec![CssComponentValue::try_number("+1e1").unwrap()]).unwrap(),
    ] {
        let original = &values.items()[0];
        let signed = clip_length(original);
        let border = declaration(CssKnownProperty::BorderTopWidth, values.clone());
        let CssKnownPropertyValueRef::BorderTopWidth(border) =
            border.known().unwrap().property_value().unwrap()
        else {
            panic!("border")
        };
        let CssBorderWidth::Length(nonnegative) = border.value() else {
            panic!("nonnegative length")
        };
        let direct = CssTextShadowLayer::try_new(
            false,
            signed.clone(),
            signed.clone(),
            Some(nonnegative.clone()),
            Some(nonnegative.clone()),
            None,
        )
        .unwrap();
        let composed = CssTextShadowLayer::try_from_shadow(
            CssShadow::try_new(
                false,
                signed.clone(),
                signed.clone(),
                Some(nonnegative.clone()),
                Some(signed.clone()),
                None,
            )
            .unwrap(),
        )
        .unwrap();
        for layer in [direct, composed] {
            let before = layer.clone();
            let shadow = layer.shadow();
            for value in [
                shadow.offset_x(),
                shadow.offset_y(),
                shadow.spread_radius().unwrap(),
            ] {
                assert!(value.is_quirky_length());
                assert_eq!(value.literal_component(), Some(original));
                assert_eq!(value.origin(), original.origin());
            }
            let blur = shadow.blur_radius().unwrap();
            assert!(blur.is_quirky_length());
            assert_eq!(blur.literal_component(), Some(original));
            assert_eq!(blur.origin(), original.origin());
            assert_eq!(layer.serialize_specified().unwrap(), "10px 10px 10px 10px");
            assert_eq!(
                layer
                    .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                        100, 100, 18
                    ))
                    .unwrap_err()
                    .kind(),
                CssSpecifiedValueSerializationErrorKind::ByteLimit
            );
            assert_eq!(layer, before);
            assert_eq!(layer.serialize_specified().unwrap(), "10px 10px 10px 10px");
        }

        let margin = declaration(CssKnownProperty::MarginTop, values.clone());
        let CssKnownPropertyValueRef::MarginTop(margin) =
            margin.known().unwrap().property_value().unwrap()
        else {
            panic!("margin")
        };
        let offset = CssTextUnderlineOffsetLength::try_new(
            margin.value().length_percentage().unwrap().clone(),
        )
        .unwrap();
        assert!(offset.value().is_quirky_length());
        assert_eq!(offset.value().literal_component(), Some(original));
        assert_eq!(offset.value().origin(), original.origin());
        assert_eq!(offset.serialize_specified().unwrap(), "10px");

        let padding = declaration(CssKnownProperty::PaddingTop, values.clone());
        let CssKnownPropertyValueRef::PaddingTop(padding) =
            padding.known().unwrap().property_value().unwrap()
        else {
            panic!("padding")
        };
        let radius = CssCornerRadiusValue::new(padding.value().length_percentage().clone(), None);
        assert!(radius.horizontal().is_quirky_length());
        assert_eq!(radius.horizontal().literal_component(), Some(original));
        assert_eq!(radius.vertical().origin(), original.origin());
        assert_eq!(radius.serialize_specified().unwrap(), "10px");

        for source in [
            "text-shadow:10 10 10 10",
            "text-underline-offset:10",
            "border-top-left-radius:10",
        ] {
            assert!(!QUIRKS.parse_declaration(source).is_clean());
        }
        assert!(CssSpecifiedLength::try_from_component(original.clone()).is_err());
        assert!(CssSpecifiedNonNegativeLength::try_from_component(original.clone()).is_err());
        assert!(CssSpecifiedLengthPercentage::try_from_component(original.clone()).is_err());
        assert!(
            CssSpecifiedNonNegativeLengthPercentage::try_from_component(original.clone()).is_err()
        );
    }
}

#[test]
fn text_shadow_checked_spread_narrowing_rejects_exact_negative_pixels_and_keeps_retry_inputs() {
    for text in ["-1", "-1e-1000"] {
        for values in [
            parse_component_values(text).unwrap(),
            CssComponentValues::try_new(vec![CssComponentValue::try_number(text).unwrap()])
                .unwrap(),
        ] {
            let signed = clip_length(&values.items()[0]);
            let before = signed.clone();
            let shadow = CssShadow::try_new(
                false,
                CssSpecifiedLength::zero(),
                CssSpecifiedLength::zero(),
                Some(CssSpecifiedNonNegativeLength::zero()),
                Some(signed.clone()),
                None,
            )
            .unwrap();
            assert!(CssTextShadowLayer::try_from_shadow(shadow).is_none());
            assert_eq!(signed, before);
            assert!(signed.is_quirky_length());
            assert_eq!(signed.literal_component(), Some(&values.items()[0]));
            assert_eq!(signed.origin(), values.items()[0].origin());
        }
    }
}
