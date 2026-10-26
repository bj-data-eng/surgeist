//! The existing block formatter's concrete consuming Text adapter.

#[cfg(test)]
#[path = "wrapping_run_tests.rs"]
mod wrapping_tests;

use super::{
    TextParagraphBinding,
    fit::{self, Band, FitError, FittedLine},
    state::PendingParagraph,
};
use crate::inline::{
    AtomicInlineBoxParticipant, AtomicInlineSourceOf, InlineControlAlignment, LogicalLineBandOf,
    MixedInlineRunReportOf, PostLineClearIntent,
};
use crate::inline::{InlineControlSourceOf, InlineParticipantLayoutKind};
use crate::{
    AvailableOf, Direction, FlowAxes, InlineFragmentOutputOf, LayoutScalar, LayoutTextNumericError,
    Point, ScrollRectOf, Size, TextAlign, TextAlignLast, TextJustification,
};
use surgeist_text::{
    AtomicAllocation, AtomicBoxAllocation, AtomicFlowAllocation, AtomicLineAllocation,
    AtomicPlacement, FinalizedLine, LineAnchor, LineOrientation, LinePlacement,
    ParagraphBaseDirection, Rect,
};

/// Actual per-run incidence projection. Errors remain data until a real soft
/// choice comparison needs the corresponding requested scope.
pub(crate) struct WrappingProjection<Node> {
    context: std::sync::Arc<crate::inline_wrapping::WrappingContext<Node>>,
    flow: Node,
    edges: Vec<(
        usize,
        Result<EdgeIncidence, crate::inline_wrapping::WrappingScopeError>,
    )>,
    boxes: Vec<(
        usize,
        crate::inline_wrapping::ScopeRow<Node>,
        Result<ScopeIncidence, crate::WrappingScopeError>,
    )>,
    incoming: Option<super::boxes::BoxFrontier>,
    physical_controls: Vec<(Node, surgeist_text::PreparedControl)>,
    physical_positions: Vec<PhysicalPosition<Node>>,
    unlisted: Vec<surgeist_text::CapturedScope>,
    canonical_pairs: Vec<(surgeist_text::CapturedScope, (Node, Node))>,
    computed_cuts: Vec<(
        surgeist_text::PreparedComputedLineBoundary,
        super::capture::ComputedNativeCut,
    )>,
    rows: Vec<(
        crate::inline_wrapping::ScopeRow<Node>,
        Result<ScopeIncidence, crate::inline_wrapping::WrappingScopeError>,
    )>,
}

struct PhysicalPosition<Node> {
    node: Node,
    gaps: Result<(usize, usize), crate::WrappingScopeError>,
}

enum EdgeIncidence {
    Outside,
    Offset(usize),
    Control {
        control: surgeist_text::PreparedControl,
        after: bool,
    },
}

#[derive(Clone)]
enum ScopeIncidence {
    Outside,
    Captured(surgeist_text::CapturedScope),
    Native {
        start: EndpointIncidence,
        end: EndpointIncidence,
    },
}

#[derive(Clone)]
enum EndpointIncidence {
    BeforeRun,
    AfterRun,
    Local(surgeist_text::PreparedControl),
    Unavailable,
}

impl EndpointIncidence {
    fn consumed(
        &self,
        prepared: &surgeist_text::PreparedText,
        end: &surgeist_text::PreparedBoundary,
    ) -> Result<bool, FitError> {
        use crate::inline_wrapping::WrappingScopeError;
        match self {
            Self::BeforeRun => Ok(true),
            Self::AfterRun => Ok(false),
            Self::Local(control) => {
                if !prepared.control_is_boundary_exact(control)? {
                    return Err(WrappingScopeError::UnsupportedIncidence.into());
                }
                Ok(prepared.boundary_consumes_control(end, control)?)
            }
            Self::Unavailable => Err(WrappingScopeError::UnsupportedIncidence.into()),
        }
    }
}

