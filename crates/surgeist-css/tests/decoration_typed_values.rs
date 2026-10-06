#![forbid(unsafe_code)]
//! Functional public-model evidence paired with implementation. Expectations use
//! selected Decoration 4 §§2.1, 2.7–2.10, 3 and 4 and adopted CSSOM omissions.
use CssSpecifiedValueSerializationErrorKind as K;
use CssSpecifiedValueSerializationLimits as L;
use surgeist_css::*;

fn declaration(text: &str) -> CssDeclaration {
    let report = parse_style_attribute(text);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    assert_eq!(report.syntax().len(), 1);
    report.syntax()[0].clone()
}
fn length(text: &str) -> CssSpecifiedLength {
    CssSpecifiedLength::try_from_component(CssComponentValue::try_token(text).unwrap()).unwrap()
}
fn nonnegative(text: &str) -> CssSpecifiedNonNegativeLength {
    CssSpecifiedNonNegativeLength::try_from_component(CssComponentValue::try_token(text).unwrap())
        .unwrap()
}
fn signed_percentage(text: &str) -> CssSpecifiedLengthPercentage {
    CssSpecifiedLengthPercentage::try_from_component(CssComponentValue::try_token(text).unwrap())
        .unwrap()
}

#[test]
fn exclusive_error_lines_are_constructible_without_ordinary_flags() {
    for (kind, expected) in [
        (CssTextDecorationError::SpellingError, "spelling-error"),
        (CssTextDecorationError::GrammarError, "grammar-error"),
    ] {
        let line = CssTextDecorationLine::error(kind);
        assert_eq!(line.error_kind(), Some(kind));
        assert!(!line.is_none());
        assert!(line.components().is_empty());
        assert_eq!(line.serialize_specified().unwrap(), expected);
        assert_eq!(
            line.serialize_specified_with_limits(L::new(2, 2, expected.len()))
                .unwrap(),
            expected
        );
        assert_eq!(
            line.serialize_specified_with_limits(L::new(1, 2, expected.len()))
                .unwrap_err()
                .kind(),
            K::InputNodeLimit
        );
        let value =
            CssTextDecoration::try_new(Some(line), None, Some(CssTextDecorationStyle::Wavy), None)
                .unwrap();
        assert_eq!(
            value.serialize_specified().unwrap(),
            format!("{expected} wavy")
        );
    }
    assert!(CssTextDecorationLine::none().is_none());
    assert!(CssTextDecorationLine::try_new(vec![]).is_none());
    assert!(
        CssTextDecorationLine::try_new(vec![CssTextDecorationLineComponent::Blink; 2]).is_none()
    );
}

#[test]
fn underline_models_encode_separate_auto_and_nonempty_choices_and_signed_offset() {
    assert!(CssUnderlinePosition::try_new(None, None).is_none());
    let position = CssUnderlinePosition::try_new(None, Some(CssTextSide::Left)).unwrap();
    assert_eq!(position.vertical(), None);
    assert_eq!(position.side(), Some(CssTextSide::Left));
    assert_eq!(
        CssTextUnderlinePosition::Position(position)
            .serialize_specified()
            .unwrap(),
        "left"
    );
    let position = CssUnderlinePosition::try_new(
        Some(CssUnderlinePositionVertical::FromFont),
        Some(CssTextSide::Right),
    )
    .unwrap();
    assert_eq!(position.serialize_specified().unwrap(), "from-font right");
    let value = CssTextUnderlineOffsetLength::try_new(signed_percentage("-25%")).unwrap();
    assert_eq!(value.value().origin(), &CssValueOrigin::Programmatic);
    assert_eq!(
        CssTextUnderlineOffset::Length(value)
            .serialize_specified()
            .unwrap(),
        "-25%"
    );
    let source = declaration("text-underline-offset:-25%");
    let CssKnownPropertyValueRef::TextUnderlineOffset(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("offset wrapper")
    };
    let CssTextUnderlineOffset::Length(value) = value.value() else {
        panic!("signed payload")
    };
    assert!(matches!(value.value().origin(), CssValueOrigin::Parsed(_)));
    assert!(
        matches!(value.value().literal_component().unwrap().view(), CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) if number.representation() == "-25")
    );
}

