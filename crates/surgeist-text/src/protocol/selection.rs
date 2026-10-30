//! Admission of a complete request against one immutable preparation.

use std::sync::Arc;

use super::{ControlAllocation, LineSelectionRequest};
use crate::engine::PreparedData;

/// Shared selected-request owner; issued spacing slots retain this allocation.
/// Finalized rows retain it so context and occurrence admission are not replayed.
pub(crate) struct SelectedData {
    pub(crate) prepared: Arc<PreparedData>,
    pub(crate) scopes: crate::engine::scope_alignment::SelectedScopes,
    pub(crate) request: LineSelectionRequest,
    pub(crate) presentation: super::prepared::SelectedPresentation,
    pub(crate) spacing_slots: Vec<SpacingSlotData>,
    pub(crate) metric_policy: LineMetricPolicy,
    pub(crate) basis: crate::engine::SelectedShape,
    pub(crate) boundaries: super::boundary::Resolved,
}

impl SelectedData {
    pub(crate) fn presentation_input(&self) -> super::input::PresentationInput<'_> {
        super::input::PresentationInput {
            prepared: &self.prepared,
            presentation: &self.presentation,
            boundaries: &self.boundaries,
            start: self.request.start.index,
            computed: self.request.alternative.is_some() || self.request.incoming.is_some(),
            metric_policy: self.metric_policy,
            controls: &self.request.controls,
        }
    }
}

/// One complete semantic candidate; no provider width or eligibility is guessed.
pub(crate) struct SpacingSlotData {
    pub(crate) target: SpacingTargetData,
    pub(crate) participants: Vec<usize>,
    pub(crate) pieces: Vec<SpacingSlotPieceData>,
}

pub(crate) enum SpacingTargetData {
    Separator {
        atoms: std::ops::Range<usize>,
        base: usize,
        unit: SpacingUnitData,
    },
    Boundary {
        edge: usize,
        units: [SpacingUnitData; 2],
    },
}
impl SpacingTargetData {
    pub(crate) fn unit(&self, side: usize) -> &SpacingUnitData {
        match self {
            Self::Separator { unit, .. } => {
                assert_eq!(side, 0);
                unit
            }
            Self::Boundary { units, .. } => &units[side],
        }
    }
}
pub(crate) struct SpacingUnitData {
    pub(crate) participants: Vec<usize>,
    pub(crate) pieces: std::ops::Range<usize>,
    pub(crate) left: f64,
    pub(crate) right: f64,
}

pub(crate) struct SpacingSlotPieceData {
    pub(crate) owner: crate::SpacingPiece,
    pub(crate) presentation: super::PresentationRange,
}

/// Existing standalone pixel metric policy is not a preparation input.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) enum LineMetricPolicy {
    Native,
    Exact,
    StandaloneQuantized,
}

use std::collections::BTreeSet;

use super::prepared::{ParticipantData, PreparedBreakKind};
use super::{
    AtomicAllocation, LineContext, LineEnd, LineOrientation, LineStart, ParagraphBaseDirection,
    PreparedParticipant, PreparedText, PresentationRange, ProtocolContextField, ProtocolError,
    ProtocolHandleKind, SelectedLine, SpacingPolicy, TabContext, protocol_error,
};
use crate::{InlineBoxKind, Result, TextNumericError, TextNumericPhase};

impl PreparedText {
    /// Establishes this exact selected shaping basis and complete spacing handles.
    /// Retained resources are used; no fitting, placement or fallback selection occurs.
    pub fn select(&self, request: LineSelectionRequest) -> Result<SelectedLine> {
        select(self, request, LineMetricPolicy::Native)
    }

    /// Select using a caller-reduced feature allowance, including retained input.
    /// The allowance changes resource admission, never source or geometry identity.
    pub fn select_with_scope_allowance(
        &self,
        request: LineSelectionRequest,
        allowance: crate::ScopeAlignmentAllowance,
    ) -> Result<SelectedLine> {
        select_with_policy(
            self,
            request,
            LineMetricPolicy::Native,
            &mut crate::engine::shape::ShapeContext::default(),
            Some(allowance),
        )
    }

    /// Return the first actual off-selection endpoint still needed by retained
    /// shaping context. No shaping or font catalogue query is performed.
    pub fn boundary_context_demand(
        &self,
        request: &LineSelectionRequest,
    ) -> Result<Option<super::PreparedControl>> {
        let mut presentation = admit(self, request)?;
        super::boundary::resolve(self, request.into(), &mut presentation)
            .map(|(_, missing)| missing)
    }
}

pub(crate) fn select(
    prepared: &PreparedText,
    request: LineSelectionRequest,
    metric_policy: LineMetricPolicy,
) -> Result<SelectedLine> {
    select_with_scratch(
        prepared,
        request,
        metric_policy,
        &mut crate::engine::shape::ShapeContext::default(),
    )
}

