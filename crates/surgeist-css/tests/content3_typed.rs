#![forbid(unsafe_code)]
//! Functional checked-value expectations from Content 3 (2025-12-04), Counter
//! Styles 3 (2021-07-27), and CSSOM §6.7.2's specified default omissions.

use surgeist_css::*;

fn ident(value: &str) -> CssIdent {
    CssIdent::try_new(value).unwrap()
}
fn string(value: &str) -> CssContentString {
    CssContentString::try_new(value).unwrap()
}
fn name(value: &str) -> CssContentName {
    CssContentName::try_new(ident(value)).unwrap()
}
fn counter_name(value: &str) -> CssContentCounterName {
    CssContentCounterName::try_new(ident(value)).unwrap()
}
fn counter(value: &str, style: Option<CssCounterStyleValue>) -> CssContentCounter {
    CssContentCounter::new(counter_name(value), style)
}
fn item(value: CssContentValueItem) -> CssContentValue {
    CssContentValue::Generated(CssGeneratedContent::try_new(vec![value], None).unwrap())
}
fn parsed(value: &str) -> CssDeclaration {
    let source = format!("content:{value}");
    let report = parse_style_attribute(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    report.syntax()[0].clone()
}

#[test]
fn generic_and_ordinary_counter_names_have_distinct_checked_domains() {
    for accepted in ["none", "span", "auto", "chapter name", "1chapter"] {
        assert_eq!(name(accepted).as_str(), accepted);
    }
    for forbidden in [
        "initial",
        "inherit",
        "unset",
        "revert",
        "revert-layer",
        "default",
        "DeFaUlT",
    ] {
        assert!(
            CssContentName::try_new(ident(forbidden)).is_none(),
            "{forbidden}"
        );
        assert!(
            CssContentCounterName::try_new(ident(forbidden)).is_none(),
            "{forbidden}"
        );
    }
    assert!(CssContentCounterName::try_new(ident("none")).is_none());
    for accepted in ["span", "auto", "chapter name", "1chapter"] {
        assert_eq!(counter_name(accepted).as_str(), accepted);
    }
}

#[test]
fn all_predefined_counter_styles_normalize_but_custom_names_keep_case() {
    // Counter Styles 3's complete selected predefined set, independent of the
    // restricted predefined counter-style keyword subset.
    let predefined = [
        "decimal",
        "decimal-leading-zero",
        "arabic-indic",
        "armenian",
        "upper-armenian",
        "lower-armenian",
        "bengali",
        "cambodian",
        "khmer",
        "cjk-decimal",
        "devanagari",
        "georgian",
        "gujarati",
        "gurmukhi",
        "hebrew",
        "kannada",
        "lao",
        "malayalam",
        "mongolian",
        "myanmar",
        "oriya",
        "persian",
        "lower-roman",
        "upper-roman",
        "tamil",
        "telugu",
        "thai",
        "tibetan",
        "lower-alpha",
        "lower-latin",
        "upper-alpha",
        "upper-latin",
        "lower-greek",
        "hiragana",
        "hiragana-iroha",
        "katakana",
        "katakana-iroha",
        "disc",
        "circle",
        "square",
        "disclosure-open",
        "disclosure-closed",
        "cjk-earthly-branch",
        "cjk-heavenly-stem",
        "japanese-informal",
        "japanese-formal",
        "korean-hangul-formal",
        "korean-hanja-informal",
        "korean-hanja-formal",
        "simp-chinese-informal",
        "simp-chinese-formal",
        "trad-chinese-informal",
        "trad-chinese-formal",
        "cjk-ideographic",
        "ethiopic-numeric",
    ];
    assert_eq!(predefined.len(), 55);
    for spelling in predefined {
        let upper = spelling.to_ascii_uppercase();
        let direct = CssCounterStyleReference::try_new(ident(&upper)).unwrap();
        assert_eq!(direct.as_str(), spelling, "{spelling}");
        let CssCounterStyleValue::Named(via_outer) =
            CssCounterStyleValue::try_named(ident(&upper)).unwrap()
        else {
            panic!("named style")
        };
        assert_eq!(via_outer, direct);
        assert_eq!(
            CssCounterStyleValue::Named(direct)
                .serialize_specified()
                .unwrap(),
            spelling
        );
        let parsed = parsed(&format!("counter(chapter, {upper})"));
        let CssKnownPropertyValueRef::Content(wrapper) =
            parsed.known().unwrap().property_value().unwrap()
        else {
            panic!("parsed counter style")
        };
        let CssContentValue::Generated(generated) = wrapper.value() else {
            panic!("generated counter")
        };
        let [CssContentValueItem::Counter(counter)] = generated.items() else {
            panic!("one counter")
        };
        assert_eq!(counter.style().unwrap().named().unwrap().as_str(), spelling);
    }
    let custom = CssCounterStyleReference::try_new(ident("MyStyle")).unwrap();
    assert_eq!(custom.as_str(), "MyStyle");
    assert_eq!(
        CssCounterStyleValue::Named(custom)
            .serialize_specified()
            .unwrap(),
        "MyStyle"
    );
    let parsed_custom = parsed(r"counter(chapter\ name, MyStyle)");
    let CssKnownPropertyValueRef::Content(wrapper) =
        parsed_custom.known().unwrap().property_value().unwrap()
    else {
        panic!("escaped custom counter")
    };
    let CssContentValue::Generated(generated) = wrapper.value() else {
        panic!("custom generated")
    };
    let [CssContentValueItem::Counter(counter)] = generated.items() else {
        panic!("one custom counter")
    };
    assert_eq!(counter.name().as_str(), "chapter name");
    assert_eq!(
        counter.style().unwrap().named().unwrap().as_str(),
        "MyStyle"
    );
    for forbidden in ["none", "default", "INHERIT"] {
        assert!(CssCounterStyleReference::try_new(ident(forbidden)).is_none());
    }
}

#[test]
fn symbols_require_the_selected_count_and_only_checked_string_or_image_leaves() {
    let star = CssCounterSymbolValue::String(string("*"));
    assert!(CssSymbolsStyleValue::try_new(None, vec![]).is_none());
    for system in [CssSymbolsSystem::Numeric, CssSymbolsSystem::Alphabetic] {
        assert!(CssSymbolsStyleValue::try_new(Some(system), vec![star.clone()]).is_none());
        assert!(
            CssSymbolsStyleValue::try_new(Some(system), vec![star.clone(), star.clone()]).is_some()
        );
    }
    let image = CssImage::try_new(CssImageValue::Url(CssUrl::try_new("#star").unwrap())).unwrap();
    let style = CssCounterStyleValue::Symbols(
        CssSymbolsStyleValue::try_new(None, vec![CssCounterSymbolValue::Image(image)]).unwrap(),
    );
    assert_eq!(
        style.serialize_specified().unwrap(),
        "symbols(url(\"#star\"))"
    );
    let symbolic = CssCounterStyleValue::Symbols(
        CssSymbolsStyleValue::try_new(Some(CssSymbolsSystem::Symbolic), vec![star]).unwrap(),
    );
    assert_eq!(symbolic.serialize_specified().unwrap(), "symbols(\"*\")");
}

#[test]
fn checked_generated_content_classifies_a_sole_image_as_replacement() {
    assert!(CssGeneratedContent::try_new(vec![], None).is_none());
    assert!(CssContentAlternative::try_new(vec![]).is_none());
    let image = CssImage::try_new(CssImageValue::Url(CssUrl::try_new("#icon").unwrap())).unwrap();
    let generated =
        CssGeneratedContent::try_new(vec![CssContentValueItem::Image(image.clone())], None)
            .unwrap();
    assert!(
        matches!(generated.body(), CssGeneratedContentBodyRef::Replacement(value) if value == &image)
    );
    let list = CssGeneratedContent::try_new(
        vec![
            CssContentValueItem::Image(image),
            CssContentValueItem::String(string("caption")),
        ],
        None,
    )
    .unwrap();
    assert!(matches!(
        list.body(),
        CssGeneratedContentBodyRef::List([
            CssContentValueItem::Image(_),
            CssContentValueItem::String(_)
        ])
    ));
    let alt =
        CssContentAlternative::try_new(vec![CssContentAlternativeItem::String(string("spoken"))])
            .unwrap();
    let value = CssContentValue::Generated(
        CssGeneratedContent::try_new(
            vec![CssContentValueItem::String(string("caption"))],
            Some(alt),
        )
        .unwrap(),
    );
    assert_eq!(
        value.serialize_specified().unwrap(),
        "\"caption\" / \"spoken\""
    );
}

#[test]
fn canonical_functions_omit_only_defined_defaults_and_retain_symbolic_targets() {
    let decimal = CssCounterStyleValue::try_named(ident("DECIMAL")).unwrap();
    assert_eq!(
        item(CssContentValueItem::Counter(counter(
            "chapter",
            Some(decimal.clone())
        )))
        .serialize_specified()
        .unwrap(),
        "counter(chapter)"
    );
    let counters =
        CssContentCounters::new(counter_name("chapter"), string("."), Some(decimal.clone()));
    assert_eq!(
        item(CssContentValueItem::Counters(counters))
            .serialize_specified()
            .unwrap(),
        "counters(chapter, \".\")"
    );
    assert_eq!(
        item(CssContentValueItem::NamedString(CssNamedString::new(
            name("chapter"),
            Some(CssNamedStringMode::First)
        )))
        .serialize_specified()
        .unwrap(),
        "string(chapter)"
    );
    assert_eq!(
        item(CssContentValueItem::Content(Some(
            CssContentReferenceMode::Text
        )))
        .serialize_specified()
        .unwrap(),
        "content()"
    );
    assert_eq!(
        item(CssContentValueItem::TargetText(CssTargetText::new(
            CssContentTarget::String(string("#ch")),
            Some(CssTargetTextMode::Content)
        )))
        .serialize_specified()
        .unwrap(),
        "target-text(\"#ch\", content)"
    );
    assert_eq!(
        item(CssContentValueItem::TargetCounter(CssTargetCounter::new(
            CssContentTarget::String(string("#ch")),
            name("chapter"),
            Some(decimal)
        )))
        .serialize_specified()
        .unwrap(),
        "target-counter(\"#ch\", chapter, decimal)"
    );
    for (leader, expected) in [
        (CssLeaderValue::Dotted, "leader(\".\")"),
        (CssLeaderValue::Solid, "leader(\"_\")"),
        (CssLeaderValue::Space, "leader(\" \")"),
    ] {
        assert_eq!(
            item(CssContentValueItem::Leader(leader))
                .serialize_specified()
                .unwrap(),
            expected
        );
    }
}

#[test]
fn parsed_content_retains_strings_counters_and_images() {
    let ordinary = parsed("\"caption\" counter(chapter)");
    let CssKnownPropertyValueRef::Content(wrapper) =
        ordinary.known().unwrap().property_value().unwrap()
    else {
        panic!("content wrapper")
    };
    assert!(matches!(wrapper.value(), CssContentValue::Generated(_)));

    assert!(matches!(
        ordinary.value_components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    let current_only = parsed("contents linear-gradient(red, blue)");
    let CssKnownPropertyValueRef::Content(wrapper) =
        current_only.known().unwrap().property_value().unwrap()
    else {
        panic!("current content")
    };

    assert_eq!(
        wrapper.value().serialize_specified().unwrap(),
        "contents linear-gradient(red, blue)"
    );
    let programmatic = parse_property_value(
        CssPropertyNameRef::Known(CssKnownProperty::Content),
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("contents").unwrap()])
            .unwrap(),
        CssImportance::Normal,
    )
    .unwrap();
    assert!(matches!(
        programmatic.value_components().items()[0].origin(),
        CssValueOrigin::Programmatic
    ));
    let CssKnownPropertyValueRef::Content(value) =
        programmatic.known().unwrap().property_value().unwrap()
    else {
        panic!("programmatic content")
    };
    assert_eq!(
        value.value(),
        &CssContentValue::Generated(
            CssGeneratedContent::try_new(vec![CssContentValueItem::Contents], None).unwrap()
        )
    );
}

#[test]
fn serialization_limits_are_cumulative_and_fail_without_mutation() {
    let value = CssContentValue::Generated(
        CssGeneratedContent::try_new(
            vec![
                CssContentValueItem::String(string("a")),
                CssContentValueItem::String(string("b")),
            ],
            None,
        )
        .unwrap(),
    );
    let expected = "\"a\" \"b\"";
    assert_eq!(value.serialize_specified().unwrap(), expected);
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                3,
                3,
                expected.len()
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, expected.len() - 1),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(0, 0, 0),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
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
    assert_eq!(value.serialize_specified().unwrap(), expected);
}

