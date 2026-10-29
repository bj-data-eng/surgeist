//! Borrowed observations of the actual preparation and finalized-line owners.
//!
//! These views do not create a placed layout, copy glyph packets, or resolve a
//! font again. Coordinates are line-local until the placement owner admits an
//! actual physical anchor.

use std::{fmt, sync::Arc};

#[cfg(test)]
#[path = "readback/spacing_tests.rs"]
mod spacing_tests;

use crate::engine::{
    PreparedData,
    layout::data::{FinalizedFragmentData, FinalizedGlyph, FinalizedRun, PreparedRun},
};
use crate::{
    BidiLevel, Brush, CapturedControl, CapturedEventPosition, CapturedParagraph, CapturedScope,
    EffectiveFontInstance, FontFace, InlineBox, Point, ProcessedPosition, ProcessedRange, Range,
    Rect, Result, ScopeRole, Source, SourcePosition, SourceProcessingEvent, Text, TextNumericError,
    TextNumericPhase,
};

use super::{
    FinalizedFragment, FinalizedLine, LineSelectionRequest, PreparedAlternative, PreparedAtomic,
    PreparedBoundary, PreparedComputedLineBoundary, PreparedControl, PreparedOpportunity,
    PreparedOrigin, PreparedParticipant, PreparedScope, PreparedText, PreparedUnit,
    PreparedVirtualSeparator, PresentationRange, SelectedLine, SpacingSlot, SpacingSlotFeedback,
    SpacingSlotOwner,
    paragraph::{FragmentOwner, OriginIntersection, ParagraphControlKind, ParagraphOriginKind},
    prepared::{
        AlternativeData, AlternativeDefinition, AtomicData, BoundaryData, ComputedBoundaryData,
        ControlData, OpportunityData, ParticipantData, PreparedBreakKind, SelectedMappingKind,
        SelectedMappingPiece, UnitData,
    },
};

fn spacing_piece(
    selected: &Arc<super::selection::SelectedData>,
    piece: &super::selection::SpacingSlotPieceData,
) -> super::SpacingSlotPiece {
    let prepared = &selected.prepared;
    let owner = match piece.owner {
        crate::SpacingPiece::Atomic { occurrence } => SpacingSlotOwner::Atomic(PreparedAtomic {
            prepared: Arc::clone(prepared),
            index: prepared
                .tables
                .atomics
                .iter()
                .position(|a| a.occurrence == occurrence)
                .expect("actual atomic inventory"),
        }),
        crate::SpacingPiece::Source {
            origin,
            local,
            aggregate,
        } => SpacingSlotOwner::Source {
            origin: PreparedOrigin {
                prepared: Arc::clone(prepared),
                index: origin,
            },
            local,
            aggregate,
        },
        crate::SpacingPiece::Alternative {
            alternative, local, ..
        } => SpacingSlotOwner::Alternative {
            alternative: PreparedAlternative {
                prepared: Arc::clone(prepared),
                index: alternative,
            },
            local,
        },
        crate::SpacingPiece::Control { occurrence } => SpacingSlotOwner::Control(PreparedControl {
            prepared: Arc::clone(prepared),
            index: control_index(prepared, occurrence),
        }),
        crate::SpacingPiece::VirtualSeparator { occurrence } => {
            SpacingSlotOwner::VirtualSeparator(PreparedVirtualSeparator {
                prepared: Arc::clone(prepared),
                index: occurrence,
            })
        }
    };
    super::SpacingSlotPiece {
        owner,
        presentation: piece.presentation,
    }
}

impl SpacingSlot {
    /// Exact source, alternative, generated and atomic custody in target order.
    pub fn pieces(&self) -> impl ExactSizeIterator<Item = super::SpacingSlotPiece> + '_ {
        self.selected.spacing_slots[self.index]
            .pieces
            .iter()
            .map(|piece| spacing_piece(&self.selected, piece))
    }
}

impl super::SpacingUnit {
    /// Actual governing full values before neighboring-side averaging.
    #[must_use]
    pub fn letter_spacing_before(&self) -> f64 {
        self.selected.spacing_slots[self.slot]
            .target
            .unit(self.side)
            .left
    }
    #[must_use]
    pub fn letter_spacing_after(&self) -> f64 {
        self.selected.spacing_slots[self.slot]
            .target
            .unit(self.side)
            .right
    }
    pub fn pieces(&self) -> impl ExactSizeIterator<Item = super::SpacingSlotPiece> + '_ {
        let range = self.selected.spacing_slots[self.slot]
            .target
            .unit(self.side)
            .pieces
            .clone();
        self.selected.spacing_slots[self.slot].pieces[range]
            .iter()
            .map(|piece| spacing_piece(&self.selected, piece))
    }
    pub fn participants(&self) -> impl Iterator<Item = &PreparedParticipant> {
        self.selected.spacing_slots[self.slot]
            .target
            .unit(self.side)
            .participants
            .iter()
            .map(|&i| &self.selected.request.participants[i])
    }
}

impl FinalizedLine {
    /// Whether this exact selected line retains actual in-flow content.
    ///
    /// Retained original or computed text counts even at zero or signed advance,
    /// as do genuine in-flow atomics with zero geometry and positive inline
    /// boundary advances. Hard separators, control struts, discarded text,
    /// empty anchors and out-of-flow
    /// roles do not independently count. This is neither visibility nor the
    /// prefix API's broader retained-flow participation observation.
    #[must_use]
    pub fn has_in_flow_content(&self) -> bool {
        self.inner.data.prefix.has_in_flow_content
    }

    /// Issues semantic nonformatted eligibility without changing finalization.
    /// The consumer must additionally establish its actual box/decorations
    /// disposition; absent glyph ink or zero numerical advance is insufficient.
    pub fn nonformatted(&self) -> Result<super::NonFormattedLine> {
        let request = &self.inner.selected.request;
        if request.context.end == super::LineEnd::Forced
            || matches!(&request.termination, super::PreparedTermination::Ordinary(offer)
                if offer.view().kind() == PreparedBreakKind::Forced)
        {
            return Err(super::protocol_error(
                super::ProtocolError::NonFormattedForcedLine,
            ));
        }
        if self.has_in_flow_content() {
            return Err(super::protocol_error(
                super::ProtocolError::NonFormattedInFlowContent,
            ));
        }
        Ok(super::NonFormattedLine { line: self.clone() })
    }

    /// Actual feedback for every issued selected slot, in that same order.
    /// Discarded slots remain present with zero advance. Iteration clones only
    /// the issued slot capability; it does not allocate an inventory or glyphs.
    pub fn spacing_slots(&self) -> impl ExactSizeIterator<Item = SpacingSlotFeedback> + '_ {
        self.inner
            .data
            .spacing_slots
            .iter()
            .enumerate()
            .map(|(index, row)| SpacingSlotFeedback {
                slot: SpacingSlot {
                    selected: Arc::clone(&self.inner.selected),
                    index,
                },
                disposition: row.disposition,
                advance: row.advance,
                geometry: row.geometry,
                stretchability: row.stretchability,
                traversal_order: row.traversal_order,
                authored_word_spacing: row.authored,
                authored_letter_spacing: row.letter,
                allocation: row.allocation,
            })
    }
}

fn processed_range(prepared: &PreparedData, range: Range) -> ProcessedRange {
    prepared
        .analyzed
        .processed
        .range(range.start, range.end)
        .expect("issued correspondence has validated processed bounds")
}

fn processed_position(prepared: &PreparedData, position: usize) -> ProcessedPosition {
    prepared
        .analyzed
        .processed
        .position(position)
        .expect("issued correspondence has a validated processed endpoint")
}

fn control_index(prepared: &PreparedData, occurrence: usize) -> usize {
    prepared.tables.control_rows[occurrence]
}

fn resolved_level(level: u8) -> BidiLevel {
    BidiLevel::try_new(level).expect("analysis issued a valid resolved bidi level")
}

