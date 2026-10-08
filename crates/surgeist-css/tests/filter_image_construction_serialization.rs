#![forbid(unsafe_code)]
//! Functional evidence for the new checked Filter Effects 1 §12 image APIs.
//! String execution interpretation remains undefined by the selected draft.
//! Independent structural limits and adopted provider tariffs are authored contracts.

#[path = "support/descriptor_phases.rs"]
mod descriptor_phases;
use descriptor_phases::*;

use surgeist_css::{
    CssSpecifiedValueSerializationErrorKind as K, CssSpecifiedValueSerializationLimits as L, *,
};

fn declaration(property: CssKnownProperty, text: &str) -> CssDeclaration {
    parse_property_value(
        CssPropertyNameRef::Known(property),
        parse_component_values(text).unwrap(),
        CssImportance::Normal,
    )
    .unwrap()
}
fn list(text: &str) -> CssFilterFunctionList {
    let source = declaration(CssKnownProperty::Filter, text);
    let CssKnownPropertyValueRef::Filter(value) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("filter property")
    };
    let CssFilter::Functions(value) = value.value() else {
        panic!("nonempty list")
    };
    value.clone()
}
fn image(text: &str) -> CssImageValue {
    let source = declaration(CssKnownProperty::BackgroundImage, text);
    let CssKnownPropertyValueRef::BackgroundImage(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("image property")
    };
    assert_eq!(value.images().images().len(), 1);
    value.images().images()[0].clone()
}
fn payload(text: &str) -> CssFilterImage {
    let CssImageValue::Filter(value) = image(text) else {
        panic!("filter image")
    };
    *value
}
fn string_input() -> CssFilterImageInput {
    CssFilterImageInput::from_string(CssFilterImageString::try_new("a").unwrap())
}
fn math(depth: usize, leaf: &str) -> CssComponentValues {
    let mut source = leaf.to_owned();
    for _ in 0..depth {
        source = format!("calc({source})");
    }
    let values = parse_component_values(&source).unwrap();
    assert_eq!(values.nesting_depth() as usize, depth);
    values
}
fn length(depth: usize) -> CssSpecifiedLength {
    CssSpecifiedLength::try_from_calculation(
        CssLengthCalculation::try_from_components(math(depth, "1px")).unwrap(),
    )
    .unwrap()
}
fn nonnegative_length(depth: usize) -> CssSpecifiedNonNegativeLength {
    CssSpecifiedNonNegativeLength::try_from_calculation(
        CssLengthCalculation::try_from_components(math(depth, "1px")).unwrap(),
    )
    .unwrap()
}
fn named(name: &str) -> CssColor {
    CssColor::from_named(CssNamedColor::try_new(name).unwrap())
}
fn colors(depth: usize) -> CssColor {
    let mut color = named("red");
    for _ in 0..depth {
        color =
            CssColor::from_light_dark(CssLightDarkColor::try_new(color, named("blue")).unwrap());
    }
    color
}
fn modified_url(depth: usize) -> CssUrl {
    CssUrl::from_parts(
        CssUrlFunction::Src,
        "a.svg",
        vec![CssUrlModifier::Function(
            CssUrlModifierFunction::try_new(CssIdent::try_new("hint").unwrap(), math(depth, "1"))
                .unwrap(),
        )],
    )
}

#[test]
fn string_input_retains_decoding_exact_token_and_authored_coordinates() {
    let source = "/*😀*/filter('é\\61 \\a x',blur())";
    let components = parse_component_values(source).unwrap();
    let original = components.clone();
    let function = components
        .items()
        .iter()
        .find_map(|item| match item.view() {
            CssComponentValueRef::Function(value) => Some(value),
            _ => None,
        })
        .unwrap();
    let token = function.values().items()[0].clone();
    let value = CssFilterImageString::try_from_component(token.clone()).unwrap();
    assert_eq!(value.as_str(), "éa\nx");
    assert_eq!(value.component(), &token);
    assert_eq!(value.origin(), token.origin());
    let CssValueOrigin::Parsed(origin) = value.origin() else {
        panic!("parsed token origin")
    };
    assert_eq!(origin.source().as_str(), source);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        "/*😀*/filter(".len()
    );
    assert_eq!(
        origin.span().start().column().value() as usize,
        "/*😀*/filter(".encode_utf16().count()
    );
    assert_eq!(value.serialize_specified().unwrap(), "\"éa\\a x\"");
    let parsed = payload(source);
    let CssFilterImageInputRef::String(retained) = parsed.input().view() else {
        panic!("distinct String")
    };
    assert_eq!(retained, &value);
    assert_eq!(retained.component(), &token);
    assert_eq!(components, original);
}