impl<Node: Copy + Eq> WrappingProjection<Node> {
    pub(crate) fn new(
        context: &std::sync::Arc<crate::inline_wrapping::WrappingContext<Node>>,
        graph: &crate::engine::validation::ValidationGraph<Node>,
        binding: &TextParagraphBinding<Node>,
        associations: &super::capture::ScopeAssociations<Node>,
    ) -> Option<Self> {
        use super::capture::CapturedScopeRelation;
        use crate::inline_wrapping::{ScopeEndpoint, WrappingScopeError};
        let project = |row: &crate::inline_wrapping::ScopeRow<Node>| {
            let relation = row.capture.as_ref().map(|scope| {
                (
                    scope,
                    associations.captured_scope_relation(&binding.prepared, scope),
                )
            });
            match relation {
                Some((_, CapturedScopeRelation::Outside)) => Ok(ScopeIncidence::Outside),
                Some((scope, CapturedScopeRelation::Local)) => {
                    Ok(ScopeIncidence::Captured(scope.clone()))
                }
                Some((_, CapturedScopeRelation::Unavailable)) | None => row
                    .pair
                    .ok_or(WrappingScopeError::UnsupportedIncidence)
                    .and_then(|pair| {
                        context.native_pair_projection(
                            graph,
                            binding.container,
                            &binding.nodes,
                            pair,
                        )
                    })
                    .map(|pair| {
                        let endpoint = |endpoint| match endpoint {
                            ScopeEndpoint::BeforeRun => EndpointIncidence::BeforeRun,
                            ScopeEndpoint::AfterRun => EndpointIncidence::AfterRun,
                            ScopeEndpoint::Local(node) => binding
                                .controls
                                .iter()
                                .find(|control| control.node == node)
                                .map_or(EndpointIncidence::Unavailable, |control| {
                                    EndpointIncidence::Local(control.occurrence.clone())
                                }),
                        };
                        ScopeIncidence::Native {
                            start: endpoint(pair.start),
                            end: endpoint(pair.end),
                        }
                    }),
            }
        };
        let rows = context
            .rows()
            .filter(|row| row.flow == binding.container)
            .map(|row| (row.clone(), project(row)))
            .collect::<Vec<_>>();
        let boxes = context
            .box_rows(binding.container)
            .map(|(id, row)| (id, row.clone(), project(row)))
            .collect::<Vec<_>>();
        let canonical_pairs: Vec<_> = associations
            .native_pairs()
            .map(|(scope, pair)| (scope.clone(), pair))
            .collect();
        let mut physical_positions: Vec<PhysicalPosition<Node>> = Vec::new();
        for (_, row) in context.box_rows(binding.container) {
            for node in row.pair.into_iter().flat_map(|(start, end)| [start, end]) {
                if physical_positions.iter().any(|actual| actual.node == node) {
                    continue;
                }
                let position = context
                    .native_pair_projection(graph, binding.container, &binding.nodes, (node, node))
                    .and_then(|pair| match pair.start {
                        ScopeEndpoint::BeforeRun => Ok((0, 0)),
                        ScopeEndpoint::AfterRun => Ok((binding.nodes.len(), binding.nodes.len())),
                        ScopeEndpoint::Local(node) => binding
                            .nodes
                            .iter()
                            .position(|actual| *actual == node)
                            .map(|at| (at, at + 1))
                            .ok_or(WrappingScopeError::UnsupportedIncidence),
                    });
                physical_positions.push(PhysicalPosition {
                    node,
                    gaps: position,
                });
            }
        }
        for (_, (start, end)) in &canonical_pairs {
            for node in [*start, *end] {
                if physical_positions.iter().any(|actual| actual.node == node) {
                    continue;
                }
                let position = context
                    .native_pair_projection(graph, binding.container, &binding.nodes, (node, node))
                    .and_then(|pair| match pair.start {
                        ScopeEndpoint::BeforeRun => Ok((0, 0)),
                        ScopeEndpoint::AfterRun => Ok((binding.nodes.len(), binding.nodes.len())),
                        ScopeEndpoint::Local(node) => binding
                            .nodes
                            .iter()
                            .position(|actual| *actual == node)
                            .map(|at| (at, at + 1))
                            .ok_or(WrappingScopeError::UnsupportedIncidence),
                    });
                physical_positions.push(PhysicalPosition {
                    node,
                    gaps: position,
                });
            }
        }
        let participants = binding.prepared.participants().collect::<Vec<_>>();
        let physical_edge = |pair: (Node, Node), side: surgeist_text::SourceBoxSide| {
            let node = if side == surgeist_text::SourceBoxSide::Before {
                pair.0
            } else {
                pair.1
            };
            context
                .native_pair_projection(graph, binding.container, &binding.nodes, (node, node))
                .and_then(|projected| match projected.start {
                    ScopeEndpoint::BeforeRun | ScopeEndpoint::AfterRun => {
                        Ok(EdgeIncidence::Outside)
                    }
                    ScopeEndpoint::Local(node) => binding
                        .controls
                        .iter()
                        .find(|c| c.node == node)
                        .map(|c| EdgeIncidence::Control {
                            control: c.occurrence.clone(),
                            after: side == surgeist_text::SourceBoxSide::After,
                        })
                        .ok_or(WrappingScopeError::UnsupportedIncidence),
                })
        };
        let edges=context.edge_locations(binding.container).filter(|(_,edge)|edge.effect.avoided).map(|(id,edge)| {
            let incidence=match &edge.box_ {
                crate::GeneratedBoxRef::Node(node)=> {
                    if let Some(atomic)=binding.atomics.iter().find(|a|a.node==*node) {
                        participants.iter().position(|p|matches!(p,surgeist_text::PreparedParticipant::Atomic(a)if a==&atomic.occurrence))
                            .map(|offset|EdgeIncidence::Offset(offset+usize::from(edge.side==surgeist_text::SourceBoxSide::After)))
                            .ok_or(WrappingScopeError::UnsupportedIncidence)
                    } else {context.native_pair_projection(graph,binding.container,&binding.nodes,(*node,*node)).and_then(|p|match p.start {
                        ScopeEndpoint::BeforeRun|ScopeEndpoint::AfterRun=>Ok(EdgeIncidence::Outside),
                        ScopeEndpoint::Local(_)=>Err(WrappingScopeError::UnsupportedIncidence),
                    })}
                }
                crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::Scope(scope))=>match associations.captured_scope_relation(&binding.prepared,scope){
                    CapturedScopeRelation::Outside=>Ok(EdgeIncidence::Outside),
                    CapturedScopeRelation::Local|CapturedScopeRelation::Unavailable=>context.scope_pair(scope)
                        .ok_or(WrappingScopeError::UnsupportedIncidence).and_then(|pair|physical_edge(pair,edge.side)),
                },
                crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::NativePair{start,end})=>physical_edge((*start,*end),edge.side),
            }; (id,incidence)
        }).collect::<Vec<_>>();
        (!rows.is_empty()
            || !edges.is_empty()
            || boxes
                .iter()
                .any(|(_, row, _)| row.metric_contribution.is_some()))
        .then(|| Self {
            context: context.clone(),
            flow: binding.container,
            rows,
            boxes,
            incoming: None,
            physical_positions,
            canonical_pairs,
            computed_cuts: participants
                .iter()
                .filter_map(|participant| {
                    let surgeist_text::PreparedParticipant::ComputedLineBoundary(boundary) =
                        participant
                    else {
                        return None;
                    };
                    associations
                        .computed_cut(boundary)
                        .map(|cut| (boundary.clone(), cut))
                })
                .collect(),
            unlisted: binding
                .prepared
                .captured_paragraph()
                .map_or_else(Vec::new, |paragraph| {
                    paragraph
                        .scope_intersections()
                        .map(|intersection| intersection.scope())
                        .filter(|scope| {
                            scope.role() == surgeist_text::ScopeRole::Inline
                                && !context
                                    .box_rows(binding.container)
                                    .any(|(_, row)| row.capture.as_ref() == Some(scope))
                        })
                        .collect()
                }),
            physical_controls: binding
                .controls
                .iter()
                .map(|control| (control.node, control.occurrence.clone()))
                .collect(),
            edges,
        })
    }

    pub(crate) fn with_incoming(mut self, incoming: Option<super::boxes::BoxFrontier>) -> Self {
        self.incoming = incoming;
        self
    }

    fn classify_boundary(
        &self,
        prepared: &surgeist_text::PreparedText,
        end: &surgeist_text::PreparedBoundary,
    ) -> Result<crate::inline_wrapping::AvoidClass, FitError> {
        let mut projected = self.rows.iter();
        let mut class = self.context.classify::<FitError>(self.flow, |row| {
            let (actual, incidence) = projected
                .next()
                .ok_or(crate::inline_wrapping::WrappingScopeError::InconsistentIncidence)?;
            if actual != row {
                return Err(
                    crate::inline_wrapping::WrappingScopeError::InconsistentIncidence.into(),
                );
            }
            match incidence
                .as_ref()
                .map_err(|error| FitError::Wrapping(*error))?
            {
                ScopeIncidence::Outside => Ok(false),
                ScopeIncidence::Captured(scope) => {
                    Ok(prepared.boundary_inside_captured_scope(end, scope)?)
                }
                ScopeIncidence::Native {
                    start,
                    end: closing,
                } => Ok(start.consumed(prepared, end)? && !closing.consumed(prepared, end)?),
            }
        })?;
        for (id, incidence) in &self.edges {
            let matched = match incidence.as_ref().map_err(|e| FitError::Wrapping(*e))? {
                EdgeIncidence::Outside => false,
                EdgeIncidence::Offset(offset) => end.view().participant_offset() == *offset,
                EdgeIncidence::Control { control, after } => {
                    if !prepared.control_is_boundary_exact(control)? {
                        return Err(crate::WrappingScopeError::UnsupportedIncidence.into());
                    }
                    let at=prepared.participants().position(|p|matches!(p,surgeist_text::PreparedParticipant::Control(c)if &c==control)).ok_or(crate::WrappingScopeError::UnsupportedIncidence)?;
                    end.view().participant_offset() == at + usize::from(*after)
                }
            };
            if matched {
                class.add_edge(*id);
            }
        }
        Ok(class)
    }
}

pub(crate) struct Atomic<Node, S: LayoutScalar> {
    pub node: Node,
    pub item: AtomicInlineBoxParticipant<S>,
    pub content_box: ScrollRectOf<S>,
    pub relative_offset: Point<S>,
}

pub(crate) struct Control<Node, S: LayoutScalar> {
    pub node: Node,
    pub source_index: usize,
    pub kind: InlineParticipantLayoutKind,
    pub allocation: surgeist_text::ControlAllocation,
    pub endpoint: Option<crate::inline::InlineEndpointOf<S>>,
}

#[derive(Clone, Copy)]
pub(crate) enum Frame<S: LayoutScalar> {
    Measurement,
    Canonical { origin: Point<f64>, size: Size<S> },
}

pub(crate) struct Context<Node, S: LayoutScalar> {
    pub wrapping: Option<WrappingProjection<Node>>,
    pub formatted: super::formatted::FormattedTailSnapshot,
    pub line_box_context: crate::line_box::LineBoxContext,
    pub frame: Frame<S>,
    pub containing_size: Size<S>,
    pub inset: Point<S>,
    pub right_inset: S,
    pub cursor_block: S,
    pub axes: FlowAxes,
    pub available: AvailableOf<S>,
    pub alignment: TextAlign,
    pub last_alignment: TextAlignLast,
    pub justification: TextJustification,
    pub indent: crate::text_indent::ResolvedTextIndent<S>,
    pub indent_state: crate::text_indent::IndentLineState,
    pub hyphenation_state: crate::hyphenation::LineHyphenationState,
    pub hyphenate_limit_zone: crate::LengthPercentageOf<S>,
    pub tail: fit::TailContext,
}

impl<Node, S: LayoutScalar> Context<Node, S> {
    fn inline_start_inset(&self) -> S {
        self.axes
            .logical_edges(crate::Edges::new(
                self.inset.y,
                self.right_inset,
                S::ZERO,
                self.inset.x,
            ))
            .inline_start
    }
}

/// Separate normal-flow allocation and rigidly placed Text metric bounds.
pub(crate) struct LineBounds<S: LayoutScalar> {
    pub allocation: ScrollRectOf<S>,
    pub metrics: ScrollRectOf<S>,
}

pub(crate) struct Output<Node, S: LayoutScalar> {
    pub formatted: super::formatted::FormattedTailUses,
    pub box_uses: Vec<super::boxes::BoxTrialUse>,
    pub report: MixedInlineRunReportOf<S>,
    pub paragraph: PendingParagraph<Node>,
    pub line_bounds: Vec<LineBounds<S>>,
    pub fragments: Vec<(crate::InlineFragmentOwner<Node>, InlineFragmentOutputOf<S>)>,
    pub atomic_origins: Vec<(Node, Point<f64>)>,
    pub static_positions: Vec<(Node, Point<S>)>,
}

