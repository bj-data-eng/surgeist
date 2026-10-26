use super::scroll::retained_child_scroll_geometry;
use super::{Constants, with_block_scroll_projections};
use crate::error::layout_child_geometry_error;
use crate::geometry::{LogicalEdgesOf, LogicalPointOf, LogicalSizeOf, PhysicalAxis};
use crate::inline::PostLineClearIntent;
use crate::scroll::ScrollContributionAccumulatorOf;
use crate::{
    AvailableOf, Clear, Compute, Edges, Float, FloatExclusion, FloatExclusionIntervalErrorOf,
    FloatExclusionIntervalOf, FloatExclusionQueryOf, LayoutErrorKindOf, LayoutErrorOf,
    LayoutErrorSiteOf, LayoutInvalidInputOf, LayoutMissingContext, LayoutOperation, LayoutResultOf,
    LayoutScalar, NodeOutputOf, Point, ScrollGeometryOf, ScrollRectOf, Size, Traverse,
};

#[derive(Clone)]
pub(super) struct PendingFloat<Node, S: LayoutScalar> {
    pub(super) accepted: Option<AcceptedFloatPlacement<S>>,
    pub(super) node: Node,
    pub(super) source_index: usize,
    pub(super) side: Float,
    pub(super) clear: Clear,
    pub(super) block_start: S,
    pub(super) size: Size<S>,
    pub(super) content_size: Size<S>,
    pub(super) border: Edges<S>,
    pub(super) padding: Edges<S>,
    pub(super) margin: Edges<S>,
    pub(super) float_exclusion: FloatExclusion,
    pub(super) child_compute_geometry: Option<ScrollGeometryOf<S>>,
}

/// The real inline-space role of this placement invocation. Intrinsic Max
/// has independent side frontiers, not a finite zero-width containing line.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum FloatInlineSpace {
    Finite,
    IntrinsicMaximum,
}

/// A resolved logical placement; publishing transforms this exact plan and
/// never repeats placement without its preceding-inline constraint.
#[derive(Clone, Debug)]
pub(super) struct AcceptedFloatPlacement<S: LayoutScalar> {
    pub(super) boxes: Option<crate::text_layout::boxes::BoxAssignment>,
    pub(super) logical_origin: LogicalPointOf<S>,
    pub(super) inline_space: FloatInlineSpace,
    pub(super) margin_block_start: S,
    pub(super) intrinsic_inline_demand: Option<S>,
    pub(super) prefix: f64,
    pub(super) observation: AcceptedFloatPrefix,
    pub(super) line: surgeist_text::FinalizedLine,
}

/// Distinct authentic receipts: a represented original seam, or the genuine
/// empty ParagraphEnd after an already consumed terminal mandatory boundary.
#[derive(Clone, Debug)]
pub(super) enum AcceptedFloatPrefix {
    Seam(surgeist_text::TextSeam),
    EmptyParagraphEnd {
        boundary: surgeist_text::PreparedBoundary,
        original: surgeist_text::TextSeam,
    },
}
impl AcceptedFloatPrefix {
    pub(super) fn observe(
        &self,
        line: &surgeist_text::FinalizedLine,
    ) -> Result<(f64, bool), crate::text_layout::fit::FitError> {
        use crate::text_layout::fit::FitError;
        match self {
            Self::Seam(seam) => {
                let observation = line.prefix_at(seam)?;
                Ok((
                    observation.advance(),
                    observation.has_retained_flow_participation(),
                ))
            }
            Self::EmptyParagraphEnd { boundary, original } => {
                let selected = line.selected_line();
                let request = selected.request();
                let owner = match original {
                    surgeist_text::TextSeam::Origin { origin, .. } => origin.prepared_text(),
                    surgeist_text::TextSeam::BeforeParticipant(participant)
                    | surgeist_text::TextSeam::AfterParticipant(participant) => match participant {
                        surgeist_text::PreparedParticipant::Text(p) => p.prepared_text(),
                        surgeist_text::PreparedParticipant::Atomic(p) => p.prepared_text(),
                        surgeist_text::PreparedParticipant::Control(p) => p.prepared_text(),
                        surgeist_text::PreparedParticipant::ComputedLineBoundary(p) => {
                            p.prepared_text()
                        }
                    },
                };
                if owner != line.prepared_text()
                    || request.start != *boundary
                    || request.end != *boundary
                    || !request.participants.is_empty()
                    || request.context.end != surgeist_text::LineEnd::Paragraph
                {
                    return Err(FitError::InvalidGeometry);
                }
                // This is the actual complete empty-line fold, not a guessed
                // source-origin mapping or a borrowed previous-line prefix.
                Ok((line.feedback().advance, false))
            }
        }
    }
}

