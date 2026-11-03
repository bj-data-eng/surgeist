use super::*;
use crate::output::{NamedBaselineDemand, NamedBaselineSet};
use crate::{
    BaselineFactOf, BaselineName, FlowAxes, GridBaselineError, GridBaselineReference,
    GridBaselineReferences, PhysicalSide,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum GridBaselineSelectionError {
    Unknown,
    Invalid(GridBaselineError),
    UnsupportedLane(GridAxisKind),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct GridBaselineSelection<S: LayoutScalar> {
    pub(super) reference: GridBaselineReference,
    pub(super) set: AncestorBaselineRole,
    pub(super) coordinate: S,
    pub(super) synthesized: bool,
}

fn baseline_set(alignment: AlignItems) -> Option<AncestorBaselineRole> {
    match alignment {
        AlignItems::Baseline => Some(AncestorBaselineRole::First),
        AlignItems::LastBaseline => Some(AncestorBaselineRole::Last),
        _ => None,
    }
}

pub(super) fn grid_baseline_reference(
    references: GridBaselineReferences,
    axis: GridAxisKind,
) -> GridBaselineReference {
    match axis {
        GridAxisKind::Column => references.columns,
        GridAxisKind::Row => references.rows,
    }
}

pub(super) fn effective_grid_baseline_references<S: LayoutScalar, Node>(
    local: GridBaselineReferences,
    parent: &GridParentContext<S, Node>,
) -> GridBaselineReferences {
    GridBaselineReferences {
        columns: parent
            .columns
            .as_ref()
            .map_or(local.columns, |axis| axis.baseline_reference),
        rows: parent
            .rows
            .as_ref()
            .map_or(local.rows, |axis| axis.baseline_reference),
    }
}

/// Grid chooses the actual set; Inline BaselineSource does not select it.
pub(super) fn grid_baseline_demand<S: LayoutScalar>(
    references: GridBaselineReferences,
    container: FlowAxes,
    child: FlowAxes,
    justify: AlignItems,
    align: AlignItems,
    input: ComputeInputOf<S>,
) -> ComputeInputOf<S> {
    let input = input.with_named_baseline_demand(NamedBaselineDemand::None);
    let (axis, alignment) = if child.block_axis() == container.inline_axis() {
        (GridAxisKind::Column, justify)
    } else {
        (GridAxisKind::Row, align)
    };
    match (
        grid_baseline_reference(references, axis),
        baseline_set(alignment),
    ) {
        (GridBaselineReference::Named(name), Some(set)) => {
            input.with_named_baseline_demand(NamedBaselineDemand::Fact {
                name,
                set: match set {
                    AncestorBaselineRole::First => NamedBaselineSet::First,
                    AncestorBaselineRole::Last => NamedBaselineSet::Last,
                },
            })
        }
        _ => input,
    }
}

pub(super) fn select_grid_baseline<S: LayoutScalar>(
    reference: GridBaselineReference,
    alignment: AlignItems,
    child: FlowAxes,
    physical_axis: PhysicalAxis,
    output: ComputeOutputOf<S>,
) -> Result<Option<GridBaselineSelection<S>>, GridBaselineSelectionError> {
    let Some(set) = baseline_set(alignment) else {
        return Ok(None);
    };
    if child.block_axis() != physical_axis {
        return Ok(None);
    }
    let (coordinate, synthesized) = match reference {
        GridBaselineReference::Physical => {
            let baselines = output.baselines();
            match set {
                AncestorBaselineRole::First => (
                    baselines.first_or_synthesize_block(child, output.size),
                    baselines.first_block(child).is_none(),
                ),
                AncestorBaselineRole::Last => (
                    baselines.last_or_synthesize_block(child, output.size),
                    baselines.last_block(child).is_none(),
                ),
            }
        }
        GridBaselineReference::Named(name) => {
            let pair = output.named_baselines.get(name);
            let fact = match set {
                AncestorBaselineRole::First => pair.first(),
                AncestorBaselineRole::Last => pair.last(),
            };
            match fact {
                BaselineFactOf::Unknown => return Err(GridBaselineSelectionError::Unknown),
                BaselineFactOf::Available(value) => {
                    if value.axis() != physical_axis {
                        return Err(GridBaselineSelectionError::Invalid(
                            GridBaselineError::AxisMismatch {
                                expected: physical_axis,
                                actual: value.axis(),
                            },
                        ));
                    }
                    (value.offset(), false)
                }
                BaselineFactOf::Absent => {
                    let extent = child.block_axis_extent(output.size);
                    let under = match child.line_under() {
                        PhysicalSide::Top | PhysicalSide::Left => S::ZERO,
                        PhysicalSide::Bottom | PhysicalSide::Right => extent,
                    };
                    let coordinate = match name {
                        BaselineName::Alphabetic => under,
                        BaselineName::Central => extent / S::from_f64(2.0),
                    };
                    if name == BaselineName::Central && extent > S::ZERO && coordinate == S::ZERO {
                        return Err(GridBaselineSelectionError::Invalid(
                            GridBaselineError::UnrepresentableCoordinate,
                        ));
                    }
                    (coordinate, true)
                }
            }
        }
    };
    if reference != GridBaselineReference::Physical && !coordinate.is_finite() {
        return Err(GridBaselineSelectionError::Invalid(
            GridBaselineError::UnrepresentableCoordinate,
        ));
    }
    Ok(Some(GridBaselineSelection {
        reference,
        set,
        coordinate,
        synthesized,
    }))
}

pub(super) fn grid_baseline_error_at_site<Node, S: LayoutScalar, M>(
    site: LayoutErrorSiteOf<Node>,
    error: GridBaselineSelectionError,
) -> LayoutErrorOf<Node, S, M> {
    LayoutErrorOf::new(
        site,
        LayoutOperation::ChildLayout,
        match error {
            GridBaselineSelectionError::Unknown => {
                LayoutErrorKindOf::MissingContext(crate::LayoutMissingContext::NamedGridBaseline)
            }
            GridBaselineSelectionError::Invalid(error) => {
                LayoutErrorKindOf::InvalidInput(crate::LayoutInvalidInputOf::GridBaseline(error))
            }
            GridBaselineSelectionError::UnsupportedLane(axis) => {
                LayoutErrorKindOf::UnsupportedCapability(
                    crate::LayoutUnsupportedCapability::NamedGridLanesBaseline { axis },
                )
            }
        },
    )
}

pub(super) fn grid_baseline_error<Node, S: LayoutScalar, M>(
    owner: Node,
    subject: Node,
    error: GridBaselineSelectionError,
) -> LayoutErrorOf<Node, S, M> {
    grid_baseline_error_at_site(
        LayoutErrorSiteOf::ContainerSubject {
            container: owner,
            subject,
        },
        error,
    )
}

/// Whether this actual span would make a synthesized port cyclic. The
/// selected source (Physical or named Absent) decides whether to apply it.
pub(super) fn grid_baseline_span_would_cycle<S: LayoutScalar>(
    tracks: &[TrackSizingOf<S>],
    basis: Option<S>,
) -> bool {
    tracks.len() > 1
        && tracks
            .iter()
            .any(|track| track_accepts_intrinsic_contribution(track, basis))
}

pub(super) fn grid_item_baseline_demand<S: LayoutScalar>(
    parent: &GridContainerProjection<'_, S>,
    child: &GridItemProjection<'_, S>,
    query_axis: Option<GridAxisKind>,
    input: ComputeInputOf<S>,
) -> Result<ComputeInputOf<S>, GridBaselineSelectionError> {
    let input = input.with_named_baseline_demand(NamedBaselineDemand::None);
    let flow = FlowAxes::new(child.writing_mode, child.direction);
    let container_flow = FlowAxes::new(parent.writing_mode, parent.direction);
    let axis = if flow.block_axis() == container_flow.inline_axis() {
        GridAxisKind::Column
    } else {
        GridAxisKind::Row
    };
    if query_axis.is_some_and(|queried| queried != axis) {
        return Ok(input);
    }
    if matches!(flow.line_over_edge(child.margin), LengthAutoOf::Auto)
        || matches!(flow.line_under_edge(child.margin), LengthAutoOf::Auto)
    {
        return Ok(input);
    }
    let justify = child
        .justify_self
        .or(parent.justify_items)
        .unwrap_or(AlignItems::Stretch);
    let align = child
        .align_self
        .or(parent.align_items)
        .unwrap_or(AlignItems::Stretch);
    let selected = match axis {
        GridAxisKind::Column => justify,
        GridAxisKind::Row => align,
    };
    if parent.display.establishes_grid_lanes_formatting_context()
        && axis != grid_axis_for_grid_lanes(parent)
        && baseline_set(selected).is_some()
        && matches!(
            grid_baseline_reference(parent.baseline_references, axis),
            GridBaselineReference::Named(_)
        )
    {
        return Err(GridBaselineSelectionError::UnsupportedLane(axis));
    }
    Ok(grid_baseline_demand(
        parent.baseline_references,
        container_flow,
        flow,
        justify,
        align,
        input,
    ))
}

pub(super) fn grid_baseline_participation<S: LayoutScalar>(
    reference: GridBaselineReference,
    alignment: AlignItems,
    auto_margins: bool,
    cyclic_span: bool,
    output: ComputeOutputOf<S>,
    child: FlowAxes,
    physical_axis: PhysicalAxis,
) -> Result<BaselineParticipation, GridBaselineSelectionError> {
    let Some(set) = baseline_set(alignment) else {
        return Ok(BaselineParticipation::default());
    };
    let compatible = child.block_axis() == physical_axis;
    // Physical metadata keeps its established fallback/presence semantics.
    // An ineligible named participant must not demand an unavailable fact.
    let selected = if reference == GridBaselineReference::Physical || (compatible && !auto_margins)
    {
        select_grid_baseline(reference, alignment, child, child.block_axis(), output)?
    } else {
        None
    };
    let synthesized = selected.is_some_and(|port| port.synthesized);
    let mut group = Some(match set {
        AncestorBaselineRole::First => BaselineGroupKind::Major,
        AncestorBaselineRole::Last => BaselineGroupKind::Minor,
    });
    if !compatible || (synthesized && cyclic_span) {
        group = None;
    }
    Ok(BaselineParticipation {
        participates: group.is_some() && !auto_margins,
        group,
        synthesized,
        fallback_alignment: Some(match set {
            AncestorBaselineRole::First => AlignItems::Start,
            AncestorBaselineRole::Last => AlignItems::End,
        }),
    })
}
