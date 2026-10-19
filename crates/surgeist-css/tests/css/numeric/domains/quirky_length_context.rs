#![forbid(unsafe_code)]
//! Values4 WD20240312 Appendix C. Independent number-token -> px expectations.
//! Unitless admission composes with the selected named shorthand assignment mode.
use surgeist_css::*;

const STANDARDS: CssParserContext = CssParserContext::new(CssParserMode::Standards);
const QUIRKS: CssParserContext = CssParserContext::new(CssParserMode::Quirks);

// Exactly the Appendix C examples/eligible names, with independently authored
// stimuli and px results. This is a specialized admission oracle, not metadata.
const ELIGIBLE: &[(&str, &str, &str)] = &[
    ("background-position", "7 8", "7px 8px"),
    ("border-spacing", "7 8", "7px 8px"),
    ("border-top-width", "7", "7px"),
    ("border-right-width", "7", "7px"),
    ("border-bottom-width", "7", "7px"),
    ("border-left-width", "7", "7px"),
    ("border-width", "7", "7px"),
    ("bottom", "7", "7px"),
    ("clip", "rect(1, 2, 3, 4)", "rect(1px, 2px, 3px, 4px)"),
    ("font-size", "7", "7px"),
    ("height", "7", "7px"),
    ("left", "7", "7px"),
    ("letter-spacing", "7", "7px"),
    ("margin-right", "7", "7px"),
    ("margin-left", "7", "7px"),
    ("margin-top", "7", "7px"),
    ("margin-bottom", "7", "7px"),
    ("margin", "7", "7px"),
    ("max-height", "7", "7px"),
    ("max-width", "7", "7px"),
    ("min-height", "7", "7px"),
    ("min-width", "7", "7px"),
    ("padding-top", "7", "7px"),
    ("padding-right", "7", "7px"),
    ("padding-bottom", "7", "7px"),
    ("padding-left", "7", "7px"),
    ("padding", "7", "7px"),
    ("right", "7", "7px"),
    ("text-indent", "7", "7px"),
    ("top", "7", "7px"),
    ("vertical-align", "7", "7px"),
    ("width", "7", "7px"),
    ("word-spacing", "7", "7px"),
];

fn property(name: &str) -> CssKnownProperty {
    CssKnownProperty::from_name(name).unwrap()
}
fn checked(context: CssParserContext, name: &str, text: &str) -> CssDeclaration {
    context
        .parse_property_value_for_grammar(
            property(name).grammar(),
            parse_component_values(text).unwrap(),
            CssImportance::Important,
        )
        .unwrap()
}
fn rejected(context: CssParserContext, name: &str, text: &str) {
    assert!(
        context
            .parse_property_value_for_grammar(
                property(name).grammar(),
                parse_component_values(text).unwrap(),
                CssImportance::Normal,
            )
            .is_err(),
        "{name}: {text}"
    );
    let css = format!("{name}:{text};color:red");
    let report = context.parse_style_attribute(&css);
    assert_eq!(
        report.syntax().len(),
        1,
        "{css}: {:?}",
        report.diagnostics()
    );
    assert_eq!(
        report.syntax()[0].known().unwrap().property(),
        CssKnownProperty::Color
    );
    assert!(!report.is_clean());
}
fn supports_declaration(condition: &CssSupportsCondition) -> &CssSupportsDeclaration {
    let CssSupportsConditionKind::Declaration(value) = condition.kind() else {
        panic!("declaration condition")
    };
    value
}

