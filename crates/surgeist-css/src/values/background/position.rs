//! Checked authored coordinate families and consumer-specific position restrictions.
use crate::values::position::position_syntax::PositionAxes;
use crate::values::position::{CssHorizontalPosition, CssVerticalPosition};

/// One checked physical layer of background-position's one/two/three/four grammar.
///
/// # Specified serialization
///
/// Serializes the checked background position in horizontal-then-vertical order. In a three-
/// component position, the absent edge offset remains omitted. Percentages and calculations retain
/// their symbolic specified meaning.
///
/// # Specified serialization
///
/// In a three-component position, the absent edge offset remains omitted.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBackgroundPosition {
    axes: PositionAxes,
}
impl CssBackgroundPosition {
    /// Checks background arity directly, including a lone edge offset beside a keyword.
    #[must_use]
    pub fn try_new(
        horizontal: CssHorizontalPosition,
        vertical: CssVerticalPosition,
    ) -> Option<Self> {
        let axes = PositionAxes {
            horizontal,
            vertical,
        };
        if !axes.is_physical()
            || axes.horizontal.is_edge_offset()
                && matches!(axes.vertical, CssVerticalPosition::Offset(_))
            || axes.vertical.is_edge_offset()
                && matches!(axes.horizontal, CssHorizontalPosition::Offset(_))
        {
            return None;
        }
        Some(Self { axes })
    }
    /// Returns the authored horizontal axis and edge origin.
    #[must_use]
    pub const fn horizontal(&self) -> &CssHorizontalPosition {
        &self.axes.horizontal
    }
    /// Returns the authored vertical axis and edge origin.
    #[must_use]
    pub const fn vertical(&self) -> &CssVerticalPosition {
        &self.axes.vertical
    }
}

/// A nonempty authored comma list of `background-position` layers.
///
/// # Specified serialization
///
/// Serializes the nonempty authored layers in comma order.
#[derive(Clone, Debug, PartialEq)]
pub struct CssBackgroundPositionList {
    positions: Vec<CssBackgroundPosition>,
}

impl CssBackgroundPositionList {
    /// Constructs a nonempty list from already validated background-position layers.
    #[must_use]
    pub fn try_new(positions: Vec<CssBackgroundPosition>) -> Option<Self> {
        if positions.is_empty() {
            None
        } else {
            Some(Self::new(positions))
        }
    }

    #[must_use]
    pub(crate) const fn new(positions: Vec<CssBackgroundPosition>) -> Self {
        Self { positions }
    }

    /// Returns the authored layers in comma order.
    #[must_use]
    pub fn positions(&self) -> &[CssBackgroundPosition] {
        &self.positions
    }
}
