#![forbid(unsafe_code)]

//! Checked font-face component provenance and strict reentry contracts.

use surgeist_css::{
    CssAuthoredFontFaceDescriptorValue as Authored, CssComponentValue, CssComponentValueErrorKind,
    CssComponentValueLimits, CssComponentValueRef, CssComponentValues,
    CssFontFaceDescriptorKind as Kind, CssFontFaceDescriptorValue as Ordinary,
    CssFontFaceValueErrorKind, CssFontFaceWidth, CssFontWidth, CssSerializedOrigin, CssValueOrigin,
    CssValueTokenRef, parse_component_values, parse_font_face_descriptor_value,
};

fn pending(kind: Kind) -> surgeist_css::CssPendingFontFaceDescriptorValue {
    let value = Authored::try_from_components(
        kind,
        parse_component_values("env(selection, fallback)").unwrap(),
    )
    .unwrap();
    let Authored::Pending(value) = value else {
        panic!("valid env() must defer")
    };
    value
}

#[test]
fn strict_reentry_rejects_nested_residual_functions_and_recovered_src_members() {
    let width = pending(Kind::FontWidth);
    for replacement in [
        "calc(env(width) + 1%)",
        "calc(var(--width) + 1%)",
        "{var(--width)}",
        r"c\61 lc(ENV(width))",
    ] {
        let error = width
            .reparse_after_substitution(parse_component_values(replacement).unwrap())
            .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssFontFaceValueErrorKind::ResidualSubstitution
        );
        assert!(matches!(
            error.origin(),
            CssSerializedOrigin::Token(CssValueOrigin::Parsed(_))
        ));
    }

    let source = pending(Kind::Src);
    let raw = parse_font_face_descriptor_value("bogus, url(good.woff2)", Kind::Src);
    assert!(
        raw.syntax().is_some(),
        "ordinary raw parsing retains a valid member"
    );
    assert!(!raw.is_clean(), "ordinary raw parsing reports bad member");
    let replacement = parse_component_values("bogus, url(good.woff2)").unwrap();
    for error in [
        Authored::try_from_components(Kind::Src, replacement.clone()).unwrap_err(),
        source.reparse_after_substitution(replacement).unwrap_err(),
    ] {
        assert!(matches!(
            error.kind(),
            CssFontFaceValueErrorKind::Grammar(_)
        ));
        assert!(matches!(
            error.origin(),
            CssSerializedOrigin::Token(CssValueOrigin::Parsed(_))
        ));
    }
}

#[test]
fn mixed_width_replacement_keeps_exact_numeric_origin_and_pending_components_keep_mixed_origins() {
    let mut items = parse_component_values("62.500000000000000000001% ")
        .unwrap()
        .items()
        .to_vec();
    items.push(CssComponentValue::try_ident("condensed").unwrap());
    let replacement = CssComponentValues::try_new(items).unwrap();
    let Ordinary::FontWidth(CssFontFaceWidth::Range {
        start: CssFontWidth::Percentage(start),
        end: Some(CssFontWidth::Keyword(_)),
    }) = pending(Kind::FontWidth)
        .reparse_after_substitution(replacement)
        .unwrap()
    else {
        panic!("one exact percentage and one keyword")
    };
    let CssValueOrigin::Parsed(origin) = start.origin() else {
        panic!("percentage retains original parsed token")
    };
    assert_eq!(origin.source().as_str(), "62.500000000000000000001% ");
    assert!(matches!(
        start.literal_component().unwrap().view(),
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
            if number.representation() == "62.500000000000000000001"
    ));
    assert_eq!(start.serialize_specified().unwrap(), "62.5%");

    let mut mixed = parse_component_values("env(selection) ")
        .unwrap()
        .items()
        .to_vec();
    mixed.push(CssComponentValue::try_ident("later").unwrap());
    let value =
        Authored::try_from_components(Kind::FontWidth, CssComponentValues::try_new(mixed).unwrap())
            .unwrap();
    let Authored::Pending(pending_value) = value else {
        panic!("mixed pending value")
    };
    assert!(matches!(
        pending_value.components().items()[0].origin(),
        CssValueOrigin::Parsed(_)
    ));
    assert!(matches!(
        pending_value.components().items().last().unwrap().origin(),
        CssValueOrigin::Programmatic
    ));

    let programmatic = CssComponentValues::try_new(vec![
        CssComponentValue::try_token("62.500000000000000000001%").unwrap(),
    ])
    .unwrap();
    let Ordinary::FontWidth(CssFontFaceWidth::Range {
        start: CssFontWidth::Percentage(start),
        end: None,
    }) = pending(Kind::FontWidth)
        .reparse_after_substitution(programmatic)
        .unwrap()
    else {
        panic!("programmatic percentage")
    };
    assert_eq!(start.origin(), &CssValueOrigin::Programmatic);
    assert!(matches!(
        start.literal_component().unwrap().view(),
        CssComponentValueRef::Token(CssValueTokenRef::Percentage(number))
            if number.representation() == "62.500000000000000000001"
    ));
    assert_eq!(start.serialize_specified().unwrap(), "62.5%");

    let invalid =
        CssComponentValues::try_new(vec![CssComponentValue::try_ident("bad").unwrap()]).unwrap();
    let error = Authored::try_from_components(Kind::FontWidth, invalid).unwrap_err();
    assert!(matches!(
        error.kind(),
        CssFontFaceValueErrorKind::Grammar(_)
    ));
    assert_eq!(
        error.origin(),
        &CssSerializedOrigin::Token(CssValueOrigin::Programmatic)
    );
}

