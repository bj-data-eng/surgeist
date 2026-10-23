//! Bounded, meaning-preserving specified serialization for Lists 3 marker styles.

use crate::content_serialization;
use crate::specified_rule_serialization::SpecifiedRuleWriter;
use crate::{
    CssImageValue, CssListStylePosition, CssListStyleTypeValue, CssListStyleValue, CssMarkerSide,
    CssSerializationError,
};

type Result<T> = std::result::Result<T, CssSerializationError>;

impl crate::specified_rule_serialization::SpecifiedProvider for CssMarkerSide {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.keyword(match self {
            Self::MatchSelf => "match-self",
            Self::MatchParent => "match-parent",
        })?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssMarkerSide);

fn append_type(writer: &mut SpecifiedRuleWriter, value: &CssListStyleTypeValue) -> Result<()> {
    match value {
        CssListStyleTypeValue::None => writer.keyword("none"),
        CssListStyleTypeValue::String(value) => {
            writer.node()?;
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
    writer.keyword(match value {
        CssListStylePosition::Inside => "inside",
        CssListStylePosition::Outside => "outside",
    })
}

impl crate::specified_rule_serialization::SpecifiedProvider for CssListStylePosition {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        append_position(writer, *self)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssListStylePosition);

impl crate::specified_rule_serialization::SpecifiedProvider for CssListStyleTypeValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        append_type(writer, self)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssListStyleTypeValue);

impl crate::specified_rule_serialization::SpecifiedProvider for CssListStyleValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.node()?; // The checked shorthand aggregate.

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
        if emit_position || position.is_some() {
            writer.source_property(crate::CssKnownProperty::ListStylePosition, |writer| {
                if emit_position {
                    if let Some(value) = position {
                        append_position(writer, value)?;
                    } else {
                        writer.context.charge_projection(1)?;
                        writer.append("outside")?;
                    }
                    emitted = true;
                } else {
                    writer.node()?;
                }
                Ok(())
            })?;
        }
        if let Some(image) = self.image() {
            writer.source_property(crate::CssKnownProperty::ListStyleImage, |writer| {
                if emit_image {
                    if emitted {
                        writer.append(" ")?;
                    }
                    image.append_specified(writer)?;
                    emitted = true;
                } else {
                    writer.node()?;
                }
                Ok(())
            })?;
        }
        if let Some(style_type) = self.style_type() {
            writer.source_property(crate::CssKnownProperty::ListStyleType, |writer| {
                if emit_type {
                    if emitted {
                        writer.append(" ")?;
                    }
                    append_type(writer, style_type)
                } else {
                    writer.node()
                }
            })?;
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssListStyleValue);
