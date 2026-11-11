//! Immutable history reads over the actual frozen arena and evidence owner.
//!
//! Qualification is metered before artifact transfer. Reads allocate nothing: inventory
//! and atom access are fixed, while traversal advances one graph header using
//! caller-owned scratch. This private owner is not a public parse-success seal.

use super::*;
use provenance::{
    AssociationView, ConstructionCause, GroupBody, HistoryData, LeafValue, NativeAssociation,
    OriginGroup,
};
use surgeist_html::{
    construction::HtmlConstructionResult,
    construction::{HtmlDocument, HtmlFragment, HtmlNode},
};

/// Only the metered history qualification paths construct this owner. Generic native freeze does not
/// establish the complete D1/lexical and history-payload qualification.
///
/// It holds what a published report reads from, apart from the arena that
/// built it. The storage is there, the evidence is there, and the source and
/// the complete D1 projection are held beside the evidence, so no reader asks
/// whether any of them is missing.
pub(crate) struct FrozenHistory {
    /// Its evidence is the field below, moved out of it.
    storage: FrozenStorage,
    evidence: PublishedEvidence,
    source: crate::HtmlSource,
    rows: OwnedVec<NativeAssociation>,
}
/// The evidence of a published report. Publication moves the source and the
/// complete projection out of it and into the report beside it, so what is
/// left cannot answer a question about either. This type exposes only what a
/// published report reads from the rest, and the evidence it wraps is
/// reachable only to the tests that corrupt a report.
pub(crate) struct PublishedEvidence(ParseEvidence);
impl PublishedEvidence {
    /// The construction cause a reference names, when this evidence issued it.
    pub(crate) fn retained_cause(
        &self,
        reference: &provenance::ConstructionCauseRef,
    ) -> Option<&ConstructionCause> {
        self.0.retained_cause(reference)
    }
    /// The committed diagnostics.
    pub(crate) fn diagnostics(&self) -> evidence::HtmlParseDiagnostics<'_> {
        self.0.diagnostics()
    }
    fn history<'a>(&'a self, source: &'a crate::HtmlSource) -> HistoryData<'a> {
        self.0.published_history(source)
    }
    fn associations<'a>(
        &'a self,
        source: &'a crate::HtmlSource,
        rows: &'a [NativeAssociation],
    ) -> AssociationView<'a> {
        self.0.published_associations(source, rows)
    }
}
/// A prepared arena that holds no custody of what a published report needs.
/// The arena stays where it was, for release.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Unpublishable;
impl Unpublishable {
    /// The refusal as the session error, with a custody fault latched as the
    /// controller's first stop.
    pub(crate) fn latch(self, c: &mut ParseControl) -> SessionError {
        latch_registry_row(c)
    }
}

impl ParserArenaSession {
    #[cfg(test)]
    pub(crate) fn freeze_history(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<FrozenHistory, SessionError> {
        let storage = self.active()?;
        storage.require(c)?;
        let evidence = storage.evidence.as_ref().ok_or(SessionError::WrongToken)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        payload_write::<HistoryData<'_>>(c, 1)?;
        let data = evidence.history_data()?;
        qualify(
            provenance::LeafStrings::with_records(
                &storage.payloads,
                ConstructionView::Active(storage.construction.storage(c)?),
            ),
            &data,
            c,
        )?;
        // Admit the actual new wrapper before the existing recoverable arena
        // transfer. There is no fallible operation after successful freeze.
        payload_write::<FrozenHistory>(c, 1)?;
        FrozenHistory::publish(&mut Some(self.freeze(c)?)).ok_or(SessionError::WrongToken)
    }
}

fn header(c: &mut ParseControl) -> Result<(), SessionError> {
    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
    c.charge(
        HtmlParseWorkCategory::InspectByte,
        mem::size_of::<OriginGroup>() as u64,
    )?;
    Ok(())
}

fn source_range(
    c: &mut ParseControl,
    source: &crate::HtmlSource,
    range: &crate::HtmlSourceRange,
) -> Result<(), SessionError> {
    c.charge(
        HtmlParseWorkCategory::InspectByte,
        mem::size_of::<crate::HtmlSourceRange>() as u64,
    )?;
    c.require_source_range(range)?;
    if range.start() > range.end() || range.end() > source.len() {
        return Err(SessionError::WrongToken);
    }
    // HtmlSourceRange's checked constructor already qualifies endpoint encoding;
    // same immutable source identity/basis above preserves that qualification.
    Ok(())
}

fn qualify(
    strings: provenance::LeafStrings<'_>,
    data: &HistoryData<'_>,
    c: &mut ParseControl,
) -> Result<(), SessionError> {
    c.require_source(data.source)?;
    for (index, cause) in data.causes.iter().enumerate() {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        c.charge(
            HtmlParseWorkCategory::InspectByte,
            mem::size_of::<ConstructionCause>() as u64,
        )?;
        if let Some(range) = cause.source() {
            source_range(c, data.source, range)?;
        }
        if cause
            .original_cause()
            .is_some_and(|original| original >= index)
            || cause.trigger().is_some_and(|trigger| trigger >= index)
            || cause
                .original()
                .is_some_and(|group| group >= data.groups.len())
            || cause
                .incoming()
                .is_some_and(|group| group >= data.groups.len())
        {
            return Err(SessionError::WrongToken);
        }
        if let Some(computation) = cause.computation() {
            provenance::validate_input_computation(c, *computation, data.groups, index)?;
        }
        // Captured qualifications were established by the sealed ingress;
        // captured facts remain nonlexical and do not acquire a source range.
    }
    for (index, group) in data.groups.iter().enumerate() {
        header(c)?;
        let Some(group_cause) = data.causes.get(group.cause) else {
            return Err(SessionError::WrongToken);
        };
        if group
            .retirement
            .is_some_and(|cause| cause >= data.causes.len())
        {
            return Err(SessionError::WrongToken);
        }
        if let Some(range) = group_cause.computation() {
            for edge in range.edges() {
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                if edge.is_some_and(|edge| edge.group >= index) {
                    return Err(SessionError::WrongToken);
                }
            }
        }
        match group.body {
            GroupBody::Leaf {
                start,
                length,
                value,
            } => {
                let end = start.checked_add(length).ok_or(SessionError::WrongToken)?;
                let Some(origins) = data.origins.get(start..end) else {
                    return Err(SessionError::WrongToken);
                };
                strings.units(c, value, group.units)?;
                for origin in origins {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    c.charge(
                        HtmlParseWorkCategory::InspectByte,
                        mem::size_of::<HtmlParseTextOrigin>() as u64,
                    )?;
                    if origin.start >= origin.end || origin.end > group.units {
                        return Err(SessionError::WrongToken);
                    }
                    source_range(c, data.source, &origin.source)?;
                }
            }
            GroupBody::Alias { original } => {
                if original >= index {
                    return Err(SessionError::WrongToken);
                }
                header(c)?;
                if data
                    .groups
                    .get(original)
                    .is_none_or(|original| original.units != group.units)
                {
                    return Err(SessionError::WrongToken);
                }
            }
            GroupBody::Concat {
                original,
                incoming,
                offset,
            } => {
                if original >= index || incoming >= index {
                    return Err(SessionError::WrongToken);
                }
                header(c)?;
                let left = data
                    .groups
                    .get(original)
                    .ok_or(SessionError::WrongToken)?
                    .units;
                header(c)?;
                let right = data
                    .groups
                    .get(incoming)
                    .ok_or(SessionError::WrongToken)?
                    .units;
                if left != offset || offset.checked_add(right) != Some(group.units) {
                    return Err(SessionError::WrongToken);
                }
            }
        }
    }
    Ok(())
}

impl FrozenHistory {
    /// Moves a qualified arena out of `slot` into its published form. An
    /// arena that cannot give it up, because it holds no storage, evidence,
    /// source or complete projection, stays in `slot` whole and unchanged.
    fn publish(slot: &mut Option<FrozenArena>) -> Option<Self> {
        let arena = slot.as_mut()?;
        let mut storage = arena.storage.take()?;
        let Some(mut evidence) = storage.evidence.take() else {
            arena.storage = Some(storage);
            return None;
        };
        let Some((source, rows)) = evidence.take_published() else {
            storage.evidence = Some(evidence);
            arena.storage = Some(storage);
            return None;
        };
        // Nothing is left in the arena that anything reads.
        *slot = None;
        Some(Self {
            storage,
            evidence: PublishedEvidence(evidence),
            source,
            rows,
        })
    }
    pub(super) fn storage(&self) -> &FrozenStorage {
        &self.storage
    }
    pub(super) fn storage_mut(&mut self) -> &mut FrozenStorage {
        &mut self.storage
    }
    pub(crate) fn evidence(&self) -> &PublishedEvidence {
        &self.evidence
    }
    /// The published evidence, for tests that corrupt a report.
    #[cfg(test)]
    pub(super) fn evidence_mut(&mut self) -> &mut ParseEvidence {
        &mut self.evidence.0
    }
    /// The published association rows, for tests that corrupt a report.
    #[cfg(test)]
    pub(super) fn rows_for_test(&mut self) -> &mut [NativeAssociation] {
        self.rows.as_mut_slice_for_test()
    }
    pub(crate) fn view(&self) -> HistoryView<'_> {
        HistoryView {
            storage: &self.storage,
            evidence: &self.evidence,
            data: self.evidence.history(&self.source),
        }
    }
    pub(crate) fn associations(&self) -> AssociationView<'_> {
        self.evidence
            .associations(&self.source, self.rows.as_slice())
    }
    pub(crate) fn fragment(&self) -> Option<&HtmlFragment> {
        match &self.storage.view {
            HtmlConstructionResult::Fragment(view) => Some(view),
            _ => None,
        }
    }
    pub(crate) fn document(&self) -> Option<&HtmlDocument> {
        match &self.storage.view {
            HtmlConstructionResult::Document(view) => Some(view),
            _ => None,
        }
    }
    /// The arena this was published from, with the evidence whole again.
    #[cfg(test)]
    pub(super) fn into_arena(self) -> FrozenArena {
        let Self {
            mut storage,
            evidence: PublishedEvidence(mut evidence),
            source,
            rows,
        } = self;
        evidence.restore_published(source, rows);
        storage.evidence = Some(evidence);
        FrozenArena {
            storage: Some(storage),
        }
    }
    /// Prepublication disposal of the actual frozen owner.
    #[cfg(test)]
    pub(crate) fn release(self, c: &mut ParseControl) -> Result<(), SessionError> {
        self.into_arena().release(c)
    }
}

