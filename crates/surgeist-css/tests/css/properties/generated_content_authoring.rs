//! Generated content, lists and counters (CSS Generated Content 3, Lists 3)
//! keep authored shapes through the public parser and constructors.

use surgeist_css::*;

use crate::css::support::parse_report::CssParseReportTestExt;
use crate::css::support::sheet_cases::*;

fn content_counter_name(spelling: &str) -> CssContentCounterName {
    CssContentCounterName::try_new(CssIdent::try_new(spelling).unwrap()).unwrap()
}

fn counter_style(spelling: &str) -> CssCounterStyleValue {
    CssCounterStyleValue::try_named(CssIdent::try_new(spelling).unwrap()).unwrap()
}

#[test]
fn list_counter_and_content_models_preserve_authored_shapes() {
    let counter_name = content_counter_name("section");
    let counter_style = counter_style("chapter-style");
    let counter = CssContentCounter::new(counter_name.clone(), Some(counter_style.clone()));
    assert_eq!(counter.name(), &counter_name);
    assert_eq!(counter.style(), Some(&counter_style));

    let separator = CssContentString::try_new(".").unwrap();
    let counters =
        CssContentCounters::new(counter_name.clone(), separator.clone(), Some(counter_style));
    assert_eq!(counters.name(), &counter_name);
    assert_eq!(counters.separator(), &separator);
    assert!(counters.style().is_some());

    let content_list = CssGeneratedContent::try_new(
        vec![
            CssContentValueItem::String(CssContentString::try_new("Chapter ").unwrap()),
            CssContentValueItem::Counter(counter),
            CssContentValueItem::Counters(counters),
            CssContentValueItem::Contents,
            CssContentValueItem::Image(
                CssImage::try_new(CssImageValue::Url(CssUrl::new("marker.svg"))).unwrap(),
            ),
            CssContentValueItem::OpenQuote,
            CssContentValueItem::CloseQuote,
            CssContentValueItem::NoOpenQuote,
            CssContentValueItem::NoCloseQuote,
        ],
        None,
    )
    .unwrap();
    assert_eq!(content_list.items().len(), 9);
    assert!(content_list.alternative().is_none());
    let [
        CssContentValueItem::String(prefix),
        CssContentValueItem::Counter(counter),
        CssContentValueItem::Counters(counters),
        CssContentValueItem::Contents,
        CssContentValueItem::Image(image),
        CssContentValueItem::OpenQuote,
        CssContentValueItem::CloseQuote,
        CssContentValueItem::NoOpenQuote,
        CssContentValueItem::NoCloseQuote,
    ] = content_list.items()
    else {
        panic!("expected authored generated content order");
    };
    assert_eq!(prefix.as_str(), "Chapter ");
    assert_eq!(counter.name(), &counter_name);
    assert_eq!(counters.separator(), &separator);
    let CssImageValue::Url(url) = image.value() else {
        panic!("expected generated content image URL");
    };
    assert_eq!(url.as_str(), "marker.svg");
    let list_style = CssListStyleValue::try_new(
        Some(CssListStyleTypeValue::String(
            CssContentString::try_new("*").unwrap(),
        )),
        Some(CssListStylePosition::Inside),
        Some(CssImageValue::Url(CssUrl::new("bullet.svg"))),
    )
    .unwrap();
    assert!(matches!(
        list_style.style_type(),
        Some(CssListStyleTypeValue::String(_))
    ));
    assert_eq!(list_style.position(), Some(CssListStylePosition::Inside));
    assert!(matches!(list_style.image(), Some(CssImageValue::Url(_))));
    let counter_change = CssCounterChangeValue::new(
        counter_name.clone(),
        Some(CssIntegerValue::Literal(CssIntegerLiteral::from_i32(4))),
    );
    assert_eq!(counter_change.name(), &counter_name);
    assert_integer_literal(counter_change.value().unwrap(), "4");
    let changes = CssCounterChangesValue::try_changes(vec![counter_change]).unwrap();
    assert_eq!(changes.changes().unwrap().len(), 1);
    assert_eq!(changes.changes().unwrap()[0].name(), &counter_name);
}