macro_rules! prepared_view {
    ($view:ident, $handle:ident, $field:ident, $row:ty) => {
        /// A borrowed row from the actual immutable preparation.
        pub struct $view<'a> {
            prepared: &'a Arc<PreparedData>,
            index: usize,
        }

        impl fmt::Debug for $view<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.debug_struct(stringify!($view))
                    .field("ordinal", &self.index)
                    .finish_non_exhaustive()
            }
        }

        impl $view<'_> {
            fn data(&self) -> &$row {
                &self.prepared.tables.$field[self.index]
            }

            /// Retains this exact issued row and its preparation allocation.
            #[must_use]
            pub fn handle(&self) -> $handle {
                $handle {
                    prepared: Arc::clone(self.prepared),
                    index: self.index,
                }
            }
        }

        impl $handle {
            /// Borrows this handle's actual issued row without allocating.
            #[must_use]
            pub fn view(&self) -> $view<'_> {
                $view {
                    prepared: &self.prepared,
                    index: self.index,
                }
            }
        }
    };
}

prepared_view!(PreparedUnitView, PreparedUnit, units, UnitData);
prepared_view!(PreparedAtomicView, PreparedAtomic, atomics, AtomicData);
prepared_view!(PreparedControlView, PreparedControl, controls, ControlData);
prepared_view!(
    PreparedComputedLineBoundaryView,
    PreparedComputedLineBoundary,
    computed_boundaries,
    ComputedBoundaryData
);
prepared_view!(
    PreparedBoundaryView,
    PreparedBoundary,
    boundaries,
    BoundaryData
);
prepared_view!(
    PreparedOpportunityView,
    PreparedOpportunity,
    opportunities,
    OpportunityData
);
prepared_view!(
    PreparedAlternativeView,
    PreparedAlternative,
    alternatives,
    AlternativeData
);

/// Borrowed observation of an actual contributing captured scope.
pub struct PreparedScopeView<'a> {
    prepared: &'a Arc<PreparedData>,
    index: usize,
}
impl fmt::Debug for PreparedScopeView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedScopeView")
            .field("ordinal", &self.index)
            .finish_non_exhaustive()
    }
}
impl PreparedScope {
    /// Canonical captured owner, independent of this preparation's identity.
    #[must_use]
    pub fn captured_scope(&self) -> CapturedScope {
        let topology = self
            .prepared
            .analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .expect("issuing a contributing scope proves contextual topology");
        CapturedScope {
            capture: Arc::clone(&topology.capture),
            index: self.prepared.analyzed.transform.scopes[self.index].ordinal,
        }
    }

    #[must_use]
    pub fn view(&self) -> PreparedScopeView<'_> {
        PreparedScopeView {
            prepared: &self.prepared,
            index: self.index,
        }
    }
}
impl PreparedScopeView<'_> {
    #[must_use]
    pub fn handle(&self) -> PreparedScope {
        PreparedScope {
            prepared: Arc::clone(self.prepared),
            index: self.index,
        }
    }
    /// Ordinal in the authentic producer scope capture, never scope identity.
    #[must_use]
    pub fn capture_ordinal(&self) -> usize {
        self.prepared.analyzed.transform.scopes[self.index].ordinal
    }
    #[must_use]
    pub fn role(&self) -> ScopeRole {
        self.handle().captured_scope().role()
    }
    #[must_use]
    pub fn style(&self) -> &crate::Style {
        self.capture_fact()
            .style
            .as_ref()
            .expect("issuing an emitted contributor proves its captured Style")
    }
    /// Genuine entry occurrence only when local to this preparation's paragraph.
    /// Use the canonical captured scope to observe a remote global endpoint.
    #[must_use]
    pub fn entry_control(&self) -> Option<PreparedControl> {
        self.capture_fact()
            .boundaries
            .and_then(|pair| self.local_control(pair.start))
    }
    /// Genuine exit occurrence only when local to this preparation's paragraph.
    #[must_use]
    pub fn exit_control(&self) -> Option<PreparedControl> {
        self.capture_fact()
            .boundaries
            .and_then(|pair| self.local_control(pair.end))
    }

    fn capture_fact(&self) -> &crate::processing::contextual::CapturedScopeData {
        let topology = self
            .prepared
            .analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .expect("issuing a contributing scope proves contextual topology");
        &topology.capture.scopes[self.capture_ordinal()]
    }

    fn local_control(&self, global: usize) -> Option<PreparedControl> {
        let topology = self
            .prepared
            .analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .expect("issuing a contributing scope proves contextual topology");
        let order = topology.capture.controls[global].order;
        topology
            .control_orders
            .iter()
            .position(|&candidate| candidate == order)
            .map(|occurrence| PreparedControl {
                prepared: Arc::clone(self.prepared),
                index: control_index(self.prepared, occurrence),
            })
    }
}

/// Borrowed observation of one real emitted virtual separator occurrence.
pub struct PreparedVirtualSeparatorView<'a> {
    prepared: &'a Arc<PreparedData>,
    index: usize,
}
impl fmt::Debug for PreparedVirtualSeparatorView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedVirtualSeparatorView")
            .field("ordinal", &self.index)
            .finish_non_exhaustive()
    }
}
impl PreparedVirtualSeparator {
    #[must_use]
    pub fn view(&self) -> PreparedVirtualSeparatorView<'_> {
        PreparedVirtualSeparatorView {
            prepared: &self.prepared,
            index: self.index,
        }
    }
}
impl PreparedVirtualSeparatorView<'_> {
    #[must_use]
    pub fn handle(&self) -> PreparedVirtualSeparator {
        PreparedVirtualSeparator {
            prepared: Arc::clone(self.prepared),
            index: self.index,
        }
    }
    #[must_use]
    pub fn scope(&self) -> PreparedScope {
        PreparedScope {
            prepared: Arc::clone(self.prepared),
            index: self.prepared.analyzed.transform.virtuals[self.index].scope,
        }
    }
    /// Real emitted occurrence ordinal, independent of its numeric source seam.
    #[must_use]
    pub fn occurrence_ordinal(&self) -> usize {
        self.index
    }
    #[must_use]
    pub fn source_position(&self) -> SourcePosition {
        SourcePosition::from_unchecked(self.prepared.analyzed.transform.virtuals[self.index].source)
    }
    #[must_use]
    pub fn processed_position(&self) -> ProcessedPosition {
        processed_position(
            self.prepared,
            self.prepared.analyzed.transform.virtuals[self.index].processed,
        )
    }
    /// Actual controls before this event seam, in authentic mixed-event order.
    pub fn preceding_controls(&self) -> impl Iterator<Item = PreparedControl> + '_ {
        self.control_sequence()
            .take(self.prepared.analyzed.transform.virtuals[self.index].control_cut)
    }
    /// Actual controls after this event seam, in authentic mixed-event order.
    pub fn following_controls(&self) -> impl Iterator<Item = PreparedControl> + '_ {
        self.control_sequence()
            .skip(self.prepared.analyzed.transform.virtuals[self.index].control_cut)
    }
    fn control_sequence(&self) -> impl Iterator<Item = PreparedControl> + '_ {
        self.prepared
            .analyzed
            .paragraph
            .anchor_order
            .iter()
            .filter_map(|anchor| {
                let super::paragraph::SemanticAnchorItem::Control(occurrence) = anchor.item else {
                    return None;
                };
                Some(PreparedControl {
                    prepared: Arc::clone(self.prepared),
                    index: control_index(self.prepared, occurrence),
                })
            })
    }
    /// Every actual logical unit containing an emission from this occurrence.
    pub fn units(&self) -> impl Iterator<Item = PreparedUnit> + '_ {
        self.prepared.tables.units.iter().enumerate()
            .filter(|(_, unit)| unit.atoms.clone().any(|atom| {
                matches!(self.prepared.analyzed.transform.component_for_atom(atom).owner,
                    crate::transform::relation::ComponentOwner::Virtual(owner) if owner == self.index)
            }))
            .map(|(index, _)| PreparedUnit { prepared: Arc::clone(self.prepared), index })
    }
}

impl PreparedText {
    /// Canonical contextual paragraph, including when no virtual output exists.
    /// Flat and standalone preparation have no contextual capture authority.
    #[must_use]
    pub fn captured_paragraph(&self) -> Option<CapturedParagraph> {
        self.inner
            .analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .map(|topology| CapturedParagraph {
                capture: Arc::clone(&topology.capture),
                index: topology.paragraph,
            })
    }

