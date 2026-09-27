#![forbid(unsafe_code)]

//! Ordinary font-face weight descriptor contract from Fonts 4 §4.4.

use surgeist_css::{
    CssAbsoluteFontWeight as Absolute, CssAuthoredFontFaceDescriptorValue as Authored,
    CssComponentValue, CssComponentValues, CssFontFaceDescriptorKind as Kind,
    CssFontFaceDescriptorValue as Ordinary, CssFontFaceValueErrorKind, CssFontFaceWeight,
    CssFontWeightNumber, CssRecoveryAction, CssRule, CssSerializedOrigin,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits, CssValueOrigin,
    parse_component_values, parse_font_face_descriptor_value, parse_sheet,
};

fn ordinary(text: &str) -> CssFontFaceWeight {
    let report = parse_font_face_descriptor_value(text, Kind::FontWeight);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let Some(Authored::Ordinary(Ordinary::FontWeight(value))) = report.syntax() else {
        panic!("ordinary weight descriptor: {text}")
    };
    value.clone()
}

fn numeric(text: &str) -> Absolute {
    Absolute::Number(
        CssFontWeightNumber::try_from_component(CssComponentValue::try_number(text).unwrap())
            .unwrap(),
    )
}

#[test]
fn auto_single_and_two_endpoint_weights_keep_authored_arity_and_order() {
    assert_eq!(ordinary("auto"), CssFontFaceWeight::Auto);
    assert_eq!(ordinary("auto").serialize_specified().unwrap(), "auto");
    for (text, expected) in [
        (
            "normal",
            CssFontFaceWeight::Range {
                start: Absolute::Normal,
                end: None,
            },
        ),
        (
            "bold",
            CssFontFaceWeight::Range {
                start: Absolute::Bold,
                end: None,
            },
        ),
        (
            "725.25",
            CssFontFaceWeight::Range {
                start: numeric("725.25"),
                end: None,
            },
        ),
        (
            "bold 300",
            CssFontFaceWeight::Range {
                start: Absolute::Bold,
                end: Some(numeric("300")),
            },
        ),
        (
            "300 bold",
            CssFontFaceWeight::Range {
                start: numeric("300"),
                end: Some(Absolute::Bold),
            },
        ),
        (
            "normal bold",
            CssFontFaceWeight::Range {
                start: Absolute::Normal,
                end: Some(Absolute::Bold),
            },
        ),
        (
            "bold normal",
            CssFontFaceWeight::Range {
                start: Absolute::Bold,
                end: Some(Absolute::Normal),
            },
        ),
        (
            "700 300",
            CssFontFaceWeight::Range {
                start: numeric("700"),
                end: Some(numeric("300")),
            },
        ),
    ] {
        assert_eq!(ordinary(text), expected, "{text}");
        assert_eq!(expected.serialize_specified().unwrap(), text, "{text}");
    }
    assert_ne!(ordinary("bold"), ordinary("bold bold"));
    assert_ne!(ordinary("auto"), ordinary("normal"));
}

#[test]
fn fractional_math_and_invalid_descriptor_values_keep_neighbors() {
    let value = ordinary("calc(0) 1000");
    let CssFontFaceWeight::Range {
        start: Absolute::Number(start),
        end: Some(Absolute::Number(end)),
    } = value
    else {
        panic!("symbolic start and literal end")
    };
    assert!(start.calculation().is_some());
    assert_eq!(start.serialize_specified().unwrap(), "calc(0)");
    assert!(end.literal_component().is_some());
    assert_eq!(end.serialize_specified().unwrap(), "1000");
    for invalid in [
        "bolder",
        "lighter",
        "auto 300",
        "300 auto",
        "300 400 500",
        "0",
        "1001",
        "10%",
    ] {
        let raw = parse_font_face_descriptor_value(invalid, Kind::FontWeight);
        assert!(raw.syntax().is_none(), "{invalid}");
        let source =
            format!("@font-face{{font-weight:normal;font-weight:{invalid};font-display:swap}}");
        let report = parse_sheet(&source);
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDescriptor
        );
        let [CssRule::FontFace(face)] = report.syntax().rules() else {
            panic!("retained font face")
        };
        assert_eq!(face.descriptors().occurrences().len(), 2, "{invalid}");
        let selected = face.descriptors().effective(Kind::FontWeight).unwrap();
        assert!(matches!(
            selected.value(),
            Authored::Ordinary(Ordinary::FontWeight(CssFontFaceWeight::Range {
                start: Absolute::Normal,
                end: None
            }))
        ));
    }
}