#[test]
fn list_counter_and_content_constructors_reject_invalid_states() {
    assert_eq!(CssContentString::try_new("bad\0string"), None);
    assert_eq!(CssGeneratedContent::try_new(Vec::new(), None), None);
    assert_eq!(CssCounterChangesValue::try_changes(Vec::new()), None);
    assert_eq!(CssListStyleValue::try_new(None, None, None), None);

    for name in [
        "",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
        "none",
    ] {
        assert_eq!(
            CssIdent::try_new(name)
                .ok()
                .and_then(CssContentCounterName::try_new),
            None,
            "{name:?} rejected"
        );
    }
    assert_eq!(content_counter_name("list-item").as_str(), "list-item");
}

#[test]
fn counter_style_name_constructor_uses_decoded_counter_style_ident_rules() {
    assert_eq!(
        CssCounterStyleName::try_new("chapter-style")
            .unwrap()
            .as_str(),
        "chapter-style"
    );
    assert_eq!(
        CssCounterStyleName::try_new("auto").unwrap().as_str(),
        "auto"
    );
    assert_eq!(
        CssCounterStyleName::try_new("span").unwrap().as_str(),
        "span"
    );
    assert_eq!(
        CssCounterStyleName::try_new("UPPER-ROMAN")
            .unwrap()
            .as_str(),
        "upper-roman"
    );

    for name in [
        "1chapter",
        "-1chapter",
        "chapter style",
        "chapter;",
        ".chapter",
    ] {
        assert_eq!(
            CssCounterStyleName::try_new(name).unwrap().as_str(),
            name,
            "{name:?} retains decoded identity"
        );
    }

    for name in [
        "",
        "a\0b",
        "default",
        "none",
        "inherit",
        "initial",
        "unset",
        "revert",
        "revert-layer",
    ] {
        assert_eq!(
            CssCounterStyleName::try_new(name),
            None,
            "{name:?} rejected"
        );
    }
}

#[test]
fn parses_generated_content_values_symbolically() {
    let cases = [
        ("content normal", "normal", CssContentValue::Normal),
        ("content none", "none", CssContentValue::None),
        (
            "content string",
            "\"Chapter \"",
            CssContentValue::Generated(
                CssGeneratedContent::try_new(
                    vec![CssContentValueItem::String(
                        CssContentString::try_new("Chapter ").unwrap(),
                    )],
                    None,
                )
                .unwrap(),
            ),
        ),
        (
            "content url",
            "url(marker.svg)",
            CssContentValue::Generated(
                CssGeneratedContent::try_new(
                    vec![CssContentValueItem::Image(
                        CssImage::try_new(CssImageValue::Url(CssUrl::new("marker.svg"))).unwrap(),
                    )],
                    None,
                )
                .unwrap(),
            ),
        ),
        (
            "content counter",
            "counter(section)",
            CssContentValue::Generated(
                CssGeneratedContent::try_new(
                    vec![CssContentValueItem::Counter(CssContentCounter::new(
                        content_counter_name("section"),
                        None,
                    ))],
                    None,
                )
                .unwrap(),
            ),
        ),
        (
            "content counter with style",
            "counter(section, upper-roman)",
            CssContentValue::Generated(
                CssGeneratedContent::try_new(
                    vec![CssContentValueItem::Counter(CssContentCounter::new(
                        content_counter_name("section"),
                        Some(counter_style("upper-roman")),
                    ))],
                    None,
                )
                .unwrap(),
            ),
        ),
        (
            "content counters",
            "counters(section, \".\")",
            CssContentValue::Generated(
                CssGeneratedContent::try_new(
                    vec![CssContentValueItem::Counters(CssContentCounters::new(
                        content_counter_name("section"),
                        CssContentString::try_new(".").unwrap(),
                        None,
                    ))],
                    None,
                )
                .unwrap(),
            ),
        ),
        (
            "content counters with style",
            "counters(section, \".\", lower-alpha)",
            CssContentValue::Generated(
                CssGeneratedContent::try_new(
                    vec![CssContentValueItem::Counters(CssContentCounters::new(
                        content_counter_name("section"),
                        CssContentString::try_new(".").unwrap(),
                        Some(counter_style("lower-alpha")),
                    ))],
                    None,
                )
                .unwrap(),
            ),
        ),
        (
            "content quote keywords",
            "open-quote close-quote no-open-quote no-close-quote",
            CssContentValue::Generated(
                CssGeneratedContent::try_new(
                    vec![
                        CssContentValueItem::OpenQuote,
                        CssContentValueItem::CloseQuote,
                        CssContentValueItem::NoOpenQuote,
                        CssContentValueItem::NoCloseQuote,
                    ],
                    None,
                )
                .unwrap(),
            ),
        ),
    ];

    for (label, authored_value, expected_value) in cases {
        let actual = property_payload!(
            parse_single_declaration("content", authored_value),
            Content,
            value
        );
        assert_eq!(actual, expected_value, "{label}");
    }
}

