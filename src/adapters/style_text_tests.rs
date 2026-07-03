use super::style_text::{
    lower_line_height, lower_style_text_to_text_style, lower_text_decoration, lower_text_slant,
    lower_text_weight,
};
use super::{AdapterBoundary, AdapterErrorKind};
use crate::{style, text};

fn assert_style_text_value_error(
    error: &super::AdapterError,
    expected_property: &str,
    expected_reason: &str,
) {
    assert_eq!(error.boundary(), AdapterBoundary::StyleToText);
    assert!(matches!(
        error.kind(),
        AdapterErrorKind::StyleTextValue { property, reason }
            if property == expected_property && reason.contains(expected_reason)
    ));
}

#[test]
fn style_text_default_value_lowers_to_text_style() {
    let value = style::TextValue::default();

    let lowered = lower_style_text_to_text_style(&value).unwrap();

    assert!(lowered.font.family.is_empty());
    assert_eq!(lowered.size, 16.0);
    assert_eq!(lowered.font.weight, text::Weight::Normal);
    assert_eq!(lowered.font.slant, text::Slant::Normal);
    assert_eq!(lowered.line_height, text::LineHeight::FontSizeRelative(1.0));
    assert_eq!(lowered.brush, text::Brush::color(0.0, 0.0, 0.0, 1.0));
    assert_eq!(value.alignment, style::StyleTextAlign::Start);
    assert_eq!(lowered.wrap, text::Wrap::Word);
    assert_eq!(lowered.white_space, text::WhiteSpace::Preserve);
    assert_eq!(lowered.word_break, text::WordBreak::Normal);
    assert_eq!(lowered.overflow_wrap, text::OverflowWrap::Normal);
    assert_eq!(lowered.underline, text::Decoration::none());
    assert_eq!(lowered.strikethrough, text::Decoration::none());
    assert_eq!(value.selection_color, style::Color::TRANSPARENT);
}

#[test]
fn style_text_weight_conversion_covers_every_variant() {
    assert_eq!(
        lower_text_weight(style::TextWeight::Thin),
        text::Weight::Thin
    );
    assert_eq!(
        lower_text_weight(style::TextWeight::ExtraLight),
        text::Weight::ExtraLight
    );
    assert_eq!(
        lower_text_weight(style::TextWeight::Light),
        text::Weight::Light
    );
    assert_eq!(
        lower_text_weight(style::TextWeight::Normal),
        text::Weight::Normal
    );
    assert_eq!(
        lower_text_weight(style::TextWeight::Medium),
        text::Weight::Medium
    );
    assert_eq!(
        lower_text_weight(style::TextWeight::SemiBold),
        text::Weight::SemiBold
    );
    assert_eq!(
        lower_text_weight(style::TextWeight::Bold),
        text::Weight::Bold
    );
    assert_eq!(
        lower_text_weight(style::TextWeight::ExtraBold),
        text::Weight::ExtraBold
    );
    assert_eq!(
        lower_text_weight(style::TextWeight::Black),
        text::Weight::Black
    );
}

#[test]
fn style_text_slant_conversion_covers_every_variant() {
    assert_eq!(
        lower_text_slant(style::TextSlant::Normal).unwrap(),
        text::Slant::Normal
    );
    assert_eq!(
        lower_text_slant(style::TextSlant::Italic).unwrap(),
        text::Slant::Italic
    );
    assert_eq!(
        lower_text_slant(style::TextSlant::Oblique(None)).unwrap(),
        text::Slant::Oblique(None)
    );
    assert_eq!(
        lower_text_slant(style::TextSlant::Oblique(Some(12.0))).unwrap(),
        text::Slant::Oblique(Some(12.0))
    );
}

#[test]
fn style_text_wrap_conversion_covers_every_variant() {
    let none = style::TextValue::default().wrap(style::TextWrap::None);
    let word = style::TextValue::default().wrap(style::TextWrap::Word);

    assert_eq!(
        lower_style_text_to_text_style(&none).unwrap().wrap,
        text::Wrap::None
    );
    assert_eq!(
        lower_style_text_to_text_style(&word).unwrap().wrap,
        text::Wrap::Word
    );
}

#[test]
fn style_text_white_space_conversion_covers_every_variant() {
    let collapse = style::TextValue::default().white_space(style::WhiteSpace::Collapse);
    let preserve = style::TextValue::default().white_space(style::WhiteSpace::Preserve);

    assert_eq!(
        lower_style_text_to_text_style(&collapse)
            .unwrap()
            .white_space,
        text::WhiteSpace::Collapse
    );
    assert_eq!(
        lower_style_text_to_text_style(&preserve)
            .unwrap()
            .white_space,
        text::WhiteSpace::Preserve
    );
}

#[test]
fn style_text_word_break_conversion_covers_every_variant() {
    let normal = style::TextValue::default().word_break(style::WordBreak::Normal);
    let break_all = style::TextValue::default().word_break(style::WordBreak::BreakAll);
    let keep_all = style::TextValue::default().word_break(style::WordBreak::KeepAll);

    assert_eq!(
        lower_style_text_to_text_style(&normal).unwrap().word_break,
        text::WordBreak::Normal
    );
    assert_eq!(
        lower_style_text_to_text_style(&break_all)
            .unwrap()
            .word_break,
        text::WordBreak::BreakAll
    );
    assert_eq!(
        lower_style_text_to_text_style(&keep_all)
            .unwrap()
            .word_break,
        text::WordBreak::KeepAll
    );
}

