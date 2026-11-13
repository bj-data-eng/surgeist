use super::*;
use crate::sizing::resolve::{ResolvedPreferredSize, resolve_preferred_sizing};
use crate::{ComputeInputOf, LengthResolutionStatus, SizingMode};

/// Borrowed original sizing functions, before inherited geometry is lowered to
/// fixed pixel tracks. Mapping changes only this view, never the source table.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct GridMinimumTrackFunctions<'a, S: LayoutScalar> {
    source: GridMinimumFunctionSource<'a, S>,
    reversed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum GridMinimumFunctionSource<'a, S: LayoutScalar> {
    Original {
        tracks: &'a [TrackSizingOf<S>],
        basis: Option<S>,
    },
    Qualified(&'a [GridMinimumTrackFact<S>]),
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct GridMinimumTrackFact<S: LayoutScalar> {
    automatic: bool,
    flexible: bool,
    fixed: Option<S>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) enum GridMinimumFunctionRetentionError<S: LayoutScalar> {
    Allocation,
    Sizing(SizingResolutionError<S>),
}

impl<'a, S: LayoutScalar> GridMinimumTrackFunctions<'a, S> {
    pub(in crate::grid) const fn new(tracks: &'a [TrackSizingOf<S>], basis: Option<S>) -> Self {
        Self {
            source: GridMinimumFunctionSource::Original { tracks, basis },
            reversed: false,
        }
    }

    pub(in crate::grid) const fn from_qualified(facts: &'a [GridMinimumTrackFact<S>]) -> Self {
        Self {
            source: GridMinimumFunctionSource::Qualified(facts),
            reversed: false,
        }
    }

    /// Requalify authored functions in this lender's actual phase. Retained
    /// inherited facts keep the original lender's already qualified basis.
    pub(in crate::grid) fn requalified(self, basis: Option<S>) -> Self {
        let source = match self.source {
            GridMinimumFunctionSource::Original { tracks, .. } => {
                GridMinimumFunctionSource::Original { tracks, basis }
            }
            GridMinimumFunctionSource::Qualified(facts) => {
                GridMinimumFunctionSource::Qualified(facts)
            }
        };
        Self { source, ..self }
    }

    fn len(self) -> usize {
        match self.source {
            GridMinimumFunctionSource::Original { tracks, .. } => tracks.len(),
            GridMinimumFunctionSource::Qualified(facts) => facts.len(),
        }
    }

    fn fact(self, offset: usize) -> Result<GridMinimumTrackFact<S>, SizingResolutionError<S>> {
        let index = if self.reversed {
            self.len() - offset - 1
        } else {
            offset
        };
        match self.source {
            GridMinimumFunctionSource::Qualified(facts) => Ok(facts[index]),
            GridMinimumFunctionSource::Original { tracks, basis } => {
                let track = &tracks[index];
                let automatic = match &track.min {
                    MinTrackSizingOf::Auto => true,
                    MinTrackSizingOf::Calculation(calculation) => {
                        match resolve_track_calculation(calculation, basis).status() {
                            LengthResolutionStatus::Resolved
                            | LengthResolutionStatus::MissingBasis => {
                                crate::grid::tracks::track_calculation_is_cyclic_auto(
                                    calculation,
                                    basis,
                                )
                            }
                            status => return Err(status.into()),
                        }
                    }
                    MinTrackSizingOf::MinContent | MinTrackSizingOf::MaxContent => false,
                };
                let fixed = effective_fixed_track_maximum(track, basis)
                    .map_err(SizingResolutionError::from)?;
                Ok(GridMinimumTrackFact {
                    automatic,
                    flexible: track.max.is_flexible(),
                    fixed,
                })
            }
        }
    }

    /// One actual owned inherited-axis proof backing. Subsequent existing
    /// context Clone operations retain their ordinary Native allocating policy.
    pub(in crate::grid) fn retain(
        self,
    ) -> Result<Vec<GridMinimumTrackFact<S>>, GridMinimumFunctionRetentionError<S>> {
        let mut facts = Vec::new();
        facts
            .try_reserve_exact(self.len())
            .map_err(|_| GridMinimumFunctionRetentionError::Allocation)?;
        for offset in 0..self.len() {
            facts.push(
                self.fact(offset)
                    .map_err(GridMinimumFunctionRetentionError::Sizing)?,
            );
        }
        Ok(facts)
    }

    pub(in crate::grid) fn mapped_span(
        self,
        start: usize,
        end: usize,
        reversed: bool,
    ) -> Option<Self> {
        if start >= end || end > self.len() {
            return None;
        }
        let (start, end) = if self.reversed {
            (self.len() - end, self.len() - start)
        } else {
            (start, end)
        };
        let source = match self.source {
            GridMinimumFunctionSource::Original { tracks, basis } => {
                GridMinimumFunctionSource::Original {
                    tracks: tracks.get(start..end)?,
                    basis,
                }
            }
            GridMinimumFunctionSource::Qualified(facts) => {
                GridMinimumFunctionSource::Qualified(facts.get(start..end)?)
            }
        };
        Some(Self {
            source,
            reversed: self.reversed != reversed,
        })
    }

    pub(in crate::grid) fn span_facts(
        self,
        active_gutter: S,
    ) -> Result<GridMinimumSpanFacts<S>, SizingResolutionError<S>> {
        let mut automatic = false;
        let mut flexible = false;
        let mut fixed = Some(finite(active_gutter)?);
        for offset in 0..self.len() {
            let fact = self.fact(offset)?;
            automatic |= fact.automatic;
            flexible |= fact.flexible;
            fixed = match (fixed, fact.fixed) {
                (Some(span), Some(maximum)) => Some(finite(span + maximum)?),
                _ => None,
            };
        }
        Ok(GridMinimumSpanFacts {
            content_minimum_eligible: automatic && (self.len() == 1 || !flexible),
            fixed_max_span: if self.len() == 0 { None } else { fixed },
        })
    }
}

/// A successfully qualified span. An absent fixed cap is distinct from absent
/// span authority; only constructors reading original functions create this.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(in crate::grid) struct GridMinimumSpanFacts<S: LayoutScalar> {
    content_minimum_eligible: bool,
    fixed_max_span: Option<S>,
}