#[test]
fn content_attribute_reference_retains_pending_authored_components() {
    for authored in [
        "attr(data-label)",
        "target-counter(attr(href), page)",
        "\"Chapter \" counter(section, upper-roman) counters(item, \".\", lower-alpha) attr(data-label) open-quote",
    ] {
        let input = format!(".chapter::before {{ content: {authored}; }}");
        let declaration = single_declaration(&input);
        assert_eq!(
            declaration.known().map(|known| known.property()),
            Some(CssKnownProperty::Content)
        );
        let pending = declaration
            .known()
            .unwrap()
            .substitution_dependent()
            .expect("valid attr() remains pending until attribute substitution");
        assert_eq!(pending.as_css(), authored);
        assert!(declaration.known().unwrap().property_value().is_none());
        let components = declaration.value_components();
        assert!(!components.items().is_empty());
        assert_eq!(components.serialize().unwrap().as_css().trim(), authored);
        assert!(components.items().iter().all(|component| matches!(
            component.origin(),
            CssValueOrigin::Parsed(origin) if origin.source().as_str() == input.as_str()
        )));
    }
}

#[test]
fn parses_list_style_longhands_and_shorthand_symbolically() {
    assert_eq!(
        property_payload!(
            parse_single_declaration("list-style-type", "square"),
            ListStyleType,
            value
        ),
        CssListStyleTypeValue::CounterStyle(counter_style("square"))
    );
    assert_eq!(
        property_payload!(
            parse_single_declaration("list-style-type", "custom-counter"),
            ListStyleType,
            value
        ),
        CssListStyleTypeValue::CounterStyle(counter_style("custom-counter"))
    );
    assert_eq!(
        property_payload!(
            parse_single_declaration("list-style-type", "\"*\""),
            ListStyleType,
            value
        ),
        CssListStyleTypeValue::String(CssContentString::try_new("*").unwrap(),)
    );
    assert_eq!(
        property_payload!(
            parse_single_declaration("list-style-position", "inside"),
            ListStylePosition,
            value
        ),
        CssListStylePosition::Inside
    );
    assert_eq!(
        property_payload!(
            parse_single_declaration("list-style-image", "url(marker.svg)"),
            ListStyleImage,
            value
        ),
        CssImageValue::Url(CssUrl::new("marker.svg"),)
    );

    let list_style = property_payload!(
        parse_single_declaration("list-style", "url(marker.svg) inside square"),
        ListStyle,
        value
    );
    assert_eq!(
        list_style.style_type(),
        Some(&CssListStyleTypeValue::CounterStyle(counter_style(
            "square"
        )))
    );
    assert_eq!(list_style.position(), Some(CssListStylePosition::Inside));
    assert_eq!(
        list_style.image(),
        Some(&CssImageValue::Url(CssUrl::new("marker.svg")))
    );

    let list_style = property_payload!(
        parse_single_declaration("list-style", "none inside"),
        ListStyle,
        value
    );
    assert_eq!(list_style.style_type(), Some(&CssListStyleTypeValue::None));
    assert_eq!(list_style.image(), Some(&CssImageValue::None));
    assert_eq!(list_style.position(), Some(CssListStylePosition::Inside));

    let list_style = property_payload!(
        parse_single_declaration("list-style", "none"),
        ListStyle,
        value
    );
    assert_eq!(list_style.style_type(), Some(&CssListStyleTypeValue::None));
    assert_eq!(list_style.image(), Some(&CssImageValue::None));
    assert_eq!(list_style.position(), None);

    for authored_value in ["square none", "none square"] {
        let list_style = property_payload!(
            parse_single_declaration("list-style", authored_value),
            ListStyle,
            value
        );
        assert_eq!(
            list_style.style_type(),
            Some(&CssListStyleTypeValue::CounterStyle(counter_style(
                "square"
            ))),
            "{authored_value}"
        );
        assert_eq!(
            list_style.image(),
            Some(&CssImageValue::None),
            "{authored_value}"
        );
        assert_eq!(list_style.position(), None, "{authored_value}");
    }

    for authored_value in ["url(marker.svg) none", "none url(marker.svg)"] {
        let list_style = property_payload!(
            parse_single_declaration("list-style", authored_value),
            ListStyle,
            value
        );
        assert_eq!(
            list_style.style_type(),
            Some(&CssListStyleTypeValue::None),
            "{authored_value}"
        );
        assert_eq!(
            list_style.image(),
            Some(&CssImageValue::Url(CssUrl::new("marker.svg"))),
            "{authored_value}"
        );
        assert_eq!(list_style.position(), None, "{authored_value}");
    }
}

