//! Bounded specified serialization for the authored Fonts 4 shorthand.
//! System font selection and font matching remain external to this crate.

use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssAbsoluteFontWeight, CssExplicitFont, CssFontStyle, CssFontStyleKeyword, CssFontValue,
    CssFontVariant, CssFontWeight, CssFontWidthKeyword, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits, CssSystemFont,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

impl CssSystemFont {
    const fn as_css(self) -> &'static str {
        match self {
            Self::Caption => "caption",
            Self::Icon => "icon",
            Self::Menu => "menu",
            Self::MessageBox => "message-box",
            Self::SmallCaption => "small-caption",
            Self::StatusBar => "status-bar",
        }
    }
}

impl CssFontValue {
    /// Serializes a complete authored shorthand without resolving system fonts.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes with one cumulative input, projection, and byte budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        match self {
            Self::Explicit(font) => font.serialize_specified_with_limits(limits),
            Self::System(system) => {
                let mut writer = SpecifiedRuleWriter::new(limits);
                writer.context.charge_input(1)?;
                writer.context.charge_projection(1)?;
                writer.append(system.as_css())?;
                Ok(writer.css)
            }
        }
    }
}

impl CssExplicitFont {
    /// Serializes the authored components in Fonts 4 grammar order.
    /// Optional `normal` components are omitted because they have the same
    /// shorthand expansion as an omitted component.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes atomically under one cumulative resource budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        if let Some(style) = self.style() {
            if matches!(style, CssFontStyle::Keyword(CssFontStyleKeyword::Normal)) {
                writer.context.charge_input(1)?;
            } else {
                style.append_specified(&mut writer.context, &mut writer.css)?;
                writer.append(" ")?;
            }
        }
        if let Some(variant) = self.variant() {
            writer.context.charge_input(1)?;
            if variant != CssFontVariant::Normal {
                writer.context.charge_projection(1)?;
                writer.append("small-caps ")?;
            }
        }
        if let Some(weight) = self.weight() {
            if matches!(
                weight,
                CssFontWeight::Absolute(CssAbsoluteFontWeight::Normal)
            ) {
                writer.context.charge_input(1)?;
            } else {
                weight.append_specified(&mut writer.context, &mut writer.css)?;
                writer.append(" ")?;
            }
        }
        if let Some(width) = self.stretch() {
            writer.context.charge_input(1)?;
            if width != CssFontWidthKeyword::Normal {
                writer.context.charge_projection(1)?;
                writer.append(width.as_css())?;
                writer.append(" ")?;
            }
        }
        self.size()
            .append_specified(&mut writer.context, &mut writer.css)?;
        if let Some(line_height) = self.line_height() {
            if matches!(line_height, crate::CssLineHeight::Normal) {
                writer.context.charge_input(1)?;
            } else {
                writer.append("/")?;
                line_height.append_specified(&mut writer.context, &mut writer.css)?;
            }
        }
        writer.append(" ")?;
        self.families().append_specified(&mut writer)?;
        Ok(writer.css)
    }
}
