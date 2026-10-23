//! Sizing 4 authored shorthands and finite intrinsic-size controls.

use crate::specified_serialization::SpecifiedSerializationContext;
use crate::{CssMaxSizeValue, CssSerializationError, CssSizeValue};

type SerializationResult<T> = Result<T, CssSerializationError>;

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

impl<T: PairValue> crate::specified_rule_serialization::SpecifiedProvider for Pair<T> {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> SerializationResult<()> {
        writer.source_member(0, |writer| {
            self.width
                .serialize_into(&mut writer.context, &mut writer.css)
        })?;
        if let Some(height) = &self.authored_height {
            writer.source_member(1, |writer| {
                writer.append(" ")?;
                height.serialize_into(&mut writer.context, &mut writer.css)
            })?;
        }
        Ok(())
    }
}

/// One- or two-value `size` or `min-size` shorthand, in width–height order.
///
/// # Specified serialization
///
/// Serializes the authored component count.
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
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssSizePair {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.0.append_to_rule_writer(writer)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssSizePair);

/// One- or two-value `max-size` shorthand, in width–height order.
///
/// # Specified serialization
///
/// Serializes the authored component count.
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
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssMaxSizePair {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        self.0.append_to_rule_writer(writer)
    }
}
crate::specified_rule_serialization::specified_serialization!(CssMaxSizePair);

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

impl crate::specified_rule_serialization::SpecifiedProvider for CssFrameSizing {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
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
crate::specified_rule_serialization::specified_serialization!(CssFrameSizing);

/// Authored min-content contribution compression choice.
///
/// # Specified serialization
///
/// Serializes the canonical specified keyword sequence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssMinIntrinsicSizing {
    Legacy,
    ZeroIfScroll,
    ZeroIfExtrinsic,
    ZeroIfScrollAndExtrinsic,
}

impl crate::specified_rule_serialization::SpecifiedProvider for CssMinIntrinsicSizing {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        match self {
            Self::Legacy => writer.keyword("legacy"),
            Self::ZeroIfScroll => writer.keyword("zero-if-scroll"),
            Self::ZeroIfExtrinsic => writer.keyword("zero-if-extrinsic"),
            Self::ZeroIfScrollAndExtrinsic => {
                let context = &mut writer.context;
                let output = &mut writer.css;
                context.charge_nodes(1)?;
                context.append(output, "zero-if-scroll")?;
                context.append(output, " ")?;
                context.charge_nodes(1)?;
                context.append(output, "zero-if-extrinsic")?;
                Ok(())
            }
        }
    }
}
crate::specified_rule_serialization::specified_serialization!(CssMinIntrinsicSizing);