/// Standalone selection shares its existing bounded shaping scratch.
pub(crate) fn select_with_scratch(
    prepared: &PreparedText,
    request: LineSelectionRequest,
    metric_policy: LineMetricPolicy,
    scratch: &mut crate::engine::shape::ShapeContext,
) -> Result<SelectedLine> {
    select_with_policy(prepared, request, metric_policy, scratch, None)
}

fn select_with_policy(
    prepared: &PreparedText,
    request: LineSelectionRequest,
    metric_policy: LineMetricPolicy,
    scratch: &mut crate::engine::shape::ShapeContext,
    allowance: Option<crate::ScopeAlignmentAllowance>,
) -> Result<SelectedLine> {
    let mut presentation = admit(prepared, &request)?;
    let (boundaries, missing) =
        super::boundary::resolve(prepared, (&request).into(), &mut presentation)?;
    if let Some(missing) = missing {
        return Err(protocol_error(ProtocolError::MissingBoundaryContext {
            occurrence: missing.index,
        }));
    }
    scope_request_roots(&prepared.inner, &request)?;
    crate::engine::scope_alignment::operation_limits(&prepared.inner, allowance)?;
    let input = super::input::PresentationInput {
        prepared: &prepared.inner,
        presentation: &presentation,
        boundaries: &boundaries,
        start: request.start.index,
        computed: request.alternative.is_some() || request.incoming.is_some(),
        metric_policy,
        controls: &request.controls,
    };
    // No handles exist until this exact selected invocation has completed.
    let (shaped, mapping, scope_plan) = crate::engine::shape::finalize(
        input,
        scratch,
        request.context.spacing,
        allowance,
        crate::engine::scope_alignment::PlanInput::Selection(&request),
    )?;
    crate::engine::layout::qualify_selected_paint(input, &shaped, &request.participants)?;
    let characters = crate::engine::spacing::character::build(
        &presentation.projection,
        &presentation.analysis,
        &prepared.inner.styles,
        prepared.inner.analyzed.processed.source().boxes(),
        Some(&presentation),
        request.context.spacing == SpacingPolicy::InterCharacter,
        Some(&shaped.character_exclusions),
    )?;
    crate::engine::spacing::character::admit(
        input,
        &characters,
        request.context.spacing == SpacingPolicy::InterCharacter,
    )?;
    let order = super::prepared::presentation::order_presentation(
        &presentation,
        prepared.inner.analyzed.processed.source().boxes(),
        &shaped,
    )?;
    let targets = match request.context.spacing {
        SpacingPolicy::None | SpacingPolicy::InterCharacter => Vec::new(),
        SpacingPolicy::Spaces => presentation
            .units
            .iter()
            .filter(|unit| unit.eligible_space)
            .map(|unit| crate::engine::spacing::SeparatorCandidate {
                atoms: unit.atoms.clone(),
                base: unit.atoms.start,
            })
            .collect(),
        SpacingPolicy::InterWord => crate::engine::spacing::candidates(
            &presentation.projection,
            &presentation.analysis,
            &boundaries.presentation,
        )?,
    };
    let mut participant_at = vec![None; presentation.projection.atoms.len()];
    for unit in &presentation.units {
        for atom in unit.atoms.clone() {
            participant_at[atom] = unit.participant;
        }
    }
    let mut spacing_slots: Vec<_> = targets
        .into_iter()
        .map(|candidate| {
            let atoms = candidate.atoms;
            let mut participants = Vec::new();
            for atom in atoms.clone() {
                if let Some(index) = participant_at[atom]
                    && participants.last() != Some(&index)
                {
                    participants.push(index);
                }
            }
            let pieces =
                crate::engine::spacing::slot_pieces(&prepared.inner, &presentation, atoms.clone());
            SpacingSlotData {
                target: SpacingTargetData::Separator {
                    atoms,
                    base: candidate.base,
                    unit: SpacingUnitData {
                        participants: participants.clone(),
                        pieces: 0..pieces.len(),
                        left: prepared.inner.styles[presentation.analysis.infos[candidate.base].1]
                            .style
                            .letter_spacing,
                        right: prepared.inner.styles[presentation.analysis.infos[candidate.base].1]
                            .style
                            .letter_spacing,
                    },
                },
                participants,
                pieces,
            }
        })
        .collect();
    if request.context.spacing == SpacingPolicy::InterCharacter {
        for (edge, boundary) in characters.boundaries.iter().enumerate() {
            let mut pieces = Vec::new();
            let mut participants = Vec::new();
            let mut units = Vec::new();
            for index in [boundary.before, boundary.after] {
                let unit = &characters.units[index];
                let start = pieces.len();
                let mut own = Vec::new();
                for atoms in &unit.atoms {
                    pieces.extend(crate::engine::spacing::slot_pieces(
                        &prepared.inner,
                        &presentation,
                        atoms.clone(),
                    ));
                    for atom in atoms.clone() {
                        if let Some(i) = participant_at[atom]
                            && !own.contains(&i)
                        {
                            own.push(i);
                        }
                    }
                }
                for &occurrence in &unit.atomics {
                    let index = presentation.units.iter().position(|u| matches!(u.kind, super::prepared::PresentationUnitKind::Atomic { occurrence: i } if i == occurrence)).expect("selected atomic");
                    let actual = &presentation.units[index];
                    pieces.push(SpacingSlotPieceData {
                        owner: crate::SpacingPiece::Atomic { occurrence },
                        presentation: PresentationRange {
                            start: actual.presentation.start,
                            end: actual.presentation.end,
                        },
                    });
                    if let Some(i) = actual.participant
                        && !own.contains(&i)
                    {
                        own.push(i);
                    }
                }
                for &i in &own {
                    if !participants.contains(&i) {
                        participants.push(i);
                    }
                }
                units.push(SpacingUnitData {
                    participants: own,
                    pieces: start..pieces.len(),
                    left: unit.left,
                    right: unit.right,
                });
            }
            let units: [SpacingUnitData; 2] = units
                .try_into()
                .unwrap_or_else(|_| unreachable!("two endpoints"));
            spacing_slots.push(SpacingSlotData {
                target: SpacingTargetData::Boundary { edge, units },
                participants,
                pieces,
            });
        }
    }
    let scopes = crate::engine::scope_alignment::select(
        &prepared.inner,
        &presentation,
        &shaped,
        &request,
        scope_plan,
    )?;
    Ok(SelectedLine {
        inner: Arc::new(SelectedData {
            scopes,
            prepared: Arc::clone(&prepared.inner),
            request,
            presentation,
            spacing_slots,
            metric_policy,
            basis: crate::engine::SelectedShape {
                shaped,
                mapping,
                characters,
                order,
            },
            boundaries,
        }),
    })
}

