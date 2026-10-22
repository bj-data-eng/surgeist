//! Admission of a complete request against one immutable preparation.

use std::sync::Arc;

use super::LineSelectionRequest;
use crate::{Range, engine::PreparedData};

/// Shared selected-request owner; issued spacing slots retain this allocation.
/// Finalized rows retain it so context and occurrence admission are not replayed.
pub(crate) struct SelectedData {
    pub(crate) prepared: Arc<PreparedData>,
    pub(crate) request: LineSelectionRequest,
    pub(crate) presentation: super::prepared::SelectedPresentation,
    pub(crate) spacing_slots: Vec<SpacingSlotData>,
    pub(crate) metric_policy: LineMetricPolicy,
}

/// A supported logical space contribution, before any provider-independent delta.
pub(crate) struct SpacingSlotData {
    /// Exact selected presentation unit; replacement and original ownership
    /// cannot be recovered from equal source ranges or insertion anchors.
    pub(crate) unit: usize,
    pub(crate) participant: Option<usize>,
    pub(crate) presentation: super::PresentationRange,
    pub(crate) source: Range,
}

/// Existing standalone pixel metric policy is not a preparation input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LineMetricPolicy {
    Native,
    Exact,
    StandaloneQuantized,
}

use std::collections::BTreeSet;

use super::prepared::{ParticipantData, PreparedBreakKind};
use super::{
    AtomicAllocation, HangingContext, LineContext, LineEnd, LineOrientation, LineStart,
    ParagraphBaseDirection, PreparedParticipant, PreparedText, PresentationRange,
    ProtocolContextField, ProtocolError, ProtocolHandleKind, SelectedLine, SpacingPolicy,
    TabContext, protocol_error,
};
use crate::{InlineBoxKind, Result, TextNumericError, TextNumericPhase};

impl PreparedText {
    /// Checks the entire mixed request against this exact captured preparation.
    /// No line is shaped, fitted, placed, or selected as a fallback here.
    pub fn select(&self, request: LineSelectionRequest) -> Result<SelectedLine> {
        select(self, request, LineMetricPolicy::Native)
    }
}

