//! Layout primitives and CSS used-flow resolution.

pub use surgeist_layout::*;

use surgeist_css::values::layout::{
    CssGridAutoFlow, CssGridAutoFlowAxis, CssGridTrackList, CssItemDirection, CssItemPack,
};

/// A CSS flow value whose used interpretation is unavailable.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum GridFlowResolutionError {
    UnsupportedItemDirection,
    UnsupportedItemPack,
    UnsupportedAutoFlow,
    UnsupportedAutoFlowAxis,
}

impl std::fmt::Display for GridFlowResolutionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::UnsupportedItemDirection => "unsupported item-direction",
            Self::UnsupportedItemPack => "unsupported item-pack",
            Self::UnsupportedAutoFlow => "unsupported grid-auto-flow",
            Self::UnsupportedAutoFlowAxis => "unsupported grid-auto-flow axis",
        })
    }
}

impl std::error::Error for GridFlowResolutionError {}

/// Resolves effective CSS auto-flow and packing for ordinary Grid.
///
/// Normal and bare dense use row auto-placement. Explicit axes are retained.
/// Either control can request dense packing; balance has no additional effect.
/// Callers supply values after resolving globals, substitutions, and shorthand
/// expansion, following the
/// [Grid packing policy](https://github.com/bj-data-eng/surgeist/issues/1000#issuecomment-6704697720).
pub fn resolve_grid_auto_flow(
    auto_flow: CssGridAutoFlow,
    item_pack: CssItemPack,
) -> Result<GridAutoFlow, GridFlowResolutionError> {
    let (column, auto_dense) = match auto_flow {
        CssGridAutoFlow::Normal | CssGridAutoFlow::Dense => {
            (false, auto_flow == CssGridAutoFlow::Dense)
        }
        CssGridAutoFlow::ExplicitAxis(mode) => (
            match mode.axis() {
                CssGridAutoFlowAxis::Row => false,
                CssGridAutoFlowAxis::Column => true,
                _ => return Err(GridFlowResolutionError::UnsupportedAutoFlowAxis),
            },
            mode.dense(),
        ),
        _ => return Err(GridFlowResolutionError::UnsupportedAutoFlow),
    };
    Ok(
        match (column, resolve_grid_density(auto_dense, item_pack)?) {
            (false, false) => GridAutoFlow::Row,
            (false, true) => GridAutoFlow::RowDense,
            (true, false) => GridAutoFlow::Column,
            (true, true) => GridAutoFlow::ColumnDense,
        },
    )
}

/// Resolves independently cascaded CSS controls to one Grid Lanes axis pair.
///
/// A non-auto item direction selects the grid axis and primary placement sign.
/// Otherwise an explicit auto-flow axis selects orientation. Normal and bare
/// dense select rows only when columns are none and rows are present. Packing
/// density is requested by either auto-flow or item-pack. The balance facet
/// has no additional used effect.
///
/// Callers supply effective property values after resolving globals and
/// substitutions. A direction supplied by item-flow enters as item-direction.
/// Orientation follows the policy recorded in issue
/// [#993](https://github.com/bj-data-eng/surgeist/issues/993). Density follows the
/// [Grid packing policy](https://github.com/bj-data-eng/surgeist/issues/1000#issuecomment-6704697720).
pub fn resolve_grid_lanes_flow(
    item_direction: CssItemDirection,
    auto_flow: CssGridAutoFlow,
    item_pack: CssItemPack,
    columns: &CssGridTrackList,
    rows: &CssGridTrackList,
) -> Result<GridLanesFlow, GridFlowResolutionError> {
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
                _ => return Err(GridFlowResolutionError::UnsupportedAutoFlowAxis),
            },
            mode.dense(),
        ),
        _ => return Err(GridFlowResolutionError::UnsupportedAutoFlow),
    };
    let (grid_axis, reversed) = match item_direction {
        CssItemDirection::Auto => (auto_axis, false),
        CssItemDirection::Row => (GridAxisKind::Column, false),
        CssItemDirection::RowReverse => (GridAxisKind::Column, true),
        CssItemDirection::Column => (GridAxisKind::Row, false),
        CssItemDirection::ColumnReverse => (GridAxisKind::Row, true),
        _ => return Err(GridFlowResolutionError::UnsupportedItemDirection),
    };
    Ok(GridLanesFlow::new(
        grid_axis,
        reversed,
        resolve_grid_density(dense, item_pack)?,
    ))
}

fn resolve_grid_density(
    auto_dense: bool,
    item_pack: CssItemPack,
) -> Result<bool, GridFlowResolutionError> {
    let pack_dense = match item_pack {
        CssItemPack::Normal | CssItemPack::Balance => false,
        CssItemPack::Dense | CssItemPack::DenseBalance => true,
        _ => return Err(GridFlowResolutionError::UnsupportedItemPack),
    };
    Ok(auto_dense || pack_dense)
}
