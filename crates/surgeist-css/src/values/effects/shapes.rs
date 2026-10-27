use crate::values::common::{CssBoxEdgeKeyword, CssUrl};
use crate::values::shape::CssBasicShape;

/// The supported authored subset of `clip-path`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssClipPath {
    None,
    Url(CssUrl),
    GeometryBox(CssBoxEdgeKeyword),
    BasicShape(CssClipPathShape),
}

/// An authored shape with an optional explicit clipping reference box.
/// Box omission stays distinct from explicit border-box and requires downstream context.
#[derive(Clone, Debug, PartialEq)]
pub struct CssClipPathShape {
    shape: CssBasicShape,
    reference_box: Option<CssBoxEdgeKeyword>,
}

impl CssClipPathShape {
    /// Composes checked inputs without inferring a default reference box.
    #[must_use]
    pub const fn new(shape: CssBasicShape, reference_box: Option<CssBoxEdgeKeyword>) -> Self {
        Self {
            shape,
            reference_box,
        }
    }
    #[must_use]
    pub const fn shape(&self) -> &CssBasicShape {
        &self.shape
    }
    #[must_use]
    pub const fn reference_box(&self) -> Option<CssBoxEdgeKeyword> {
        self.reference_box
    }
}
