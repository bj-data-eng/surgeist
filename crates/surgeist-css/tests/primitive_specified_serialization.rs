#![forbid(unsafe_code)]
//! Independent represented-keyword expectations from the selected property grammars:
//! Text4 2026-08-14 #propdef-text-transform, #propdef-white-space, #propdef-text-wrap, #propdef-word-break;
//! Images3 2023-12-18 #propdef-object-fit, #propdef-image-rendering;
//! UI4 2026-01-20 #propdef-resize, #propdef-user-select, #propdef-pointer-events;
//! TextDecor3 2022-05-05 #propdef-text-decoration-style;
//! Transforms1 2019-02-14 #propdef-transform-box; Compositing1 2024-03-21 #propdef-mix-blend-mode, #propdef-isolation.
//! These functional tests accompany new specified-text APIs; broader grammar admission is separate.
use CssSpecifiedValueSerializationErrorKind as Kind;
use CssSpecifiedValueSerializationLimits as Limits;
use surgeist_css::*;

fn assert_limits(
    expected: &str,
    emit: impl Fn(Limits) -> Result<String, CssSpecifiedValueSerializationError>,
) {
    assert_eq!(emit(Limits::new(1, 1, expected.len())).unwrap(), expected);
    for (limits, kind) in [
        (Limits::new(0, 1, expected.len()), Kind::InputNodeLimit),
        (Limits::new(1, 0, expected.len()), Kind::ProjectionNodeLimit),
        (Limits::new(1, 1, expected.len() - 1), Kind::ByteLimit),
    ] {
        assert_eq!(emit(limits).unwrap_err().kind(), kind);
        assert_eq!(emit(Limits::new(1, 1, expected.len())).unwrap(), expected);
    }
}
macro_rules! cases {
    ($property:literal, $view:ident, $accessor:ident, [$(($value:expr, $expected:literal)),+ $(,)?]) => {
        for (value, expected) in [$(($value, $expected)),+] {
            assert_eq!(value.serialize_specified().unwrap(), expected);
            assert_limits(expected, |limits| value.serialize_specified_with_limits(limits));
            let authored = expected.to_ascii_uppercase();
            let report = parse_declaration(&format!("{}:{}!important", $property, authored));
            assert!(report.is_clean(), "{}:{}: {:?}", $property, authored, report.diagnostics());
            let declaration = report.syntax().as_ref().unwrap();
            let CssKnownPropertyValueRef::$view(wrapper) = declaration.known().unwrap().property_value().unwrap()
            else { panic!("expected typed {}", $property); };
            assert_eq!(wrapper.$accessor(), &value);
            assert_eq!(wrapper.$accessor().serialize_specified().unwrap(), expected);
            assert_eq!(wrapper.as_css(), authored);
            assert_eq!(declaration.importance(), CssImportance::Important);
        }
    };
}

#[test]
fn text_wrap_represented_states_emit_canonical_keywords() {
    cases!(
        "text-wrap",
        TextWrap,
        value,
        [
            (
                CssTextWrap::try_new(Some(CssTextWrapMode::Wrap), None).unwrap(),
                "wrap"
            ),
            (
                CssTextWrap::try_new(Some(CssTextWrapMode::NoWrap), None).unwrap(),
                "nowrap"
            ),
            (
                CssTextWrap::try_new(None, Some(CssTextWrapStyle::Balance)).unwrap(),
                "balance"
            ),
            (
                CssTextWrap::try_new(None, Some(CssTextWrapStyle::Pretty)).unwrap(),
                "pretty"
            ),
            (
                CssTextWrap::try_new(None, Some(CssTextWrapStyle::Stable)).unwrap(),
                "stable"
            ),
        ]
    );
}

#[test]
fn white_space_represented_states_emit_canonical_keywords() {
    cases!(
        "white-space",
        WhiteSpace,
        value,
        [
            (
                CssWhiteSpace::from_keyword(CssWhiteSpaceKeyword::Normal),
                "normal"
            ),
            (
                CssWhiteSpace::try_new(None, Some(CssTextWrapMode::NoWrap), None).unwrap(),
                "nowrap"
            ),
            (
                CssWhiteSpace::from_keyword(CssWhiteSpaceKeyword::Pre),
                "pre"
            ),
            (
                CssWhiteSpace::from_keyword(CssWhiteSpaceKeyword::PreWrap),
                "pre-wrap"
            ),
            (
                CssWhiteSpace::from_keyword(CssWhiteSpaceKeyword::PreLine),
                "pre-line"
            ),
            (
                CssWhiteSpace::try_new(Some(CssWhiteSpaceCollapse::BreakSpaces), None, None)
                    .unwrap(),
                "break-spaces"
            ),
        ]
    );
}

#[test]
fn word_break_represented_states_emit_canonical_keywords() {
    cases!(
        "word-break",
        WordBreak,
        value,
        [
            (CssWordBreak::Normal, "normal"),
            (CssWordBreak::BreakAll, "break-all"),
            (CssWordBreak::KeepAll, "keep-all"),
            (CssWordBreak::BreakWord, "break-word"),
        ]
    );
}

#[test]
fn text_transform_represented_states_emit_canonical_keywords() {
    cases!(
        "text-transform",
        TextTransform,
        value,
        [
            (CssTextTransform::None, "none"),
            (CssTextTransform::MathAuto, "math-auto"),
            (
                CssTextTransform::Transforms(
                    CssTextTransformSet::try_new(
                        Some(CssTextTransformCase::Capitalize),
                        false,
                        false
                    )
                    .unwrap()
                ),
                "capitalize"
            ),
            (
                CssTextTransform::Transforms(
                    CssTextTransformSet::try_new(
                        Some(CssTextTransformCase::Uppercase),
                        false,
                        false
                    )
                    .unwrap()
                ),
                "uppercase"
            ),
            (
                CssTextTransform::Transforms(
                    CssTextTransformSet::try_new(
                        Some(CssTextTransformCase::Lowercase),
                        false,
                        false
                    )
                    .unwrap()
                ),
                "lowercase"
            ),
        ]
    );
}

