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
    AtomicInlineBoxParticipant, AtomicInlineSourceOf, InlineBaselineReportOf,
    InlineControlAlignment, InlineSourcePositionOf, LogicalLineBandOf, MixedInlineRunReportOf,
    PhysicalBaselineReport, PostLineClearIntent,
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
    segment: surgeist_text::PreparedSourceSegment,
    scope_alignment_active: bool,
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
        run: &super::AdmittedRun<'_, Node>,
        associations: &super::capture::ScopeAssociations<Node>,
    ) -> Option<Self> {
        let binding = run.binding;
        use super::capture::CapturedScopeRelation;
        use crate::inline_wrapping::{ScopeEndpoint, WrappingScopeError};
        let native_projection = |pair: (Node, Node)| -> Result<
            crate::inline_wrapping::NativePairProjection<Node>,
            WrappingScopeError,
        > {
            if run.nodes().is_empty() {
                let owner = graph
                    .index(binding.container)
                    .ok_or(WrappingScopeError::NodeNotReachable)?;
                let endpoint = |node| -> Result<ScopeEndpoint<Node>, WrappingScopeError> {
                    let rank = graph.children[owner]
                        .iter()
                        .position(|&index| graph.nodes[index] == node)
                        .ok_or(WrappingScopeError::DifferentFlow)?;
                    Ok(if rank < run.native.start {
                        ScopeEndpoint::BeforeRun
                    } else {
                        ScopeEndpoint::AfterRun
                    })
                };
                Ok(crate::inline_wrapping::NativePairProjection {
                    start: endpoint(pair.0)?,
                    end: endpoint(pair.1)?,
                })
            } else {
                context.native_pair_projection(graph, binding.container, run.nodes(), pair)
            }
        };
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
                    .and_then(&native_projection)
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
                let position = native_projection((node, node)).and_then(|pair| match pair.start {
                    ScopeEndpoint::BeforeRun => Ok((0, 0)),
                    ScopeEndpoint::AfterRun => Ok((run.nodes().len(), run.nodes().len())),
                    ScopeEndpoint::Local(node) => run
                        .nodes()
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
                let position = native_projection((node, node)).and_then(|pair| match pair.start {
                    ScopeEndpoint::BeforeRun => Ok((0, 0)),
                    ScopeEndpoint::AfterRun => Ok((run.nodes().len(), run.nodes().len())),
                    ScopeEndpoint::Local(node) => run
                        .nodes()
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
            native_projection((node, node)).and_then(|projected| match projected.start {
                ScopeEndpoint::BeforeRun | ScopeEndpoint::AfterRun => Ok(EdgeIncidence::Outside),
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
                    } else {native_projection((*node,*node)).and_then(|p|match p.start {
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
        let mut unlisted = Vec::new();
        if let Some(owner) = binding.prepared.captured_owner() {
            for member in owner.members() {
                for intersection in member.scope_intersections() {
                    let scope = intersection.scope();
                    if scope.role() == surgeist_text::ScopeRole::Inline
                        && !context
                            .box_rows(binding.container)
                            .any(|(_, row)| row.capture.as_ref() == Some(&scope))
                        && !unlisted.contains(&scope)
                    {
                        unlisted.push(scope);
                    }
                }
            }
        }
        (!rows.is_empty()
            || !edges.is_empty()
            || run.nodes().is_empty() && !unlisted.is_empty()
            || boxes
                .iter()
                .any(|(_, row, _)| row.metric_contribution.is_some()))
        .then(|| Self {
            segment: run.segment.clone(),
            scope_alignment_active: false,
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
                        .and_then(|cut| cut.relative_to(run.inventory.clone()))
                        .map(|cut| (boundary.clone(), cut))
                })
                .collect(),
            unlisted,
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

    fn align_scope_metrics<S: LayoutScalar>(
        &mut self,
        binding: &TextParagraphBinding<Node>,
        associations: &super::capture::ScopeAssociations<Node>,
        resources: &super::scope_alignment::ScopeAlignmentResources<S>,
    ) -> Result<(), FitError> {
        use super::scope_alignment::ScopeAlignmentFacts;
        if binding.prepared.scope_alignment_retention().is_none() {
            return Ok(());
        }
        let Some(paragraph) = binding.prepared.captured_owner() else {
            return Ok(());
        };
        let limits = paragraph.domain().scope_alignment_limits().ok_or_else(|| {
            super::scope_alignment::refusal(surgeist_text::ScopeAlignmentError::LimitsRequired)
        })?;
        let facts = resources.snapshot();
        let mut active = false;
        for row in facts.rows() {
            ScopeAlignmentFacts::visit(resources, limits, 1)?;
            if row.scope.domain() == paragraph.domain() {
                active = true;
                break;
            }
        }
        if !active {
            return Ok(());
        }
        self.scope_alignment_active = true;
        for (_, row, _) in &mut self.boxes {
            ScopeAlignmentFacts::visit(resources, limits, 1)?;
            let mut own = None;
            for fact in facts.rows() {
                ScopeAlignmentFacts::visit(resources, limits, 1)?;
                let matches = if row.capture.as_ref() == Some(&fact.scope) {
                    true
                } else {
                    ScopeAlignmentFacts::visit(
                        resources,
                        limits,
                        associations.captured_binding_work(&fact.scope),
                    )?;
                    matches!(associations.captured_binding(&fact.scope),
                        Some(super::TextScopeAssociation::Inline { start, end }) if row.pair == Some((start, end)))
                };
                if matches {
                    own = Some(fact);
                    break;
                }
            }
            if let Some(own) = own
                && let Some(crate::InlineBoxMetricContribution::Strut(strut)) =
                    &mut row.metric_contribution
            {
                strut.alignment =
                    super::scope_alignment::intrinsic_alignment(own, strut.alignment)?;
            }
        }
        Ok(())
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
                    Ok(prepared.boundary_inside_captured_scope(&self.segment, end, scope)?)
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
    pub scope_alignments: super::scope_alignment::ScopeAlignmentResources<S>,
    pub orientation: LineOrientation,
    pub inline_baseline_context: crate::InlineBaselineContext,
    pub groups: crate::group_alignment::Snapshot,
    pub wrapping: Option<WrappingProjection<Node>>,
    pub formatted: super::formatted::FormattedTailSnapshot,
    pub line_box_context: crate::line_box::LineBoxContext,
    pub frame: Frame<S>,
    pub containing_size: Size<S>,
    pub content_insets: crate::Edges<S>,
    pub cursor_block: S,
    pub axes: FlowAxes,
    pub available: AvailableOf<S>,
    pub alignment: TextAlign,
    pub wrap_style: crate::TextWrapStyle,
    pub last_alignment: TextAlignLast,
    pub justification: TextJustification,
    pub indent: crate::text_indent::ResolvedTextIndent<S>,
    pub indent_state: crate::text_indent::IndentLineState,
    pub hyphenation_state: crate::hyphenation::LineHyphenationState,
    pub hyphenate_limit_zone: crate::LengthPercentageOf<S>,
    pub tail: fit::TailContext,
}

impl<Node, S: LayoutScalar> Context<Node, S> {
    fn frame_size(&self) -> Size<S> {
        match self.frame {
            Frame::Canonical { size, .. } => size,
            Frame::Measurement => self.containing_size,
        }
    }

    fn line_frame(&self, origin: Point<f64>) -> LineFrame {
        let size = self.frame_size();
        LineFrame {
            axes: self.axes,
            orientation: self.orientation,
            origin,
            size: Size::new(size.width.to_f64(), size.height.to_f64()),
            insets: self.content_insets.map(LayoutScalar::to_f64),
            cursor_block: self.cursor_block.to_f64(),
        }
    }
}

/// One owning mapping for actual Text consumption, independent of box-only flow.
/// Other preparation pairings retain their expected basis so Text preserves its
/// existing context validation and diagnostic phase.
pub(crate) fn text_orientation(
    mode: crate::WritingMode,
    prepared: &surgeist_text::PreparedText,
) -> Result<LineOrientation, FitError> {
    match mode {
        crate::WritingMode::HorizontalTb => Ok(LineOrientation::Horizontal),
        crate::WritingMode::SidewaysRl => Ok(LineOrientation::SidewaysClockwise),
        crate::WritingMode::VerticalRl | crate::WritingMode::VerticalLr
            if prepared.orientation() == LineOrientation::SidewaysClockwise
                && matches!(
                    prepared.analyzed_source().typography(),
                    surgeist_text::AnalysisTypography::HorizontalComposition
                ) =>
        {
            Ok(LineOrientation::SidewaysClockwise)
        }
        crate::WritingMode::VerticalRl | crate::WritingMode::VerticalLr => {
            Ok(LineOrientation::Vertical)
        }
        crate::WritingMode::SidewaysLr => Err(FitError::TextWritingMode(mode)),
    }
}

/// One authority rule for admission and named output. Physical consumption
/// never asserts a name, even when the actual preparation uses one.
pub(crate) fn named_baseline(
    prepared: &surgeist_text::PreparedText,
    axes: FlowAxes,
    context: crate::InlineBaselineContext,
) -> bool {
    match (context, prepared.baseline_reference()) {
        (
            crate::InlineBaselineContext::Named(crate::BaselineName::Central),
            Some(surgeist_text::FontBaseline::Central),
        ) => {
            matches!(
                axes.writing_mode(),
                crate::WritingMode::VerticalRl | crate::WritingMode::VerticalLr
            ) && prepared.orientation() == LineOrientation::Vertical
                && matches!(
                    prepared.analyzed_source().typography(),
                    surgeist_text::AnalysisTypography::Upright { .. }
                        | surgeist_text::AnalysisTypography::Mixed
                )
        }
        (
            crate::InlineBaselineContext::Named(crate::BaselineName::Alphabetic),
            Some(surgeist_text::FontBaseline::Alphabetic),
        ) => {
            matches!(
                prepared.analyzed_source().typography(),
                surgeist_text::AnalysisTypography::HorizontalComposition
            ) && matches!(
                prepared.orientation(),
                LineOrientation::Horizontal | LineOrientation::SidewaysClockwise
            ) && text_orientation(axes.writing_mode(), prepared)
                .is_ok_and(|orientation| orientation == prepared.orientation())
        }
        _ => false,
    }
}

/// Adapter-local coordinates belong to the fitter; these anchors and rectangles
/// belong to the actual physical container. FlowAxes maps the available interval
/// once. Finalized local u already includes bidi and alignment, so it is never
/// reflected again. Block progression and cell over-to-under are separate maps.
#[derive(Clone, Copy)]
struct LineFrame {
    axes: FlowAxes,
    orientation: LineOrientation,
    origin: Point<f64>,
    size: Size<f64>,
    insets: crate::Edges<f64>,
    cursor_block: f64,
}

impl LineFrame {
    fn orientation(self) -> LineOrientation {
        self.orientation
    }

    fn vertical_cell(self) -> bool {
        self.orientation() != LineOrientation::Horizontal
    }

    fn decreasing_block(self) -> bool {
        self.axes.block_start() == crate::PhysicalSide::Right
    }

    fn inline_origin(self, start: f64, available: f64) -> Result<f64, FitError> {
        if self.vertical_cell() {
            let interval = self.axes.physical_point(
                crate::geometry::LogicalPointOf::new(
                    add(self.axes.logical_edges(self.insets).inline_start, start)?,
                    0.0,
                ),
                crate::geometry::LogicalSizeOf::new(available, 0.0),
                self.size,
            );
            round(add(self.origin.y, finite(interval.y)?)?)
        } else {
            let local_x = if self.axes.direction() == Direction::Rtl {
                self.size.width - self.insets.right - start - available
            } else {
                self.insets.left + start
            };
            round(add(self.origin.x, local_x)?)
        }
    }

    fn cross(self, block: f64, baseline: f64, height: f64) -> Result<f64, FitError> {
        if self.decreasing_block() {
            add(
                self.origin.x,
                finite(self.size.width - add(self.cursor_block, add(block, baseline)?)?)?,
            )
        } else if self.vertical_cell() {
            add(
                add(self.origin.x, self.cursor_block)?,
                add(block, finite(height - baseline)?)?,
            )
        } else {
            add(
                add(self.origin.y, self.cursor_block)?,
                add(block, baseline)?,
            )
        }
    }

    fn block(self, cross: f64, baseline: f64, height: f64) -> Result<f64, FitError> {
        if self.decreasing_block() {
            finite(self.size.width - (cross - self.origin.x) - self.cursor_block - baseline)
        } else if self.vertical_cell() {
            finite(cross - self.origin.x - self.cursor_block - (height - baseline))
        } else {
            finite(cross - self.origin.y - self.cursor_block - baseline)
        }
    }

    fn canonical_block(
        self,
        proposed: f64,
        minimum: Option<f64>,
        baseline: f64,
        height: f64,
    ) -> Result<f64, FitError> {
        let mut canonical = round(self.cross(proposed, baseline, height)?)?;
        if let Some(minimum) = minimum {
            let limit = self.cross(minimum, baseline, height)?;
            // Minimum logical progression follows the actual block-start side.
            // Snap the actual constrained anchor, not its negated displacement.
            canonical = if self.decreasing_block() {
                canonical.min(finite(limit.floor())?)
            } else {
                canonical.max(finite(limit.ceil())?)
            };
        }
        self.block(canonical, baseline, height)
    }

    fn anchor(
        self,
        start: f64,
        available: f64,
        block: f64,
        baseline: f64,
        height: f64,
    ) -> Result<LineAnchor, FitError> {
        Ok(LineAnchor {
            inline_origin: self.inline_origin(start, available)?,
            baseline: round(self.cross(block, baseline, height)?)?,
        })
    }

    fn cross_offset(self, cross: f64, offset: f64) -> Result<f64, FitError> {
        add(
            cross,
            if self.vertical_cell() {
                -offset
            } else {
                offset
            },
        )
    }

    fn cross_distance(self, from: f64, to: f64) -> Result<f64, FitError> {
        finite(if self.vertical_cell() {
            from - to
        } else {
            to - from
        })
    }

    fn aligned_strut(
        self,
        anchor: LineAnchor,
        feedback: surgeist_text::LineFeedback,
        metrics: surgeist_text::LineStrut,
        alignment: surgeist_text::LineBoxAlignment,
        scope_displacement: f64,
    ) -> Result<surgeist_text::LineStrut, FitError> {
        let over = if self.vertical_cell() {
            let offset = match alignment {
                surgeist_text::LineBoxAlignment::Baseline
                | surgeist_text::LineBoxAlignment::Shift(_) => -metrics.baseline,
                surgeist_text::LineBoxAlignment::Top => -feedback.baseline,
                surgeist_text::LineBoxAlignment::Bottom => {
                    -feedback.baseline + feedback.line_height - metrics.line_extent
                }
            };
            self.cross_offset(anchor.baseline, offset)?
        } else {
            match alignment {
                surgeist_text::LineBoxAlignment::Baseline
                | surgeist_text::LineBoxAlignment::Shift(_) => anchor.baseline - metrics.baseline,
                surgeist_text::LineBoxAlignment::Top => anchor.baseline - feedback.baseline,
                surgeist_text::LineBoxAlignment::Bottom => {
                    anchor.baseline - feedback.baseline + feedback.line_height - metrics.line_extent
                }
            }
        };
        // Match the actual baseline first. Apply the allocated displacement in
        // physical coordinates before the existing final edge quantization.
        let over = if let surgeist_text::LineBoxAlignment::Shift(offset) = alignment {
            add(
                over,
                if self.vertical_cell() {
                    offset.get()
                } else {
                    -offset.get()
                },
            )?
        } else {
            over
        };
        let over = add(
            over,
            if self.vertical_cell() {
                scope_displacement
            } else {
                -scope_displacement
            },
        )?;
        self.strut(over, metrics.baseline, metrics.line_extent)
    }

    fn strut(
        self,
        over: f64,
        baseline: f64,
        extent: f64,
    ) -> Result<surgeist_text::LineStrut, FitError> {
        let snapped_over = round(over)?;
        Ok(surgeist_text::LineStrut {
            baseline: self
                .cross_distance(snapped_over, round(self.cross_offset(over, baseline)?)?)?,
            line_extent: self
                .cross_distance(snapped_over, round(self.cross_offset(over, extent)?)?)?,
        })
    }

    fn point(
        self,
        anchor: LineAnchor,
        baseline: f64,
        inline: f64,
        cross: f64,
    ) -> Result<Point<f64>, FitError> {
        if self.vertical_cell() {
            Ok(Point::new(
                add(anchor.baseline, baseline - cross)?,
                add(anchor.inline_origin, inline)?,
            ))
        } else {
            Ok(Point::new(
                add(anchor.inline_origin, inline)?,
                add(anchor.baseline, cross - baseline)?,
            ))
        }
    }

    fn relative_point<S: LayoutScalar>(self, point: Point<f64>) -> Result<Point<S>, FitError> {
        Ok(Point::new(
            number(point.x - self.origin.x)?,
            number(point.y - self.origin.y)?,
        ))
    }

    fn qualify_scope_position<S: LayoutScalar>(
        self,
        point: Point<f64>,
        displacement: f64,
    ) -> Result<(), FitError> {
        if displacement == 0.0 {
            return Ok(());
        }
        let (matched, actual, origin) = if self.vertical_cell() {
            (add(point.x, -displacement)?, point.x, self.origin.x)
        } else {
            (add(point.y, displacement)?, point.y, self.origin.y)
        };
        qualify_shift_coordinate::<S>(add(matched, -origin)?, add(actual, -origin)?)
    }

    fn qualify_scope_frames<S: LayoutScalar>(
        self,
        anchor: LineAnchor,
        line: &FinalizedLine,
    ) -> Result<(), FitError> {
        for allocation in line.scope_allocations() {
            // This is the actually allocated native frame origin, at the same
            // accepted common baseline. It covers continuation lines without
            // a selected endpoint control as well as glyph-bearing frames.
            let cumulative = allocation.cumulative_displacement.get();
            let mut point = self.anchor_point(anchor);
            if self.vertical_cell() {
                point.x = add(point.x, cumulative)?;
            } else {
                point.y = add(point.y, -cumulative)?;
            }
            self.qualify_scope_position::<S>(point, allocation.displacement.get())?;
        }
        Ok(())
    }

    fn anchor_point(self, anchor: LineAnchor) -> Point<f64> {
        if self.vertical_cell() {
            Point::new(anchor.baseline, anchor.inline_origin)
        } else {
            Point::new(anchor.inline_origin, anchor.baseline)
        }
    }

    /// Absolute layout consumes the flow block-start edge, while native Text
    /// cells progress from line-over to line-under in both vertical modes.
    fn static_position<S: LayoutScalar>(
        self,
        anchor: LineAnchor,
        baseline: f64,
        height: f64,
        inline: f64,
        scope_displacement: f64,
    ) -> Result<Point<S>, FitError> {
        let cross = if self.vertical_cell() && !self.decreasing_block() {
            height
        } else {
            0.0
        };
        let matched = self.point(anchor, baseline, inline, cross)?;
        let actual = self.point(anchor, baseline, inline, add(cross, -scope_displacement)?)?;
        if scope_displacement != 0.0 {
            let (matched, actual, origin) = if self.vertical_cell() {
                (matched.x, actual.x, self.origin.x)
            } else {
                (matched.y, actual.y, self.origin.y)
            };
            qualify_shift_coordinate::<S>(matched - origin, actual - origin)?;
        }
        self.relative_point(actual)
    }

    fn physical_rect(
        self,
        anchor: LineAnchor,
        baseline: f64,
        rect: Rect,
    ) -> Result<Rect, FitError> {
        let point = self.point(anchor, baseline, rect.origin.x, rect.origin.y)?;
        let (origin, size) = if self.vertical_cell() {
            let end_x = add(point.x, -rect.size.height)?;
            let end_y = add(point.y, rect.size.width)?;
            (
                Point::new(point.x.min(end_x), point.y.min(end_y)),
                Size::new(rect.size.height.abs(), rect.size.width.abs()),
            )
        } else {
            (point, Size::new(rect.size.width, rect.size.height))
        };
        Ok(Rect::new(origin.x, origin.y, size.width, size.height))
    }

    fn rect<S: LayoutScalar>(
        self,
        anchor: LineAnchor,
        baseline: f64,
        rect: Rect,
    ) -> Result<ScrollRectOf<S>, FitError> {
        let physical = self.physical_rect(anchor, baseline, rect)?;
        ScrollRectOf::try_new(
            self.relative_point(Point::new(physical.origin.x, physical.origin.y))?,
            Size::new(number(physical.size.width)?, number(physical.size.height)?),
        )
        .map_err(|_| FitError::InvalidGeometry)
    }

    /// Inverse cell mapping of an unrotated physical child rectangle.
    fn local_rect(self, anchor: LineAnchor, baseline: f64, rect: Rect) -> Result<Rect, FitError> {
        if self.vertical_cell() {
            Ok(Rect::new(
                finite(rect.origin.y - anchor.inline_origin)?,
                finite(baseline - (add(rect.origin.x, rect.size.width)? - anchor.baseline))?,
                rect.size.height,
                rect.size.width,
            ))
        } else {
            Ok(Rect::new(
                finite(rect.origin.x - anchor.inline_origin)?,
                add(rect.origin.y - anchor.baseline, baseline)?,
                rect.size.width,
                rect.size.height,
            ))
        }
    }

    fn atomic_rect(
        self,
        anchor: LineAnchor,
        baseline: f64,
        slot: surgeist_text::Point,
        rect: Rect,
    ) -> Result<Rect, FitError> {
        if self.vertical_cell() {
            self.physical_rect(anchor, baseline, translate(rect, slot.x, slot.y)?)
        } else {
            translate(
                rect,
                add(anchor.inline_origin, slot.x)?,
                add(anchor.baseline, slot.y - baseline)?,
            )
        }
    }

    fn atomic_local_rect(
        self,
        anchor: LineAnchor,
        baseline: f64,
        slot: surgeist_text::Point,
        rect: Rect,
    ) -> Result<Rect, FitError> {
        if self.vertical_cell() {
            translate(self.local_rect(anchor, baseline, rect)?, -slot.x, -slot.y)
        } else {
            Ok(Rect::new(
                finite(rect.origin.x - anchor.inline_origin - slot.x)?,
                finite(rect.origin.y - add(anchor.baseline, slot.y - baseline)?)?,
                rect.size.width,
                rect.size.height,
            ))
        }
    }

    fn atomic_cross(
        self,
        anchor: LineAnchor,
        baseline: f64,
        slot: surgeist_text::Point,
        offset: f64,
    ) -> Result<f64, FitError> {
        if self.vertical_cell() {
            Ok(self
                .point(anchor, baseline, slot.x, add(slot.y, offset)?)?
                .x)
        } else {
            add(add(anchor.baseline, slot.y - baseline)?, offset)
        }
    }

    /// Numeric participants retain their flow-block-start reference. Text uses
    /// line-over, which is the opposite border edge in VerticalLr.
    fn atomic_baseline<S: LayoutScalar>(
        self,
        item: AtomicInlineBoxParticipant<S>,
    ) -> Result<f64, FitError> {
        if self.vertical_cell() {
            if let Some(baseline) = item.first_baseline {
                finite(if self.decreasing_block() {
                    baseline.to_f64()
                } else {
                    item.size.width.to_f64() - baseline.to_f64()
                })
            } else {
                // Physical fallback is the actual line-under margin edge.
                add(item.size.width.to_f64(), item.margin.left.to_f64())
            }
        } else {
            Ok(item
                .first_baseline
                .unwrap_or(item.size.height + item.margin.bottom)
                .to_f64())
        }
    }

    fn participant_baseline(self, border: Rect, cross: f64) -> Result<f64, FitError> {
        finite(if self.decreasing_block() {
            add(border.origin.x, border.size.width)? - cross
        } else if self.vertical_cell() {
            cross - border.origin.x
        } else {
            cross - border.origin.y
        })
    }

    /// Normal-flow allocation follows block progression, not Text cell direction.
    fn allocation_rect<S: LayoutScalar>(
        self,
        start: f64,
        width: f64,
        block: f64,
        height: f64,
    ) -> Result<ScrollRectOf<S>, FitError> {
        if self.vertical_cell() {
            // With B=0, the allocation's over edge is block-start in Vrl/Srl
            // and block-start+H in Vlr. Its cell extent then runs toward left.
            self.rect(
                LineAnchor {
                    inline_origin: self.inline_origin(start, width)?,
                    baseline: self.cross(block, 0.0, height)?,
                },
                0.0,
                Rect::new(0.0, 0.0, width, height),
            )
        } else {
            ScrollRectOf::try_new(
                Point::new(
                    number(self.inline_origin(start, width)? - self.origin.x)?,
                    number(add(self.cursor_block, block)?)?,
                ),
                Size::new(number(width)?, number(height)?),
            )
            .map_err(|_| FitError::InvalidGeometry)
        }
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
    pub nonformatted_controls: Vec<(Node, Point<S>)>,
}

/// One invocation's coupled line-band and source-event transitions. Each
/// callback borrows the same actual provisional session, never a public root.
pub(crate) struct LineEnvironment<Band, Clear, Replay, Accept, Reset, Checkpoint> {
    pub band: Band,
    pub reset: Reset,
    pub checkpoint_group: Checkpoint,
    pub clear: Clear,
    pub replay: Replay,
    pub accept: Accept,
}

pub(crate) fn layout<Node: Copy + Eq, S: LayoutScalar>(
    run: &super::AdmittedRun<'_, Node>,
    associations: &super::capture::ScopeAssociations<Node>,
    atomics: &[Atomic<Node, S>],
    controls: &[Control<Node, S>],
    mut context: Context<Node, S>,
    reusable: &[fit::ReusableLine],
    environment: LineEnvironment<
        impl FnMut(f64, f64) -> Result<Band, FitError>,
        impl FnMut(f64, &surgeist_text::LineSelectionRequest) -> Result<Option<f64>, FitError>,
        impl FnMut(&FinalizedLine, fit::EncounterReplay) -> Result<Vec<fit::EncounterBasis>, FitError>,
        impl FnMut() -> Result<(), FitError>,
        impl FnMut(fit::EncounterReset),
        impl FnMut(),
    >,
) -> Result<Output<Node, S>, FitError> {
    let binding = run.binding;
    let LineEnvironment {
        band: band_source,
        reset: reset_candidate,
        checkpoint_group,
        clear: clear_source,
        replay: replay_encounters,
        accept: accept_encounters,
    } = environment;
    let origin = match context.frame {
        Frame::Measurement => Point::ZERO,
        Frame::Canonical { origin, .. } => origin,
    };
    let frame = context.line_frame(origin);
    // Only actual nominated own-box endpoints are intrinsic here. Their scope
    // property is executed by the scope allocator, while independent child
    // controls keep their own line-edge alignment.
    let mut intrinsic = None;
    let mut first_own = None;
    for control in controls {
        if control.allocation.strut.is_some()
            && let Some(row) = super::scope_alignment::nominated_endpoint(
                &context.scope_alignments,
                associations,
                control.node,
                &control.allocation.occurrence,
            )?
        {
            first_own = Some(row);
            break;
        }
    }
    if let Some(first) = first_own {
        let (mut owned, lease) = context
            .scope_alignments
            .preparation_buffer::<Control<Node, S>>(
                &binding.prepared,
                &first.scope,
                controls.len(),
            )?;
        for control in controls {
            let own_box = super::scope_alignment::nominated_endpoint(
                &context.scope_alignments,
                associations,
                control.node,
                &control.allocation.occurrence,
            )?;
            let mut allocation = control.allocation.clone();
            if let Some(row) = own_box
                && let Some(strut) = &mut allocation.strut
            {
                strut.alignment =
                    super::scope_alignment::intrinsic_alignment(row, strut.alignment)?;
            }
            owned.push(Control {
                node: control.node,
                source_index: control.source_index,
                kind: control.kind,
                allocation,
                endpoint: control.endpoint,
            });
        }
        intrinsic = Some((owned, lease));
    }
    let controls = intrinsic
        .as_ref()
        .map(|(values, _)| values.as_slice())
        .unwrap_or(controls);
    if let Some(wrapping) = &mut context.wrapping {
        wrapping.align_scope_metrics(binding, associations, &context.scope_alignments)?;
    }
    let allocations = binding
        .atomics
        .iter()
        .filter(|row| run.nodes().contains(&row.node))
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
                    frame,
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
    let orientation = frame.orientation();
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
            groups: context.groups.clone(),
            wrapping,
            formatted: &context.formatted,
            line_box_context: context.line_box_context,
            prepared: &binding.prepared,
            segment: run.segment.clone(),
            scope_alignments: &context.scope_alignments,
            available,
            base_direction: direction,
            block_direction: match context.axes.direction() {
                Direction::Ltr => ParagraphBaseDirection::LeftToRight,
                Direction::Rtl => ParagraphBaseDirection::RightToLeft,
            },
            alignment: context.alignment,
            wrap_style: context.wrap_style,
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
            frame.canonical_block(
                position.proposed,
                position.minimum,
                baseline,
                position.height,
            )
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
            checkpoint_group,
        },
    )?;
    output(
        run,
        associations,
        atomics,
        controls,
        &context,
        origin,
        lines,
    )
}

fn initial_allocation<Node: Copy + Eq, S: LayoutScalar>(
    frame: LineFrame,
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
    let (flow, fit) = flow(
        frame,
        item,
        item.size.width.to_f64(),
        item.size.height.to_f64(),
        frame.atomic_baseline(item)?,
    )?;
    Ok(AtomicAllocation {
        occurrence,
        allocation: AtomicLineAllocation::InFlow {
            fit,
            alignment: alignment(item.alignment)?,
            flow,
            box_geometry: initial_box(frame, atomic)?,
        },
    })
}

fn initial_box<Node, S: LayoutScalar>(
    frame: LineFrame,
    atomic: &Atomic<Node, S>,
) -> Result<AtomicBoxAllocation, FitError> {
    let item = atomic.item;
    let baseline = frame.atomic_baseline(item)?;
    let content = atomic.content_box;
    if frame.vertical_cell() {
        let x = add(item.margin.top.to_f64(), atomic.relative_offset.y.to_f64())?;
        let y = add(-baseline, -atomic.relative_offset.x.to_f64())?;
        Ok(AtomicBoxAllocation {
            border: Rect::new(x, y, item.size.height.to_f64(), item.size.width.to_f64()),
            content: Rect::new(
                add(x, content.origin().y.to_f64())?,
                add(
                    y,
                    finite(
                        item.size.width.to_f64()
                            - content.origin().x.to_f64()
                            - content.size().width.to_f64(),
                    )?,
                )?,
                content.size().height.to_f64(),
                content.size().width.to_f64(),
            ),
            baseline: -atomic.relative_offset.x.to_f64(),
        })
    } else {
        let x = add(item.margin.left.to_f64(), atomic.relative_offset.x.to_f64())?;
        let y = add(-baseline, atomic.relative_offset.y.to_f64())?;
        Ok(AtomicBoxAllocation {
            border: Rect::new(x, y, item.size.width.to_f64(), item.size.height.to_f64()),
            content: Rect::new(
                add(x, content.origin().x.to_f64())?,
                add(y, content.origin().y.to_f64())?,
                content.size().width.to_f64(),
                content.size().height.to_f64(),
            ),
            baseline: atomic.relative_offset.y.to_f64(),
        })
    }
}

fn flow<S: LayoutScalar>(
    frame: LineFrame,
    item: AtomicInlineBoxParticipant<S>,
    width: f64,
    height: f64,
    baseline: f64,
) -> Result<(AtomicFlowAllocation, surgeist_text::AtomicFitAllocation), FitError> {
    let (left, right, over, under, inline, cross) = if frame.vertical_cell() {
        (
            item.margin.top,
            item.margin.bottom,
            item.margin.right,
            item.margin.left,
            height,
            width,
        )
    } else {
        (
            item.margin.left,
            item.margin.right,
            item.margin.top,
            item.margin.bottom,
            width,
            height,
        )
    };
    let left = left.to_f64();
    let right = right.to_f64();
    let css = surgeist_text::CssMarginBoxFit::try_new(left, inline, right)?;
    let advance = css.advance();
    let (occupied_start, occupied_end) = css.occupied_inline_bounds();
    Ok((
        AtomicFlowAllocation {
            advance,
            before_baseline: add(over.to_f64(), baseline)?,
            after_baseline: if item.first_baseline.is_some() {
                add(cross - baseline, under.to_f64())?
            } else {
                0.0
            },
            occupied_start,
            occupied_end,
        },
        surgeist_text::AtomicFitAllocation::CssMarginBox(css),
    ))
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
    let frame = context.line_frame(origin);
    let anchor = finalized_anchor(
        context,
        origin,
        band.start,
        line.metric_bounds().size.width,
        block,
        line,
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
            let initial = initial_box(frame, atomic)?;
            let (raw_x, raw_y) = raw_box(frame, atomic, anchor, slot.origin, feedback.baseline)?;
            let slot_cross = frame.atomic_cross(anchor, feedback.baseline, slot.origin, 0.0)?;
            let left = round(raw_x)?;
            let top = round(raw_y)?;
            if slot.scope_displacement.get() != 0.0 {
                let amount = slot.scope_displacement.get();
                let (matched, shifted, parent) = if frame.vertical_cell() {
                    (raw_x - amount, left, frame.origin.x)
                } else {
                    (raw_y + amount, top, frame.origin.y)
                };
                // Scope placement precedes this same actual edge policy. Its
                // contribution is not applied again to the published box.
                qualify_shift_coordinate::<S>((matched + 0.5).floor() - parent, shifted - parent)?;
            }
            if let InlineControlAlignment::Shift(offset) = item.alignment
                && offset.get() != S::ZERO
            {
                let amount = offset.get().to_f64();
                let (matched, shifted, parent) = if frame.vertical_cell() {
                    (raw_x - amount, left, frame.origin.x)
                } else {
                    (raw_y + amount, top, frame.origin.y)
                };
                // Both points undergo the actual edge policy before narrowing.
                // Undoing the shift after rounding would lose threshold crossings.
                let matched = (matched + 0.5).floor();
                qualify_shift_coordinate::<S>(matched - parent, shifted - parent)?;
            }
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
            let actual_baseline = round(if frame.vertical_cell() {
                frame.atomic_cross(anchor, feedback.baseline, slot.origin, initial.baseline)?
            } else {
                add(raw_y, frame.atomic_baseline(item)?)?
            })?;
            let baseline_over = if frame.vertical_cell() {
                finite(right - actual_baseline)?
            } else {
                finite(actual_baseline - top)?
            };
            let (flow, fit) = flow(frame, item, width, height, baseline_over)?;
            Ok(AtomicAllocation {
                occurrence: slot.occurrence,
                allocation: AtomicLineAllocation::InFlow {
                    fit,
                    alignment: alignment(item.alignment)?,
                    flow,
                    box_geometry: AtomicBoxAllocation {
                        border: frame.atomic_local_rect(
                            anchor,
                            feedback.baseline,
                            slot.origin,
                            Rect::new(left, top, width, height),
                        )?,
                        content: frame.atomic_local_rect(
                            anchor,
                            feedback.baseline,
                            slot.origin,
                            Rect::new(
                                content_x,
                                content_y,
                                (content_right - content_x).max(0.0),
                                (content_bottom - content_y).max(0.0),
                            ),
                        )?,
                        baseline: frame.cross_distance(slot_cross, actual_baseline)?,
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
    let anchor = finalized_anchor(
        context,
        origin,
        band.start,
        line.metric_bounds().size.width,
        block,
        line,
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
            let scope_displacement = match line
                .control_slots()
                .find(|slot| slot.control() == allocation.occurrence)
            {
                Some(slot) => slot.scope_displacement().get(),
                None if line.scope_allocations().is_empty() => 0.0,
                None => return Err(FitError::Dependency),
            };
            let strut = original
                .allocation
                .strut
                .map(|strut| {
                    let metrics = context.line_frame(origin).aligned_strut(
                        anchor,
                        feedback,
                        surgeist_text::LineStrut {
                            baseline: strut.baseline,
                            line_extent: strut.line_extent,
                        },
                        strut.alignment,
                        scope_displacement,
                    )?;
                    Ok::<_, FitError>(surgeist_text::ControlStrut {
                        baseline: metrics.baseline,
                        line_extent: metrics.line_extent,
                        alignment: strut.alignment,
                    })
                })
                .transpose()?;
            Ok(surgeist_text::ControlAllocation {
                occurrence: allocation.occurrence.clone(),
                strut,
                boundary: original.allocation.boundary.clone(),
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
    let anchor = finalized_anchor(
        context,
        origin,
        band.start,
        line.metric_bounds().size.width,
        block,
        line,
    )?;
    originals
        .iter()
        .enumerate()
        .map(|(index, strut)| {
            let displacement = line
                .line_box_strut_scope_displacement(index)
                .ok_or(FitError::Dependency)?
                .get();
            Ok(surgeist_text::LineBoxStrut {
                scope: strut.scope.clone(),
                metrics: context.line_frame(origin).aligned_strut(
                    anchor,
                    feedback,
                    strut.metrics,
                    strut.alignment,
                    displacement,
                )?,
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
    let anchor = finalized_anchor(
        context,
        origin,
        band.start,
        line.metric_bounds().size.width,
        block,
        line,
    )?;
    let frame = context.line_frame(origin);
    let over = frame.cross_offset(anchor.baseline, -strut.baseline)?;
    let canonical_over = round(over)?;
    Ok(Some(surgeist_text::LineStrut {
        baseline: frame.cross_distance(canonical_over, anchor.baseline)?,
        line_extent: frame.cross_distance(
            canonical_over,
            round(frame.cross_offset(over, strut.line_extent)?)?,
        )?,
    }))
}

fn line_anchor<Node, S: LayoutScalar>(
    context: &Context<Node, S>,
    origin: Point<f64>,
    start: f64,
    available: f64,
    block: f64,
    baseline: f64,
    height: f64,
) -> Result<LineAnchor, FitError> {
    context
        .line_frame(origin)
        .anchor(start, available, block, baseline, height)
}

fn finalized_anchor<Node, S: LayoutScalar>(
    context: &Context<Node, S>,
    origin: Point<f64>,
    start: f64,
    available: f64,
    block: f64,
    line: &FinalizedLine,
) -> Result<LineAnchor, FitError> {
    let disposition = fit::LineDisposition::resolve(line)?;
    line_anchor(
        context,
        origin,
        start,
        available,
        block,
        disposition.baseline(line),
        disposition.height(line),
    )
}

fn output<Node: Copy + Eq, S: LayoutScalar>(
    run: &super::AdmittedRun<'_, Node>,
    associations: &super::capture::ScopeAssociations<Node>,
    atomics: &[Atomic<Node, S>],
    controls: &[Control<Node, S>],
    context: &Context<Node, S>,
    origin: Point<f64>,
    lines: Vec<FittedLine>,
) -> Result<Output<Node, S>, FitError> {
    let binding = run.binding;
    let frame = context.line_frame(origin);
    let orientation = frame.orientation();
    let named = named_baseline(
        &binding.prepared,
        context.axes,
        context.inline_baseline_context,
    );
    let mut report = MixedInlineRunReportOf {
        baselines: InlineBaselineReportOf::Native(PhysicalBaselineReport {
            context: context.inline_baseline_context,
            first: None,
            last: None,
            first_named: false,
            last_named: false,
            frame: match context.frame {
                Frame::Measurement => crate::block::baseline_frame::BaselineFrame::Measured {
                    used_size: frame.size,
                    axes: frame.axes,
                },
                Frame::Canonical { .. } => {
                    crate::block::baseline_frame::BaselineFrame::CanonicalLocal { axes: frame.axes }
                }
            },
        }),
        lines: Vec::new(),
        hyphenation_state: context.hyphenation_state,
        indent_state: context.indent_state,
        line_box_context: context.line_box_context,
        inline_extent: S::ZERO,
        block_extent: S::ZERO,
        float_edge_phase: None,
        atomics: Vec::new(),
        controls: Vec::new(),
        group_lines: Vec::new(),
        nonformatted_controls: Vec::new(),
        post_line_clear_intents: Vec::new(),
        line_bands: Vec::new(),
        formatted_lines: None,
    };
    let program = lines.first().and_then(|line| line.program.clone());
    let mut natural_advances = Vec::new();
    let mut placements = Vec::new();
    let mut bands = Vec::new();
    let mut flows = Vec::new();
    let mut amounts = Vec::new();
    let scope_bases =
        super::scope_alignment::ScopeBases::from_lines(&lines, &context.scope_alignments)?;
    let mut encounters = Vec::new();
    let mut zones = Vec::new();
    let mut phrases = Vec::new();
    let mut box_assignments = Vec::new();
    let mut box_uses = Vec::new();
    let mut line_bounds = Vec::new();
    let mut fragments = Vec::new();
    let mut atomic_origins = Vec::new();
    let mut static_positions = Vec::new();
    let mut nonformatted_controls = Vec::new();
    let mut outflows = Vec::new();
    let mut formatted = super::formatted::FormattedTailUses::default();
    let observe_formatted = context.formatted.observes()
        || lines.iter().any(|line| {
            line.role_uses
                .iter()
                .any(|used| used.demanded_assignment().is_some())
        });
    for (entry_index, line) in lines.into_iter().enumerate() {
        natural_advances.push(line.natural_advance);
        let line_index = report.line_bands.len();
        let is_formatted = line.disposition.is_formatted();
        if is_formatted && context.groups.is_active() {
            let selected = line.finalized.selected_line();
            let request = selected.request();
            if let Some(width) = request.context.available_inline {
                report
                    .group_lines
                    .push(crate::group_alignment::NaturalLine {
                        width,
                        indent: request.context.indent,
                        advance: line.natural_advance,
                        direction: request.context.line_box_direction,
                        used: request.context.alignment_insets,
                        witness: crate::group_alignment::Witness::Text {
                            request: Box::new(request.clone()),
                            amounts: line.amounts.clone(),
                        },
                    });
            }
        }
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
        if observe_formatted && is_formatted {
            let selected = line.finalized.selected_line();
            formatted
                .accepted
                .push(super::formatted::FormattedTailEndpoint::new(
                    selected.request().end.clone(),
                    selected.request().alternative.clone(),
                ));
        }
        if is_formatted
            && context.tail != fit::TailContext::Unrestricted
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
        let admission = match context.available {
            AvailableOf::MinContent => line
                .finalized
                .measurements()
                .margin_edge_min_content_contribution(),
            AvailableOf::MaxContent => line
                .finalized
                .measurements()
                .margin_edge_max_content_contribution(),
            AvailableOf::Definite(_) => line.finalized.measurements().margin_edge_fit(),
        };
        let indent = line.finalized.selected_line().request().context.indent;
        let css_width = fit::css_margin_edge_width(admission, indent)?;
        if is_formatted {
            report.indent_state.commit(
                line.finalized.selected_line().request().context.end
                    == surgeist_text::LineEnd::Forced,
            );
        }
        let anchor = finalized_anchor(
            context,
            origin,
            line.inline_start,
            line.available,
            line.placed_start,
            &line.finalized,
        )?;
        // Every semantic receipt remains ordered, including a phantom
        // completion. Only formatted entries below create line geometry.
        bands.push(line.bands);
        flows.push(line.flow);
        amounts.push(line.amounts);
        encounters.push(line.encounters);
        zones.push(line.zone_basis);
        phrases.push(line.phrase_basis);
        if let fit::LineDisposition::NonFormatted(completion) = line.disposition {
            let used = line_anchor(
                context,
                origin,
                line.inline_start,
                line.available,
                line.placed_start,
                0.0,
                0.0,
            )?;
            frame.qualify_scope_frames::<S>(used, &line.finalized)?;
            for slot in line.finalized.control_slots() {
                let actual = binding
                    .controls
                    .iter()
                    .find(|row| row.occurrence == slot.control())
                    .expect("retained nonformatted control has actual capture ownership");
                let (left, _) = slot.strip_interval();
                let point = frame.point(used, 0.0, left, slot.slot().y)?;
                frame.qualify_scope_position::<S>(point, slot.scope_displacement().get())?;
                nonformatted_controls.push((actual.node, frame.relative_point(point)?));
            }
            for slot in line.finalized.atomic_slots() {
                let actual = binding
                    .atomics
                    .iter()
                    .find(|row| row.occurrence == slot.occurrence)
                    .expect("retained nonformatted outflow has actual capture ownership");
                if !matches!(slot.allocation, AtomicLineAllocation::OutOfFlow) {
                    return Err(FitError::InvalidGeometry);
                }
                static_positions.push((
                    actual.node,
                    frame.static_position(
                        used,
                        0.0,
                        0.0,
                        slot.origin.x,
                        slot.scope_displacement.get(),
                    )?,
                ));
                outflows.push((entry_index, slot.occurrence, actual.node));
            }
            placements.push(surgeist_text::PlacementEntry::NonFormatted(
                surgeist_text::NonFormattedLinePlacement {
                    line: completion,
                    origin: {
                        let point = frame.anchor_point(used);
                        surgeist_text::Point::new(point.x, point.y)
                    },
                    orientation,
                    atomics: Vec::new(),
                },
            ));
            continue;
        }
        frame.qualify_scope_frames::<S>(anchor, &line.finalized)?;
        // Keep the accepted physical anchor in its actual used box frame.
        // In particular, an indefinite end-affine frame is resolved only when
        // the owning block knows its final border-box extent.
        let origin_baseline = context.axes.block_axis_coordinate(origin);
        let local_baseline = add(anchor.baseline, -origin_baseline)?;
        add(-origin_baseline, anchor.baseline)?;
        // Normal-flow allocations and placed metric bounds both contribute to
        // overflow, including lines without a source fragment or ink. Intrinsic
        // allocation composes actual signed indent once with the requested
        // admission operands; metric and source geometry remain independent.
        let width = match line
            .finalized
            .selected_line()
            .request()
            .context
            .available_inline
        {
            Some(width) => width,
            None => match css_width {
                Some(width) => width,
                None => finite(contribution.advance() + indent)?.max(0.0),
            },
        };
        let allocation = frame.allocation_rect(
            line.inline_start,
            width,
            line.flow.start,
            feedback.line_height,
        )?;
        let metric_frame = line.finalized.metric_bounds();
        let metrics = if frame.vertical_cell() {
            frame.rect(anchor, feedback.baseline, metric_frame)?
        } else {
            ScrollRectOf::try_new(
                Point::new(
                    number(anchor.inline_origin + metric_frame.origin.x - origin.x)?,
                    number(
                        add(anchor.baseline, -feedback.baseline)? + metric_frame.origin.y
                            - origin.y,
                    )?,
                ),
                Size::new(
                    number(metric_frame.size.width)?,
                    number(metric_frame.size.height)?,
                ),
            )
            .map_err(|_| FitError::InvalidGeometry)?
        };
        line_bounds.push(LineBounds {
            allocation,
            metrics,
        });
        let InlineBaselineReportOf::Native(baselines) = &mut report.baselines else {
            unreachable!("the concrete Text adapter constructs native baseline evidence");
        };
        if baselines.first.is_none() {
            baselines.first_named = named;
        }
        baselines.first.get_or_insert(local_baseline);
        baselines.last = Some(local_baseline);
        baselines.last_named = named;
        report.block_extent = report.block_extent.max(number(line.flow.next)?);
        let inline_end = match css_width {
            Some(width) => add(line.inline_start, width)?,
            None => line.inline_start + contribution.advance() + indent,
        };
        report.inline_extent = report.inline_extent.max(number(
            inline_end
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
                .map(|rect| frame.rect(anchor, feedback.baseline, rect))
                .transpose()?;
            fragments.push((
                owner,
                InlineFragmentOutputOf::new(
                    fragment,
                    rect,
                    frame.relative_point(frame.anchor_point(anchor))?,
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
            let (left, right) = slot.strip_interval();
            let inline_extent = number(finite(right - left)?)?;
            let physical = frame.point(anchor, feedback.baseline, left, slot.slot().y)?;
            frame.qualify_scope_position::<S>(physical, slot.scope_displacement().get())?;
            if let Some(strut) = control.allocation.strut
                && let surgeist_text::LineBoxAlignment::Shift(offset) = strut.alignment
                && offset.get() != 0.0
            {
                let (matched, shifted, parent) = if frame.vertical_cell() {
                    (physical.x - offset.get(), physical.x, frame.origin.x)
                } else {
                    (physical.y + offset.get(), physical.y, frame.origin.y)
                };
                // Control anchors have no atomic edge-quantization stage.
                qualify_shift_coordinate::<S>(matched - parent, shifted - parent)?;
            }
            if let Some(components) = control
                .allocation
                .boundary
                .as_ref()
                .and_then(surgeist_text::InlineBoundaryAllocation::box_components)
                && components.margin() != 0.0
            {
                let flow = slot.flow_interval();
                let strip = slot.strip_interval();
                let context = control
                    .allocation
                    .boundary
                    .as_ref()
                    .and_then(surgeist_text::InlineBoundaryAllocation::box_context)
                    .expect("component edge has checked context");
                let left_side = match control.kind {
                    InlineParticipantLayoutKind::InlineBoundaryStart => {
                        context.start_side() == surgeist_text::LineEdgeSide::Start
                    }
                    InlineParticipantLayoutKind::InlineBoundaryEnd => {
                        context.start_side() == surgeist_text::LineEdgeSide::End
                    }
                    _ => unreachable!("a component edge retains its admitted endpoint role"),
                };
                let (before, after) = if left_side {
                    (flow.0, strip.0)
                } else {
                    (strip.1, flow.1)
                };
                let before = frame.point(anchor, feedback.baseline, before, slot.slot().y)?;
                let after = frame.point(anchor, feedback.baseline, after, slot.slot().y)?;
                let (before, after) = if frame.vertical_cell() {
                    (before.y - frame.origin.y, after.y - frame.origin.y)
                } else {
                    (before.x - frame.origin.x, after.x - frame.origin.x)
                };
                let actual = number::<S>(after)?;
                if S::from_f64(before) == actual {
                    return Err(FitError::Numeric(
                        components.margin(),
                        LayoutTextNumericError::NonZeroLost,
                    ));
                }
            }
            let physical = frame.relative_point(physical)?;
            report.controls.push(InlineControlSourceOf {
                kind: control.kind,
                source_index: control.source_index,
                position: InlineSourcePositionOf::Physical(physical),
                inline_extent,
                endpoint: control.endpoint,
                margin_flow: None,
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
                    frame.static_position(
                        anchor,
                        feedback.baseline,
                        feedback.line_height,
                        slot.origin.x,
                        slot.scope_displacement.get(),
                    )?,
                ));
                outflows.push((entry_index, slot.occurrence, binding.node));
                continue;
            };
            let atomic = atomics
                .iter()
                .find(|row| row.node == binding.node)
                .expect("actual atomic measurement remains retained");
            let border =
                frame.atomic_rect(anchor, feedback.baseline, slot.origin, box_geometry.border)?;
            let content =
                frame.atomic_rect(anchor, feedback.baseline, slot.origin, box_geometry.content)?;
            let baseline = frame.atomic_cross(
                anchor,
                feedback.baseline,
                slot.origin,
                box_geometry.baseline,
            )?;
            atomic_origins.push((atomic.node, Point::new(border.origin.x, border.origin.y)));
            let size = Size::new(number(border.size.width)?, number(border.size.height)?);
            let physical = Point::new(
                number(border.origin.x - origin.x.to_f64())?,
                number(border.origin.y - origin.y.to_f64())?,
            );
            let mut item = atomic.item;
            let (raw_x, raw_y) = raw_box(frame, atomic, anchor, slot.origin, feedback.baseline)?;
            item.border = canonical_edges(raw_x, raw_y, atomic.item.size, atomic.item.border)?;
            item.padding = canonical_edges(raw_x, raw_y, atomic.item.size, atomic.item.padding)?;
            item.first_baseline = item
                .first_baseline
                .map(|_| number(frame.participant_baseline(border, baseline)?))
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
                position: InlineSourcePositionOf::Physical(physical),
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
        placements.push(surgeist_text::PlacementEntry::Formatted(LinePlacement {
            line: line.finalized,
            anchor,
            orientation,
            atomics: atomic_placements,
        }));
    }
    let mut nodes = Vec::new();
    nodes
        .try_reserve_exact(run.nodes().len())
        .map_err(|_| FitError::Allocation)?;
    nodes.extend_from_slice(run.nodes());
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
        nonformatted_controls,
        paragraph: PendingParagraph {
            binding_index: run.binding_index,
            segment: run.segment.clone(),
            container: binding.container,
            nodes,
            prepared: binding.prepared.clone(),
            entries: placements,
            wrap_style: context.wrap_style,
            program,
            natural_advances,
            boxes: box_assignments,
            bands,
            flows,
            amounts,
            scopes: scope_bases,
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
    frame: LineFrame,
    atomic: &Atomic<Node, S>,
    anchor: LineAnchor,
    slot: surgeist_text::Point,
    line_baseline: f64,
) -> Result<(f64, f64), FitError> {
    let border = frame.atomic_rect(
        anchor,
        line_baseline,
        slot,
        initial_box(frame, atomic)?.border,
    )?;
    Ok((border.origin.x, border.origin.y))
}

fn alignment<S: LayoutScalar>(
    value: InlineControlAlignment<S>,
) -> Result<surgeist_text::LineBoxAlignment, FitError> {
    Ok(match value {
        InlineControlAlignment::Baseline => surgeist_text::LineBoxAlignment::Baseline,
        InlineControlAlignment::Top => surgeist_text::LineBoxAlignment::Top,
        InlineControlAlignment::Bottom => surgeist_text::LineBoxAlignment::Bottom,
        InlineControlAlignment::Shift(offset) => surgeist_text::LineBoxAlignment::Shift(
            surgeist_text::BaselineShift::try_new(offset.get().to_f64())?,
        ),
    })
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
    crate::scalar::checked_geometry_add(base, delta).map_err(FitError::from)
}

/// Qualify the actual recipient stage, after any caller-owned pixel policy.
/// The comparison-only matched point need not fit the destination scalar lane.
fn qualify_shift_coordinate<S: LayoutScalar>(matched: f64, shifted: f64) -> Result<(), FitError> {
    let narrowed = number::<S>(shifted)?;
    if matched != shifted && S::from_f64(matched) == narrowed {
        return Err(FitError::Numeric(
            shifted,
            LayoutTextNumericError::NonZeroLost,
        ));
    }
    Ok(())
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
                prepared.captured_owner().is_some_and(|paragraph| {
                    paragraph.domain() == scope.domain()
                        && paragraph
                            .domain()
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
        super::boxes::BoxFrontier::boundary(
            self.segment.clone(),
            boundary.clone(),
            incidence,
            native,
            relations,
        )
    }

    // A local box that really exits before the paragraph must be discharged
    // by an issued outgoing incidence. A retained force pin cannot stand in for
    // that completion, and a scope continuing into another paragraph is exempt.
    fn closes_segment(
        &self,
        outgoing: &super::boxes::BoxFrontier,
        segment: &surgeist_text::PreparedSourceSegment,
    ) -> Result<bool, FitError> {
        use surgeist_text::CapturedScopeIncidence as R;
        let prepared = outgoing.boundary.prepared_text();
        let Some(paragraph) = prepared.captured_owner() else {
            return Ok(true);
        };
        let Some(paragraph_exit) = segment.exit_position() else {
            return Err(crate::WrappingScopeError::UnsupportedIncidence.into());
        };
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
        if prepared.captured_owner().is_none() {
            return Ok(vec![self.cut(boundary, None)?]);
        }
        let mut incidences = prepared.boundary_incidences(&self.segment, boundary)?;
        let forced = self.segment.opportunity_views().any(|offer| {
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
                    prepared.captured_owner().is_some_and(|paragraph| {
                        paragraph.domain() == scope.domain()
                            && paragraph
                                .domain()
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
        let source_segment = choice.source_segment();
        if source_segment != self.segment
            || choice
                .incoming
                .is_some_and(|entry| entry.frontier().segment != source_segment)
        {
            return Err(crate::WrappingScopeError::UnsupportedIncidence.into());
        }
        // A preceding native run's receipt retains its issuer. A new structural
        // segment establishes its own genuine entry incidence, even at an equal
        // PreparedBoundary; no old prefix is relabeled to the new segment.
        let entry = choice.incoming.or(self
            .incoming
            .as_ref()
            .filter(|frontier| frontier.segment == source_segment)
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
                    if self.closes_segment(&outgoing, &choice.source_segment())? {
                        completed.push(outgoing);
                    }
                }
                ends = completed.into_iter().last().into_iter().collect();
                if ends.is_empty() && start == end && end.prepared_text().captured_owner().is_some()
                {
                    let mut relations = Vec::new();
                    for (id, row, _) in &self.boxes {
                        let relation = row
                            .capture
                            .as_ref()
                            .map(|scope| {
                                super::boxes::horizon_relation(&choice.source_segment(), scope)
                            })
                            .transpose()?
                            .flatten()
                            .or_else(|| incoming.relation(*id));
                        if let Some(relation) = relation {
                            relations.push((*id, relation));
                        }
                    }
                    if let Some(completion) =
                        super::boxes::BoxFrontier::complete(&incoming, &choice, relations)?
                        && self.closes_segment(&completion, &choice.source_segment())?
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
                    let allocation = match &row.metric_contribution {
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
                            let mut strut = strut.clone();
                            if owned.is_empty() || strut.scope.is_some() {
                                if let Some(owner) = row.source_owner() {
                                    if let Some(declared) = &strut.scope
                                        && declared != &owner
                                    {
                                        let reason = if !declared
                                            .domain()
                                            .enter_position()
                                            .same_capture(&owner.enter_position())
                                        {
                                            surgeist_text::ScopeAlignmentError::ForeignCapture
                                        } else {
                                            surgeist_text::ScopeAlignmentError::BoundaryMismatch {
                                                scope: owner.capture_ordinal(),
                                            }
                                        };
                                        return Err(super::scope_alignment::refusal(reason).into());
                                    }
                                    strut.scope = Some(owner);
                                } else if self.scope_alignment_active || strut.scope.is_some() {
                                    let scope = strut.scope.as_ref().map_or_else(|| {
                                        start.prepared_text().captured_owner()
                                            .expect("active scope alignment has a captured paragraph").domain().capture_ordinal()
                                    }, surgeist_text::CapturedScope::capture_ordinal);
                                    return Err(super::scope_alignment::refusal(surgeist_text::ScopeAlignmentError::UnrepresentedOwnership { scope }).into());
                                }
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
