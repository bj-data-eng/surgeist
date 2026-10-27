/// A representable decoded Values 4 custom identifier, preserving authored case.
/// CSS-wide keywords and `default` are excluded; consuming grammars own any
/// additional keyword exclusions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssCustomIdent(CssIdent);

impl CssCustomIdent {
    /// Returns a custom identifier, or `None` when `value` is not a valid identifier or is a CSS-wide keyword or `default` (ASCII case-insensitive).
    #[must_use]
    pub fn try_new(value: impl Into<String>) -> Option<Self> {
        Self::try_from_ident(CssIdent::try_new(value).ok()?)
    }

    pub(crate) fn try_from_ident(value: CssIdent) -> Option<Self> {
        (!is_reserved_custom_ident(value.as_str())).then_some(Self(value))
    }

    /// Returns the decoded identifier text, preserving authored case.
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    pub(crate) fn retained_buffer_bytes(&self) -> usize {
        self.0.value.capacity()
    }
}

pub(crate) fn is_reserved_custom_ident(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "inherit" | "initial" | "unset" | "revert" | "revert-layer" | "default"
    )
}

/// An authored CSS Box Model Level 3 `<box>` edge keyword.
///
/// The value names a box edge only. It does not select a layout, paint, or
/// coordinate-space policy until a consuming property is interpreted by its
/// downstream owner.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum CssBoxEdgeKeyword {
    /// `content-box`: the content box edge.
    ContentBox,
    /// `padding-box`: the padding box edge.
    PaddingBox,
    /// `border-box`: the border box edge.
    BorderBox,
    /// `margin-box`: the margin box edge.
    MarginBox,
    /// `fill-box`: the object bounding box edge.
    FillBox,
    /// `stroke-box`: the stroke bounding box edge.
    StrokeBox,
    /// `view-box`: the nearest SVG viewport box edge.
    ViewBox,
}

impl CssBoxEdgeKeyword {
    /// Converts one decoded CSS keyword to its typed edge identity.
    #[must_use]
    pub fn from_keyword(keyword: &str) -> Option<Self> {
        Some(match keyword.to_ascii_lowercase().as_str() {
            "content-box" => Self::ContentBox,
            "padding-box" => Self::PaddingBox,
            "border-box" => Self::BorderBox,
            "margin-box" => Self::MarginBox,
            "fill-box" => Self::FillBox,
            "stroke-box" => Self::StrokeBox,
            "view-box" => Self::ViewBox,
            _ => return None,
        })
    }

    /// Returns the canonical CSS keyword spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ContentBox => "content-box",
            Self::PaddingBox => "padding-box",
            Self::BorderBox => "border-box",
            Self::MarginBox => "margin-box",
            Self::FillBox => "fill-box",
            Self::StrokeBox => "stroke-box",
            Self::ViewBox => "view-box",
        }
    }
}

/// One decoded CSS identifier retained as authored value syntax.
///
/// Parsed and checked construction do not apply consumer-specific keyword exclusions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CssIdent {
    value: String,
}

impl CssIdent {
    /// The original lexical/property owner has checked the decoded name and
    /// admitted this exact String allocation.
    pub(crate) fn from_admitted(value: String) -> Self {
        Self { value }
    }

    pub(crate) fn backing_capacity(&self) -> usize {
        self.value.capacity()
    }
    /// Constructs a representable decoded CSS identifier, retaining its case and punctuation.
    /// Consumer-specific keyword exclusions are intentionally not applied.
    pub fn try_new(
        value: impl Into<String>,
    ) -> Result<Self, crate::component::CssComponentValueError> {
        let value = value.into();
        crate::component::CssComponentValue::try_ident(value.clone())?;
        Ok(Self { value })
    }

    #[must_use]
    pub(crate) fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    /// Returns the decoded, case-preserving identifier value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.value
    }
}
