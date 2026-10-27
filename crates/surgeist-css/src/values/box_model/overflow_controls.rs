//! Authored clipping, scrolling, and ellipsis controls from CSS Overflow 3.

use crate::component::CssComponentValue;
use crate::numeric::{CssLength, CssNonNegative, CssNumeric};
use crate::values::common::CssBoxEdgeKeyword;
use crate::values::text::CssTextOverflow;

/// The authored `<visual-box> || <length [0,∞]>` clip-margin value.
///
/// # Specified serialization
///
/// Serializes only the authored components in canonical box-first order.
#[derive(Clone, Debug)]
pub struct CssOverflowClipMargin {
    authored_box_edge: Option<CssBoxEdgeKeyword>,
    authored_offset: Option<CssNumeric<CssLength, CssNonNegative>>,
}

impl PartialEq for CssOverflowClipMargin {
    fn eq(&self, other: &Self) -> bool {
        self.authored_box_edge == other.authored_box_edge
            && match (&self.authored_offset, &other.authored_offset) {
                (None, None) => true,
                (Some(left), Some(right)) => left == right,
                _ => false,
            }
    }
}

impl Eq for CssOverflowClipMargin {}

impl CssOverflowClipMargin {
    /// Checks that at least one component is authored and the edge is visual.
    #[must_use]
    pub fn try_new(
        authored_box_edge: Option<CssBoxEdgeKeyword>,
        authored_offset: Option<CssNumeric<CssLength, CssNonNegative>>,
    ) -> Option<Self> {
        if authored_box_edge.is_none() && authored_offset.is_none() {
            return None;
        }
        if authored_box_edge.is_some_and(|edge| {
            !matches!(
                edge,
                CssBoxEdgeKeyword::ContentBox
                    | CssBoxEdgeKeyword::PaddingBox
                    | CssBoxEdgeKeyword::BorderBox
            )
        }) {
            return None;
        }
        Some(Self {
            authored_box_edge,
            authored_offset,
        })
    }

    /// Constructs the specified initial `0px`, with no authored box edge.
    #[must_use]
    pub fn initial() -> Self {
        Self::try_new(None, Some(Self::zero_px())).expect("valid clip-margin initial")
    }

    fn zero_px() -> CssNumeric<CssLength, CssNonNegative> {
        CssNumeric::<CssLength, CssNonNegative>::try_from_component(
            CssComponentValue::try_dimension("0", "px").expect("valid dimension"),
        )
        .expect("valid nonnegative length")
    }

    /// Borrows the authored visual box, if present.
    pub const fn authored_box_edge(&self) -> Option<CssBoxEdgeKeyword> {
        self.authored_box_edge
    }

    /// Borrows the authored offset, if present.
    pub fn authored_offset(&self) -> Option<&CssNumeric<CssLength, CssNonNegative>> {
        self.authored_offset.as_ref()
    }

    /// Returns the effective box edge, defaulting to `padding-box`.
    pub const fn box_edge(&self) -> CssBoxEdgeKeyword {
        match self.authored_box_edge {
            Some(edge) => edge,
            None => CssBoxEdgeKeyword::PaddingBox,
        }
    }

    /// Returns the effective offset, defaulting to exact `0px`.
    pub fn offset(&self) -> CssNumeric<CssLength, CssNonNegative> {
        self.authored_offset.clone().unwrap_or_else(Self::zero_px)
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider
    for CssOverflowClipMargin
{
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        context.charge_nodes(1)?;
        if let Some(edge) = self.authored_box_edge {
            context.charge_nodes(1)?;
            context.append(output, edge.as_str())?;
        }
        if let Some(offset) = &self.authored_offset {
            if self.authored_box_edge.is_some() {
                context.append(output, " ")?;
            }
            let captured = offset.capture_specified(context)?;
            context.append(output, &captured)?;
        }
        Ok(())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssOverflowClipMargin);

/// The authored scroll-behavior keyword, before scroll execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollBehavior {
    /// `auto`: the scroll container is scrolled in an instant fashion.
    Auto,
    /// `smooth`: the scroll container is scrolled in a smooth fashion, using a user-agent-defined timing function and duration; user agents may ignore this property.
    Smooth,
}

impl CssScrollBehavior {
    const fn as_css(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Smooth => "smooth",
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssScrollBehavior {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(self.as_css())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssScrollBehavior);

/// The valid authored scrollbar-gutter keyword combinations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollbarGutter {
    /// `auto`: classic scrollbars create a gutter only when `overflow` is `scroll`, or `auto` and the box overflows; overlay scrollbars consume no space.
    Auto,
    /// `stable`: the gutter is present for classic scrollbars when `overflow` is `hidden`, `scroll` or `auto`, whether or not the box overflows; overlay scrollbars consume no space.
    Stable,
    /// `stable both-edges`: like `stable`, and when a gutter would be present on one of the inline-start or inline-end edges, another is present on the opposite edge.
    StableBothEdges,
}

impl CssScrollbarGutter {
    const fn as_css(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Stable => "stable",
            Self::StableBothEdges => "stable both-edges",
        }
    }
}
impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssScrollbarGutter {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(self.as_css())
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssScrollbarGutter);

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssTextOverflow {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Clip => "clip",
            Self::Ellipsis => "ellipsis",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssTextOverflow);

/// Authored scroll-anchoring opt-out, applying to all elements with discrete animation.
/// Anchor selection and scroll adjustment belong to downstream style and layout owners.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOverflowAnchor {
    /// `auto`: the element is potentially eligible to participate in anchor node selection for any scrolling box created by it or an ancestor.
    Auto,
    /// `none`: the element and its descendants, apart from those nested inside another scrolling element, are not eligible to participate in anchor node selection.
    None,
}

impl crate::serialization::specified_rule_serialization::SpecifiedProvider for CssOverflowAnchor {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::serialization::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::serialization::CssSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::None => "none",
        })
    }
}
crate::serialization::specified_rule_serialization::specified_serialization!(CssOverflowAnchor);
