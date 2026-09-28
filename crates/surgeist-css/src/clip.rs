//! Bounded specified serialization for the deprecated CSS Masking `clip` value.

use crate::specified_serialization::{
    SpecifiedSerializationContext, serialize_checked_pure_length_into,
};
use crate::{
    CssClip, CssClipEdge, CssClipLength, CssClipRect, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits,
};

type Result<T> = std::result::Result<T, CssSpecifiedValueSerializationError>;

impl CssClipLength {
    /// Serializes one checked signed clip length before layout resolution.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes one checked length under input, projection, and byte limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.serialize_specified_into(&mut context, &mut output)?;
        Ok(output)
    }

    fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        serialize_checked_pure_length_into(self.value(), context, output)
    }
}

impl CssClipEdge {
    /// Serializes one `auto` or signed-length rectangle edge.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes one edge under input, projection, and byte limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.serialize_specified_into(&mut context, &mut output)?;
        Ok(output)
    }

    fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        match self {
            Self::Auto => {
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(output, "auto")
            }
            Self::Length(length) => length.serialize_specified_into(context, output),
        }
    }
}

impl CssClipRect {
    /// Serializes four effective edges in top, right, bottom, left order.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes the complete rectangle with one cumulative resource budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.serialize_specified_into(&mut context, &mut output)?;
        Ok(output)
    }

    fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> Result<()> {
        context.charge_input(1)?;
        context.charge_projection(1)?;
        context.append(output, "rect(")?;
        for (index, edge) in [self.top(), self.right(), self.bottom(), self.left()]
            .into_iter()
            .enumerate()
        {
            if index != 0 {
                context.append(output, ", ")?;
            }
            edge.serialize_specified_into(context, output)?;
        }
        context.append(output, ")")
    }
}

impl CssClip {
    /// Serializes `auto` or a checked rectangle without applying clipping.
    pub fn serialize_specified(&self) -> Result<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }

    /// Serializes one clip value under cumulative input, projection, and byte limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> Result<String> {
        match self {
            Self::Auto => {
                crate::specified_serialization::serialize_keyword_sequence("auto", limits)
            }
            Self::Rect(rect) => rect.serialize_specified_with_limits(limits),
        }
    }
}