    /// Actual distinct scopes which own emitted virtual occurrences.
    pub fn scopes(&self) -> impl ExactSizeIterator<Item = PreparedScope> + '_ {
        (0..self.inner.analyzed.transform.scopes.len()).map(|index| PreparedScope {
            prepared: Arc::clone(&self.inner),
            index,
        })
    }
    /// Actual emitted virtual occurrences in semantic order.
    pub fn virtual_separators(
        &self,
    ) -> impl ExactSizeIterator<Item = PreparedVirtualSeparator> + '_ {
        (0..self.inner.analyzed.transform.virtuals.len()).map(|index| PreparedVirtualSeparator {
            prepared: Arc::clone(&self.inner),
            index,
        })
    }

    /// Borrows every original occurrence, including distinct empty occurrences.
    pub fn origin_views(&self) -> impl ExactSizeIterator<Item = PreparedOriginView<'_>> {
        (0..self.inner.analyzed.paragraph.origins.len()).map(|index| PreparedOriginView {
            prepared: &self.inner,
            index,
        })
    }

    /// Borrows actual logical units and their preliminary facts.
    pub fn unit_views(&self) -> impl ExactSizeIterator<Item = PreparedUnitView<'_>> {
        (0..self.inner.tables.units.len()).map(|index| PreparedUnitView {
            prepared: &self.inner,
            index,
        })
    }

    /// Borrows actual atomic occurrences, without treating their widths as text.
    pub fn atomic_views(&self) -> impl ExactSizeIterator<Item = PreparedAtomicView<'_>> {
        (0..self.inner.tables.atomics.len()).map(|index| PreparedAtomicView {
            prepared: &self.inner,
            index,
        })
    }

    /// Borrows actual ordered control occurrences.
    pub fn control_views(&self) -> impl ExactSizeIterator<Item = PreparedControlView<'_>> {
        (0..self.inner.tables.controls.len()).map(|index| PreparedControlView {
            prepared: &self.inner,
            index,
        })
    }

    /// Borrows mixed-sequence boundaries and their incident source extrema.
    pub fn boundary_views(&self) -> impl ExactSizeIterator<Item = PreparedBoundaryView<'_>> {
        (0..self.inner.tables.boundaries.len()).map(|index| PreparedBoundaryView {
            prepared: &self.inner,
            index,
        })
    }

    /// Borrows only the opportunities actually issued by preparation.
    pub fn opportunity_views(&self) -> impl ExactSizeIterator<Item = PreparedOpportunityView<'_>> {
        (0..self.inner.tables.opportunities.len()).map(|index| PreparedOpportunityView {
            prepared: &self.inner,
            index,
        })
    }

    /// Borrows actual prepared alternatives, including retained replacement text.
    pub fn alternative_views(&self) -> impl ExactSizeIterator<Item = PreparedAlternativeView<'_>> {
        self.inner
            .tables
            .ordinary_alternatives
            .iter()
            .map(|&index| PreparedAlternativeView {
                prepared: &self.inner,
                index,
            })
    }

    /// Issues the actual complete mixed sequence in its captured logical order.
    pub fn participants(&self) -> impl ExactSizeIterator<Item = PreparedParticipant> + '_ {
        self.inner
            .tables
            .participants
            .iter()
            .map(|participant| match *participant {
                ParticipantData::Text(index) => PreparedParticipant::Text(PreparedUnit {
                    prepared: Arc::clone(&self.inner),
                    index,
                }),
                ParticipantData::Atomic(index) => PreparedParticipant::Atomic(PreparedAtomic {
                    prepared: Arc::clone(&self.inner),
                    index,
                }),
                ParticipantData::ComputedLineBoundary(index) => {
                    PreparedParticipant::ComputedLineBoundary(PreparedComputedLineBoundary {
                        prepared: Arc::clone(&self.inner),
                        index,
                    })
                }
                ParticipantData::Control(index) => PreparedParticipant::Control(PreparedControl {
                    prepared: Arc::clone(&self.inner),
                    index,
                }),
            })
    }
}

/// A borrowed actual original-source occurrence, independent of paint-run order.
pub struct PreparedOriginView<'a> {
    prepared: &'a Arc<PreparedData>,
    index: usize,
}

impl fmt::Debug for PreparedOriginView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedOriginView")
            .field("ordinal", &self.index)
            .finish_non_exhaustive()
    }
}

impl PreparedOrigin {
    /// Borrows the original occurrence retained by this issued handle.
    #[must_use]
    pub fn view(&self) -> PreparedOriginView<'_> {
        PreparedOriginView {
            prepared: &self.prepared,
            index: self.index,
        }
    }
}

impl PreparedOriginView<'_> {
    /// Genuine original occurrence in global mixed-event order, when captured.
    #[must_use]
    pub fn capture_position(&self) -> Option<CapturedEventPosition> {
        self.prepared
            .analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .map(|topology| CapturedEventPosition {
                capture: Arc::clone(&topology.capture),
                index: topology.origin_orders[self.index],
            })
    }

    /// Retains the actual occurrence capability; equal Source values do not alias it.
    #[must_use]
    pub fn handle(&self) -> PreparedOrigin {
        PreparedOrigin {
            prepared: Arc::clone(self.prepared),
            index: self.index,
        }
    }

    /// The exact retained original source for this occurrence.
    #[must_use]
    pub fn source(&self) -> &Source {
        self.prepared.analyzed.paragraph.origins[self.index]
            .original(&self.prepared.analyzed.processed)
    }

    /// Original local range within this occurrence's retained source.
    #[must_use]
    pub fn local_range(&self) -> Range {
        self.prepared.analyzed.paragraph.origins[self.index].local
    }

    /// The occurrence's original range in the one canonical aggregate source.
    #[must_use]
    pub fn aggregate_range(&self) -> Range {
        self.prepared.analyzed.paragraph.origins[self.index].aggregate
    }

    /// Whether the supplied original is authored or genuinely generated text.
    #[must_use]
    pub fn kind(&self) -> ParagraphOriginKind {
        self.prepared.analyzed.paragraph.origins[self.index].kind
    }
}

impl PreparedUnitView<'_> {
    /// Aggregate original UTF-16 extent, retained independently of processing.
    #[must_use]
    pub fn source_range(&self) -> Range {
        self.data().source
    }

    /// Exact extent in this preparation's canonical processed backing.
    #[must_use]
    pub fn processed_range(&self) -> ProcessedRange {
        processed_range(self.prepared, self.data().processed)
    }

    /// The actual distinct empty origin, when this is its zero-glyph unit.
    #[must_use]
    pub fn empty_origin(&self) -> Option<PreparedOrigin> {
        self.data().empty_origin.map(|index| PreparedOrigin {
            prepared: Arc::clone(self.prepared),
            index,
        })
    }

    /// Preliminary logical advance; selected boundaries may change shaping.
    #[must_use]
    pub fn advance(&self) -> f64 {
        self.data().advance
    }

    /// Preliminary ascent above the baseline.
    #[must_use]
    pub fn ascent(&self) -> f64 {
        self.data().ascent
    }

    /// Preliminary descent below the baseline.
    #[must_use]
    pub fn descent(&self) -> f64 {
        self.data().descent
    }

    /// Preliminary line height, before the selected context is finalized.
    #[must_use]
    pub fn line_height(&self) -> f64 {
        self.data().line_height
    }

    /// Actual resolved paragraph level before selected-line L1 processing.
    #[must_use]
    pub fn level(&self) -> BidiLevel {
        resolved_level(self.data().level)
    }

    /// Whether the actual analyzed unit is whitespace.
    #[must_use]
    pub fn is_whitespace(&self) -> bool {
        self.data().is_whitespace
    }

    /// Exact original occurrence intersections for this actual issued unit.
    /// Equal numeric anchors do not add another empty occurrence to this unit.
    pub fn origins(&self) -> impl Iterator<Item = PreparedOriginIntersectionView<'_>> {
        let empty_origin = self.data().empty_origin;
        self.prepared
            .analyzed
            .paragraph
            .origin_intersections(&self.prepared.analyzed.processed, self.data().source)
            .filter(move |intersection| {
                if let Some(origin) = empty_origin {
                    intersection.origin == origin
                } else {
                    !self.prepared.analyzed.paragraph.origins[intersection.origin]
                        .aggregate
                        .is_empty()
                }
            })
            .map(|association| PreparedOriginIntersectionView {
                prepared: self.prepared,
                association,
            })
    }

    /// Actual generated-control contributors in semantic order. Equal anchors
    /// do not merge different captured occurrences.
    pub fn controls(&self) -> impl ExactSizeIterator<Item = PreparedControl> + '_ {
        self.prepared
            .analyzed
            .transform
            .controls_for_atoms(self.data().atoms.clone())
            .into_iter()
            .map(|occurrence| PreparedControl {
                prepared: Arc::clone(self.prepared),
                index: control_index(self.prepared, occurrence),
            })
    }

    /// All actual virtual contributors, retaining distinct occurrences at equal seams.
    pub fn virtual_separators(
        &self,
    ) -> impl ExactSizeIterator<Item = PreparedVirtualSeparator> + '_ {
        self.prepared
            .analyzed
            .transform
            .virtuals_for_atoms(self.data().atoms.clone())
            .into_iter()
            .map(|index| PreparedVirtualSeparator {
                prepared: Arc::clone(self.prepared),
                index,
            })
    }

    /// Borrows the actual canonical instances overlapping this preliminary unit.
    /// Empty origin-only units do not acquire a fabricated default face.
    pub fn font_instances(&self) -> impl Iterator<Item = Result<EffectiveFontInstance<'_>>> {
        let atoms = &self.data().atoms;
        self.prepared
            .shaped
            .runs
            .iter()
            .filter(move |run| run.atoms.start < atoms.end && atoms.start < run.atoms.end)
            .map(|run| {
                EffectiveFontInstance::qualify_synthesized(
                    run.font.face(),
                    run.font_size,
                    &run.coords,
                    run.synthesis,
                )
            })
    }
}