/// One invocation's coupled line-band and source-event transitions. Each
/// callback borrows the same actual provisional session, never a public root.
pub(crate) struct LineEnvironment<Band, Clear, Replay, Accept, Reset> {
    pub band: Band,
    pub reset: Reset,
    pub clear: Clear,
    pub replay: Replay,
    pub accept: Accept,
}

pub(crate) fn layout<Node: Copy + Eq, S: LayoutScalar>(
    binding: &TextParagraphBinding<Node>,
    associations: &super::capture::ScopeAssociations<Node>,
    atomics: &[Atomic<Node, S>],
    controls: &[Control<Node, S>],
    context: Context<Node, S>,
    reusable: &[fit::ReusableLine],
    environment: LineEnvironment<
        impl FnMut(f64, f64) -> Result<Band, FitError>,
        impl FnMut(f64, &surgeist_text::LineSelectionRequest) -> Result<Option<f64>, FitError>,
        impl FnMut(&FinalizedLine, fit::EncounterReplay) -> Result<Vec<fit::EncounterBasis>, FitError>,
        impl FnMut() -> Result<(), FitError>,
        impl FnMut(fit::EncounterReset),
    >,
) -> Result<Output<Node, S>, FitError> {
    let LineEnvironment {
        band: band_source,
        reset: reset_candidate,
        clear: clear_source,
        replay: replay_encounters,
        accept: accept_encounters,
    } = environment;
    let origin = match context.frame {
        Frame::Measurement => Point::ZERO,
        Frame::Canonical { origin, .. } => origin,
    };
    let allocations = binding
        .atomics
        .iter()
        .map(|row| {
            if row.occurrence.view().original_box().kind()
                == surgeist_text::InlineBoxKind::OutOfFlow
            {
                Ok(AtomicAllocation {
                    occurrence: row.occurrence.clone(),
                    allocation: AtomicLineAllocation::OutOfFlow,
                })
            } else {
                initial_allocation(
                    binding,
                    atomics
                        .iter()
                        .find(|atomic| atomic.node == row.node)
                        .expect("capture admitted the actual in-flow atomic measurement"),
                )
            }
        })
        .collect::<Result<Vec<_>, _>>()?;
    let control_allocations: Vec<_> = controls
        .iter()
        .map(|control| control.allocation.clone())
        .collect();
    let direction = match binding.prepared.analyzed_source().direction() {
        surgeist_text::Direction::RightToLeft => ParagraphBaseDirection::RightToLeft,
        surgeist_text::Direction::LeftToRight => ParagraphBaseDirection::LeftToRight,
        surgeist_text::Direction::Auto => {
            unreachable!("analysis lends a resolved original direction")
        }
    };
    let orientation = if context.axes.writing_mode().is_vertical() {
        LineOrientation::Vertical
    } else {
        LineOrientation::Horizontal
    };
    let available = match context.available {
        AvailableOf::Definite(value) => fit::Available::Definite(value.to_f64()),
        AvailableOf::MinContent => fit::Available::Minimum,
        AvailableOf::MaxContent => fit::Available::Maximum,
    };
    let wrapping = context
        .wrapping
        .as_ref()
        .map(|projection| projection as &fit::WrappingClassifier<'_>);
    let lines = fit::fit(
        fit::FitInput {
            wrapping,
            formatted: &context.formatted,
            line_box_context: context.line_box_context,
            prepared: &binding.prepared,
            available,
            base_direction: direction,
            block_direction: match context.axes.direction() {
                Direction::Ltr => ParagraphBaseDirection::LeftToRight,
                Direction::Rtl => ParagraphBaseDirection::RightToLeft,
            },
            alignment: context.alignment,
            last_alignment: context.last_alignment,
            justification: context.justification,
            indent: context.indent.widened(),
            indent_state: context.indent_state,
            hyphenation_state: context.hyphenation_state,
            tail: context.tail,
            hyphenate_limit_zone: crate::LengthPercentageOf::from_coefficients(
                context.hyphenate_limit_zone.absolute_px().to_f64(),
                context.hyphenate_limit_zone.percent_fraction().to_f64(),
            )
            .map_err(|_| FitError::InvalidGeometry)?,
            orientation,
            atomics: &allocations,
            controls: &control_allocations,
            line_strut: binding.line_strut,
            reusable,
        },
        band_source,
        |position, baseline| {
            let absolute = add(
                add(origin.y.to_f64(), context.cursor_block.to_f64())?,
                add(position.proposed, baseline)?,
            )?;
            let mut canonical = round(absolute)?;
            if let Some(minimum) = position.minimum {
                let absolute_minimum = add(
                    add(origin.y, context.cursor_block.to_f64())?,
                    add(minimum, baseline)?,
                )?;
                // A real float or clear constraint excludes earlier anchors.
                // Choose the first permitted point on the same CSS-pixel grid;
                // ordinary unconstrained placement still uses nearest rounding.
                canonical = canonical.max(finite(absolute_minimum.ceil())?);
            }
            finite(canonical - origin.y.to_f64() - context.cursor_block.to_f64() - baseline)
        },
        |line, band, block, box_struts| {
            Ok(fit::LineAllocations {
                line_box_struts: canonical_box_struts(
                    box_struts, &context, origin, line, band, block,
                )?,
                atomics: canonical_allocations(
                    binding, atomics, &context, origin, line, band, block,
                )?,
                controls: canonical_controls(controls, &context, origin, line, band, block)?,
                line_strut: canonical_line_strut(
                    binding.line_strut,
                    &context,
                    origin,
                    line,
                    band,
                    block,
                )?,
            })
        },
        clear_source,
        fit::EncounterCallbacks {
            replay: replay_encounters,
            accept: accept_encounters,
            reset: reset_candidate,
        },
    )?;
    output(
        binding,
        associations,
        atomics,
        controls,
        &context,
        origin,
        lines,
    )
}

fn initial_allocation<Node: Copy + Eq, S: LayoutScalar>(
    binding: &TextParagraphBinding<Node>,
    atomic: &Atomic<Node, S>,
) -> Result<AtomicAllocation, FitError> {
    let occurrence = binding
        .atomics
        .iter()
        .find(|row| row.node == atomic.node)
        .expect("capture admitted every actual atomic node")
        .occurrence
        .clone();
    let item = atomic.item;
    let baseline = item
        .first_baseline
        .unwrap_or(item.size.height + item.margin.bottom)
        .to_f64();
    let x = add(item.margin.left.to_f64(), atomic.relative_offset.x.to_f64())?;
    let y = add(-baseline, atomic.relative_offset.y.to_f64())?;
    let content = atomic.content_box;
    Ok(AtomicAllocation {
        occurrence,
        allocation: AtomicLineAllocation::InFlow {
            alignment: alignment(item.alignment),
            flow: flow(
                item,
                item.size.width.to_f64(),
                item.size.height.to_f64(),
                baseline,
            )?,
            box_geometry: AtomicBoxAllocation {
                border: Rect::new(x, y, item.size.width.to_f64(), item.size.height.to_f64()),
                content: Rect::new(
                    add(x, content.origin().x.to_f64())?,
                    add(y, content.origin().y.to_f64())?,
                    content.size().width.to_f64(),
                    content.size().height.to_f64(),
                ),
                baseline: atomic.relative_offset.y.to_f64(),
            },
        },
    })
}

fn flow<S: LayoutScalar>(
    item: AtomicInlineBoxParticipant<S>,
    width: f64,
    height: f64,
    baseline: f64,
) -> Result<AtomicFlowAllocation, FitError> {
    let left = item.margin.left.to_f64();
    let right = item.margin.right.to_f64();
    let advance = add(add(left, width)?, right)?;
    Ok(AtomicFlowAllocation {
        advance,
        before_baseline: add(item.margin.top.to_f64(), baseline)?,
        after_baseline: if item.first_baseline.is_some() {
            add(height - baseline, item.margin.bottom.to_f64())?
        } else {
            0.0
        },
        occupied_start: 0.0f64.min(left).min(advance),
        occupied_end: 0.0f64.max(add(left, width)?).max(advance),
    })
}

fn canonical_allocations<Node: Copy + Eq, S: LayoutScalar>(
    binding: &TextParagraphBinding<Node>,
    atomics: &[Atomic<Node, S>],
    context: &Context<Node, S>,
    origin: Point<f64>,
    line: &FinalizedLine,
    band: Band,
    block: f64,
) -> Result<Vec<AtomicAllocation>, FitError> {
    let feedback = line.feedback();
    let anchor = line_anchor(
        context,
        origin,
        band.start,
        line.metric_bounds().size.width,
        block,
        feedback.baseline,
    )?;
    line.atomic_slots()
        .map(|slot| {
            if slot.allocation == AtomicLineAllocation::OutOfFlow {
                return Ok(AtomicAllocation {
                    occurrence: slot.occurrence,
                    allocation: AtomicLineAllocation::OutOfFlow,
                });
            }
            let actual = binding
                .atomics
                .iter()
                .find(|row| row.occurrence == slot.occurrence)
                .expect("finalizer returned an admitted atomic occurrence");
            let atomic = atomics
                .iter()
                .find(|atomic| atomic.node == actual.node)
                .expect("every admitted in-flow atomic was measured by its Layout owner");
            let item = atomic.item;
            let baseline = item
                .first_baseline
                .unwrap_or(item.size.height + item.margin.bottom)
                .to_f64();
            let (raw_x, raw_y) = raw_box(atomic, anchor, slot.origin, feedback.baseline)?;
            let slot_y = add(anchor.baseline, slot.origin.y - feedback.baseline)?;
            let left = round(raw_x)?;
            let top = round(raw_y)?;
            let right = round(add(raw_x, item.size.width.to_f64())?)?;
            let bottom = round(add(raw_y, item.size.height.to_f64())?)?;
            let width = finite((right - left).max(0.0))?;
            let height = finite((bottom - top).max(0.0))?;
            let content = atomic.content_box;
            let content_x = round(add(raw_x, content.origin().x.to_f64())?)?;
            let content_y = round(add(raw_y, content.origin().y.to_f64())?)?;
            let content_right = round(add(
                add(raw_x, content.origin().x.to_f64())?,
                content.size().width.to_f64(),
            )?)?;
            let content_bottom = round(add(
                add(raw_y, content.origin().y.to_f64())?,
                content.size().height.to_f64(),
            )?)?;
            let actual_baseline = round(add(raw_y, baseline)?)?;
            Ok(AtomicAllocation {
                occurrence: slot.occurrence,
                allocation: AtomicLineAllocation::InFlow {
                    alignment: alignment(item.alignment),
                    flow: flow(item, width, height, actual_baseline - top)?,
                    box_geometry: AtomicBoxAllocation {
                        border: Rect::new(
                            left - anchor.inline_origin - slot.origin.x,
                            top - slot_y,
                            width,
                            height,
                        ),
                        content: Rect::new(
                            content_x - anchor.inline_origin - slot.origin.x,
                            content_y - slot_y,
                            (content_right - content_x).max(0.0),
                            (content_bottom - content_y).max(0.0),
                        ),
                        baseline: actual_baseline - slot_y,
                    },
                },
            })
        })
        .collect()
}

fn canonical_controls<Node: Copy + Eq, S: LayoutScalar>(
    controls: &[Control<Node, S>],
    context: &Context<Node, S>,
    origin: Point<f64>,
    line: &FinalizedLine,
    band: Band,
    block: f64,
) -> Result<Vec<surgeist_text::ControlAllocation>, FitError> {
    let feedback = line.feedback();
    let anchor = line_anchor(
        context,
        origin,
        band.start,
        line.metric_bounds().size.width,
        block,
        feedback.baseline,
    )?;
    line.selected_line()
        .request()
        .controls
        .iter()
        .map(|allocation| {
            let original = controls
                .iter()
                .find(|control| control.allocation.occurrence == allocation.occurrence)
                .expect("selected control retains its original Layout allocation");
            let strut = original
                .allocation
                .strut
                .map(|strut| {
                    let top = match strut.alignment {
                        surgeist_text::LineBoxAlignment::Baseline => {
                            anchor.baseline - strut.baseline
                        }
                        surgeist_text::LineBoxAlignment::Top => anchor.baseline - feedback.baseline,
                        surgeist_text::LineBoxAlignment::Bottom => {
                            anchor.baseline - feedback.baseline + feedback.line_height
                                - strut.line_extent
                        }
                    };
                    let canonical_top = round(top)?;
                    let baseline = round(add(top, strut.baseline)?)? - canonical_top;
                    let extent = round(add(top, strut.line_extent)?)? - canonical_top;
                    Ok::<_, FitError>(surgeist_text::ControlStrut {
                        baseline,
                        line_extent: extent,
                        alignment: strut.alignment,
                    })
                })
                .transpose()?;
            Ok(surgeist_text::ControlAllocation {
                occurrence: allocation.occurrence.clone(),
                strut,
                boundary: original.allocation.boundary,
            })
        })
        .collect()
}

fn canonical_box_struts<Node, S: LayoutScalar>(
    originals: &[surgeist_text::LineBoxStrut],
    context: &Context<Node, S>,
    origin: Point<f64>,
    line: &FinalizedLine,
    band: Band,
    block: f64,
) -> Result<Vec<surgeist_text::LineBoxStrut>, FitError> {
    if originals.is_empty() {
        return Ok(Vec::new());
    }
    let feedback = line.feedback();
    let anchor = line_anchor(
        context,
        origin,
        band.start,
        line.metric_bounds().size.width,
        block,
        feedback.baseline,
    )?;
    originals
        .iter()
        .map(|strut| {
            let top = match strut.alignment {
                surgeist_text::LineBoxAlignment::Baseline => {
                    anchor.baseline - strut.metrics.baseline
                }
                surgeist_text::LineBoxAlignment::Top => anchor.baseline - feedback.baseline,
                surgeist_text::LineBoxAlignment::Bottom => {
                    anchor.baseline - feedback.baseline + feedback.line_height
                        - strut.metrics.line_extent
                }
            };
            let canonical_top = round(top)?;
            Ok(surgeist_text::LineBoxStrut {
                metrics: surgeist_text::LineStrut {
                    baseline: round(add(top, strut.metrics.baseline)?)? - canonical_top,
                    line_extent: round(add(top, strut.metrics.line_extent)?)? - canonical_top,
                },
                alignment: strut.alignment,
            })
        })
        .collect()
}

fn canonical_line_strut<Node, S: LayoutScalar>(
    original: Option<surgeist_text::LineStrut>,
    context: &Context<Node, S>,
    origin: Point<f64>,
    line: &FinalizedLine,
    band: Band,
    block: f64,
) -> Result<Option<surgeist_text::LineStrut>, FitError> {
    let Some(strut) = original else {
        return Ok(None);
    };
    let feedback = line.feedback();
    let anchor = line_anchor(
        context,
        origin,
        band.start,
        line.metric_bounds().size.width,
        block,
        feedback.baseline,
    )?;
    let top = add(anchor.baseline, -strut.baseline)?;
    let canonical_top = round(top)?;
    Ok(Some(surgeist_text::LineStrut {
        baseline: anchor.baseline - canonical_top,
        line_extent: round(add(top, strut.line_extent)?)? - canonical_top,
    }))
}

fn line_inline_origin<Node, S: LayoutScalar>(
    context: &Context<Node, S>,
    origin: f64,
    start: f64,
    available: f64,
) -> Result<f64, FitError> {
    let containing_width = match context.frame {
        Frame::Canonical { size, .. } => size.width,
        Frame::Measurement => context.containing_size.width,
    }
    .to_f64();
    let local_x = if context.axes.direction() == Direction::Rtl {
        containing_width - context.right_inset.to_f64() - start - available
    } else {
        context.inset.x.to_f64() + start
    };
    round(add(origin, local_x)?)
}

fn line_anchor<Node, S: LayoutScalar>(
    context: &Context<Node, S>,
    origin: Point<f64>,
    start: f64,
    available: f64,
    block: f64,
    baseline: f64,
) -> Result<LineAnchor, FitError> {
    Ok(LineAnchor {
        inline_origin: line_inline_origin(context, origin.x, start, available)?,
        baseline: round(add(
            add(origin.y.to_f64(), context.cursor_block.to_f64())?,
            add(block, baseline)?,
        )?)?,
    })
}

fn output<Node: Copy + Eq, S: LayoutScalar>(
    binding: &TextParagraphBinding<Node>,
    associations: &super::capture::ScopeAssociations<Node>,
    atomics: &[Atomic<Node, S>],
    controls: &[Control<Node, S>],
    context: &Context<Node, S>,
    origin: Point<f64>,
    lines: Vec<FittedLine>,
) -> Result<Output<Node, S>, FitError> {
    let mut report = MixedInlineRunReportOf {
        lines: Vec::new(),
        hyphenation_state: context.hyphenation_state,
        indent_state: context.indent_state,
        line_box_context: context.line_box_context,
        inline_extent: S::ZERO,
        block_extent: S::ZERO,
        float_edge_phase: None,
        first_baseline: None,
        last_baseline: None,
        atomics: Vec::new(),
        controls: Vec::new(),
        post_line_clear_intents: Vec::new(),
        line_bands: Vec::new(),
        formatted_lines: None,
    };
    let mut placements = Vec::new();
    let mut bands = Vec::new();
    let mut flows = Vec::new();
    let mut amounts = Vec::new();
    let mut encounters = Vec::new();
    let mut zones = Vec::new();
    let mut phrases = Vec::new();
    let mut box_assignments = Vec::new();
    let mut box_uses = Vec::new();
    let mut line_bounds = Vec::new();
    let mut fragments = Vec::new();
    let mut atomic_origins = Vec::new();
    let mut static_positions = Vec::new();
    let mut outflows = Vec::new();
    let mut formatted = super::formatted::FormattedTailUses::default();
    let observe_formatted = context.formatted.observes()
        || lines.iter().any(|line| {
            line.role_uses
                .iter()
                .any(|used| used.demanded_assignment().is_some())
        });
    for (line_index, line) in lines.into_iter().enumerate() {
        box_assignments.push(line.boxes.clone());
        box_uses.extend(line.box_uses);
        for used in line.role_uses {
            if let Some(demand) = used.demanded_assignment()
                && !formatted.demands.contains(demand)
            {
                formatted.demands.push(demand.clone());
            }
            formatted.roles.push(used);
        }
        if observe_formatted {
            let selected = line.finalized.selected_line();
            formatted
                .accepted
                .push(super::formatted::FormattedTailEndpoint::new(
                    selected.request().end.clone(),
                    selected.request().alternative.clone(),
                ));
        }
        if context.tail != fit::TailContext::Unrestricted
            && matches!(context.available, AvailableOf::Definite(_))
        {
            let selected = line.finalized.selected_line();
            report.lines.push(crate::hyphenation::CommittedInlineLine {
                end: selected.request().context.end,
                content: line.finalized.has_in_flow_content(),
                hyphenated: selected.request().alternative.is_some(),
            });
        }
        report.hyphenation_state = line.hyphenation_end;
        report.line_box_context = line.line_box_end;
        let feedback = line.finalized.feedback();
        let contribution = match context.available {
            AvailableOf::MinContent => line.finalized.measurements().min_content_contribution(),
            AvailableOf::MaxContent => line.finalized.measurements().max_content_contribution(),
            AvailableOf::Definite(_) => line.finalized.measurements().fit(),
        };
        report.indent_state.commit(
            line.finalized.selected_line().request().context.end == surgeist_text::LineEnd::Forced,
        );
        let anchor = line_anchor(
            context,
            origin,
            line.inline_start,
            line.available,
            line.placed_start,
            feedback.baseline,
        )?;
        let local_baseline = anchor.baseline - origin.y.to_f64() - context.cursor_block.to_f64();
        // Normal-flow allocations and placed metric bounds both contribute to
        // overflow, including lines without a source fragment or ink. Intrinsic
        // allocation uses the exact requested contribution plus signed indent;
        // actual Text metric and source-fragment geometry remain independent.
        let width = match line
            .finalized
            .selected_line()
            .request()
            .context
            .available_inline
        {
            Some(width) => width,
            None => finite(
                contribution.advance() + line.finalized.selected_line().request().context.indent,
            )?
            .max(0.0),
        };
        let allocation = ScrollRectOf::try_new(
            Point::new(
                number(
                    line_inline_origin(context, origin.x, line.inline_start, width)? - origin.x,
                )?,
                number(add(context.cursor_block.to_f64(), line.flow.start)?)?,
            ),
            Size::new(number(width)?, number(feedback.line_height)?),
        )
        .map_err(|_| FitError::InvalidGeometry)?;
        let metric_frame = line.finalized.metric_bounds();
        let metrics = ScrollRectOf::try_new(
            Point::new(
                number(anchor.inline_origin + metric_frame.origin.x - origin.x)?,
                number(
                    add(anchor.baseline, -feedback.baseline)? + metric_frame.origin.y - origin.y,
                )?,
            ),
            Size::new(
                number(metric_frame.size.width)?,
                number(metric_frame.size.height)?,
            ),
        )
        .map_err(|_| FitError::InvalidGeometry)?;
        line_bounds.push(LineBounds {
            allocation,
            metrics,
        });
        report.first_baseline.get_or_insert(number(local_baseline)?);
        report.last_baseline = Some(number(local_baseline)?);
        report.block_extent = report.block_extent.max(number(line.flow.next)?);
        report.inline_extent = report.inline_extent.max(number(
            line.inline_start
                + contribution.advance()
                + line.finalized.selected_line().request().context.indent
                + line
                    .bands
                    .combined()
                    .intrinsic_end_occupation
                    .unwrap_or(0.0),
        )?);
        report.line_bands.push(LogicalLineBandOf {
            inline_start: number(line.inline_start)?,
            inline_end: number(line.inline_start + line.available)?,
            block_start: number(line.flow.start)?,
            block_end: number(line.flow.end)?,
        });
        report
            .post_line_clear_intents
            .push(PostLineClearIntent::None);
        for fragment in line.finalized.fragments() {
            let view = fragment.view();
            let owner = fragment_owner(binding, associations, view.owner());
            let rect = view
                .bounds()
                .map(|rect| {
                    let x = add(anchor.inline_origin, rect.origin.x)? - origin.x.to_f64();
                    let y = add(anchor.baseline, rect.origin.y - feedback.baseline)?
                        - origin.y.to_f64();
                    ScrollRectOf::try_new(
                        Point::new(number(x)?, number(y)?),
                        Size::new(number(rect.size.width)?, number(rect.size.height)?),
                    )
                    .map_err(|_| FitError::InvalidGeometry)
                })
                .transpose()?;
            fragments.push((
                owner,
                InlineFragmentOutputOf::new(
                    fragment,
                    rect,
                    Point::new(
                        number(anchor.inline_origin - origin.x.to_f64())?,
                        number(anchor.baseline - origin.y.to_f64())?,
                    ),
                    line_index,
                ),
            ));
        }
        for slot in line.finalized.control_slots() {
            let binding = binding
                .controls
                .iter()
                .find(|row| row.occurrence == slot.control())
                .expect("finalized control retains its admitted occurrence");
            let control = controls
                .iter()
                .find(|row| row.node == binding.node)
                .expect("capture admitted every actual control node");
            let (left, right) = slot.inline_bounds();
            let inline_extent = number(finite(right - left)?)?;
            let physical = Point::new(
                number(add(anchor.inline_origin, left)? - origin.x)?,
                number(add(anchor.baseline, slot.slot().y - feedback.baseline)? - origin.y)?,
            );
            let logical = context.axes.logical_point(
                physical,
                Size::new(inline_extent, S::ZERO),
                context.containing_size,
            );
            report.controls.push(InlineControlSourceOf {
                kind: control.kind,
                source_index: control.source_index,
                inline_start: logical.inline - context.inline_start_inset(),
                inline_extent,
                endpoint: control.endpoint,
                block_start: logical.block - context.cursor_block,
                line_index,
                visual_index: None,
            });
        }
        let mut atomic_placements = Vec::new();
        for slot in line.finalized.atomic_slots() {
            let binding = binding
                .atomics
                .iter()
                .find(|row| row.occurrence == slot.occurrence)
                .expect("Text returned a retained admitted occurrence");
            let AtomicLineAllocation::InFlow { box_geometry, .. } = slot.allocation else {
                static_positions.push((
                    binding.node,
                    Point::new(
                        number(add(anchor.inline_origin, slot.origin.x)? - origin.x)?,
                        number(anchor.baseline - feedback.baseline - origin.y)?,
                    ),
                ));
                outflows.push((line_index, slot.occurrence, binding.node));
                continue;
            };
            let atomic = atomics
                .iter()
                .find(|row| row.node == binding.node)
                .expect("actual atomic measurement remains retained");
            let slot_y = add(anchor.baseline, slot.origin.y - feedback.baseline)?;
            let border = translate(
                box_geometry.border,
                add(anchor.inline_origin, slot.origin.x)?,
                slot_y,
            )?;
            let content = translate(
                box_geometry.content,
                add(anchor.inline_origin, slot.origin.x)?,
                slot_y,
            )?;
            let baseline = add(slot_y, box_geometry.baseline)?;
            atomic_origins.push((atomic.node, Point::new(border.origin.x, border.origin.y)));
            let size = Size::new(number(border.size.width)?, number(border.size.height)?);
            let physical = Point::new(
                number(border.origin.x - origin.x.to_f64())?,
                number(border.origin.y - origin.y.to_f64())?,
            );
            let logical = context
                .axes
                .logical_point(physical, size, context.containing_size);
            let mut item = atomic.item;
            let (raw_x, raw_y) = raw_box(atomic, anchor, slot.origin, feedback.baseline)?;
            item.border = canonical_edges(raw_x, raw_y, atomic.item.size, atomic.item.border)?;
            item.padding = canonical_edges(raw_x, raw_y, atomic.item.size, atomic.item.padding)?;
            item.first_baseline = item
                .first_baseline
                .map(|_| number(baseline - border.origin.y))
                .transpose()?;
            item.size = size;
            // Content overflow extent and content-box allocation are distinct.
            // Preserve the actual child's overflow extent; never replace it by
            // the smaller content-box rectangle supplied to Text.
            item.content_size = Size::new(
                number(
                    (round(add(raw_x, atomic.item.content_size.width.to_f64())?)? - round(raw_x)?)
                        .max(0.0),
                )?,
                number(
                    (round(add(raw_y, atomic.item.content_size.height.to_f64())?)? - round(raw_y)?)
                        .max(0.0),
                )?,
            );
            report.atomics.push(AtomicInlineSourceOf {
                item,
                inline_start: logical.inline - context.inline_start_inset(),
                block_start: logical.block - context.cursor_block,
                line_index,
                visual_index: None,
            });
            atomic_placements.push(AtomicPlacement {
                occurrence: slot.occurrence,
                border,
                content,
                baseline,
            });
        }
        bands.push(line.bands);
        flows.push(line.flow);
        amounts.push(line.amounts);
        encounters.push(line.encounters);
        zones.push(line.zone_basis);
        phrases.push(line.phrase_basis);
        placements.push(LinePlacement {
            line: line.finalized,
            anchor,
            orientation: LineOrientation::Horizontal,
            atomics: atomic_placements,
        });
    }
    let indent_end = report.indent_state;
    let hyphenation_end = report.hyphenation_state;
    let line_box_end = report.line_box_context;
    Ok(Output {
        formatted,
        box_uses,
        report,
        line_bounds,
        fragments,
        atomic_origins,
        static_positions,
        paragraph: PendingParagraph {
            container: binding.container,
            nodes: binding.nodes.clone(),
            prepared: binding.prepared.clone(),
            lines: placements,
            boxes: box_assignments,
            bands,
            flows,
            amounts,
            encounters,
            zones,
            phrases,
            outflows,
            indent: context.indent.widened(),
            indent_start: context.indent_state,
            line_box_start: context.line_box_context,
            line_box_end,
            indent_end,
            hyphenation_start: context.hyphenation_state,
            hyphenation_end,
            tail: context.tail,
        },
    })
}

fn fragment_owner<Node: Copy + Eq>(
    binding: &TextParagraphBinding<Node>,
    associations: &super::capture::ScopeAssociations<Node>,
    owner: surgeist_text::FinalizedFragmentOwner,
) -> crate::InlineFragmentOwner<Node> {
    let node = match owner {
        surgeist_text::FinalizedFragmentOwner::Origin(origin) => {
            binding
                .origins
                .iter()
                .find(|row| row.origin == origin)
                .expect("capture admitted original occurrence")
                .node
        }
        surgeist_text::FinalizedFragmentOwner::Control(control) => {
            binding
                .controls
                .iter()
                .find(|row| row.occurrence == control)
                .expect("capture admitted explicit control")
                .node
        }
        surgeist_text::FinalizedFragmentOwner::Alternative(alternative) => {
            match alternative.view().definition() {
                surgeist_text::PreparedAlternativeDefinition::Source { origin, .. } => {
                    binding
                        .origins
                        .iter()
                        .find(|row| row.origin == origin)
                        .expect("capture admitted actual replacement source")
                        .node
                }
                surgeist_text::PreparedAlternativeDefinition::Automatic { cut } => {
                    let origin = cut
                        .view()
                        .preceding_source_pieces()
                        .last()
                        .expect("automatic cut owns original EGC pieces")
                        .origin();
                    binding
                        .origins
                        .iter()
                        .find(|row| row.origin == origin)
                        .expect("capture admitted automatic cut owner")
                        .node
                }
                surgeist_text::PreparedAlternativeDefinition::Control { control, .. } => {
                    binding
                        .controls
                        .iter()
                        .find(|row| row.occurrence == control)
                        .expect("capture admitted actual replacement control")
                        .node
                }
            }
        }
        surgeist_text::FinalizedFragmentOwner::VirtualSeparator(occurrence) => {
            let scope = occurrence.view().scope();
            return crate::InlineFragmentOwner::Scope(
                associations
                    .binding(&scope)
                    .expect("capture resolved the exact contributing scope"),
            );
        }
    };
    crate::InlineFragmentOwner::Node(node)
}

fn raw_box<Node, S: LayoutScalar>(
    atomic: &Atomic<Node, S>,
    anchor: LineAnchor,
    slot: surgeist_text::Point,
    line_baseline: f64,
) -> Result<(f64, f64), FitError> {
    let item = atomic.item;
    let baseline = item
        .first_baseline
        .unwrap_or(item.size.height + item.margin.bottom)
        .to_f64();
    Ok((
        add(
            add(anchor.inline_origin, slot.x)?,
            add(item.margin.left.to_f64(), atomic.relative_offset.x.to_f64())?,
        )?,
        add(
            add(anchor.baseline, slot.y - line_baseline)?,
            add(-baseline, atomic.relative_offset.y.to_f64())?,
        )?,
    ))
}

fn alignment(value: InlineControlAlignment) -> surgeist_text::LineBoxAlignment {
    match value {
        InlineControlAlignment::Baseline => surgeist_text::LineBoxAlignment::Baseline,
        InlineControlAlignment::Top => surgeist_text::LineBoxAlignment::Top,
        InlineControlAlignment::Bottom => surgeist_text::LineBoxAlignment::Bottom,
    }
}

fn canonical_edges<S: LayoutScalar>(
    x: f64,
    y: f64,
    size: Size<S>,
    edges: crate::Edges<S>,
) -> Result<crate::Edges<S>, FitError> {
    let right = add(x, size.width.to_f64())?;
    let bottom = add(y, size.height.to_f64())?;
    Ok(crate::Edges::new(
        number(round(add(y, edges.top.to_f64())?)? - round(y)?)?,
        number(round(right)? - round(add(right, -edges.right.to_f64())?)?)?,
        number(round(bottom)? - round(add(bottom, -edges.bottom.to_f64())?)?)?,
        number(round(add(x, edges.left.to_f64())?)? - round(x)?)?,
    ))
}

fn translate(rect: Rect, x: f64, y: f64) -> Result<Rect, FitError> {
    Ok(Rect::new(
        add(rect.origin.x, x)?,
        add(rect.origin.y, y)?,
        rect.size.width,
        rect.size.height,
    ))
}
pub(crate) fn number<S: LayoutScalar>(value: f64) -> Result<S, FitError> {
    finite(value)?;
    let number = S::from_f64(value);
    if !number.is_finite() {
        return Err(FitError::Numeric(value, LayoutTextNumericError::OutOfRange));
    }
    if value != 0.0 && number == S::ZERO {
        return Err(FitError::Numeric(
            value,
            LayoutTextNumericError::NonZeroLost,
        ));
    }
    Ok(number)
}
pub(super) fn add(base: f64, delta: f64) -> Result<f64, FitError> {
    let result = finite(base + delta)?;
    if delta != 0.0 && result == base {
        return Err(FitError::Numeric(
            delta,
            LayoutTextNumericError::NonZeroLost,
        ));
    }
    Ok(result)
}
fn finite(value: f64) -> Result<f64, FitError> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(FitError::NonFinite(value))
    }
}
fn round(value: f64) -> Result<f64, FitError> {
    finite(value)?;
    finite((value + 0.5).floor())
}

