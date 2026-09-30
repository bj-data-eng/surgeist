#![forbid(unsafe_code)]

//! Functional current-value checks for Lists 3 and Counter Styles 3. Expected
//! canonical text follows CSSOM's meaning-preserving shorthand rule and the
//! pinned list-style parsing cases, including keyword/custom-name ambiguity.

use surgeist_css::*;

fn parsed(property: CssKnownProperty, value: &str) -> CssDeclaration {
    let source = format!("{}:{value}", property.canonical_name());
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax().as_slice()[0].clone()
}

fn named(name: &str) -> CssCounterStyleValue {
    CssCounterStyleValue::try_named(CssIdent::try_new(name).unwrap()).unwrap()
}

fn string(value: &str) -> CssContentString {
    CssContentString::try_new(value).unwrap()
}

fn shorthand(value: &CssDeclaration) -> &CssListStyleValue {
    let CssKnownPropertyValueRef::ListStyle(wrapper) =
        value.known().unwrap().property_value().unwrap()
    else {
        panic!("typed list-style")
    };
    wrapper.value()
}

#[test]
fn checked_construction_keeps_all_absent_invalid_and_borrows_authored_components() {
    assert!(CssListStyleValue::try_new(None, None, None).is_none());
    let type_value = CssListStyleTypeValue::CounterStyle(named("CustomMarker"));
    let image = CssImageValue::Url(CssUrl::try_new("#marker").unwrap());
    let value = CssListStyleValue::try_new(
        Some(type_value.clone()),
        Some(CssListStylePosition::Inside),
        Some(image.clone()),
    )
    .unwrap();
    assert_eq!(value.style_type(), Some(&type_value));
    assert_eq!(value.position(), Some(CssListStylePosition::Inside));
    assert_eq!(value.image(), Some(&image));
    assert_eq!(
        value.serialize_specified().unwrap(),
        "inside url(\"#marker\") CustomMarker"
    );
    assert_eq!(
        CssListStyleTypeValue::initial()
            .serialize_specified()
            .unwrap(),
        "disc"
    );
    assert_eq!(
        CssListStylePosition::Outside.serialize_specified().unwrap(),
        "outside"
    );
}

#[test]
fn parsed_wrappers_keep_named_functional_and_image_domains() {
    let type_old = parsed(CssKnownProperty::ListStyleType, "square");
    let CssKnownPropertyValueRef::ListStyleType(wrapper) =
        type_old.known().unwrap().property_value().unwrap()
    else {
        panic!("type")
    };
    assert_eq!(wrapper.value().serialize_specified().unwrap(), "square");

    let type_new = parsed(CssKnownProperty::ListStyleType, "symbols(cyclic \"*\")");
    let CssKnownPropertyValueRef::ListStyleType(wrapper) =
        type_new.known().unwrap().property_value().unwrap()
    else {
        panic!("functional type")
    };
    assert_eq!(
        wrapper.value().serialize_specified().unwrap(),
        "symbols(cyclic \"*\")"
    );

    let image_old = parsed(CssKnownProperty::ListStyleImage, "url(\"#star\")");
    let CssKnownPropertyValueRef::ListStyleImage(wrapper) =
        image_old.known().unwrap().property_value().unwrap()
    else {
        panic!("image")
    };
    assert_eq!(
        wrapper.value().serialize_specified().unwrap(),
        "url(\"#star\")"
    );

    let image_new = parsed(
        CssKnownProperty::ListStyleImage,
        "linear-gradient(red, blue)",
    );
    let CssKnownPropertyValueRef::ListStyleImage(wrapper) =
        image_new.known().unwrap().property_value().unwrap()
    else {
        panic!("gradient image")
    };
    assert_eq!(
        wrapper.value().serialize_specified().unwrap(),
        "linear-gradient(red, blue)"
    );

    let old = parsed(CssKnownProperty::ListStyle, "inside url(\"a\") square");
    let CssKnownPropertyValueRef::ListStyle(wrapper) =
        old.known().unwrap().property_value().unwrap()
    else {
        panic!("shorthand")
    };

    assert_eq!(
        wrapper.value().position(),
        Some(CssListStylePosition::Inside)
    );
    let new = parsed(CssKnownProperty::ListStyle, "symbols(cyclic \"*\") inside");
    let CssKnownPropertyValueRef::ListStyle(wrapper) =
        new.known().unwrap().property_value().unwrap()
    else {
        panic!("functional shorthand")
    };
    assert_eq!(
        wrapper.value().position(),
        Some(CssListStylePosition::Inside)
    );
    assert_eq!(wrapper.value().image(), None);
    let Some(CssListStyleTypeValue::CounterStyle(CssCounterStyleValue::Symbols(symbols))) =
        wrapper.value().style_type()
    else {
        panic!("functional counter style")
    };
    assert_eq!(symbols.system(), Some(CssSymbolsSystem::Cyclic));
    assert_eq!(
        symbols.symbols(),
        &[CssCounterSymbolValue::String(string("*"))]
    );
}

