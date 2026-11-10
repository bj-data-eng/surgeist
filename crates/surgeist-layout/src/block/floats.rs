use super::relative::{self, InsetReference};
use super::scroll::{BlockScrollContributions, retained_child_scroll_geometry};
use super::{Constants, with_block_scroll_projections};
use crate::error::{layout_child_geometry_error, value_resolution_error};
use crate::geometry::{LogicalEdgesOf, LogicalPointOf, LogicalSizeOf, PhysicalAxis};
use crate::inline::PostLineClearIntent;
use crate::output::MeasuredInsetBasisOf;
use crate::text_layout::scope_alignment::{EntryBacking, EntryFunding, ScopeLease};
use crate::value::ResolvedLengthAutoOf;
use crate::{
    AxisSizingRequestOf, Clear, Compute, Edges, Float, FloatExclusion,
    FloatExclusionIntervalErrorOf, FloatExclusionIntervalOf, FloatExclusionQueryOf,
    LayoutErrorKindOf, LayoutErrorOf, LayoutErrorSiteOf, LayoutInvalidInputOf,
    LayoutMissingContext, LayoutOperation, LayoutResultOf, LayoutScalar, LengthResolutionStatus,
    NodeOutputOf, Point, ScrollGeometryOf, ScrollRectOf, Size, Traverse,
};

#[cfg(test)]
#[path = "floats/funding_tests.rs"]
mod funding_tests;

#[derive(Clone)]
pub(super) struct PendingFloat<Node, S: LayoutScalar> {
    pub(super) placement: PendingPlacement<Node, S>,
    pub(super) node: Node,
    pub(super) source_index: usize,
    pub(super) side: Float,
    pub(super) clear: Clear,
    pub(super) block_start: S,
    pub(super) size: Size<S>,
    /// Actual child-local facts for publication, independent of placement.
    pub(super) named_baselines: crate::NamedBaselinesOf<S>,
    pub(super) content_size: Size<S>,
    pub(super) border: Edges<S>,
    pub(super) padding: Edges<S>,
    pub(super) margin: Edges<S>,
    pub(super) float_exclusion: FloatExclusion,
    pub(super) child_compute_geometry: Option<ScrollGeometryOf<S>>,
}