/// Both coordinates are issued by the same actual ledger placement. Keeping
/// the original logical result avoids reversing a scalar frame transform.
pub(super) struct FloatPlacement<S: LayoutScalar> {
    pub(super) margin_block_start: S,
    pub(super) intrinsic_inline_demand: Option<S>,
    pub(super) logical: LogicalPointOf<S>,
    pub(super) physical: Point<S>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FloatLedgerSide {
    LineStart,
    LineEnd,
}

impl FloatLedgerSide {
    fn from_float(side: Float) -> Self {
        match side {
            Float::Left | Float::None => Self::LineStart,
            Float::Right => Self::LineEnd,
        }
    }

    fn is_cleared_by(self, clear: Clear) -> bool {
        match clear {
            Clear::None => false,
            Clear::Left => self == Self::LineStart,
            Clear::Right => self == Self::LineEnd,
            Clear::Both => true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct PhysicalMarginBox<S: LayoutScalar> {
    origin: Point<S>,
    size: Size<S>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
struct FloatLedgerOrder(usize);

#[derive(Clone, Copy, Debug)]
struct FloatLedgerEntry<S: LayoutScalar, Node> {
    node: Node,
    side: FloatLedgerSide,
    exclusion: FloatExclusion,
    physical_margin_box: PhysicalMarginBox<S>,
    inline_start: S,
    inline_end: S,
    block_start: S,
    block_end: S,
    ledger_order: FloatLedgerOrder,
}

impl<S: LayoutScalar, Node: Copy> FloatLedgerEntry<S, Node> {
    fn overlaps_block_span(self, block_start: S, block_end: S) -> bool {
        self.block_start < block_end && block_start < self.block_end
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct FloatBand<S: LayoutScalar> {
    pub(crate) inline_start: S,
    pub(crate) inline_end: S,
    pub(crate) next_transition: Option<S>,
    #[cfg(test)]
    pub(crate) evaluated: usize,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct BfcBandPlacement<S: LayoutScalar> {
    pub(super) location: Point<S>,
    pub(super) available_inline: S,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct BfcBandCandidate<S: LayoutScalar> {
    pub(super) block_start: S,
    pub(super) size: Size<S>,
    pub(super) margin: Edges<S>,
    pub(super) clear: Clear,
    pub(super) fallback: Point<S>,
    /// Ordinary auto inline sizing may shrink to a float band. A replacement
    /// instead needs a band that fits its resolved natural or authored size.
    pub(super) inline_size_can_shrink: bool,
}

#[derive(Clone, Copy)]
pub(super) struct ProviderBandContext<'a, Tree, Node> {
    pub(super) tree: &'a Tree,
    pub(super) container: Node,
    pub(super) enabled: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FloatBandQueryPurpose {
    PhysicalMarginBoxCollision,
    RectangularLineBand,
    ProviderBand,
}

#[derive(Clone, Debug)]
pub(crate) struct FloatExclusions<S: LayoutScalar, Node = ()> {
    flow_axes: crate::geometry::FlowAxes,
    containing_size: Size<S>,
    pub(super) containing_inline_start: S,
    pub(super) containing_inline_end: S,
    ledger: Vec<FloatLedgerEntry<S, Node>>,
}

#[derive(Clone, Debug)]
pub(crate) struct InheritedFloatExclusions<S: LayoutScalar, Node> {
    parent_flow_axes: crate::geometry::FlowAxes,
    parent_containing_size: Size<S>,
    child_logical_location: LogicalPointOf<S>,
    ledger: Vec<FloatLedgerEntry<S, Node>>,
}

impl<S: LayoutScalar, Node: Copy> FloatExclusions<S, Node> {
    pub(crate) fn new(
        flow_axes: crate::geometry::FlowAxes,
        containing_size: Size<S>,
        content_inline_size: S,
        inset: LogicalEdgesOf<S>,
    ) -> Self {
        Self {
            flow_axes,
            containing_size,
            containing_inline_start: inset.inline_start,
            containing_inline_end: inset.inline_start + content_inline_size,
            ledger: Vec::new(),
        }
    }

    /// Rebase this actual influencing float ledger to the preceding canonical
    /// pass. Inherited entries keep their genuine node and side provenance.
    pub(super) fn canonical_for_text(
        &self,
        container: Node,
        frames: &[crate::text_layout::state::CanonicalFrame<Node, S>],
    ) -> Result<Option<Self>, crate::text_layout::fit::FitError>
    where
        Node: Eq,
    {
        let Some(container) = frames.iter().find(|frame| frame.node == container) else {
            return Ok(None);
        };
        let content = container
            .output
            .scroll_geometry
            .ok_or(crate::text_layout::fit::FitError::InvalidGeometry)?
            .content_box();
        let logical_content_origin =
            self.flow_axes
                .logical_point(content.origin(), content.size(), container.output.size);
        let logical_content = self.flow_axes.logical_size(content.size());
        let mut result = self.clone();
        result.containing_size = container.output.size;
        result.containing_inline_start = logical_content_origin.inline;
        result.containing_inline_end = logical_content_origin.inline + logical_content.inline;
        for entry in &mut result.ledger {
            let frame = frames
                .iter()
                .find(|frame| frame.node == entry.node)
                .ok_or(crate::text_layout::fit::FitError::InvalidGeometry)?;
            let margin = frame.output.margin;
            let origin = Point::new(
                crate::text_layout::run::number::<S>(
                    frame.origin.x - container.origin.x - margin.left.to_f64(),
                )?,
                crate::text_layout::run::number::<S>(
                    frame.origin.y - container.origin.y - margin.top.to_f64(),
                )?,
            );
            let size = Size::new(
                crate::text_layout::run::number::<S>(
                    frame.output.size.width.to_f64() + margin.left.to_f64() + margin.right.to_f64(),
                )?,
                crate::text_layout::run::number::<S>(
                    frame.output.size.height.to_f64()
                        + margin.top.to_f64()
                        + margin.bottom.to_f64(),
                )?,
            );
            let logical_origin = self
                .flow_axes
                .logical_point(origin, size, result.containing_size);
            let logical_size = self.flow_axes.logical_size(size);
            entry.physical_margin_box = PhysicalMarginBox { origin, size };
            entry.inline_start = logical_origin.inline;
            entry.inline_end = logical_origin.inline + logical_size.inline;
            entry.block_start = logical_origin.block;
            entry.block_end = logical_origin.block + logical_size.block;
        }
        Ok(Some(result))
    }

    pub(super) fn is_empty(&self) -> bool {
        self.ledger.is_empty()
    }

    /// Every entry in an encounter's incoming ledger precedes that source
    /// window. Its actual margin-box top is a constraint even after expiry.
    pub(super) fn preceding_float_top(&self) -> Option<S> {
        self.ledger
            .iter()
            .map(|entry| entry.block_start)
            .reduce(S::max)
    }

    pub(super) fn for_ordinary_child(
        &self,
        child_logical_location: LogicalPointOf<S>,
    ) -> InheritedFloatExclusions<S, Node> {
        InheritedFloatExclusions {
            parent_flow_axes: self.flow_axes,
            parent_containing_size: self.containing_size,
            child_logical_location,
            ledger: self.ledger.clone(),
        }
    }

    pub(super) fn inherit_into_child(&mut self, inherited: &InheritedFloatExclusions<S, Node>) {
        let child_size = self.containing_size;
        let child_logical_size = inherited.parent_flow_axes.logical_size(child_size);
        let child_physical_origin = inherited.parent_flow_axes.physical_point(
            inherited.child_logical_location,
            child_logical_size,
            inherited.parent_containing_size,
        );
        let mut inherited_order = self.ledger.len();
        self.ledger
            .extend(inherited.ledger.iter().copied().map(|entry| {
                let physical_origin = Point::new(
                    entry.physical_margin_box.origin.x - child_physical_origin.x,
                    entry.physical_margin_box.origin.y - child_physical_origin.y,
                );
                let physical_size = entry.physical_margin_box.size;
                let logical_origin =
                    self.flow_axes
                        .logical_point(physical_origin, physical_size, child_size);
                let logical_size = self.flow_axes.logical_size(physical_size);
                let parent_side = match entry.side {
                    FloatLedgerSide::LineStart => inherited.parent_flow_axes.inline_start(),
                    FloatLedgerSide::LineEnd => inherited.parent_flow_axes.inline_end(),
                };
                let side = if parent_side == self.flow_axes.inline_start() {
                    FloatLedgerSide::LineStart
                } else if parent_side == self.flow_axes.inline_end() {
                    FloatLedgerSide::LineEnd
                } else {
                    let distance_from_start =
                        (logical_origin.inline - self.containing_inline_start).abs();
                    let distance_from_end = (self.containing_inline_end
                        - (logical_origin.inline + logical_size.inline))
                        .abs();
                    if distance_from_start <= distance_from_end {
                        FloatLedgerSide::LineStart
                    } else {
                        FloatLedgerSide::LineEnd
                    }
                };
                let ledger_order = FloatLedgerOrder(inherited_order);
                inherited_order += 1;
                FloatLedgerEntry {
                    node: entry.node,
                    side,
                    physical_margin_box: PhysicalMarginBox {
                        origin: physical_origin,
                        size: physical_size,
                    },
                    inline_start: logical_origin.inline,
                    inline_end: logical_origin.inline + logical_size.inline,
                    block_start: logical_origin.block,
                    block_end: logical_origin.block + logical_size.block,
                    ledger_order,
                    ..entry
                }
            }));
    }

    pub(super) fn place_float<'input, Tree, M>(
        &mut self,
        provider: ProviderBandContext<'_, Tree, Node>,
        float: &PendingFloat<Node, S>,
    ) -> LayoutResultOf<Node, Point<S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        self.place_float_for_line(provider, float, None)
            .map(|placement| placement.physical)
    }

    /// Record an already accepted border-box origin in this exact ledger frame.
    /// The same operation serves ordinary placement and publication of a plan
    /// that was accepted with its real preceding-inline constraint.
    fn record_placement(&mut self, float: &PendingFloat<Node, S>, border: LogicalPointOf<S>) {
        let margin = self.flow_axes.logical_edges(float.margin);
        let size = self.flow_axes.logical_size(float.size);
        let origin = LogicalPointOf::new(
            border.inline - margin.inline_start,
            border.block - margin.block_start,
        );
        let size = LogicalSizeOf::new(
            size.inline + margin.inline_sum(),
            size.block + margin.block_sum(),
        );
        self.ledger.push(FloatLedgerEntry {
            node: float.node,
            side: FloatLedgerSide::from_float(float.side),
            exclusion: float.float_exclusion,
            physical_margin_box: PhysicalMarginBox {
                origin: self
                    .flow_axes
                    .physical_point(origin, size, self.containing_size),
                size: self.flow_axes.physical_size(size),
            },
            inline_start: origin.inline,
            inline_end: origin.inline + size.inline,
            block_start: origin.block,
            block_end: origin.block + size.block,
            ledger_order: FloatLedgerOrder(self.ledger.len()),
        });
    }

    pub(super) fn place_float_for_line<'input, Tree, M>(
        &mut self,
        provider: ProviderBandContext<'_, Tree, Node>,
        float: &PendingFloat<Node, S>,
        line: Option<(f64, S)>,
    ) -> LayoutResultOf<Node, FloatPlacement<S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        self.place_float_in_space(provider, float, line, FloatInlineSpace::Finite)
    }

    pub(super) fn place_float_intrinsic<'input, Tree, M>(
        &mut self,
        provider: ProviderBandContext<'_, Tree, Node>,
        float: &PendingFloat<Node, S>,
    ) -> LayoutResultOf<Node, FloatPlacement<S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        self.place_float_in_space(provider, float, None, FloatInlineSpace::IntrinsicMaximum)
    }

    /// End-side occupation from the same raw, height-qualified ledger query.
    /// Reversed intrinsic frontiers remain meaningful independent distances.
    pub(super) fn end_occupation(&self, band: &FloatBand<S>) -> S {
        self.containing_inline_end - band.inline_end
    }

    fn place_float_in_space<'input, Tree, M>(
        &mut self,
        provider: ProviderBandContext<'_, Tree, Node>,
        float: &PendingFloat<Node, S>,
        line: Option<(f64, S)>,
        inline_space: FloatInlineSpace,
    ) -> LayoutResultOf<Node, FloatPlacement<S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let side = FloatLedgerSide::from_float(float.side);
        let logical_size = self.flow_axes.logical_size(float.size);
        let logical_margin = self.flow_axes.logical_edges(float.margin);
        let margin_box_size = LogicalSizeOf::new(
            logical_size.inline + logical_margin.inline_sum(),
            logical_size.block + logical_margin.block_sum(),
        );
        let mut candidate_block = self.clearance_block(float.block_start, float.clear);

        loop {
            let candidate_block_end = candidate_block + margin_box_size.block;
            let band = if provider.enabled {
                self.query_provider_band(
                    provider.tree,
                    provider.container,
                    candidate_block,
                    candidate_block_end,
                )?
            } else {
                self.query_physical_margin_box_collisions(candidate_block, candidate_block_end)
            };
            let available_inline = (band.inline_end - band.inline_start).max(S::ZERO);
            if inline_space == FloatInlineSpace::Finite
                && let Some((prefix, line_end)) = line
                && candidate_block < line_end
                && margin_box_size.block > S::ZERO
                && prefix + margin_box_size.inline.to_f64() > available_inline.to_f64()
            {
                candidate_block = line_end;
                continue;
            }
            if inline_space == FloatInlineSpace::IntrinsicMaximum
                || margin_box_size.inline <= available_inline
                || band.next_transition.is_none()
            {
                let margin_box_inline = match side {
                    FloatLedgerSide::LineStart => band.inline_start,
                    FloatLedgerSide::LineEnd => band.inline_end - margin_box_size.inline,
                };
                let content_origin = LogicalPointOf::new(
                    margin_box_inline + logical_margin.inline_start,
                    candidate_block + logical_margin.block_start,
                );
                // This is the float's own actual collision-height query, not
                // a fabricated height for a zero-height Text line. Each side's
                // raw occupation belongs to this exact source/clear cohort.
                let intrinsic_inline_demand = if inline_space == FloatInlineSpace::IntrinsicMaximum
                {
                    let demand = band.inline_start - self.containing_inline_start
                        + margin_box_size.inline
                        + self.end_occupation(&band);
                    Some(
                        crate::text_layout::run::number::<S>(demand.to_f64()).map_err(|error| {
                            super::inline_run::text_run_error(provider.container, error)
                        })?,
                    )
                } else {
                    None
                };
                self.record_placement(float, content_origin);
                return Ok(FloatPlacement {
                    margin_block_start: candidate_block,
                    intrinsic_inline_demand,
                    logical: content_origin,
                    physical: self.flow_axes.physical_point(
                        content_origin,
                        logical_size,
                        self.containing_size,
                    ),
                });
            }
            candidate_block = band
                .next_transition
                .expect("a retry is entered only with a finite later transition");
        }
    }