#[test]
fn nested_symbols_images_preserve_math_modifier_order_and_shared_budget() {
    let authored = "counter(chapter, symbols(linear-gradient(red calc(1px + 2%), blue) url(\"#star\" policy(flag))))";
    let source = parsed(authored);
    let full_source = format!("content:{authored}");
    assert!(matches!(
        source.value_components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    let CssKnownPropertyValueRef::Content(wrapper) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("nested content")
    };
    let CssContentValue::Generated(generated) = wrapper.value() else {
        panic!("generated content")
    };
    let [CssContentValueItem::Counter(counter)] = generated.items() else {
        panic!("one counter")
    };
    let Some(CssCounterStyleValue::Symbols(symbols)) = counter.style() else {
        panic!("functional style")
    };
    let [
        CssCounterSymbolValue::Image(first),
        CssCounterSymbolValue::Image(second),
    ] = symbols.symbols()
    else {
        panic!("ordered image symbols")
    };
    let CssImageValue::Gradient(CssGradient::Linear(gradient)) = first.value() else {
        panic!("linear gradient symbol")
    };
    let CssColorStopListItem::Stop(first_stop) = &gradient.stops().items()[0] else {
        panic!("first gradient stop")
    };
    let CssLength::Calc(CssCalcLength::Typed(calculation)) = first_stop.position().unwrap().value()
    else {
        panic!("symbolic first-stop math")
    };
    let CssValueOrigin::Parsed(calc_origin) = calculation.origin() else {
        panic!("original calculation source")
    };
    assert_eq!(calc_origin.source().as_str(), full_source);
    let calc_span = calc_origin.span();
    assert_eq!(
        &full_source
            [calc_span.start().byte_offset().value()..calc_span.end().byte_offset().value()],
        "calc("
    );
    let [calculation_root] = calculation.components().items() else {
        panic!("one retained calc function")
    };
    let CssComponentValueRef::Function(function) = calculation_root.view() else {
        panic!("retained calc arguments")
    };
    for expected in ["1px", "2%"] {
        assert!(
            function.values().items().iter().any(|child| {
                let CssValueOrigin::Parsed(origin) = child.origin() else {
                    return false;
                };
                let span = origin.span();
                origin.source().as_str() == full_source
                    && &full_source
                        [span.start().byte_offset().value()..span.end().byte_offset().value()]
                        == expected
            }),
            "missing original {expected} child"
        );
    }
    let CssImageValue::Url(url) = second.value() else {
        panic!("url symbol")
    };
    assert_eq!(url.as_str(), "#star");
    let [CssUrlModifier::Function(modifier)] = url.modifiers() else {
        panic!("one ordered URL modifier")
    };
    assert_eq!(modifier.name(), "policy");
    let [argument] = modifier.argument_components().items() else {
        panic!("one modifier argument")
    };
    let CssValueOrigin::Parsed(argument_origin) = argument.origin() else {
        panic!("original modifier argument")
    };
    assert_eq!(argument_origin.source().as_str(), full_source);
    let argument_span = argument_origin.span();
    assert_eq!(
        &full_source[argument_span.start().byte_offset().value()
            ..argument_span.end().byte_offset().value()],
        "flag"
    );
    let canonical = "counter(chapter, symbols(linear-gradient(red calc(2% + 1px), blue) url(\"#star\" policy(flag))))";
    assert_eq!(wrapper.value().serialize_specified().unwrap(), canonical);
    assert_eq!(
        wrapper
            .value()
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                128,
                256,
                canonical.len(),
            ))
            .unwrap(),
        canonical
    );
    assert_eq!(
        wrapper
            .value()
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                128,
                256,
                canonical.len() - 1,
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(
        wrapper
            .value()
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                6,
                256,
                canonical.len(),
            ))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::InputNodeLimit
    );
}

#[test]
fn two_image_symbols_share_the_outer_input_and_projection_limits() {
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
    let value = item(CssContentValueItem::Counter(counter(
        "chapter",
        Some(CssCounterStyleValue::Symbols(symbols)),
    )));
    let expected = "counter(chapter, symbols(url(\"#a\") url(\"#b\")))";
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(
                8,
                8,
                expected.len(),
            ))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(7, 8, expected.len()),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(8, 7, expected.len()),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
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
}
