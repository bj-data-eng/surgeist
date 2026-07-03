//! Lower style-owned text values into the text crate input surface.

use super::{AdapterError, AdapterErrorKind, AdapterResult};
use crate::{style, text};

pub fn lower_style_text_to_text_style(value: &style::TextValue) -> AdapterResult<text::Style> {
    let font_family = lower_font_family(&value.font_family)?;
    let size = lower_font_size(&value.font_size)?;
    let line_height = lower_line_height(&value.line_height)?;
    let brush = lower_color_brush(value.color, "color")?;
    let underline = lower_text_decoration(value.underline, "underline")?;
    let strikethrough = lower_text_decoration(value.strikethrough, "strikethrough")?;
    validate_default_text_align(value.alignment)?;
    validate_default_selection_color(value.selection_color)?;

    Ok(text::Style {
        font: text::Font {
            family: font_family,
            weight: lower_text_weight(value.font_weight),
            width: text::Width::Normal,
            slant: lower_text_slant(value.font_style)?,
            features: Vec::new(),
            variations: Vec::new(),
        },
        size,
        line_height,
        letter_spacing: 0.0,
        word_spacing: 0.0,
        brush,
        underline,
        strikethrough,
        locale: None,
        direction: text::Direction::Auto,
        white_space: lower_white_space(value.white_space),
        word_break: lower_word_break(value.word_break),
        wrap: lower_text_wrap(value.wrap),
        overflow_wrap: lower_overflow_wrap(value.overflow_wrap),
    })
}

fn lower_font_family(families: &[String]) -> AdapterResult<Vec<String>> {
    style::FontFamilyList::new(families.iter().cloned())
        .map_err(|error| adapter_error("font-family", error.to_string()))?;
    Ok(families.to_vec())
}

fn lower_font_size(length: &style::Length) -> AdapterResult<f32> {
    validate_length(length, "font-size")?;
    match length {
        style::Length::Px(value) if *value > 0.0 => Ok(*value),
        style::Length::Px(_) => Err(adapter_error(
            "font-size",
            "font size px value must be greater than zero",
        )),
        _ => Err(adapter_error("font-size", "font size must be a px length")),
    }
}

pub(super) fn lower_line_height(length: &style::Length) -> AdapterResult<text::LineHeight> {
    validate_length(length, "line-height")?;
    match length {
        style::Length::Normal => Ok(text::LineHeight::MetricsRelative(1.0)),
        style::Length::Px(value) if *value > 0.0 => Ok(text::LineHeight::Absolute(*value)),
        style::Length::Px(_) => Err(adapter_error(
            "line-height",
            "line height px value must be greater than zero",
        )),
        style::Length::Percent(value) if *value > 0.0 => {
            Ok(text::LineHeight::FontSizeRelative(*value / 100.0))
        }
        style::Length::Percent(_) => Err(adapter_error(
            "line-height",
            "line height percent value must be greater than zero",
        )),
        _ => Err(adapter_error(
            "line-height",
            "line height must be normal, px, or percent",
        )),
    }
}

fn validate_length(length: &style::Length, property: &'static str) -> AdapterResult<()> {
    length
        .validate()
        .map_err(|error| adapter_error(property, error.to_string()))
}

pub(super) const fn lower_text_weight(value: style::TextWeight) -> text::Weight {
    match value {
        style::TextWeight::Thin => text::Weight::Thin,
        style::TextWeight::ExtraLight => text::Weight::ExtraLight,
        style::TextWeight::Light => text::Weight::Light,
        style::TextWeight::Normal => text::Weight::Normal,
        style::TextWeight::Medium => text::Weight::Medium,
        style::TextWeight::SemiBold => text::Weight::SemiBold,
        style::TextWeight::Bold => text::Weight::Bold,
        style::TextWeight::ExtraBold => text::Weight::ExtraBold,
        style::TextWeight::Black => text::Weight::Black,
    }
}

pub(super) fn lower_text_slant(value: style::TextSlant) -> AdapterResult<text::Slant> {
    match value {
        style::TextSlant::Normal => Ok(text::Slant::Normal),
        style::TextSlant::Italic => Ok(text::Slant::Italic),
        style::TextSlant::Oblique(None) => Ok(text::Slant::Oblique(None)),
        style::TextSlant::Oblique(Some(angle)) if angle.is_finite() => {
            Ok(text::Slant::Oblique(Some(angle)))
        }
        style::TextSlant::Oblique(Some(_)) => Err(adapter_error(
            "font-style",
            "font oblique angle must be finite",
        )),
    }
}