    pub(super) fn place_bfc_block<'input, Tree, M>(
        &self,
        provider: ProviderBandContext<'_, Tree, Node>,
        candidate: BfcBandCandidate<S>,
    ) -> LayoutResultOf<Node, BfcBandPlacement<S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        let BfcBandCandidate {
            block_start,
            size,
            margin,
            clear,
            fallback,
            inline_size_can_shrink,
        } = candidate;
        let logical_size = self.flow_axes.logical_size(size);
        let logical_margin = self.flow_axes.logical_edges(margin);
        let margin_box_inline = logical_size.inline + logical_margin.inline_sum();
        let margin_box_block = logical_size.block + logical_margin.block_sum();
        let fallback_logical = self
            .flow_axes
            .logical_point(fallback, size, self.containing_size);
        let mut candidate_block = self.clearance_block(block_start, clear);
        loop {
            let candidate_block_end = candidate_block + margin_box_block;
            let band = if provider.enabled {
                self.query_provider_band(
                    provider.tree,
                    provider.container,
                    candidate_block,
                    candidate_block_end,
                )?
            } else {
                self.query_physical_margin_box_collisions(candidate_block, candidate_block_end)
            };
            let fallback_start = fallback_logical.inline - logical_margin.inline_start;
            let fallback_end =
                fallback_logical.inline + logical_size.inline + logical_margin.inline_end;
            let available_inline = (band.inline_end - band.inline_start).max(S::ZERO);
            let required_inline = if inline_size_can_shrink {
                logical_margin.inline_sum()
            } else {
                margin_box_inline
            };
            if required_inline <= available_inline {
                let inline = if !inline_size_can_shrink
                    && fallback_start >= band.inline_start
                    && fallback_end <= band.inline_end
                {
                    fallback_logical.inline
                } else {
                    band.inline_start + logical_margin.inline_start
                };
                return Ok(BfcBandPlacement {
                    location: self.flow_axes.physical_point(
                        LogicalPointOf::new(inline, candidate_block),
                        logical_size,
                        self.containing_size,
                    ),
                    available_inline,
                });
            }
            if let Some(next_transition) = band.next_transition {
                candidate_block = next_transition;
            } else {
                return Ok(BfcBandPlacement {
                    location: self.flow_axes.physical_point(
                        LogicalPointOf::new(fallback_logical.inline, candidate_block),
                        logical_size,
                        self.containing_size,
                    ),
                    available_inline,
                });
            }
        }
    }

    pub(super) fn clearance_block(&self, block: S, clear: Clear) -> S {
        self.ledger
            .iter()
            .copied()
            .filter(|entry| entry.side.is_cleared_by(clear))
            .map(|entry| entry.block_end)
            .fold(block, S::max)
    }

    pub(super) fn clearance_for_line_intent(&self, block: S, clear: PostLineClearIntent) -> S {
        self.clearance_block(
            block,
            match clear {
                PostLineClearIntent::None => Clear::None,
                PostLineClearIntent::LineStart => Clear::Left,
                PostLineClearIntent::LineEnd => Clear::Right,
                PostLineClearIntent::Both => Clear::Both,
            },
        )
    }

    fn query_physical_margin_box_collisions(&self, block_start: S, block_end: S) -> FloatBand<S> {
        self.query_band_without_provider(
            block_start,
            block_end,
            FloatBandQueryPurpose::PhysicalMarginBoxCollision,
        )
    }

    pub(crate) fn query_rectangular_line_band(&self, block_start: S, block_end: S) -> FloatBand<S> {
        self.query_band_without_provider(
            block_start,
            block_end,
            FloatBandQueryPurpose::RectangularLineBand,
        )
    }

    pub(super) fn query_provider_band<'input, Tree, M>(
        &self,
        tree: &Tree,
        container: Node,
        block_start: S,
        block_end: S,
    ) -> LayoutResultOf<Node, FloatBand<S>, S, M>
    where
        Tree: Compute<'input, M, Node = Node, Scalar = S>,
    {
        self.query_band_for(
            block_start,
            block_end,
            FloatBandQueryPurpose::ProviderBand,
            |subject, expected| {
                let site = LayoutErrorSiteOf::ContainerSubject { container, subject };
                match tree.float_exclusion_interval(subject, expected) {
                    None => Err(LayoutErrorOf::new(
                        site,
                        LayoutOperation::FloatExclusionQuery,
                        LayoutErrorKindOf::MissingContext(
                            LayoutMissingContext::FloatExclusionProvider,
                        ),
                    )),
                    Some(Err(error)) => Err(LayoutErrorOf::new(
                        site,
                        LayoutOperation::FloatExclusionQuery,
                        LayoutErrorKindOf::Measurement(error),
                    )),
                    Some(Ok(None)) => Ok(None),
                    Some(Ok(Some(interval))) => {
                        let actual = interval.originating_query();
                        if actual != expected {
                            return Err(LayoutErrorOf::new(
                                site,
                                LayoutOperation::FloatExclusionQuery,
                                LayoutErrorKindOf::InvalidInput(
                                    LayoutInvalidInputOf::FloatExclusionProviderOutput {
                                        error: FloatExclusionIntervalErrorOf::QueryMismatch {
                                            expected,
                                            actual,
                                        },
                                    },
                                ),
                            ));
                        }
                        Ok(Some(interval))
                    }
                }
            },
        )
    }

    fn query_band_without_provider(
        &self,
        block_start: S,
        block_end: S,
        purpose: FloatBandQueryPurpose,
    ) -> FloatBand<S> {
        let result = self.query_band_for(
            block_start,
            block_end,
            purpose,
            |_, _| -> Result<Option<FloatExclusionIntervalOf<S>>, core::convert::Infallible> {
                unreachable!("provider-free band purposes never request a shape interval")
            },
        );
        match result {
            Ok(band) => band,
            Err(never) => match never {},
        }
    }

    fn query_band_for<E>(
        &self,
        block_start: S,
        block_end: S,
        purpose: FloatBandQueryPurpose,
        mut shape_provider: impl FnMut(
            Node,
            FloatExclusionQueryOf<S>,
        ) -> Result<Option<FloatExclusionIntervalOf<S>>, E>,
    ) -> Result<FloatBand<S>, E> {
        debug_assert!(
            self.ledger
                .windows(2)
                .all(|pair| pair[0].ledger_order <= pair[1].ledger_order),
            "float ledger remains in source order"
        );
        let mut inline_start = self.containing_inline_start;
        let mut inline_end = self.containing_inline_end;
        let mut next_transition = None;
        #[cfg(test)]
        let mut evaluated = 0;

        for entry in self.ledger.iter().copied() {
            #[cfg(test)]
            {
                evaluated += 1;
            }
            debug_assert!(
                entry.physical_margin_box.origin.x.is_finite()
                    && entry.physical_margin_box.origin.y.is_finite()
                    && entry.physical_margin_box.size.width.is_finite()
                    && entry.physical_margin_box.size.height.is_finite(),
                "placed float margin boxes remain finite"
            );
            if !entry.overlaps_block_span(block_start, block_end) {
                continue;
            }
            let logical_interval = match (purpose, entry.exclusion) {
                (FloatBandQueryPurpose::PhysicalMarginBoxCollision, _) => {
                    Some((entry.inline_start, entry.inline_end))
                }
                (FloatBandQueryPurpose::RectangularLineBand, FloatExclusion::MarginBox)
                | (FloatBandQueryPurpose::ProviderBand, FloatExclusion::MarginBox) => {
                    Some((entry.inline_start, entry.inline_end))
                }
                (FloatBandQueryPurpose::RectangularLineBand, FloatExclusion::Shape) => None,
                (FloatBandQueryPurpose::ProviderBand, FloatExclusion::Shape) => {
                    let query = self.provider_query(entry, block_start, block_end);
                    shape_provider(entry.node, query)?
                        .map(|interval| self.logical_inline_interval(interval))
                }
            };
            if let Some((entry_inline_start, entry_inline_end)) = logical_interval {
                match entry.side {
                    FloatLedgerSide::LineStart => {
                        inline_start = inline_start.max(entry_inline_end);
                    }
                    FloatLedgerSide::LineEnd => {
                        inline_end = inline_end.min(entry_inline_start);
                    }
                }
                next_transition = Some(
                    next_transition
                        .map_or(entry.block_end, |current: S| current.min(entry.block_end)),
                );
            }
        }

        Ok(FloatBand {
            inline_start,
            inline_end,
            next_transition,
            #[cfg(test)]
            evaluated,
        })
    }

    fn provider_query(
        &self,
        entry: FloatLedgerEntry<S, Node>,
        block_start: S,
        block_end: S,
    ) -> FloatExclusionQueryOf<S> {
        debug_assert!(block_start.is_finite() && block_end.is_finite());
        debug_assert!(block_start <= block_end);
        let logical_band_size = LogicalSizeOf::new(S::ZERO, block_end - block_start);
        let physical_band_origin = self.flow_axes.physical_point(
            LogicalPointOf::new(S::ZERO, block_start),
            logical_band_size,
            self.containing_size,
        );
        let physical_band_size = self.flow_axes.physical_size(logical_band_size);
        let band_minimum = self.flow_axes.block_axis_coordinate(physical_band_origin);
        let band_maximum = band_minimum + self.flow_axes.block_axis_extent(physical_band_size);
        let margin_box = ScrollRectOf::try_new(
            entry.physical_margin_box.origin,
            entry.physical_margin_box.size,
        )
        .expect("placed float margin boxes satisfy scroll-rectangle invariants");
        FloatExclusionQueryOf::try_new(margin_box, self.flow_axes, band_minimum, band_maximum)
            .expect("canonical finite line bands satisfy provider-query invariants")
    }

    fn logical_inline_interval(&self, interval: FloatExclusionIntervalOf<S>) -> (S, S) {
        let physical_extent = match self.flow_axes.inline_axis() {
            PhysicalAxis::Horizontal => self.containing_size.width,
            PhysicalAxis::Vertical => self.containing_size.height,
        };
        if self
            .flow_axes
            .logical_axis_progression(crate::LogicalAxis::Inline)
            .is_decreasing()
        {
            (
                physical_extent - interval.maximum(),
                physical_extent - interval.minimum(),
            )
        } else {
            (interval.minimum(), interval.maximum())
        }
    }
}