fn admit(
    prepared: &PreparedText,
    request: &LineSelectionRequest,
) -> Result<super::prepared::SelectedPresentation> {
    let input = super::input::RequestInput::from(request);
    let (start, end) = admit_range(prepared, input)?;
    let segment = super::segments::admit(prepared, &request.segment)?;
    let tables = &prepared.inner.tables;
    let bounds_error = || {
        protocol_error(ProtocolError::InvalidBoundaryRange {
            start: request.start.index,
            end: request.end.index,
            participants: tables.participants.len(),
        })
    };
    let ordinary = match &request.termination {
        super::PreparedTermination::Ordinary(termination) => {
            super::segments::admit_opportunity(prepared, &request.segment, termination)?;
            require_prepared(
                prepared,
                &termination.prepared,
                ProtocolHandleKind::Opportunity,
                termination.index,
            )?;
            let opportunity = tables.opportunities.get(termination.index).ok_or_else(|| {
                protocol_error(ProtocolError::IllegalOpportunity {
                    opportunity: termination.index,
                })
            })?;
            if opportunity.boundary != request.end.index {
                return Err(protocol_error(ProtocolError::IllegalOpportunity {
                    opportunity: termination.index,
                }));
            }
            if let Some(actual) = opportunity.source_segment
                && actual != request.segment.index
            {
                return Err(protocol_error(ProtocolError::MismatchedSourceSegment {
                    expected: request.segment.index,
                    actual,
                }));
            }
            Some((termination.index, opportunity))
        }
        super::PreparedTermination::Conditional(termination) => {
            termination.validate(prepared, request)?;
            None
        }
        super::PreparedTermination::PhraseMeasurement(portion) => {
            portion.validate(prepared, request)?;
            None
        }
        super::PreparedTermination::Measurement(portion) => {
            portion.validate(prepared, request)?;
            None
        }
    };
    // Opportunities are issued in mixed-boundary order. A mandatory break
    // consumed by this interval must be its actual termination, even when a
    // paragraph-end opportunity shares that final boundary. Breaks at start
    // have already been consumed by the preceding selected line.
    let first = tables
        .opportunities
        .partition_point(|row| tables.boundaries[row.boundary].participant <= start.participant);
    if tables
        .opportunities
        .iter()
        .enumerate()
        .skip(first)
        .take_while(|(_, row)| tables.boundaries[row.boundary].participant <= end.participant)
        .find(|(_, row)| {
            row.kind == PreparedBreakKind::Forced
                && row.source_segment == Some(request.segment.index)
        })
        .is_some_and(|(index, _)| ordinary.is_none_or(|(actual, _)| index != actual))
    {
        return Err(protocol_error(match ordinary {
            Some((opportunity, _)) => ProtocolError::IllegalOpportunity { opportunity },
            None => ProtocolError::InvalidConditionalSelection {
                start: request.start.index,
                end: request.end.index,
            },
        }));
    }
    if start.participant == end.participant
        && ordinary.is_none_or(|(_, row)| row.kind != PreparedBreakKind::ParagraphEnd)
    {
        return Err(bounds_error());
    }
    let actual_end = match ordinary.map(|(_, row)| row.kind) {
        None | Some(PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary) => LineEnd::Soft,
        Some(PreparedBreakKind::Forced) => LineEnd::Forced,
        Some(PreparedBreakKind::ParagraphEnd) => LineEnd::Paragraph,
    };
    if request.context.end != actual_end {
        return Err(protocol_error(ProtocolError::MismatchedContext {
            field: ProtocolContextField::LineEnd,
        }));
    }
    let segment_start = &tables.boundaries[segment.start];
    let actual_start = if start.participant == segment_start.participant {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    if request.context.start != actual_start {
        return Err(protocol_error(ProtocolError::MismatchedContext {
            field: ProtocolContextField::LineStart,
        }));
    }
    if let Some((index, opportunity)) = ordinary
        && opportunity.kind == PreparedBreakKind::Discretionary
        && request.alternative.is_none()
    {
        return Err(protocol_error(ProtocolError::MissingAlternative {
            opportunity: index,
        }));
    }
    if let Some(alternative) = &request.alternative
        && let Some((index, opportunity)) = ordinary
    {
        require_prepared(
            prepared,
            &alternative.prepared,
            ProtocolHandleKind::Alternative,
            alternative.index,
        )?;
        if !opportunity.alternatives.contains(&alternative.index)
            || tables
                .alternatives
                .get(alternative.index)
                .is_none_or(|row| row.opportunity != Some(index))
        {
            return Err(protocol_error(ProtocolError::ForeignAlternative {
                alternative: alternative.index,
                opportunity: index,
            }));
        }
    }
    admit_geometry(prepared, input)
}

pub(crate) fn admit_range<'a>(
    prepared: &'a PreparedText,
    request: super::input::RequestInput<'_>,
) -> Result<(
    &'a super::prepared::BoundaryData,
    &'a super::prepared::BoundaryData,
)> {
    super::segments::admit(prepared, request.segment)?;
    super::continuation::validate(prepared, request.segment, request.start, request.incoming)?;
    let tables = &prepared.inner.tables;
    require_prepared(
        prepared,
        &request.start.prepared,
        ProtocolHandleKind::Boundary,
        request.start.index,
    )?;
    require_prepared(
        prepared,
        &request.end.prepared,
        ProtocolHandleKind::Boundary,
        request.end.index,
    )?;
    let bounds_error = || {
        protocol_error(ProtocolError::InvalidBoundaryRange {
            start: request.start.index,
            end: request.end.index,
            participants: tables.participants.len(),
        })
    };
    let start = tables
        .boundaries
        .get(request.start.index)
        .ok_or_else(bounds_error)?;
    let end = tables
        .boundaries
        .get(request.end.index)
        .ok_or_else(bounds_error)?;
    if start.participant > end.participant
        || start.relation_cut > end.relation_cut
        || end.participant > tables.participants.len()
        || request.participants.len() != end.participant - start.participant
    {
        return Err(bounds_error());
    }
    super::segments::admit_boundaries(prepared, request.segment, request.start, request.end)?;
    for (position, (actual, expected)) in request
        .participants
        .iter()
        .zip(&tables.participants[start.participant..end.participant])
        .enumerate()
    {
        let (owner, index, kind, matches_kind) = match (actual, expected) {
            (PreparedParticipant::Text(value), ParticipantData::Text(index)) => (
                &value.prepared,
                value.index,
                ProtocolHandleKind::Unit,
                value.index == *index,
            ),
            (PreparedParticipant::Atomic(value), ParticipantData::Atomic(index)) => (
                &value.prepared,
                value.index,
                ProtocolHandleKind::Atomic,
                value.index == *index,
            ),
            (
                PreparedParticipant::ComputedLineBoundary(value),
                ParticipantData::ComputedLineBoundary(index),
            ) => (
                &value.prepared,
                value.index,
                ProtocolHandleKind::ComputedLineBoundary,
                value.index == *index,
            ),
            (PreparedParticipant::Control(value), ParticipantData::Control(index)) => (
                &value.prepared,
                value.index,
                ProtocolHandleKind::Control,
                value.index == *index,
            ),
            _ => {
                return Err(protocol_error(ProtocolError::ParticipantMismatch {
                    position,
                }));
            }
        };
        require_prepared(prepared, owner, kind, index)?;
        if !matches_kind {
            return Err(protocol_error(ProtocolError::ParticipantMismatch {
                position,
            }));
        }
    }
    Ok((start, end))
}

