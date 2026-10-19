#![forbid(unsafe_code)]

//! Ordinary Fonts 4 font-face style descriptor and checked range contracts.

use surgeist_css::{
    CssAuthoredFontFaceDescriptorValue as Authored, CssComponentValue, CssComponentValues,
    CssFontFaceDescriptorKind as Kind, CssFontFaceDescriptorValue as Ordinary,
    CssFontFaceObliqueRange, CssFontFaceStyle, CssFontFaceValueErrorKind, CssFontObliqueAngle,
    CssFontStyleKeyword, CssRecoveryAction, CssRule, CssSerializedOrigin,
    CssSpecifiedValueSerializationErrorKind, CssSpecifiedValueSerializationLimits, CssValueOrigin,
    parse_component_values, parse_font_face_descriptor_value, parse_sheet,
};

fn angle(text: &str) -> CssFontObliqueAngle {
    CssFontObliqueAngle::try_from_component(CssComponentValue::try_token(text).unwrap()).unwrap()
}

fn ordinary(text: &str) -> CssFontFaceStyle {
    let report = parse_font_face_descriptor_value(text, Kind::FontStyle);
    assert!(report.is_clean(), "{text}: {:?}", report.diagnostics());
    let Some(Authored::Ordinary(Ordinary::FontStyle(value))) = report.syntax() else {
        panic!("ordinary style descriptor: {text}")
    };
    value.clone()
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
        assert_eq!(value.serialize_specified().unwrap(), text);
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
            range.start().serialize_specified().unwrap(),
            start,
            "{text}"
        );
        assert_eq!(
            range
                .end()
                .map(|end| end.serialize_specified().unwrap())
                .as_deref(),
            end,
            "{text}"
        );
        assert_eq!(
            range.serialize_specified().unwrap(),
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
    assert_eq!(range.serialize_specified().unwrap(), "90deg -0.25turn");
    assert_eq!(
        CssFontFaceStyle::Oblique { range: Some(range) }
            .serialize_specified()
            .unwrap(),
        "oblique 90deg -0.25turn"
    );
    let single = CssFontFaceObliqueRange::new(angle("0rad"), None);
    assert!(single.end().is_none());
    assert_eq!(single.serialize_specified().unwrap(), "0rad");
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
        let raw = parse_font_face_descriptor_value(invalid, Kind::FontStyle);
        assert!(raw.syntax().is_none(), "{invalid}");
        let source =
            format!("@font-face{{font-style:normal;font-style:{invalid};font-display:swap}}");
        let report = parse_sheet(&source);
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
    let empty = parse_sheet("@font-face{}");
    assert!(empty.is_clean());
    assert!(matches!(empty.syntax().rules(), [CssRule::FontFace(_)]));
}

#[test]
fn duplicate_pending_and_mixed_origin_reentry_select_last_admitted_style() {
    let source = "@font-face{font-style:oblique 10deg;font-style:env(style);font-style:bad;font-style:italic}";
    let report = parse_sheet(source);
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
        parse_component_values("env(style, oblique)").unwrap(),
    )
    .unwrap();
    let Authored::Pending(pending) = checked else {
        panic!("pending descriptor")
    };
    let mut components = parse_component_values("oblique 10deg ")
        .unwrap()
        .items()
        .to_vec();
    let replacement = pending
        .reparse_after_substitution(CssComponentValues::try_new(components.clone()).unwrap())
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
        .reparse_after_substitution(CssComponentValues::try_new(components).unwrap())
        .unwrap();
    let Ordinary::FontStyle(CssFontFaceStyle::Oblique { range: Some(range) }) = replacement else {
        panic!("mixed range")
    };
    assert_eq!(range.start().serialize_specified().unwrap(), "10deg");
    assert_eq!(range.end().unwrap().origin(), &CssValueOrigin::Programmatic);
    assert_eq!(
        range.end().unwrap().serialize_specified().unwrap(),
        "-20grad"
    );
    for text in ["env(style)", "var(--style)", "oblique 91deg"] {
        assert!(
            pending
                .reparse_after_substitution(parse_component_values(text).unwrap())
                .is_err(),
            "{text}"
        );
    }
}

#[test]
fn strict_checked_construction_rejects_invalid_and_adjacent_tokens_with_real_origin() {
    let invalid = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("oblique").unwrap(),
        CssComponentValue::try_token("91deg").unwrap(),
    ])
    .unwrap();
    let error = Authored::try_from_components(Kind::FontStyle, invalid).unwrap_err();
    assert!(matches!(
        error.kind(),
        CssFontFaceValueErrorKind::Grammar(_)
    ));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );

    let adjacent = CssComponentValues::try_new(vec![
        CssComponentValue::try_ident("oblique").unwrap(),
        CssComponentValue::try_number("1").unwrap(),
        CssComponentValue::try_token("0deg").unwrap(),
    ])
    .unwrap();
    assert!(Authored::try_from_components(Kind::FontStyle, adjacent).is_err());
}

#[test]
fn descriptor_and_range_serialization_enforce_aggregate_budgets_atomically() {
    let range = CssFontFaceObliqueRange::new(angle("10deg"), Some(angle("20deg")));
    assert_eq!(range.serialize_specified().unwrap(), "10deg 20deg");
    assert_eq!(
        range
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(2, 2, 11))
            .unwrap(),
        "10deg 20deg"
    );
    for (limits, expected) in [
        (
            CssSpecifiedValueSerializationLimits::new(1, 2, 11),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(2, 1, 11),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(2, 2, 10),
            CssSpecifiedValueSerializationErrorKind::ByteLimit,
        ),
    ] {
        assert_eq!(
            range
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            expected
        );
    }
    let value = CssFontFaceStyle::Oblique { range: Some(range) };
    assert_eq!(value.serialize_specified().unwrap(), "oblique 10deg 20deg");
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(3, 3, 19))
            .unwrap(),
        "oblique 10deg 20deg"
    );
    for (limits, expected) in [
        (
            CssSpecifiedValueSerializationLimits::new(2, 3, 19),
            CssSpecifiedValueSerializationErrorKind::InputNodeLimit,
        ),
        (
            CssSpecifiedValueSerializationLimits::new(3, 2, 19),
            CssSpecifiedValueSerializationErrorKind::ProjectionNodeLimit,
        ),
    ] {
        assert_eq!(
            value
                .serialize_specified_with_limits(limits)
                .unwrap_err()
                .kind(),
            expected
        );
    }
    assert_eq!(
        value
            .serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::new(8, 8, 10))
            .unwrap_err()
            .kind(),
        CssSpecifiedValueSerializationErrorKind::ByteLimit
    );
    assert_eq!(value.serialize_specified().unwrap(), "oblique 10deg 20deg");
}