// Logical1 §4.7 assignments for the three Appendix C named shorthands only.
fn logical_targets(name: &str) -> [CssKnownProperty; 4] {
    match name {
        "margin" => [
            CssKnownProperty::MarginBlockStart,
            CssKnownProperty::MarginInlineStart,
            CssKnownProperty::MarginBlockEnd,
            CssKnownProperty::MarginInlineEnd,
        ],
        "padding" => [
            CssKnownProperty::PaddingBlockStart,
            CssKnownProperty::PaddingInlineStart,
            CssKnownProperty::PaddingBlockEnd,
            CssKnownProperty::PaddingInlineEnd,
        ],
        "border-width" => [
            CssKnownProperty::BorderBlockStartWidth,
            CssKnownProperty::BorderInlineStartWidth,
            CssKnownProperty::BorderBlockEndWidth,
            CssKnownProperty::BorderInlineEndWidth,
        ],
        _ => panic!("selected shorthand"),
    }
}
fn logical_value_css(item: &CssLonghandContribution) -> String {
    match item.ordinary_value().unwrap().view() {
        CssLonghandValueRef::MarginBlockStart(v)
        | CssLonghandValueRef::MarginInlineStart(v)
        | CssLonghandValueRef::MarginBlockEnd(v)
        | CssLonghandValueRef::MarginInlineEnd(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::PaddingBlockStart(v)
        | CssLonghandValueRef::PaddingInlineStart(v)
        | CssLonghandValueRef::PaddingBlockEnd(v)
        | CssLonghandValueRef::PaddingInlineEnd(v) => v.serialize_specified().unwrap(),
        CssLonghandValueRef::BorderBlockStartWidth(v)
        | CssLonghandValueRef::BorderInlineStartWidth(v)
        | CssLonghandValueRef::BorderBlockEndWidth(v)
        | CssLonghandValueRef::BorderInlineEndWidth(v) => v.serialize_specified().unwrap(),
        _ => panic!("logical terminal, never a complementary physical target"),
    }
}
fn logical_numeric_origin(item: &CssLonghandContribution) -> &CssValueOrigin {
    match item.ordinary_value().unwrap().view() {
        CssLonghandValueRef::MarginBlockStart(v)
        | CssLonghandValueRef::MarginInlineStart(v)
        | CssLonghandValueRef::MarginBlockEnd(v)
        | CssLonghandValueRef::MarginInlineEnd(v) => {
            let CssMarginValue::LengthPercentage(v) = v else {
                panic!("numeric margin")
            };
            v.origin()
        }
        CssLonghandValueRef::PaddingBlockStart(v)
        | CssLonghandValueRef::PaddingInlineStart(v)
        | CssLonghandValueRef::PaddingBlockEnd(v)
        | CssLonghandValueRef::PaddingInlineEnd(v) => v.origin(),
        CssLonghandValueRef::BorderBlockStartWidth(v)
        | CssLonghandValueRef::BorderInlineStartWidth(v)
        | CssLonghandValueRef::BorderBlockEndWidth(v)
        | CssLonghandValueRef::BorderInlineEndWidth(v) => {
            let CssBorderWidth::Length(v) = v else {
                panic!("numeric border width")
            };
            v.origin()
        }
        _ => panic!("logical numeric terminal"),
    }
}

#[test]
fn listed_number_tokens_are_lengths_only_in_document_quirks_mode() {
    for &(name, text, px) in ELIGIBLE {
        rejected(STANDARDS, name, text);
        let css = format!("{name}:{text}!important");
        let list = QUIRKS.parse_style_attribute(&css);
        assert!(list.is_clean(), "{css}: {:?}", list.diagnostics());
        let one = QUIRKS.parse_declaration(&css);
        assert!(one.is_clean());
        let p = property(name);
        let raw = parse_component_values(text).unwrap();
        let raw_before = raw.clone();
        let text_name = QUIRKS.parse_property_value_text(
            text,
            CssPropertyNameRef::Known(p),
            CssImportance::Important,
        );
        let text_grammar = QUIRKS.parse_property_value_text_for_grammar(
            text,
            p.grammar(),
            CssImportance::Important,
        );
        assert!(text_name.is_clean() && text_grammar.is_clean());
        for value in [
            list.syntax()[0].clone(),
            one.syntax().as_ref().unwrap().clone(),
            QUIRKS
                .parse_property_value(
                    CssPropertyNameRef::Known(p),
                    raw.clone(),
                    CssImportance::Important,
                )
                .unwrap(),
            QUIRKS
                .parse_property_value_for_grammar(
                    p.grammar(),
                    raw.clone(),
                    CssImportance::Important,
                )
                .unwrap(),
            text_name.syntax().as_ref().unwrap().clone(),
            text_grammar.syntax().as_ref().unwrap().clone(),
        ] {
            assert_eq!(value.parser_context(), QUIRKS);
            assert_eq!(value.importance(), CssImportance::Important);
            assert_eq!(
                value.to_specified_css().unwrap(),
                format!("{name}: {px} !important;")
            );
            assert_eq!(value.value_components().serialize().unwrap().as_css(), text);
            let emitted = value.to_specified_css().unwrap();
            assert!(parse_style_attribute(&emitted).is_clean());
        }
        assert_eq!(raw, raw_before);
    }
}

#[test]
fn fractions_exponents_signed_lengths_and_rust_authored_tokens_keep_exact_origins() {
    for (text, px) in [
        ("+7", "7px"),
        ("1.0", "1px"),
        ("1e1", "10px"),
        ("-.25", "-0.25px"),
    ] {
        let parsed = parse_component_values(text).unwrap();
        let generated =
            CssComponentValues::try_new(vec![CssComponentValue::try_number(text).unwrap()])
                .unwrap();
        for values in [parsed.clone(), generated.clone()] {
            let before = values.clone();
            assert!(
                STANDARDS
                    .parse_property_value_for_grammar(
                        CssKnownProperty::MarginTop.grammar(),
                        values.clone(),
                        CssImportance::Normal
                    )
                    .is_err()
            );
            let value = QUIRKS
                .parse_property_value_for_grammar(
                    CssKnownProperty::MarginTop.grammar(),
                    values.clone(),
                    CssImportance::Normal,
                )
                .unwrap();
            assert_eq!(value.value_components(), &values);
            let CssKnownPropertyValueRef::MarginTop(margin) =
                value.known().unwrap().property_value().unwrap()
            else {
                panic!("margin")
            };
            let CssMarginValue::LengthPercentage(length) = margin.value() else {
                panic!("length")
            };
            assert_eq!(length.origin(), values.items()[0].origin());
            assert_eq!(length.serialize_specified().unwrap(), px);
            assert_eq!(
                value.to_specified_css().unwrap(),
                format!("margin-top: {px};")
            );
            assert_eq!(values, before);
        }
        assert_eq!(generated.items()[0].origin(), &CssValueOrigin::Programmatic);
    }
    let css = "/*😀*/ MARGIN-TOP:7!important";
    let report = QUIRKS.parse_style_attribute(css);
    assert!(report.is_clean());
    let declaration = &report.syntax()[0];
    assert_eq!(
        declaration.position().unwrap().byte_offset().value(),
        "/*😀*/ ".len()
    );
    assert_eq!(
        declaration.position().unwrap().column().value() as usize,
        "/*😀*/ ".encode_utf16().count()
    );
    assert_eq!(declaration.parsed_value().unwrap().source().as_str(), css);
}

#[test]
fn expressly_listed_quads_assign_every_slot_without_complementary_writes() {
    for (text, assigned) in [
        ("1", ["1px", "1px", "1px", "1px"]),
        ("1 2", ["1px", "2px", "1px", "2px"]),
        ("1 2 3", ["1px", "2px", "3px", "2px"]),
        ("1 2 3 4", ["1px", "2px", "3px", "4px"]),
    ] {
        for name in ["margin", "padding", "border-width"] {
            let source = checked(QUIRKS, name, text);
            let actual = match source.known().unwrap().property_value().unwrap() {
                CssKnownPropertyValueRef::Margin(v) => v
                    .value()
                    .assigned_values()
                    .map(|v| v.serialize_specified().unwrap()),
                CssKnownPropertyValueRef::Padding(v) => v
                    .value()
                    .assigned_values()
                    .map(|v| v.serialize_specified().unwrap()),
                CssKnownPropertyValueRef::BorderWidth(v) => v
                    .value()
                    .assigned_values()
                    .map(|v| v.serialize_specified().unwrap()),
                _ => panic!("quad"),
            };
            assert_eq!(actual, assigned);
            let CssExpansion::Contributions(CssContributions::Longhands(items)) =
                expand_declaration(&source).unwrap()
            else {
                panic!("quad contributions")
            };
            assert_eq!(items.items().len(), 4);
            for item in items.items() {
                assert!(item.source().same_occurrence(&source));
                assert_eq!(item.source().parser_context(), QUIRKS);
            }
        }
    }
    for name in ["margin", "padding", "border-width"] {
        rejected(QUIRKS, name, "1 2 3 4 5");
    }
}

#[test]
fn logical_marker_keeps_named_shorthand_quirk_permission_and_assigns_only_logical_sides() {
    for (numbers, px, assigned) in [
        ("1", "1px", ["1px", "1px", "1px", "1px"]),
        ("1 2", "1px 2px", ["1px", "2px", "1px", "2px"]),
        ("1 2 3", "1px 2px 3px", ["1px", "2px", "3px", "2px"]),
        ("1 2 3 4", "1px 2px 3px 4px", ["1px", "2px", "3px", "4px"]),
    ] {
        for name in ["margin", "padding", "border-width"] {
            let text = format!("logical {numbers}");
            rejected(STANDARDS, name, &text);
            let css = format!("{name}:{text}!important");
            let parsed = QUIRKS.parse_style_attribute(&css);
            assert!(parsed.is_clean(), "{css}: {:?}", parsed.diagnostics());
            for source in [parsed.syntax()[0].clone(), checked(QUIRKS, name, &text)] {
                let kind = match source.known().unwrap().property_value().unwrap() {
                    CssKnownPropertyValueRef::Margin(v) => v.value().kind(),
                    CssKnownPropertyValueRef::Padding(v) => v.value().kind(),
                    CssKnownPropertyValueRef::BorderWidth(v) => v.value().kind(),
                    _ => panic!("selected shorthand"),
                };
                assert_eq!(kind, CssBoxSideKind::Logical);
                assert_eq!(source.parser_context(), QUIRKS);
                assert_eq!(
                    source.value_components().serialize().unwrap().as_css(),
                    text
                );
                assert_eq!(
                    source.to_specified_css().unwrap(),
                    format!("{name}: logical {px} !important;")
                );
                let CssExpansion::Contributions(CssContributions::Longhands(items)) =
                    expand_declaration(&source).unwrap()
                else {
                    panic!("logical assignment")
                };
                assert_eq!(items.items().len(), 4);
                for ((item, target), expected) in items
                    .items()
                    .iter()
                    .zip(logical_targets(name))
                    .zip(assigned)
                {
                    assert_eq!(item.property(), target);
                    assert_eq!(logical_value_css(item), expected);
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().importance(), CssImportance::Important);
                    assert!(item.replacement_components().is_none());
                }
            }
        }
    }
    let source = checked(QUIRKS, "margin", "logical auto -1 2% 3px");
    assert_eq!(
        source.to_specified_css().unwrap(),
        "margin: logical auto -1px 2% 3px !important;"
    );
    let CssExpansion::Contributions(CssContributions::Longhands(items)) =
        expand_declaration(&source).unwrap()
    else {
        panic!("mixed logical margins")
    };
    assert_eq!(items.items().len(), 4);
    for ((item, target), expected) in items
        .items()
        .iter()
        .zip(logical_targets("margin"))
        .zip(["auto", "-1px", "2%", "3px"])
    {
        assert_eq!(item.property(), target);
        assert_eq!(logical_value_css(item), expected);
    }
    let namespaces = CssNamespaceContext::default();
    for name in ["margin", "padding", "border-width"] {
        let condition = format!("({name}:logical 1 2)");
        assert!(
            supports_declaration(
                &QUIRKS
                    .parse_supports_condition(&condition, &namespaces)
                    .unwrap()
            )
            .known()
            .is_some()
        );
        assert!(
            supports_declaration(
                &STANDARDS
                    .parse_supports_condition(&condition, &namespaces)
                    .unwrap()
            )
            .known()
            .is_none()
        );
        assert!(
            supports_declaration(&parse_css_supports_condition(&condition).unwrap())
                .known()
                .is_none()
        );
        assert!(
            parse_css_supports_declaration(name, "logical 1 2")
                .unwrap()
                .known()
                .is_none()
        );
    }
}