/// Exclusive, owner-issued transfer qualification over the exact retained
/// frozen slot. There is no caller-supplied readiness flag or arena identifier.
pub(crate) struct PreparedHistory<'a> {
    input: &'a mut Option<FrozenArena>,
}
impl<'a> PreparedHistory<'a> {
    /// The prepared arena, for tests that corrupt what publication will take.
    #[cfg(test)]
    pub(crate) fn arena_for_test(&mut self) -> Option<&mut FrozenArena> {
        self.input.as_mut()
    }
    pub(crate) fn prepare(
        input: &'a mut Option<FrozenArena>,
        c: &mut ParseControl,
    ) -> Result<Self, SessionError> {
        let arena = input.as_ref().ok_or(SessionError::Consumed)?;
        let storage = arena.storage.as_ref().ok_or(SessionError::Consumed)?;
        c.require_identity(&storage.control)?;
        storage.effects.require_freezable(c)?;
        let evidence = storage.evidence.as_ref().ok_or(SessionError::WrongToken)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        payload_write::<HistoryData<'_>>(c, 1)?;
        let data = evidence.history_data()?;
        // The existing active freeze and this retained frozen transfer share
        // the same source/graph/leaf qualification, not two validators.
        qualify(
            provenance::LeafStrings::with_records(
                &storage.payloads,
                ConstructionView::Frozen(&storage.view),
            ),
            &data,
            c,
        )?;
        payload_write::<Self>(c, 1)?;
        payload_write::<FrozenHistory>(c, 1)?;
        payload_write::<Option<FrozenArena>>(c, 1)?;
        Ok(Self { input })
    }
    /// All useful work and receiver admissions precede the consuming caller's
    /// final checkpoint/snapshot. Commit only moves the qualified actual slot.
    ///
    /// The slot was qualified as occupied and complete and is exclusively
    /// borrowed, so a refusal is an impossible custody state. The arena is put
    /// back for release, and the caller latches the refusal on its controller.
    pub(crate) fn commit(self) -> Result<FrozenHistory, Unpublishable> {
        FrozenHistory::publish(self.input).ok_or(Unpublishable)
    }
}

/// A group reference is a borrowed, same-evidence qualification, never a payload
/// slot or a caller assertion about the identity of another report.
#[derive(Clone, Copy)]
pub struct HtmlParseOriginGroup<'a> {
    evidence: &'a PublishedEvidence,
    index: usize,
}
pub(crate) struct HistoryView<'a> {
    storage: &'a FrozenStorage,
    evidence: &'a PublishedEvidence,
    data: HistoryData<'a>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseHistoryError {
    ForeignOwner,
    InsufficientScratch {
        required: usize,
        supplied: usize,
    },
    /// The report's history holds a reference its own tables cannot resolve.
    Unresolved(read::HtmlParseUnresolvedReference),
}

impl std::fmt::Display for HtmlParseHistoryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ForeignOwner => f.write_str("the group belongs to another report"),
            Self::InsufficientScratch { required, supplied } => write!(
                f,
                "the history cursor needs {required} scratch frames and was given {supplied}"
            ),
            Self::Unresolved(_) => {
                f.write_str("the report's history holds a reference it cannot resolve")
            }
        }
    }
}
impl std::error::Error for HtmlParseHistoryError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Unresolved(fault) => Some(fault),
            _ => None,
        }
    }
}

pub(crate) struct HistoryHeader<'a> {
    pub(crate) units: usize,
    pub(crate) cause: &'a ConstructionCause,
    pub(crate) retirement: Option<&'a ConstructionCause>,
    pub(crate) body: HtmlParseHistoryBody<'a>,
}
pub enum HtmlParseHistoryBody<'a> {
    Leaf(HtmlParseHistoryLeaf<'a>),
    Alias {
        original: HtmlParseOriginGroup<'a>,
    },
    Concat {
        original: HtmlParseOriginGroup<'a>,
        incoming: HtmlParseOriginGroup<'a>,
        offset: usize,
    },
}
/// Exact retained units and original atoms, with an independently accumulated
/// logical offset. The units are borrowed, never cloned or rebased: a range of the string of the
/// record that holds the field, or the retained string of a registry row.
pub struct HtmlParseHistoryLeaf<'a> {
    units: &'a [u16],
    origins: &'a [HtmlParseTextOrigin],
    offset: usize,
}
pub struct HtmlParseHistoryOrigin<'a> {
    pub(crate) original: &'a HtmlParseTextOrigin,
    pub(crate) start: usize,
    pub(crate) end: usize,
}
impl<'a> HtmlParseHistoryLeaf<'a> {
    pub fn units(&self) -> &'a [u16] {
        self.units
    }
    pub fn unit(&self, index: usize) -> Option<u16> {
        self.units.get(index).copied()
    }
    pub fn offset(&self) -> usize {
        self.offset
    }
    pub fn origins(&self) -> &'a [HtmlParseTextOrigin] {
        self.origins
    }
    pub fn origin(&self, index: usize) -> Option<HtmlParseHistoryOrigin<'a>> {
        let original = self.origins.get(index)?;
        Some(HtmlParseHistoryOrigin {
            original,
            // The offset and the atom's end lie within the group's units.
            start: self.offset.saturating_add(original.start),
            end: self.offset.saturating_add(original.end),
        })
    }
}