#[test]
fn shorthand_canonical_text_omits_only_meaning_preserving_initials() {
    for (authored, expected) in [
        ("none", "none"),
        ("none none", "none"),
        ("inside none none", "inside none"),
        ("disc outside none", "outside"),
        ("inside disc", "inside"),
        ("url(\"a\") disc outside", "url(\"a\")"),
        ("square src(\"#m\") inside", "inside src(\"#m\") square"),
        ("inside outside", "inside outside"),
        ("outside inside", "outside inside"),
    ] {
        let source = parsed(CssKnownProperty::ListStyle, authored);
        assert_eq!(
            shorthand(&source).serialize_specified().unwrap(),
            expected,
            "{authored}"
        );
        assert_eq!(
            source.value_components().serialize().unwrap().as_css(),
            authored,
            "authored spelling stays distinct"
        );
    }
}

#[test]
fn constructed_custom_axis_keyword_gets_a_disambiguating_position() {
    for name in ["inside", "outside"] {
        let value = CssListStyleValue::try_new(
            Some(CssListStyleTypeValue::CounterStyle(named(name))),
            None,
            None,
        )
        .unwrap();
        let expected = format!("outside {name}");
        assert_eq!(value.serialize_specified().unwrap(), expected);
        let reparsed = parsed(CssKnownProperty::ListStyle, &expected);
        let current = shorthand(&reparsed);
        assert_eq!(current.position(), Some(CssListStylePosition::Outside));
        assert!(
            matches!(current.style_type(), Some(CssListStyleTypeValue::CounterStyle(style)) if matches!(style.named(), Some(value) if value.as_str() == name))
        );
    }
}

#[test]
fn shorthand_charges_present_defaults_and_inserted_disambiguators() {
    let initial = CssListStyleValue::try_new(
        Some(CssListStyleTypeValue::initial()),
        Some(CssListStylePosition::Outside),
        Some(CssImageValue::None),
    )
    .unwrap();
    assert_eq!(
        initial
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(4, 4, 7))
            .unwrap(),
        "outside"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(3, 4, 7),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 3, 7),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(4, 4, 6),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            initial
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
    let disambiguated = CssListStyleValue::try_new(
        Some(CssListStyleTypeValue::CounterStyle(named("inside"))),
        None,
        None,
    )
    .unwrap();
    let expected = "outside inside";
    assert_eq!(
        disambiguated
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                2,
                3,
                expected.len(),
            ))
            .unwrap(),
        expected
    );
    assert_eq!(
        disambiguated
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                2,
                2,
                expected.len(),
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit
    );
    assert_eq!(disambiguated.serialize_specified().unwrap(), expected);
}