#[test]
fn checked_string_rejects_wrong_recovered_and_nul_inputs_with_real_origins() {
    for spelling in ["name", "url(a)", "42", "blur()"] {
        let values = parse_component_values(spelling).unwrap();
        let token = values.items()[0].clone();
        let error = CssFilterImageString::try_from_component(token.clone()).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidToken);
        assert_eq!(error.origin(), token.origin());
        assert_eq!(values.items()[0], token);
        let semantic: &dyn std::error::Error = &error;
        assert!(semantic.source().is_none());
    }
    let token = parse_component_values("'unterminated").unwrap().items()[0].clone();
    let error = CssFilterImageString::try_from_component(token.clone()).unwrap_err();
    assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidToken);
    let CssValueOrigin::ImplicitClosure { .. } = error.origin() else {
        panic!("original recovered EOF")
    };
    assert!(matches!(
        token.view(),
        CssComponentValueRef::Token(CssValueTokenRef::String("unterminated"))
    ));
    let error = CssFilterImageString::try_new("x\0y").unwrap_err();
    assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidString);
    assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
    for decoded in ["", "é😀", "a\"b\\c\nx"] {
        let value = CssFilterImageString::try_new(decoded).unwrap();
        assert_eq!(value.as_str(), decoded);
        assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
        assert!(
            matches!(value.component().view(), CssComponentValueRef::Token(CssValueTokenRef::String(text)) if text == decoded)
        );
    }
}

#[test]
fn input_views_separate_strings_urls_real_none_pairs_and_nested_filter_images() {
    assert_eq!(
        CssFilterImageInput::try_from_image(CssImageValue::None).unwrap_err(),
        CssImageConstructionError::NotImage
    );
    let text = CssFilterImageString::try_new("a.svg").unwrap();
    let input = CssFilterImageInput::from_string(text.clone());
    assert_eq!(input.view(), CssFilterImageInputRef::String(&text));
    assert_eq!(input.serialize_specified().unwrap(), "\"a.svg\"");
    for value in [
        CssImageValue::Url(CssUrl::new("a.svg")),
        CssImageValue::LightDark(Box::new(
            CssLightDarkImage::try_new(CssImageValue::None, CssImageValue::None).unwrap(),
        )),
        image("filter('a',blur())"),
    ] {
        let checked = CssImage::try_new(value.clone()).unwrap();
        let input = CssFilterImageInput::from_image(checked.clone());
        let CssFilterImageInputRef::Image(borrowed) = input.view() else {
            panic!("checked Image")
        };
        assert_eq!(borrowed, &checked);
        assert_eq!(borrowed.value(), &value);
        assert_eq!(
            CssFilterImageInput::try_from_image(value.clone()).unwrap(),
            input
        );
        assert_eq!(
            input.serialize_specified().unwrap(),
            value.serialize_specified().unwrap()
        );
        assert_eq!(
            input.serialize_specified().unwrap(),
            checked.serialize_specified().unwrap()
        );
    }
    assert!(CssFilterFunctionList::try_new(Vec::new()).is_none());
}