#[test]
fn logical_marker_preserves_ranges_functions_arity_and_logical_property_exclusions() {
    for name in ["margin", "padding", "border-width"] {
        for context in [STANDARDS, QUIRKS] {
            let source = checked(context, name, "logical 1px 2px");
            assert_eq!(
                source.to_specified_css().unwrap(),
                format!("{name}: logical 1px 2px !important;")
            );
            for text in [
                "logical",
                "logical logical 1",
                "1 logical",
                "logical 1 2 3 4 5",
                "logical calc(7)",
                "logical calc(0)",
            ] {
                rejected(context, name, text);
            }
        }
    }
    for name in ["padding", "border-width"] {
        rejected(QUIRKS, name, "logical 1 -2");
    }
    for name in [
        "margin-block-start",
        "margin-inline-start",
        "margin-block-end",
        "margin-inline-end",
        "margin-block",
        "margin-inline",
        "padding-block-start",
        "padding-inline-start",
        "padding-block-end",
        "padding-inline-end",
        "padding-block",
        "padding-inline",
        "border-block-start-width",
        "border-inline-start-width",
        "border-block-end-width",
        "border-inline-end-width",
        "border-block-width",
        "border-inline-width",
    ] {
        for context in [STANDARDS, QUIRKS] {
            rejected(context, name, "7");
        }
    }
    for name in ["inset", "scroll-margin", "scroll-padding"] {
        rejected(QUIRKS, name, "logical 7");
    }
}

