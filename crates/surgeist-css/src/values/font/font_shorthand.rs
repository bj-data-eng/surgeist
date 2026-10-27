//! Bounded specified serialization for the authored Fonts 4 shorthand.
//! System font selection and font matching remain external to this crate.

use crate::values::font::{
    CssAbsoluteFontWeight, CssExplicitFont, CssFontStyle, CssFontStyleKeyword, CssFontValue,
    CssFontVariant, CssFontWeight, CssFontWidthKeyword, CssSystemFont,
};

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

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssFontValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        match self {
            Self::Explicit(font) => font.append_to_rule_writer(writer),
            Self::System(system) => {
                writer.node()?;
                writer.append(system.as_css())?;
                Ok(())
            }
        }
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssFontValue);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssExplicitFont {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        if let Some(style) = self.style() {
            writer.source_property(crate::property::CssKnownProperty::FontStyle, |writer| {
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
            writer.source_property(
                crate::property::CssKnownProperty::FontVariantCaps,
                |writer| {
                    writer.context.charge_input(1)?;
                    if variant != CssFontVariant::Normal {
                        writer.context.charge_projection(1)?;
                        writer.append("small-caps ")?;
                    }
                    Ok(())
                },
            )?;
        }
        if let Some(weight) = self.weight() {
            writer.source_property(crate::property::CssKnownProperty::FontWeight, |writer| {
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
            writer.source_property(crate::property::CssKnownProperty::FontWidth, |writer| {
                writer.context.charge_input(1)?;
                if width != CssFontWidthKeyword::Normal {
                    writer.context.charge_projection(1)?;
                    writer.append(width.as_css())?;
                    writer.append(" ")?;
                }
                Ok(())
            })?;
        }
        writer.source_property(crate::property::CssKnownProperty::FontSize, |writer| {
            self.size()
                .append_specified(&mut writer.context, &mut writer.css)
        })?;
        if let Some(line_height) = self.line_height() {
            writer.source_property(crate::property::CssKnownProperty::LineHeight, |writer| {
                if matches!(line_height, crate::values::text::CssLineHeight::Normal) {
                    writer.context.charge_input(1)?;
                } else {
                    // CSSOM "serialize a CSS value" step 5 spaces every item
                    // except before ","; `/` is an item. The two bytes are
                    // ordinary output; node work is unchanged.
                    writer.append(" / ")?;
                    line_height.append_specified(&mut writer.context, &mut writer.css)?;
                }
                Ok(())
            })?;
        }
        writer.source_property(crate::property::CssKnownProperty::FontFamily, |writer| {
            writer.append(" ")?;
            self.families().append_specified(writer)
        })?;
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssExplicitFont);