#[test]
fn parses_counter_change_values_symbolically() {
    assert_eq!(
        property_payload!(
            parse_single_declaration("counter-reset", "none"),
            CounterReset,
            value
        ),
        CssCounterChangesValue::none()
    );

    for property in ["counter-reset", "counter-increment", "counter-set"] {
        let value = match property {
            "counter-reset" => {
                property_payload!(
                    parse_single_declaration("counter-reset", "section 2 page -1 item"),
                    CounterReset,
                    value
                )
            }
            "counter-increment" => property_payload!(
                parse_single_declaration("counter-increment", "section 2 page -1 item"),
                CounterIncrement,
                value
            ),
            "counter-set" => {
                property_payload!(
                    parse_single_declaration("counter-set", "section 2 page -1 item"),
                    CounterSet,
                    value
                )
            }
            _ => unreachable!(),
        };
        let changes = value.changes().expect("nonempty counter changes");
        assert_eq!(changes.len(), 3, "{property}");
        assert_eq!(changes[0].name().as_str(), "section");
        assert_integer_literal(changes[0].value().unwrap(), "2");
        assert_eq!(changes[1].name().as_str(), "page");
        assert_integer_literal(changes[1].value().unwrap(), "-1");
        assert_eq!(changes[2].name().as_str(), "item");
        assert_eq!(changes[2].value(), None);
    }
}

#[test]
fn public_api_exposes_generated_content_list_style_and_counter_values() {
    let sheet = parse_sheet(
        r#"
            .chapter::before {
                content: "Chapter " counter(section, upper-roman) counters(item, ".", lower-alpha) open-quote;
                list-style: url(marker.svg) inside square;
                counter-reset: section 1 page -1 item;
            }
        "#,
    )
    .expect("clean parse report");

    let style = style_rule(&sheet.rules()[0]);
    let content = style
        .declarations()
        .iter()
        .find(|declaration| {
            declaration.known().map(|known| known.property()) == Some(CssKnownProperty::Content)
        })
        .unwrap();
    let CssContentValue::Generated(content_list) = property_payload!(*content, Content, value)
    else {
        panic!("expected content item list");
    };
    let [
        CssContentValueItem::String(prefix),
        CssContentValueItem::Counter(counter),
        CssContentValueItem::Counters(counters),
        CssContentValueItem::OpenQuote,
    ] = content_list.items()
    else {
        panic!("expected inspectable generated content items");
    };
    assert_eq!(prefix.as_str(), "Chapter ");
    assert_eq!(counter.name().as_str(), "section");
    assert_eq!(counter.style(), Some(&counter_style("upper-roman")));
    assert_eq!(counters.name().as_str(), "item");
    assert_eq!(counters.separator().as_str(), ".");
    assert_eq!(counters.style(), Some(&counter_style("lower-alpha")));

    let CssSelector::Compound(selector) = style.selectors().selectors()[0].selector() else {
        panic!("expected compound pseudo-element selector");
    };
    assert_pseudo_elements(
        selector.pseudo_elements().unwrap(),
        &[CssPseudoElement::Before],
    );

    let list_style = style
        .declarations()
        .iter()
        .find(|declaration| {
            declaration.known().map(|known| known.property()) == Some(CssKnownProperty::ListStyle)
        })
        .unwrap();
    let list_style = property_payload!(*list_style, ListStyle, value);
    assert_eq!(
        list_style.style_type(),
        Some(&CssListStyleTypeValue::CounterStyle(counter_style(
            "square"
        )))
    );
    assert_eq!(list_style.position(), Some(CssListStylePosition::Inside));
    let Some(CssImageValue::Url(marker)) = list_style.image() else {
        panic!("expected marker URL slot");
    };
    assert_eq!(marker.as_str(), "marker.svg");

    let counter_reset = style
        .declarations()
        .iter()
        .find(|declaration| {
            declaration.known().map(|known| known.property())
                == Some(CssKnownProperty::CounterReset)
        })
        .unwrap();
    let changes = property_payload!(*counter_reset, CounterReset, value);
    let [section, page, item] = changes.changes().unwrap() else {
        panic!("expected three counter changes");
    };
    assert_eq!(section.name().as_str(), "section");
    assert_integer_literal(section.value().unwrap(), "1");
    assert_eq!(page.name().as_str(), "page");
    assert_integer_literal(page.value().unwrap(), "-1");
    assert_eq!(item.name().as_str(), "item");
    assert_eq!(item.value(), None);
}