#[test]
fn pending_logical_number_replacement_selects_logical_targets_and_retains_both_origins() {
    for name in ["margin", "padding", "border-width"] {
        for pending in ["var(--length)", "env(length)", "attr(data-length *)"] {
            let source = checked(QUIRKS, name, pending);
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending shorthand")
            };
            let bad = "logical calc(7)";
            let error = handle
                .reenter(parse_component_values(bad).unwrap())
                .unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("invalid Length math")
            };
            let origin = match error.origin() {
                CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin))
                | CssSerializedOrigin::End(Some(CssValueOrigin::Parsed(origin))) => origin,
                other => panic!("replacement source: {other:?}"),
            };
            assert_eq!(origin.source().as_str(), bad);
            assert_eq!(
                handle
                    .reenter(parse_component_values("logical var(--again)").unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            let generated = CssComponentValues::try_new(vec![
                CssComponentValue::try_ident("logical").unwrap(),
                CssComponentValue::try_token(" ").unwrap(),
                CssComponentValue::try_number("1").unwrap(),
                CssComponentValue::try_token(" ").unwrap(),
                CssComponentValue::try_number("2").unwrap(),
            ])
            .unwrap();
            for replacement in [parse_component_values("logical 1 2").unwrap(), generated] {
                let replacement_before = replacement.clone();
                for _ in 0..2 {
                    let CssContributions::Longhands(items) =
                        handle.reenter(replacement.clone()).unwrap()
                    else {
                        panic!("completed logical shorthand")
                    };
                    assert_eq!(items.items().len(), 4);
                    for (((item, target), expected), token) in items
                        .items()
                        .iter()
                        .zip(logical_targets(name))
                        .zip(["1px", "2px", "1px", "2px"])
                        .zip([2, 4, 2, 4])
                    {
                        assert_eq!(item.property(), target);
                        assert_eq!(logical_value_css(item), expected);
                        assert_eq!(
                            logical_numeric_origin(item),
                            replacement.items()[token].origin()
                        );
                        assert!(item.source().same_occurrence(&source));
                        assert_eq!(item.source().parser_context(), QUIRKS);
                        assert_eq!(item.source().importance(), CssImportance::Important);
                        assert_eq!(item.replacement_components(), Some(&replacement));
                    }
                }
                assert_eq!(replacement, replacement_before);
                let standard = checked(STANDARDS, name, pending);
                let CssExpansion::Pending(handle) = expand_declaration(&standard).unwrap() else {
                    panic!("standard pending")
                };
                assert!(handle.reenter(replacement).is_err());
            }
            assert_eq!(source, before);
            assert_eq!(
                source.value_components().serialize().unwrap().as_css(),
                pending
            );
        }
    }
}