fn select(
    prepared: &PreparedText,
    request: LineSelectionRequest,
    metric_policy: LineMetricPolicy,
) -> Result<SelectedLine> {
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
        || end.participant > tables.participants.len()
        || request.participants.len() != end.participant - start.participant
    {
        return Err(bounds_error());
    }
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
    require_prepared(
        prepared,
        &request.termination.prepared,
        ProtocolHandleKind::Opportunity,
        request.termination.index,
    )?;
    let opportunity = tables
        .opportunities
        .get(request.termination.index)
        .ok_or_else(|| {
            protocol_error(ProtocolError::IllegalOpportunity {
                opportunity: request.termination.index,
            })
        })?;
    if opportunity.boundary != request.end.index {
        return Err(protocol_error(ProtocolError::IllegalOpportunity {
            opportunity: request.termination.index,
        }));
    }
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
        .find(|(_, row)| row.kind == PreparedBreakKind::Forced)
        .is_some_and(|(index, _)| index != request.termination.index)
    {
        return Err(protocol_error(ProtocolError::IllegalOpportunity {
            opportunity: request.termination.index,
        }));
    }
    if start.participant == end.participant && opportunity.kind != PreparedBreakKind::ParagraphEnd {
        return Err(bounds_error());
    }
    let actual_end = match opportunity.kind {
        PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary => LineEnd::Soft,
        PreparedBreakKind::Forced => LineEnd::Forced,
        PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
    };
    if request.context.end != actual_end {
        return Err(protocol_error(ProtocolError::MismatchedContext {
            field: ProtocolContextField::LineEnd,
        }));
    }
    let actual_start = if start.participant == 0 {
        LineStart::Paragraph
    } else {
        LineStart::Continuation
    };
    if request.context.start != actual_start {
        return Err(protocol_error(ProtocolError::MismatchedContext {
            field: ProtocolContextField::LineStart,
        }));
    }
    if opportunity.kind == PreparedBreakKind::Discretionary && request.alternative.is_none() {
        return Err(protocol_error(ProtocolError::MissingAlternative {
            opportunity: request.termination.index,
        }));
    }
    if let Some(alternative) = &request.alternative {
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
                .is_none_or(|row| row.opportunity != request.termination.index)
        {
            return Err(protocol_error(ProtocolError::ForeignAlternative {
                alternative: alternative.index,
                opportunity: request.termination.index,
            }));
        }
    }
    validate_context(
        &request.context,
        prepared.inner.analyzed.analysis.base_level,
    )?;
    let mut expected_atomics = BTreeSet::new();
    for participant in &request.participants {
        if let PreparedParticipant::Atomic(atomic) = participant
            && !expected_atomics.insert(atomic.index)
        {
            return Err(protocol_error(ProtocolError::DuplicateAtomic {
                occurrence: atomic.index,
            }));
        }
    }
    let mut observed_atomics = BTreeSet::new();
    for allocation in &request.atomics {
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
    for allocation in &request.controls {
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
        if let Some(strut) = allocation.strut {
            finite("control baseline", strut.baseline)?;
            finite("control line extent", strut.line_extent)?;
            if strut.baseline < 0.0 || strut.line_extent < strut.baseline {
                return Err(protocol_error(ProtocolError::InvalidControlGeometry {
                    occurrence: control.index,
                }));
            }
            let kind = tables.controls[control.index].kind;
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
    if let Some(strut) = request.line_strut {
        finite("empty line baseline", strut.baseline)?;
        finite("empty line extent", strut.line_extent)?;
        if strut.baseline < 0.0 || strut.line_extent < strut.baseline {
            return Err(protocol_error(ProtocolError::InvalidLineStrut));
        }
    }
    let presentation = super::prepared::select_presentation(&prepared.inner, &request)?;
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
    let spacing_slots = if request.context.spacing == SpacingPolicy::Spaces {
        presentation
            .units
            .iter()
            .enumerate()
            .filter(|(_, unit)| unit.eligible_space)
            .map(|(index, unit)| SpacingSlotData {
                unit: index,
                participant: unit.participant,
                presentation: PresentationRange {
                    start: unit.presentation.start,
                    end: unit.presentation.end,
                },
                source: unit.source,
            })
            .collect()
    } else {
        Vec::new()
    };
    Ok(SelectedLine {
        inner: Arc::new(SelectedData {
            prepared: Arc::clone(&prepared.inner),
            request,
            presentation,
            spacing_slots,
            metric_policy,
        }),
    })
}

fn require_prepared(
    expected: &PreparedText,
    actual: &Arc<PreparedData>,
    kind: ProtocolHandleKind,
    occurrence: usize,
) -> Result<()> {
    if Arc::ptr_eq(&expected.inner, actual) {
        Ok(())
    } else {
        Err(protocol_error(ProtocolError::ForeignPreparation {
            kind,
            occurrence,
        }))
    }
}

fn validate_context(context: &LineContext, base_level: u8) -> Result<()> {
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
    if context.orientation != LineOrientation::Horizontal {
        return Err(protocol_error(ProtocolError::UnsupportedContext {
            field: ProtocolContextField::Orientation,
        }));
    }
    if matches!(context.tabs, TabContext::Stops { .. }) {
        return Err(protocol_error(ProtocolError::UnsupportedContext {
            field: ProtocolContextField::TabStops,
        }));
    }
    if context.hanging != HangingContext::None {
        return Err(protocol_error(ProtocolError::UnsupportedContext {
            field: ProtocolContextField::Hanging,
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

/// Lower only the characterized identity-processing standalone chooser. Native
/// paragraph capture always supplies its actual issued boundaries directly.
pub(crate) fn lower_standalone(
    prepared: Arc<PreparedData>,
    selection: crate::engine::LineSelection,
    metric_policy: LineMetricPolicy,
) -> Result<Arc<SelectedData>> {
    use super::{
        AtomicBoxAllocation, AtomicFlowAllocation, PreparedAtomic, PreparedBoundary,
        PreparedOpportunity,
    };
    let tables = &prepared.tables;
    let invalid = || {
        protocol_error(ProtocolError::InvalidBoundaryRange {
            start: selection.items.start,
            end: selection.items.end,
            participants: tables.participants.len(),
        })
    };
    if prepared.analyzed.paragraph.base_direction.is_some() {
        return Err(invalid());
    }
    let start = tables
        .boundaries
        .iter()
        .position(|row| row.preliminary_item == Some(selection.items.start))
        .ok_or_else(invalid)?;
    let end = tables
        .boundaries
        .iter()
        .rposition(|row| row.preliminary_item == Some(selection.items.end))
        .ok_or_else(invalid)?;
    if tables.boundaries[start].source_after != selection.source.start
        || tables.boundaries[end].source_before != selection.source.end
    {
        return Err(invalid());
    }
    let kind = match selection.break_kind {
        crate::engine::BreakKind::Soft => PreparedBreakKind::Allowed,
        crate::engine::BreakKind::Explicit => PreparedBreakKind::Forced,
        crate::engine::BreakKind::End => PreparedBreakKind::ParagraphEnd,
    };
    let termination = tables
        .opportunities
        .iter()
        .position(|row| row.boundary == end && row.kind == kind)
        .ok_or_else(|| protocol_error(ProtocolError::IllegalOpportunity { opportunity: end }))?;
    let from = tables.boundaries[start].participant;
    let to = tables.boundaries[end].participant;
    if from > to {
        return Err(invalid());
    }
    let handle = PreparedText {
        inner: Arc::clone(&prepared),
    };
    let participants = handle.participants().skip(from).take(to - from).collect();
    let atomics = tables
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
                    prepared: Arc::clone(&prepared),
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
        .collect();
    let request = LineSelectionRequest {
        start: PreparedBoundary {
            prepared: Arc::clone(&prepared),
            index: start,
        },
        end: PreparedBoundary {
            prepared: Arc::clone(&prepared),
            index: end,
        },
        participants,
        termination: PreparedOpportunity {
            prepared: Arc::clone(&prepared),
            index: termination,
        },
        alternative: None,
        context: LineContext {
            base_direction: if prepared.analyzed.analysis.base_level & 1 == 0 {
                ParagraphBaseDirection::LeftToRight
            } else {
                ParagraphBaseDirection::RightToLeft
            },
            orientation: LineOrientation::Horizontal,
            start: if from == 0 {
                LineStart::Paragraph
            } else {
                LineStart::Continuation
            },
            end: match kind {
                PreparedBreakKind::Allowed | PreparedBreakKind::Discretionary => LineEnd::Soft,
                PreparedBreakKind::Forced => LineEnd::Forced,
                PreparedBreakKind::ParagraphEnd => LineEnd::Paragraph,
            },
            available_inline: selection.available,
            indent: selection.indent,
            alignment: selection.alignment,
            tabs: TabContext::FontAdvance,
            hanging: HangingContext::None,
            spacing: SpacingPolicy::Spaces,
        },
        atomics,
        controls: Vec::new(),
        line_strut: None,
    };
    Ok(select(&handle, request, metric_policy)?.inner)
}