/// A queried intersection from the canonical original occurrence owner.
/// No glyph run or content hash is used to reconstruct this association.
pub struct PreparedOriginIntersectionView<'a> {
    pub(crate) prepared: &'a Arc<PreparedData>,
    pub(crate) association: OriginIntersection,
}

impl fmt::Debug for PreparedOriginIntersectionView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PreparedOriginIntersectionView")
            .field("aggregate_range", &self.association.aggregate)
            .field("local_range", &self.association.local)
            .finish_non_exhaustive()
    }
}

impl PreparedOriginIntersectionView<'_> {
    /// Retains the actual origin occurrence, independent of equal source values.
    #[must_use]
    pub fn origin(&self) -> PreparedOrigin {
        PreparedOrigin {
            prepared: Arc::clone(self.prepared),
            index: self.association.origin,
        }
    }
    /// Original local extent within that retained occurrence.
    #[must_use]
    pub const fn local_range(&self) -> Range {
        self.association.local
    }
    /// Original aggregate-source intersection.
    #[must_use]
    pub const fn aggregate_range(&self) -> Range {
        self.association.aggregate
    }
    /// Actual extent in the one processed owner.
    #[must_use]
    pub fn processed_range(&self) -> ProcessedRange {
        processed_range(self.prepared, self.association.processed)
    }
}

impl PreparedAtomicView<'_> {
    /// Genuine atomic occurrence in global mixed-event order, when captured.
    #[must_use]
    pub fn capture_position(&self) -> Option<CapturedEventPosition> {
        self.prepared
            .analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .map(|topology| CapturedEventPosition {
                capture: Arc::clone(&topology.capture),
                index: topology.capture.atomic_orders[self.data().occurrence],
            })
    }

    /// The exact original box occurrence, including its actual role and ID.
    #[must_use]
    pub fn original_box(&self) -> &InlineBox {
        &self.prepared.analyzed.processed.source().boxes()[self.data().occurrence]
    }

    /// Original aggregate source anchor.
    #[must_use]
    pub fn source_position(&self) -> SourcePosition {
        SourcePosition::from_unchecked(self.data().source)
    }

    /// Actual corresponding anchor in this preparation's processed backing.
    #[must_use]
    pub fn processed_position(&self) -> ProcessedPosition {
        processed_position(self.prepared, self.data().processed)
    }

    /// Actual resolved level before selected-line L1 processing.
    #[must_use]
    pub fn level(&self) -> BidiLevel {
        resolved_level(self.data().level)
    }
}

impl PreparedControlView<'_> {
    /// Genuine control occurrence in global mixed-event order, when captured.
    #[must_use]
    pub fn capture_position(&self) -> Option<CapturedEventPosition> {
        self.prepared
            .analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .map(|topology| CapturedEventPosition {
                capture: Arc::clone(&topology.capture),
                index: topology.control_orders[self.data().occurrence],
            })
    }

    /// Actual captured semantic control, rather than fabricated authored text.
    #[must_use]
    pub fn kind(&self) -> &ParagraphControlKind {
        &self.prepared.analyzed.paragraph.controls[self.data().occurrence].kind
    }

    /// Original aggregate source anchor.
    #[must_use]
    pub fn source_position(&self) -> SourcePosition {
        SourcePosition::from_unchecked(self.data().source)
    }

    /// Corresponding processed anchor.
    #[must_use]
    pub fn processed_position(&self) -> ProcessedPosition {
        processed_position(self.prepared, self.data().processed)
    }

    /// Actual resolved level before selected-line L1 processing.
    #[must_use]
    pub fn level(&self) -> BidiLevel {
        resolved_level(self.data().level)
    }
}

impl PreparedControl {
    /// Canonical semantic control, while this handle retains preparation identity.
    #[must_use]
    pub fn captured_control(&self) -> Option<CapturedControl> {
        let topology = self.prepared.analyzed.paragraph.scope_topology.as_ref()?;
        let order = topology.control_orders[self.view().data().occurrence];
        let SourceProcessingEvent::Control { control, .. } = topology.capture.events[order].event
        else {
            unreachable!("contextual admission indexes genuine control events")
        };
        Some(CapturedControl {
            capture: Arc::clone(&topology.capture),
            index: control,
        })
    }
}

impl PreparedBoundaryView<'_> {
    /// Actual offset in the complete mixed participant sequence.
    /// This observation cannot construct a boundary or authorize a break.
    #[must_use]
    pub fn participant_offset(&self) -> usize {
        self.data().participant
    }

    /// The original incident endpoint selected by Before affinity.
    #[must_use]
    pub fn source_before(&self) -> SourcePosition {
        SourcePosition::from_unchecked(self.data().source_before)
    }

    /// The original incident endpoint selected by After affinity.
    #[must_use]
    pub fn source_after(&self) -> SourcePosition {
        SourcePosition::from_unchecked(self.data().source_after)
    }

    /// This actual mixed boundary's processed endpoint.
    #[must_use]
    pub fn processed_position(&self) -> ProcessedPosition {
        processed_position(self.prepared, self.data().processed)
    }
}

impl PreparedOpportunityView<'_> {
    /// The exact prepared boundary at which this opportunity was issued.
    #[must_use]
    pub fn boundary(&self) -> PreparedBoundary {
        PreparedBoundary {
            prepared: Arc::clone(self.prepared),
            index: self.data().boundary,
        }
    }

    /// Actual terminating behavior; no replacement is inferred from it.
    #[must_use]
    pub fn kind(&self) -> PreparedBreakKind {
        self.data().kind
    }

    /// Borrows only the actual alternatives belonging to this opportunity.
    pub fn alternatives(&self) -> impl ExactSizeIterator<Item = PreparedAlternativeView<'_>> {
        self.data()
            .alternatives
            .iter()
            .map(|&index| PreparedAlternativeView {
                prepared: self.prepared,
                index,
            })
    }
}

