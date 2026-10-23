//! The existing block formatter's concrete consuming Text adapter.

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

pub(crate) struct Atomic<Node, S: LayoutScalar> {
    pub node: Node,
    pub item: AtomicInlineBoxParticipant<S>,
    pub content_box: ScrollRectOf<S>,
    pub relative_offset: Point<S>,
}

pub(crate) struct Control<Node> {
    pub node: Node,
    pub source_index: usize,
    pub kind: InlineParticipantLayoutKind,
    pub allocation: surgeist_text::ControlAllocation,
}

#[derive(Clone, Copy)]
pub(crate) enum Frame<S: LayoutScalar> {
    Measurement,
    Canonical { origin: Point<f64>, size: Size<S> },
}

pub(crate) struct Context<S: LayoutScalar> {
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
}

impl<S: LayoutScalar> Context<S> {
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
    pub report: MixedInlineRunReportOf<S>,
    pub paragraph: PendingParagraph<Node>,
    pub line_bounds: Vec<LineBounds<S>>,
    pub fragments: Vec<(Node, InlineFragmentOutputOf<S>)>,
    pub atomic_origins: Vec<(Node, Point<f64>)>,
    pub static_positions: Vec<(Node, Point<S>)>,
}

/// One invocation's coupled line-band and source-event transitions. Each
/// callback borrows the same actual provisional session, never a public root.
pub(crate) struct LineEnvironment<Band, Clear, Replay, Accept> {
    pub band: Band,
    pub clear: Clear,
    pub replay: Replay,
    pub accept: Accept,
}

pub(crate) fn layout<Node: Copy + Eq, S: LayoutScalar>(
    binding: &TextParagraphBinding<Node>,
    atomics: &[Atomic<Node, S>],
    controls: &[Control<Node>],
    context: Context<S>,
    reusable: &[fit::ReusableLine],
    environment: LineEnvironment<
        impl FnMut(f64, f64) -> Result<Band, FitError>,
        impl FnMut(f64, &surgeist_text::LineSelectionRequest) -> Result<Option<f64>, FitError>,
        impl FnMut(&FinalizedLine, fit::EncounterReplay) -> Result<Vec<fit::EncounterBasis>, FitError>,
        impl FnMut() -> Result<(), FitError>,
    >,
) -> Result<Output<Node, S>, FitError> {
    let LineEnvironment {
        band: band_source,
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
    let lines = fit::fit(
        fit::FitInput {
            prepared: &binding.prepared,
            available,
            base_direction: direction,
            alignment: context.alignment,
            last_alignment: context.last_alignment,
            justification: context.justification,
            indent: context.indent.widened(),
            indent_state: context.indent_state,
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
        |line, band, block| {
            Ok(fit::LineAllocations {
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
        replay_encounters,
        accept_encounters,
    )?;
    output(binding, atomics, controls, &context, origin, lines)
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
    context: &Context<S>,
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
        line.selected_line()
            .request()
            .context
            .available_inline
            .unwrap_or(feedback.advance),
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
    controls: &[Control<Node>],
    context: &Context<S>,
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
        line.selected_line()
            .request()
            .context
            .available_inline
            .unwrap_or(feedback.advance),
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
            })
        })
        .collect()
}

fn canonical_line_strut<S: LayoutScalar>(
    original: Option<surgeist_text::LineStrut>,
    context: &Context<S>,
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
        line.selected_line()
            .request()
            .context
            .available_inline
            .unwrap_or(feedback.advance),
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

fn line_anchor<S: LayoutScalar>(
    context: &Context<S>,
    origin: Point<f64>,
    start: f64,
    available: f64,
    block: f64,
    baseline: f64,
) -> Result<LineAnchor, FitError> {
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
    Ok(LineAnchor {
        inline_origin: round(add(origin.x.to_f64(), local_x)?)?,
        baseline: round(add(
            add(origin.y.to_f64(), context.cursor_block.to_f64())?,
            add(block, baseline)?,
        )?)?,
    })
}

fn output<Node: Copy + Eq, S: LayoutScalar>(
    binding: &TextParagraphBinding<Node>,
    atomics: &[Atomic<Node, S>],
    controls: &[Control<Node>],
    context: &Context<S>,
    origin: Point<f64>,
    lines: Vec<FittedLine>,
) -> Result<Output<Node, S>, FitError> {
    let mut report = MixedInlineRunReportOf {
        indent_state: context.indent_state,
        inline_extent: S::ZERO,
        block_extent: S::ZERO,
        float_edge_phase: None,
        first_baseline: None,
        last_baseline: None,
        atomics: Vec::new(),
        controls: Vec::new(),
        post_line_clear_intents: Vec::new(),
        line_bands: Vec::new(),
    };
    let mut placements = Vec::new();
    let mut bands = Vec::new();
    let mut flows = Vec::new();
    let mut amounts = Vec::new();
    let mut encounters = Vec::new();
    let mut line_bounds = Vec::new();
    let mut fragments = Vec::new();
    let mut atomic_origins = Vec::new();
    let mut static_positions = Vec::new();
    let mut outflows = Vec::new();
    for (line_index, line) in lines.into_iter().enumerate() {
        let feedback = line.finalized.feedback();
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
        // overflow, including lines without a source fragment or ink. Native
        // intrinsic width includes the signed start margin and excludes observed
        // trailing whitespace rather than using glyph advance.
        let width = match line
            .finalized
            .selected_line()
            .request()
            .context
            .available_inline
        {
            Some(width) => width,
            None => finite(
                feedback.advance - feedback.trailing_whitespace
                    + line.finalized.selected_line().request().context.indent,
            )?
            .max(0.0),
        };
        let allocation = ScrollRectOf::try_new(
            Point::new(
                number(anchor.inline_origin - origin.x)?,
                number(add(context.cursor_block.to_f64(), line.flow.start)?)?,
            ),
            Size::new(number(width)?, number(feedback.line_height)?),
        )
        .map_err(|_| FitError::InvalidGeometry)?;
        let metrics = ScrollRectOf::try_new(
            Point::new(
                number(anchor.inline_origin - origin.x)?,
                number(add(anchor.baseline, -feedback.baseline)? - origin.y)?,
            ),
            Size::new(number(width)?, number(feedback.line_height)?),
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
                + feedback.advance
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
            let node = fragment_node(binding, view.owner());
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
                node,
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
            let physical = Point::new(
                number(add(anchor.inline_origin, slot.slot().x)? - origin.x)?,
                number(add(anchor.baseline, slot.slot().y - feedback.baseline)? - origin.y)?,
            );
            let logical = context
                .axes
                .logical_point(physical, Size::ZERO, context.containing_size);
            report.controls.push(InlineControlSourceOf {
                kind: control.kind,
                source_index: control.source_index,
                inline_start: logical.inline - context.inline_start_inset(),
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
        placements.push(LinePlacement {
            line: line.finalized,
            anchor,
            orientation: LineOrientation::Horizontal,
            atomics: atomic_placements,
        });
    }
    let indent_end = report.indent_state;
    Ok(Output {
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
            bands,
            flows,
            amounts,
            encounters,
            outflows,
            indent: context.indent.widened(),
            indent_start: context.indent_state,
            indent_end,
        },
    })
}

fn fragment_node<Node: Copy + Eq>(
    binding: &TextParagraphBinding<Node>,
    owner: surgeist_text::FinalizedFragmentOwner,
) -> Node {
    match owner {
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
    }
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
fn add(base: f64, delta: f64) -> Result<f64, FitError> {
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
