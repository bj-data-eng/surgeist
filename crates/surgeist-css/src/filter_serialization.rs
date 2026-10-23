//! Bounded authored filter serialization, before contextual resolution or execution.

use crate::{
    CssFilter, CssFilterAmount, CssFilterFunction, CssFilterFunctionList, CssSerializationError,
    specified_rule_serialization::SpecifiedRuleWriter,
};

type Result<T> = std::result::Result<T, CssSerializationError>;

macro_rules! specified_filter_methods {
    ($ty:ty) => {
        impl crate::specified_rule_serialization::SpecifiedProvider for $ty {
            fn append_to_rule_writer(
                &self,
                writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
            ) -> std::result::Result<(), crate::CssSerializationError> {
                self.append_specified(writer)?;
                Ok(())
            }
        }
        crate::specified_rule_serialization::specified_serialization!($ty);
    };
}

impl CssFilter {
    pub(crate) fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        match self {
            Self::None => {
                writer.node()?;
                writer.append("none")
            }
            Self::Functions(functions) => functions.append_specified(writer),
        }
    }
}
specified_filter_methods!(CssFilter);

impl CssFilterFunctionList {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        writer.node()?;
        writer.space_separated(self.functions(), |writer, function| {
            function.append_specified(writer)
        })?;
        Ok(())
    }
}
specified_filter_methods!(CssFilterFunctionList);

impl CssFilterFunction {
    fn append_specified(&self, writer: &mut SpecifiedRuleWriter) -> Result<()> {
        // These children already own the complete function and its aggregate charge.
        match self {
            Self::DropShadow(value) => return value.append_specified(writer),
            Self::Url(value) => return value.append_specified(writer),
            _ => {}
        }
        writer.node()?;
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
specified_filter_methods!(CssFilterFunction);

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
