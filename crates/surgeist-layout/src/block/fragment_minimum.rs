//! Block-owned conversion between whole-box minima and fragment content.
use super::*;

#[derive(Clone, Copy, Debug)]
pub(crate) struct Override<S: LayoutScalar> {
    pub(crate) axis: PhysicalAxis,
    pub(crate) content: S,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Receipt<S: LayoutScalar> {
    pub(crate) content: S,
}

pub(super) fn receipt<S: LayoutScalar>(
    style: &crate::NodeInputOf<S>,
    constants: &Constants<S>,
) -> Result<Option<Receipt<S>>, crate::sizing::resolve::SizingResolutionError<S>> {
    let minimum = match constants.flow_axes.block_axis() {
        PhysicalAxis::Horizontal => &style.min_size.width,
        PhysicalAxis::Vertical => &style.min_size.height,
    };
    if !absolute(minimum, constants.flow_axes.block_axis())? {
        return Ok(None);
    }
    let Some(outer) = constants.logical_node_min_size().block else {
        return Ok(None);
    };
    let insets = constants.logical_content_box_inset().block_sum();
    Ok(Some(Receipt {
        content: (outer - insets).max(S::ZERO),
    }))
}

pub(super) fn apply<S: LayoutScalar>(
    replacement: Option<Override<S>>,
    mut minimum: Size<Option<S>>,
    insets: Size<S>,
) -> Size<Option<S>> {
    if let Some(replacement) = replacement {
        match replacement.axis {
            PhysicalAxis::Horizontal => minimum.width = Some(replacement.content + insets.width),
            PhysicalAxis::Vertical => minimum.height = Some(replacement.content + insets.height),
        }
    }
    minimum
}

pub(crate) fn absolute<S: LayoutScalar>(
    minimum: &crate::MinSizeOf<S>,
    axis: PhysicalAxis,
) -> Result<bool, crate::sizing::resolve::SizingResolutionError<S>> {
    use crate::sizing::resolve::{FixedFallbackSource, fixed_fallback_minimum};
    // Qualification belongs to sizing dispatch; Constants retains ownership of
    // the already-resolved scalar and its box-sizing conversion.
    fixed_fallback_minimum(minimum, SizingAlgorithm::Block, axis, None)
        .map(|source| matches!(source, FixedFallbackSource::Fixed(_)))
}

/// Raw inset dependence is an admission fact, independent of the widths a
/// particular sequence happens to use or the frame where the box first fits.
pub(crate) fn basis_dependent_border_box<S: LayoutScalar>(
    style: &crate::NodeInputOf<S>,
    axes: crate::FlowAxes,
) -> bool {
    if style.box_sizing != BoxSizing::BorderBox {
        return false;
    }
    let padding = axes.logical_edges(style.padding);
    let border = axes.logical_edges(style.border);
    [
        padding.block_start,
        padding.block_end,
        border.block_start,
        border.block_end,
    ]
    .into_iter()
    .any(crate::LengthOf::depends_on_basis)
}