#[test]
fn background_and_spacing_offsets_admit_every_length_slot_and_keep_nonlength_parts() {
    for (name, text, expected) in [
        ("background-position", "-1 2", "-1px 2px"),
        ("background-position", "right 3 top", "right 3px top"),
        (
            "background-position",
            "bottom 4 right 3",
            "right 3px bottom 4px",
        ),
        ("background-position", "1 2, 3% 4", "1px 2px, 3% 4px"),
        ("border-spacing", "3", "3px 3px"),
        ("border-spacing", "3 4px", "3px 4px"),
        ("margin", "auto -1 2% 3px", "auto -1px 2% 3px"),
        (
            "text-indent",
            "7 hanging each-line",
            "7px hanging each-line",
        ),
    ] {
        let value = checked(QUIRKS, name, text);
        assert_eq!(
            value.to_specified_css().unwrap(),
            format!("{name}: {expected} !important;")
        );
        rejected(STANDARDS, name, text);
    }
}

#[test]
fn clip_rect_exception_admits_all_four_edges_without_relaxing_nested_math_or_shapes() {
    for (text, expected) in [
        ("rect(1, 2, 3, 4)", "rect(1px, 2px, 3px, 4px)"),
        ("rect(1 2 3 4)", "rect(1px, 2px, 3px, 4px)"),
        ("rect(auto, -2, 3px, 4)", "rect(auto, -2px, 3px, 4px)"),
        ("r\\65 ct(1, 2, 3, 4)", "rect(1px, 2px, 3px, 4px)"),
    ] {
        assert_eq!(
            checked(QUIRKS, "clip", text).to_specified_css().unwrap(),
            format!("clip: {expected} !important;")
        );
        rejected(STANDARDS, "clip", text);
    }
    for text in [
        "rect(1, 2 3, 4)",
        "rect(1, 2, 3)",
        "rect(1%, 2, 3, 4)",
        "rect(calc(1), 2, 3, 4)",
        "rect(calc(0), 2px, 3px, 4px)",
    ] {
        rejected(QUIRKS, "clip", text);
    }
    rejected(QUIRKS, "clip-path", "rect(1 2 3 4)");
}