impl<S: LayoutScalar> GridMinimumSpanFacts<S> {
    pub(in crate::grid) const fn fixed_max_span(self) -> Option<S> {
        self.fixed_max_span
    }
}

/// Actual per-item facts after the opposite-axis preparation. `flow_axes` maps
/// the Grid axis; edge percentages use the prepared containing flow instead.
pub(in crate::grid) struct GridMinimumContext<'a, 'input, S: LayoutScalar> {
    pub(in crate::grid) item: &'a GridItemProjection<'input, S>,
    pub(in crate::grid) flow_axes: FlowAxes,
    pub(in crate::grid) axis: GridAxisKind,
    pub(in crate::grid) prepared: ComputeInputOf<S>,
    /// Containing extent used only for the margins in the fixed span caps.
    pub(in crate::grid) margin_basis: Size<Option<S>>,
    pub(in crate::grid) span: GridMinimumSpanFacts<S>,
    /// The opposite span's fixed maxima plus active gutters, if all are fixed.
    /// None means no applicable fixed cap, not an unknown prepared extent.
    pub(in crate::grid) opposite_fixed_max_span: Option<S>,
    /// Actual requested inherited axes in the child's own logical frame.
    pub(in crate::grid) subgrid_requests: Size<bool>,
    pub(in crate::grid) algorithm: SizingAlgorithm,
}

pub(in crate::grid) enum GridMinimumPlan<S: LayoutScalar> {
    /// Grid's nondependent, non-Auto preferred-size branch uses the
    /// min-content contribution in this same prepared context.
    ReuseMinContent,
    /// A border-box known dimension for an actual child query, without margins.
    KnownBorder(S),
    ContentSuggestion(GridMinimumContentSuggestion<S>),
}

