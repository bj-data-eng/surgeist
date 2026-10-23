//! Authored clipping, scrolling, and ellipsis controls from CSS Overflow 3.

use crate::{CssBoxEdgeKeyword, CssComponentValue, CssSpecifiedNonNegativeLength, CssTextOverflow};

/// The authored `<visual-box> || <length [0,∞]>` clip-margin value.
///
/// # Specified serialization
///
/// Serializes only the authored components in canonical box-first order.
#[derive(Clone, Debug)]
pub struct CssOverflowClipMargin {
    authored_box_edge: Option<CssBoxEdgeKeyword>,
    authored_offset: Option<CssSpecifiedNonNegativeLength>,
}

impl PartialEq for CssOverflowClipMargin {
    fn eq(&self, other: &Self) -> bool {
        self.authored_box_edge == other.authored_box_edge
            && match (&self.authored_offset, &other.authored_offset) {
                (None, None) => true,
                (Some(left), Some(right)) => left.structural_eq(right),
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
        authored_offset: Option<CssSpecifiedNonNegativeLength>,
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

    fn zero_px() -> CssSpecifiedNonNegativeLength {
        CssSpecifiedNonNegativeLength::try_from_component(
            CssComponentValue::try_dimension("0", "px").expect("valid dimension"),
        )
        .expect("valid nonnegative length")
    }

    /// Borrows the authored visual box, if present.
    pub const fn authored_box_edge(&self) -> Option<CssBoxEdgeKeyword> {
        self.authored_box_edge
    }

    /// Borrows the authored offset, if present.
    pub fn authored_offset(&self) -> Option<&CssSpecifiedNonNegativeLength> {
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
    pub fn offset(&self) -> CssSpecifiedNonNegativeLength {
        self.authored_offset.clone().unwrap_or_else(Self::zero_px)
    }
}
impl crate::specified_rule_serialization::SpecifiedProvider for CssOverflowClipMargin {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        let context = &mut writer.context;
        let output = &mut writer.css;
        context.charge_nodes(1)?;
        if let Some(edge) = self.authored_box_edge {
            context.charge_nodes(1)?;
            context.append(output, edge.as_css_str())?;
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
crate::specified_rule_serialization::specified_serialization!(CssOverflowClipMargin);

/// The authored scroll-behavior keyword, before scroll execution.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollBehavior {
    Auto,
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
impl crate::specified_rule_serialization::SpecifiedProvider for CssScrollBehavior {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.keyword(self.as_css())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssScrollBehavior);

/// The valid authored scrollbar-gutter keyword combinations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssScrollbarGutter {
    Auto,
    Stable,
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
impl crate::specified_rule_serialization::SpecifiedProvider for CssScrollbarGutter {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.keyword(self.as_css())
    }
}
crate::specified_rule_serialization::specified_serialization!(CssScrollbarGutter);

impl crate::specified_rule_serialization::SpecifiedProvider for CssTextOverflow {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.keyword(match self {
            Self::Clip => "clip",
            Self::Ellipsis => "ellipsis",
        })
    }
}
crate::specified_rule_serialization::specified_serialization!(CssTextOverflow);

/// Authored scroll-anchoring opt-out, applying to all elements with discrete animation.
/// Anchor selection and scroll adjustment belong to downstream style and layout owners.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum CssOverflowAnchor {
    Auto,
    None,
}

impl crate::specified_rule_serialization::SpecifiedProvider for CssOverflowAnchor {
    fn append_to_rule_writer(
        &self,
        writer: &mut crate::specified_rule_serialization::SpecifiedRuleWriter,
    ) -> std::result::Result<(), crate::CssSerializationError> {
        writer.keyword(match self {
            Self::Auto => "auto",
            Self::None => "none",
        })
    }
}
crate::specified_rule_serialization::specified_serialization!(CssOverflowAnchor);