#[test]
fn excluded_properties_references_and_function_interiors_keep_ordinary_grammar() {
    for (name, text) in [
        ("background", "7 8"),
        ("background-size", "7 8"),
        ("border", "7 solid red"),
        ("border-top", "7 solid red"),
        ("outline-width", "7"),
        ("outline", "7 solid red"),
        ("font", "7 serif"),
        ("inset", "7"),
        ("size", "7"),
        ("margin-inline-start", "7"),
        ("margin-inline", "7"),
        ("margin-block", "7"),
        ("padding-block-start", "7"),
        ("padding-inline", "7"),
        ("border-block-start-width", "7"),
        ("border-inline-width", "7"),
        ("inline-size", "7"),
        ("block-size", "7"),
        ("scroll-margin", "7"),
        ("scroll-padding", "7"),
        ("transform", "translate(7, 8)"),
        ("shape-outside", "circle(7)"),
    ] {
        for context in [STANDARDS, QUIRKS] {
            rejected(context, name, text);
        }
    }
    for name in ["margin-top", "width", "font-size", "background-position"] {
        for text in ["calc(7)", "calc(0)", "min(7, 8)", "calc(7 + 1px)"] {
            for context in [STANDARDS, QUIRKS] {
                rejected(context, name, text);
            }
        }
    }
}

#[test]
fn ordinary_zero_dimensions_percentages_numbers_and_css_wide_values_are_controls() {
    assert!(
        CssSpecifiedLength::try_from_component(CssComponentValue::try_number("7").unwrap())
            .is_err()
    );
    assert!(
        CssSpecifiedLengthPercentage::try_from_component(
            CssComponentValue::try_number("7").unwrap()
        )
        .is_err()
    );
    assert!(!parse_style_attribute("margin-top:7").is_clean());
    for context in [STANDARDS, QUIRKS] {
        for (name, text, expected) in [
            ("margin-top", "0", "0"),
            ("width", "7px", "7px"),
            ("width", "7%", "7%"),
            ("margin-top", "calc(7px + 1px)", "calc(8px)"),
            ("line-height", "7", "7"),
            ("border-image-outset", "7", "7"),
        ] {
            assert_eq!(
                checked(context, name, text).to_specified_css().unwrap(),
                format!("{name}: {expected} !important;")
            );
        }
        for text in ["inherit", "initial", "unset", "revert", "revert-layer"] {
            assert_eq!(
                checked(context, "margin", text).to_specified_css().unwrap(),
                format!("margin: {text} !important;")
            );
        }
    }
}

#[test]
fn decoded_property_spelling_selects_the_same_scoped_length_permission() {
    for source in ["MARGIN-TOP:7", r"m\61 rgin-top:7"] {
        let report = QUIRKS.parse_declaration(source);
        assert!(report.is_clean());
        assert_eq!(
            report
                .syntax()
                .as_ref()
                .unwrap()
                .to_specified_css()
                .unwrap(),
            "margin-top: 7px;"
        );
        assert!(STANDARDS.parse_declaration(source).syntax().is_none());
    }
}

