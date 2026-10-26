//! Ordinary Fonts 4 font-face style descriptor and checked range contracts.

use crate::css::support::failure_classes::SerializationFailure;
use crate::css::support::failure_classes::serialization_failure;
use crate::css::support::rules::invalid_descriptor;
use crate::css::support::rules::ordinary_font_face_descriptor;
use surgeist_css::{
    CssAngle, CssAuthoredFontFaceDescriptorValue as Authored, CssAuthoredFontFaceDescriptorValue,
    CssComponentValue, CssComponentValues, CssFontFaceDescriptorKind as Kind,
    CssFontFaceDescriptorValue as Ordinary, CssFontFaceObliqueRange, CssFontFaceStyle,
    CssFontStyleKeyword, CssNumeric, CssObliqueAngleRange, CssParser, CssRecoveryAction, CssRule,
    CssSerializationLimits, CssSerializedOrigin, CssValueOrigin,
};

fn angle(text: &str) -> CssNumeric<CssAngle, CssObliqueAngleRange> {
    CssNumeric::<CssAngle, CssObliqueAngleRange>::try_from_component(
        CssComponentValue::try_token(text).unwrap(),
    )
    .unwrap()
}

fn ordinary(text: &str) -> CssFontFaceStyle {
    let Ordinary::FontStyle(value) = ordinary_font_face_descriptor(text, Kind::FontStyle) else {
        panic!("ordinary style descriptor: {text}")
    };
    value
}

#[test]
fn auto_keywords_and_oblique_ranges_preserve_omission_arity_and_order() {
    assert_eq!(ordinary("auto"), CssFontFaceStyle::Auto);
    for (text, keyword) in [
        ("normal", CssFontStyleKeyword::Normal),
        ("italic", CssFontStyleKeyword::Italic),
        ("left", CssFontStyleKeyword::Left),
        ("right", CssFontStyleKeyword::Right),
    ] {
        let value = ordinary(text);
        assert_eq!(value, CssFontFaceStyle::Keyword(keyword));
        assert_eq!(value.serialize_specified(Default::default()).unwrap(), text);
    }
    assert_eq!(
        ordinary("oblique"),
        CssFontFaceStyle::Oblique { range: None }
    );
    assert_ne!(ordinary("oblique"), ordinary("oblique 14deg"));
    for (text, start, end, expected_range) in [
        ("oblique 10deg", "10deg", None, "10deg"),
        (
            "oblique -90deg 90deg",
            "-90deg",
            Some("90deg"),
            "-90deg 90deg",
        ),
        (
            "oblique 20deg -10deg",
            "20deg",
            Some("-10deg"),
            "20deg -10deg",
        ),
        (
            "oblique .25turn -100grad",
            "0.25turn",
            Some("-100grad"),
            "0.25turn -100grad",
        ),
        (
            "oblique calc(100deg) 10grad",
            "calc(100deg)",
            Some("10grad"),
            "calc(100deg) 10grad",
        ),
    ] {
        let CssFontFaceStyle::Oblique { range: Some(range) } = ordinary(text) else {
            panic!("range: {text}")
        };
        assert_eq!(
            range
                .start()
                .serialize_specified(Default::default())
                .unwrap(),
            start,
            "{text}"
        );
        assert_eq!(
            range
                .end()
                .map(|end| end.serialize_specified(Default::default()).unwrap())
                .as_deref(),
            end,
            "{text}"
        );
        assert_eq!(
            range.serialize_specified(Default::default()).unwrap(),
            expected_range,
            "{text}"
        );
    }
}

#[test]
fn checked_range_owns_checked_endpoints_without_ordering_or_degree_projection() {
    let start = angle("90deg");
    let end = angle("-.25turn");
    let range = CssFontFaceObliqueRange::new(start.clone(), Some(end.clone()));
    assert_eq!(range.start(), &start);
    assert_eq!(range.end(), Some(&end));
    assert_eq!(
        range.serialize_specified(Default::default()).unwrap(),
        "90deg -0.25turn"
    );
    assert_eq!(
        CssFontFaceStyle::Oblique { range: Some(range) }
            .serialize_specified(Default::default())
            .unwrap(),
        "oblique 90deg -0.25turn"
    );
    let single = CssFontFaceObliqueRange::new(angle("0rad"), None);
    assert!(single.end().is_none());
    assert_eq!(
        single.serialize_specified(Default::default()).unwrap(),
        "0rad"
    );
}