impl<'a> HistoryView<'a> {
    #[cfg(test)]
    pub(crate) fn source(&self) -> &'a crate::HtmlSource {
        self.data.source
    }
    pub(crate) fn len(&self) -> usize {
        self.data.groups.len()
    }
    pub(crate) fn group(&self, index: usize) -> Option<HtmlParseOriginGroup<'a>> {
        (index < self.len()).then_some(HtmlParseOriginGroup {
            evidence: self.evidence,
            index,
        })
    }
    fn require(&self, group: HtmlParseOriginGroup<'_>) -> Result<usize, HtmlParseHistoryError> {
        if !std::ptr::eq(self.evidence, group.evidence) {
            return Err(HtmlParseHistoryError::ForeignOwner);
        }
        Ok(group.index)
    }
    /// The leaf a group's body names, or the fault if the report cannot
    /// resolve its backing or its origins.
    fn leaf(
        &self,
        (start, length, value): (usize, usize, LeafValue),
        group_units: usize,
        offset: usize,
    ) -> Result<HtmlParseHistoryLeaf<'a>, read::HtmlParseUnresolvedReference> {
        let units = match value {
            LeafValue::Empty => &[],
            LeafValue::Payload(slot) => {
                let Some(RetainedPayload::String { value, .. }) =
                    self.storage.payloads.values.get(slot)
                else {
                    return Err(unresolved(read::HtmlParseUnresolvedKind::Fact));
                };
                value.code_units()
            }
            LeafValue::Field { string, offset } => {
                let Some(units) = string
                    .field()
                    .units(ConstructionView::Frozen(&self.storage.view))
                    .and_then(|string| {
                        let end = offset.checked_add(group_units)?;
                        string.get(offset..end)
                    })
                else {
                    return Err(unresolved(read::HtmlParseUnresolvedKind::Fact));
                };
                units
            }
            // A pending leaf cannot freeze into a history.
            LeafValue::Pending => return Err(unresolved(read::HtmlParseUnresolvedKind::Fact)),
        };
        let origins = start
            .checked_add(length)
            .and_then(|end| self.data.origins.get(start..end))
            .ok_or_else(|| unresolved(read::HtmlParseUnresolvedKind::Group))?;
        Ok(HtmlParseHistoryLeaf {
            units,
            origins,
            offset,
        })
    }
    pub(crate) fn header(
        &self,
        group: HtmlParseOriginGroup<'_>,
    ) -> Result<HistoryHeader<'a>, HtmlParseHistoryError> {
        let index = self.require(group)?;
        let group = self
            .data
            .groups
            .get(index)
            .ok_or_else(|| unresolved_history(read::HtmlParseUnresolvedKind::Group))?;
        let reference = |index| HtmlParseOriginGroup {
            evidence: self.evidence,
            index,
        };
        let body = match group.body {
            GroupBody::Leaf {
                start,
                length,
                value,
            } => HtmlParseHistoryBody::Leaf(
                self.leaf((start, length, value), group.units, 0)
                    .map_err(HtmlParseHistoryError::Unresolved)?,
            ),
            GroupBody::Alias { original } => HtmlParseHistoryBody::Alias {
                original: reference(original),
            },
            GroupBody::Concat {
                original,
                incoming,
                offset,
            } => HtmlParseHistoryBody::Concat {
                original: reference(original),
                incoming: reference(incoming),
                offset,
            },
        };
        let cause = |index: usize| {
            self.data
                .causes
                .get(index)
                .ok_or_else(|| unresolved_history(read::HtmlParseUnresolvedKind::Cause))
        };
        Ok(HistoryHeader {
            units: group.units,
            cause: cause(group.cause)?,
            retirement: group.retirement.map(cause).transpose()?,
            body,
        })
    }
    pub(crate) fn cursor<'s>(
        &'s self,
        group: HtmlParseOriginGroup<'_>,
        scratch: &'s mut [HtmlParseHistoryFrame],
        cancellation: Option<&'s crate::HtmlParseCancellation>,
    ) -> Result<HtmlParseHistoryCursor<'a, 's>, HtmlParseHistoryError> {
        let index = self.require(group)?;
        // Strict backward headers bound the pending DFS stack by index+1 even
        // for a shared DAG. Setup neither scans the graph nor clears scratch.
        let required = index.saturating_add(1);
        if scratch.len() < required {
            return Err(HtmlParseHistoryError::InsufficientScratch {
                required,
                supplied: scratch.len(),
            });
        }
        // `required` is at least one, so the scratch has a first frame.
        let Some(first) = scratch.first_mut() else {
            return Err(HtmlParseHistoryError::InsufficientScratch {
                required,
                supplied: 0,
            });
        };
        *first = HtmlParseHistoryFrame { index, offset: 0 };
        Ok(HtmlParseHistoryCursor {
            view: self,
            scratch,
            length: 1,
            cancellation,
            cancelled: false,
            fault: None,
        })
    }
}

/// Private fields prevent caller-written traversal authority. Scratch may hold
/// stale frames: only the cursor's initialized prefix is ever read.
#[derive(Clone, Copy, Default)]
pub struct HtmlParseHistoryFrame {
    index: usize,
    offset: usize,
}
pub struct HtmlParseHistoryCursor<'a, 's> {
    view: &'s HistoryView<'a>,
    scratch: &'s mut [HtmlParseHistoryFrame],
    length: usize,
    cancellation: Option<&'s crate::HtmlParseCancellation>,
    cancelled: bool,
    fault: Option<read::HtmlParseUnresolvedReference>,
}
fn unresolved(kind: read::HtmlParseUnresolvedKind) -> read::HtmlParseUnresolvedReference {
    read::HtmlParseUnresolvedReference::new(kind)
}
fn unresolved_history(kind: read::HtmlParseUnresolvedKind) -> HtmlParseHistoryError {
    HtmlParseHistoryError::Unresolved(unresolved(kind))
}
pub enum HtmlParseHistoryStep<'a> {
    Progress,
    Leaf(HtmlParseHistoryLeaf<'a>),
    Complete,
    Cancelled,
    /// The history holds a reference its own tables cannot resolve. The cursor
    /// stays faulted, so a caller that continues is told again.
    Fault(read::HtmlParseUnresolvedReference),
}
impl<'a> HtmlParseHistoryCursor<'a, '_> {
    /// The fault, latched: a cursor that met one reports it from then on.
    fn fail(&mut self, kind: read::HtmlParseUnresolvedKind) -> HtmlParseHistoryStep<'a> {
        let fault = *self.fault.get_or_insert_with(|| unresolved(kind));
        HtmlParseHistoryStep::Fault(fault)
    }
    pub fn advance(&mut self) -> HtmlParseHistoryStep<'a> {
        if let Some(fault) = self.fault {
            return HtmlParseHistoryStep::Fault(fault);
        }
        if self.cancelled
            || self
                .cancellation
                .is_some_and(|signal| signal.is_cancelled())
        {
            self.cancelled = true;
            return HtmlParseHistoryStep::Cancelled;
        }
        let Some(top) = self.length.checked_sub(1) else {
            return HtmlParseHistoryStep::Complete;
        };
        self.length = top;
        let Some(frame) = self.scratch.get(top).copied() else {
            return self.fail(read::HtmlParseUnresolvedKind::Group);
        };
        let Some(group) = self.view.data.groups.get(frame.index) else {
            return self.fail(read::HtmlParseUnresolvedKind::Group);
        };
        match group.body {
            GroupBody::Leaf {
                start,
                length,
                value,
            } => match self
                .view
                .leaf((start, length, value), group.units, frame.offset)
            {
                Ok(leaf) => HtmlParseHistoryStep::Leaf(leaf),
                Err(fault) => self.fail(fault.kind()),
            },
            GroupBody::Alias { original } => {
                let Some(slot) = self.scratch.get_mut(top) else {
                    return self.fail(read::HtmlParseUnresolvedKind::Group);
                };
                *slot = HtmlParseHistoryFrame {
                    index: original,
                    offset: frame.offset,
                };
                // The stack is bounded by the scratch the cursor was given.
                self.length = top.saturating_add(1);
                HtmlParseHistoryStep::Progress
            }
            GroupBody::Concat {
                original,
                incoming,
                offset,
            } => {
                // The offset and the left part lie within the group's units.
                let right = HtmlParseHistoryFrame {
                    index: incoming,
                    offset: frame.offset.saturating_add(offset),
                };
                let left = HtmlParseHistoryFrame {
                    index: original,
                    offset: frame.offset,
                };
                // Strict backward headers bound the stack by the scratch the
                // cursor was given, so a graph that outgrows it is not one.
                let Some([first, second]) = self.scratch.get_mut(top..=top.saturating_add(1))
                else {
                    return self.fail(read::HtmlParseUnresolvedKind::Group);
                };
                *first = right;
                *second = left;
                self.length = top.saturating_add(2);
                HtmlParseHistoryStep::Progress
            }
        }
    }
}

