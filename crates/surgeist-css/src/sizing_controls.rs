//! Sizing 4 authored shorthands and finite intrinsic-size controls.

use crate::specified_serialization::{SpecifiedSerializationContext, serialize_keyword_sequence};
use crate::{
    CssMaxSizeValue, CssSizeValue, CssSpecifiedValueSerializationError,
    CssSpecifiedValueSerializationLimits,
};

type SerializationResult<T> = Result<T, CssSpecifiedValueSerializationError>;

trait PairValue {
    fn serialize_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()>;
}

impl PairValue for CssSizeValue {
    fn serialize_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        self.serialize_specified_into(context, output)
    }
}
impl PairValue for CssMaxSizeValue {
    fn serialize_into(
        &self,
        context: &mut SpecifiedSerializationContext,
        output: &mut String,
    ) -> SerializationResult<()> {
        self.serialize_specified_into(context, output)
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Pair<T> {
    width: T,
    authored_height: Option<T>,
}

impl<T> Pair<T> {
    fn height(&self) -> &T {
        self.authored_height.as_ref().unwrap_or(&self.width)
    }
}

impl<T: PairValue> Pair<T> {
    fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let mut context = SpecifiedSerializationContext::new(limits);
        let mut output = String::new();
        self.width.serialize_into(&mut context, &mut output)?;
        if let Some(height) = &self.authored_height {
            context.append(&mut output, " ")?;
            height.serialize_into(&mut context, &mut output)?;
        }
        Ok(output)
    }
}

/// One- or two-value `size` or `min-size` shorthand, in width–height order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssSizePair(Pair<CssSizeValue>);

impl CssSizePair {
    /// Preserves the first value and optional explicitly authored second.
    #[must_use]
    pub fn new(width: CssSizeValue, authored_height: Option<CssSizeValue>) -> Self {
        Self(Pair {
            width,
            authored_height,
        })
    }
    /// Borrows the first specified value.
    pub fn width(&self) -> &CssSizeValue {
        &self.0.width
    }
    /// Borrows the explicitly authored height, when present.
    pub fn authored_height(&self) -> Option<&CssSizeValue> {
        self.0.authored_height.as_ref()
    }
    /// Borrows the effective height, repeating width if omitted.
    pub fn height(&self) -> &CssSizeValue {
        self.0.height()
    }
    /// Serializes the authored component count.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Serializes both components under one shared resource budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        self.0.serialize_specified_with_limits(limits)
    }
}

/// One- or two-value `max-size` shorthand, in width–height order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssMaxSizePair(Pair<CssMaxSizeValue>);

impl CssMaxSizePair {
    /// Preserves the first value and optional explicitly authored second.
    #[must_use]
    pub fn new(width: CssMaxSizeValue, authored_height: Option<CssMaxSizeValue>) -> Self {
        Self(Pair {
            width,
            authored_height,
        })
    }
    /// Borrows the first specified value.
    pub fn width(&self) -> &CssMaxSizeValue {
        &self.0.width
    }
    /// Borrows the explicitly authored height, when present.
    pub fn authored_height(&self) -> Option<&CssMaxSizeValue> {
        self.0.authored_height.as_ref()
    }
    /// Borrows the effective height, repeating width if omitted.
    pub fn height(&self) -> &CssMaxSizeValue {
        self.0.height()
    }
    /// Serializes the authored component count.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Serializes both components under one shared resource budget.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        self.0.serialize_specified_with_limits(limits)
    }
}

/// How a replaced element may expose its internal content size.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssFrameSizing {
    Auto,
    ContentWidth,
    ContentHeight,
    ContentBlockSize,
    ContentInlineSize,
}

impl CssFrameSizing {
    /// Serializes the canonical specified keyword.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Serializes under explicit limits.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        let text = match self {
            Self::Auto => "auto",
            Self::ContentWidth => "content-width",
            Self::ContentHeight => "content-height",
            Self::ContentBlockSize => "content-block-size",
            Self::ContentInlineSize => "content-inline-size",
        };
        serialize_keyword_sequence(text, limits)
    }
}

/// Authored min-content contribution compression choice.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMinIntrinsicSizing {
    Legacy,
    ZeroIfScroll,
    ZeroIfExtrinsic,
    ZeroIfScrollAndExtrinsic,
}

impl CssMinIntrinsicSizing {
    /// Serializes the canonical specified keyword sequence.
    pub fn serialize_specified(&self) -> SerializationResult<String> {
        self.serialize_specified_with_limits(CssSpecifiedValueSerializationLimits::default())
    }
    /// Serializes under explicit limits, with scroll before extrinsic.
    pub fn serialize_specified_with_limits(
        &self,
        limits: CssSpecifiedValueSerializationLimits,
    ) -> SerializationResult<String> {
        match self {
            Self::Legacy => serialize_keyword_sequence("legacy", limits),
            Self::ZeroIfScroll => serialize_keyword_sequence("zero-if-scroll", limits),
            Self::ZeroIfExtrinsic => serialize_keyword_sequence("zero-if-extrinsic", limits),
            Self::ZeroIfScrollAndExtrinsic => {
                let mut context = SpecifiedSerializationContext::new(limits);
                let mut output = String::new();
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(&mut output, "zero-if-scroll")?;
                context.append(&mut output, " ")?;
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(&mut output, "zero-if-extrinsic")?;
                Ok(output)
            }
        }
    }
}