pub(in crate::grid) struct GridMinimumContentSuggestion<S: LayoutScalar> {
    query: ComputeInputOf<S>,
    adjustment: S,
    padding_border: S,
    lower: Option<S>,
    upper: Option<S>,
    cap: Option<S>,
    maximum: Option<S>,
}

impl<S: LayoutScalar> GridMinimumContentSuggestion<S> {
    pub(in crate::grid) const fn query(&self) -> ComputeInputOf<S> {
        self.query
    }

    /// The input is the queried border extent. All suggestion clamps
    /// below operate in the item's CSS sizing-box coordinates before restoring
    /// the border-box known dimension.
    pub(in crate::grid) fn finish(self, actual_border: S) -> Result<S, SizingResolutionError<S>> {
        let suggestion = finite(actual_border)?;
        if suggestion < S::ZERO {
            return Err(invalid(suggestion));
        }
        let suggestion = finite(suggestion - self.adjustment)?.max(S::ZERO);
        let suggestion = clamp(suggestion, self.lower, self.upper);
        let suggestion = clamp(suggestion, None, self.cap);
        let suggestion = clamp(suggestion, None, self.maximum);
        border_known(suggestion, self.adjustment, self.padding_border)
    }
}

/// Execute the planned used-minimum query through the caller's real child
/// boundary, preserving its normal box and selected baseline facts.
pub(in crate::grid) fn compute_grid_minimum_contribution<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    context: GridMinimumContext<'_, '_, S>,
    query: impl FnMut(ComputeInputOf<S>) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M>,
) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M> {
    GridMinimumQuery::begin(node, context)?.compute(node, query)
}

/// A minimum contribution may need a content suggestion before it can issue
/// the final child request with a known border extent.
pub(in crate::grid) enum GridMinimumQuery<S: LayoutScalar> {
    Final(ComputeInputOf<S>),
    Suggestion {
        prepared: ComputeInputOf<S>,
        axis: PhysicalAxis,
        suggestion: GridMinimumContentSuggestion<S>,
    },
}

type GridMinimumReceipt<Node, S, M> =
    LayoutResultOf<Node, ControlFlow<ComputeOutputOf<S>, ComputeInputOf<S>>, S, M>;

impl<S: LayoutScalar> GridMinimumQuery<S> {
    pub(in crate::grid) fn begin<Node: Copy, M>(
        node: Node,
        context: GridMinimumContext<'_, '_, S>,
    ) -> LayoutResultOf<Node, Self, S, M> {
        let prepared = context.prepared;
        let axis = grid_axis_physical_axis(context.flow_axes, context.axis);
        let plan =
            plan_grid_minimum(context).map_err(|error| sizing_resolution_error(node, error))?;
        Ok(match plan {
            GridMinimumPlan::ReuseMinContent => Self::Final(prepared),
            GridMinimumPlan::KnownBorder(border) => {
                Self::Final(minimum_known_border_input(prepared, axis, border))
            }
            GridMinimumPlan::ContentSuggestion(suggestion) => Self::Suggestion {
                prepared,
                axis,
                suggestion,
            },
        })
    }

    pub(in crate::grid) fn compute<Node: Copy, M>(
        self,
        node: Node,
        mut query: impl FnMut(ComputeInputOf<S>) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M>,
    ) -> LayoutResultOf<Node, ComputeOutputOf<S>, S, M> {
        let output = query(self.input())?;
        match self.accept(node, output)? {
            ControlFlow::Break(output) => Ok(output),
            ControlFlow::Continue(input) => query(input),
        }
    }

    pub(in crate::grid) fn input(&self) -> ComputeInputOf<S> {
        match self {
            Self::Final(input) => *input,
            Self::Suggestion { suggestion, .. } => suggestion.query(),
        }
    }

