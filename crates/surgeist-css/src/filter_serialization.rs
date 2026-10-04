//! Bounded authored filter serialization, before contextual resolution or execution.

use crate::{
    CssFilter, CssFilterAmount, CssFilterFunction, CssFilterFunctionList,
    CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

macro_rules! specified_filter_methods {
    () => {
        /// Serializes specified filters in authored order, preserving omitted arguments.
        /// Child providers retain exact ordinary magnitudes and symbolic values.
        pub fn serialize_specified(&self) -> Result<String> {
            self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
        }

        /// Uses one cumulative input, projection, and UTF-8 byte budget.
        /// Failure returns no partial CSS and leaves the authored value unchanged.
        pub fn serialize_specified_with_limits(
            &self,
            limits: CssSpecifiedValueSerializationLimits,
        ) -> Result<String> {
            let mut writer = SpecifiedRuleWriter::new(limits);
            self.append_specified(&mut writer)?;
            Ok(writer.css)
        }
    };
}

impl CssFilter {
    specified_filter_methods!();

    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::None => {
                charge_aggregate(writer)?;
                writer.append("none")
            }
            Self::Functions(functions) => functions.append_specified(writer),
        }
    }
}

impl CssFilterFunctionList {
    specified_filter_methods!();

    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        charge_aggregate(writer)?;
        for (index, function) in self.functions().iter().enumerate() {
            if index != 0 {
                writer.append(" ")?;
            }
            function.append_specified(writer)?;
        }
        Ok(())
    }
}

impl CssFilterFunction {
    specified_filter_methods!();

    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        // These children already own the complete function and its aggregate charge.
        match self {
            Self::DropShadow(value) => return value.append_specified(writer),
            Self::Url(value) => return value.append_specified(writer),
            _ => {}
        }
        charge_aggregate(writer)?;
        match self {
            Self::Blur(value) => {
                writer.append("blur(")?;
                if let Some(length) = value.authored_length() {
                    length.append_specified(&mut writer.context, &mut writer.css)?;
                }
            }
            Self::HueRotate(value) => {
                writer.append("hue-rotate(")?;
                if let Some(angle) = value.authored_angle() {
                    angle.append_specified(&mut writer.context, &mut writer.css)?;
                }
            }
            Self::Brightness(value) => append_amount("brightness(", value, writer)?,
            Self::Contrast(value) => append_amount("contrast(", value, writer)?,
            Self::Grayscale(value) => append_amount("grayscale(", value, writer)?,
            Self::Invert(value) => append_amount("invert(", value, writer)?,
            Self::Opacity(value) => append_amount("opacity(", value, writer)?,
            Self::Saturate(value) => append_amount("saturate(", value, writer)?,
            Self::Sepia(value) => append_amount("sepia(", value, writer)?,
            Self::DropShadow(_) | Self::Url(_) => unreachable!("delegated complete function"),
        }
        writer.append(")")
    }
}

fn append_amount(
    opening: &str,
    value: &CssFilterAmount,
    writer: &mut SpecifiedRuleWriter,
) -> Result<()> {
    writer.append(opening)?;
    match value {
        CssFilterAmount::Default => Ok(()),
        CssFilterAmount::Number(value) => {
            value.append_specified(&mut writer.context, &mut writer.css)
        }
        CssFilterAmount::HintedNumberCalculation(value) => {
            value.append_specified(&mut writer.context, &mut writer.css)
        }
        CssFilterAmount::Percentage(value) => {
            value.append_specified(&mut writer.context, &mut writer.css)
        }
    }
}

fn charge_aggregate(writer: &mut SpecifiedRuleWriter) -> Result<()> {
    writer.context.charge_input(1)?;
    writer.context.charge_projection(1)
}
