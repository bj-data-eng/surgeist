#![forbid(unsafe_code)]
//! Composed specified-component controls, independently selected from CSSOM
//! WD20210826 §2.1 and §6.7.2. Color 4's serialization section replaces the
//! older CSSOM color recipe; Fonts 4 and Speech 1 own family grammar.
//! https://www.w3.org/TR/2021/WD-cssom-1-20210826/#serialize-a-css-component-value
//! https://www.w3.org/TR/2021/WD-cssom-1-20210826/#common-serializing-idioms
//! Authored URL targets and relative units stay context-free. These controls
//! do not resolve URLs, fonts, colors, layout, inheritance or used values.

use surgeist_css::*;

type Limits = CssSpecifiedValueSerializationLimits;
type Kind = CssSpecifiedValueSerializationErrorKind;

fn component(text: &str) -> CssComponentValue {
    let values = parse_component_values(text).unwrap();
    let [value] = values.items() else {
        panic!("one scalar component: {text}")
    };
    value.clone()
}

fn declaration(name: &str, text: &str) -> CssDeclaration {
    let report = parse_style_attribute(&format!("{name}:{text}!important"));
    assert!(
        report.is_clean(),
        "{name}:{text}: {:?}",
        report.diagnostics()
    );
    let [value] = report.syntax().as_slice() else {
        panic!("one ordinary declaration")
    };
    value.clone()
}

fn specified(value: &CssDeclaration) -> String {
    match value.known().unwrap().property_value().unwrap() {
        CssKnownPropertyValueRef::Color(v) => v.value().to_specified_css().unwrap(),
        CssKnownPropertyValueRef::Filter(v) => v.value().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::AspectRatio(v) => v.ratio().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::Clip(v) => v.clip().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::Content(v) => v.value().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::FontSize(v) => v.size().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::FontFamily(v) => v.families().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::VoiceFamily(v) => v.value().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::BorderTopWidth(v) => v.value().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::BorderTopStyle(v) => v.value().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::Top(v) => v.value().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::Right(v) => v.value().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::Bottom(v) => v.value().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::Left(v) => v.value().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::MarginTop(v) => v.value().serialize_specified().unwrap(),
        CssKnownPropertyValueRef::PaddingTop(v) => v.value().serialize_specified().unwrap(),
        _ => panic!("fixture selects a concrete component owner"),
    }
}