#[test]
fn rejects_invalid_generated_content_and_list_counter_forms() {
    for (property_name, authored_value) in [
        ("content", "normal \"x\""),
        ("content", "counter()"),
        ("content", "counters(item)"),
        ("content", "attr()"),
        ("list-style-position", "center"),
        ("list-style-image", "red"),
        ("list-style-type", "symbols(numeric \"*\")"),
        ("list-style", "inside outside square"),
        ("list-style", "none none square url(a)"),
        ("counter-reset", "none item"),
        ("counter-increment", "1"),
        ("counter-set", "inherit 1"),
    ] {
        RejectedDeclarationCase {
            label: authored_value,
            property_name,
            authored_value,
            expected_error: ExpectedErrorKind::InvalidPropertyValue {
                property: property_name,
            },
            property_name_should_be_recognized: true,
        }
        .assert_rejects();
    }
}

#[test]
fn content_accepts_generated_lists_replacement_and_symbols_style() {
    for (authored, expected) in [
        ("\"x\" / \"alt\"", "\"x\" / \"alt\""),
        ("contents", "contents"),
        ("linear-gradient(red, blue)", "linear-gradient(red, blue)"),
        (
            "counter(item, symbols(cyclic \"*\" \"+\"))",
            "counter(item, symbols(cyclic \"*\" \"+\"))",
        ),
    ] {
        let declaration = parse_single_declaration("content", authored);
        let CssKnownPropertyValueRef::Content(wrapper) = declaration
            .known()
            .and_then(|known| known.property_value())
            .expect("typed content value")
        else {
            panic!("expected typed content for {authored}");
        };
        let CssContentValue::Generated(generated) = wrapper.value() else {
            panic!("expected generated content for {authored}");
        };
        match authored {
            "\"x\" / \"alt\"" => {
                let [CssContentValueItem::String(text)] = generated.items() else {
                    panic!("expected single generated string");
                };
                assert_eq!(text.as_str(), "x");
                let [CssContentAlternativeItem::String(alternative)] =
                    generated.alternative().unwrap().items()
                else {
                    panic!("expected single alternative string");
                };
                assert_eq!(alternative.as_str(), "alt");
            }
            "contents" => {
                assert_eq!(generated.items(), &[CssContentValueItem::Contents]);
                assert!(generated.alternative().is_none());
            }
            "linear-gradient(red, blue)" => {
                let CssGeneratedContentBodyRef::Replacement(image) = generated.body() else {
                    panic!("expected sole image replacement");
                };
                let CssImageValue::Gradient(CssGradient::Linear(gradient)) = image.value() else {
                    panic!("expected linear replacement gradient");
                };
                assert!(gradient.direction().is_none());
                let [
                    CssColorStopListItem::Stop(red),
                    CssColorStopListItem::Stop(blue),
                ] = gradient.stops().items()
                else {
                    panic!("expected two ordered replacement colors");
                };
                assert_eq!(red.color().named().unwrap().name(), "red");
                assert_eq!(blue.color().named().unwrap().name(), "blue");
                assert!(red.position().is_none());
                assert!(blue.position().is_none());
                assert!(generated.alternative().is_none());
            }
            _ => {
                let [CssContentValueItem::Counter(counter)] = generated.items() else {
                    panic!("expected single generated counter");
                };
                assert_eq!(counter.name().as_str(), "item");
                let Some(CssCounterStyleValue::Symbols(style)) = counter.style() else {
                    panic!("expected symbols counter style");
                };
                assert_eq!(style.system(), Some(CssSymbolsSystem::Cyclic));
                let [
                    CssCounterSymbolValue::String(first),
                    CssCounterSymbolValue::String(second),
                ] = style.symbols()
                else {
                    panic!("expected ordered counter symbols");
                };
                assert_eq!(first.as_str(), "*");
                assert_eq!(second.as_str(), "+");
                assert!(generated.alternative().is_none());
            }
        }
        assert_eq!(wrapper.value().serialize_specified().unwrap(), expected);
    }
}
