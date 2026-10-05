//! Sizing 4 authored shorthands and finite intrinsic-size controls.

use crate::specified_serialization::SpecifiedSerializationContext;
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
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        self.width.serialize_into(context, output)?;
        if let Some(height) = &self.authored_height {
            context.append(output, " ")?;
            height.serialize_into(context, output)?;
        }
        Ok(())
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
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        self.0.append_to_rule_writer(writer)
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
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        self.0.append_to_rule_writer(writer)
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
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        let text = match self {
            Self::Auto => "auto",
            Self::ContentWidth => "content-width",
            Self::ContentHeight => "content-height",
            Self::ContentBlockSize => "content-block-size",
            Self::ContentInlineSize => "content-inline-size",
        };
        writer.keyword(text)
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
        let mut writer = crate::specified_rule_serialization::SpecifiedRuleWriter::new(limits);
        self.append_to_rule_writer(&mut writer)?;
        Ok(writer.css)
    }

    pub(crate) fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSpecifiedValueSerializationError> {
        match self {
            Self::Legacy => writer.keyword("legacy"),
            Self::ZeroIfScroll => writer.keyword("zero-if-scroll"),
            Self::ZeroIfExtrinsic => writer.keyword("zero-if-extrinsic"),
            Self::ZeroIfScrollAndExtrinsic => {
                let context = &mut writer.context;
                let output = &mut writer.css;
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(output, "zero-if-scroll")?;
                context.append(output, " ")?;
                context.charge_input(1)?;
                context.charge_projection(1)?;
                context.append(output, "zero-if-extrinsic")?;
                Ok(())
            }
        }
    }
}