#[test]
fn direct_filter_image_retains_all_functions_sibling_order_and_omission_fields() {
    let functions = list(
        "blur() brightness(1) contrast() drop-shadow(light-dark(red, blue) 1px -2px) grayscale() hue-rotate() invert() opacity() saturate() sepia() src('#f')",
    );
    assert_eq!(functions.functions().len(), 11);
    let before = functions.clone();
    let input = string_input();
    let value = CssFilterImage::try_new(input.clone(), functions.clone()).unwrap();
    assert_eq!(value.input(), &input);
    assert_eq!(value.filters(), &before);
    assert!(
        matches!(&value.filters().functions()[0], CssFilterFunction::Blur(value) if value.authored_length().is_none())
    );
    assert!(matches!(
        &value.filters().functions()[1],
        CssFilterFunction::Brightness(CssFilterAmount::Number(_))
    ));
    assert!(matches!(
        &value.filters().functions()[2],
        CssFilterFunction::Contrast(CssFilterAmount::Default)
    ));
    assert!(
        matches!(&value.filters().functions()[5], CssFilterFunction::HueRotate(value) if value.authored_angle().is_none())
    );
    assert!(
        matches!(&value.filters().functions()[10], CssFilterFunction::Url(value) if value.function() == CssUrlFunction::Src)
    );
    assert_eq!(
        value.serialize_specified().unwrap(),
        "filter(\"a\", blur() brightness(1) contrast() drop-shadow(light-dark(red, blue) 1px -2px) grayscale() hue-rotate() invert() opacity() saturate() sepia() src(\"#f\"))"
    );
    assert_eq!(functions, before);
}

fn assert_string_filter(value: &CssImageValue) {
    let CssImageValue::Filter(filter) = value else {
        panic!("typed Filter image")
    };
    let CssFilterImageInputRef::String(input) = filter.input().view() else {
        panic!("retained String operand")
    };
    assert_eq!(input.as_str(), "é");
    assert!(matches!(input.origin(), CssValueOrigin::Parsed(_)));
    assert!(
        matches!(filter.filters().functions(), [CssFilterFunction::Blur(value)] if value.authored_length().is_none())
    );
}

#[test]
fn typed_importing_views_content_symbols_shapes_and_counter_styles_retain_filter_payloads() {
    for name in [
        "background-image",
        "border-image-source",
        "list-style-image",
        "mask-image",
        "mask-border-source",
        "content",
        "shape-outside",
    ] {
        let property = CssKnownProperty::from_name(name).unwrap();
        let source = declaration(property, "filter('é',blur())");
        let value = match source.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::BackgroundImage(value) => value.images().images()[0].clone(),
            CssKnownPropertyValueRef::BorderImageSource(value) => value.source().clone(),
            CssKnownPropertyValueRef::ListStyleImage(value) => value.value().clone(),
            CssKnownPropertyValueRef::MaskImage(value) => value.images().images()[0].clone(),
            CssKnownPropertyValueRef::MaskBorderSource(value) => value.source().clone(),
            CssKnownPropertyValueRef::Content(value) => {
                let CssContentValue::Generated(content) = value.value() else {
                    panic!("generated Content")
                };
                let [CssContentValueItem::Image(image)] = content.items() else {
                    panic!("one Image item")
                };
                image.value().clone()
            }
            CssKnownPropertyValueRef::ShapeOutside(value) => {
                let CssShapeOutside::Image(image) = value.value() else {
                    panic!("Shapes Image branch")
                };
                image.value().clone()
            }
            _ => panic!("selected image importing property"),
        };
        assert_string_filter(&value);
        assert_eq!(
            source.to_specified_css().unwrap(),
            format!("{name}: filter(\"é\", blur());")
        );
    }
    let source = declaration(
        CssKnownProperty::ListStyleType,
        "symbols(filter('é',blur()))",
    );
    let CssKnownPropertyValueRef::ListStyleType(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("list style type")
    };
    let CssListStyleTypeValue::CounterStyle(CssCounterStyleValue::Symbols(symbols)) = value.value()
    else {
        panic!("symbols()")
    };
    let [CssCounterSymbolValue::Image(image)] = symbols.symbols() else {
        panic!("one Image symbol")
    };
    assert_string_filter(image.value());

    let text = "@counter-style ImageCase { symbols:a; prefix:filter('é',blur()); }";
    let report = parse_sheet(text);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::CounterStyle(rule)] = report.syntax().rules() else {
        panic!("counter-style rule")
    };
    let CssCounterSymbol::Image(image) = rule.descriptors().prefix().unwrap().ordinary_prefix()
    else {
        panic!("checked counter Image")
    };
    assert_string_filter(image.value());
    let CssImageValue::Filter(filter) = image.value() else {
        panic!("counter Filter")
    };
    let CssFilterImageInputRef::String(input) = filter.input().view() else {
        panic!("counter String")
    };
    let CssValueOrigin::Parsed(origin) = input.origin() else {
        panic!("full sheet token source")
    };
    assert_eq!(origin.source().as_str(), text);
    assert_eq!(
        origin.span().start().byte_offset().value(),
        text.find("'é'").unwrap()
    );
}