impl HtmlParseHistoryOrigin<'_> {
    pub fn original(&self) -> &HtmlParseTextOrigin {
        self.original
    }
    pub fn start(&self) -> usize {
        self.start
    }
    pub fn end(&self) -> usize {
        self.end
    }
}
impl FrozenHistory {
    /// The node a stored index names, or the fault.
    pub(super) fn native_node(
        &self,
        index: usize,
    ) -> Result<HtmlNode<'_>, read::HtmlParseUnresolvedReference> {
        self.storage
            .view
            .node_at(index)
            .map(|node| node.as_node())
            .ok_or(read::HtmlParseUnresolvedReference::new(
                read::HtmlParseUnresolvedKind::Node,
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenizer::{states::HtmlTokenizerState, tests::LexerFixture, token::Event};
    use crate::{HtmlParseCancellation, HtmlParseLimits};
    use surgeist_html::construction::HtmlDocumentMode;

    fn limits() -> HtmlParseLimits {
        super::super::tests::limits()
    }

    #[test]
    fn an_unpublishable_history_latches_the_registry_row_as_the_first_stop() {
        let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
        let stop = HtmlParseStop::InternalInvariant(
            crate::parser::control::HtmlParseInvariant::RegistryRow,
        );
        assert_eq!(c.stop(), None);
        assert_eq!(Unpublishable.latch(&mut c), SessionError::Stop(stop));
        assert_eq!(c.stop(), Some(stop));
    }
    fn zero(c: &ParseControl) {
        super::super::tests::storage_zero(c);
        assert_eq!(
            c.usage().resource(HtmlParseResource::Diagnostics).current,
            0
        );
    }
    fn literal(
        source: &crate::HtmlSource,
        cancel: HtmlParseCancellation,
    ) -> (ParseControl, ParserArenaSession, usize) {
        let mut c = ParseControl::new(limits(), cancel);
        c.admit_source(source).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        session.initialize_parse_evidence(&mut c, source).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let mut node = None;
        loop {
            let event = lexer
                .next_in(
                    &mut c,
                    false,
                    session.active_mut().unwrap().evidence.as_mut().unwrap(),
                )
                .unwrap()
                .unwrap();
            let Event::Token(event) = event else {
                event.release(&mut c).unwrap();
                continue;
            };
            let token = event.into_token(&mut c).unwrap();
            session
                .active_mut()
                .unwrap()
                .evidence
                .as_mut()
                .unwrap()
                .lexical_disposition(&mut c, token.lexical.as_ref(), false)
                .unwrap();
            match token.kind {
                TokenKind::Text(text) => {
                    node = Some(
                        session
                            .insert_text(&mut c, &root, None, &mut Some(text), None)
                            .unwrap()
                            .unwrap()
                            ._node,
                    );
                }
                TokenKind::Eof => break,
                _ => panic!("independent literal contains text only"),
            }
        }
        lexer.release(&mut c).unwrap();
        let storage = session.active().unwrap();
        let group = storage
            .payloads
            .detail_origins
            .peek(FieldRef::Text(node.unwrap().index))
            .expect("canonical native text group");
        // Observe the actual group through the existing controlled lender;
        // inventory assertions below independently describe the literal graph.
        let evidence = storage.evidence.as_ref().unwrap();
        assert!(evidence.require_group(group).unwrap().units > 0);
        // Each actual text merge appends its current group after its incoming
        // leaf. No group is produced by the EOF token.
        let group_index = evidence.history_counts().1 - 1;
        (c, session, group_index)
    }
    fn projected(source: &crate::HtmlSource) -> (ParseControl, ParserArenaSession, usize) {
        let (mut c, mut session, group) = literal(source, HtmlParseCancellation::new());
        session.prepare_native_associations(&mut c, source).unwrap();
        (c, session, group)
    }

    #[test]
    fn immutable_history_borrows_retired_utf16_leaf_units_and_exact_rebased_origins() {
        use crate::parser::tokenizer::token::HtmlParseOriginKind::{
            CharacterReference, Literal, Newline,
        };
        let source = crate::HtmlSource::utf8("A🙂\r\n&NotEqualTilde;");
        let (mut c, mut session, group) = projected(&source);
        // The leaves are ranges of the text's own units, which no copy holds.
        let text_range = {
            let storage = session.active().unwrap();
            let ConstructionNodeKind::Text(text) = storage
                .construction
                .test_storage()
                .node_at(1)
                .unwrap()
                .kind()
            else {
                panic!("native text")
            };
            let start = text.code_units().as_ptr() as usize;
            start..start + 2 * text.len()
        };
        let owner = session.freeze_history(&mut c).unwrap();
        let before = c.usage().clone();
        {
            let view = owner.view();
            assert_eq!(view.source().identity(), source.identity());
            assert!(owner.document().is_none());
            let text = owner.fragment().unwrap().root().children().next().unwrap();
            let surgeist_html::construction::HtmlNodeKind::Text(text) =
                owner.fragment().unwrap().node(&text).unwrap().kind()
            else {
                panic!("native text")
            };
            assert_eq!(text.code_units(), [65, 0xd83d, 0xde42, 10, 0x2242, 0x0338]);
            let mut scratch = vec![HtmlParseHistoryFrame::default(); view.len()];
            let mut cursor = view
                .cursor(view.group(group).unwrap(), &mut scratch, None)
                .unwrap();
            let mut output = Vec::new();
            let mut origins = Vec::new();
            let mut actions = 0;
            loop {
                match cursor.advance() {
                    HtmlParseHistoryStep::Fault(fault) => panic!("{fault}"),
                    HtmlParseHistoryStep::Progress => actions += 1,
                    HtmlParseHistoryStep::Leaf(leaf) => {
                        actions += 1;
                        assert!(
                            text_range.contains(&(leaf.units().as_ptr() as usize)),
                            "the text's own units, not copied output"
                        );
                        assert_eq!(leaf.unit(leaf.units().len()), None);
                        output.extend_from_slice(leaf.units());
                        for index in 0..leaf.origins().len() {
                            let origin = leaf.origin(index).unwrap();
                            assert!(std::ptr::eq(origin.original, &leaf.origins()[index]));
                            origins.push((
                                origin.start,
                                origin.end,
                                origin.original.source.start(),
                                origin.original.source.end(),
                                origin.original.kind,
                            ));
                        }
                        assert!(leaf.origin(leaf.origins().len()).is_none());
                    }
                    HtmlParseHistoryStep::Complete => break,
                    HtmlParseHistoryStep::Cancelled => panic!("no query cancellation"),
                }
            }
            assert_eq!(output, [65, 0xd83d, 0xde42, 10, 0x2242, 0x0338]);
            assert_eq!(
                origins,
                [
                    (0, 1, 0, 1, Literal),
                    (1, 3, 1, 5, Literal),
                    (3, 4, 5, 7, Newline),
                    (4, 5, 7, 22, CharacterReference),
                    (5, 6, 7, 22, CharacterReference)
                ]
            );
            // Four Data leaves (the two-unit entity is one text token), and
            // three actual Concats. One action inspects one graph header.
            assert_eq!(actions, 7);
        }
        assert_eq!(
            c.usage(),
            before,
            "completed reads do no parser work/allocation"
        );
        owner.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_requires_real_d1_and_lexical_eof_before_owner_transfer() {
        let source = crate::HtmlSource::utf8("AB");
        let (mut c, mut session, _) = literal(&source, HtmlParseCancellation::new());
        assert!(matches!(
            session.freeze_history(&mut c),
            Err(SessionError::WrongToken)
        ));
        assert!(session.active().is_ok());
        assert!(c.stop().is_none());
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        let owner = session.freeze_history(&mut c).unwrap();
        assert!(matches!(session.active(), Err(SessionError::Consumed)));
        assert!(owner.associations().len() > 0);
        owner.release(&mut c).unwrap();
        zero(&c);
        // A native-only source-free arena cannot certify history either.
        let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        assert!(matches!(
            session.freeze_history(&mut c),
            Err(SessionError::WrongToken)
        ));
        session.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn a_prepared_history_without_its_projection_refuses_to_publish_and_keeps_the_arena() {
        let source = crate::HtmlSource::utf8("AB");
        let (mut c, mut session, _) = projected(&source);
        let mut slot = Some(session.freeze(&mut c).unwrap());
        let mut prepared = PreparedHistory::prepare(&mut slot, &mut c).unwrap();
        prepared
            .arena_for_test()
            .unwrap()
            .storage
            .as_mut()
            .unwrap()
            .evidence
            .as_mut()
            .unwrap()
            .drop_projection_for_test(&mut c);
        assert!(
            prepared.commit().is_err(),
            "a history that lacks its projection is not published"
        );
        let mut arena = slot.take().expect("the arena stays where it was");
        arena.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn a_prepared_history_without_arena_storage_is_unpublishable_and_keeps_the_arena() {
        let source = crate::HtmlSource::utf8("AB");
        let (mut c, mut session, _) = projected(&source);
        let mut slot = Some(session.freeze(&mut c).unwrap());
        let mut prepared = PreparedHistory::prepare(&mut slot, &mut c).unwrap();
        let storage = prepared.arena_for_test().unwrap().storage.take().unwrap();
        assert_eq!(prepared.commit().err(), Some(Unpublishable));
        let arena = slot.as_mut().expect("the arena stays where it was");
        assert!(arena.storage.is_none());
        arena.storage = Some(storage);
        slot.take().unwrap().release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn a_prepared_history_without_evidence_is_unpublishable_and_puts_the_storage_back() {
        let source = crate::HtmlSource::utf8("AB");
        let (mut c, mut session, _) = projected(&source);
        let mut slot = Some(session.freeze(&mut c).unwrap());
        let mut prepared = PreparedHistory::prepare(&mut slot, &mut c).unwrap();
        let evidence = prepared
            .arena_for_test()
            .unwrap()
            .storage
            .as_mut()
            .unwrap()
            .evidence
            .take()
            .unwrap();
        assert_eq!(prepared.commit().err(), Some(Unpublishable));
        let arena = slot.as_mut().expect("the arena stays where it was");
        let storage = arena.storage.as_mut().expect("the storage was put back");
        assert!(storage.evidence.is_none());
        storage.evidence = Some(evidence);
        slot.take().unwrap().release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn a_prepared_history_without_its_source_is_unpublishable_and_puts_everything_back() {
        let source = crate::HtmlSource::utf8("AB");
        let (mut c, mut session, _) = projected(&source);
        let mut slot = Some(session.freeze(&mut c).unwrap());
        let mut prepared = PreparedHistory::prepare(&mut slot, &mut c).unwrap();
        let held = prepared
            .arena_for_test()
            .unwrap()
            .storage
            .as_mut()
            .unwrap()
            .evidence
            .as_mut()
            .unwrap()
            .take_source_for_test()
            .unwrap();
        assert_eq!(prepared.commit().err(), Some(Unpublishable));
        let arena = slot.as_mut().expect("the arena stays where it was");
        let evidence = arena
            .storage
            .as_mut()
            .expect("the storage was put back")
            .evidence
            .as_mut()
            .expect("the evidence was put back");
        // The refusal took nothing else out, so the projection is still there.
        assert!(evidence.holds_projection_for_test());
        assert!(evidence.take_source_for_test().is_none());
        evidence.restore_source_for_test(held);
        slot.take().unwrap().release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_foreign_scope_and_small_scratch_leave_caller_storage_untouched() {
        let source = crate::HtmlSource::utf8("AB");
        let (mut c, mut session, group) = projected(&source);
        let (mut foreign_c, mut foreign_session, foreign_group) = projected(&source);
        let owner = session.freeze_history(&mut c).unwrap();
        let foreign = foreign_session.freeze_history(&mut foreign_c).unwrap();
        {
            let view = owner.view();
            let other = foreign.view();
            assert_eq!(
                view.header(other.group(foreign_group).unwrap()).err(),
                Some(HtmlParseHistoryError::ForeignOwner)
            );
            let mut scratch = [HtmlParseHistoryFrame {
                index: usize::MAX,
                offset: 99,
            }; 2];
            assert_eq!(
                view.cursor(view.group(group).unwrap(), &mut scratch, None)
                    .err(),
                Some(HtmlParseHistoryError::InsufficientScratch {
                    required: 3,
                    supplied: 2
                })
            );
            assert!(
                scratch
                    .iter()
                    .all(|frame| frame.index == usize::MAX && frame.offset == 99)
            );
            let mut scratch = [HtmlParseHistoryFrame {
                index: usize::MAX,
                offset: 99,
            }; 5];
            {
                let cursor = view
                    .cursor(view.group(group).unwrap(), &mut scratch, None)
                    .unwrap();
                assert_eq!(cursor.length, 1);
            }
            assert_eq!(scratch[0].index, group);
            assert!(
                scratch[1..]
                    .iter()
                    .all(|frame| frame.index == usize::MAX && frame.offset == 99)
            );
        }
        owner.release(&mut c).unwrap();
        foreign.release(&mut foreign_c).unwrap();
        zero(&c);
        zero(&foreign_c);
    }

    #[test]
    fn immutable_history_old_parse_cancel_does_not_revoke_reads_query_cancel_is_local() {
        let source = crate::HtmlSource::utf8("AB");
        let old_cancel = HtmlParseCancellation::new();
        let (mut c, mut session, group) = literal(&source, old_cancel.clone());
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        let owner = session.freeze_history(&mut c).unwrap();
        let before = c.usage().clone();
        old_cancel.cancel();
        {
            let view = owner.view();
            let mut scratch = [HtmlParseHistoryFrame::default(); 3];
            let query_cancel = HtmlParseCancellation::new();
            let mut cursor = view
                .cursor(
                    view.group(group).unwrap(),
                    &mut scratch,
                    Some(&query_cancel),
                )
                .unwrap();
            assert!(matches!(cursor.advance(), HtmlParseHistoryStep::Progress));
            query_cancel.cancel();
            assert!(matches!(cursor.advance(), HtmlParseHistoryStep::Cancelled));
            assert!(matches!(cursor.advance(), HtmlParseHistoryStep::Cancelled));
            let mut cursor = view
                .cursor(view.group(group).unwrap(), &mut scratch, None)
                .unwrap();
            assert!(matches!(cursor.advance(), HtmlParseHistoryStep::Progress));
            assert!(
                matches!(cursor.advance(), HtmlParseHistoryStep::Leaf(leaf) if leaf.units()==[65] && leaf.offset()==0)
            );
            assert!(
                matches!(cursor.advance(), HtmlParseHistoryStep::Leaf(leaf) if leaf.units()==[66] && leaf.offset()==1)
            );
            assert!(matches!(cursor.advance(), HtmlParseHistoryStep::Complete));
        }
        assert_eq!(c.usage(), before);
        owner.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_alias_empty_and_shared_dag_advance_one_header_per_action() {
        let source = crate::HtmlSource::utf8("AB");
        let (mut c, mut session, original_index) = literal(&source, HtmlParseCancellation::new());
        let storage = session.active_mut().unwrap();
        let evidence = storage.evidence.as_mut().unwrap();
        // The records are the fragment root and the text.
        let slot = storage
            .payloads
            .detail_origins
            .peek(FieldRef::Text(1))
            .unwrap();
        let mut reference = evidence.copy_group_reference(&mut c, slot).unwrap();
        // Actual canonical DAG: six doublings share each prior header as both
        // children. Four Copy headers then alias the resulting 128 units.
        let mut units = 2;
        for _ in 0..6 {
            reference = evidence
                .merge_groups(&mut c, &reference, &reference, units, units * 2)
                .unwrap();
            units *= 2;
        }
        for _ in 0..4 {
            reference = evidence.copy_group(&mut c, &reference).unwrap();
        }
        let final_index = evidence.history_counts().1 - 1;
        let empty_index = evidence.history_counts().1;
        evidence.source_independent_group(&mut c, 0).unwrap();
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        let owner = session.freeze_history(&mut c).unwrap();
        let before = c.usage().clone();
        {
            let view = owner.view();
            let header = view.header(view.group(final_index).unwrap()).unwrap();
            assert_eq!(header.units, 128);
            assert_eq!(
                header.cause.operation(),
                provenance::HtmlParseConstructionOperation::Copy
            );
            assert!(header.retirement.is_none());
            let HtmlParseHistoryBody::Alias { original } = header.body else {
                panic!("copy alias")
            };
            assert_eq!(view.header(original).unwrap().units, 128);
            let header = view.header(view.group(original_index).unwrap()).unwrap();
            let HtmlParseHistoryBody::Concat {
                original,
                incoming,
                offset,
            } = header.body
            else {
                panic!("real AB merge")
            };
            assert_eq!(offset, 1);
            for (group, expected) in [(original, 65), (incoming, 66)] {
                let header = view.header(group).unwrap();
                let HtmlParseHistoryBody::Leaf(leaf) = header.body else {
                    panic!("literal leaf")
                };
                assert_eq!(leaf.units(), [expected]);
                // A previously occupied the native Text slot and is retired;
                // incoming B was registered Detached and never occupied it.
                assert_eq!(header.retirement.is_some(), expected == 65);
            }
            let header = view.header(view.group(empty_index).unwrap()).unwrap();
            assert_eq!(
                header.cause.operation(),
                provenance::HtmlParseConstructionOperation::SourceIndependent
            );
            let HtmlParseHistoryBody::Leaf(empty) = header.body else {
                panic!("zero-unit defining leaf")
            };
            assert!(empty.units().is_empty() && empty.origins().is_empty());
            let mut scratch = vec![HtmlParseHistoryFrame::default(); view.len()];
            let mut cursor = view
                .cursor(view.group(final_index).unwrap(), &mut scratch, None)
                .unwrap();
            let mut progress = 0;
            let mut leaves = 0;
            loop {
                match cursor.advance() {
                    HtmlParseHistoryStep::Fault(fault) => panic!("{fault}"),
                    HtmlParseHistoryStep::Progress => progress += 1,
                    HtmlParseHistoryStep::Leaf(leaf) => {
                        assert_eq!(leaf.offset(), leaves);
                        assert_eq!(leaf.units(), [if leaves % 2 == 0 { 65 } else { 66 }]);
                        let origin = leaf.origin(0).unwrap();
                        assert_eq!((origin.start, origin.end), (leaves, leaves + 1));
                        assert_eq!(origin.original.source.start(), leaves % 2);
                        leaves += 1;
                    }
                    HtmlParseHistoryStep::Complete => break,
                    HtmlParseHistoryStep::Cancelled => panic!("no query cancellation"),
                }
            }
            // 128 leaves have 127 binary headers, plus exactly four aliases.
            assert_eq!((leaves, progress), (128, 131));
        }
        assert_eq!(c.usage(), before);
        owner.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_qualification_exact_work_and_one_short_keep_actual_cleanup_owner() {
        let source = crate::HtmlSource::utf8("AB");
        for short in [false, true] {
            let (mut c, mut session, _) = projected(&source);
            let storage = session.active().unwrap();
            let data = storage.evidence.as_ref().unwrap().history_data().unwrap();
            assert_eq!((data.groups.len(), data.origins.len()), (3, 2));
            let causes = data.causes.len() as u64;
            let ranged_causes = data
                .causes
                .iter()
                .filter(|cause| cause.source().is_some())
                .count() as u64;
            // AB: three headers + two Concat child-header reads, two leaf string
            // visits (a leaf in a record is one read), two origin reads.
            // Each source range has its own admitted header inspection and the
            // canonical range check's RecordVisit+ProviderCall.
            let ranges = ranged_causes + 2;
            let visits = causes + 5 + 2 + 2 + ranges;
            let inspect = causes * mem::size_of::<ConstructionCause>() as u64
                + 5 * mem::size_of::<OriginGroup>() as u64
                + 2 * mem::size_of::<HtmlParseTextOrigin>() as u64
                + ranges * mem::size_of::<crate::HtmlSourceRange>() as u64;
            let exact = visits + inspect + ranges;
            let used = c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                HtmlParseWorkCategory::StateStep,
                limits().work_credits() - used - exact + u64::from(short),
            )
            .unwrap();
            let before = c.usage().clone();
            let outcome = qualify(storage.leaf_strings(), &data, &mut c);
            if short {
                let first = c.stop().unwrap();
                assert!(matches!(
                    first,
                    HtmlParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    }
                ));
                assert_eq!(outcome, Err(SessionError::Stop(first)));
                assert!(session.active().is_ok());
                assert!(
                    matches!(session.freeze_history(&mut c), Err(SessionError::Stop(stop)) if stop == first)
                );
                let at_stop = c.first_stop_usage().cloned();
                session.release(&mut c).unwrap();
                assert_eq!(c.stop(), Some(first));
                assert_eq!(c.first_stop_usage(), at_stop.as_ref());
            } else {
                outcome.unwrap();
                assert_eq!(
                    c.usage().work(HtmlParseWorkCategory::RecordVisit)
                        - before.work(HtmlParseWorkCategory::RecordVisit),
                    visits
                );
                assert_eq!(
                    c.usage().work(HtmlParseWorkCategory::InspectByte)
                        - before.work(HtmlParseWorkCategory::InspectByte),
                    inspect
                );
                assert_eq!(
                    c.usage().work(HtmlParseWorkCategory::ProviderCall)
                        - before.work(HtmlParseWorkCategory::ProviderCall),
                    ranges
                );
                assert_eq!(
                    c.usage().work(HtmlParseWorkCategory::CopyByte),
                    before.work(HtmlParseWorkCategory::CopyByte)
                );
                assert_eq!(
                    c.usage().work(HtmlParseWorkCategory::RecordWrite),
                    before.work(HtmlParseWorkCategory::RecordWrite)
                );
                // Cleanup may newly latch exhaustion; the borrowed active owner
                // is retained for the ordinary terminal disposal retry.
                if session.release(&mut c).is_err() {
                    session.release(&mut c).unwrap();
                }
            }
            zero(&c);
        }
    }

    #[test]
    fn immutable_history_reset_keeps_retired_raw_graph_without_duplicate_backing() {
        let source = crate::HtmlSource::utf8("<textarea>AB");
        let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let Event::Token(event) = lexer
            .next_in(
                &mut c,
                false,
                session.active_mut().unwrap().evidence.as_mut().unwrap(),
            )
            .unwrap()
            .unwrap()
        else {
            panic!("textarea")
        };
        let token = event.into_token(&mut c).unwrap();
        session
            .active_mut()
            .unwrap()
            .evidence
            .as_mut()
            .unwrap()
            .lexical_disposition(&mut c, token.lexical.as_ref(), false)
            .unwrap();
        let TokenKind::Tag(tag) = token.kind else {
            panic!("textarea")
        };
        let node = session
            .allocate_html_tag_for_creation(
                &mut c,
                &mut Some(tag),
                &root,
                None,
                CreationContext::Ordinary,
            )
            .unwrap();
        session.insert(&mut c, &root, &node, None, None).unwrap();
        loop {
            let Event::Token(event) = lexer
                .next_in(
                    &mut c,
                    false,
                    session.active_mut().unwrap().evidence.as_mut().unwrap(),
                )
                .unwrap()
                .unwrap()
            else {
                continue;
            };
            let token = event.into_token(&mut c).unwrap();
            session
                .active_mut()
                .unwrap()
                .evidence
                .as_mut()
                .unwrap()
                .lexical_disposition(&mut c, token.lexical.as_ref(), false)
                .unwrap();
            match token.kind {
                TokenKind::Text(text) => {
                    session
                        .insert_text(&mut c, &node, None, &mut Some(text), None)
                        .unwrap();
                }
                TokenKind::Eof => break,
                _ => panic!("literal AB then EOF"),
            }
        }
        lexer.release(&mut c).unwrap();
        let raw = session.textarea_state(&mut c, &node).unwrap().raw;
        assert_eq!(
            session
                .textarea_raw_value(&mut c, &node)
                .unwrap()
                .code_units(),
            [65, 66]
        );
        let old_group = {
            let storage = session.active_mut().unwrap();
            let RetainedPayload::String {
                origins: Some(group),
                ..
            } = &storage.payloads.values[raw]
            else {
                panic!("canonical raw group")
            };
            let evidence = storage.evidence.as_mut().unwrap();
            // Preserve this actual current raw group through the canonical
            // Copy/Alias producer; no pre-projection immutable lender is used.
            let index = evidence.history_counts().1;
            let copy = evidence.copy_group(&mut c, group).unwrap();
            drop(copy);
            index
        };
        for _ in 0..3 {
            session.parser_stack_removed(&mut c, &node, None).unwrap();
        }
        assert!(matches!(
            session.active().unwrap().payloads.values[raw],
            RetainedPayload::HistoricalString { .. }
        ));
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        let owner = session.freeze_history(&mut c).unwrap();
        {
            let view = owner.view();
            let group = view.group(old_group).unwrap();
            assert_eq!(view.header(group).unwrap().units, 2);
            let mut scratch = vec![HtmlParseHistoryFrame::default(); old_group + 1];
            let mut cursor = view.cursor(group, &mut scratch, None).unwrap();
            let mut units = Vec::new();
            loop {
                match cursor.advance() {
                    HtmlParseHistoryStep::Fault(fault) => panic!("{fault}"),
                    HtmlParseHistoryStep::Leaf(leaf) => {
                        assert_eq!(leaf.offset(), units.len());
                        for origin in leaf.origins() {
                            assert!(origin.source.start() >= 10 && origin.source.end() <= 12);
                        }
                        units.extend_from_slice(leaf.units());
                    }
                    HtmlParseHistoryStep::Progress => {}
                    HtmlParseHistoryStep::Complete => break,
                    HtmlParseHistoryStep::Cancelled => panic!("no query cancellation"),
                }
            }
            assert_eq!(units, [65, 66]);
        }
        owner.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_formatting_replay_lends_distinct_backings_and_original_entity_atoms() {
        use crate::parser::tokenizer::token::HtmlParseOriginKind::{CharacterReference, Literal};
        let source = crate::HtmlSource::utf8("<b a='X&amp;Y'>");
        let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let root = session.root(&mut c).unwrap();
        let html = session.allocate_implicit_html(&mut c).unwrap();
        session.insert(&mut c, &root, &html, None, None).unwrap();
        let body = session.allocate_implicit_body(&mut c, None).unwrap();
        session.insert(&mut c, &html, &body, None, None).unwrap();
        let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let Event::Token(event) = lexer
            .next_in(
                &mut c,
                false,
                session.active_mut().unwrap().evidence.as_mut().unwrap(),
            )
            .unwrap()
            .unwrap()
        else {
            panic!("start tag")
        };
        let token = event.into_token(&mut c).unwrap();
        session
            .active_mut()
            .unwrap()
            .evidence
            .as_mut()
            .unwrap()
            .lexical_disposition(&mut c, token.lexical.as_ref(), false)
            .unwrap();
        let TokenKind::Tag(tag) = token.kind else {
            panic!("start tag")
        };
        let mut creation = session
            .retain_formatting_creation(&mut c, &mut Some(tag), token.source)
            .unwrap();
        let first = session
            .create_formatting_from_creation(&mut c, &creation, &body)
            .unwrap();
        let second = session
            .create_formatting_from_creation(&mut c, &creation, &body)
            .unwrap();
        assert!(!session.same_node(&mut c, &first, &second).unwrap());
        session.insert(&mut c, &body, &first, None, None).unwrap();
        session.insert(&mut c, &body, &second, None, None).unwrap();
        creation.release(&mut c).unwrap();
        let event = lexer
            .next_in(
                &mut c,
                false,
                session.active_mut().unwrap().evidence.as_mut().unwrap(),
            )
            .unwrap()
            .unwrap();
        assert!(
            matches!(&event, Event::Token(token) if matches!(&token.token_for_tests().kind, TokenKind::Eof))
        );
        event.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        let owner = session.freeze_history(&mut c).unwrap();
        {
            let view = owner.view();
            let mut pointers = Vec::new();
            for index in 0..view.len() {
                let header = view.header(view.group(index).unwrap()).unwrap();
                let HtmlParseHistoryBody::Leaf(leaf) = header.body else {
                    continue;
                };
                if leaf.units() != [88, 38, 89] {
                    continue;
                }
                let atoms = leaf
                    .origins()
                    .iter()
                    .map(|atom| {
                        (
                            atom.start,
                            atom.end,
                            atom.source.start(),
                            atom.source.end(),
                            atom.kind,
                        )
                    })
                    .collect::<Vec<_>>();
                assert_eq!(
                    atoms,
                    [
                        (0, 1, 6, 7, Literal),
                        (1, 2, 7, 12, CharacterReference),
                        (2, 3, 12, 13, Literal)
                    ]
                );
                pointers.push(leaf.units().as_ptr());
            }
            assert_eq!(pointers.len(), 2, "two actual replay field leaves");
            assert_ne!(
                pointers[0], pointers[1],
                "actual independently owned replay backings"
            );
        }
        owner.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_native_only_projection_without_lexical_eof_cannot_transfer() {
        let source = crate::HtmlSource::utf8("A");
        let mut c = ParseControl::new(limits(), HtmlParseCancellation::new());
        c.admit_source(&source).unwrap();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        session.initialize_parse_evidence(&mut c, &source).unwrap();
        let root = session.root(&mut c).unwrap();
        let mut lexer = LexerFixture::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
        let Event::Token(event) = lexer
            .next_in(
                &mut c,
                false,
                session.active_mut().unwrap().evidence.as_mut().unwrap(),
            )
            .unwrap()
            .unwrap()
        else {
            panic!("literal A")
        };
        let token = event.into_token(&mut c).unwrap();
        session
            .active_mut()
            .unwrap()
            .evidence
            .as_mut()
            .unwrap()
            .lexical_disposition(&mut c, token.lexical.as_ref(), false)
            .unwrap();
        let TokenKind::Text(text) = token.kind else {
            panic!("literal Text")
        };
        session
            .insert_text(&mut c, &root, None, &mut Some(text), None)
            .unwrap();
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        assert!(session.native_associations(&mut c).unwrap().len() > 0);
        assert!(matches!(
            session.freeze_history(&mut c),
            Err(SessionError::WrongToken)
        ));
        assert!(session.active().is_ok() && c.stop().is_none());
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_source_independent_nonempty_units_are_not_lexical_claims() {
        let source = crate::HtmlSource::utf8("AB");
        let (mut c, mut session, _) = literal(&source, HtmlParseCancellation::new());
        // Use the actual source-independent producer. An implied element after
        // lexical input may instead retain the current governing token cause;
        // lack of field origins alone does not make that cause source-less.
        for literal in [b"p".as_slice(), b"http://www.w3.org/1999/xhtml".as_slice()] {
            let value = literal_string(&mut c, literal).unwrap();
            let storage = session.active_mut().unwrap();
            let group = storage
                .evidence
                .as_mut()
                .unwrap()
                .source_independent_group(&mut c, literal.len())
                .unwrap();
            let (value, _) = storage
                .register_group_string_owned(&mut c, value, Some(group))
                .unwrap();
            drop(value);
        }
        session
            .prepare_native_associations(&mut c, &source)
            .unwrap();
        let owner = session.freeze_history(&mut c).unwrap();
        {
            let view = owner.view();
            let mut saw_name = false;
            let mut saw_namespace = false;
            for index in 0..view.len() {
                let header = view.header(view.group(index).unwrap()).unwrap();
                let HtmlParseHistoryBody::Leaf(leaf) = header.body else {
                    continue;
                };
                if header.cause.operation()
                    == provenance::HtmlParseConstructionOperation::SourceIndependent
                {
                    assert!(header.cause.source().is_none() && header.cause.captured().is_none());
                    assert!(leaf.origins().is_empty());
                    if leaf.units() == [112] {
                        saw_name = true;
                    }
                    if leaf.units()
                        == "http://www.w3.org/1999/xhtml"
                            .encode_utf16()
                            .collect::<Vec<_>>()
                    {
                        saw_namespace = true;
                    }
                }
            }
            assert!(
                saw_name && saw_namespace,
                "actual independently retained source-less literals"
            );
        }
        owner.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn immutable_history_captured_namespace_keeps_context_qualification_without_lexical_origins() {
        use crate::*;
        use surgeist_html::{
            construction::HtmlDocumentMode,
            values::{Namespace, OrderedAttributes, QualifiedName},
        };
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(10, 10, 10, 1000, 10, 10, 10, 16_777_216),
            HtmlHostFactsRevision::fresh(),
        );
        let document = builder
            .add_document(
                HtmlDocumentMode::NoQuirks,
                HtmlScriptingStatus::Disabled,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        let context = builder
            .add_element(
                &document,
                QualifiedName::new(Namespace::from_uri("urn:retained"), None, "context"),
                OrderedAttributes::default(),
                HtmlCapturedIsValue::Absent,
                HtmlCaptureRegistryInput::Null,
            )
            .unwrap();
        builder.establish_root(&context).unwrap();
        let capture = builder.finish().unwrap();
        let context =
            HtmlFragmentContext::element(&capture, &capture.subject_id(&context).unwrap()).unwrap();
        let request =
            HtmlFragmentParseRequest::new(HtmlSource::utf8(""), context, limits()).unwrap();
        let mut c = ParseControl::new(*request.limits(), request.cancellation().clone());
        c.admit_source(request.source()).unwrap();
        let mut session =
            ParserArenaSession::new(&mut c, RootKind::Document(HtmlDocumentMode::NoQuirks))
                .unwrap();
        session.initialize_standalone_document(&mut c).unwrap();
        session
            .initialize_parse_evidence(&mut c, request.source())
            .unwrap();
        session.initialize_fragment(&mut c, &request).unwrap();
        let parser_document = session.root(&mut c).unwrap();
        session
            .validate_fragment_proof(&mut c, &parser_document)
            .unwrap();
        // Exercise the existing actual source-owner copy path only after real
        // request/bootstrap/profile qualification; no captured trigger is minted
        // by this reader or test and no synthetic lexical range is supplied.
        let before = session
            .active()
            .unwrap()
            .evidence
            .as_ref()
            .unwrap()
            .history_counts()
            .1;
        let (namespace, slot) = session
            .active_mut()
            .unwrap()
            .copy_context_namespace(&mut c)
            .unwrap();
        assert_eq!(
            namespace.uri().unwrap().code_units(),
            "urn:retained".encode_utf16().collect::<Vec<_>>()
        );
        let slot = slot.unwrap();
        let pointer = namespace.uri().unwrap().code_units().as_ptr();
        drop(namespace);
        let mut lexer =
            LexerFixture::new(request.source(), HtmlTokenizerState::Data, None, &mut c).unwrap();
        let event = lexer
            .next_in(
                &mut c,
                false,
                session.active_mut().unwrap().evidence.as_mut().unwrap(),
            )
            .unwrap()
            .unwrap();
        assert!(
            matches!(&event, Event::Token(token) if matches!(token.token_for_tests().kind, TokenKind::Eof))
        );
        event.release(&mut c).unwrap();
        lexer.release(&mut c).unwrap();
        session
            .prepare_native_associations(&mut c, request.source())
            .unwrap();
        let owner = session.freeze_history(&mut c).unwrap();
        {
            let view = owner.view();
            let header = view.header(view.group(before).unwrap()).unwrap();
            assert!(header.cause.source().is_none());
            let captured = header.cause.captured().unwrap();
            assert_eq!(&captured.capture, request.context().capture().identity());
            assert_eq!(
                &captured.host_revision,
                request.context().capture().host_facts_revision()
            );
            assert_eq!(&captured.context, request.context().context_element());
            assert_eq!(
                &captured.target_document,
                request.context().target_document()
            );
            let HtmlParseHistoryBody::Leaf(leaf) = header.body else {
                panic!("captured namespace leaf")
            };
            assert_eq!(leaf.units().as_ptr(), pointer);
            assert_eq!(
                leaf.units(),
                "urn:retained".encode_utf16().collect::<Vec<_>>()
            );
            assert!(leaf.origins().is_empty());
            assert!(matches!(
                &view.storage.payloads.values[slot],
                RetainedPayload::String {
                    location: FieldRef::Detached,
                    ..
                }
            ));
        }
        owner.release(&mut c).unwrap();
        zero(&c);
    }

    #[test]
    fn committing_a_prepared_history_over_an_empty_slot_is_unpublishable() {
        // The slot is exclusively borrowed and was qualified as occupied; an
        // empty one is an impossible custody state, never a panic. The caller
        // latches it on its controller.
        let mut slot: Option<FrozenArena> = None;
        let prepared = PreparedHistory { input: &mut slot };
        assert_eq!(prepared.commit().err(), Some(Unpublishable));
    }
}