    pub(in crate::grid) fn accept<Node: Copy, M>(
        self,
        node: Node,
        output: ComputeOutputOf<S>,
    ) -> GridMinimumReceipt<Node, S, M> {
        match self {
            Self::Final(_) => Ok(ControlFlow::Break(output)),
            Self::Suggestion {
                prepared,
                axis,
                suggestion,
            } => {
                let actual_border = match axis {
                    PhysicalAxis::Horizontal => output.size.width,
                    PhysicalAxis::Vertical => output.size.height,
                };
                let border = suggestion
                    .finish(actual_border)
                    .map_err(|error| sizing_resolution_error(node, error))?;
                Ok(ControlFlow::Continue(minimum_known_border_input(
                    prepared, axis, border,
                )))
            }
        }
    }
}

fn minimum_known_border_input<S: LayoutScalar>(
    prepared: ComputeInputOf<S>,
    physical_axis: PhysicalAxis,
    known_border: S,
) -> ComputeInputOf<S> {
    let mut known = prepared.known();
    match physical_axis {
        PhysicalAxis::Horizontal => known.width = Some(known_border),
        PhysicalAxis::Vertical => known.height = Some(known_border),
    }
    let mut input = ComputeInputOf::for_child(
        prepared.run_mode(),
        prepared.sizing_mode(),
        prepared.requested_axis(),
        known,
        prepared.parent(),
        prepared.containing_layout_context(),
        prepared
            .sizing_request()
            .with_axes(prepared.sizing_request().axes()),
    )
    .with_baseline_demand_from(prepared)
    .with_settled_auto_scrollbars(prepared.settled_auto_scrollbars())
    .with_containing_auto_scrollbar_pass(prepared.containing_auto_scrollbar_pass());
    if let Some(indent) = prepared.block_indent_context() {
        input = input.with_block_indent_context(indent);
    }
    input
}