#[test]
fn nested_symbols_images_share_the_shorthand_budget_and_original_source() {
    let symbols = CssSymbolsStyleValue::try_new(
        None,
        ["#a", "#b"]
            .into_iter()
            .map(|target| {
                CssCounterSymbolValue::Image(
                    CssImage::try_new(CssImageValue::Url(CssUrl::try_new(target).unwrap()))
                        .unwrap(),
                )
            })
            .collect(),
    )
    .unwrap();
    let value = CssListStyleValue::try_new(
        Some(CssListStyleTypeValue::CounterStyle(
            CssCounterStyleValue::Symbols(symbols),
        )),
        None,
        None,
    )
    .unwrap();
    let expected = "symbols(url(\"#a\") url(\"#b\"))";
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                6,
                6,
                expected.len(),
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(0, 6, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(5, 6, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 5, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(6, 6, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }

    let authored = "symbols(linear-gradient(red calc(1px + 2%), blue) url(\"#star\" policy(flag)))";
    for property in [CssKnownProperty::ListStyleType, CssKnownProperty::ListStyle] {
        let full_source = format!("{}:{authored}", property.canonical_name());
        let source = parsed(property, authored);
        let current = match source.known().unwrap().property_value().unwrap() {
            CssKnownPropertyValueRef::ListStyleType(wrapper) => wrapper.value(),
            CssKnownPropertyValueRef::ListStyle(wrapper) => wrapper.value().style_type().unwrap(),
            _ => panic!("checked list style type"),
        };
        let CssListStyleTypeValue::CounterStyle(CssCounterStyleValue::Symbols(symbols)) = current
        else {
            panic!("checked symbols")
        };
        let [
            CssCounterSymbolValue::Image(first),
            CssCounterSymbolValue::Image(second),
        ] = symbols.symbols()
        else {
            panic!("two image symbols")
        };
        let CssImageValue::Gradient(CssGradient::Linear(gradient)) = first.value() else {
            panic!("linear gradient")
        };
        let CssColorStopListItem::Stop(stop) = &gradient.stops().items()[0] else {
            panic!("first gradient stop")
        };
        let calc = stop
            .position()
            .unwrap()
            .calculation()
            .expect("retained exact calculation");
        let CssValueOrigin::Parsed(origin) = calc.origin() else {
            panic!("original math origin")
        };
        assert_eq!(origin.source().as_str(), full_source);
        let span = origin.span();
        assert_eq!(
            &full_source[span.start().byte_offset().value()..span.end().byte_offset().value()],
            "calc("
        );
        let CssImageValue::Url(url) = second.value() else {
            panic!("URL symbol")
        };
        let [CssUrlModifier::Function(modifier)] = url.modifiers() else {
            panic!("one URL modifier")
        };
        let [argument] = modifier.argument_components().items() else {
            panic!("one argument")
        };
        let CssValueOrigin::Parsed(origin) = argument.origin() else {
            panic!("original modifier argument")
        };
        assert_eq!(origin.source().as_str(), full_source);
        let span = origin.span();
        assert_eq!(
            &full_source[span.start().byte_offset().value()..span.end().byte_offset().value()],
            "flag"
        );
        assert_eq!(
            current.serialize_specified().unwrap(),
            "symbols(linear-gradient(red calc(2% + 1px), blue) url(\"#star\" policy(flag)))"
        );
    }
}

#[test]
fn string_styles_and_image_values_serialize_without_layout_or_resource_lookup() {
    let string_style = CssListStyleTypeValue::String(string("a\"b"));
    assert_eq!(string_style.serialize_specified().unwrap(), "\"a\\\"b\"");
    let value = CssListStyleValue::try_new(
        Some(string_style),
        None,
        Some(CssImageValue::Url(CssUrl::try_new("").unwrap())),
    )
    .unwrap();
    assert_eq!(value.serialize_specified().unwrap(), "url(\"\") \"a\\\"b\"");
    let source = parsed(CssKnownProperty::ListStyle, "\"a\\\"b\" url(\"\")");
    assert_eq!(
        shorthand(&source).serialize_specified().unwrap(),
        value.serialize_specified().unwrap()
    );
}