pub(crate) fn admit_geometry(
    prepared: &PreparedText,
    request: super::input::RequestInput<'_>,
) -> Result<super::prepared::SelectedPresentation> {
    let tables = &prepared.inner.tables;
    validate_context(
        request.context,
        prepared.inner.analyzed.analysis.base_level,
        prepared.orientation(),
    )?;
    let mut expected_atomics = BTreeSet::new();
    for participant in request.participants {
        if let PreparedParticipant::Atomic(atomic) = participant
            && !expected_atomics.insert(atomic.index)
        {
            return Err(protocol_error(ProtocolError::DuplicateAtomic {
                occurrence: atomic.index,
            }));
        }
    }
    let mut observed_atomics = BTreeSet::new();
    for allocation in request.atomics {
        let atomic = &allocation.occurrence;
        require_prepared(
            prepared,
            &atomic.prepared,
            ProtocolHandleKind::Atomic,
            atomic.index,
        )?;
        if !expected_atomics.contains(&atomic.index) {
            return Err(protocol_error(ProtocolError::UnexpectedAtomic {
                occurrence: atomic.index,
            }));
        }
        if !observed_atomics.insert(atomic.index) {
            return Err(protocol_error(ProtocolError::DuplicateAtomic {
                occurrence: atomic.index,
            }));
        }
        validate_atomic(allocation)?;
        let row = &tables.atomics[atomic.index];
        let in_flow = prepared.inner.analyzed.processed.source().boxes()[row.occurrence].kind()
            == InlineBoxKind::InFlow;
        if in_flow
            != matches!(
                allocation.allocation,
                super::AtomicLineAllocation::InFlow { .. }
            )
        {
            return Err(protocol_error(ProtocolError::InvalidAtomicGeometry {
                occurrence: atomic.index,
            }));
        }
    }
    if let Some(occurrence) = expected_atomics.difference(&observed_atomics).next() {
        return Err(protocol_error(ProtocolError::MissingAtomic {
            occurrence: *occurrence,
        }));
    }
    let expected_controls: BTreeSet<_> = request
        .participants
        .iter()
        .filter_map(|participant| {
            if let PreparedParticipant::Control(control) = participant {
                Some(control.index)
            } else {
                None
            }
        })
        .collect();
    let mut observed_controls = BTreeSet::new();
    for allocation in request.controls {
        let control = &allocation.occurrence;
        require_prepared(
            prepared,
            &control.prepared,
            ProtocolHandleKind::Control,
            control.index,
        )?;
        if !expected_controls.contains(&control.index) {
            return Err(protocol_error(ProtocolError::UnexpectedControl {
                occurrence: control.index,
            }));
        }
        if !observed_controls.insert(control.index) {
            return Err(protocol_error(ProtocolError::DuplicateControl {
                occurrence: control.index,
            }));
        }
        let kind = &prepared.inner.analyzed.paragraph.controls
            [tables.controls[control.index].occurrence]
            .kind;
        if super::boundary::is_endpoint(kind) {
            if allocation.boundary.is_none() {
                return Err(protocol_error(
                    ProtocolError::MissingControlBoundaryGeometry {
                        occurrence: control.index,
                    },
                ));
            }
        } else if allocation.boundary.is_some() {
            return Err(protocol_error(ProtocolError::InvalidControlGeometry {
                occurrence: control.index,
            }));
        }
        if let Some(strut) = allocation.strut {
            finite("control baseline", strut.baseline)?;
            finite("control line extent", strut.line_extent)?;
            if strut.baseline < 0.0 || strut.line_extent < strut.baseline {
                return Err(protocol_error(ProtocolError::InvalidControlGeometry {
                    occurrence: control.index,
                }));
            }
            let kind = &prepared.inner.analyzed.paragraph.controls
                [tables.controls[control.index].occurrence]
                .kind;
            if !matches!(
                kind,
                super::paragraph::ParagraphControlKind::ForcedBreak
                    | super::paragraph::ParagraphControlKind::InlineBoundaryStart
                    | super::paragraph::ParagraphControlKind::InlineBoundaryEnd
            ) {
                return Err(protocol_error(ProtocolError::InvalidControlGeometry {
                    occurrence: control.index,
                }));
            }
        }
    }
    if let Some(occurrence) = expected_controls.difference(&observed_controls).next() {
        return Err(protocol_error(ProtocolError::MissingControl {
            occurrence: *occurrence,
        }));
    }
    for allocation in request.line_box_struts {
        finite("line box baseline", allocation.metrics.baseline)?;
        finite("line box extent", allocation.metrics.line_extent)?;
        if allocation.metrics.baseline < 0.0
            || allocation.metrics.line_extent < allocation.metrics.baseline
        {
            return Err(protocol_error(ProtocolError::InvalidLineStrut));
        }
    }
    if let Some(strut) = request.line_strut {
        finite("empty line baseline", strut.baseline)?;
        finite("empty line extent", strut.line_extent)?;
        if strut.baseline < 0.0 || strut.line_extent < strut.baseline {
            return Err(protocol_error(ProtocolError::InvalidLineStrut));
        }
    }
    if matches!(
        (&request.context.tabs, prepared.inner.options.tabs()),
        (TabContext::FontAdvance, crate::TabPreparation::CssStops)
            | (
                TabContext::CssStops { .. } | TabContext::IntrinsicCssStops,
                crate::TabPreparation::FontAdvance
            )
    ) {
        return Err(protocol_error(ProtocolError::MismatchedContext {
            field: ProtocolContextField::TabPreparation,
        }));
    }
    let presentation = super::prepared::select_presentation(&prepared.inner, request)?;
    if request.context.tabs == TabContext::NoTabs
        && presentation
            .units
            .iter()
            .any(|unit| unit.code_point == Some(u32::from(b'\t')))
    {
        return Err(protocol_error(ProtocolError::UnsupportedContext {
            field: ProtocolContextField::TabStops,
        }));
    }
    Ok(presentation)
}