impl PreparedAlternativeView<'_> {
    /// Actual cut observation, including suppressed manual candidates elsewhere.
    #[must_use]
    pub fn hyphenation_cut(&self) -> super::PreparedHyphenationCut {
        super::PreparedHyphenationCut {
            prepared: Arc::clone(self.prepared),
            index: self.data().hyphenation_cut,
        }
    }
    /// The actual owning opportunity, qualified by this preparation allocation.
    #[must_use]
    pub fn opportunity(&self) -> Option<PreparedOpportunity> {
        self.data().opportunity.map(|index| PreparedOpportunity {
            prepared: Arc::clone(self.prepared),
            index,
        })
    }

    #[must_use]
    pub fn boundary(&self) -> PreparedBoundary {
        PreparedBoundary {
            prepared: Arc::clone(self.prepared),
            index: self.data().boundary,
        }
    }

    #[must_use]
    pub fn availability(&self) -> super::PreparedAlternativeAvailability {
        if self.data().opportunity.is_some() {
            super::PreparedAlternativeAvailability::Ordinary
        } else {
            super::PreparedAlternativeAvailability::AutoPhraseSuppressed
        }
    }

    /// Original source displaced by this actual alternative.
    #[must_use]
    pub fn source_range(&self) -> Option<Range> {
        match self.data().definition {
            AlternativeDefinition::Source { source, .. } => Some(source),
            AlternativeDefinition::Control { .. } | AlternativeDefinition::Automatic { .. } => None,
        }
    }

    /// The authentic defining source or native control occurrence.
    #[must_use]
    pub fn definition(&self) -> PreparedAlternativeDefinition {
        let origin = |index| PreparedOrigin {
            prepared: Arc::clone(self.prepared),
            index,
        };
        match self.data().definition {
            AlternativeDefinition::Source {
                origin: index,
                source,
            } => PreparedAlternativeDefinition::Source {
                origin: origin(index),
                source,
            },
            AlternativeDefinition::Control {
                control,
                origin: incident,
            } => PreparedAlternativeDefinition::Control {
                control: PreparedControl {
                    prepared: Arc::clone(self.prepared),
                    index: control_index(self.prepared, control),
                },
                origin: incident.map(origin),
            },
            AlternativeDefinition::Automatic { cut, .. } => {
                PreparedAlternativeDefinition::Automatic {
                    cut: super::PreparedHyphenationCut {
                        prepared: Arc::clone(self.prepared),
                        index: cut,
                    },
                }
            }
        }
    }

    /// Original insertion anchor; replacement glyphs are never authored characters.
    #[must_use]
    pub fn insertion_anchor(&self) -> SourcePosition {
        SourcePosition::from_unchecked(self.data().insertion_anchor)
    }

    /// Exact retained computed replacement text, not text inferred from glyphs.
    #[must_use]
    pub fn text(&self) -> &Text {
        &self.data().text
    }
}

/// Authentic definition of an issued replacement, without fabricated source text.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PreparedAlternativeDefinition {
    Automatic {
        cut: super::PreparedHyphenationCut,
    },
    Source {
        origin: PreparedOrigin,
        source: Range,
    },
    Control {
        control: PreparedControl,
        origin: Option<PreparedOrigin>,
    },
}

impl FinalizedLine {
    /// Borrows the actual paint-order runs of this finalized line.
    pub fn run_views(&self) -> impl ExactSizeIterator<Item = FinalizedRunView<'_>> {
        self.inner.data.runs.iter().map(|run| FinalizedRunView {
            line: self,
            run,
            shaped: &self.inner.data.font_runs[run.run],
        })
    }

    /// Issues actual origin-intersection fragments from this finalized allocation.
    /// A fragment identifies a source occurrence, not a paint-run ordinal.
    pub fn fragments(&self) -> impl ExactSizeIterator<Item = FinalizedFragment> + '_ {
        (0..self.inner.data.fragments.len()).map(|index| FinalizedFragment {
            finalized: Arc::clone(&self.inner),
            index,
        })
    }

    /// Borrows the actual finalized source fragments without retaining new handles.
    pub fn fragment_views(&self) -> impl ExactSizeIterator<Item = FinalizedFragmentView<'_>> {
        self.inner
            .data
            .fragments
            .iter()
            .map(|fragment| FinalizedFragmentView {
                prepared: &self.inner.selected.prepared,
                fragment,
            })
    }

    /// Borrows the actual selected source/processed/presentation correspondence.
    pub fn mapping_views(&self) -> impl ExactSizeIterator<Item = SelectedMappingView<'_>> {
        self.inner
            .selected
            .basis
            .mapping
            .pieces
            .iter()
            .map(|piece| SelectedMappingView {
                prepared: &self.inner.selected.prepared,
                piece,
            })
    }

    /// Same-instance text outline ink in line-local coordinates.
    ///
    /// Every glyph is qualified by the published font-outline owner. Empty is
    /// source-proved; unavailable or unsupported outlines remain typed errors.
    /// This excludes atomic boxes and does not select Render paint representations.
    /// Bounds conservatively translate the origin-zero canonical font outline
    /// in the logical metric domain; they do not claim Render's positioned
    /// canonical-point or representation-selected paint bounds. This operation
    /// is requested independently and is not a prerequisite for ordinary shaping.
    pub fn outline_ink(&self) -> Result<LineOutlineInk> {
        let mut ink = None;
        for run in self.run_views() {
            let instance = run.effective_instance()?;
            let origin = run.origin();
            for glyph in run.glyphs() {
                let geometry = instance
                    .glyph_outline(glyph.id(), &self.inner.selected.prepared.font_catalogue)?;
                let Some(bounds) = geometry.bounds() else {
                    continue;
                };
                let x = outline_add("outline logical pen", origin.x, glyph.pen())?;
                let x = outline_add("outline glyph x offset", x, f64::from(glyph.x()))?;
                let y = outline_add("outline glyph y offset", origin.y, f64::from(glyph.y()))?;
                let left = outline_add("outline left", x, bounds.origin.x)?;
                let top = outline_add("outline top", y, bounds.origin.y)?;
                let right = outline_add("outline right", left, bounds.size.width)?;
                let bottom = outline_add("outline bottom", top, bounds.size.height)?;
                ink = Some(match ink {
                    None => (left, top, right, bottom),
                    Some((x0, y0, x1, y1)) => (
                        f64::min(x0, left),
                        f64::min(y0, top),
                        f64::max(x1, right),
                        f64::max(y1, bottom),
                    ),
                });
            }
        }
        let Some((left, top, right, bottom)) = ink else {
            return Ok(LineOutlineInk::Empty);
        };
        let width = right - left;
        let height = bottom - top;
        for (field, value) in [
            ("outline union width", width),
            ("outline union height", height),
        ] {
            if !value.is_finite() {
                return Err(crate::error::text_numeric_error(
                    field,
                    TextNumericPhase::GlyphOutline,
                    TextNumericError::NonFinite,
                    value,
                ));
            }
        }
        Ok(LineOutlineInk::Bounds(Rect::new(left, top, width, height)))
    }
}

/// Text-owned same-instance glyph-outline ink, separate from advance/line boxes
/// and from Render's representation-selected graph ink.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum LineOutlineInk {
    Empty,
    Bounds(Rect),
}

fn outline_add(field: &'static str, base: f64, delta: f64) -> Result<f64> {
    let actual = base + delta;
    let reason = if !actual.is_finite() {
        Some(TextNumericError::NonFinite)
    } else if delta != 0.0 && actual == base {
        Some(TextNumericError::NonZeroLost)
    } else {
        None
    };
    if let Some(reason) = reason {
        return Err(crate::error::text_numeric_error(
            field,
            TextNumericPhase::GlyphOutline,
            reason,
            delta,
        ));
    }
    Ok(actual)
}

impl FinalizedFragment {
    /// Retains the exact actual finalized line which owns this fragment.
    #[must_use]
    pub fn finalized_line(&self) -> FinalizedLine {
        FinalizedLine {
            inner: Arc::clone(&self.finalized),
        }
    }

    /// Borrows this actual selected occurrence intersection.
    #[must_use]
    pub fn view(&self) -> FinalizedFragmentView<'_> {
        FinalizedFragmentView {
            prepared: &self.finalized.selected.prepared,
            fragment: &self.finalized.data.fragments[self.index],
        }
    }
}

/// Authentic occurrence which owns a finalized fragment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FinalizedFragmentOwner {
    Origin(PreparedOrigin),
    Control(PreparedControl),
    Alternative(PreparedAlternative),
    VirtualSeparator(PreparedVirtualSeparator),
}

/// One actual selected occurrence and its optional original-source association.
pub struct FinalizedFragmentView<'a> {
    prepared: &'a Arc<PreparedData>,
    fragment: &'a FinalizedFragmentData,
}

impl fmt::Debug for FinalizedFragmentView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FinalizedFragmentView")
            .field("owner", &self.owner())
            .field("aggregate_range", &self.aggregate_range())
            .finish_non_exhaustive()
    }
}