impl<Node: Copy + Eq> WrappingProjection<Node> {
    fn cut(
        &self,
        boundary: &surgeist_text::PreparedBoundary,
        incidence: Option<surgeist_text::PreparedBoundaryIncidence>,
    ) -> Result<super::boxes::BoxFrontier, FitError> {
        use surgeist_text::CapturedScopeIncidence as R;
        let prepared = boundary.prepared_text();
        let mut relations = Vec::new();
        let mut native_gaps = Vec::new();
        let mut unavailable_native = false;
        for (id, row, projected) in &self.boxes {
            let captured = row.capture.as_ref().filter(|scope| {
                prepared.captured_paragraph().is_some_and(|paragraph| {
                    paragraph.domain() == scope.domain()
                        && paragraph
                            .enter_position()
                            .same_capture(&scope.enter_position())
                })
            });
            let relation = if let (Some(incidence), Some(scope)) = (&incidence, captured) {
                Some(prepared.incidence_scope_relation(incidence, scope)?)
            } else {
                match projected {
                    Ok(ScopeIncidence::Outside) => None,
                    Ok(ScopeIncidence::Native { start, end }) => {
                        let entered = start.consumed(&prepared, boundary)?;
                        let exited = end.consumed(&prepared, boundary)?;
                        Some(if exited {
                            R::After
                        } else if entered {
                            R::Inside
                        } else {
                            R::Before
                        })
                    }
                    Ok(ScopeIncidence::Captured(_)) | Err(_) => {
                        if row.metric_contribution.is_some() {
                            return Err(crate::WrappingScopeError::UnsupportedIncidence.into());
                        }
                        None
                    }
                }
            };
            if let Some(relation) = relation {
                if matches!(relation, R::AtBefore | R::AtAfter) {
                    if let Some((start, end)) = row.pair {
                        let node = if relation == R::AtBefore { start } else { end };
                        if let Some((_, occurrence)) = self
                            .physical_controls
                            .iter()
                            .find(|(actual, _)| *actual == node)
                            && !prepared.control_is_boundary_exact(occurrence)?
                        {
                            return Err(crate::WrappingScopeError::UnsupportedIncidence.into());
                        }
                        let position = self
                            .physical_positions
                            .iter()
                            .find(|actual| actual.node == node)
                            .ok_or(crate::WrappingScopeError::UnsupportedIncidence)?;
                        let (before, after) = position
                            .gaps
                            .as_ref()
                            .map_err(|error| FitError::Wrapping(*error))?;
                        native_gaps.push(if relation == R::AtBefore {
                            *before
                        } else {
                            *after
                        });
                    } else {
                        unavailable_native = true;
                    }
                }
                relations.push((*id, relation));
            }
        }
        if let Some(incidence) = &incidence {
            for scope in &self.unlisted {
                let relation = prepared.incidence_scope_relation(incidence, scope)?;
                if matches!(relation, R::AtBefore | R::AtAfter) {
                    if let Some((_, (start, end))) = self
                        .canonical_pairs
                        .iter()
                        .find(|(actual, _)| actual == scope)
                    {
                        let node = if relation == R::AtBefore {
                            *start
                        } else {
                            *end
                        };
                        if let Some((_, occurrence)) = self
                            .physical_controls
                            .iter()
                            .find(|(actual, _)| *actual == node)
                            && !prepared.control_is_boundary_exact(occurrence)?
                        {
                            return Err(crate::WrappingScopeError::UnsupportedIncidence.into());
                        }
                        let position = self
                            .physical_positions
                            .iter()
                            .find(|actual| actual.node == node)
                            .ok_or(crate::WrappingScopeError::UnsupportedIncidence)?;
                        let (before, after) = position
                            .gaps
                            .as_ref()
                            .map_err(|error| FitError::Wrapping(*error))?;
                        native_gaps.push(if relation == R::AtBefore {
                            *before
                        } else {
                            *after
                        });
                    } else {
                        unavailable_native = true;
                    }
                }
            }
        }
        let pinned = boundary
            .view()
            .participant_offset()
            .checked_sub(1)
            .and_then(|at| prepared.participants().nth(at))
            .and_then(|participant| {
                let surgeist_text::PreparedParticipant::ComputedLineBoundary(boundary) =
                    participant
                else {
                    return None;
                };
                self.computed_cuts
                    .iter()
                    .find(|(actual, _)| *actual == boundary)
                    .map(|(_, cut)| *cut)
            });
        let native = if let Some(cut) = pinned {
            super::boxes::NativeBoxCut::Proven(cut)
        } else if unavailable_native {
            super::boxes::NativeBoxCut::Unavailable
        } else if let Some(gap) = super::capture::ComputedNativeCut::enclosing(&native_gaps) {
            super::boxes::NativeBoxCut::Proven(gap)
        } else {
            super::boxes::NativeBoxCut::Prefix
        };
        Ok(super::boxes::BoxFrontier::boundary(
            boundary.clone(),
            incidence,
            native,
            relations,
        ))
    }