/// Unresolved trials and actual accepted native/Text placements are distinct.
#[derive(Clone)]
pub(super) enum PendingPlacement<Node, S: LayoutScalar> {
    Unresolved,
    Native(Box<NativeFloatPlacement<Node, S>>),
    Text(Box<AcceptedFloatPlacement<S>>),
}
impl<Node, S: LayoutScalar> PendingPlacement<Node, S> {
    pub(super) fn text(&self) -> Option<&AcceptedFloatPlacement<S>> {
        match self {
            Self::Text(receipt) => Some(receipt),
            Self::Unresolved | Self::Native(_) => None,
        }
    }
}
#[derive(Clone)]
struct NativeFloatBasis<Node, S: LayoutScalar> {
    node: Node,
    source_index: usize,
    side: Float,
    clear: Clear,
    block_start: S,
    size: Size<S>,
    content_size: Size<S>,
    border: Edges<S>,
    padding: Edges<S>,
    margin: Edges<S>,
    exclusion: FloatExclusion,
    geometry: Option<ScrollGeometryOf<S>>,
}
impl<Node: Copy + Eq, S: LayoutScalar> NativeFloatBasis<Node, S> {
    fn for_pending(pending: &PendingFloat<Node, S>) -> Self {
        Self {
            node: pending.node,
            source_index: pending.source_index,
            side: pending.side,
            clear: pending.clear,
            block_start: pending.block_start,
            size: pending.size,
            content_size: pending.content_size,
            border: pending.border,
            padding: pending.padding,
            margin: pending.margin,
            exclusion: pending.float_exclusion,
            geometry: pending.child_compute_geometry,
        }
    }
    fn matches(&self, pending: &PendingFloat<Node, S>) -> bool {
        self.node == pending.node
            && self.source_index == pending.source_index
            && self.side == pending.side
            && self.clear == pending.clear
            && self.block_start == pending.block_start
            && self.size == pending.size
            && self.content_size == pending.content_size
            && self.border == pending.border
            && self.padding == pending.padding
            && self.margin == pending.margin
            && self.exclusion == pending.float_exclusion
            && self.geometry == pending.child_compute_geometry
    }
}
#[derive(Clone)]
pub(super) struct NativeFloatPlacement<Node, S: LayoutScalar> {
    basis: NativeFloatBasis<Node, S>,
    logical: LogicalPointOf<S>,
    axes: crate::FlowAxes,
    inline_start: S,
    inline_end: S,
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

/// Distinct authentic receipts: a represented original seam, or the
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
                Ok((observation.advance(), observation.has_in_flow_content()))
            }
            Self::EmptyParagraphEnd { boundary, original } => {
                let selected = line.selected_line();
                let request = selected.request();
                let owner = match original {
                    surgeist_text::TextSeam::Origin { origin, .. } => origin.prepared_text(),
                    surgeist_text::TextSeam::BeforeParticipant(participant)
                    | surgeist_text::TextSeam::AfterParticipant(participant) => match participant {
                        surgeist_text::PreparedParticipant::Text(p) => p.prepared_text(),
                        surgeist_text::PreparedParticipant::Cell(p) => p.prepared_text(),
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

/// The actual ledger placement. Keeping the original logical result avoids
/// reversing a scalar frame transform before final physical publication.
pub(super) struct FloatPlacement<S: LayoutScalar> {
    pub(super) margin_block_start: S,
    pub(super) intrinsic_inline_demand: Option<S>,
    pub(super) logical: LogicalPointOf<S>,
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

#[derive(Clone, Debug)]
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
    fn overlaps_block_span(&self, block_start: S, block_end: S) -> bool {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FloatBandQueryPurpose {
    PhysicalMarginBoxCollision,
    RectangularLineBand,
    ProviderBand,
}

pub(crate) struct FloatExclusions<S: LayoutScalar, Node = ()> {
    flow_axes: crate::geometry::FlowAxes,
    containing_size: Size<S>,
    common_block_origin: S,
    pub(super) containing_inline_start: S,
    pub(super) containing_inline_end: S,
    ledger: Vec<FloatLedgerEntry<S, Node>>,
    backing: EntryBacking,
    funding: Option<EntryFunding<S>>,
}

#[derive(Debug)]
pub(crate) struct InheritedFloatExclusions<S: LayoutScalar, Node> {
    parent_flow_axes: crate::geometry::FlowAxes,
    parent_containing_size: Size<S>,
    parent_common_block_origin: S,
    child_logical_location: LogicalPointOf<S>,
    ledger: Vec<FloatLedgerEntry<S, Node>>,
    _ledger_storage: Option<ScopeLease>,
    _owner_storage: Option<ScopeLease>,
}

/// A prospective entry projection has fixed admitted storage. It cannot use
/// allocator slack as another funded source row or escape as a completed BFC.
pub(in crate::block) struct EntryFloatLedger<S: LayoutScalar, Node> {
    ledger: FloatExclusions<S, Node>,
    admitted: usize,
}

fn entry_visit<Node, S: LayoutScalar, M>(
    node: Node,
    funding: Option<&EntryFunding<S>>,
    count: usize,
) -> LayoutResultOf<Node, (), S, M> {
    funding
        .map_or(Ok(()), |funding| funding.visit(count))
        .map_err(|error| {
            LayoutErrorOf::new(
                LayoutErrorSiteOf::Node(node),
                LayoutOperation::ChildLayout,
                LayoutErrorKindOf::Text(error),
            )
        })
}

fn entry_bound<Node: Copy, S: LayoutScalar, M>(
    node: Node,
    ledger: &[FloatLedgerEntry<S, Node>],
    clear: Clear,
    axes: crate::FlowAxes,
    receiver: crate::FlowAxes,
    origin: S,
    funding: Option<&EntryFunding<S>>,
) -> LayoutResultOf<Node, Option<CommonBfcPosition<S>>, S, M> {
    let clear = if receiver.inline_start() == axes.inline_start() {
        clear
    } else {
        match clear {
            Clear::Left => Clear::Right,
            Clear::Right => Clear::Left,
            value => value,
        }
    };
    let mut bound: Option<S> = None;
    for row in ledger {
        entry_visit(node, funding, 1)?;
        if row.side.is_cleared_by(clear) {
            bound = Some(bound.map_or(row.block_end, |old| old.max(row.block_end)));
        }
    }
    bound
        .map(|bound| {
            let value = origin + bound;
            if value.is_finite() {
                Ok(CommonBfcPosition(value))
            } else {
                Err(crate::error::sizing_resolution_error(
                    node,
                    LengthResolutionStatus::InvalidNumeric { value }.into(),
                ))
            }
        })
        .transpose()
}

impl<S: LayoutScalar, Node: Copy> EntryFloatLedger<S, Node> {
    pub(in crate::block) fn entry_inline_window(&mut self, start: S, end: S) {
        self.ledger.entry_inline_window(start, end);
    }
    pub(in crate::block) fn entry_place_float<M>(
        &mut self,
        node: Node,
        pending: &PendingFloat<Node, S>,
        funding: Option<&EntryFunding<S>>,
    ) -> LayoutResultOf<Node, FloatPlacement<S>, S, M> {
        assert!(
            self.ledger.ledger.len() < self.admitted,
            "a projected source row has admitted backing"
        );
        self.ledger
            .place_float_in_space(node, pending, None, FloatInlineSpace::Finite, funding)
    }
    pub(in crate::block) fn entry_relevant_bound<M>(
        &self,
        node: Node,
        clear: Clear,
        receiver: crate::FlowAxes,
        funding: Option<&EntryFunding<S>>,
    ) -> LayoutResultOf<Node, Option<CommonBfcPosition<S>>, S, M> {
        entry_bound(
            node,
            &self.ledger.ledger,
            clear,
            self.ledger.flow_axes,
            receiver,
            self.ledger.common_block_origin,
            funding,
        )
    }
}

/// A logical block coordinate in the common BFC, independent of physical-axis
/// orientation and of a receiving child's later settled origin.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub(crate) struct CommonBfcPosition<S: LayoutScalar>(S);
impl<S: LayoutScalar> CommonBfcPosition<S> {
    pub(in crate::block) fn get(self) -> S {
        self.0
    }
}

impl<S: LayoutScalar, Node> InheritedFloatExclusions<S, Node> {
    pub(in crate::block) fn common_border(
        &self,
        parent_local: S,
    ) -> Result<CommonBfcPosition<S>, LengthResolutionStatus<S>> {
        let value = self.parent_common_block_origin + parent_local;
        if value.is_finite() {
            Ok(CommonBfcPosition(value))
        } else {
            Err(LengthResolutionStatus::InvalidNumeric { value })
        }
    }

    pub(in crate::block) fn parent_axes(&self) -> crate::FlowAxes {
        self.parent_flow_axes
    }
    pub(in crate::block) fn entry_ledger_funded(
        &self,
        inline_start: S,
        inline_end: S,
        additional: usize,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<(EntryFloatLedger<S, Node>, Option<ScopeLease>)>
    where
        Node: Copy,
    {
        let admitted = self.ledger.len().checked_add(additional).ok_or_else(|| {
            crate::text_layout::scope_alignment::refusal(
                surgeist_text::ScopeAlignmentError::Overflow {
                    kind: surgeist_text::ScopeAlignmentLimitKind::Records,
                },
            )
        })?;
        let lease;
        let mut ledger = if let Some(funding) = funding.filter(|_| admitted != 0) {
            let (ledger, storage) = funding.buffer(admitted)?;
            lease = Some(storage);
            ledger
        } else {
            let mut ledger = Vec::new();
            ledger.try_reserve_exact(admitted).map_err(|_| {
                crate::text_layout::scope_alignment::refusal(
                    surgeist_text::ScopeAlignmentError::Allocation {
                        kind: surgeist_text::ScopeAlignmentLimitKind::PayloadBytes,
                    },
                )
            })?;
            lease = None;
            ledger
        };
        for row in &self.ledger {
            if let Some(funding) = funding {
                funding.visit(1)?;
            }
            ledger.push(row.clone());
        }
        Ok((
            EntryFloatLedger {
                admitted,
                ledger: FloatExclusions {
                    flow_axes: self.parent_flow_axes,
                    containing_size: self.parent_containing_size,
                    common_block_origin: self.parent_common_block_origin,
                    containing_inline_start: inline_start,
                    containing_inline_end: inline_end,
                    ledger,
                    backing: EntryBacking::default(),
                    funding: None,
                },
            },
            lease,
        ))
    }

    pub(in crate::block) fn entry_relevant_bound<M>(
        &self,
        node: Node,
        clear: Clear,
        receiver: crate::FlowAxes,
        funding: Option<&EntryFunding<S>>,
    ) -> LayoutResultOf<Node, Option<CommonBfcPosition<S>>, S, M>
    where
        Node: Copy,
    {
        entry_bound(
            node,
            &self.ledger,
            clear,
            self.parent_flow_axes,
            receiver,
            self.parent_common_block_origin,
            funding,
        )
    }
}

impl<S: LayoutScalar, Node: Copy> FloatExclusions<S, Node> {
    pub(in crate::block) fn common_border(
        &self,
        local: S,
    ) -> Result<CommonBfcPosition<S>, LengthResolutionStatus<S>> {
        let value = self.common_block_origin + local;
        if value.is_finite() {
            Ok(CommonBfcPosition(value))
        } else {
            Err(LengthResolutionStatus::InvalidNumeric { value })
        }
    }
    pub(in crate::block) fn entry_relevant_bound<M>(
        &self,
        node: Node,
        clear: Clear,
        receiver: crate::FlowAxes,
        funding: Option<&EntryFunding<S>>,
    ) -> LayoutResultOf<Node, Option<CommonBfcPosition<S>>, S, M> {
        entry_bound(
            node,
            &self.ledger,
            clear,
            self.flow_axes,
            receiver,
            self.common_block_origin,
            funding,
        )
    }
    pub(crate) fn new(
        flow_axes: crate::geometry::FlowAxes,
        containing_size: Size<S>,
        content_inline_size: S,
        inset: LogicalEdgesOf<S>,
    ) -> Self {
        Self {
            flow_axes,
            containing_size,
            common_block_origin: S::ZERO,
            containing_inline_start: inset.inline_start,
            containing_inline_end: inset.inline_start + content_inline_size,
            ledger: Vec::new(),
            backing: EntryBacking::default(),
            funding: None,
        }
    }

    pub(super) fn fund_entry(&mut self, funding: Option<&EntryFunding<S>>) {
        debug_assert!(self.ledger.is_empty());
        self.funding = funding.cloned();
    }

    pub(super) fn try_clone(&self) -> surgeist_text::Result<Self> {
        let mut result = Self {
            flow_axes: self.flow_axes,
            containing_size: self.containing_size,
            common_block_origin: self.common_block_origin,
            containing_inline_start: self.containing_inline_start,
            containing_inline_end: self.containing_inline_end,
            ledger: Vec::new(),
            backing: EntryBacking::default(),
            funding: self.funding.clone(),
        };
        if self.funding.is_none() {
            result.ledger = self.ledger.clone();
            return Ok(result);
        }
        for row in &self.ledger {
            if let Some(funding) = &self.funding {
                funding.visit(1)?;
            }
            result
                .backing
                .reserve_push(&mut result.ledger, result.funding.as_ref())?;
            result.ledger.push(row.clone());
        }
        Ok(result)
    }

    pub(in crate::block) fn entry_inline_window(&mut self, start: S, end: S) {
        self.containing_inline_start = start;
        self.containing_inline_end = end;
    }

    /// Rebase this actual influencing float ledger to the preceding canonical
    /// pass. Inherited entries keep their node and side provenance.
    pub(super) fn canonical_for_text(
        &self,
        container: Node,
        frames: &[crate::text_layout::state::CanonicalFrame<Node, S>],
    ) -> Result<Option<Self>, crate::text_layout::fit::FitError>
    where
        Node: Eq,
    {
        let frame_for = |node| -> Result<
            Option<&crate::text_layout::state::CanonicalFrame<Node, S>>,
            crate::text_layout::fit::FitError,
        > {
            for frame in frames {
                if let Some(funding) = &self.funding {
                    funding.visit(1)?;
                }
                if frame.node == node {
                    return Ok(Some(frame));
                }
            }
            Ok(None)
        };
        let Some(container) = frame_for(container)? else {
            return Ok(None);
        };
        let content = container
            .normal
            .content_box()
            .ok_or(crate::text_layout::fit::FitError::InvalidGeometry)?;
        let logical_content_origin =
            self.flow_axes
                .logical_point(content.origin(), content.size(), container.normal.size);
        let logical_content = self.flow_axes.logical_size(content.size());
        let mut result = self.try_clone()?;
        result.containing_size = container.normal.size;
        result.containing_inline_start = logical_content_origin.inline;
        result.containing_inline_end = logical_content_origin.inline + logical_content.inline;
        for entry in &mut result.ledger {
            if let Some(funding) = &self.funding {
                funding.visit(1)?;
            }
            let frame =
                frame_for(entry.node)?.ok_or(crate::text_layout::fit::FitError::InvalidGeometry)?;
            let margin = frame.output.margin;
            let origin = Point::new(
                crate::text_layout::run::number::<S>(
                    frame.normal_origin.x - container.normal_origin.x - margin.left.to_f64(),
                )?,
                crate::text_layout::run::number::<S>(
                    frame.normal_origin.y - container.normal_origin.y - margin.top.to_f64(),
                )?,
            );
            let size = Size::new(
                crate::text_layout::run::number::<S>(
                    frame.normal.size.width.to_f64() + margin.left.to_f64() + margin.right.to_f64(),
                )?,
                crate::text_layout::run::number::<S>(
                    frame.normal.size.height.to_f64()
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
            parent_common_block_origin: self.common_block_origin,
            child_logical_location,
            ledger: self.ledger.clone(),
            _ledger_storage: None,
            _owner_storage: None,
        }
    }

    /// The frame's own Arc retains the two allocation leases. Cloning an
    /// Incoming handle therefore pins the actual shared backing only once.
    pub(super) fn for_entry_child(
        &self,
        location: LogicalPointOf<S>,
        funding: Option<&EntryFunding<S>>,
    ) -> surgeist_text::Result<InheritedFloatExclusions<S, Node>> {
        let Some(funding) = funding else {
            return Ok(self.for_ordinary_child(location));
        };
        let owner = funding.header::<(usize, usize, InheritedFloatExclusions<S, Node>)>()?;
        let storage;
        let ledger = if self.ledger.is_empty() {
            storage = None;
            Vec::new()
        } else {
            let (rows, lease) = funding.buffer(self.ledger.len())?;
            storage = Some(lease);
            rows
        };
        let mut frame = InheritedFloatExclusions {
            parent_flow_axes: self.flow_axes,
            parent_containing_size: self.containing_size,
            parent_common_block_origin: self.common_block_origin,
            child_logical_location: location,
            ledger,
            _ledger_storage: storage,
            _owner_storage: Some(owner),
        };
        for row in &self.ledger {
            funding.visit(1)?;
            frame.ledger.push(row.clone());
        }
        Ok(frame)
    }

    pub(super) fn inherit_into_child(
        &mut self,
        inherited: &InheritedFloatExclusions<S, Node>,
    ) -> surgeist_text::Result<()> {
        self.common_block_origin =
            inherited.parent_common_block_origin + inherited.child_logical_location.block;
        let child_size = self.containing_size;
        let child_logical_size = inherited.parent_flow_axes.logical_size(child_size);
        let child_physical_origin = inherited.parent_flow_axes.physical_point(
            inherited.child_logical_location,
            child_logical_size,
            inherited.parent_containing_size,
        );
        for (inherited_order, entry) in (self.ledger.len()..).zip(&inherited.ledger) {
            if let Some(funding) = &self.funding {
                funding.visit(1)?;
            }
            self.backing
                .reserve_push(&mut self.ledger, self.funding.as_ref())?;
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
            self.ledger.push(FloatLedgerEntry {
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
                exclusion: entry.exclusion,
            });
        }
        Ok(())
    }

    pub(super) fn retain_native(
        &self,
        pending: &mut PendingFloat<Node, S>,
        placement: FloatPlacement<S>,
    ) where
        Node: Eq,
    {
        pending.placement = PendingPlacement::Native(Box::new(NativeFloatPlacement {
            basis: NativeFloatBasis::for_pending(pending),
            logical: placement.logical,
            axes: self.flow_axes,
            inline_start: self.containing_inline_start,
            inline_end: self.containing_inline_end,
        }));
    }

    pub(super) fn len(&self) -> usize {
        self.ledger.len()
    }
    pub(super) fn bottom(&self) -> S {
        self.ledger
            .iter()
            .map(|entry| entry.block_end)
            .fold(S::ZERO, S::max)
    }
    pub(super) fn import_child(
        &mut self,
        child: &super::float_context::Completed<Node, S>,
        location: LogicalPointOf<S>,
    ) -> surgeist_text::Result<bool>
    where
        Node: Eq,
    {
        let Some(incoming) = &child.frame else {
            return Ok(false);
        };
        if incoming.parent_flow_axes != self.flow_axes
            || incoming.parent_containing_size != self.containing_size
            || incoming.child_logical_location != location
            || child.inherited != incoming.ledger.len()
            || child.ledger.ledger.len() < child.inherited
        {
            return Ok(false);
        }
        for (left, right) in child
            .ledger
            .ledger
            .iter()
            .take(child.inherited)
            .zip(&incoming.ledger)
        {
            if let Some(funding) = &self.funding {
                funding.visit(1)?;
            }
            if left.node != right.node || left.exclusion != right.exclusion {
                return Ok(false);
            }
        }
        if incoming.ledger.len() != self.ledger.len() {
            return Ok(false);
        }
        for (left, right) in incoming.ledger.iter().zip(&self.ledger) {
            if let Some(funding) = &self.funding {
                funding.visit(1)?;
            }
            if left.node != right.node
                || left.side != right.side
                || left.ledger_order != right.ledger_order
            {
                return Ok(false);
            }
        }
        let child_origin = self.flow_axes.physical_point(
            location,
            self.flow_axes.logical_size(child.output.size),
            self.containing_size,
        );
        for entry in child.ledger.ledger.iter().skip(child.inherited) {
            if let Some(funding) = &self.funding {
                funding.visit(1)?;
            }
            self.backing
                .reserve_push(&mut self.ledger, self.funding.as_ref())?;
            let mut entry = entry.clone();
            let physical_origin = Point::new(
                entry.physical_margin_box.origin.x + child_origin.x,
                entry.physical_margin_box.origin.y + child_origin.y,
            );
            let size = entry.physical_margin_box.size;
            let origin = self
                .flow_axes
                .logical_point(physical_origin, size, self.containing_size);
            let logical_size = self.flow_axes.logical_size(size);
            let physical_side = match entry.side {
                FloatLedgerSide::LineStart => child.ledger.flow_axes.inline_start(),
                FloatLedgerSide::LineEnd => child.ledger.flow_axes.inline_end(),
            };
            entry.side = if physical_side == self.flow_axes.inline_start() {
                FloatLedgerSide::LineStart
            } else {
                FloatLedgerSide::LineEnd
            };
            entry.physical_margin_box.origin = physical_origin;
            entry.inline_start = origin.inline;
            entry.inline_end = origin.inline + logical_size.inline;
            entry.block_start = origin.block;
            entry.block_end = origin.block + logical_size.block;
            entry.ledger_order = FloatLedgerOrder(self.ledger.len());
            self.ledger.push(entry);
        }
        Ok(true)
    }

    /// Record an already accepted border-box origin in this exact ledger frame.
    /// The same operation serves ordinary placement and publication of a plan
    /// that was accepted with its real preceding-inline constraint.
    fn record_placement(
        &mut self,
        float: &PendingFloat<Node, S>,
        border: LogicalPointOf<S>,
    ) -> surgeist_text::Result<()> {
        self.backing
            .reserve_push(&mut self.ledger, self.funding.as_ref())?;
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
        Ok(())
    }

    pub(super) fn place_float_for_line<M>(
        &mut self,
        container: Node,
        float: &PendingFloat<Node, S>,
        line: Option<(f64, S)>,
    ) -> LayoutResultOf<Node, FloatPlacement<S>, S, M> {
        self.place_float_in_space(container, float, line, FloatInlineSpace::Finite, None)
    }

    pub(super) fn place_float_intrinsic<M>(
        &mut self,
        container: Node,
        float: &PendingFloat<Node, S>,
    ) -> LayoutResultOf<Node, FloatPlacement<S>, S, M> {
        self.place_float_in_space(
            container,
            float,
            None,
            FloatInlineSpace::IntrinsicMaximum,
            None,
        )
    }

    /// End-side occupation from the same raw, height-qualified ledger query.
    /// Reversed intrinsic frontiers remain meaningful independent distances.
    pub(super) fn end_occupation(&self, band: &FloatBand<S>) -> S {
        self.containing_inline_end - band.inline_end
    }

    fn place_float_in_space<M>(
        &mut self,
        container: Node,
        float: &PendingFloat<Node, S>,
        line: Option<(f64, S)>,
        inline_space: FloatInlineSpace,
        funding: Option<&EntryFunding<S>>,
    ) -> LayoutResultOf<Node, FloatPlacement<S>, S, M> {
        let owned_funding = self.funding.clone();
        let funding = owned_funding.as_ref().or(funding);
        let side = FloatLedgerSide::from_float(float.side);
        let logical_size = self.flow_axes.logical_size(float.size);
        let logical_margin = self.flow_axes.logical_edges(float.margin);
        let margin_box_size = LogicalSizeOf::new(
            logical_size.inline + logical_margin.inline_sum(),
            logical_size.block + logical_margin.block_sum(),
        );
        let mut preceding: Option<S> = None;
        for row in &self.ledger {
            entry_visit(container, funding, 1)?;
            preceding = Some(preceding.map_or(row.block_start, |old| old.max(row.block_start)));
        }
        let source_block = preceding.map_or(float.block_start, |top| float.block_start.max(top));
        let mut candidate_block = source_block;
        for row in &self.ledger {
            entry_visit(container, funding, 1)?;
            if row.side.is_cleared_by(float.clear) {
                candidate_block = candidate_block.max(row.block_end);
            }
        }

        loop {
            let candidate_block_end = candidate_block + margin_box_size.block;
            let band = self.query_band_for(
                candidate_block,
                candidate_block_end,
                FloatBandQueryPurpose::PhysicalMarginBoxCollision,
                |_, _| -> LayoutResultOf<Node, Option<FloatExclusionIntervalOf<S>>, S, M> {
                    unreachable!("physical float placement never requests a shape contour")
                },
                || entry_visit(container, funding, 1),
            )?;
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
                        crate::text_layout::run::number::<S>(demand.to_f64())
                            .map_err(|error| super::inline_run::text_run_error(container, error))?,
                    )
                } else {
                    None
                };
                entry_visit(container, funding, 1)?;
                self.record_placement(float, content_origin)
                    .map_err(|error| super::inline_run::text_run_error(container, error.into()))?;
                return Ok(FloatPlacement {
                    margin_block_start: candidate_block,
                    intrinsic_inline_demand,
                    logical: content_origin,
                });
            }
            candidate_block = band
                .next_transition
                .expect("a retry is entered only with a finite later transition");
        }
    }

    fn rectangular_bfc_exclusions(&self) -> impl Iterator<Item = &FloatLedgerEntry<S, Node>> {
        self.ledger.iter().filter(|entry| {
            let empty_at_edge = entry.inline_start == entry.inline_end
                && (entry.inline_start <= self.containing_inline_start
                    || entry.inline_end >= self.containing_inline_end);
            entry.block_start < entry.block_end && !empty_at_edge
        })
    }

    pub(super) fn rectangular_bfc_opportunity(
        &self,
        block_start: S,
        margin: LogicalEdgesOf<ResolvedLengthAutoOf<S>>,
    ) -> (S, Option<S>) {
        let next = self
            .rectangular_bfc_exclusions()
            .flat_map(|entry| [entry.block_start, entry.block_end])
            .filter(|edge| *edge > block_start)
            .reduce(S::min);
        let (start, end, _) =
            self.rectangular_bfc_border_bounds(block_start, next.unwrap_or(block_start), margin);
        ((end - start).max(S::ZERO), next)
    }

    pub(super) fn rectangular_bfc_position(
        &self,
        block_start: S,
        logical_size: LogicalSizeOf<S>,
        margin: LogicalEdgesOf<ResolvedLengthAutoOf<S>>,
        fallback_inline: S,
    ) -> Option<LogicalPointOf<S>> {
        let (first, end, constrained) = self.rectangular_bfc_border_bounds(
            block_start,
            block_start + logical_size.block,
            margin,
        );
        if !constrained {
            return Some(LogicalPointOf::new(fallback_inline, block_start));
        }

        let last = end - logical_size.inline;
        if first > last {
            return None;
        }
        let start_auto = matches!(margin.inline_start, ResolvedLengthAutoOf::Auto);
        let end_auto = matches!(margin.inline_end, ResolvedLengthAutoOf::Auto);
        let inline = if start_auto {
            let count = 1 + usize::from(end_auto);
            first + (last - first) / S::from_usize(count)
        } else if !end_auto && fallback_inline >= first && fallback_inline <= last {
            fallback_inline
        } else {
            first
        };
        Some(LogicalPointOf::new(inline, block_start))
    }

    fn rectangular_bfc_border_bounds(
        &self,
        block_start: S,
        block_end: S,
        margin: LogicalEdgesOf<ResolvedLengthAutoOf<S>>,
    ) -> (S, S, bool) {
        let (left, right) = self.rectangular_bfc_frontiers(block_start, block_end);
        let margin = margin.map(super::in_flow::resolved_length_auto_fallback_zero);
        let start = self.containing_inline_start + margin.inline_start;
        let end = self.containing_inline_end - margin.inline_end;
        (
            left.map_or(start, |edge| start.max(edge)),
            right.map_or(end, |edge| end.min(edge)),
            left.is_some() || right.is_some(),
        )
    }

    fn rectangular_bfc_frontiers(&self, block_start: S, block_end: S) -> (Option<S>, Option<S>) {
        let mut left = None;
        let mut right = None;
        for entry in self.rectangular_bfc_exclusions() {
            if entry.overlaps_block_span(block_start, block_end) {
                match entry.side {
                    FloatLedgerSide::LineStart => {
                        left = Some(
                            left.map_or(entry.inline_end, |edge: S| edge.max(entry.inline_end)),
                        );
                    }
                    FloatLedgerSide::LineEnd => {
                        right =
                            Some(right.map_or(entry.inline_start, |edge: S| {
                                edge.min(entry.inline_start)
                            }));
                    }
                }
            }
        }
        (left, right)
    }

    pub(super) fn place_bfc_block(
        &self,
        block_start: S,
        size: LogicalSizeOf<S>,
        margin: LogicalEdgesOf<S>,
        clear: Clear,
        fallback_inline: S,
    ) -> LogicalPointOf<S> {
        let margin_box_inline = size.inline + margin.inline_sum();
        let margin_box_block = size.block + margin.block_sum();
        let mut candidate_block = self.clearance_block(block_start, clear);
        loop {
            let band = self.query_physical_margin_box_collisions(
                candidate_block,
                candidate_block + margin_box_block,
            );
            let fallback_start = fallback_inline - margin.inline_start;
            let fallback_end = fallback_inline + size.inline + margin.inline_end;
            if margin_box_inline <= (band.inline_end - band.inline_start).max(S::ZERO) {
                let inline =
                    if fallback_start >= band.inline_start && fallback_end <= band.inline_end {
                        fallback_inline
                    } else {
                        band.inline_start + margin.inline_start
                    };
                return LogicalPointOf::new(inline, candidate_block);
            }
            if let Some(next_transition) = band.next_transition {
                candidate_block = next_transition;
            } else {
                return LogicalPointOf::new(fallback_inline, candidate_block);
            }
        }
    }

    pub(super) fn clearance_block(&self, block: S, clear: Clear) -> S {
        self.ledger
            .iter()
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

    pub(crate) fn query_rectangular_line_band(
        &self,
        block_start: S,
        block_end: S,
    ) -> surgeist_text::Result<FloatBand<S>> {
        self.query_band_for(
            block_start,
            block_end,
            FloatBandQueryPurpose::RectangularLineBand,
            |_, _| unreachable!("rectangular line bands never request a shape contour"),
            || {
                self.funding
                    .as_ref()
                    .map_or(Ok(()), |funding| funding.visit(1))
            },
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
            || entry_visit(container, self.funding.as_ref(), 1),
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
            || Ok(()),
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
        mut inspect: impl FnMut() -> Result<(), E>,
    ) -> Result<FloatBand<S>, E> {
        #[cfg(debug_assertions)]
        for pair in self.ledger.windows(2) {
            inspect()?;
            debug_assert!(
                pair[0].ledger_order <= pair[1].ledger_order,
                "float ledger remains in source order"
            );
        }
        let mut inline_start = self.containing_inline_start;
        let mut inline_end = self.containing_inline_end;
        let mut next_transition = None;
        #[cfg(test)]
        let mut evaluated = 0;

        for entry in &self.ledger {
            inspect()?;
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
        entry: &FloatLedgerEntry<S, Node>,
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
    inset_basis: MeasuredInsetBasisOf<S>,
    constants: &Constants<S>,
    contributions: &mut BlockScrollContributions<S>,
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
        let normal_location = if let Some(accepted) = float.placement.text() {
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
            float_exclusions
                .record_placement(float, accepted.logical_origin)
                .map_err(|error| super::inline_run::text_run_error(container, error.into()))?;
            constants.flow_axes.physical_point(
                accepted.logical_origin,
                constants.flow_axes.logical_size(float.size),
                container_size,
            )
        } else {
            let PendingPlacement::Native(receipt) = &float.placement else {
                return Err(super::float_context::missing(float.node));
            };
            let frame_matches = receipt.axes == constants.flow_axes
                && receipt.inline_start == float_exclusions.containing_inline_start
                && receipt.inline_end == float_exclusions.containing_inline_end;
            if !receipt.basis.matches(float) || !frame_matches {
                return Err(super::float_context::missing(float.node));
            }
            float_exclusions
                .record_placement(float, receipt.logical)
                .map_err(|error| super::inline_run::text_run_error(container, error.into()))?;
            // Block auto size may settle after placement. The accepted logical
            // position belongs to the unchanged inline frame; project it once
            // against the final block extent, including decreasing block flow.
            constants.flow_axes.physical_point(
                receipt.logical,
                constants.flow_axes.logical_size(float.size),
                container_size,
            )
        };
        let offset = relative::resolve(
            tree,
            float.node,
            tree.node_input(float.node).inset,
            InsetReference::Accepted(Some(inset_basis)),
            constants.flow_axes,
        )?;
        let visual_location =
            Point::new(normal_location.x + offset.x, normal_location.y + offset.y);
        for value in [visual_location.x, visual_location.y] {
            if !value.is_finite() {
                return Err(value_resolution_error(
                    float.node,
                    LengthResolutionStatus::InvalidNumeric { value },
                ));
            }
        }
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
            .include_in_flow_geometry(visual_location, float.margin, scroll_geometry)
            .map_err(|error| layout_child_geometry_error(container, float.node, error))?;
        tree.set_unrounded(crate::LayoutOutputEntryOf::ordinary_unrounded(
            float.node,
            crate::output::NormalGeometryOf::box_geometry(
                normal_location,
                float.size,
                scroll_geometry.content_box(),
            ),
            NodeOutputOf::<S> {
                source_index: crate::SourceIndex::new(float.source_index),
                location: visual_location,
                size: float.size,
                named_baselines: float.named_baselines,
                content_size: float.content_size,
                border: float.border,
                padding: float.padding,
                margin: float.margin,
                ..NodeOutputOf::new()
            }
            .with_scroll_geometry(Some(scroll_geometry)),
        ));
    }

    Ok(())
}

pub(super) struct FloatIntrinsics<S: LayoutScalar> {
    available_inline: AxisSizingRequestOf<S>,
    contribution: S,
    line_start: S,
    line_end: S,
}

impl<S: LayoutScalar> FloatIntrinsics<S> {
    pub(super) const fn new(available_inline: AxisSizingRequestOf<S>) -> Self {
        Self {
            available_inline,
            contribution: S::ZERO,
            line_start: S::ZERO,
            line_end: S::ZERO,
        }
    }

    pub(super) fn add(&mut self, width: S, float: Float, clear: Clear) {
        match (
            self.available_inline.operation(),
            self.available_inline.space(),
        ) {
            (crate::SizingOperation::Normal, crate::AvailableSpaceOf::Definite(_)) => {}
            (crate::SizingOperation::MinContent, _) => {
                self.contribution = self.contribution.max(width)
            }
            (crate::SizingOperation::MaxContent, _)
            | (crate::SizingOperation::Normal, crate::AvailableSpaceOf::Indefinite) => {
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
        match (
            self.available_inline.operation(),
            self.available_inline.space(),
        ) {
            (crate::SizingOperation::Normal, crate::AvailableSpaceOf::Definite(_)) => {}
            (crate::SizingOperation::MinContent, _) => {
                self.contribution = self.contribution.max(width)
            }
            (crate::SizingOperation::MaxContent, _)
            | (crate::SizingOperation::Normal, crate::AvailableSpaceOf::Indefinite) => {
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