#[test]
fn nonnegative_slots_and_bad_token_categories_fail_atomically_at_original_origin() {
    for name in [
        "padding",
        "border-width",
        "border-spacing",
        "width",
        "height",
        "min-width",
        "max-height",
        "font-size",
    ] {
        rejected(QUIRKS, name, "-1");
    }
    for text in ["7foo", "\"7\"", "7,", "inherit 7", "7!important"] {
        let values = parse_component_values(text).unwrap();
        let before = values.clone();
        let error = QUIRKS
            .parse_property_value_for_grammar(
                CssKnownProperty::MarginTop.grammar(),
                values.clone(),
                CssImportance::Normal,
            )
            .unwrap_err();
        let origin = match error.origin() {
            CssSerializedOrigin::Token(origin) | CssSerializedOrigin::End(Some(origin)) => origin,
            other => panic!("original token origin: {other:?}"),
        };
        assert!(matches!(origin, CssValueOrigin::Parsed(_)));
        if let CssValueOrigin::Parsed(origin) = origin {
            assert_eq!(origin.source().as_str(), text);
        }
        assert_eq!(values, before);
    }
}

#[test]
fn pending_reentry_retains_document_mode_original_and_replacement_origins_after_failure() {
    for &(name, text, _) in ELIGIBLE {
        for pending in ["var(--length)", "env(length)", "attr(data-length *)"] {
            let source = checked(QUIRKS, name, pending);
            let before = source.clone();
            let CssExpansion::Pending(handle) = expand_declaration(&source).unwrap() else {
                panic!("pending")
            };
            let bad = parse_component_values("bogus").unwrap();
            let error = handle.reenter(bad).unwrap_err();
            let CssExpansionErrorKind::InvalidReplacement(error) = error.kind() else {
                panic!("grammar error")
            };
            let origin = match error.origin() {
                CssSerializedOrigin::Token(CssValueOrigin::Parsed(origin))
                | CssSerializedOrigin::End(Some(CssValueOrigin::Parsed(origin))) => origin,
                other => panic!("replacement source: {other:?}"),
            };
            assert_eq!(origin.source().as_str(), "bogus");
            assert_eq!(origin.span().start().byte_offset().value(), 0);
            assert_eq!(
                handle
                    .reenter(parse_component_values("var(--again)").unwrap())
                    .unwrap_err()
                    .kind(),
                &CssExpansionErrorKind::ResidualSubstitution
            );
            let replacement = parse_component_values(text).unwrap();
            for _ in 0..2 {
                let CssContributions::Longhands(items) =
                    handle.reenter(replacement.clone()).unwrap()
                else {
                    panic!("ordinary completion")
                };
                for item in items.items() {
                    assert!(item.source().same_occurrence(&source));
                    assert_eq!(item.source().parser_context(), QUIRKS);
                    assert_eq!(item.source().importance(), CssImportance::Important);
                    assert_eq!(item.replacement_components(), Some(&replacement));
                }
            }
            assert_eq!(source, before);
            assert_eq!(
                source.value_components().serialize().unwrap().as_css(),
                pending
            );
            let standard = checked(STANDARDS, name, pending);
            let CssExpansion::Pending(handle) = expand_declaration(&standard).unwrap() else {
                panic!("standard pending")
            };
            assert!(handle.reenter(replacement).is_err());
        }
    }
}

#[test]
fn sheet_rule_and_block_fronts_keep_quirks_mode_and_normalized_occurrence_order() {
    let namespaces = CssNamespaceContext::default();
    let report = QUIRKS.parse_sheet(".a{margin-top:7!important;margin-top:8}.b{width:9}");
    assert!(report.is_clean());
    let before = report.clone();
    let exact = CssNormalizationLimits::try_new(1, 2, 3, 3).unwrap();
    let normalized = normalize_sheet_with_limits(report.syntax(), exact).unwrap();
    let declarations: Vec<_> = normalized
        .items()
        .iter()
        .filter_map(|item| match item {
            CssNormalizedItem::Declaration(v) => Some(v),
            _ => None,
        })
        .collect();
    assert_eq!(declarations.len(), 3);
    for (order, declaration) in declarations.iter().enumerate() {
        assert_eq!(declaration.order(), order);
        assert_eq!(declaration.source().parser_context(), QUIRKS);
    }
    assert_eq!(
        declarations[0].source().importance(),
        CssImportance::Important
    );
    assert_eq!(declarations[1].source().importance(), CssImportance::Normal);
    let error = normalize_sheet_with_limits(
        report.syntax(),
        CssNormalizationLimits::try_new(1, 2, 3, 2).unwrap(),
    )
    .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssNormalizationErrorKind::LimitExceeded {
            resource: CssNormalizationResource::Contributions,
            limit: 2
        }
    );
    assert_eq!(error.declaration_order(), Some(2));
    assert!(
        error
            .declaration()
            .unwrap()
            .same_occurrence(declarations[2].source())
    );
    assert_eq!(report, before);
    assert!(normalize_sheet_with_limits(report.syntax(), exact).is_ok());
    let rule = QUIRKS.parse_rule(".a{margin-top:7}", &namespaces);
    assert!(rule.is_clean());
    assert_eq!(
        rule.syntax().as_ref().unwrap().to_specified_css().unwrap(),
        ".a { margin-top: 7px; }"
    );
    let block = QUIRKS.parse_style_block("{margin-top:7}", &namespaces);
    assert!(block.is_clean());
    assert_eq!(
        block.syntax().as_ref().unwrap().declarations()[0].parser_context(),
        QUIRKS
    );
}