    // A local box that really exits before the paragraph must be discharged
    // by an issued outgoing incidence. A retained force pin cannot stand in for
    // that completion, and a scope continuing into another paragraph is exempt.
    fn closes_paragraph(&self, outgoing: &super::boxes::BoxFrontier) -> Result<bool, FitError> {
        use surgeist_text::CapturedScopeIncidence as R;
        let prepared = outgoing.boundary.prepared_text();
        let Some(paragraph) = prepared.captured_paragraph() else {
            return Ok(true);
        };
        let paragraph_exit = paragraph.exit_position();
        for scope in self
            .boxes
            .iter()
            .filter_map(|(_, row, _)| row.capture.as_ref())
            .chain(&self.unlisted)
        {
            let scope_exit = scope.exit_position();
            if scope.domain() != paragraph.domain()
                || !scope_exit.same_capture(&paragraph_exit)
                || scope_exit.partial_cmp(&paragraph_exit) != Some(std::cmp::Ordering::Less)
            {
                continue;
            }
            if !matches!(outgoing.scope_relation(scope)?, Some(R::AtAfter | R::After)) {
                return Ok(false);
            }
        }
        Ok(true)
    }

    fn cuts(
        &self,
        boundary: &surgeist_text::PreparedBoundary,
    ) -> Result<Vec<super::boxes::BoxFrontier>, FitError> {
        let prepared = boundary.prepared_text();
        if prepared.captured_paragraph().is_none() {
            return Ok(vec![self.cut(boundary, None)?]);
        }
        let mut incidences = prepared.boundary_incidences(boundary)?;
        let forced = prepared.opportunity_views().any(|offer| {
            offer.boundary() == *boundary
                && offer.kind() == surgeist_text::PreparedBreakKind::Forced
        });
        if !forced {
            use surgeist_text::CapturedScopeIncidence as R;
            // An original pin/content split that prevents a whole exterior
            // placement stays authoritative. Only the freely realizable
            // exterior family needs this generated-subtree completion.
            let mut excluded = vec![false; incidences.len()];
            for (id, row, _) in &self.boxes {
                let Some(scope) = row.capture.as_ref().filter(|scope| {
                    prepared.captured_paragraph().is_some_and(|paragraph| {
                        paragraph.domain() == scope.domain()
                            && paragraph
                                .enter_position()
                                .same_capture(&scope.enter_position())
                    })
                }) else {
                    continue;
                };
                let relations = incidences
                    .iter()
                    .map(|incidence| prepared.incidence_scope_relation(incidence, scope))
                    .collect::<Result<Vec<_>, _>>()?;
                if !relations.contains(&R::Inside)
                    || !relations.contains(&R::AtBefore)
                    || !relations.contains(&R::AtAfter)
                {
                    continue;
                }
                let generated_text = prepared.virtual_separators().any(|separator| {
                    let contributor = separator.view().scope().captured_scope();
                    std::iter::successors(Some(contributor), |scope| scope.parent())
                        .any(|owner| owner == *scope)
                });
                if generated_text || !self.context.whole_subtree(*id)? {
                    continue;
                }
                for (excluded, relation) in excluded.iter_mut().zip(relations) {
                    *excluded |= relation == R::Inside;
                }
            }
            incidences = incidences
                .into_iter()
                .zip(excluded)
                .filter_map(|(incidence, excluded)| (!excluded).then_some(incidence))
                .collect();
        }
        incidences
            .into_iter()
            .map(|incidence| self.cut(boundary, Some(incidence)))
            .collect()
    }
}