#[test]
fn emphasis_mark_and_position_retain_explicit_slots_and_original_origins() {
    assert!(CssTextEmphasisMark::try_new(None, None).is_none());
    let fill_only = CssTextEmphasisMark::try_new(Some(CssTextEmphasisFill::Filled), None).unwrap();
    assert_eq!(fill_only.shape(), None);
    assert_eq!(fill_only.serialize_specified().unwrap(), "filled");
    let assumed = CssTextEmphasisMark::try_new(None, Some(CssTextEmphasisShape::Circle)).unwrap();
    assert_eq!(assumed.fill(), None);
    assert_eq!(assumed.fill_origin(), None);
    assert_eq!(assumed.effective_fill(), CssTextEmphasisFill::Filled);
    let source = declaration("text-emphasis-style:filled dot");
    let CssKnownPropertyValueRef::TextEmphasisStyle(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("style wrapper")
    };
    let CssTextEmphasisStyle::Mark(mark) = value.value() else {
        panic!("mark")
    };
    let before = mark.clone();
    assert_eq!(mark.fill(), Some(CssTextEmphasisFill::Filled));
    assert_eq!(mark.shape(), Some(CssTextEmphasisShape::Dot));
    assert!(matches!(
        mark.fill_origin(),
        Some(CssValueOrigin::Parsed(_))
    ));
    assert!(matches!(
        mark.shape_origin(),
        Some(CssValueOrigin::Parsed(_))
    ));
    assert_ne!(mark.fill_origin(), mark.shape_origin());
    assert_eq!(
        mark.serialize_specified_with_limits(L::new(2, 1, 3))
            .unwrap(),
        "dot"
    );
    assert_eq!(
        mark.serialize_specified_with_limits(L::new(1, 1, 3))
            .unwrap_err()
            .kind(),
        K::InputNodeLimit
    );
    assert_eq!(mark, &before);
    for (text, explicit) in [("over", false), ("right over", true)] {
        let source = declaration(&format!("text-emphasis-position:{text}"));
        let CssKnownPropertyValueRef::TextEmphasisPosition(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("position wrapper")
        };
        let value = value.value();
        assert_eq!(value.vertical(), CssTextEmphasisVertical::Over);
        assert_eq!(value.effective_side(), CssTextSide::Right);
        assert_eq!(value.side().is_some(), explicit);
        assert_eq!(value.side_origin().is_some(), explicit);
        assert!(matches!(value.vertical_origin(), CssValueOrigin::Parsed(_)));
        assert_eq!(value.serialize_specified().unwrap(), "over");
        assert_eq!(
            value
                .serialize_specified_with_limits(L::new(if explicit { 2 } else { 1 }, 1, 4))
                .unwrap(),
            "over"
        );
    }
}

#[test]
fn emphasis_strings_keep_decoded_content_and_provenance_without_ua_truncation() {
    for content in ["", "hello", "😀x", "á", "a\"b\\c"] {
        let value = CssTextEmphasisString::try_new(content).unwrap();
        assert_eq!(value.as_str(), content);
        assert_eq!(value.origin(), &CssValueOrigin::Programmatic);
        let component = value.component().clone();
        assert_eq!(
            CssTextEmphasisString::try_from_component(component.clone())
                .unwrap()
                .component(),
            &component
        );
        let serialized = value.serialize_specified().unwrap();
        let parsed = parse_component_values(&serialized).unwrap();
        let parsed = CssTextEmphasisString::try_from_component(parsed.items()[0].clone()).unwrap();
        assert_eq!(parsed.as_str(), content);
        assert!(matches!(parsed.origin(), CssValueOrigin::Parsed(_)));
    }
    assert!(CssTextEmphasisString::try_new("\0").is_err());
    assert!(
        CssTextEmphasisString::try_from_component(CssComponentValue::try_ident("hello").unwrap())
            .is_none()
    );
}