#[test]
fn duplicate_and_pending_weight_occurrences_select_last_admitted_value() {
    let source = "@font-face{font-weight:700 300;font-weight:env(weight);font-weight:bad;font-weight:bold 300}";
    let report = parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("font face")
    };
    let items = face.descriptors().occurrences().collect::<Vec<_>>();
    assert_eq!(items.len(), 3);
    for (record, text) in
        items
            .iter()
            .zip(["font-weight:700", "font-weight:env", "font-weight:bold"])
    {
        assert_eq!(
            record.position().unwrap().byte_offset().value(),
            source.find(text).unwrap()
        );
    }
    assert!(matches!(items[1].value(), Authored::Pending(_)));
    assert!(std::ptr::eq(
        face.descriptors().effective(Kind::FontWeight).unwrap(),
        items[2]
    ));
    let CssFontFaceWeight::Range {
        start: Absolute::Bold,
        end: Some(Absolute::Number(end)),
    } = ordinary("bold 300")
    else {
        panic!("mixed last value")
    };
    assert_eq!(end.serialize_specified().unwrap(), "300");

    let report = parse_sheet("@font-face{font-weight:700;font-weight:env(weight);font-weight:bad}");
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("font face")
    };
    assert!(matches!(
        face.descriptors()
            .effective(Kind::FontWeight)
            .unwrap()
            .value(),
        Authored::Pending(_)
    ));
}

#[test]
fn pending_env_reentry_returns_exact_ordinary_weight_without_reordering() {
    let checked = Authored::try_from_components(
        Kind::FontWeight,
        parse_component_values("env(weight, 700)").unwrap(),
    )
    .unwrap();
    let Authored::Pending(pending) = checked else {
        panic!("pending descriptor")
    };
    let mut components = parse_component_values("700 ").unwrap().items().to_vec();
    components.push(CssComponentValue::try_number("300.25").unwrap());
    let replacement = CssComponentValues::try_new(components).unwrap();
    let value = pending.reparse_after_substitution(replacement).unwrap();
    let Ordinary::FontWeight(CssFontFaceWeight::Range {
        start: Absolute::Number(start),
        end: Some(Absolute::Number(end)),
    }) = value
    else {
        panic!("two numeric endpoints")
    };
    let CssValueOrigin::Parsed(origin) = start.origin() else {
        panic!("original first endpoint")
    };
    assert_eq!(origin.source().as_str(), "700 ");
    assert_eq!(origin.span().start().byte_offset().value(), 0);
    assert_eq!(origin.span().end().byte_offset().value(), 3);
    assert_eq!(end.origin(), &CssValueOrigin::Programmatic);
    assert_eq!(start.serialize_specified().unwrap(), "700");
    assert_eq!(end.serialize_specified().unwrap(), "300.25");
}

#[test]
fn descriptor_serialization_is_canonical_and_budgeted() {
    let value = ordinary("bold 300.25");
    assert_eq!(value.serialize_specified().unwrap(), "bold 300.25");
    let error = value
        .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(8, 8, 3))
        .unwrap_err();
    assert_eq!(
        error.kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(value.serialize_specified().unwrap(), "bold 300.25");

    let two_keywords = CssFontFaceWeight::Range {
        start: Absolute::Normal,
        end: Some(Absolute::Bold),
    };
    assert_eq!(
        two_keywords
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 11))
            .unwrap(),
        "normal bold"
    );
    for (limits, expected) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, 11),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, 11),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
    ] {
        assert_eq!(
            two_keywords
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            expected
        );
    }
    assert_eq!(two_keywords.serialize_specified().unwrap(), "normal bold");
}

#[test]
fn checked_descriptor_rejects_programmatic_invalid_and_adjacent_number_tokens() {
    let invalid =
        CssComponentValues::try_new(vec![CssComponentValue::try_number("1001").unwrap()]).unwrap();
    let error = Authored::try_from_components(Kind::FontWeight, invalid).unwrap_err();
    assert!(matches!(
        error.kind(),
        CssFontFaceValueErrorKind::Grammar(_)
    ));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );

    let adjacent = CssComponentValues::try_new(vec![
        CssComponentValue::try_number("1").unwrap(),
        CssComponentValue::try_number("00").unwrap(),
    ])
    .unwrap();
    let error = Authored::try_from_components(Kind::FontWeight, adjacent).unwrap_err();
    assert!(matches!(
        error.kind(),
        CssFontFaceValueErrorKind::Grammar(_)
    ));
    let checked = Authored::try_from_components(
        Kind::FontWeight,
        parse_component_values("env(weight)").unwrap(),
    )
    .unwrap();
    let Authored::Pending(pending) = checked else {
        panic!("pending weight")
    };
    let adjacent = CssComponentValues::try_new(vec![
        CssComponentValue::try_number("1").unwrap(),
        CssComponentValue::try_number("00").unwrap(),
    ])
    .unwrap();
    assert!(pending.reparse_after_substitution(adjacent).is_err());
}
