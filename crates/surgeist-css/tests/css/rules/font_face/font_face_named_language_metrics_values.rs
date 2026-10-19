#![forbid(unsafe_code)]
//! Independent Fonts 4 descriptor payload expectations, CSSOM string escaping,
//! and Values 4 specified percentage/range semantics through public APIs.

use surgeist_css::*;

type Kind = CssFontFaceDescriptorKind;
type Value = CssFontFaceDescriptorValue;
type Limits = CssSpecifiedValueSerializationLimits;
type Error = CssSpecifiedValueSerializationError;
type ErrorKind = CssSpecifiedValueSerializationErrorKind;

fn parsed(kind: Kind, text: &str) -> Value {
    let report = parse_font_face_descriptor_value(text, kind);
    assert!(report.is_clean(), "{}: {text}: {report:?}", kind.css_name());
    let Some(CssAuthoredFontFaceDescriptorValue::Ordinary(value)) = report.syntax() else {
        panic!("ordinary descriptor payload")
    };
    value.clone()
}

fn emit(value: &Value, limits: Limits) -> Result<String, Error> {
    match value {
        Value::FontNamedInstance(value) => value.serialize_specified_with_limits(limits),
        Value::FontLanguageOverride(value) => value.serialize_specified_with_limits(limits),
        Value::AscentOverride(value)
        | Value::DescentOverride(value)
        | Value::LineGapOverride(value) => value.serialize_specified_with_limits(limits),
        _ => panic!("one of the five descriptor payloads"),
    }
}

fn assert_primitive(value: Value, expected: &str) {
    let before = value.clone();
    assert_eq!(emit(&value, Limits::default()).unwrap(), expected);
    assert_eq!(
        emit(&value, Limits::new(1, 1, expected.len())).unwrap(),
        expected
    );
    for (limits, expected_kind) in [
        (Limits::new(0, 1, expected.len()), ErrorKind::InputNodeLimit),
        (
            Limits::new(1, 0, expected.len()),
            ErrorKind::ProjectionNodeLimit,
        ),
        (Limits::new(1, 1, expected.len() - 1), ErrorKind::ByteLimit),
    ] {
        assert_eq!(emit(&value, limits).unwrap_err().kind(), expected_kind);
        assert_eq!(value, before);
    }
    assert_eq!(parsed(value.kind(), expected), value);
    assert_eq!(emit(&value, Limits::default()).unwrap(), expected);
}