#[test]
fn emphasis_semantic_equality_preserves_authored_slots_and_borrows_original_provenance() {
    fn style(text: &str) -> CssTextEmphasisStyle {
        let source = declaration(text);
        let CssKnownPropertyValueRef::TextEmphasisStyle(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("style wrapper")
        };
        value.value().clone()
    }
    fn position(text: &str) -> CssTextEmphasisPosition {
        let source = declaration(text);
        let CssKnownPropertyValueRef::TextEmphasisPosition(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("position wrapper")
        };
        value.value().clone()
    }
    let left = style("text-emphasis-style:open");
    let right = style("/*😀*/text-emphasis-style:open!important");
    assert_eq!(left, right);
    let (CssTextEmphasisStyle::Mark(left), CssTextEmphasisStyle::Mark(right)) = (left, right)
    else {
        panic!("marks")
    };
    assert_ne!(left.fill_origin(), right.fill_origin());
    assert_eq!(
        left,
        CssTextEmphasisMark::try_new(Some(CssTextEmphasisFill::Open), None).unwrap()
    );
    let explicit = style("text-emphasis-style:filled dot");
    let omitted = style("text-emphasis-style:dot");
    assert_ne!(explicit, omitted);
    assert_eq!(
        explicit.serialize_specified().unwrap(),
        omitted.serialize_specified().unwrap()
    );
    assert_ne!(explicit, style("text-emphasis-style:open dot"));

    let left = position("text-emphasis-position:over right");
    let right = position("/*😀*/text-emphasis-position:right over!important");
    assert_eq!(left, right);
    assert_ne!(left.vertical_origin(), right.vertical_origin());
    assert_ne!(left.side_origin(), right.side_origin());
    assert_eq!(
        left,
        CssTextEmphasisPosition::new(CssTextEmphasisVertical::Over, Some(CssTextSide::Right))
    );
    let omitted = position("text-emphasis-position:over");
    assert_ne!(left, omitted);
    assert_eq!(
        left.serialize_specified().unwrap(),
        omitted.serialize_specified().unwrap()
    );
    assert_ne!(left, position("text-emphasis-position:under right"));
    assert_ne!(left, position("text-emphasis-position:over left"));

    let plain = style("text-emphasis-style:'hello'");
    let escaped = style(r#"/*😀*/text-emphasis-style:"\68 ello"!important"#);
    assert_eq!(plain, escaped);
    let (CssTextEmphasisStyle::String(plain), CssTextEmphasisStyle::String(escaped)) =
        (plain, escaped)
    else {
        panic!("strings")
    };
    assert_eq!(plain.as_str(), "hello");
    assert_eq!(escaped.as_str(), "hello");
    assert_ne!(
        plain, escaped,
        "the retained string leaf still compares original components"
    );
    assert_ne!(plain.origin(), escaped.origin());
    assert_ne!(plain.component(), escaped.component());
    assert_eq!(
        CssTextEmphasisStyle::String(plain),
        CssTextEmphasisStyle::String(CssTextEmphasisString::try_new("hello").unwrap())
    );
    assert_ne!(
        style("text-emphasis-style:'hello'"),
        style("text-emphasis-style:'Hello'")
    );
}

#[test]
fn underline_offset_owner_compares_exact_numeric_structure_without_erasing_child_origins() {
    fn offset(text: &str) -> CssTextUnderlineOffsetLength {
        let source = declaration(text);
        let CssKnownPropertyValueRef::TextUnderlineOffset(value) =
            source.known().unwrap().property_value().unwrap()
        else {
            panic!("offset wrapper")
        };
        let CssTextUnderlineOffset::Length(value) = value.value() else {
            panic!("length")
        };
        value.clone()
    }
    let literal = offset("text-underline-offset:-25%");
    let shifted = offset("/*😀*/text-underline-offset:-25%!important");
    assert_eq!(literal, shifted);
    assert_ne!(
        literal.value(),
        shifted.value(),
        "raw scalar equality retains provenance"
    );
    assert_ne!(literal.value().origin(), shifted.value().origin());
    assert_eq!(
        literal,
        CssTextUnderlineOffsetLength::try_new(signed_percentage("-25%")).unwrap()
    );
    for different in ["-25px", "-25.0%", "25%"] {
        assert_ne!(
            literal,
            offset(&format!("text-underline-offset:{different}")),
            "{different}"
        );
    }
    let calculation = offset("text-underline-offset:calc(1px + 2%)");
    let shifted = offset("/*😀*/text-underline-offset:CALC(1px + 2%)!important");
    assert_eq!(calculation, shifted);
    assert_ne!(
        calculation.value().calculation(),
        shifted.value().calculation()
    );
    for different in [
        "calc(1px + 2px)",
        "calc(1px - 2%)",
        "calc((1px + 2%))",
        "calc(2px + 2%)",
    ] {
        assert_ne!(
            calculation,
            offset(&format!("text-underline-offset:{different}")),
            "{different}"
        );
    }
}

#[test]
fn checked_emphasis_string_and_aggregate_reject_recovered_quotes_without_losing_browser_values() {
    for text in ["'hello", "\"hello"] {
        let components = parse_component_values(text).unwrap();
        let original = components.items()[0].clone();
        assert!(CssTextEmphasisString::try_from_component(original.clone()).is_none());
        assert!(matches!(original.origin(), CssValueOrigin::Parsed(_)));
        let report = parse_style_attribute(&format!("text-emphasis:{text}"));
        assert!(!report.is_clean());
        assert!(!report.diagnostics().is_empty());
        assert_eq!(report.syntax().len(), 1);
        let CssKnownPropertyValueRef::TextEmphasis(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("recovered emphasis wrapper")
        };
        let style = value.value().style().unwrap();
        let CssTextEmphasisStyle::String(string) = style else {
            panic!("retained recovered string")
        };
        assert_eq!(string.as_str(), "hello");
        assert!(matches!(string.origin(), CssValueOrigin::Parsed(_)));
        let before = string.component().clone();
        assert!(CssTextEmphasisString::try_from_component(before.clone()).is_none());
        assert!(CssTextEmphasis::try_new(Some(style.clone()), None).is_none());
        assert_eq!(string.component(), &before);
        assert_eq!(style.serialize_specified().unwrap(), "\"hello\"");
    }
    for text in ["'hello'", "\"hello\""] {
        let components = parse_component_values(text).unwrap();
        let string =
            CssTextEmphasisString::try_from_component(components.items()[0].clone()).unwrap();
        assert!(
            CssTextEmphasis::try_new(Some(CssTextEmphasisStyle::String(string)), None).is_some()
        );
    }
}

#[test]
fn checked_text_shadow_narrows_spread_but_reuses_retained_box_children() {
    let negative = CssShadow::try_new(
        true,
        length("-1px"),
        length("2px"),
        Some(nonnegative("3px")),
        Some(length("-4px")),
        Some(CssColor::current_color()),
    )
    .unwrap();
    assert_eq!(
        negative.serialize_specified().unwrap(),
        "currentcolor -1px 2px 3px -4px inset"
    );
    assert!(CssTextShadowLayer::try_from_shadow(negative).is_none());
    let underflow = CssShadow::try_new(
        false,
        length("0"),
        length("0"),
        Some(nonnegative("0")),
        Some(length("-1e-999px")),
        None,
    )
    .unwrap();
    assert!(CssTextShadowLayer::try_from_shadow(underflow).is_none());
    assert!(
        CssTextShadowLayer::try_new(
            false,
            length("1px"),
            length("2px"),
            None,
            Some(nonnegative("0")),
            None
        )
        .is_none()
    );
    let layer = CssTextShadowLayer::try_new(
        true,
        length("-1px"),
        length("2px"),
        Some(nonnegative("3px")),
        Some(nonnegative("4px")),
        Some(CssColor::current_color()),
    )
    .unwrap();
    assert!(layer.shadow().inset());
    assert!(layer.shadow().color().unwrap().is_current_color());
    let expected = "currentcolor -1px 2px 3px 4px inset";
    assert_eq!(layer.serialize_specified().unwrap(), expected);
    assert_eq!(
        layer
            .serialize_specified_with_limits(L::new(7, 7, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (L::new(6, 7, expected.len()), K::InputNodeLimit),
        (L::new(7, 6, expected.len()), K::ProjectionNodeLimit),
        (L::new(7, 7, expected.len() - 1), K::ByteLimit),
    ] {
        assert_eq!(
            layer
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(layer.serialize_specified().unwrap(), expected);
    }
    assert!(CssTextShadowList::try_new(vec![]).is_none());
    let list = CssTextShadowList::try_new(vec![layer.clone(), layer.clone()]).unwrap();
    assert_eq!(list.shadows(), &[layer.clone(), layer]);
    let expected = format!("{expected}, {expected}");
    let value = CssTextShadow::Shadows(list);
    let before = value.clone();
    assert_eq!(
        value
            .serialize_specified_with_limits(L::new(15, 15, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (L::new(14, 15, expected.len()), K::InputNodeLimit),
        (L::new(15, 14, expected.len()), K::ProjectionNodeLimit),
        (L::new(15, 15, expected.len() - 1), K::ByteLimit),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, before);
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
    let source = declaration("text-shadow:1px 2px calc(-1px) calc(-2px) inset");
    let CssKnownPropertyValueRef::TextShadow(value) =
        source.known().unwrap().property_value().unwrap()
    else {
        panic!("shadow wrapper")
    };
    let CssTextShadow::Shadows(list) = value.value() else {
        panic!("list")
    };
    assert!(CssTextShadowLayer::try_from_shadow(list.shadows()[0].shadow().clone()).is_some());
}

#[test]
fn checked_aggregates_reject_recovered_numeric_and_nested_color_graphs() {
    let report = parse_style_attribute("text-decoration-thickness:calc(-1px");
    assert!(!report.is_clean());
    let CssKnownPropertyValueRef::TextDecorationThickness(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("thickness")
    };
    let CssTextDecorationThickness::Length(value) = value.value() else {
        panic!("length")
    };
    assert!(CssTextUnderlineOffsetLength::try_new(value.clone()).is_none());
    let report = parse_style_attribute("text-decoration-color:light-dark(red,rgb(calc(1 + 2),0,0");
    assert!(!report.is_clean());
    let CssKnownPropertyValueRef::TextDecorationColor(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("nested color")
    };
    let color = value.value().clone();
    assert!(CssTextEmphasis::try_new(None, Some(color.clone())).is_none());
    assert!(
        CssTextShadowLayer::try_new(false, length("1px"), length("2px"), None, None, Some(color))
            .is_none()
    );
    let report = parse_style_attribute("text-shadow:calc(1px");
    assert!(!report.is_clean());
    // The malformed missing-offset occurrence is rejected atomically.
    assert!(report.syntax().is_empty());
    let report = parse_style_attribute("text-shadow:1px calc(2px");
    assert!(!report.is_clean());
    let CssKnownPropertyValueRef::TextShadow(value) = report.syntax()[0]
        .known()
        .unwrap()
        .property_value()
        .unwrap()
    else {
        panic!("recovered layer")
    };
    let CssTextShadow::Shadows(list) = value.value() else {
        panic!("list")
    };
    assert!(CssTextShadowLayer::try_from_shadow(list.shadows()[0].shadow().clone()).is_none());
    assert!(CssTextShadowList::try_new(list.shadows().to_vec()).is_none());
    let offset = list.shadows()[0].shadow().offset_y().clone();
    assert!(CssTextShadowLayer::try_new(false, offset, length("2px"), None, None, None).is_none());
    for source in [
        "text-shadow:1px 2px calc(3px",
        "text-shadow:1px 2px 0 calc(4px",
    ] {
        let report = parse_style_attribute(source);
        assert!(!report.is_clean());
        let CssKnownPropertyValueRef::TextShadow(value) = report.syntax()[0]
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("recovered numeric slot")
        };
        let CssTextShadow::Shadows(list) = value.value() else {
            panic!("list")
        };
        let shadow = list.shadows()[0].shadow();
        assert!(CssTextShadowLayer::try_from_shadow(shadow.clone()).is_none());
        assert!(CssTextShadowList::try_new(list.shadows().to_vec()).is_none());
    }
}
