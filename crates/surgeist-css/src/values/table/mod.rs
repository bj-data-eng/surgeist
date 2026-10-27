//! Table property values.
//!
//! The table properties `border-collapse`, `caption-side`, `empty-cells` and `table-layout`.

pub(crate) mod border_spacing;
pub(crate) mod syntax;

#[doc(inline)]
pub use syntax::{CssBorderCollapse, CssCaptionSide, CssEmptyCells, CssTableLayout};
