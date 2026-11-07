//! Layout primitives and CSS used-flow resolution.

pub use surgeist_layout::*;

use surgeist_css::values::layout::{
    CssGridAutoFlow, CssGridAutoFlowAxis, CssGridTrackList, CssItemDirection,
};

/// A CSS flow value whose used interpretation is unavailable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GridLanesFlowResolutionError {
    UnsupportedItemDirection,
    UnsupportedAutoFlow,
    UnsupportedAutoFlowAxis,
}

impl std::fmt::Display for GridLanesFlowResolutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedItemDirection => "unsupported item-direction",
            Self::UnsupportedAutoFlow => "unsupported grid-auto-flow",
            Self::UnsupportedAutoFlowAxis => "unsupported grid-auto-flow axis",
        })
    }
}

impl std::error::Error for GridLanesFlowResolutionError {}

/// Resolves independently cascaded CSS controls to one Grid Lanes axis pair.
///
/// A non-auto item direction selects the grid axis and primary placement sign.
/// Otherwise an explicit auto-flow axis selects orientation. Normal and bare
/// dense select rows only when columns are none and rows are present. Packing
/// density comes from auto-flow independently of that orientation choice.
///
/// Callers supply effective property values after resolving globals and
/// substitutions. A direction supplied by item-flow enters as item-direction.
/// Cross-property precedence follows the Surgeist policy recorded in issue
/// [#993](https://github.com/bj-data-eng/surgeist/issues/993).
pub fn resolve_grid_lanes_flow(
    item_direction: CssItemDirection,
    auto_flow: CssGridAutoFlow,
    columns: &CssGridTrackList,
    rows: &CssGridTrackList,
) -> Result<GridLanesFlow, GridLanesFlowResolutionError> {
    let (auto_axis, dense) = match auto_flow {
        CssGridAutoFlow::Normal | CssGridAutoFlow::Dense => (
            if columns.is_none() && !rows.is_none() {
                GridAxisKind::Row
            } else {
                GridAxisKind::Column
            },
            auto_flow == CssGridAutoFlow::Dense,
        ),
        CssGridAutoFlow::ExplicitAxis(mode) => (
            match mode.axis() {
                CssGridAutoFlowAxis::Row => GridAxisKind::Column,
                CssGridAutoFlowAxis::Column => GridAxisKind::Row,
                _ => return Err(GridLanesFlowResolutionError::UnsupportedAutoFlowAxis),
            },
            mode.dense(),
        ),
        _ => return Err(GridLanesFlowResolutionError::UnsupportedAutoFlow),
    };
    let (grid_axis, reversed) = match item_direction {
        CssItemDirection::Auto => (auto_axis, false),
        CssItemDirection::Row => (GridAxisKind::Column, false),
        CssItemDirection::RowReverse => (GridAxisKind::Column, true),
        CssItemDirection::Column => (GridAxisKind::Row, false),
        CssItemDirection::ColumnReverse => (GridAxisKind::Row, true),
        _ => return Err(GridLanesFlowResolutionError::UnsupportedItemDirection),
    };
    Ok(GridLanesFlow::new(grid_axis, reversed, dense))
}
