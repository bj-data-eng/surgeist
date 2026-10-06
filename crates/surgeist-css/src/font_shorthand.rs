//! Bounded specified serialization for the authored Fonts 4 shorthand.
//! System font selection and font matching remain external to this crate.

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
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        match self {
            Self::Explicit(font) => font.append_to_rule_writer(writer),
            Self::System(system) => {
                writer.context.charge_input(1)?;
                writer.context.charge_projection(1)?;
                writer.append(system.as_css())?;
                Ok(())
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
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        if let Some(style) = self.style() {
            writer.source_property(crate::CssKnownProperty::FontStyle, |writer| {
                if matches!(style, CssFontStyle::Keyword(CssFontStyleKeyword::Normal)) {
                    writer.context.charge_input(1)?;
                } else {
                    style.append_specified(&mut writer.context, &mut writer.css)?;
                    writer.append(" ")?;
                }
                Ok(())
            })?;
        }
        if let Some(variant) = self.variant() {
            writer.source_property(crate::CssKnownProperty::FontVariantCaps, |writer| {
                writer.context.charge_input(1)?;
                if variant != CssFontVariant::Normal {
                    writer.context.charge_projection(1)?;
                    writer.append("small-caps ")?;
                }
                Ok(())
            })?;
        }
        if let Some(weight) = self.weight() {
            writer.source_property(crate::CssKnownProperty::FontWeight, |writer| {
                if matches!(
                    weight,
                    CssFontWeight::Absolute(CssAbsoluteFontWeight::Normal)
                ) {
                    writer.context.charge_input(1)?;
                } else {
                    weight.append_specified(&mut writer.context, &mut writer.css)?;
                    writer.append(" ")?;
                }
                Ok(())
            })?;
        }
        if let Some(width) = self.stretch() {
            writer.source_property(crate::CssKnownProperty::FontWidth, |writer| {
                writer.context.charge_input(1)?;
                if width != CssFontWidthKeyword::Normal {
                    writer.context.charge_projection(1)?;
                    writer.append(width.as_css())?;
                    writer.append(" ")?;
                }
                Ok(())
            })?;
        }
        writer.source_property(crate::CssKnownProperty::FontSize, |writer| {
            self.size()
                .append_specified(&mut writer.context, &mut writer.css)
        })?;
        if let Some(line_height) = self.line_height() {
            writer.source_property(crate::CssKnownProperty::LineHeight, |writer| {
                if matches!(line_height, crate::CssLineHeight::Normal) {
                    writer.context.charge_input(1)?;
                } else {
                    writer.append("/")?;
                    line_height.append_specified(&mut writer.context, &mut writer.css)?;
                }
                Ok(())
            })?;
        }
        writer.source_property(crate::CssKnownProperty::FontFamily, |writer| {
            writer.append(" ")?;
            self.families().append_specified(writer)
        })?;
        Ok(())
    }
}