fn require_data(
    expected: &PreparedData,
    actual: &Arc<PreparedData>,
    kind: ProtocolHandleKind,
    occurrence: usize,
) -> Result<()> {
    if std::ptr::eq(expected, actual.as_ref()) {
        Ok(())
    } else {
        Err(protocol_error(ProtocolError::ForeignPreparation {
            kind,
            occurrence,
        }))
    }
}
/// Pure root association, before any scoped workspace. Full semantic admission
/// remains with the existing request/flow owners and is not replayed by census.
pub(crate) fn scope_allocation_roots(
    prepared: &PreparedData,
    atomics: &[AtomicAllocation],
    controls: &[ControlAllocation],
    context: &[super::InlineBoundaryContextAllocation],
    struts: &[super::LineBoxStrut],
) -> Result<()> {
    for row in atomics {
        require_data(
            prepared,
            &row.occurrence.prepared,
            ProtocolHandleKind::Atomic,
            row.occurrence.index,
        )?;
    }
    for row in controls {
        require_data(
            prepared,
            &row.occurrence.prepared,
            ProtocolHandleKind::Control,
            row.occurrence.index,
        )?;
    }
    for row in context {
        require_data(
            prepared,
            &row.occurrence.prepared,
            ProtocolHandleKind::Control,
            row.occurrence.index,
        )?;
    }
    for row in struts {
        if let Some(scope) = &row.scope {
            let topology = prepared
                .analyzed
                .paragraph
                .scope_topology
                .as_ref()
                .ok_or_else(|| {
                    crate::engine::scope_alignment::refusal(
                        crate::ScopeAlignmentError::MissingCapture,
                    )
                })?;
            if !Arc::ptr_eq(&topology.capture, &scope.capture) {
                return Err(crate::engine::scope_alignment::refusal(
                    crate::ScopeAlignmentError::ForeignCapture,
                ));
            }
            if scope.capture.scopes[scope.index].domain != topology.domain {
                return Err(crate::engine::scope_alignment::refusal(
                    crate::ScopeAlignmentError::WrongDomain,
                ));
            }
        }
    }
    Ok(())
}
pub(crate) fn scope_request_roots(
    prepared: &PreparedData,
    request: &LineSelectionRequest,
) -> Result<()> {
    require_data(
        prepared,
        &request.segment.prepared,
        ProtocolHandleKind::SourceSegment,
        request.segment.index,
    )?;
    require_data(
        prepared,
        &request.start.prepared,
        ProtocolHandleKind::Boundary,
        request.start.index,
    )?;
    require_data(
        prepared,
        &request.end.prepared,
        ProtocolHandleKind::Boundary,
        request.end.index,
    )?;
    for row in &request.participants {
        let (owner, kind, index) = match row {
            super::PreparedParticipant::Text(row) => {
                (&row.prepared, ProtocolHandleKind::Unit, row.index)
            }
            super::PreparedParticipant::Atomic(row) => {
                (&row.prepared, ProtocolHandleKind::Atomic, row.index)
            }
            super::PreparedParticipant::Control(row) => {
                (&row.prepared, ProtocolHandleKind::Control, row.index)
            }
            super::PreparedParticipant::ComputedLineBoundary(row) => (
                &row.prepared,
                ProtocolHandleKind::ComputedLineBoundary,
                row.index,
            ),
        };
        require_data(prepared, owner, kind, index)?;
    }
    if let Some(row) = &request.alternative {
        require_data(
            prepared,
            &row.prepared,
            ProtocolHandleKind::Alternative,
            row.index,
        )?;
    }
    if let Some(row) = &request.incoming {
        require_data(
            prepared,
            &row.alternative.prepared,
            ProtocolHandleKind::Alternative,
            row.alternative.index,
        )?;
        require_data(
            prepared,
            &row.boundary.prepared,
            ProtocolHandleKind::Boundary,
            row.boundary.index,
        )?;
    }
    match &request.termination {
        super::PreparedTermination::Ordinary(row) => require_data(
            prepared,
            &row.prepared,
            ProtocolHandleKind::Opportunity,
            row.index,
        )?,
        super::PreparedTermination::Conditional(row) => require_data(
            prepared,
            &row.prepared_text().inner,
            ProtocolHandleKind::ConditionalOpportunity,
            row.boundary().index,
        )?,
        super::PreparedTermination::Measurement(row) => require_data(
            prepared,
            &row.word.prepared,
            ProtocolHandleKind::WordPortion,
            row.word.index,
        )?,
        super::PreparedTermination::PhraseMeasurement(row) => {
            if !std::ptr::eq(prepared, row.phrase.prepared.inner.as_ref()) {
                return Err(protocol_error(ProtocolError::InvalidPhrasePortion));
            }
        }
    }
    scope_allocation_roots(
        prepared,
        &request.atomics,
        &request.controls,
        &request.boundary_context,
        &request.line_box_struts,
    )
}