#[test]
fn bookmark_label_keeps_filter_image_identity_and_one_cumulative_list_budget() {
    let text = "filter('é',blur())";
    let source = declaration(CssKnownProperty::BookmarkLabel, text);
    let original = source.clone();
    let CssKnownPropertyValueRef::BookmarkLabel(label) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("bookmark-label")
    };
    let [CssContentValueItem::Image(image)] = label.value().items() else {
        panic!("one retained Image item")
    };
    assert_string_filter(image.value());
    let expected = "bookmark-label: filter(\"é\", blur());";
    // Declaration/name2 + Content list1 + Filter/String/list/blur4.
    for (limits, kind) in [
        (L::new(6, 7, expected.len()), K::InputNodeLimit),
        (L::new(7, 6, expected.len()), K::ProjectionNodeLimit),
        (L::new(7, 7, expected.len() - 1), K::ByteLimit),
    ] {
        assert_eq!(
            source
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(source, original);
    }
    assert_eq!(
        source
            .to_specified_css_with_limits(L::new(7, 7, expected.len()))
            .unwrap(),
        expected
    );
    let report = parse_style_attribute(expected);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    assert_eq!(report.syntax()[0].to_specified_css().unwrap(), expected);

    let pending = declaration(CssKnownProperty::BookmarkLabel, "var(--image)");
    let CssExpansion::Pending(handle) = expand_declaration(&pending).unwrap() else {
        panic!("pending label")
    };
    let replacement = parse_component_values(text).unwrap();
    let before = replacement.clone();
    assert!(handle.reenter(replacement.clone()).is_ok());
    assert!(handle.source().same_occurrence(&pending));
    assert_eq!(replacement, before);
}

fn functions_at(depth: usize) -> Vec<CssFilterFunction> {
    let number = CssFilterAmount::Number(
        CssSpecifiedNonNegativeNumber::try_from_calculation(
            CssNumberCalculation::try_from_components(math(depth, "1")).unwrap(),
        )
        .unwrap(),
    );
    let percentage = CssFilterAmount::Percentage(
        CssSpecifiedNonNegativePercentage::try_from_calculation(
            CssPercentageCalculation::try_from_components(math(depth, "1%")).unwrap(),
        )
        .unwrap(),
    );
    // The division's retained parenthesis block adds one level to these calc wrappers.
    let mut hinted_source = "(1px + 1%) / 1px".to_owned();
    for _ in 0..depth - 1 {
        hinted_source = format!("calc({hinted_source})");
    }
    let hinted_components = parse_component_values(&hinted_source).unwrap();
    assert_eq!(hinted_components.nesting_depth() as usize, depth);
    let hinted = CssFilterAmount::HintedNumberCalculation(
        CssHintedNumberCalculation::try_from_components(hinted_components).unwrap(),
    );
    let angle = CssAngleOrZero::Angle(
        CssAngleValue::try_from_calculation(
            CssAngleCalculation::try_from_components(math(depth, "1deg")).unwrap(),
        )
        .unwrap(),
    );
    let plain = || {
        CssSpecifiedLength::try_from_component(CssComponentValue::try_token("0px").unwrap())
            .unwrap()
    };
    vec![
        CssFilterFunction::Blur(CssFilterBlur::new(nonnegative_length(depth))),
        CssFilterFunction::HueRotate(CssFilterHueRotate::new(angle)),
        CssFilterFunction::Brightness(number.clone()),
        CssFilterFunction::Contrast(number.clone()),
        CssFilterFunction::Grayscale(number.clone()),
        CssFilterFunction::Invert(number.clone()),
        CssFilterFunction::Opacity(number.clone()),
        CssFilterFunction::Saturate(number.clone()),
        CssFilterFunction::Sepia(number),
        CssFilterFunction::Opacity(percentage),
        CssFilterFunction::Brightness(hinted),
        CssFilterFunction::DropShadow(CssDropShadow::new(length(depth), plain(), None, None)),
        CssFilterFunction::DropShadow(CssDropShadow::new(plain(), length(depth), None, None)),
        CssFilterFunction::DropShadow(CssDropShadow::new(
            plain(),
            plain(),
            Some(nonnegative_length(depth)),
            None,
        )),
        CssFilterFunction::DropShadow(CssDropShadow::new(
            plain(),
            plain(),
            None,
            Some(colors(depth)),
        )),
    ]
}

