//! Bounded specified serialization for checked CSS2 table border spacing.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{
    CssBorderSpacing, CssSpecifiedValueSerializationError, CssSpecifiedValueSerializationLimits,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

impl CssBorderSpacing {
    /// Serializes the effective horizontal and vertical lengths, including a repeated axis.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes both axes with one shared input, projection, and CSS byte budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        context.charge_input(1)?;
        context.charge_projection(1)?;
        self.horizontal()
            .append_specified(&mut context, &mut output)?;
        context.append(&mut output, " ")?;
        self.vertical()
            .append_specified(&mut context, &mut output)?;
        Ok(output)
    }
}