#[test]
fn invalid_arity_domains_and_bounds_recover_one_descriptor_and_keep_neighbors() {
    for invalid in [
        "auto 10deg",
        "italic 10deg",
        "oblique 0",
        "oblique 10px",
        "oblique 10%",
        "oblique 91deg",
        "oblique -91deg",
        "oblique 10deg 20deg 30deg",
        "oblique 1.5707963267948968rad",
        "oblique calc(10px)",
    ] {
        let raw = CssAuthoredFontFaceDescriptorValue::parse(
            &CssParser::default(),
            invalid,
            Kind::FontStyle,
        );
        assert!(raw.syntax().is_none(), "{invalid}");
        let source =
            format!("@font-face{{font-style:normal;font-style:{invalid};font-display:swap}}");
        let report = CssParser::default().parse_sheet(&source);
        assert_eq!(report.diagnostics().len(), 1, "{invalid}");
        assert_eq!(
            report.diagnostics()[0].action(),
            CssRecoveryAction::DropDescriptor,
            "{invalid}"
        );
        let [CssRule::FontFace(face)] = report.syntax().rules() else {
            panic!("font face")
        };
        assert_eq!(face.descriptors().occurrences().len(), 2, "{invalid}");
        let effective = face.descriptors().effective(Kind::FontStyle).unwrap();
        assert!(matches!(
            effective.value(),
            Authored::Ordinary(Ordinary::FontStyle(CssFontFaceStyle::Keyword(
                CssFontStyleKeyword::Normal
            )))
        ));
    }
    let empty = CssParser::default().parse_sheet("@font-face{}");
    assert!(empty.is_clean());
    assert!(matches!(empty.syntax().rules(), [CssRule::FontFace(_)]));
}

#[test]
fn duplicate_pending_and_mixed_origin_reentry_select_last_admitted_style() {
    let source = "@font-face{font-style:oblique 10deg;font-style:env(style);font-style:bad;font-style:italic}";
    let report = CssParser::default().parse_sheet(source);
    assert_eq!(report.diagnostics().len(), 1);
    let [CssRule::FontFace(face)] = report.syntax().rules() else {
        panic!("font face")
    };
    let occurrences = face.descriptors().occurrences().collect::<Vec<_>>();
    assert_eq!(occurrences.len(), 3);
    assert!(matches!(occurrences[1].value(), Authored::Pending(_)));
    assert!(std::ptr::eq(
        face.descriptors().effective(Kind::FontStyle).unwrap(),
        occurrences[2]
    ));

    let checked = Authored::try_from_components(
        Kind::FontStyle,
        CssComponentValues::parse(&CssParser::default(), "env(style, oblique)").unwrap(),
        Default::default(),
    )
    .unwrap();
    let Authored::Pending(pending) = checked else {
        panic!("pending descriptor")
    };
    let mut components = CssComponentValues::parse(&CssParser::default(), "oblique 10deg ")
        .unwrap()
        .items()
        .to_vec();
    let replacement = pending
        .reparse_after_substitution(
            CssComponentValues::try_new(components.clone(), Default::default()).unwrap(),
            Default::default(),
        )
        .unwrap();
    let Ordinary::FontStyle(CssFontFaceStyle::Oblique { range: Some(range) }) = replacement else {
        panic!("reentered range")
    };
    let CssValueOrigin::Parsed(origin) = range.start().origin() else {
        panic!("parsed endpoint")
    };
    assert_eq!(origin.source().as_str(), "oblique 10deg ");
    assert_eq!(origin.span().start().byte_offset().value(), 8);
    assert_eq!(origin.span().end().byte_offset().value(), 13);

    components.push(CssComponentValue::try_token("-20grad").unwrap());
    let replacement = pending
        .reparse_after_substitution(
            CssComponentValues::try_new(components, Default::default()).unwrap(),
            Default::default(),
        )
        .unwrap();
    let Ordinary::FontStyle(CssFontFaceStyle::Oblique { range: Some(range) }) = replacement else {
        panic!("mixed range")
    };
    assert_eq!(
        range
            .start()
            .serialize_specified(Default::default())
            .unwrap(),
        "10deg"
    );
    assert_eq!(range.end().unwrap().origin(), &CssValueOrigin::Programmatic);
    assert_eq!(
        range
            .end()
            .unwrap()
            .serialize_specified(Default::default())
            .unwrap(),
        "-20grad"
    );
    for text in ["env(style)", "var(--style)"] {
        assert!(
            pending
                .reparse_after_substitution(
                    CssComponentValues::parse(&CssParser::default(), text).unwrap(),
                    Default::default()
                )
                .unwrap_err()
                .is_residual_substitution(),
            "{text}"
        );
    }
    // Fonts 4 §4.2: an oblique angle lies in [-90deg, 90deg].
    let error = pending
        .reparse_after_substitution(
            CssComponentValues::parse(&CssParser::default(), "oblique 91deg").unwrap(),
            Default::default(),
        )
        .unwrap_err();
    let Some(detail) = error.diagnostic() else {
        panic!("grammar failure: {error:?}")
    };
    invalid_descriptor("font-face", "font-style")
        .at("91deg")
        .assert_kind(detail);
}