#[cfg(test)]
impl<S: LayoutScalar> FloatExclusions<S> {
    pub(crate) fn record_test_float(
        &mut self,
        side: FloatLedgerSide,
        exclusion: FloatExclusion,
        logical_origin: LogicalPointOf<S>,
        logical_size: LogicalSizeOf<S>,
    ) {
        let origin =
            self.flow_axes
                .physical_point(logical_origin, logical_size, self.containing_size);
        let size = self.flow_axes.physical_size(logical_size);
        let ledger_order = FloatLedgerOrder(self.ledger.len());
        self.ledger.push(FloatLedgerEntry {
            node: (),
            side,
            exclusion,
            physical_margin_box: PhysicalMarginBox { origin, size },
            inline_start: logical_origin.inline,
            inline_end: logical_origin.inline + logical_size.inline,
            block_start: logical_origin.block,
            block_end: logical_origin.block + logical_size.block,
            ledger_order,
        });
    }
}

pub(super) fn layout_floats<'input, Tree, S, M>(
    tree: &mut Tree,
    container: <Tree as Traverse>::Node,
    floats: &[PendingFloat<<Tree as Traverse>::Node, S>],
    container_size: Size<S>,
    constants: &Constants<S>,
    contributions: &mut ScrollContributionAccumulatorOf<S>,
) -> LayoutResultOf<<Tree as Traverse>::Node, (), S, M>
where
    Tree: Compute<'input, M, Scalar = S>,
    S: LayoutScalar,
{
    let logical_container_size = constants.flow_axes.logical_size(container_size);
    let logical_inset = constants.logical_content_box_inset();
    let mut float_exclusions = FloatExclusions::new(
        constants.flow_axes,
        container_size,
        (logical_container_size.inline - logical_inset.inline_sum()).max(S::ZERO),
        logical_inset,
    );

    for float in floats {
        let location = if let Some(accepted) = &float.accepted {
            if accepted.inline_space != FloatInlineSpace::Finite {
                return Err(super::inline_run::text_run_error(
                    container,
                    crate::text_layout::fit::FitError::InvalidGeometry,
                ));
            }
            if accepted.boxes.as_ref().is_some_and(|boxes| {
                boxes.outgoing.boundary != accepted.line.selected_line().request().end
            }) {
                return Err(super::inline_run::text_run_error(
                    container,
                    crate::text_layout::fit::FitError::InvalidGeometry,
                ));
            }
            let (advance, _) = accepted
                .observation
                .observe(&accepted.line)
                .map_err(|error| super::inline_run::text_run_error(container, error))?;
            if advance != accepted.prefix {
                return Err(super::inline_run::text_run_error(
                    container,
                    crate::text_layout::fit::FitError::InvalidGeometry,
                ));
            }
            float_exclusions.record_placement(float, accepted.logical_origin);
            constants.flow_axes.physical_point(
                accepted.logical_origin,
                constants.flow_axes.logical_size(float.size),
                container_size,
            )
        } else {
            float_exclusions.place_float(
                ProviderBandContext {
                    tree,
                    container,
                    enabled: true,
                },
                float,
            )?
        };
        let scroll_geometry = with_block_scroll_projections::<Tree, M, _>(
            tree,
            float.node,
            |box_projection, target_projection| {
                retained_child_scroll_geometry(
                    box_projection,
                    target_projection,
                    float.size,
                    float.content_size,
                    float.padding,
                    float.border,
                    float.child_compute_geometry,
                )
            },
        )
        .map_err(|error| layout_child_geometry_error(container, float.node, error))?;
        contributions
            .include_in_flow_geometry(location, float.margin, scroll_geometry)
            .map_err(|error| layout_child_geometry_error(container, float.node, error))?;
        tree.set_unrounded(
            float.node,
            NodeOutputOf::<S> {
                source_index: crate::SourceIndex::new(float.source_index),
                location,
                size: float.size,
                content_size: float.content_size,
                border: float.border,
                padding: float.padding,
                margin: float.margin,
                ..NodeOutputOf::new()
            }
            .with_scroll_geometry(Some(scroll_geometry)),
        );
    }

    Ok(())
}