/// Plan only: the caller executes every real child query and retains its
/// baselines. No measured output or preferred property is synthesized here.
pub(in crate::grid) fn plan_grid_minimum<S: LayoutScalar>(
    context: GridMinimumContext<'_, '_, S>,
) -> Result<GridMinimumPlan<S>, SizingResolutionError<S>> {
    let item = context.item;
    let physical = grid_axis_physical_axis(context.flow_axes, context.axis);
    let opposite = match physical {
        PhysicalAxis::Horizontal => PhysicalAxis::Vertical,
        PhysicalAxis::Vertical => PhysicalAxis::Horizontal,
    };
    let parent = context.prepared.parent();
    let preferred = resolve_preferred_sizing(
        axis_ref(item.size, physical),
        context.algorithm,
        physical,
        axis_value(parent, physical),
        true,
    )?;
    // A resolved percentage remains container-dependent. Conversely an
    // intrinsic keyword is not collapsed into Auto by an optional resolver.
    if !axis_ref(item.size, physical).depends_on_basis()
        && !matches!(preferred, ResolvedPreferredSize::Auto)
    {
        return Ok(GridMinimumPlan::ReuseMinContent);
    }

    let GridItemBoxEdges {
        padding,
        border,
        box_sizing_adjustment,
    } = grid_item_box_edges(item, context.prepared.containing_flow_axes(), parent)?;
    let padding_border = finite(axis_value((padding + border).sum_axes(), physical))?;
    let adjustment = finite(axis_value(box_sizing_adjustment, physical))?;
    let minimum = resolve_standalone_subgrid_item_minimum_optional(
        axis_ref(item.min_size, physical),
        item,
        context.subgrid_requests,
        context.algorithm,
        physical,
        // A cyclic minimum percentage resolves against zero for intrinsic
        // contributions, including any nonpercentage term in a calculation.
        Some(axis_value(parent, physical).unwrap_or(S::ZERO)),
        false,
    )?;

    if let GridItemMinimum::StandaloneIntrinsic(minimum) = minimum {
        return Ok(GridMinimumPlan::ContentSuggestion(
            GridMinimumContentSuggestion {
                query: content_query(context.prepared, physical, minimum.operation()),
                adjustment,
                padding_border,
                lower: None,
                upper: None,
                cap: None,
                maximum: None,
            },
        ));
    }
    if !axis_ref(item.min_size, physical).is_auto() {
        return Ok(GridMinimumPlan::KnownBorder(border_known(
            minimum.resolved().unwrap_or(S::ZERO),
            adjustment,
            padding_border,
        )?));
    }

    let automatic_content_minimum =
        !grid_axis_computed_overflow(item, context.flow_axes, context.axis).is_scrollable()
            && context.span.content_minimum_eligible;
    if !automatic_content_minimum {
        return Ok(GridMinimumPlan::KnownBorder(padding_border));
    }

    let maximum = resolve_maximum_optional(
        axis_ref(item.max_size, physical),
        context.algorithm,
        physical,
        axis_value(parent, physical),
        true,
    )?;
    let margin = transpose_edges_result(
        context
            .prepared
            .containing_flow_axes()
            .zip_physical_edges_with_inline_extent(
                item.margin,
                context.margin_basis,
                resolve_auto_or_zero,
            ),
    )?;
    let span_cap = context.span.fixed_max_span;
    let cap = sizing_cap(
        span_cap,
        axis_value(margin.sum_axes(), physical),
        adjustment,
    )?;
    if let ResolvedPreferredSize::Definite(specified) = preferred {
        let specified = clamp(specified, None, cap);
        let specified = clamp(specified, None, maximum);
        return Ok(GridMinimumPlan::KnownBorder(border_known(
            specified,
            adjustment,
            padding_border,
        )?));
    }

    let opposite_minimum = resolve_standalone_subgrid_item_minimum_optional(
        axis_ref(item.min_size, opposite),
        item,
        context.subgrid_requests,
        context.algorithm,
        opposite,
        axis_value(parent, opposite),
        true,
    )?
    .resolved();
    let opposite_maximum = resolve_maximum_optional(
        axis_ref(item.max_size, opposite),
        context.algorithm,
        opposite,
        axis_value(parent, opposite),
        true,
    )?;
    let mut lower = None;
    let mut upper = None;
    if let Some(ratio) = item.aspect_ratio {
        let transfer = |value| transfer_size(value, ratio.get(), physical);
        lower = opposite_minimum.map(transfer).transpose()?;
        upper = opposite_maximum.map(transfer).transpose()?;
        // Only the actual opposite preferred property establishes a transfer.
        // A span/stretch known dimension is deliberately not inspected here.
        let opposite_preferred = resolve_preferred_sizing(
            axis_ref(item.size, opposite),
            context.algorithm,
            opposite,
            axis_value(parent, opposite),
            true,
        )?;
        if item.item_is_replaced
            && let ResolvedPreferredSize::Definite(value) = opposite_preferred
        {
            let opposite_cap = sizing_cap(
                context.opposite_fixed_max_span,
                axis_value(margin.sum_axes(), opposite),
                axis_value(box_sizing_adjustment, opposite),
            )?;
            let value = clamp(value, opposite_minimum, opposite_maximum);
            let value = clamp(value, None, opposite_cap);
            let transferred = transfer(value)?;
            // Grid's transfer cap resolves an indefinite percentage against
            // zero; this is separate from ordinary intrinsic basis resolution.
            let transfer_cap = replaced_cap(&context, physical)?;
            let transferred = clamp(transferred, None, transfer_cap);
            return Ok(GridMinimumPlan::KnownBorder(border_known(
                transferred,
                adjustment,
                padding_border,
            )?));
        }
    }

    // All represented replaced items are compressible under the retained
    // sizing rule. No control identity is inferred for non-replaced inputs.
    let maximum = if item.item_is_replaced {
        replaced_cap(&context, physical)?
    } else {
        maximum
    };
    Ok(GridMinimumPlan::ContentSuggestion(
        GridMinimumContentSuggestion {
            query: content_query(context.prepared, physical, SizingOperation::MinContent),
            adjustment,
            padding_border,
            lower,
            upper,
            cap,
            maximum,
        },
    ))
}