impl FinalizedFragmentView<'_> {
    /// Retains the exact actual occurrence; no source-offset inference is used.
    #[must_use]
    pub fn owner(&self) -> FinalizedFragmentOwner {
        match self.fragment.association.owner {
            FragmentOwner::Origin(index) => FinalizedFragmentOwner::Origin(PreparedOrigin {
                prepared: Arc::clone(self.prepared),
                index,
            }),
            FragmentOwner::Control(occurrence) => {
                FinalizedFragmentOwner::Control(PreparedControl {
                    prepared: Arc::clone(self.prepared),
                    index: control_index(self.prepared, occurrence),
                })
            }
            FragmentOwner::VirtualSeparator(index) => {
                FinalizedFragmentOwner::VirtualSeparator(PreparedVirtualSeparator {
                    prepared: Arc::clone(self.prepared),
                    index,
                })
            }
            FragmentOwner::Alternative(index) => {
                FinalizedFragmentOwner::Alternative(PreparedAlternative {
                    prepared: Arc::clone(self.prepared),
                    index,
                })
            }
        }
    }

    /// Genuine incident source occurrence, when one exists.
    #[must_use]
    pub fn origin(&self) -> Option<PreparedOrigin> {
        self.fragment
            .association
            .source
            .as_ref()
            .map(|source| PreparedOrigin {
                prepared: Arc::clone(self.prepared),
                index: source.origin,
            })
    }

    #[must_use]
    pub fn local_range(&self) -> Option<Range> {
        self.fragment
            .association
            .source
            .as_ref()
            .map(|source| source.local)
    }

    #[must_use]
    pub fn aggregate_range(&self) -> Option<Range> {
        self.fragment
            .association
            .source
            .as_ref()
            .map(|source| source.aggregate)
    }

    #[must_use]
    pub fn processed_range(&self) -> Option<ProcessedRange> {
        self.fragment
            .association
            .source
            .as_ref()
            .map(|source| processed_range(self.prepared, source.processed))
    }

    /// Actual line-local source/occurrence geometry, never glyph paint ink.
    #[must_use]
    pub const fn bounds(&self) -> Option<Rect> {
        self.fragment.bounds
    }
}

/// Borrowed actual native-control slot retained by the finalizer.
pub struct FinalizedControlView<'a> {
    prepared: &'a Arc<PreparedData>,
    occurrence: usize,
    slot: Point,
    level: u8,
    inline_end: f64,
}
impl fmt::Debug for FinalizedControlView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FinalizedControlView")
            .field("slot", &self.slot)
            .field("level", &self.level)
            .finish_non_exhaustive()
    }
}
impl FinalizedControlView<'_> {
    #[must_use]
    pub fn control(&self) -> PreparedControl {
        PreparedControl {
            prepared: Arc::clone(self.prepared),
            index: control_index(self.prepared, self.occurrence),
        }
    }
    #[must_use]
    pub const fn slot(&self) -> Point {
        self.slot
    }
    #[must_use]
    pub const fn inline_bounds(&self) -> (f64, f64) {
        (self.slot.x, self.inline_end)
    }
    #[must_use]
    pub fn level(&self) -> BidiLevel {
        resolved_level(self.level)
    }
}
impl FinalizedLine {
    /// Borrows actual finalized native-control slots, including line terminators.
    pub fn control_slots(&self) -> impl Iterator<Item = FinalizedControlView<'_>> {
        self.inner.data.anchors.iter().filter_map(|anchor| {
            let super::prepared::PresentationUnitKind::Control { occurrence } =
                self.inner.selected.presentation.units[anchor.unit].kind
            else {
                return None;
            };
            Some(FinalizedControlView {
                prepared: &self.inner.selected.prepared,
                occurrence,
                slot: anchor.slot,
                level: anchor.level,
                inline_end: anchor.inline_end,
            })
        })
    }
}

impl SelectedLine {
    /// Borrows the exact complete admitted request, including its issued handles.
    /// Consumers can compare actual context without retaining a second snapshot.
    #[must_use]
    pub fn request(&self) -> &LineSelectionRequest {
        &self.inner.request
    }
}

/// What actually happened to a selected source/presentation interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PresentationMappingKind {
    Source,
    ComputedLineBoundary,
    Deleted,
    Replacement,
    GeneratedSeparator,
}

/// Borrowed exact correspondence owned by this finalized line.
pub struct SelectedMappingView<'a> {
    prepared: &'a Arc<PreparedData>,
    piece: &'a SelectedMappingPiece,
}

impl fmt::Debug for SelectedMappingView<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SelectedMappingView")
            .field("source", &self.piece.source)
            .field("processed", &self.piece.processed)
            .field("presentation", &self.piece.presentation)
            .finish_non_exhaustive()
    }
}

impl SelectedMappingView<'_> {
    /// Original aggregate source; deletion does not erase this association.
    #[must_use]
    pub const fn source_range(&self) -> Range {
        self.piece.source
    }

    /// Actual canonical processed interval, independent of selected replacement.
    #[must_use]
    pub fn processed_range(&self) -> ProcessedRange {
        processed_range(self.prepared, self.piece.processed)
    }

    /// Line-local presentation UTF-16 extent, never an authored source range.
    #[must_use]
    pub const fn presentation_range(&self) -> PresentationRange {
        PresentationRange {
            start: self.piece.presentation.start,
            end: self.piece.presentation.end,
        }
    }

    /// The executed source, deletion or replacement role of this piece.
    #[must_use]
    pub const fn kind(&self) -> PresentationMappingKind {
        match self.piece.kind {
            SelectedMappingKind::Source => PresentationMappingKind::Source,
            SelectedMappingKind::ComputedLineBoundary { .. } => {
                PresentationMappingKind::ComputedLineBoundary
            }
            SelectedMappingKind::Deleted => PresentationMappingKind::Deleted,
            SelectedMappingKind::Replacement { .. } => PresentationMappingKind::Replacement,
            SelectedMappingKind::GeneratedSeparator { .. } => {
                PresentationMappingKind::GeneratedSeparator
            }
        }
    }

    /// Retains the actual selected replacement capability, when this is its piece.
    #[must_use]
    pub fn alternative(&self) -> Option<PreparedAlternative> {
        match self.piece.kind {
            SelectedMappingKind::Replacement { alternative, .. } => Some(PreparedAlternative {
                prepared: Arc::clone(self.prepared),
                index: alternative,
            }),
            _ => None,
        }
    }

    /// The genuine control which emitted this source-less separator.
    pub fn separator_control(&self) -> Option<PreparedControl> {
        match self.piece.kind {
            SelectedMappingKind::GeneratedSeparator {
                owner: crate::transform::relation::GeneratedSeparatorOwner::Control(control),
            } => Some(PreparedControl {
                prepared: Arc::clone(self.prepared),
                index: control_index(self.prepared, control),
            }),
            _ => None,
        }
    }

    /// The actual virtual occurrence which emitted this source-less separator.
    #[must_use]
    pub fn separator_virtual(&self) -> Option<PreparedVirtualSeparator> {
        match self.piece.kind {
            SelectedMappingKind::GeneratedSeparator {
                owner: crate::transform::relation::GeneratedSeparatorOwner::Virtual(index),
            } => Some(PreparedVirtualSeparator {
                prepared: Arc::clone(self.prepared),
                index,
            }),
            _ => None,
        }
    }

    /// The actual original insertion anchor of a replacement piece.
    #[must_use]
    pub fn insertion_anchor(&self) -> Option<SourcePosition> {
        match self.piece.kind {
            SelectedMappingKind::Replacement { anchor, .. } => {
                Some(SourcePosition::from_unchecked(anchor))
            }
            _ => None,
        }
    }
}

/// An actual finalized run, before any physical line placement.
#[derive(Debug)]
pub struct FinalizedRunView<'a> {
    line: &'a FinalizedLine,
    run: &'a FinalizedRun,
    shaped: &'a PreparedRun,
}