pub(super) struct FloatIntrinsics<S: LayoutScalar> {
    available_inline: AvailableOf<S>,
    contribution: S,
    line_start: S,
    line_end: S,
}

impl<S: LayoutScalar> FloatIntrinsics<S> {
    pub(super) const fn new(available_inline: AvailableOf<S>) -> Self {
        Self {
            available_inline,
            contribution: S::ZERO,
            line_start: S::ZERO,
            line_end: S::ZERO,
        }
    }

    pub(super) fn add(&mut self, width: S, float: Float, clear: Clear) {
        match self.available_inline {
            AvailableOf::<S>::Definite(_) => {}
            AvailableOf::<S>::MinContent => self.contribution = self.contribution.max(width),
            AvailableOf::<S>::MaxContent => {
                match clear {
                    Clear::None => {}
                    Clear::Left => self.line_start = S::ZERO,
                    Clear::Right => self.line_end = S::ZERO,
                    Clear::Both => {
                        self.line_start = S::ZERO;
                        self.line_end = S::ZERO;
                    }
                }
                match float {
                    Float::Left | Float::None => self.line_start = self.line_start + width,
                    Float::Right => self.line_end = self.line_end + width,
                }
                self.contribution = self.contribution.max(self.line_start + self.line_end);
            }
        }
    }

    /// Captured Text-line and float-query receipts own Max cohort composition.
    /// Neither a global side sum nor an artificial Text height participates.
    pub(super) fn add_captured(&mut self, width: S, intrinsic_demand: Option<S>) {
        match self.available_inline {
            AvailableOf::Definite(_) => {}
            AvailableOf::MinContent => self.contribution = self.contribution.max(width),
            AvailableOf::MaxContent => {
                self.contribution = self.contribution.max(width);
                if let Some(demand) = intrinsic_demand {
                    self.contribution = self.contribution.max(demand);
                }
            }
        }
    }

    pub(super) const fn result(&self) -> S {
        self.contribution
    }
}
