//! Cartesian access to a `CssPosition`.
//!
//! `object-position`, `perspective-origin` and `mask-position` hold the full Values 5 position.
//! Tests that check a physical or x/y keyword position read its two axes through this view and
//! fail if the position is not a Cartesian one.

use surgeist_css::values::position::{
    CssHorizontalPosition, CssPosition, CssPositionRef, CssVerticalPosition,
};

/// The horizontal and vertical axes of a Cartesian position.
pub trait CartesianPosition {
    /// The horizontal axis; panics for a flow-relative position.
    fn horizontal(&self) -> &CssHorizontalPosition;
    /// The vertical axis; panics for a flow-relative position.
    fn vertical(&self) -> &CssVerticalPosition;
}

impl CartesianPosition for CssPosition {
    #[track_caller]
    fn horizontal(&self) -> &CssHorizontalPosition {
        match self.view() {
            CssPositionRef::Cartesian(position) => position.horizontal(),
            other => panic!("a Cartesian position, found {other:?}"),
        }
    }

    #[track_caller]
    fn vertical(&self) -> &CssVerticalPosition {
        match self.view() {
            CssPositionRef::Cartesian(position) => position.vertical(),
            other => panic!("a Cartesian position, found {other:?}"),
        }
    }
}
