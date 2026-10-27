use crate::values::common::{CssBoxEdgeKeyword, CssUrl};
use crate::values::shape::CssBasicShape;

/// The supported authored subset of `clip-path`.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum CssClipPath {
    /// `none`: no clipping path gets created.
    None,
    /// A `<url>` referencing a `clipPath` element.
    Url(CssUrl),
    /// A bare `<geometry-box>` keyword, whose box edges, including any corner shaping, form
    /// the clipping path.
    GeometryBox(CssBoxEdgeKeyword),
    /// A `<basic-shape>` with an optional reference box.
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
    /// Returns the authored basic shape.
    #[must_use]
    pub const fn shape(&self) -> &CssBasicShape {
        &self.shape
    }
    /// Returns the explicit reference box, or `None` when it was omitted.
    #[must_use]
    pub const fn reference_box(&self) -> Option<CssBoxEdgeKeyword> {
        self.reference_box
    }
}