#[test]
fn adjacent_number_and_percent_delimiter_do_not_become_a_percentage_token() {
    let values = CssComponentValues::try_new(vec![
        CssComponentValue::try_token("75").unwrap(),
        CssComponentValue::try_token("%").unwrap(),
    ])
    .unwrap();
    let error = Authored::try_from_components(Kind::FontWidth, values).unwrap_err();
    assert!(matches!(
        error.kind(),
        CssFontFaceValueErrorKind::Grammar(_)
    ));
}

#[test]
fn component_depth_count_and_byte_limits_fail_atomically() {
    let values = parse_component_values("env(width)").unwrap();
    for (limits, expected) in [
        (
            CssComponentValueLimits::try_new(0, usize::MAX, usize::MAX).unwrap(),
            CssComponentValueErrorKind::NestingLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, 0, usize::MAX).unwrap(),
            CssComponentValueErrorKind::ComponentLimit,
        ),
        (
            CssComponentValueLimits::try_new(256, usize::MAX, 1).unwrap(),
            CssComponentValueErrorKind::ByteLimit,
        ),
    ] {
        let error =
            Authored::try_from_components_with_limits(Kind::FontWidth, values.clone(), limits)
                .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssFontFaceValueErrorKind::Component(expected)
        );
    }
    let width = pending(Kind::FontWidth);
    let replacement = parse_component_values("75%").unwrap();
    let error = width
        .reparse_after_substitution_with_limits(
            replacement,
            CssComponentValueLimits::try_new(256, usize::MAX, 1).unwrap(),
        )
        .unwrap_err();
    assert_eq!(
        error.kind(),
        &CssFontFaceValueErrorKind::Component(CssComponentValueErrorKind::ByteLimit)
    );
}

#[test]
fn checked_construction_rejects_implicit_eof_closures_before_serialization() {
    for text in [
        "env(width",
        "env(width, {fallback",
        "env(width, \"fallback",
        "env(width)/*",
    ] {
        let values = parse_component_values(text).unwrap();
        let error = Authored::try_from_components(Kind::FontWidth, values).unwrap_err();
        assert_eq!(
            error.kind(),
            &CssFontFaceValueErrorKind::RecoveredComponent,
            "{text}"
        );
        assert!(
            matches!(
                error.origin(),
                CssSerializedOrigin::Token(CssValueOrigin::ImplicitClosure { .. })
            ),
            "{text}: {error:?}"
        );
    }

    for (kind, text) in [
        (Kind::Src, "url(face"),
        (Kind::FontWidth, "calc(75%"),
        (Kind::FontWidth, "75%/*"),
    ] {
        let error = pending(kind)
            .reparse_after_substitution(parse_component_values(text).unwrap())
            .unwrap_err();
        assert_eq!(
            error.kind(),
            &CssFontFaceValueErrorKind::RecoveredComponent,
            "{text}"
        );
        assert!(matches!(
            error.origin(),
            CssSerializedOrigin::Token(CssValueOrigin::ImplicitClosure { .. })
        ));
    }
}
