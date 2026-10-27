//! Sizing 4 authored shorthands and finite intrinsic-size controls.

use crate::specified_inverse::InverseValueEquality as _;
use crate::values::box_model::{CssMaxSizeValue, CssSizeValue};
use crate::values::common::common_values::{side_element, side_pair_entry};

// `size` and `min-size` take `<'width'>{1,2}`; `max-size` takes `<'max-width'>{1,2}`.
// Both omit a second component that equals the first.
side_element!(
    CssSizeValue,
    same = |a, b, context| a.inverse_equal(b, context)
);
side_element!(
    CssMaxSizeValue,
    same = |a, b, context| a.inverse_equal(b, context)
);
side_pair_entry!(CssSizeValue, CssMaxSizeValue);

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

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssFrameSizing {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
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
crate::serialization::specified_rule_serialization::specified_serialization!(CssFrameSizing);

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

impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssMinIntrinsicSizing
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
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
crate::serialization::specified_rule_serialization::specified_serialization!(CssMinIntrinsicSizing);