impl<Node: Copy + Eq> super::boxes::BoxResolver for WrappingProjection<Node> {
    fn choices(
        &self,
        choice: super::boxes::BoxChoice<'_>,
    ) -> Result<Vec<super::boxes::BoxAssignment>, FitError> {
        let start = choice.start;
        let end = choice.end;
        let controls = choice.controls;
        let ending = choice.ending;
        use crate::InlineBoxMetricContribution as M;
        use surgeist_text::CapturedScopeIncidence as R;
        // The receiver's Text progress boundary keeps its own spelling. The
        // original frontier stays with its issuer across administrative runs.
        let entry = choice.incoming.or(self
            .incoming
            .as_ref()
            .map(super::boxes::BoxEntry::ConsumedTo));
        let incoming = entry.map(super::boxes::BoxEntry::frontier);
        let starts = if let Some(incoming) = incoming {
            if matches!(entry, Some(super::boxes::BoxEntry::UnconsumedFrom(_)))
                || incoming.boundary == *start
                || incoming.boundary.prepared_text() != start.prepared_text()
            {
                vec![incoming.clone()]
            } else {
                self.cuts(start)?
                    .into_iter()
                    .filter(|cut| {
                        incoming.relations().iter().all(|(id, prior)| {
                            !matches!(prior, R::AtAfter | R::After)
                                || !matches!(
                                    cut.relation(*id),
                                    Some(R::Before | R::AtBefore | R::Inside)
                                )
                        })
                    })
                    .collect()
            }
        } else {
            self.cuts(start)?.into_iter().take(1).collect()
        };
        let boundary_ends = self.cuts(end)?;
        if starts.is_empty() {
            return Err(crate::WrappingScopeError::UnsupportedIncidence.into());
        }
        let mut choices = Vec::new();
        for incoming in starts {
            let mut ends = boundary_ends.clone();
            if ending == surgeist_text::LineEnd::Paragraph {
                // Keep an existing exterior incidence and its physical proof
                // when it already discharges all local boxes.
                let mut completed = Vec::new();
                for outgoing in ends {
                    if self.closes_paragraph(&outgoing)? {
                        completed.push(outgoing);
                    }
                }
                ends = completed.into_iter().last().into_iter().collect();
                if ends.is_empty()
                    && start == end
                    && let Some(paragraph) = end.prepared_text().captured_paragraph()
                {
                    let mut relations = Vec::new();
                    for (id, row, _) in &self.boxes {
                        let relation = row
                            .capture
                            .as_ref()
                            .map(|scope| super::boxes::horizon_relation(&paragraph, scope))
                            .transpose()?
                            .flatten()
                            .or_else(|| incoming.relation(*id));
                        if let Some(relation) = relation {
                            relations.push((*id, relation));
                        }
                    }
                    if let Some(completion) =
                        super::boxes::BoxFrontier::complete(&incoming, &choice, relations)?
                        && self.closes_paragraph(&completion)?
                    {
                        ends.push(completion);
                    }
                }
            }
            if ends.is_empty() {
                return Err(crate::WrappingScopeError::UnsupportedIncidence.into());
            }
            for outgoing in &ends {
                let mut contributions = Vec::new();
                let mut valid = true;
                for (id, row, _) in &self.boxes {
                    let before = incoming.relation(*id);
                    let after = outgoing.relation(*id);
                    let finished = matches!(before, Some(R::AtAfter | R::After));
                    let entered = matches!(after, Some(R::Inside | R::AtAfter | R::After));
                    if finished && matches!(after, Some(R::Before | R::AtBefore | R::Inside)) {
                        valid = false;
                        break;
                    }
                    if finished || !entered {
                        continue;
                    }
                    // Actual selected controls can already carry this box's
                    // contribution. Numeric equality never establishes ownership.
                    let owned: Vec<_> = row
                        .pair
                        .into_iter()
                        .flat_map(|(start, end)| [start, end])
                        .filter_map(|node| {
                            let (_, occurrence) = self
                                .physical_controls
                                .iter()
                                .find(|(actual, _)| *actual == node)?;
                            controls
                                .iter()
                                .find(|allocation| allocation.occurrence == *occurrence)
                                .and_then(|allocation| allocation.strut)
                        })
                        .collect();
                    let allocation = match row.metric_contribution {
                        Some(M::NoContribution) => {
                            if !owned.is_empty() {
                                return Err(crate::WrappingScopeError::ConflictingBoxMetrics.into());
                            }
                            None
                        }
                        Some(M::Strut(strut)) => {
                            if owned.iter().any(|actual| {
                                actual.baseline != strut.metrics.baseline
                                    || actual.line_extent != strut.metrics.line_extent
                                    || actual.alignment != strut.alignment
                            }) {
                                return Err(crate::WrappingScopeError::ConflictingBoxMetrics.into());
                            }
                            owned.is_empty().then_some(strut)
                        }
                        None => {
                            if owned.is_empty() {
                                return Err(crate::WrappingScopeError::MissingBoxMetrics.into());
                            }
                            None
                        }
                    };
                    if let Some(original) = allocation {
                        contributions.push(super::boxes::BoxContribution {
                            scope: *id,
                            original,
                        });
                    }
                }
                if !valid {
                    continue;
                }
                for scope in &self.unlisted {
                    let end_relation = outgoing
                        .scope_relation(scope)?
                        .ok_or(crate::WrappingScopeError::UnsupportedIncidence)?;
                    let start_relation = incoming
                        .scope_relation(scope)?
                        .ok_or(crate::WrappingScopeError::UnsupportedIncidence)?;
                    if !matches!(start_relation, R::AtAfter | R::After)
                        && matches!(end_relation, R::Inside | R::AtAfter | R::After)
                    {
                        let same_box_control = controls.iter().any(|allocation| {
                            allocation.strut.is_some()
                                && allocation
                                    .occurrence
                                    .captured_control()
                                    .is_some_and(|control| {
                                        scope.entry_control().as_ref() == Some(&control)
                                            || scope.exit_control().as_ref() == Some(&control)
                                    })
                        });
                        if !same_box_control {
                            return Err(crate::WrappingScopeError::MissingBoxMetrics.into());
                        }
                    }
                }
                if valid {
                    choices.push(super::boxes::BoxAssignment {
                        incoming: incoming.clone(),
                        outgoing: outgoing.clone(),
                        contributions,
                    });
                }
            }
        }
        if choices.is_empty() {
            return Err(crate::WrappingScopeError::InconsistentIncidence.into());
        }
        Ok(choices)
    }