#[test]
fn whole_filter_graph_checks_every_filter_and_numeric_color_role_in_every_sibling_slot() {
    let valid = functions_at(254);
    let invalid = functions_at(255);
    assert_eq!(valid.len(), invalid.len());
    // Child254 + ordinary filter/drop-shadow1 + filter-image1 = 256; next = 257.
    for (role, (valid, invalid)) in valid.iter().zip(&invalid).enumerate() {
        let valid_before = valid.clone();
        let invalid_before = invalid.clone();
        for position in 0..3 {
            let mut functions = vec![CssFilterFunction::Blur(CssFilterBlur::omitted()); 3];
            functions[position] = invalid.clone();
            let list = CssFilterFunctionList::try_new(functions).unwrap();
            let before = list.clone();
            for _ in 0..2 {
                assert_eq!(
                    CssFilterImage::try_new(string_input(), list.clone()).unwrap_err(),
                    CssImageConstructionError::NestingLimit,
                    "role {role}, slot {position}"
                );
                assert_eq!(list, before);
                let admitted = CssFilterImage::try_new(
                    string_input(),
                    CssFilterFunctionList::try_new(vec![valid.clone()]).unwrap(),
                )
                .unwrap();
                assert_eq!(
                    admitted.filters().functions(),
                    std::slice::from_ref(&valid_before)
                );
                assert_eq!(invalid, &invalid_before);
            }
        }
    }
}

#[test]
fn complete_checked_image_inputs_count_enclosing_filter_for_all_gradient_families() {
    let gradient = |depth, family| {
        let stops = CssColorStopList::try_new(vec![
            CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
                colors(depth),
                None,
            ))),
            CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
                named("blue"),
                None,
            ))),
        ])
        .unwrap();
        let value = match family {
            0 => CssGradient::Linear(CssLinearGradient::new(None, stops)),
            1 => CssGradient::RepeatingLinear(CssLinearGradient::new(None, stops)),
            2 => CssGradient::Radial(CssRadialGradient::try_new(None, None, None, stops).unwrap()),
            _ => CssGradient::RepeatingRadial(
                CssRadialGradient::try_new(None, None, None, stops).unwrap(),
            ),
        };
        CssImageValue::Gradient(value)
    };
    for family in 0..4 {
        let valid = CssFilterImageInput::try_from_image(gradient(254, family)).unwrap();
        let invalid = CssFilterImageInput::try_from_image(gradient(255, family)).unwrap();
        let before = invalid.clone();
        assert_eq!(
            CssFilterImageInput::try_from_image(gradient(256, family)).unwrap_err(),
            CssImageConstructionError::NestingLimit
        );
        for _ in 0..2 {
            assert_eq!(
                CssFilterImage::try_new(invalid.clone(), list("blur()")).unwrap_err(),
                CssImageConstructionError::NestingLimit
            );
            assert_eq!(invalid, before);
            assert!(CssFilterImage::try_new(valid.clone(), list("blur()")).is_ok());
        }
    }
}