#[test]
fn style_text_overflow_wrap_conversion_covers_every_variant() {
    let normal = style::TextValue::default().overflow_wrap(style::OverflowWrap::Normal);
    let anywhere = style::TextValue::default().overflow_wrap(style::OverflowWrap::Anywhere);
    let break_word = style::TextValue::default().overflow_wrap(style::OverflowWrap::BreakWord);

    assert_eq!(
        lower_style_text_to_text_style(&normal)
            .unwrap()
            .overflow_wrap,
        text::OverflowWrap::Normal
    );
    assert_eq!(
        lower_style_text_to_text_style(&anywhere)
            .unwrap()
            .overflow_wrap,
        text::OverflowWrap::Anywhere
    );
    assert_eq!(
        lower_style_text_to_text_style(&break_word)
            .unwrap()
            .overflow_wrap,
        text::OverflowWrap::BreakWord
    );
}

#[test]
fn style_text_decoration_conversion_preserves_enabled_metrics_and_brush() {
    let decoration = style::Decoration::solid(Some(style::Color::rgba(0.25, 0.5, 0.75, 1.0)))
        .unwrap()
        .with_offset(1.25)
        .unwrap()
        .with_size(2.5)
        .unwrap();

    let lowered = lower_text_decoration(decoration, "underline").unwrap();

    assert!(lowered.enabled);
    assert_eq!(lowered.offset, Some(1.25));
    assert_eq!(lowered.size, Some(2.5));
    assert_eq!(
        lowered.brush,
        Some(text::Brush::color(0.25, 0.5, 0.75, 1.0))
    );
}

#[test]
fn style_text_line_height_conversion_preserves_css_length_semantics() {
    assert_eq!(
        lower_line_height(&style::Length::NORMAL).unwrap(),
        text::LineHeight::MetricsRelative(1.0)
    );
    assert_eq!(
        lower_line_height(&style::Length::px(24.0)).unwrap(),
        text::LineHeight::Absolute(24.0)
    );
    assert_eq!(
        lower_line_height(&style::Length::percent(125.0)).unwrap(),
        text::LineHeight::FontSizeRelative(1.25)
    );
}

#[test]
fn style_text_rejects_non_finite_oblique_angle() {
    let value = style::TextValue::default().style(style::TextSlant::Oblique(Some(f32::NAN)));

    let error = lower_style_text_to_text_style(&value).unwrap_err();

    assert_style_text_value_error(&error, "font-style", "finite");
}

#[test]
fn style_text_rejects_unsupported_font_size_length() {
    let value = style::TextValue::default().size(style::Length::percent(100.0));

    let error = lower_style_text_to_text_style(&value).unwrap_err();

    assert_style_text_value_error(&error, "font-size", "px");
}

#[test]
fn style_text_rejects_out_of_range_text_color_channels() {
    let value = style::TextValue::default().color(style::Color::rgba(1.25, 0.0, 0.0, 1.0));

    let error = lower_style_text_to_text_style(&value).unwrap_err();

    assert_style_text_value_error(&error, "color", "between 0.0 and 1.0");
}

#[test]
fn style_text_rejects_out_of_range_decoration_brush_channels() {
    let underline = style::Decoration::solid(Some(style::Color::rgba(0.0, -0.1, 0.0, 1.0)))
        .expect("finite decoration brush is style-valid");
    let value = style::TextValue::default().underline(underline);

    let error = lower_style_text_to_text_style(&value).unwrap_err();

    assert_style_text_value_error(&error, "underline", "between 0.0 and 1.0");
}

#[test]
fn style_text_rejects_zero_enabled_decoration_size() {
    let underline = style::Decoration::solid(None)
        .expect("solid underline decoration is valid")
        .with_size(0.0)
        .expect("zero decoration size is style-valid");
    let value = style::TextValue::default().underline(underline);

    let error = lower_style_text_to_text_style(&value).unwrap_err();

    assert_style_text_value_error(&error, "underline", "greater than zero");
}

#[test]
fn style_text_rejects_non_default_alignment() {
    let value = style::TextValue::default().align(style::StyleTextAlign::Center);

    let error = lower_style_text_to_text_style(&value).unwrap_err();

    assert_style_text_value_error(&error, "text-align", "cannot be represented");
}

#[test]
fn style_text_allows_default_compatible_alignment() {
    let start = style::TextValue::default().align(style::StyleTextAlign::Start);
    let auto = style::TextValue::default().align(style::StyleTextAlign::Auto);

    lower_style_text_to_text_style(&start).unwrap();
    lower_style_text_to_text_style(&auto).unwrap();
}

#[test]
fn style_text_rejects_non_default_selection_color() {
    let value =
        style::TextValue::default().selection_color(style::Color::rgba(0.0, 0.0, 1.0, 0.25));

    let error = lower_style_text_to_text_style(&value).unwrap_err();

    assert_style_text_value_error(&error, "selection-color", "cannot be represented");
}
