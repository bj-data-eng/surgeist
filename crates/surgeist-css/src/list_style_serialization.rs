//! Bounded, meaning-preserving specified serialization for Lists 3 marker styles.

use crate::content_serialization;
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssImageValue, CssListStylePosition, CssListStyleTypeValue, CssListStyleValue,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

fn charge(writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)
}

fn keyword(writer: &mut SpecifiedRuleWriter, value: &str) -> Result<()> {
    charge(writer)?;
    writer.append(value)
}

fn append_type(writer: &mut SpecifiedRuleWriter, value: &CssListStyleTypeValue) -> Result<()> {
    match value {
        CssListStyleTypeValue::None => keyword(writer, "none"),
        CssListStyleTypeValue::String(value) => {
            charge(writer)?;
            writer.append_string(value.as_str())
        }
        CssListStyleTypeValue::CounterStyle(value) => content_serialization::style(writer, value),
    }
}

fn is_disc(value: &CssListStyleTypeValue) -> bool {
    matches!(value, CssListStyleTypeValue::CounterStyle(style) if matches!(style.named(), Some(name) if name.as_str() == "disc"))
}

fn is_axis_keyword_name(value: &CssListStyleTypeValue) -> bool {
    matches!(value, CssListStyleTypeValue::CounterStyle(style) if matches!(style.named(), Some(name) if name.as_str().eq_ignore_ascii_case("inside") || name.as_str().eq_ignore_ascii_case("outside")))
}

fn append_position(writer: &mut SpecifiedRuleWriter, value: CssListStylePosition) -> Result<()> {
    keyword(
        writer,
        match value {
            CssListStylePosition::Inside => "inside",
            CssListStylePosition::Outside => "outside",
        },
    )
}

impl CssListStylePosition {
    /// Serializes a checked marker position without placing a marker.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Uses one input, projection, and byte budget for this keyword.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        append_position(&mut writer, *self)?;
        Ok(writer.css)
    }
}

impl CssListStyleTypeValue {
    /// Serializes an authored counter style, string, or `none` without resolution.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Applies one cumulative input, projection, and CSS byte budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        append_type(&mut writer, self)?;
        Ok(writer.css)
    }
}

impl CssListStyleValue {
    /// Serializes effective marker values in position, image, type order.
    ///
    /// Initial values are omitted only where parsing still preserves the same
    /// meaning. An `outside` position is inserted when a named `inside` or
    /// `outside` counter style would otherwise be parsed as the position.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Charges each authored component even when its initial spelling is omitted.
    /// An inserted disambiguating or all-initial `outside` costs one projection
    /// node, but no authored input node.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut writer = SpecifiedRuleWriter::new(limits);
        charge(&mut writer)?; // The checked shorthand aggregate.

        let emit_type = self.style_type().is_some_and(|value| !is_disc(value));
        let emit_image = self
            .image()
            .is_some_and(|value| !matches!(value, CssImageValue::None));
        let position = self.position();
        let emit_position = position == Some(CssListStylePosition::Inside)
            || (self.style_type().is_some_and(is_axis_keyword_name)
                && position.unwrap_or(CssListStylePosition::Outside)
                    == CssListStylePosition::Outside)
            || (!emit_type && !emit_image);

        let mut emitted = false;
        if emit_position {
            if let Some(value) = position {
                append_position(&mut writer, value)?;
            } else {
                writer.context.charge_projection(1)?;
                writer.append("outside")?;
            }
            emitted = true;
        } else if position.is_some() {
            charge(&mut writer)?;
        }

        if let Some(image) = self.image() {
            if emit_image {
                if emitted {
                    writer.append(" ")?;
                }
                image.append_specified(&mut writer)?;
                emitted = true;
            } else {
                charge(&mut writer)?;
            }
        }

        if let Some(style_type) = self.style_type() {
            if emit_type {
                if emitted {
                    writer.append(" ")?;
                }
                append_type(&mut writer, style_type)?;
            } else {
                charge(&mut writer)?;
            }
        }
        Ok(writer.css)
    }
}