#[test]
fn named_instance_strings_keep_decoded_names_and_original_components() {
    let input = r#""Gr\6f tesque""#;
    let component = parse_component_values(input).unwrap().items()[0].clone();
    let name = CssFontNamedInstanceString::try_from_component(component.clone()).unwrap();
    assert_eq!(name.as_str(), "Grotesque");
    assert_eq!(name.component(), &component);
    let CssValueOrigin::Parsed(origin) = name.origin() else {
        panic!("string retains its parsed source")
    };
    assert_eq!(origin.source().as_str(), input);
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    assert_eq!(origin.span().end().byte_offset().value(), input.len());
    assert_eq!(name.serialize_specified().unwrap(), "\"Grotesque\"");
    let programmatic = CssFontNamedInstanceString::try_new("Grotesque").unwrap();
    assert_eq!(programmatic.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(name, programmatic);
    assert_eq!(
        programmatic
            .serialize_specified_with_limits(Limits::new(1, 1, 11))
            .unwrap(),
        "\"Grotesque\""
    );
    assert_eq!(
        programmatic
            .serialize_specified_with_limits(Limits::new(1, 1, 10))
            .unwrap_err()
            .kind(),
        ErrorKind::ByteLimit
    );
    assert!(CssFontNamedInstanceString::try_new("a\0b").is_err());
    for component in [
        CssComponentValue::try_ident("auto").unwrap(),
        CssComponentValue::try_number("1").unwrap(),
        CssComponentValue::try_token("10%").unwrap(),
    ] {
        let error = CssFontNamedInstanceString::try_from_component(component).unwrap_err();
        assert_eq!(error.kind(), CssComponentValueErrorKind::InvalidString);
        assert_eq!(error.origin(), &CssValueOrigin::Programmatic);
    }
}

#[test]
fn strings_and_keywords_emit_canonical_text_under_exact_limits() {
    assert_primitive(Value::FontNamedInstance(CssFontNamedInstance::Auto), "auto");
    assert_primitive(
        Value::FontLanguageOverride(CssFontLanguageOverride::Normal),
        "normal",
    );
    for (decoded, expected) in [
        ("", "\"\""),
        ("auto", "\"auto\""),
        ("LongTag", "\"LongTag\""),
        ("é😀", "\"é😀\""),
        ("A\"\\\n\u{1}'", "\"A\\\"\\\\\\a \\1 '\""),
    ] {
        let name = CssFontNamedInstanceString::try_new(decoded).unwrap();
        assert_eq!(name.serialize_specified().unwrap(), expected);
        assert_primitive(
            Value::FontNamedInstance(CssFontNamedInstance::String(name)),
            expected,
        );
        assert_primitive(
            Value::FontLanguageOverride(CssFontLanguageOverride::String(
                CssFontLanguageString::try_new(decoded).unwrap(),
            )),
            expected,
        );
    }
    for kind in [Kind::FontNamedInstance, Kind::FontLanguageOverride] {
        let value = parsed(kind, "\"\0\"");
        assert_eq!(emit(&value, Limits::default()).unwrap(), "\"\u{fffd}\"");
    }
}

fn metric(value: &Value) -> &CssFontMetricOverride {
    match value {
        Value::AscentOverride(value)
        | Value::DescentOverride(value)
        | Value::LineGapOverride(value) => value,
        _ => panic!("metric override payload"),
    }
}

#[test]
fn three_metric_branches_share_exact_percentage_payloads_without_losing_identity() {
    for kind in [
        Kind::AscentOverride,
        Kind::DescentOverride,
        Kind::LineGapOverride,
    ] {
        let normal = parsed(kind, "NoRmAl");
        assert_eq!(metric(&normal), &CssFontMetricOverride::Normal);
        assert_primitive(normal, "normal");
        for (text, expected) in [
            ("0%", "0%"),
            ("-0%", "0%"),
            ("125.000000000000000000001%", "125%"),
            ("500%", "500%"),
        ] {
            let value = parsed(kind, text);
            assert_eq!(value.kind(), kind);
            let CssFontMetricOverride::Percentage(percentage) = metric(&value) else {
                panic!("exact percentage payload")
            };
            let component = percentage.literal_component().unwrap();
            let CssComponentValueRef::Token(CssValueTokenRef::Percentage(number)) =
                component.view()
            else {
                panic!("retained percentage token")
            };
            assert_eq!(number.representation(), text.strip_suffix('%').unwrap());
            let CssValueOrigin::Parsed(origin) = percentage.origin() else {
                panic!("retained numeric provenance")
            };
            assert_eq!(origin.source().as_str(), text);
            // Canonical output can round/change lexical sign without changing the authored payload.
            let before = value.clone();
            assert_eq!(
                emit(&value, Limits::new(1, 1, expected.len())).unwrap(),
                expected
            );
            assert_eq!(
                emit(&value, Limits::new(1, 1, expected.len() - 1))
                    .unwrap_err()
                    .kind(),
                ErrorKind::ByteLimit
            );
            assert_eq!(value, before);
        }
    }
    let percentage = CssSpecifiedNonNegativePercentage::try_from_component(
        CssComponentValue::try_token("125%").unwrap(),
    )
    .unwrap();
    let constructed = CssFontMetricOverride::Percentage(percentage);
    assert_eq!(constructed.serialize_specified().unwrap(), "125%");
    assert_eq!(
        constructed
            .serialize_specified_with_limits(Limits::new(1, 1, 4))
            .unwrap(),
        "125%"
    );
    for token in ["-0.000000000000000000001%", "10px", "0"] {
        assert!(
            CssSpecifiedNonNegativePercentage::try_from_component(
                CssComponentValue::try_token(token).unwrap(),
            )
            .is_err()
        );
    }
}

#[test]
fn metric_calculations_preserve_authored_range_type_and_origins_on_atomic_failure() {
    for kind in [
        Kind::AscentOverride,
        Kind::DescentOverride,
        Kind::LineGapOverride,
    ] {
        for (text, expected) in [
            ("calc(50% + 25%)", "calc(75%)"),
            ("calc(-10%)", "calc(-10%)"),
        ] {
            let value = parsed(kind, text);
            let CssFontMetricOverride::Percentage(percentage) = metric(&value) else {
                panic!("typed percentage calculation")
            };
            assert!(percentage.literal_component().is_none());
            let calculation = percentage.calculation().unwrap();
            assert_eq!(calculation.result_type(), CssCalculationType::Percentage);
            assert_eq!(calculation.components().serialize().unwrap().as_css(), text);
            let CssValueOrigin::Parsed(origin) = percentage.origin() else {
                panic!("calculation retains parsed provenance")
            };
            assert_eq!(origin.source().as_str(), text);
            let before = value.clone();
            assert_eq!(emit(&value, Limits::default()).unwrap(), expected);
            assert_eq!(
                emit(
                    &value,
                    Limits::new(usize::MAX, usize::MAX, expected.len() - 1)
                )
                .unwrap_err()
                .kind(),
                ErrorKind::ByteLimit
            );
            assert_eq!(
                emit(&value, Limits::new(0, usize::MAX, usize::MAX))
                    .unwrap_err()
                    .kind(),
                ErrorKind::InputNodeLimit
            );
            assert_eq!(
                emit(&value, Limits::new(usize::MAX, 0, usize::MAX))
                    .unwrap_err()
                    .kind(),
                ErrorKind::ProjectionNodeLimit
            );
            assert_eq!(value, before);
            assert_eq!(calculation.components().serialize().unwrap().as_css(), text);
        }
    }
    let calculation = CssPercentageCalculation::try_from_components(
        parse_component_values("calc(-10%)").unwrap(),
    )
    .unwrap();
    let constructed = CssFontMetricOverride::Percentage(
        CssSpecifiedNonNegativePercentage::try_from_calculation(calculation).unwrap(),
    );
    assert_eq!(constructed.serialize_specified().unwrap(), "calc(-10%)");
}

#[test]
fn substituted_values_return_concrete_descriptor_branches_and_original_replacement_origins() {
    for (kind, text) in [
        (Kind::FontNamedInstance, "\"Grotesque\""),
        (Kind::FontLanguageOverride, "\"TRK\""),
        (Kind::AscentOverride, "125%"),
        (Kind::DescentOverride, "25%"),
        (Kind::LineGapOverride, "0%"),
    ] {
        let pending_report = parse_font_face_descriptor_value("env(selection)", kind);
        let Some(CssAuthoredFontFaceDescriptorValue::Pending(pending)) = pending_report.syntax()
        else {
            panic!("whole env deferral")
        };
        let replacement = parse_component_values(text).unwrap();
        let ordinary = pending.reparse_after_substitution(replacement).unwrap();
        assert_eq!(ordinary.kind(), kind);
        let origin = match &ordinary {
            Value::FontNamedInstance(CssFontNamedInstance::String(name)) => name.origin(),
            Value::FontLanguageOverride(CssFontLanguageOverride::String(language)) => {
                language.origin()
            }
            value => {
                let CssFontMetricOverride::Percentage(percentage) = metric(value) else {
                    panic!("percentage reentry")
                };
                percentage.origin()
            }
        };
        let CssValueOrigin::Parsed(origin) = origin else {
            panic!("replacement provenance")
        };
        assert_eq!(origin.source().as_str(), text);
        assert_eq!(parsed(kind, text), ordinary);
    }
}