#[test]
fn url_input_and_each_list_sibling_include_modifier_and_original_argument_depth() {
    // URL1 + modifier1 + arguments253 + filter-image1 = 256, then 257.
    let valid = modified_url(253);
    let invalid = modified_url(254);
    let before = invalid.clone();
    assert!(
        CssFilterImage::try_new(
            CssFilterImageInput::try_from_image(CssImageValue::Url(valid.clone())).unwrap(),
            list("blur()")
        )
        .is_ok()
    );
    assert_eq!(
        CssFilterImage::try_new(
            CssFilterImageInput::try_from_image(CssImageValue::Url(invalid.clone())).unwrap(),
            list("blur()")
        )
        .unwrap_err(),
        CssImageConstructionError::NestingLimit
    );
    for position in 0..3 {
        let mut functions = vec![CssFilterFunction::Blur(CssFilterBlur::omitted()); 3];
        functions[position] = CssFilterFunction::Url(invalid.clone());
        assert_eq!(
            CssFilterImage::try_new(
                string_input(),
                CssFilterFunctionList::try_new(functions.clone()).unwrap()
            )
            .unwrap_err(),
            CssImageConstructionError::NestingLimit
        );
        functions[position] = CssFilterFunction::Url(valid.clone());
        assert!(
            CssFilterImage::try_new(
                string_input(),
                CssFilterFunctionList::try_new(functions).unwrap()
            )
            .is_ok()
        );
    }
    assert_eq!(invalid, before);
}

#[test]
fn mixed_full_image_color_math_and_url_graphs_share_one_enclosing_depth() {
    let stops = CssColorStopList::try_new(vec![
        CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
            colors(254),
            None,
        ))),
        CssColorStopListItem::Stop(Box::new(CssGradientColorStop::from_color(
            named("blue"),
            None,
        ))),
    ])
    .unwrap();
    let input = CssFilterImageInput::try_from_image(CssImageValue::Gradient(CssGradient::Linear(
        CssLinearGradient::new(None, stops),
    )))
    .unwrap();
    let input_before = input.clone();
    let mut siblings = functions_at(254);
    siblings.push(CssFilterFunction::Url(modified_url(253)));
    let valid = CssFilterFunctionList::try_new(siblings.clone()).unwrap();
    let before = valid.clone();
    // Each child path reaches 255; siblings do not sum function nesting.
    let graph = CssFilterImage::try_new(input.clone(), valid.clone()).unwrap();
    assert_eq!(graph.input(), &input_before);
    assert_eq!(graph.filters(), &before);
    // Raise a later URL path to256: enclosing filter image is257.
    *siblings.last_mut().unwrap() = CssFilterFunction::Url(modified_url(254));
    let invalid = CssFilterFunctionList::try_new(siblings).unwrap();
    for _ in 0..2 {
        assert_eq!(
            CssFilterImage::try_new(input.clone(), invalid.clone()).unwrap_err(),
            CssImageConstructionError::NestingLimit
        );
        assert_eq!(input, input_before);
        assert_eq!(valid, before);
        assert!(CssFilterImage::try_new(input.clone(), valid.clone()).is_ok());
    }
}

#[test]
fn nested_filter_cached_depth_and_both_light_dark_branches_obey_256_and_257() {
    let mut input = string_input();
    let mut value = None;
    let mut matched = None;
    for index in 0..255 {
        let filter = CssFilterImage::try_new(input, list("blur()")).unwrap();
        let image = CssImageValue::Filter(Box::new(filter));
        input = CssFilterImageInput::try_from_image(image.clone()).unwrap();
        if index == 253 {
            matched = Some(image.clone());
        }
        value = Some(image);
    }
    let value = value.unwrap();
    let before = value.clone();
    assert_eq!(
        CssFilterImage::try_new(input, list("blur()")).unwrap_err(),
        CssImageConstructionError::NestingLimit
    );
    for (light, dark) in [
        (value.clone(), CssImageValue::None),
        (CssImageValue::None, value.clone()),
    ] {
        assert_eq!(
            CssLightDarkImage::try_new(light, dark).unwrap_err(),
            CssImageConstructionError::NestingLimit
        );
    }
    let matched = matched.unwrap();
    for (light, dark) in [
        (matched.clone(), CssImageValue::None),
        (CssImageValue::None, matched.clone()),
    ] {
        let pair = CssLightDarkImage::try_new(light.clone(), dark.clone()).unwrap();
        assert_eq!(pair.light(), &light);
        assert_eq!(pair.dark(), &dark);
        assert!(CssImage::try_new(CssImageValue::LightDark(Box::new(pair))).is_ok());
    }
    let specified = value.serialize_specified().unwrap();
    assert_eq!(specified.matches("filter(").count(), 255);
    assert_eq!(specified.matches("blur()").count(), 255);
    assert_eq!(value, before);
    let mut current = &value;
    let mut retained = 0;
    while let CssImageValue::Filter(filter) = current {
        retained += 1;
        match filter.input().view() {
            CssFilterImageInputRef::Image(image) => current = image.value(),
            CssFilterImageInputRef::String(value) => {
                assert_eq!(value.as_str(), "a");
                break;
            }
            _ => panic!("published operand variants"),
        }
    }
    assert_eq!(retained, 255);
    drop(before);
    drop(value);
}

