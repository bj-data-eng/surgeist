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
    pub(crate) whole_insets: S,
    pub(crate) varying_border_box: bool,
}

pub(super) fn receipt<S: LayoutScalar>(
    style: &crate::NodeInputOf<S>,
    constants: &Constants<S>,
) -> Option<Receipt<S>> {
    let minimum = match constants.flow_axes.block_axis() {
        PhysicalAxis::Horizontal => &style.min_size.width,
        PhysicalAxis::Vertical => &style.min_size.height,
    };
    if !absolute(minimum) {
        return None;
    }
    let outer = constants.logical_node_min_size().block?;
    let insets = constants.logical_content_box_inset().block_sum();
    let padding = constants.flow_axes.logical_edges(style.padding);
    let border = constants.flow_axes.logical_edges(style.border);
    let dependent = [
        padding.block_start,
        padding.block_end,
        border.block_start,
        border.block_end,
    ]
    .into_iter()
    .any(crate::LengthOf::depends_on_basis);
    Some(Receipt {
        content: (outer - insets).max(S::ZERO),
        whole_insets: insets,
        varying_border_box: style.box_sizing == BoxSizing::BorderBox && dependent,
    })
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

pub(crate) fn absolute<S: LayoutScalar>(minimum: &crate::MinSizeOf<S>) -> bool {
    use crate::sizing::MinSizeView;
    match minimum.view() {
        MinSizeView::Zero => true,
        MinSizeView::Calculation(value) => !value.depends_on_basis(),
        _ => false,
    }
}
