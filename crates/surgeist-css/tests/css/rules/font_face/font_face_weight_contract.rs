//! Ordinary font-face weight descriptor contract from Fonts 4 §4.4.

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::rules::invalid_descriptor;
use crate::css::support::rules::ordinary_font_face_descriptor;
use surgeist_css::CssParser;
use surgeist_css::component::{
    CssComponentValue, CssComponentValues, CssSerializedOrigin, CssValueOrigin,
};
use surgeist_css::diagnostic::CssRecoveryAction;
use surgeist_css::numeric::{CssNumber, CssNumeric};
use surgeist_css::rule::CssRule;
use surgeist_css::rule::font_face::CssAuthoredFontFaceDescriptorValue;
use surgeist_css::rule::font_face::CssAuthoredFontFaceDescriptorValue as Authored;
use surgeist_css::rule::font_face::CssFontFaceDescriptorKind as Kind;
use surgeist_css::rule::font_face::CssFontFaceDescriptorValue as Ordinary;
use surgeist_css::serialization::CssSerializationLimits;
use surgeist_css::values::font::CssAbsoluteFontWeight as Absolute;
use surgeist_css::values::font::{CssFontFaceWeight, CssFontWeightRange};

fn ordinary(text: &str) -> CssFontFaceWeight {
    let Ordinary::FontWeight(value) = ordinary_font_face_descriptor(text, Kind::FontWeight) else {
        panic!("ordinary weight descriptor: {text}")
    };
    value
}

fn numeric(text: &str) -> Absolute {
    Absolute::Number(
        CssNumeric::<CssNumber, CssFontWeightRange>::try_from_component(
            CssComponentValue::try_number(text).unwrap(),
        )
        .unwrap(),
    )
}

#[test]
fn auto_single_and_two_endpoint_weights_keep_authored_arity_and_order() {
    assert_eq!(ordinary("auto"), CssFontFaceWeight::Auto);
    assert_eq!(
        ordinary("auto")
            .serialize_specified(Default::default())
            .unwrap(),
        "auto"
    );
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
        assert_eq!(
            expected.serialize_specified(Default::default()).unwrap(),
            text,
            "{text}"
        );
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
    assert_eq!(
        start.serialize_specified(Default::default()).unwrap(),
        "calc(0)"
    );
    assert!(end.literal().map(|literal| literal.component()).is_some());
    assert_eq!(end.serialize_specified(Default::default()).unwrap(), "1000");
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
        let raw = CssAuthoredFontFaceDescriptorValue::parse(
            &CssParser::default(),
            invalid,
            Kind::FontWeight,
        );
        assert!(raw.syntax().is_none(), "{invalid}");
        let source =
            format!("@font-face{{font-weight:normal;font-weight:{invalid};font-display:swap}}");
        let report = CssParser::default().parse_sheet(&source);
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
    let report = CssParser::default().parse_sheet(source);
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
    assert_eq!(end.serialize_specified(Default::default()).unwrap(), "300");

    let report = CssParser::default()
        .parse_sheet("@font-face{font-weight:700;font-weight:env(weight);font-weight:bad}");
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
        CssComponentValues::parse(&CssParser::default(), "env(weight, 700)").unwrap(),
        Default::default(),
    )
    .unwrap();
    let Authored::Pending(pending) = checked else {
        panic!("pending descriptor")
    };
    let mut components = CssComponentValues::parse(&CssParser::default(), "700 ")
        .unwrap()
        .items()
        .to_vec();
    components.push(CssComponentValue::try_number("300.25").unwrap());
    let replacement = CssComponentValues::try_new(components, Default::default()).unwrap();
    let value = pending
        .reparse_after_substitution(replacement, Default::default())
        .unwrap();
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
    assert_eq!(
        start.serialize_specified(Default::default()).unwrap(),
        "700"
    );
    assert_eq!(
        end.serialize_specified(Default::default()).unwrap(),
        "300.25"
    );
}

#[test]
fn descriptor_serialization_is_canonical_and_budgeted() {
    let value = ordinary("bold 300.25");
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "bold 300.25"
    );
    let error = value
        .serialize_specified(CssSerializationLimits::new(8, 8, 3))
        .unwrap_err();
    assert!(error.is_byte_limit(), "{error:?}");
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "bold 300.25"
    );

    let two_keywords = CssFontFaceWeight::Range {
        start: Absolute::Normal,
        end: Some(Absolute::Bold),
    };
    assert_eq!(
        two_keywords
            .serialize_specified(CssSerializationLimits::new(3, 3, 11))
            .unwrap(),
        "normal bold"
    );
    for (limits, expected) in [
        (
            CssSerializationLimits::new(2, 3, 11),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 2, 11),
            SerializationFailure::ProjectionNodeLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&two_keywords.serialize_specified(limits).unwrap_err()),
            expected
        );
    }
    assert_eq!(
        two_keywords
            .serialize_specified(Default::default())
            .unwrap(),
        "normal bold"
    );
}

#[test]
fn checked_descriptor_rejects_programmatic_invalid_and_adjacent_number_tokens() {
    let invalid = CssComponentValues::try_new(
        vec![CssComponentValue::try_number("1001").unwrap()],
        Default::default(),
    )
    .unwrap();
    let error =
        Authored::try_from_components(Kind::FontWeight, invalid, Default::default()).unwrap_err();
    assert!(error.diagnostic().is_some());
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );

    let adjacent = CssComponentValues::try_new(
        vec![
            CssComponentValue::try_number("1").unwrap(),
            CssComponentValue::try_number("00").unwrap(),
        ],
        Default::default(),
    )
    .unwrap();
    let error =
        Authored::try_from_components(Kind::FontWeight, adjacent, Default::default()).unwrap_err();
    assert!(error.diagnostic().is_some());
    let checked = Authored::try_from_components(
        Kind::FontWeight,
        CssComponentValues::parse(&CssParser::default(), "env(weight)").unwrap(),
        Default::default(),
    )
    .unwrap();
    let Authored::Pending(pending) = checked else {
        panic!("pending weight")
    };
    let adjacent = CssComponentValues::try_new(
        vec![
            CssComponentValue::try_number("1").unwrap(),
            CssComponentValue::try_number("00").unwrap(),
        ],
        Default::default(),
    )
    .unwrap();
    // Separate numbers serialize apart, and Fonts 4 §4.4 bounds each
    // <font-weight-absolute> number to [1, 1000].
    let error = pending
        .reparse_after_substitution(adjacent, Default::default())
        .unwrap_err();
    let Some(detail) = error.diagnostic() else {
        panic!("grammar failure: {error:?}")
    };
    invalid_descriptor("font-face", "font-weight")
        .at("00")
        .assert_kind(detail);
}