#[test]
fn failed_filter_composition_preserves_mixed_provider_components_and_original_leaves() {
    let parsed = parse_component_values("/*😀*/1px").unwrap();
    let parsed_before = parsed.clone();
    let mut components = parsed.clone();
    for _ in 0..255 {
        components = CssComponentValues::try_new(vec![
            CssComponentValue::try_function("calc", components).unwrap(),
        ])
        .unwrap();
    }
    assert_eq!(components.nesting_depth(), 255);
    let before = components.clone();
    let length = CssSpecifiedNonNegativeLength::try_from_calculation(
        CssLengthCalculation::try_from_components(components.clone()).unwrap(),
    )
    .unwrap();
    let filters = CssFilterFunctionList::try_new(vec![CssFilterFunction::Blur(
        CssFilterBlur::new(length.clone()),
    )])
    .unwrap();
    assert_eq!(
        CssFilterImage::try_new(string_input(), filters).unwrap_err(),
        CssImageConstructionError::NestingLimit
    );
    assert_eq!(length.calculation().unwrap().components(), &before);
    assert_eq!(components, before);
    assert_eq!(parsed, parsed_before);
}

#[test]
fn aggregate_filter_refusal_preserves_actual_original_opener_through_imports_and_retry() {
    let text = |wrappers: usize| {
        let mut image = "url(a.svg)".to_owned();
        for _ in 0..wrappers {
            image = format!("light-dark({image}, none)");
        }
        format!("/*😀*/filter({image}, blur())")
    };
    // Raw filter1 + LightDark255 fits256; retained URL adds one and yields257.
    let invalid = parse_component_values(&text(255)).unwrap();
    let valid = parse_component_values(&text(254)).unwrap();
    assert_eq!(invalid.nesting_depth(), 256);
    assert_eq!(valid.nesting_depth(), 255);
    let before = invalid.clone();
    let opener = invalid
        .items()
        .iter()
        .find(|item| matches!(item.view(), CssComponentValueRef::Function(_)))
        .unwrap()
        .origin()
        .clone();
    let CssValueOrigin::Parsed(origin) = &opener else {
        panic!("actual original filter opener")
    };
    assert_eq!(origin.span().start().byte_offset().value(), "/*😀*/".len());
    for name in ["background-image", "content", "shape-outside"] {
        let property = CssKnownProperty::from_name(name).unwrap();
        let source = declaration(property, "var(--image)");
        let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
            panic!("pending image")
        };
        for _ in 0..2 {
            let direct = parse_property_value(
                CssPropertyNameRef::Known(property),
                invalid.clone(),
                CssImportance::Normal,
            )
            .unwrap_err();
            let grammar = parse_property_value_for_grammar(
                property.grammar(),
                invalid.clone(),
                CssImportance::Normal,
            )
            .unwrap_err();
            assert_eq!(
                direct.kind(),
                &CssPropertyValueErrorKind::Component(CssComponentValueErrorKind::NestingLimit)
            );
            assert_eq!(direct.origin(), &CssSerializedOrigin::Token(opener.clone()));
            assert_eq!(grammar, direct);
            let error = handle.reenter(invalid.clone()).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(actual) = error.kind() else {
                panic!("resource replacement failure")
            };
            assert_eq!(actual, &direct);
            assert!(handle.reenter(valid.clone()).is_ok());
            assert!(handle.source().same_occurrence(&source));
            assert_eq!(invalid, before);
        }
    }
}

