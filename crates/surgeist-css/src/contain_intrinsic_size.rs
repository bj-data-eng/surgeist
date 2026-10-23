//! Checked authored contain-intrinsic sizes before layout and remembered-size resolution.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssSerializationError, CssSpecifiedNonNegativeLength};

type SerializationResult<T> = Result<T, CssSerializationError>;

/// The mandatory fallback after the optional `auto` marker.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub enum CssContainIntrinsicSizeFallback {
    /// No explicit intrinsic inner size when no remembered size applies.
    None,
    /// An exact nonnegative length, with math deferred to its owning stage.
    Length(CssSpecifiedNonNegativeLength),
}

impl PartialEq for CssContainIntrinsicSizeFallback {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::None, Self::None) => true,
            (Self::Length(left), Self::Length(right)) => left.structural_eq(right),
            _ => false,
        }
    }
}

impl Eq for CssContainIntrinsicSizeFallback {}

/// One contained intrinsic size axis: `auto? [ none | <length [0,∞]> ]`.
///
/// # Specified serialization
///
/// Serializes the specified value without resolving remembered sizes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContainIntrinsicSizeValue {
    uses_auto: bool,
    fallback: CssContainIntrinsicSizeFallback,
}

impl CssContainIntrinsicSizeValue {
    /// Constructs a fallback without the remembered-size marker.
    #[must_use]
    pub fn new(fallback: CssContainIntrinsicSizeFallback) -> Self {
        Self {
            uses_auto: false,
            fallback,
        }
    }

    /// Constructs a value that prefers a remembered size when one applies.
    #[must_use]
    pub fn with_auto(fallback: CssContainIntrinsicSizeFallback) -> Self {
        Self {
            uses_auto: true,
            fallback,
        }
    }

    /// Reports whether the `auto` marker was authored.
    #[must_use]
    pub fn uses_auto(&self) -> bool {
        self.uses_auto
    }

    /// Borrows the mandatory `none` or nonnegative length fallback.
    pub fn fallback(&self) -> &CssContainIntrinsicSizeFallback {
        &self.fallback
    }

    fn serialize_specified_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        if self.uses_auto {
            context.keyword(output, "auto")?;
            context.append(output, " ")?;
        }
        match &self.fallback {
            CssContainIntrinsicSizeFallback::None => context.keyword(output, "none"),
            CssContainIntrinsicSizeFallback::Length(length) => {
                let captured = length.capture_specified(context)?;
                context.append(output, &captured)
            }
        }
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssContainIntrinsicSizeValue {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.serialize_specified_into(context, output)?;
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssContainIntrinsicSizeValue);

/// The one- or two-value physical contain-intrinsic-size shorthand.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssContainIntrinsicSize {
    width: CssContainIntrinsicSizeValue,
    authored_height: Option<CssContainIntrinsicSizeValue>,
}

impl CssContainIntrinsicSize {
    /// Retains the authored width and optional height as whole checked values.
    #[must_use]
    pub fn new(
        width: CssContainIntrinsicSizeValue,
        authored_height: Option<CssContainIntrinsicSizeValue>,
    ) -> Self {
        Self {
            width,
            authored_height,
        }
    }

    /// Borrows the authored first value for width.
    pub fn width(&self) -> &CssContainIntrinsicSizeValue {
        &self.width
    }

    /// Borrows the explicitly authored second value, if any.
    pub fn authored_height(&self) -> Option<&CssContainIntrinsicSizeValue> {
        self.authored_height.as_ref()
    }

    /// Borrows the height assignment, repeating the whole width value if omitted.
    pub fn height(&self) -> &CssContainIntrinsicSizeValue {
        self.authored_height.as_ref().unwrap_or(&self.width)
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssContainIntrinsicSize {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.source_member(0, |writer| {
            self.width
                .serialize_specified_into(&mut writer.context, &mut writer.css)
        })?;
        if let Some(height) = &self.authored_height {
            writer.source_member(1, |writer| {
                writer.append(" ")?;
                height.serialize_specified_into(&mut writer.context, &mut writer.css)
            })?;
        }
        Ok(())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssContainIntrinsicSize);