fn validate_default_text_align(value: style::StyleTextAlign) -> AdapterResult<()> {
    match value {
        style::StyleTextAlign::Auto | style::StyleTextAlign::Start => Ok(()),
        style::StyleTextAlign::End
        | style::StyleTextAlign::Left
        | style::StyleTextAlign::Right
        | style::StyleTextAlign::Center
        | style::StyleTextAlign::Justify
        | style::StyleTextAlign::LegacyLeft
        | style::StyleTextAlign::LegacyRight
        | style::StyleTextAlign::LegacyCenter => Err(adapter_error(
            "text-align",
            "text alignment cannot be represented by text::Style",
        )),
    }
}

const fn lower_text_wrap(value: style::TextWrap) -> text::Wrap {
    match value {
        style::TextWrap::None => text::Wrap::None,
        style::TextWrap::Word => text::Wrap::Word,
    }
}

const fn lower_white_space(value: style::WhiteSpace) -> text::WhiteSpace {
    match value {
        style::WhiteSpace::Collapse => text::WhiteSpace::Collapse,
        style::WhiteSpace::Preserve => text::WhiteSpace::Preserve,
    }
}

const fn lower_word_break(value: style::WordBreak) -> text::WordBreak {
    match value {
        style::WordBreak::Normal => text::WordBreak::Normal,
        style::WordBreak::BreakAll => text::WordBreak::BreakAll,
        style::WordBreak::KeepAll => text::WordBreak::KeepAll,
    }
}

const fn lower_overflow_wrap(value: style::OverflowWrap) -> text::OverflowWrap {
    match value {
        style::OverflowWrap::Normal => text::OverflowWrap::Normal,
        style::OverflowWrap::Anywhere => text::OverflowWrap::Anywhere,
        style::OverflowWrap::BreakWord => text::OverflowWrap::BreakWord,
    }
}

pub(super) fn lower_text_decoration(
    value: style::Decoration,
    property: &'static str,
) -> AdapterResult<text::Decoration> {
    if let Some(offset) = value.offset()
        && !offset.is_finite()
    {
        return Err(adapter_error(
            property,
            "text decoration offset must be finite",
        ));
    }
    if let Some(size) = value.size() {
        if !size.is_finite() {
            return Err(adapter_error(
                property,
                "text decoration size must be finite",
            ));
        }
        if value.enabled() && size <= 0.0 {
            return Err(adapter_error(
                property,
                "enabled text decoration size must be greater than zero",
            ));
        }
        if size < 0.0 {
            return Err(adapter_error(
                property,
                "text decoration size must be non-negative",
            ));
        }
    }

    Ok(text::Decoration {
        enabled: value.enabled(),
        offset: value.offset(),
        size: value.size(),
        brush: value
            .brush()
            .map(|brush| lower_color_brush(brush, property))
            .transpose()?,
    })
}

fn lower_color_brush(value: style::Color, property: &'static str) -> AdapterResult<text::Brush> {
    for (channel, component) in [
        ("red", value.r),
        ("green", value.g),
        ("blue", value.b),
        ("alpha", value.a),
    ] {
        if !component.is_finite() {
            return Err(adapter_error(
                property,
                format!("{property} {channel} channel must be finite"),
            ));
        }
        if !(0.0..=1.0).contains(&component) {
            return Err(adapter_error(
                property,
                format!("{property} {channel} channel must be between 0.0 and 1.0"),
            ));
        }
    }
    Ok(text::Brush::color(value.r, value.g, value.b, value.a))
}

fn validate_default_selection_color(value: style::Color) -> AdapterResult<()> {
    lower_color_brush(value, "selection-color")?;
    if value == style::Color::TRANSPARENT {
        return Ok(());
    }

    Err(adapter_error(
        "selection-color",
        "selection color cannot be represented by text::Style",
    ))
}

fn adapter_error(property: impl Into<String>, reason: impl Into<String>) -> AdapterError {
    AdapterError::new(AdapterErrorKind::StyleTextValue {
        property: property.into(),
        reason: reason.into(),
    })
}