    fn classify(
        &self,
        assignment: &super::boxes::BoxAssignment,
    ) -> Result<crate::inline_wrapping::AvoidClass, FitError> {
        use surgeist_text::{CapturedScopeIncidence as R, SourceBoxSide};
        let cut = &assignment.outgoing;
        let Some(incidence) = cut.incidence() else {
            return self.classify_boundary(&cut.boundary.prepared_text(), &cut.boundary);
        };
        let issuer = cut.boundary.prepared_text();
        let mut class = self.context.classify::<FitError>(self.flow, |row| {
            if let Some(scope) = &row.capture {
                return Ok(issuer.incidence_scope_relation(incidence, scope)? == R::Inside);
            }
            let id = self
                .boxes
                .iter()
                .find(|(_, actual, _)| actual == row)
                .map(|(id, _, _)| *id)
                .ok_or(crate::WrappingScopeError::InconsistentIncidence)?;
            Ok(cut.relation(id) == Some(R::Inside))
        })?;
        for (id, edge) in self
            .context
            .edge_locations(self.flow)
            .filter(|(_, edge)| edge.effect.avoided)
        {
            let scope = match &edge.box_ {
                crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::Scope(scope)) => Some(scope),
                crate::GeneratedBoxRef::Inline(crate::InlineBoxRef::NativePair { start, end }) => {
                    self.boxes
                        .iter()
                        .find(|(_, row, _)| row.pair == Some((*start, *end)))
                        .and_then(|(_, row, _)| row.capture.as_ref())
                }
                crate::GeneratedBoxRef::Node(_) => None,
            };
            let matches = if let Some(scope) = scope {
                let relation = issuer.incidence_scope_relation(incidence, scope)?;
                relation
                    == if edge.side == SourceBoxSide::Before {
                        R::AtBefore
                    } else {
                        R::AtAfter
                    }
            } else {
                let (_, projected) = self
                    .edges
                    .iter()
                    .find(|(actual, _)| *actual == id)
                    .ok_or(crate::WrappingScopeError::InconsistentIncidence)?;
                match projected
                    .as_ref()
                    .map_err(|error| FitError::Wrapping(*error))?
                {
                    EdgeIncidence::Outside => false,
                    EdgeIncidence::Offset(offset) => {
                        cut.boundary.view().participant_offset() == *offset
                    }
                    EdgeIncidence::Control { control, after } => {
                        if !issuer.control_is_boundary_exact(control)? {
                            return Err(crate::WrappingScopeError::UnsupportedIncidence.into());
                        }
                        let at = issuer.participants().position(|participant| matches!(participant, surgeist_text::PreparedParticipant::Control(actual) if actual == *control)).ok_or(crate::WrappingScopeError::UnsupportedIncidence)?;
                        cut.boundary.view().participant_offset() == at + usize::from(*after)
                    }
                }
            };
            if matches {
                class.add_edge(id);
            }
        }
        Ok(class)
    }
}