#[test]
fn scalar_outputs_compose_without_losing_exact_authored_components() {
    let number_component = component("+001.2345678");
    let number = CssSpecifiedNumber::try_from_component(number_component.clone()).unwrap();
    let percentage_component = component("12.5000%");
    let percentage =
        CssSpecifiedPercentage::try_from_component(percentage_component.clone()).unwrap();
    let length_component = component("+02.5000Q");
    let length = CssSpecifiedLength::try_from_component(length_component.clone()).unwrap();
    let integer_component = component("-0002147483649");
    let integer_literal = CssIntegerLiteral::try_from_component(integer_component.clone()).unwrap();
    let integer = CssIntegerValue::Literal(integer_literal.clone());
    let time = CssTimeLiteral::try_from_component(component("1250MS")).unwrap();
    let frequency = CssFrequencyLiteral::try_from_component(component("2KHz")).unwrap();
    let resolution = CssResolutionLiteral::try_from_component(component("150DPI")).unwrap();
    let values = [
        number.serialize_specified().unwrap(),
        integer.serialize_specified().unwrap(),
        percentage.serialize_specified().unwrap(),
        length.serialize_specified().unwrap(),
        time.serialize_specified().unwrap(),
        frequency.serialize_specified().unwrap(),
        resolution.serialize_specified().unwrap(),
    ];
    let items: Vec<&str> = values.iter().map(String::as_str).collect();
    assert_eq!(
        serialize_css_whitespace_separated_list(&items).unwrap(),
        "1.234568 -2147483649 12.5% 2.5q 1.25s 2khz 1.5625dppx"
    );
    assert_eq!(number.literal_component(), Some(&number_component));
    assert_eq!(percentage.literal_component(), Some(&percentage_component));
    assert_eq!(length.literal_component(), Some(&length_component));
    assert_eq!(integer_literal.component(), &integer_component);
    assert_eq!(time.numeric().representation(), "1250");
    assert_eq!(frequency.unit(), CssFrequencyUnit::Kilohertz);
    assert_eq!(resolution.unit(), CssResolutionUnit::Dpi);
    for origin in [
        number.origin(),
        percentage.origin(),
        length.origin(),
        integer_literal.origin(),
    ] {
        assert!(matches!(origin, CssValueOrigin::Parsed(_)));
    }
    let checked_component = CssComponentValue::try_number("+001.2345678").unwrap();
    let checked = CssSpecifiedNumber::try_from_component(checked_component.clone()).unwrap();
    assert_eq!(checked.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(checked.literal_component(), Some(&checked_component));
    assert_eq!(checked.serialize_specified().unwrap(), "1.234568");
    assert!(CssIntegerLiteral::try_from_component(component("1e0")).is_err());
    assert!(CssSpecifiedLength::try_from_component(component("2hz")).is_err());
}

#[test]
fn angle_ratio_shape_and_counter_components_use_their_public_owners() {
    for (name, text, expected) in [
        ("filter", "hue-rotate(+01.2500TURN)", "hue-rotate(1.25turn)"),
        ("filter", "hue-rotate(2RAD)", "hue-rotate(2rad)"),
        ("filter", "hue-rotate(3GRAD)", "hue-rotate(3grad)"),
        ("filter", "hue-rotate(4DEG)", "hue-rotate(4deg)"),
        ("aspect-ratio", "AUTO +03.000", "auto 3 / 1"),
        ("aspect-ratio", "3 / 2", "3 / 2"),
        ("clip", "rect(1PX auto -2EM 0)", "rect(1px, auto, -2em, 0)"),
        (
            "content",
            "counter(chapter, decimal) counters(section, '/', lower-roman)",
            "counter(chapter) counters(section, \"/\", lower-roman)",
        ),
    ] {
        let value = declaration(name, text);
        let before = value.clone();
        assert_eq!(specified(&value), expected, "{name}:{text}");
        assert_eq!(specified(&declaration(name, expected)), expected);
        assert_eq!(value, before);
        assert!(value.same_occurrence(&before));
        assert!(value.parsed_value().is_some());
    }
}

#[test]
fn macro_aliases_delegate_to_concrete_property_grammar_and_family_rules() {
    // CSSOM's macro list is a grammar delegation, not permission to accept an
    // arbitrary keyword or to flatten Fonts/Speech family token boundaries.
    for (name, text, expected) in [
        ("font-size", "XX-SMALL", "xx-small"),
        ("font-size", "LARGER", "larger"),
        ("border-top-width", "MEDIUM", "medium"),
        ("border-top-style", "DASHED", "dashed"),
        ("top", "AUTO", "auto"),
        ("right", "+02PX", "2px"),
        ("bottom", "25.000%", "25%"),
        ("left", "-1EM", "-1em"),
        ("margin-top", "AUTO", "auto"),
        ("padding-top", "2Q", "2q"),
        (
            "font-family",
            "Gentium Bold, SERIF, '1Font'",
            "Gentium Bold, serif, \"1Font\"",
        ),
        ("voice-family", "FEMALE", "female"),
    ] {
        let parsed = declaration(name, text);
        let property = parsed.known().unwrap().property();
        let components = parse_component_values(text).unwrap();
        let retained = components.clone();
        let checked = parse_property_value(
            CssPropertyNameRef::Known(property),
            components,
            CssImportance::Important,
        )
        .unwrap();
        assert_eq!(specified(&parsed), expected, "{name}:{text}");
        assert_eq!(specified(&checked), expected, "checked {name}:{text}");
        assert!(parsed.position().is_some());
        assert!(checked.position().is_none());
        assert_eq!(checked.value_components(), &retained);
        assert!(!parsed.same_occurrence(&checked));
    }
    for (name, invalid) in [
        ("font-size", "dashed"),
        ("border-top-width", "auto"),
        ("border-top-style", "medium"),
        ("padding-top", "auto"),
        ("padding-top", "-1px"),
        ("voice-family", "female male"),
    ] {
        let report = parse_style_attribute(&format!("{name}:{invalid};color:red"));
        assert!(!report.is_clean(), "{name}:{invalid}");
        assert_eq!(report.syntax().len(), 1, "invalid declaration is atomic");
        assert_eq!(specified(&report.syntax()[0]), "red");
    }
}

#[test]
fn common_escaping_and_authored_wrappers_compose_already_serialized_items() {
    let identifier = serialize_css_identifier("1é😀").unwrap();
    let string = serialize_css_string("a\"\\\n\0").unwrap();
    let url = CssUrl::new("../é😀.woff2#face");
    let local = CssFontLocalName::try_new("1é😀").unwrap();
    let url_text = url.serialize_specified().unwrap();
    let local_text = local.serialize_specified().unwrap();
    assert_eq!(identifier, "\\31 é😀");
    assert_eq!(string, "\"a\\\"\\\\\\a �\"");
    assert_eq!(url_text, "url(\"../é😀.woff2#face\")");
    assert_eq!(local_text, "local(\"1é😀\")");
    assert_eq!(url.as_str(), "../é😀.woff2#face");
    assert_eq!(local.as_str(), "1é😀");
    let items = [
        identifier.as_str(),
        string.as_str(),
        url_text.as_str(),
        local_text.as_str(),
        "",
    ];
    let expected = "\\31 é😀, \"a\\\"\\\\\\a �\", url(\"../é😀.woff2#face\"), local(\"1é😀\"), ";
    assert_eq!(
        serialize_css_comma_separated_list(&items).unwrap(),
        expected
    );
    // One list and five already-serialized items; Unicode consumes UTF-8 bytes.
    assert_eq!(
        serialize_css_comma_separated_list_with_limits(&items, Limits::new(6, 6, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(5, 6, expected.len()), Kind::InputNodeLimit),
        (Limits::new(6, 5, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(6, 6, expected.len() - 1), Kind::ByteLimit),
    ] {
        assert_eq!(
            serialize_css_comma_separated_list_with_limits(&items, limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(items[2], "url(\"../é😀.woff2#face\")");
    }
}

#[test]
fn local_and_url_source_children_share_one_atomic_resource_budget() {
    let text = "LOCAL('é😀'), URL('../font.woff2')";
    let report = parse_font_face_descriptor_value(text, CssFontFaceDescriptorKind::Src);
    assert!(report.is_clean(), "{:?}", report.diagnostics());
    let Some(CssAuthoredFontFaceDescriptorValue::Ordinary(CssFontFaceDescriptorValue::Src(parsed))) =
        report.syntax()
    else {
        panic!("source list")
    };
    let checked = CssFontFaceSourceList::try_new(vec![
        CssFontFaceSource::Local(CssFontLocalName::try_new("é😀").unwrap()),
        CssFontFaceSource::Url(CssFontFaceUrlSource::new(
            CssUrl::new("../font.woff2"),
            None,
            Vec::new(),
        )),
    ])
    .unwrap();
    let expected = "local(\"é😀\"), url(\"../font.woff2\")";
    // List = 1; local wrapper/string = 2; URL source + URL/target = 3.
    for value in [parsed, &checked] {
        let before = value.clone();
        assert_eq!(
            value
                .serialize_specified_with_limits(Limits::new(6, 6, expected.len()))
                .unwrap(),
            expected
        );
        for (limits, kind) in [
            (Limits::new(5, 6, expected.len()), Kind::InputNodeLimit),
            (Limits::new(6, 5, expected.len()), Kind::ProjectionNodeLimit),
            (Limits::new(6, 6, expected.len() - 1), Kind::ByteLimit),
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
        assert_eq!(value.serialize_specified().unwrap(), expected);
    }
}

#[test]
fn rectangle_edges_share_cumulative_limits_and_keep_their_authored_origins() {
    let source = declaration("clip", "rect(+01PX auto -2EM 0)");
    let CssKnownPropertyValueRef::Clip(wrapper) = source.known().unwrap().property_value().unwrap()
    else {
        panic!("clip")
    };
    let value = wrapper.clip();
    let before = value.clone();
    let CssClip::Rect(rect) = value else {
        panic!("rectangle")
    };
    let retained: Vec<CssComponentValue> = [rect.top(), rect.bottom(), rect.left()]
        .into_iter()
        .map(|edge| {
            let CssClipEdge::Length(length) = edge else {
                panic!("ordinary length edge")
            };
            assert!(matches!(length.origin(), CssValueOrigin::Parsed(_)));
            length.literal_component().unwrap().clone()
        })
        .collect();
    let expected = "rect(1px, auto, -2em, 0)";
    // One rectangle and four effective edge scalars; clip adds no extra node.
    assert_eq!(
        value
            .serialize_specified_with_limits(Limits::new(5, 5, expected.len()))
            .unwrap(),
        expected
    );
    for (limits, kind) in [
        (Limits::new(4, 5, expected.len()), Kind::InputNodeLimit),
        (Limits::new(5, 4, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(5, 5, expected.len() - 1), Kind::ByteLimit),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            kind
        );
        assert_eq!(value, &before);
        assert!(source.same_occurrence(&source.clone()));
        for (edge, original) in [rect.top(), rect.bottom(), rect.left()]
            .into_iter()
            .zip(&retained)
        {
            let CssClipEdge::Length(length) = edge else {
                panic!("length edge")
            };
            assert_eq!(length.literal_component(), Some(original));
            assert_eq!(length.origin(), original.origin());
        }
    }
    assert_eq!(value.serialize_specified().unwrap(), expected);
}

fn color(text: &str) -> CssColor {
    let value = declaration("color", text);
    let CssKnownPropertyValueRef::Color(wrapper) = value.known().unwrap().property_value().unwrap()
    else {
        panic!("color")
    };
    wrapper.value().clone()
}

#[test]
fn declared_color4_replacement_preserves_keywords_and_missing_components() {
    for (text, expected) in [
        ("PuRpLe", "purple"),
        ("TRANSPARENT", "transparent"),
        ("CurrentColor", "currentcolor"),
        ("CanvasText", "canvastext"),
        ("ButtonFace", "buttonface"),
        ("rgb(none 0 255)", "color(srgb none 0 1)"),
        (
            "color(display-p3 1 0 0 / none)",
            "color(display-p3 1 0 0 / none)",
        ),
    ] {
        let value = color(text);
        let before = value.clone();
        assert_eq!(value.to_specified_css().unwrap(), expected, "{text}");
        assert_eq!(color(expected).to_specified_css().unwrap(), expected);
        assert_eq!(value, before);
    }
}

// CSSOM's byte-alpha rule is independently evaluated with integer fractions:
// first seek an integer percentage whose half-up byte conversion is the byte;
// otherwise round byte/255 to thousandths. This uses no production formatter,
// binary64 conversion or serializer output as an oracle.
fn byte_alpha(byte: u32) -> String {
    let thousandths =
        if let Some(percent) = (0..=100).find(|percent| (percent * 255 + 50) / 100 == byte) {
            percent * 10
        } else {
            (byte * 1000 + 127) / 255
        };
    if thousandths == 0 {
        return "0".into();
    }
    if thousandths == 1000 {
        return "1".into();
    }
    format!("0.{thousandths:03}")
        .trim_end_matches('0')
        .to_owned()
}

#[test]
fn every_byte_alpha_has_an_independent_cssom_witness_and_atomic_byte_limit() {
    for byte in 0..=255 {
        let text = format!("#000000{byte:02x}");
        let value = color(&text);
        let before = value.clone();
        let components = parse_component_values(&text).unwrap();
        let retained = components.clone();
        let checked_declaration = parse_property_value(
            CssPropertyNameRef::Known(CssKnownProperty::Color),
            components,
            CssImportance::Normal,
        )
        .unwrap();
        let CssKnownPropertyValueRef::Color(checked) = checked_declaration
            .known()
            .unwrap()
            .property_value()
            .unwrap()
        else {
            panic!("checked color")
        };
        let expected = if byte == 255 {
            "rgb(0, 0, 0)".to_owned()
        } else {
            format!("rgba(0, 0, 0, {})", byte_alpha(byte))
        };
        // A hex color is one color node in both input and projection counts.
        assert_eq!(
            value
                .to_specified_css_with_limits(Limits::new(1, 1, expected.len()))
                .unwrap(),
            expected,
            "{text}"
        );
        assert_eq!(
            value
                .to_specified_css_with_limits(Limits::new(1, 1, expected.len() - 1))
                .unwrap_err()
                .kind(),
            Kind::ByteLimit
        );
        assert_eq!(value, before);
        assert_eq!(value.to_specified_css().unwrap(), expected);
        assert_eq!(checked.value().to_specified_css().unwrap(), expected);
        assert_eq!(checked_declaration.value_components(), &retained);
    }
    for (byte, expected) in [(1, "0.004"), (127, "0.498"), (128, "0.5"), (254, "0.996")] {
        assert_eq!(byte_alpha(byte), expected);
    }
}