#[test]
fn decoration_style_represented_states_emit_canonical_keywords() {
    cases!(
        "text-decoration-style",
        TextDecorationStyle,
        value,
        [
            (CssTextDecorationStyle::Solid, "solid"),
            (CssTextDecorationStyle::Double, "double"),
            (CssTextDecorationStyle::Dotted, "dotted"),
            (CssTextDecorationStyle::Dashed, "dashed"),
            (CssTextDecorationStyle::Wavy, "wavy"),
        ]
    );
}

#[test]
fn image_rendering_represented_states_emit_canonical_keywords() {
    cases!(
        "image-rendering",
        ImageRendering,
        rendering,
        [
            (CssImageRendering::Auto, "auto"),
            (CssImageRendering::CrispEdges, "crisp-edges"),
            (CssImageRendering::Pixelated, "pixelated"),
        ]
    );
}

#[test]
fn object_fit_represented_states_emit_canonical_keywords() {
    cases!(
        "object-fit",
        ObjectFit,
        fit,
        [
            (CssObjectFit::Fill, "fill"),
            (CssObjectFit::Contain, "contain"),
            (CssObjectFit::Cover, "cover"),
            (CssObjectFit::None, "none"),
            (CssObjectFit::ScaleDown, "scale-down"),
        ]
    );
}

#[test]
fn pointer_events_represented_states_emit_canonical_keywords() {
    cases!(
        "pointer-events",
        PointerEvents,
        value,
        [
            (CssPointerEvents::Auto, "auto"),
            (CssPointerEvents::None, "none"),
        ]
    );
}

#[test]
fn user_select_represented_states_emit_canonical_keywords() {
    cases!(
        "user-select",
        UserSelect,
        value,
        [
            (CssUserSelect::Auto, "auto"),
            (CssUserSelect::Text, "text"),
            (CssUserSelect::None, "none"),
            (CssUserSelect::All, "all"),
            (CssUserSelect::Contain, "contain"),
        ]
    );
}

#[test]
fn resize_represented_states_emit_canonical_keywords() {
    cases!(
        "resize",
        Resize,
        resize,
        [
            (CssResize::None, "none"),
            (CssResize::Both, "both"),
            (CssResize::Horizontal, "horizontal"),
            (CssResize::Vertical, "vertical"),
        ]
    );
}

#[test]
fn isolation_represented_states_emit_canonical_keywords() {
    cases!(
        "isolation",
        Isolation,
        isolation,
        [
            (CssIsolation::Auto, "auto"),
            (CssIsolation::Isolate, "isolate"),
        ]
    );
}

#[test]
fn blend_mode_represented_states_emit_canonical_keywords() {
    cases!(
        "mix-blend-mode",
        MixBlendMode,
        mode,
        [
            (CssBlendMode::Normal, "normal"),
            (CssBlendMode::Darken, "darken"),
            (CssBlendMode::Multiply, "multiply"),
            (CssBlendMode::ColorBurn, "color-burn"),
            (CssBlendMode::Lighten, "lighten"),
            (CssBlendMode::Screen, "screen"),
            (CssBlendMode::ColorDodge, "color-dodge"),
            (CssBlendMode::Overlay, "overlay"),
            (CssBlendMode::SoftLight, "soft-light"),
            (CssBlendMode::HardLight, "hard-light"),
            (CssBlendMode::Difference, "difference"),
            (CssBlendMode::Exclusion, "exclusion"),
            (CssBlendMode::Hue, "hue"),
            (CssBlendMode::Saturation, "saturation"),
            (CssBlendMode::Color, "color"),
            (CssBlendMode::Luminosity, "luminosity"),
        ]
    );
}

#[test]
fn transform_box_checked_edges_emit_canonical_keywords() {
    cases!(
        "transform-box",
        TransformBox,
        reference_box,
        [
            (
                CssTransformBox::try_new(CssBoxEdgeKeyword::ContentBox).unwrap(),
                "content-box"
            ),
            (
                CssTransformBox::try_new(CssBoxEdgeKeyword::BorderBox).unwrap(),
                "border-box"
            ),
            (
                CssTransformBox::try_new(CssBoxEdgeKeyword::FillBox).unwrap(),
                "fill-box"
            ),
            (
                CssTransformBox::try_new(CssBoxEdgeKeyword::StrokeBox).unwrap(),
                "stroke-box"
            ),
            (
                CssTransformBox::try_new(CssBoxEdgeKeyword::ViewBox).unwrap(),
                "view-box"
            ),
        ]
    );
}

#[test]
fn escaped_keyword_identity_canonicalizes_without_changing_authored_text() {
    let report = parse_declaration(r"object-fit:s\63 ale-down!important");
    assert!(report.is_clean());
    let declaration = report.syntax().as_ref().unwrap();
    let CssKnownPropertyValueRef::ObjectFit(wrapper) =
        declaration.known().unwrap().property_value().unwrap()
    else {
        panic!("object-fit");
    };
    assert_eq!(wrapper.fit().serialize_specified().unwrap(), "scale-down");
    assert_eq!(wrapper.as_css(), r"s\63 ale-down");
    assert_eq!(declaration.importance(), CssImportance::Important);
}
