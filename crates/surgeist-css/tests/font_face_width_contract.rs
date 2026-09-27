#![forbid(unsafe_code)]

//! Authored @font-face font-width contracts from CSS Fonts 4 WD 2026-09-07 §4.4.

use surgeist_css::*;

fn percent(text: &str) -> CssFontWidth {
    let values = parse_component_values(text).unwrap();
    let [component] = values.items() else {
        panic!("one percentage")
    };
    CssFontWidth::Percentage(
        CssSpecifiedNonNegativePercentage::try_from_component(component.clone()).unwrap(),
    )
}

fn raw(text: &str) -> CssFontFaceWidth {
    let report = parse_font_face_descriptor_value(text, CssFontFaceDescriptorKind::FontWidth);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let Some(CssFontFaceDescriptorValue::FontWidth(value)) = report.syntax() else {
        panic!("typed font-width descriptor: {text}")
    };
    value.clone()
}

fn sheet(name: &str, text: &str) -> (CssFontFaceWidth, CssSourcePosition) {
    let source = format!("@font-face{{font-family:Demo;src:url(face);{name}:{text}}}");
    let report = parse_sheet(&source);
    assert!(report.is_clean(), "{source}: {:?}", report.diagnostics());
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("font-face rule")
    };
    let occurrence = rule.descriptors().font_width().expect("width occurrence");
    assert_eq!(
        occurrence.position().byte_offset().value(),
        source.find(name).unwrap()
    );
    (occurrence.value().clone(), occurrence.position())
}

#[test]
fn explicit_auto_and_omission_have_distinct_authored_states() {
    assert_eq!(
        CssFontFaceDescriptorKind::FontWidth.css_name(),
        "font-width"
    );
    let auto = CssFontFaceWidth::Auto;
    assert_eq!(auto.serialize_specified().unwrap(), "auto");
    assert_eq!(raw("auto"), auto);
    for name in ["font-width", "font-stretch", "FONT-WIDTH", "FONT-STRETCH"] {
        assert_eq!(sheet(name, "auto").0, auto);
    }
    let report = parse_sheet("@font-face{font-family:Demo;src:url(face)}");
    assert!(report.is_clean());
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("font-face rule")
    };
    assert!(rule.descriptors().font_width().is_none());
}

#[test]
fn checked_ranges_preserve_mixed_values_descending_order_and_authored_arity() {
    use CssFontWidthKeyword::{Condensed, Expanded};
    let single = CssFontFaceWidth::Range {
        start: percent("75%"),
        end: None,
    };
    let repeated = CssFontFaceWidth::Range {
        start: percent("75%"),
        end: Some(percent("75%")),
    };
    assert_ne!(single, repeated);
    assert_eq!(single.serialize_specified().unwrap(), "75%");
    assert_eq!(repeated.serialize_specified().unwrap(), "75% 75%");
    assert_eq!(raw("75%"), single);
    assert_eq!(raw("75% 75%"), repeated);
    let mixed = CssFontFaceWidth::Range {
        start: CssFontWidth::Keyword(Expanded),
        end: Some(percent("62.5%")),
    };
    assert_eq!(mixed.serialize_specified().unwrap(), "expanded 62.5%");
    assert_eq!(raw("expanded 62.5%"), mixed);
    assert_eq!(sheet("font-width", "expanded 62.5%").0, mixed);

    for (text, expected) in [
        ("125% 75%", "125% 75%"),
        ("expanded condensed", "expanded condensed"),
        ("condensed 125%", "condensed 125%"),
        ("125% condensed", "125% condensed"),
    ] {
        let actual = raw(text);
        assert_eq!(actual.serialize_specified().unwrap(), expected);
        assert_eq!(sheet("font-stretch", text).0, actual);
    }
    let descending = CssFontFaceWidth::Range {
        start: CssFontWidth::Keyword(Expanded),
        end: Some(CssFontWidth::Keyword(Condensed)),
    };
    assert_eq!(raw("expanded condensed"), descending);
}