pub(super) fn require_prepared(
    expected: &PreparedText,
    actual: &Arc<PreparedData>,
    kind: ProtocolHandleKind,
    occurrence: usize,
) -> Result<()> {
    require_data(&expected.inner, actual, kind, occurrence)
}

fn validate_context(
    context: &LineContext,
    base_level: u8,
    orientation: LineOrientation,
) -> Result<()> {
    let actual = if base_level & 1 == 0 {
        ParagraphBaseDirection::LeftToRight
    } else {
        ParagraphBaseDirection::RightToLeft
    };
    if context.base_direction != actual {
        return Err(protocol_error(ProtocolError::MismatchedContext {
            field: ProtocolContextField::BaseDirection,
        }));
    }
    if context.orientation == LineOrientation::Vertical && orientation != LineOrientation::Vertical
    {
        return Err(protocol_error(ProtocolError::UnsupportedContext {
            field: ProtocolContextField::Orientation,
        }));
    }
    if context.orientation != orientation {
        return Err(protocol_error(ProtocolError::MismatchedContext {
            field: ProtocolContextField::Orientation,
        }));
    }
    if let TabContext::CssStops {
        origin: Some(origin),
    } = context.tabs
    {
        finite("tab origin", origin)?;
    }
    if context.tabs == TabContext::IntrinsicCssStops && context.available_inline.is_some() {
        return Err(protocol_error(ProtocolError::MismatchedContext {
            field: ProtocolContextField::TabStops,
        }));
    }
    if let Some(value) = context.available_inline {
        finite("available inline span", value)?;
        if value < 0.0 {
            return Err(numeric(
                "available inline span",
                TextNumericError::OutOfRange,
                value,
            ));
        }
    }
    if context.alignment_insets != super::LineAlignmentInsets::ZERO
        && context.available_inline.is_none()
    {
        return Err(protocol_error(ProtocolError::UnsupportedContext {
            field: ProtocolContextField::AlignmentInsets,
        }));
    }
    finite("line indent", context.indent)?;
    Ok(())
}