#[test]
fn authored_supports_uses_document_mode_and_both_method_overloads_force_standards() {
    let namespaces = CssNamespaceContext::default();
    for &(name, text, _) in ELIGIBLE {
        let condition_text = format!("({name}:{text})");
        let quirky = QUIRKS
            .parse_supports_condition(&condition_text, &namespaces)
            .unwrap();
        assert!(
            supports_declaration(&quirky).known().is_some(),
            "{condition_text}"
        );
        assert!(
            supports_declaration(
                &STANDARDS
                    .parse_supports_condition(&condition_text, &namespaces)
                    .unwrap()
            )
            .known()
            .is_none()
        );
        assert!(
            supports_declaration(&parse_css_supports_condition(&condition_text).unwrap())
                .known()
                .is_none()
        );
        let method = parse_css_supports_declaration(name, text).unwrap();
        assert!(method.known().is_none());
        let CssValueOrigin::Parsed(origin) = method.value_components()[0].origin() else {
            panic!("separate value source")
        };
        assert_eq!(origin.source().as_str(), text);
        let css = format!("@supports {condition_text} {{.a{{{name}:{text}}}}}");
        let report = QUIRKS.parse_sheet(&css);
        assert!(report.is_clean());
        let CssRule::Supports(rule) = &report.syntax().rules()[0] else {
            panic!("supports")
        };
        assert!(supports_declaration(rule.condition()).known().is_some());
    }
    assert!(
        parse_css_supports_declaration("margin-top", "7px")
            .unwrap()
            .known()
            .is_some()
    );
    assert!(
        supports_declaration(&parse_css_supports_condition("(margin-top:7px)").unwrap())
            .known()
            .is_some()
    );
    assert!(
        QUIRKS
            .parse_supports_condition("(margin-top:7", &namespaces)
            .is_err()
    );
}

#[test]
fn converted_length_keeps_scalar_tariff_and_shared_cumulative_atomic_budgets() {
    let source = checked(QUIRKS, "margin-top", "7");
    let before = source.clone();
    let expected = "margin-top: 7px !important;";
    // Declared composition tariff: declaration + property name + numeric leaf.
    let exact = CssSpecifiedValueSerializationLimits::new(3, 3, expected.len());
    assert_eq!(
        source.to_specified_css_with_limits(exact).unwrap(),
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
    ] {
        assert_eq!(
            source
                .to_specified_css_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(source, before);
    }
    assert_eq!(
        source.to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    let report = QUIRKS.parse_sheet(".a{margin-top:7}.b{margin-top:8}");
    assert!(report.is_clean());
    let before = report.clone();
    let expected = ".a { margin-top: 7px; }\n.b { margin-top: 8px; }";
    // Rule, selector-list, compound, class, declaration-list, three-node declaration.
    for rule in report.syntax().rules() {
        assert!(
            rule.to_specified_css_with_limits(CssSpecifiedValueSerializationLimits::new(
                8,
                8,
                expected.len()
            ))
            .is_ok()
        );
    }
    let exact = CssSpecifiedValueSerializationLimits::new(17, 17, expected.len());
    assert_eq!(
        report.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
    for limits in [
        CssSpecifiedValueSerializationLimits::new(9, 17, expected.len()),
        CssSpecifiedValueSerializationLimits::new(17, 9, expected.len()),
        CssSpecifiedValueSerializationLimits::new(17, 17, expected.len() - 1),
    ] {
        assert!(
            report
                .syntax()
                .to_specified_css_with_limits(limits)
                .is_err()
        );
        assert_eq!(report, before);
    }
    assert_eq!(
        report.syntax().to_specified_css_with_limits(exact).unwrap(),
        expected
    );
}