#[test]
fn strict_checked_construction_rejects_invalid_and_adjacent_tokens_with_real_origin() {
    let invalid = CssComponentValues::try_new(
        vec![
            CssComponentValue::try_ident("oblique").unwrap(),
            CssComponentValue::try_token("91deg").unwrap(),
        ],
        Default::default(),
    )
    .unwrap();
    let error =
        Authored::try_from_components(Kind::FontStyle, invalid, Default::default()).unwrap_err();
    assert!(error.diagnostic().is_some());
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );

    let adjacent = CssComponentValues::try_new(
        vec![
            CssComponentValue::try_ident("oblique").unwrap(),
            CssComponentValue::try_number("1").unwrap(),
            CssComponentValue::try_token("0deg").unwrap(),
        ],
        Default::default(),
    )
    .unwrap();
    // Fonts 4 §4.2: oblique takes <angle>{0,2}; `1` is a <number>.
    let error =
        Authored::try_from_components(Kind::FontStyle, adjacent, Default::default()).unwrap_err();
    let Some(detail) = error.diagnostic() else {
        panic!("grammar failure: {error:?}")
    };
    invalid_descriptor("font-face", "font-style")
        .at("1")
        .assert_kind(detail);
}

#[test]
fn descriptor_and_range_serialization_enforce_aggregate_budgets_atomically() {
    let range = CssFontFaceObliqueRange::new(angle("10deg"), Some(angle("20deg")));
    assert_eq!(
        range.serialize_specified(Default::default()).unwrap(),
        "10deg 20deg"
    );
    assert_eq!(
        range
            .serialize_specified(CssSerializationLimits::new(2, 2, 11))
            .unwrap(),
        "10deg 20deg"
    );
    for (limits, expected) in [
        (
            CssSerializationLimits::new(1, 2, 11),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(2, 1, 11),
            SerializationFailure::ProjectionNodeLimit,
        ),
        (
            CssSerializationLimits::new(2, 2, 10),
            SerializationFailure::ByteLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&range.serialize_specified(limits).unwrap_err()),
            expected
        );
    }
    let value = CssFontFaceStyle::Oblique { range: Some(range) };
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "oblique 10deg 20deg"
    );
    assert_eq!(
        value
            .serialize_specified(CssSerializationLimits::new(3, 3, 19))
            .unwrap(),
        "oblique 10deg 20deg"
    );
    for (limits, expected) in [
        (
            CssSerializationLimits::new(2, 3, 19),
            SerializationFailure::InputNodeLimit,
        ),
        (
            CssSerializationLimits::new(3, 2, 19),
            SerializationFailure::ProjectionNodeLimit,
        ),
    ] {
        assert_eq!(
            serialization_failure(&value.serialize_specified(limits).unwrap_err()),
            expected
        );
    }
    assert!(
        value
            .serialize_specified(CssSerializationLimits::new(8, 8, 10))
            .unwrap_err()
            .is_byte_limit()
    );
    assert_eq!(
        value.serialize_specified(Default::default()).unwrap(),
        "oblique 10deg 20deg"
    );
}