fn validate_atomic(allocation: &AtomicAllocation) -> Result<()> {
    let super::AtomicLineAllocation::InFlow {
        flow, box_geometry, ..
    } = allocation.allocation
    else {
        return Ok(());
    };
    for (field, value) in [
        ("atomic flow advance", flow.advance),
        ("atomic before baseline", flow.before_baseline),
        ("atomic after baseline", flow.after_baseline),
        ("atomic occupied start", flow.occupied_start),
        ("atomic occupied end", flow.occupied_end),
        ("atomic baseline", box_geometry.baseline),
    ] {
        finite(field, value)?;
    }
    if flow.occupied_start > flow.occupied_end {
        return Err(protocol_error(ProtocolError::InvalidAtomicGeometry {
            occurrence: allocation.occurrence.index,
        }));
    }
    for rect in [box_geometry.border, box_geometry.content] {
        for (field, value) in [
            ("atomic x", rect.origin.x),
            ("atomic y", rect.origin.y),
            ("atomic width", rect.size.width),
            ("atomic height", rect.size.height),
            ("atomic right", rect.origin.x + rect.size.width),
            ("atomic bottom", rect.origin.y + rect.size.height),
        ] {
            finite(field, value)?;
        }
        if rect.size.width < 0.0 || rect.size.height < 0.0 {
            return Err(protocol_error(ProtocolError::InvalidAtomicGeometry {
                occurrence: allocation.occurrence.index,
            }));
        }
    }
    Ok(())
}
fn finite(field: &'static str, value: f64) -> Result<()> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(numeric(field, TextNumericError::NonFinite, value))
    }
}
fn numeric(field: &'static str, category: TextNumericError, value: f64) -> crate::Error {
    crate::error::text_numeric_error(field, TextNumericPhase::Selection, category, value)
}