impl FinalizedRunView<'_> {
    /// Actual outline axes from the authenticated uniform preparation.
    ///
    /// Horizontal and Vertical outlines are unrotated; Sideways is clockwise.
    /// The finalized pen follows the inline axis separately from raw offsets.
    #[must_use]
    pub fn glyph_basis(&self) -> crate::GlyphBasis {
        crate::GlyphBasis::for_orientation(self.line.inner.selected.prepared.options.orientation())
    }

    /// The aggregate original UTF-16 association of this run.
    ///
    /// This is source correspondence, not an origin-occurrence identity or a
    /// permission to use an arbitrary boundary as a line break.
    #[must_use]
    pub const fn range(&self) -> Range {
        self.run.source
    }

    /// Actual effective paint brush from the retained resolved style.
    #[must_use]
    pub fn brush(&self) -> Brush {
        self.line.inner.selected.prepared.styles[self.run.style]
            .style
            .brush
    }

    /// Canonical size used by the actual shaping invocation.
    #[must_use]
    pub const fn font_size(&self) -> f32 {
        self.shaped.font_size
    }

    /// Borrows the actual retained resource, face and catalogue publication.
    #[must_use]
    pub fn face(&self) -> FontFace<'_> {
        self.shaped.font.face()
    }

    /// The run's actual line-local paint origin. For named Alphabetic H/Sideways
    /// composition, this is font-design zero rather than the common alignment
    /// reference retained in the line's feedback.
    ///
    /// Coordinates are `(inline, line-under)` before physical placement.
    /// Vertical and Sideways cells map `(u,v)` to
    /// `(cross_anchor-(v-local_baseline), inline_origin+u)`; this cell map does
    /// not rotate whole-upright glyph outlines. Once placed, glyph origins use
    /// the owning run's physical origin and its separate pen/offset axes.
    #[must_use]
    pub fn origin(&self) -> Point {
        Point::new(self.run.x, self.run.y)
    }

    /// Final logical advance, including the admitted spacing allocation.
    #[must_use]
    pub const fn advance(&self) -> f64 {
        self.run.advance
    }

    /// Borrows raw provider glyph facts and their separate finalized inline pens.
    ///
    /// After placement at physical run origin `(X,Y)`, pen `p` and raw offsets
    /// `(dx,dy)` give glyph origin `(X+p+dx,Y+dy)` for Horizontal,
    /// `(X-dy,Y+p+dx)` for Sideways and `(X+dx,Y+p+dy)` for Vertical.
    /// The outline axes follow [`Self::glyph_basis`]; raw offsets are not cell
    /// coordinates and are never rotated in storage.
    pub fn glyphs(&self) -> impl ExactSizeIterator<Item = FinalizedGlyphView<'_>> {
        self.line.inner.data.glyphs[self.run.glyphs.clone()]
            .iter()
            .map(|glyph| FinalizedGlyphView { glyph })
    }

    /// Qualifies the same canonical coordinates and executable synthesis.
    pub fn effective_instance(&self) -> Result<EffectiveFontInstance<'_>> {
        EffectiveFontInstance::qualify_synthesized(
            self.face(),
            self.shaped.font_size,
            &self.shaped.coords,
            self.shaped.synthesis,
        )
    }
}

/// A borrowed glyph from the actual immutable finalized line.
///
/// The provider's canonical `f32` facts remain separate from the logical `f64`
/// pen. No physical origin is folded into either value.
#[derive(Clone, Copy, Debug)]
pub struct FinalizedGlyphView<'a> {
    glyph: &'a FinalizedGlyph,
}

impl FinalizedGlyphView<'_> {
    /// Actual glyph identifier in its run's retained face.
    #[must_use]
    pub const fn id(self) -> u32 {
        self.glyph.glyph.id
    }

    /// Raw canonical provider x offset, before the logical pen and run origin.
    #[must_use]
    pub const fn x(self) -> f32 {
        self.glyph.glyph.x
    }

    /// Raw canonical provider y offset, positive downward from the baseline.
    #[must_use]
    pub const fn y(self) -> f32 {
        self.glyph.glyph.y
    }

    /// Raw canonical provider advance, before logical spacing.
    #[must_use]
    pub const fn advance(self) -> f32 {
        self.glyph.glyph.advance
    }

    /// Actual finalized logical pen relative to this glyph's run origin.
    #[must_use]
    pub const fn pen(self) -> f64 {
        self.glyph.pen
    }

    /// Aggregate original UTF-16 association, including a genuine empty anchor.
    #[must_use]
    pub const fn range(self) -> Range {
        self.glyph.glyph.source
    }
}

