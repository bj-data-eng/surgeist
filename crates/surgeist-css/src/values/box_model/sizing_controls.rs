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
    /// `auto`: the element's internal layout intrinsic size is ignored.
    Auto,
    /// `content-width`: the intrinsic width is taken from the element's internal layout intrinsic size, if it has one.
    ContentWidth,
    /// `content-height`: the intrinsic height is taken from the element's internal layout intrinsic size, if it has one.
    ContentHeight,
    /// `content-block-size`: the intrinsic block size is taken from the internal layout intrinsic size, resolved in the element's writing mode.
    ContentBlockSize,
    /// `content-inline-size`: the intrinsic inline size is taken from the internal layout intrinsic size, resolved in the element's writing mode.
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
    /// `legacy`: the min-content contribution is handled normally.
    Legacy,
    /// `zero-if-scroll`: the min-content contribution is compressed when the box is a scroll container in that axis.
    ZeroIfScroll,
    /// `zero-if-extrinsic`: the min-content contribution is compressed when the box has an extrinsic preferred or maximum size.
    ZeroIfExtrinsic,
    /// `zero-if-scroll zero-if-extrinsic`: the min-content contribution is compressed under either condition.
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