/// Build the actual raw operation request at genuine issued mixed boundaries.
/// Native callers supply their own allocations; raw boxes retain Source geometry.
pub(crate) fn standalone_request(
    prepared: Arc<PreparedData>,
    start: usize,
    termination: super::PreparedTermination,
    alternative: Option<super::PreparedAlternative>,
    available: Option<f64>,
    indent: f64,
    alignment: crate::Alignment,
) -> Result<LineSelectionRequest> {
    use super::PreparedBoundary;
    let end = match &termination {
        super::PreparedTermination::Ordinary(offer) => offer.view().boundary().index,
        super::PreparedTermination::Conditional(cut) => cut.boundary().index,
        super::PreparedTermination::Measurement(portion) => portion.end,
        super::PreparedTermination::PhraseMeasurement(portion) => portion.end,
    };
    let tables = &prepared.tables;
    let invalid = || {
        protocol_error(ProtocolError::InvalidBoundaryRange {
            start,
            end,
            participants: tables.participants.len(),
        })
    };
    if tables.profile != crate::engine::PreparationProfile::Standalone {
        return Err(invalid());
    }
    if tables.source_segments.len() != 1 {
        return Err(protocol_error(ProtocolError::UnsupportedStandaloneAnalysis));
    }
    let ending = match &termination {
        super::PreparedTermination::Ordinary(offer) => match tables.opportunities[offer.index].kind
        {
            PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary => LineEnd::Soft,
            PreparedBreakKind::Forced => LineEnd::Forced,
            PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
        },
        super::PreparedTermination::Measurement(_)
        | super::PreparedTermination::PhraseMeasurement(_)
        | super::PreparedTermination::Conditional(_) => LineEnd::Soft,
    };
    let from = tables.boundaries[start].participant;
    let to = tables.boundaries[end].participant;
    if from > to {
        return Err(invalid());
    }
    let handle = PreparedText {
        inner: Arc::clone(&prepared),
    };
    let participants = handle.participants().skip(from).take(to - from).collect();
    let atomics = standalone_atomics(&prepared, start, end)?;
    let request = LineSelectionRequest {
        segment: handle.source_segments().next().ok_or_else(invalid)?,
        boundary_context: Vec::new(),
        incoming: None,
        start: PreparedBoundary {
            prepared: Arc::clone(&prepared),
            index: start,
        },
        end: PreparedBoundary {
            prepared: Arc::clone(&prepared),
            index: end,
        },
        participants,
        termination,
        alternative,
        context: LineContext {
            alignment_insets: crate::LineAlignmentInsets::ZERO,
            formatted_line: super::FormattedLineContext::default(),
            base_direction: if prepared.analyzed.analysis.base_level & 1 == 0 {
                ParagraphBaseDirection::LeftToRight
            } else {
                ParagraphBaseDirection::RightToLeft
            },
            // Raw Standalone owns one container and explicitly retains its
            // resolved raw bidi profile as the line-box geometry policy.
            line_box_direction: if prepared.analyzed.analysis.base_level & 1 == 0 {
                super::LineBoxDirection::LeftToRight
            } else {
                super::LineBoxDirection::RightToLeft
            },
            orientation: LineOrientation::Horizontal,
            start: if from == 0 {
                LineStart::Paragraph
            } else {
                LineStart::Continuation
            },
            end: ending,
            available_inline: available,
            indent,
            alignment,
            tabs: TabContext::FontAdvance,

            spacing: SpacingPolicy::Spaces,
        },
        atomics,
        controls: Vec::new(),
        line_box_struts: Vec::new(),
        line_strut: None,
    };
    Ok(request)
}

/// Actual raw Source-box allocations shared by selection and nonplaceable flow.
pub(crate) fn standalone_atomics(
    prepared: &Arc<PreparedData>,
    start: usize,
    end: usize,
) -> Result<Vec<AtomicAllocation>> {
    let tables = &prepared.tables;
    if tables.profile != crate::engine::PreparationProfile::Standalone
        || start > end
        || end >= tables.boundaries.len()
    {
        return Err(protocol_error(ProtocolError::InvalidMeasurementFlow));
    }
    let from = tables.boundaries[start].participant;
    let to = tables.boundaries[end].participant;
    use super::{AtomicBoxAllocation, AtomicFlowAllocation, PreparedAtomic};
    Ok(tables
        .atomics
        .iter()
        .enumerate()
        .filter(|(_, row)| row.participant >= from && row.participant < to)
        .map(|(index, row)| {
            let item = prepared.analyzed.processed.source().boxes()[row.occurrence];
            let size = item.size();
            let in_flow = item.kind() == InlineBoxKind::InFlow;
            AtomicAllocation {
                occurrence: PreparedAtomic {
                    prepared: Arc::clone(prepared),
                    index,
                },
                allocation: if in_flow {
                    super::AtomicLineAllocation::InFlow {
                        flow: AtomicFlowAllocation {
                            advance: size.width,
                            before_baseline: size.height,
                            after_baseline: 0.0,
                            occupied_start: 0.0,
                            occupied_end: size.width,
                        },
                        alignment: super::LineBoxAlignment::Baseline,
                        box_geometry: AtomicBoxAllocation {
                            border: crate::Rect::new(0.0, -size.height, size.width, size.height),
                            content: crate::Rect::new(0.0, -size.height, size.width, size.height),
                            baseline: 0.0,
                        },
                    }
                } else {
                    super::AtomicLineAllocation::OutOfFlow
                },
            }
        })
        .collect())
}