impl PreparedComputedLineBoundaryView<'_> {
    /// Original event immediately before the effective group's canonical cut.
    /// This observes the group, not an arbitrary member cause's target.
    #[must_use]
    pub fn preceding_event(&self) -> Option<CapturedEventPosition> {
        let analyzed = &self.prepared.analyzed;
        let topology = analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .expect("computed capture");
        let index = analyzed.paragraph.computed_boundaries[self.data().occurrence]
            .capture_cut
            .checked_sub(1)?;
        Some(CapturedEventPosition {
            capture: Arc::clone(&topology.capture),
            index,
        })
    }
    /// Original event immediately after the effective group's canonical cut.
    #[must_use]
    pub fn following_event(&self) -> Option<CapturedEventPosition> {
        let analyzed = &self.prepared.analyzed;
        let topology = analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .expect("computed capture");
        let index = analyzed.paragraph.computed_boundaries[self.data().occurrence].capture_cut;
        (index < topology.capture.events.len()).then(|| CapturedEventPosition {
            capture: Arc::clone(&topology.capture),
            index,
        })
    }
    #[must_use]
    pub fn domain(&self) -> CapturedScope {
        let topology = self
            .prepared
            .analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .expect("computed capture");
        CapturedScope {
            capture: Arc::clone(&topology.capture),
            index: topology.capture.domain_scopes[topology.domain],
        }
    }
    #[must_use]
    pub fn paragraph(&self) -> CapturedParagraph {
        let topology = self
            .prepared
            .analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .expect("computed capture");
        CapturedParagraph {
            capture: Arc::clone(&topology.capture),
            index: topology.paragraph,
        }
    }
    /// Genuine computed causes at this one effective mandatory incidence.
    pub fn causes(
        &self,
    ) -> impl ExactSizeIterator<Item = crate::CapturedComputedLineBoundary> + '_ {
        let analyzed = &self.prepared.analyzed;
        let topology = analyzed
            .paragraph
            .scope_topology
            .as_ref()
            .expect("computed admission retains capture");
        analyzed.paragraph.computed_boundaries[self.data().occurrence]
            .causes
            .iter()
            .map(|&index| crate::CapturedComputedLineBoundary {
                capture: Arc::clone(&topology.capture),
                index,
            })
    }
    #[must_use]
    pub fn source_position(&self) -> SourcePosition {
        SourcePosition::from_unchecked(self.data().source)
    }
    #[must_use]
    pub fn processed_position(&self) -> ProcessedPosition {
        processed_position(self.prepared, self.data().processed)
    }
    #[must_use]
    pub fn level(&self) -> BidiLevel {
        resolved_level(self.data().level)
    }
}
impl PreparedOpportunityView<'_> {
    /// Computed mandatory cause group terminating this actual opportunity.
    #[must_use]
    pub fn computed_line_boundary(&self) -> Option<PreparedComputedLineBoundary> {
        if self.data().kind != PreparedBreakKind::Forced {
            return None;
        }
        let boundary = self.prepared.tables.boundaries[self.data().boundary];
        let ParticipantData::ComputedLineBoundary(index) = *self
            .prepared
            .tables
            .participants
            .get(boundary.participant.checked_sub(1)?)?
        else {
            return None;
        };
        Some(PreparedComputedLineBoundary {
            prepared: Arc::clone(self.prepared),
            index,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Font, FontCatalogueMode, LineHeight, Options, Style, System, SystemOptions};

    const AHEM: &[u8] = include_bytes!("../../tests/fixtures/fonts/ahem/Ahem.ttf");

    fn actual_line(text: &str) -> FinalizedLine {
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(AHEM.to_vec()).unwrap();
        let layout = system
            .layout(
                Source::new(text),
                Style {
                    font: Font::new().family("Ahem"),
                    size: 20.0,
                    line_height: LineHeight::FontSizeRelative(1.0),
                    ..Style::default()
                },
                Options {
                    quantize: false,
                    ..Options::default()
                },
            )
            .unwrap();
        FinalizedLine {
            inner: Arc::clone(&layout.inner.lines[0].finalized),
        }
    }

    #[test]
    fn finalized_borrowed_glyphs_retain_actual_instance_after_system_and_layout_drop() {
        let line = actual_line("A");
        let run = line.run_views().next().unwrap();
        assert_eq!(run.range(), Range::new(0, 1));
        assert_eq!(run.origin(), Point::new(0.0, 16.0));
        assert_eq!(run.advance(), 20.0);
        let glyph = run.glyphs().next().unwrap();
        assert_eq!(
            (
                glyph.id(),
                glyph.x(),
                glyph.y(),
                glyph.advance(),
                glyph.pen()
            ),
            (35, 0.0, 0.0, 20.0, 0.0)
        );
        assert_eq!(glyph.range(), Range::new(0, 1));
        let instance = run.effective_instance().unwrap();
        assert_eq!(instance.face().bytes(), AHEM);
        assert_eq!(instance.font_size(), 20.0);
    }

    #[test]
    fn actual_outline_bounds_and_empty_space_differ_despite_equal_advances() {
        let letter = actual_line("A");
        assert_eq!(
            letter.outline_ink().unwrap(),
            LineOutlineInk::Bounds(Rect::new(0.0, 0.0, 20.0, 20.0))
        );
        let space = actual_line(" ");
        assert_eq!(space.outline_ink().unwrap(), LineOutlineInk::Empty);
        assert_eq!(
            letter.run_views().next().unwrap().advance(),
            space.run_views().next().unwrap().advance()
        );
    }

    #[test]
    fn actual_prepared_origin_and_unit_views_keep_the_same_canonical_correspondence() {
        let line = actual_line("A");
        let prepared = PreparedText {
            inner: Arc::clone(&line.inner.selected.prepared),
        };
        let origin = prepared.origin_views().next().unwrap();
        assert_eq!(origin.source().text().code_units(), [0x41]);
        assert_eq!(origin.local_range(), Range::new(0, 1));
        assert_eq!(origin.aggregate_range(), Range::new(0, 1));
        let unit = prepared.unit_views().next().unwrap();
        assert_eq!(unit.source_range(), Range::new(0, 1));
        assert_eq!(
            (
                unit.processed_range().start().get(),
                unit.processed_range().end().get()
            ),
            (0, 1)
        );
        assert_eq!(
            (unit.advance(), unit.ascent(), unit.descent()),
            (20.0, 16.0, 4.0)
        );
        let association = unit.origins().next().unwrap();
        assert_eq!(association.origin(), origin.handle());
        assert_eq!(association.local_range(), Range::new(0, 1));
        assert_eq!(
            unit.font_instances()
                .next()
                .unwrap()
                .unwrap()
                .face()
                .bytes(),
            AHEM
        );
    }

    #[test]
    fn native_empty_origin_is_a_baseline_anchor_independent_of_the_real_line_strut() {
        use crate::protocol::paragraph::{
            ParagraphAnchor, ParagraphAnchorItem, ParagraphConnection, ParagraphConnectionKind,
            ParagraphOrigin, ParagraphOriginKind, ParagraphStrut,
        };
        use crate::protocol::{
            LineAnchor, LineContext, LineEnd, LineOrientation, LinePlacement, LineStart,
            ParagraphBaseDirection, ParagraphInput, PlacementCoverage, PlacementRequest,
            PreparationOptions, PreparedBreakKind, SpacingAllocations, SpacingPolicy, TabContext,
        };
        let mut system = System::new(SystemOptions {
            font_catalogue: FontCatalogueMode::RegisteredOnly,
        })
        .unwrap();
        system.register_fonts(AHEM.to_vec()).unwrap();
        let source = Source::new("A");
        let analysis = crate::AnalyzedSource::from_paragraph(
            ParagraphInput {
                terminal_edge: crate::ParagraphTerminalEdge::BlockEnd,
                tab_reference: None,
                source: source.clone(),
                base_style: Style {
                    font: Font::new().family("Ahem"),
                    size: 20.0,
                    line_height: LineHeight::FontSizeRelative(2.0),
                    ..Style::default()
                },
                segment_breaks: crate::DocumentSegmentBreaks::LineFeedDefault,
                base_direction: ParagraphBaseDirection::LeftToRight,
                paragraph_strut: ParagraphStrut::DefaultFont,
                origins: vec![
                    ParagraphOrigin {
                        text_node: None,
                        source,
                        local: Range::new(0, 1),
                        aggregate: Range::new(0, 1),
                        kind: ParagraphOriginKind::Authored,
                    },
                    ParagraphOrigin {
                        text_node: None,
                        source: Source::new(""),
                        local: Range::new(0, 0),
                        aggregate: Range::new(1, 1),
                        kind: ParagraphOriginKind::Authored,
                    },
                ],
                connections: vec![ParagraphConnection {
                    at: 1,
                    kind: ParagraphConnectionKind::Disconnected,
                }],
                controls: vec![],
                boundary_policies: vec![],
                replacements: vec![],
                anchor_order: vec![ParagraphAnchor {
                    item: ParagraphAnchorItem::EmptyOrigin(1),
                    affinity: crate::Affinity::After,
                }],
            },
            crate::AnalysisTypography::HorizontalComposition,
        )
        .unwrap();
        let prepared = system
            .prepare(&analysis, PreparationOptions::default())
            .unwrap();
        let origin = prepared.origin_views().nth(1).unwrap().handle();
        let termination = prepared
            .opportunity_views()
            .find(|view| view.kind() == PreparedBreakKind::ParagraphEnd)
            .unwrap()
            .handle();
        let selected = prepared
            .select(LineSelectionRequest {
                boundary_context: Vec::new(),
                incoming: None,
                start: prepared.boundary_views().next().unwrap().handle(),
                end: termination.view().boundary(),
                participants: prepared.participants().collect(),
                termination: crate::PreparedTermination::Ordinary(termination),
                alternative: None,
                context: LineContext {
                    alignment_insets: crate::LineAlignmentInsets::ZERO,
                    formatted_line: crate::FormattedLineContext::default(),
                    base_direction: ParagraphBaseDirection::LeftToRight,
                    line_box_direction: crate::LineBoxDirection::LeftToRight,
                    orientation: LineOrientation::Horizontal,
                    start: LineStart::Paragraph,
                    end: LineEnd::Paragraph,
                    available_inline: None,
                    indent: 0.0,
                    alignment: crate::Alignment::Start,
                    tabs: TabContext::NoTabs,

                    spacing: SpacingPolicy::None,
                },
                atomics: vec![],
                controls: vec![],
                line_box_struts: Vec::new(),
                line_strut: None,
            })
            .unwrap();
        let line = selected.finalize(SpacingAllocations::None).unwrap();
        assert_eq!(line.feedback().baseline, 26.0);
        assert_eq!(line.feedback().line_height, 40.0);
        let empty = line
            .fragment_views()
            .find(|fragment| fragment.owner() == FinalizedFragmentOwner::Origin(origin.clone()))
            .unwrap();
        assert_eq!(empty.local_range(), Some(Range::new(0, 0)));
        assert_eq!(empty.aggregate_range(), Some(Range::new(1, 1)));
        assert_eq!(empty.bounds(), Some(Rect::new(20.0, 26.0, 0.0, 0.0)));
        let layout = crate::Layout::from_finalized_lines(PlacementRequest {
            prepared,
            coverage: PlacementCoverage::WholeParagraph,
            entries: vec![crate::PlacementEntry::Formatted(LinePlacement {
                line,
                anchor: LineAnchor {
                    inline_origin: 7.5,
                    baseline: 70.25,
                },
                orientation: LineOrientation::Horizontal,
                atomics: vec![],
            })],
        })
        .unwrap();
        assert_eq!(
            layout
                .cursor(crate::Cursor::new(1, crate::Affinity::Before))
                .unwrap()
                .rect(),
            Rect::new(27.5, 44.25, 1.0, 40.0)
        );
        let standalone = actual_line("");
        assert_eq!(
            standalone.fragment_views().next().unwrap().bounds(),
            Some(Rect::new(0.0, 0.0, 0.0, 20.0))
        );
        #[cfg(feature = "text-accessibility")]
        {
            let mut state = crate::Accessibility::new(10);
            let parent = accesskit::NodeId(1000);
            let update = layout
                .accessibility_update(&mut state, parent, parent, Point::default())
                .unwrap();
            let text = update
                .nodes
                .iter()
                .find(|(_, node)| node.role() == accesskit::Role::TextRun)
                .unwrap();
            let bounds = text.1.bounds().unwrap();
            assert_eq!((bounds.y0, bounds.y1), (44.25, 84.25));
        }
    }
}