fn budget(value: &CssFilterImage, expected: &str, input: usize, projection: usize) {
    let before = value.clone();
    for _ in 0..2 {
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
            assert_eq!(
                value
                    .serialize_specified_with_limits(limits)
                    .unwrap_err()
                    .kind(),
                kind
            );
            assert_eq!(value, &before);
        }
        assert_eq!(
            value
                .serialize_specified_with_limits(L::new(input, projection, expected.len()))
                .unwrap(),
            expected
        );
        let image = CssImageValue::Filter(Box::new(value.clone()));
        let checked = CssImage::try_new(image.clone()).unwrap();
        assert_eq!(
            image
                .serialize_specified_with_limits(L::new(input, projection, expected.len()))
                .unwrap(),
            expected
        );
        assert_eq!(
            checked
                .serialize_specified_with_limits(L::new(input, projection, expected.len()))
                .unwrap(),
            expected
        );
    }
}

#[test]
fn direct_filter_image_serializers_share_exact_image_and_filter_provider_tariffs() {
    for (source, expected, input, projection) in [
        ("filter('a',blur())", "filter(\"a\", blur())", 4, 4),
        ("filter(url(a),blur())", "filter(url(\"a\"), blur())", 5, 5),
        (
            "filter(filter('a',blur()),hue-rotate())",
            "filter(filter(\"a\", blur()), hue-rotate())",
            7,
            7,
        ),
        (
            "filter(light-dark(none,none),blur())",
            "filter(light-dark(none, none), blur())",
            6,
            6,
        ),
        (
            "filter('a',blur(calc(1px + 2em)) blur(calc(1px + 2em)))",
            "filter(\"a\", blur(calc(2em + 1px)) blur(calc(2em + 1px)))",
            13,
            15,
        ),
        (
            "filter(src('x' cors m(a b)),hue-rotate())",
            "filter(src(\"x\" cors m(a b)), hue-rotate())",
            10,
            10,
        ),
        (
            "filter(linear-gradient(to bottom,red 0%,blue 100%),blur())",
            "filter(linear-gradient(red, blue), blur())",
            13,
            13,
        ),
        (
            "filter(radial-gradient(ellipse farthest-corner at 50% 50%,red,blue),blur())",
            "filter(radial-gradient(red, blue), blur())",
            17,
            17,
        ),
    ] {
        budget(&payload(source), expected, input, projection);
    }
}

#[test]
fn standalone_string_and_input_serializers_share_utf8_budget_without_wrapper_charges() {
    let value = CssFilterImageString::try_new("é😀\"\\\nx").unwrap();
    let input = CssFilterImageInput::from_string(value.clone());
    let expected = "\"é😀\\\"\\\\\\a x\"";
    for _ in 0..2 {
        assert_eq!(
            value
                .serialize_specified_with_limits(L::new(1, 1, expected.len() - 1))
                .unwrap_err()
                .kind(),
            K::ByteLimit
        );
        assert_eq!(
            input
                .serialize_specified_with_limits(L::new(1, 1, expected.len() - 1))
                .unwrap_err()
                .kind(),
            K::ByteLimit
        );
        assert_eq!(
            value
                .serialize_specified_with_limits(L::new(1, 1, expected.len()))
                .unwrap(),
            expected
        );
        assert_eq!(
            input
                .serialize_specified_with_limits(L::new(1, 1, expected.len()))
                .unwrap(),
            expected
        );
        assert_eq!(
            input
                .serialize_specified_with_limits(L::new(0, 1, expected.len()))
                .unwrap_err()
                .kind(),
            K::InputNodeLimit
        );
        assert_eq!(
            input
                .serialize_specified_with_limits(L::new(1, 0, expected.len()))
                .unwrap_err()
                .kind(),
            K::ProjectionNodeLimit
        );
    }
    let input = CssFilterImageInput::try_from_image(CssImageValue::Url(CssUrl::new("é"))).unwrap();
    assert_eq!(
        input
            .serialize_specified_with_limits(L::new(2, 2, "url(\"é\")".len()))
            .unwrap(),
        "url(\"é\")"
    );
    assert_eq!(
        input
            .serialize_specified_with_limits(L::new(1, 2, 64))
            .unwrap_err()
            .kind(),
        K::InputNodeLimit
    );
    assert_eq!(
        input
            .serialize_specified_with_limits(L::new(2, 1, 64))
            .unwrap_err()
            .kind(),
        K::ProjectionNodeLimit
    );
}