#[test]
fn raw_and_sheet_parsing_preserve_numeric_origin_and_descriptor_name_position() {
    let text = "62.500000000000000000001% 75%";
    let raw_value = raw(text);
    let (sheet_value, position) = sheet("FoNt-StReTcH", text);
    assert_eq!(sheet_value, raw_value);
    assert_eq!(
        position.byte_offset().value(),
        "@font-face{font-family:Demo;src:url(face);".len()
    );
    let CssFontFaceWidth::Range {
        start,
        end: Some(end),
    } = sheet_value
    else {
        panic!("two authored endpoints")
    };
    let (CssFontWidth::Percentage(start), CssFontWidth::Percentage(end)) = (start, end) else {
        panic!("two numeric endpoints")
    };
    for component in [&start, &end] {
        assert!(matches!(component.origin(), CssValueOrigin::Parsed(_)));
        assert!(component.literal_component().is_some());
    }
    let CssValueOrigin::Parsed(start_origin) = start.origin() else {
        panic!("start retains sheet source")
    };
    let CssValueOrigin::Parsed(end_origin) = end.origin() else {
        panic!("end retains sheet source")
    };
    let sheet_source = start_origin.source().as_str();
    assert_eq!(sheet_source, end_origin.source().as_str());
    for (origin, expected) in [
        (start_origin, "62.500000000000000000001%"),
        (end_origin, "75%"),
    ] {
        let span = origin.span();
        assert_eq!(
            &origin.source().as_str()
                [span.start().byte_offset().value()..span.end().byte_offset().value()],
            expected
        );
    }
    assert_eq!(
        start.serialize_specified().unwrap(),
        "62.500000000000000000001%"
    );
    assert_eq!(end.serialize_specified().unwrap(), "75%");
    assert_ne!(
        start,
        CssSpecifiedNonNegativePercentage::try_from_component(
            CssComponentValue::try_token("62.5%").unwrap()
        )
        .unwrap()
    );
    let programmatic = CssSpecifiedNonNegativePercentage::try_from_component(
        CssComponentValue::try_token("62.5%").unwrap(),
    )
    .unwrap();
    assert!(matches!(
        programmatic.origin(),
        CssValueOrigin::Programmatic
    ));
}

#[test]
fn percentage_math_endpoints_remain_deferred_in_raw_and_sheet_values() {
    let text = "calc(25% - 50%) 125%";
    let raw_value = raw(text);
    let (sheet_value, position) = sheet("font-stretch", text);
    assert_eq!(raw_value, sheet_value);
    assert_eq!(
        position.byte_offset().value(),
        "@font-face{font-family:Demo;src:url(face);".len()
    );
    for value in [&raw_value, &sheet_value] {
        let CssFontFaceWidth::Range {
            start,
            end: Some(end),
        } = value
        else {
            panic!("math start and literal end")
        };
        let CssFontWidth::Percentage(start) = start else {
            panic!("percentage math start")
        };
        let CssFontWidth::Percentage(end) = end else {
            panic!("percentage literal end")
        };
        assert!(start.calculation().is_some());
        assert!(start.literal_component().is_none());
        assert!(end.calculation().is_none());
        assert_eq!(end.serialize_specified().unwrap(), "125%");
        assert!(matches!(start.origin(), CssValueOrigin::Parsed(_)));
        assert!(matches!(end.origin(), CssValueOrigin::Parsed(_)));
    }
    let CssFontFaceWidth::Range {
        start: CssFontWidth::Percentage(start),
        ..
    } = &sheet_value
    else {
        panic!("math start")
    };
    let CssValueOrigin::Parsed(origin) = start.origin() else {
        panic!("source-backed math")
    };
    let span = origin.span();
    assert_eq!(
        &origin.source().as_str()
            [span.start().byte_offset().value()..span.end().byte_offset().value()],
        "calc("
    );
}

#[test]
fn aliases_share_ordered_occurrences_and_last_valid_projection() {
    let source = "@font-face{font-stretch:condensed 125%;font-width:auto;FONT-STRETCH:125% 75%;font-width:expanded condensed}";
    let report = parse_sheet(source);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let [CssRule::FontFace(rule)] = report.syntax().rules() else {
        panic!("font-face rule")
    };
    let authored = rule
        .descriptors()
        .occurrences()
        .filter_map(|item| {
            let CssFontFaceDescriptorRef::FontWidth(value) = item else {
                return None;
            };
            Some((
                value.position().byte_offset().value(),
                value.value().serialize_specified().unwrap(),
            ))
        })
        .collect::<Vec<_>>();
    assert_eq!(
        authored,
        [
            (
                source.find("font-stretch:condensed").unwrap(),
                "condensed 125%".to_string()
            ),
            (source.find("font-width:auto").unwrap(), "auto".to_string()),
            (
                source.find("FONT-STRETCH:125%").unwrap(),
                "125% 75%".to_string()
            ),
            (
                source.find("font-width:expanded").unwrap(),
                "expanded condensed".to_string()
            ),
        ]
    );
    assert_eq!(
        rule.descriptors()
            .font_width()
            .unwrap()
            .value()
            .serialize_specified()
            .unwrap(),
        "expanded condensed"
    );
}

#[test]
fn descriptor_serializer_charges_two_endpoints_under_one_budget() {
    let auto = CssFontFaceWidth::Auto;
    assert_eq!(
        auto.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(1, 1, 4))
            .unwrap(),
        "auto"
    );
    let range = CssFontFaceWidth::Range {
        start: percent("75%"),
        end: Some(percent("125%")),
    };
    assert_eq!(
        range
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 8))
            .unwrap(),
        "75% 125%"
    );
    for (limits, kind) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, 8),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, 8),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 3, 7),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            range
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
    }
}