fn replaced_cap<S: LayoutScalar>(
    context: &GridMinimumContext<'_, '_, S>,
    axis: PhysicalAxis,
) -> Result<Option<S>, SizingResolutionError<S>> {
    let basis = Some(axis_value(context.prepared.parent(), axis).unwrap_or(S::ZERO));
    let preferred = resolve_preferred_sizing(
        axis_ref(context.item.size, axis),
        context.algorithm,
        axis,
        basis,
        false,
    )?;
    let maximum = resolve_maximum_optional(
        axis_ref(context.item.max_size, axis),
        context.algorithm,
        axis,
        basis,
        false,
    )?;
    Ok(match preferred {
        ResolvedPreferredSize::Definite(value) => Some(clamp(value, None, maximum)),
        ResolvedPreferredSize::Auto
        | ResolvedPreferredSize::MinContent
        | ResolvedPreferredSize::MaxContent => maximum,
    })
}

fn content_query<S: LayoutScalar>(
    prepared: ComputeInputOf<S>,
    axis: PhysicalAxis,
    intrinsic: SizingOperation,
) -> ComputeInputOf<S> {
    let mut known = prepared.known();
    let mut available = prepared.sizing_request().axes();
    match axis {
        PhysicalAxis::Horizontal => {
            known.width = None;
            available.width = available.width.with_operation(intrinsic);
        }
        PhysicalAxis::Vertical => {
            known.height = None;
            available.height = available.height.with_operation(intrinsic);
        }
    }
    let mut input = ComputeInputOf::for_child(
        RunMode::ComputeSize,
        SizingMode::ContentSize,
        prepared.requested_axis(),
        known,
        prepared.parent(),
        prepared.containing_layout_context(),
        prepared.sizing_request().with_axes(available),
    )
    .with_settled_auto_scrollbars(prepared.settled_auto_scrollbars())
    .with_containing_auto_scrollbar_pass(prepared.containing_auto_scrollbar_pass());
    if let Some(indent) = prepared.block_indent_context() {
        input = input.with_block_indent_context(indent);
    }
    input
}

fn axis_ref<T>(size: &Size<T>, axis: PhysicalAxis) -> &T {
    match axis {
        PhysicalAxis::Horizontal => &size.width,
        PhysicalAxis::Vertical => &size.height,
    }
}

fn axis_value<T>(size: Size<T>, axis: PhysicalAxis) -> T {
    match axis {
        PhysicalAxis::Horizontal => size.width,
        PhysicalAxis::Vertical => size.height,
    }
}

fn transfer_size<S: LayoutScalar>(
    value: S,
    ratio: S,
    axis: PhysicalAxis,
) -> Result<S, SizingResolutionError<S>> {
    finite(match axis {
        PhysicalAxis::Horizontal => value * ratio,
        PhysicalAxis::Vertical => value / ratio,
    })
}

fn clamp<S: LayoutScalar>(value: S, minimum: Option<S>, maximum: Option<S>) -> S {
    let value = maximum.map_or(value, |maximum| value.min(maximum));
    minimum.map_or(value, |minimum| value.max(minimum))
}

fn sizing_cap<S: LayoutScalar>(
    border_span: Option<S>,
    margin: S,
    adjustment: S,
) -> Result<Option<S>, SizingResolutionError<S>> {
    border_span
        .map(|span| finite(finite(span - margin)? - adjustment).map(|value| value.max(S::ZERO)))
        .transpose()
}

fn border_known<S: LayoutScalar>(
    value: S,
    adjustment: S,
    padding_border: S,
) -> Result<S, SizingResolutionError<S>> {
    Ok(finite(value + adjustment)?.max(padding_border))
}

fn finite<S: LayoutScalar>(value: S) -> Result<S, SizingResolutionError<S>> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(invalid(value))
    }
}

fn invalid<S: LayoutScalar>(value: S) -> SizingResolutionError<S> {
    LengthResolutionStatus::InvalidNumeric { value }.into()
}

#[cfg(test)]
#[path = "minimum_tests.rs"]
mod tests;
