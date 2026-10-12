//! HTML 877d Chapter 13 error invocations, retained at actual emission.
//! Flat committed diagnostics remain safe after native tree reclamation.

use super::*;
use crate::parser::{
    control::Invariant,
    tokenizer::{
        states::State,
        token::{DiagnosticCode, DiagnosticPhase},
    },
    tree_builder::{
        head::{HeadCondition, InsertionMode},
        prologue::{PrologueCondition, PrologueMode},
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiagnosticKind {
    Tokenizer(DiagnosticCode),
    Prologue(PrologueCondition),
    Tree(HeadCondition),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiagnosticContext {
    Tokenizer {
        phase: DiagnosticPhase,
        state: State,
    },
    Prologue(PrologueMode),
    Tree(InsertionMode),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiagnosticSeverity {
    Recoverable,
}
/// Selected HTML anchor plus intentional engine branch within that algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct DiagnosticRule {
    pub(crate) anchor: &'static str,
    pub(crate) branch: &'static str,
}
pub(crate) struct DiagnosticRecord {
    pub(crate) kind: DiagnosticKind,
    pub(crate) context: DiagnosticContext,
    pub(crate) severity: DiagnosticSeverity,
    pub(crate) rule: DiagnosticRule,
    location: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiagnosticToken {
    Text,
    Tag,
    Comment,
    Doctype,
    ProcessingInstruction,
    Eof,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DiagnosticLocationRole {
    InputFrontier,
    TokenEnvelope,
    TextOrigin,
    BufferedTableCharacter,
}
pub(crate) struct DiagnosticLocation {
    pub(crate) source: crate::HtmlSourceRange,
    pub(crate) role: DiagnosticLocationRole,
    pub(crate) token: Option<DiagnosticToken>,
}
impl std::ops::Deref for DiagnosticLocation {
    type Target = crate::HtmlSourceRange;
    fn deref(&self) -> &Self::Target {
        &self.source
    }
}
pub(crate) struct DiagnosticLocationInput<'a> {
    pub(crate) source: &'a crate::HtmlSourceRange,
    pub(crate) role: DiagnosticLocationRole,
    pub(crate) token: Option<DiagnosticToken>,
}
pub(super) struct EvidenceIdentity;
pub(crate) struct DiagnosticId {
    index: usize,
    owner: ControlIdentity,
    evidence: Arc<EvidenceIdentity>,
}
pub(crate) struct TokenizerDiagnosticView<'a> {
    pub(crate) code: DiagnosticCode,
    pub(crate) phase: DiagnosticPhase,
    pub(crate) state: State,
    pub(crate) source: &'a crate::HtmlSourceRange,
}
pub(crate) struct ParseEvidence {
    pub(super) identity: Arc<EvidenceIdentity>,
    identity_allocation: SharedAllocation<EvidenceIdentity>,
    records: OwnedVec<DiagnosticRecord>,
    locations: OwnedVec<DiagnosticLocation>,
    pub(super) native: super::provenance::NativeProvenance,
    diagnostics_taken: bool,
}
/// The terminal consumer receives only committed diagnostics and their actual
/// issuer. No native association, construction cause or source-map draft can
/// survive this partition as a usable parse result.
pub(crate) struct FailureDiagnosticEvidence {
    identity: Arc<EvidenceIdentity>,
    identity_allocation: Option<SharedAllocation<EvidenceIdentity>>,
    records: OwnedVec<DiagnosticRecord>,
    locations: OwnedVec<DiagnosticLocation>,
}
/// Fixed borrowed lender of committed diagnostics. It carries no execution authority,
/// allocation ticket or native storage, and each observation uses constant work.
pub(crate) struct DiagnosticEvidenceView<'a> {
    records: &'a [DiagnosticRecord],
    locations: &'a [DiagnosticLocation],
}
impl DiagnosticEvidenceView<'_> {
    pub(crate) fn len(&self) -> usize {
        self.records.len()
    }
    pub(crate) fn retained_record(
        &self,
        index: usize,
    ) -> Option<(&DiagnosticRecord, &DiagnosticLocation)> {
        self.records
            .get(index)
            .map(|record| (record, &self.locations[record.location]))
    }
    #[cfg(test)]
    pub(crate) fn records_pointer(&self) -> *const DiagnosticRecord {
        self.records.as_ptr()
    }
}
impl FailureDiagnosticEvidence {
    pub(crate) fn diagnostics(&self) -> DiagnosticEvidenceView<'_> {
        DiagnosticEvidenceView {
            records: self.records.as_slice(),
            locations: self.locations.as_slice(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.records.len()
    }
    pub(crate) fn retained_record(
        &self,
        index: usize,
    ) -> Option<(&DiagnosticRecord, &DiagnosticLocation)> {
        self.records
            .as_slice()
            .get(index)
            .map(|record| (record, &self.locations.as_slice()[record.location]))
    }
    #[cfg(test)]
    pub(crate) fn records_pointer(&self) -> *const DiagnosticRecord {
        self.records.as_slice().as_ptr()
    }
    pub(crate) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.records.release(c)?;
        self.locations.release(c)?;
        if let Some(allocation) = self.identity_allocation {
            c.release_zero_sized_shared(self.identity, allocation)
                .map_err(|error| error.into_parts().0)
        } else {
            drop(self.identity);
            Ok(())
        }
    }
}
impl ParseEvidence {
    pub(super) fn retains_diagnostics(&self) -> bool {
        !self.diagnostics_taken
    }
    pub(crate) fn diagnostics(&self) -> DiagnosticEvidenceView<'_> {
        DiagnosticEvidenceView {
            records: self.records.as_slice(),
            locations: self.locations.as_slice(),
        }
    }

    /// The same initializer is used by the actual session and direct lexical
    /// fixtures. Preparing the storage slot is fallible before owner transfer.
    pub(crate) fn prepare(c: &mut ParseControl) -> Result<Self, SessionError> {
        let evidence = Self::new(c)?;
        if let Err(stop) = payload_write::<Option<Self>>(c, 1) {
            evidence.release(c)?;
            return Err(stop);
        }
        Ok(evidence)
    }
    pub(crate) fn new(c: &mut ParseControl) -> Result<Self, ParseStop> {
        let mut records = OwnedVec::new(HtmlParseResource::Diagnostics)?;
        let mut locations = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        records.require_owner(c)?;
        locations.require_owner(c)?;
        let (identity, identity_allocation) =
            c.share_owned(EvidenceIdentity).map_err(|(stop, _)| stop)?;
        let native = match super::provenance::NativeProvenance::new(c) {
            Ok(native) => native,
            Err(stop) => {
                records.release(c)?;
                locations.release(c)?;
                c.release_zero_sized_shared(identity, identity_allocation)
                    .map_err(|error| error.into_parts().0)?;
                return Err(stop);
            }
        };
        Ok(Self {
            identity,
            identity_allocation,
            records,
            locations,
            native,
            diagnostics_taken: false,
        })
    }
    pub(crate) fn append(
        &mut self,
        c: &mut ParseControl,
        kind: DiagnosticKind,
        context: DiagnosticContext,
        rule: DiagnosticRule,
        input: DiagnosticLocationInput<'_>,
    ) -> Result<DiagnosticId, ParseStop> {
        let source = input.source;
        c.require_source_range(source)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let index = self.records.len();
        let location = self.locations.len();
        // Handle and descriptor preparation precedes the atomic pair commit.
        c.charge(
            WorkCategory::CopyByte,
            (mem::size_of::<ControlIdentity>() + mem::size_of::<Arc<EvidenceIdentity>>()) as u64,
        )?;
        let owner = c.identity();
        let evidence = self.identity.clone();
        c.charge(
            WorkCategory::CopyByte,
            mem::size_of::<crate::HtmlSourceRange>() as u64,
        )?;
        let source = DiagnosticLocation {
            source: source.clone(),
            role: input.role,
            token: input.token,
        };
        self.records.try_push_pair(
            &mut self.locations,
            c,
            DiagnosticRecord {
                kind,
                context,
                severity: DiagnosticSeverity::Recoverable,
                rule,
                location,
            },
            source,
        )?;
        Ok(DiagnosticId {
            index,
            owner,
            evidence,
        })
    }
    pub(crate) fn get<'a>(
        &'a self,
        c: &mut ParseControl,
        id: &DiagnosticId,
    ) -> Result<(&'a DiagnosticRecord, &'a DiagnosticLocation), ParseStop> {
        c.require_identity(&id.owner)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        c.charge(
            WorkCategory::InspectByte,
            mem::size_of::<Arc<EvidenceIdentity>>() as u64,
        )?;
        if !Arc::ptr_eq(&self.identity, &id.evidence) {
            return Err(c.fail_invariant(Invariant::ForeignEvidence));
        }
        self.records.require_disposal_owner(c)?;
        self.locations.require_disposal_owner(c)?;
        c.charge(WorkCategory::RecordVisit, 2)?;
        let record = self
            .records
            .as_slice()
            .get(id.index)
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        let location = self
            .locations
            .as_slice()
            .get(record.location)
            .ok_or_else(|| c.fail_invariant(Invariant::InputCursorState))?;
        Ok((record, location))
    }
    pub(crate) fn len(&self) -> usize {
        self.records.len()
    }
    /// Fixed borrowed observations of already committed, self-contained data.
    /// Used by the stopped private consumer; this does not resume parse work.
    pub(crate) fn retained_record(
        &self,
        index: usize,
    ) -> Option<(&DiagnosticRecord, &DiagnosticLocation)> {
        self.records
            .as_slice()
            .get(index)
            .map(|record| (record, &self.locations.as_slice()[record.location]))
    }
    #[cfg(test)]
    pub(crate) fn tokenizer_observation(&self, id: &DiagnosticId) -> TokenizerDiagnosticView<'_> {
        assert!(
            Arc::ptr_eq(&self.identity, &id.evidence),
            "actual issuer-qualified ID"
        );
        let (r, source) = self.retained_record(id.index).expect("real emitted ID");
        let DiagnosticKind::Tokenizer(code) = r.kind else {
            panic!("lexer ID")
        };
        let DiagnosticContext::Tokenizer { phase, state } = r.context else {
            panic!("lexer context")
        };
        TokenizerDiagnosticView {
            code,
            phase,
            state,
            source: &source.source,
        }
    }
    #[cfg(test)]
    pub(crate) fn records_pointer(&self) -> *const DiagnosticRecord {
        self.records.as_slice().as_ptr()
    }
    /// The actual active or frozen storage slot remains intact on every refusal.
    /// The caller must already be stopped. Fixed owner/header validation precedes
    /// the only transfer; cleanup does not allocate, copy, charge or poll.
    pub(crate) fn partition_failed(
        input: &mut Option<Self>,
        c: &mut ParseControl,
    ) -> Result<Option<FailureDiagnosticEvidence>, SessionError> {
        if c.stop().is_none() {
            return Err(SessionError::WrongToken);
        }
        let Some(evidence) = input.as_ref() else {
            return Ok(None);
        };
        evidence.records.require_disposal_owner(c)?;
        evidence.locations.require_disposal_owner(c)?;
        evidence.native.preflight_failed_disposal(c)?;
        let Some(Self {
            identity,
            identity_allocation,
            records,
            locations,
            native,
            diagnostics_taken: _,
        }) = input.take()
        else {
            unreachable!("the exclusively borrowed slot was preflighted")
        };
        // Same-owner flat vectors, exact checked capacities and their aggregate
        // logical/capacity totals were proved above. No ledger mutation intervenes,
        // so each cleanup subtraction is covered by that aggregate proof.
        native.release(c).expect("preflighted native cleanup");
        Ok(Some(FailureDiagnosticEvidence {
            identity,
            identity_allocation: Some(identity_allocation),
            records,
            locations,
        }))
    }
    pub(crate) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        self.native.release(c)?;
        self.records.release(c)?;
        self.locations.release(c)?;
        c.release_zero_sized_shared(self.identity, self.identity_allocation)
            .map_err(|error| error.into_parts().0)
    }
}
impl ParserArenaSession {
    pub(crate) fn retained_evidence(&self) -> Result<&ParseEvidence, SessionError> {
        let evidence = self
            .active()?
            .evidence
            .as_ref()
            .ok_or(SessionError::WrongToken)?;
        if evidence.diagnostics_taken {
            return Err(SessionError::Consumed);
        }
        Ok(evidence)
    }
    #[cfg(test)]
    pub(crate) fn observed_evidence(&self) -> &ParseEvidence {
        self.active()
            .expect("actual active session")
            .evidence
            .as_ref()
            .expect("source evidence")
    }
    pub(crate) fn initialize_parse_evidence(
        &mut self,
        c: &mut ParseControl,
        source: &crate::HtmlSource,
    ) -> Result<(), SessionError> {
        let storage = self.active_mut()?;
        storage.require(c)?;
        if storage.evidence.is_some() {
            return Err(SessionError::WrongToken);
        }
        storage.evidence = Some(ParseEvidence::prepare(c)?);
        storage
            .evidence
            .as_mut()
            .expect("retained evidence")
            .initialize_native_source(c, source)?;
        let slots = storage.prepare_node_causes(c, 1)?;
        if let Some(slot) = slots[0] {
            let RetainedPayload::ConstructionCause { node, .. } =
                &mut storage.payloads.values[slot]
            else {
                unreachable!()
            };
            *node = Some(0);
        }
        Ok(())
    }
    pub(crate) fn evidence(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<&mut ParseEvidence, SessionError> {
        let storage = self.active_mut()?;
        storage.require(c)?;
        storage.evidence.as_mut().ok_or(SessionError::WrongToken)
    }
    pub(crate) fn take_completed_evidence(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<FailureDiagnosticEvidence, SessionError> {
        let storage = self.active_mut()?;
        storage.require(c)?;
        let evidence = storage.evidence.as_mut().ok_or(SessionError::WrongToken)?;
        evidence.records.require_owner(c)?;
        evidence.locations.require_owner(c)?;
        if evidence.diagnostics_taken {
            return Err(SessionError::Consumed);
        }
        let mut records = OwnedVec::new(HtmlParseResource::Diagnostics)?;
        let mut locations = OwnedVec::new(HtmlParseResource::SourceMapEntries)?;
        records.require_owner(c)?;
        locations.require_owner(c)?;
        // The native group authority remains in the completed lender. The
        // diagnostic consumer shares the existing zero-sized identity without
        // allocating a second issuer or returning another mutable authority.
        c.charge(WorkCategory::ProviderCall, 1)?;
        payload_write::<FailureDiagnosticEvidence>(c, 1)?;
        payload_write::<bool>(c, 1)?;
        let identity = evidence.identity.clone();
        let records = mem::replace(&mut evidence.records, records);
        let locations = mem::replace(&mut evidence.locations, locations);
        evidence.diagnostics_taken = true;
        Ok(FailureDiagnosticEvidence {
            identity,
            identity_allocation: None,
            records,
            locations,
        })
    }
    /// Before reclamation, move only actual committed diagnostic ownership.
    pub(crate) fn take_failed_evidence(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<Option<FailureDiagnosticEvidence>, SessionError> {
        ParseEvidence::partition_failed(&mut self.active_mut()?.evidence, c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tree_builder::tests::{assert_released, limits};
    use crate::{HtmlParseCancellation, HtmlSource};

    fn append(
        e: &mut ParseEvidence,
        c: &mut ParseControl,
        source: &HtmlSource,
        offset: usize,
    ) -> Result<DiagnosticId, ParseStop> {
        e.append(
            c,
            DiagnosticKind::Tokenizer(DiagnosticCode::UnexpectedNullCharacter),
            DiagnosticContext::Tokenizer {
                phase: DiagnosticPhase::Tokenizer,
                state: State::Data,
            },
            DiagnosticRule {
                anchor: "parse-error-unexpected-null-character",
                branch: "unexpected-null-character",
            },
            DiagnosticLocationInput {
                source: &source.range(offset, offset + 1).unwrap(),
                role: DiagnosticLocationRole::InputFrontier,
                token: None,
            },
        )
    }
    fn owner(
        source: &HtmlSource,
        selected: crate::HtmlParseLimits,
    ) -> (ParseControl, ParseEvidence) {
        let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
        c.admit_source(source).unwrap();
        let evidence = ParseEvidence::prepare(&mut c).unwrap();
        (c, evidence)
    }

    #[test]
    fn failed_partition_refusals_preserve_the_actual_owner_and_committed_buffer() {
        let source = HtmlSource::utf8("\0");
        let (mut c, mut evidence) = owner(&source, limits(100));
        evidence.initialize_native_source(&mut c, &source).unwrap();
        append(&mut evidence, &mut c, &source, 0).unwrap();
        let pointer = evidence.records_pointer();
        let mut input = Some(evidence);
        let before = c.usage().clone();
        assert!(matches!(
            ParseEvidence::partition_failed(&mut input, &mut c),
            Err(SessionError::WrongToken)
        ));
        assert_eq!(input.as_ref().unwrap().records_pointer(), pointer);
        assert_eq!(c.usage(), &before);

        let mut foreign = ParseControl::new(limits(100), HtmlParseCancellation::new());
        foreign.fail_invariant(Invariant::InputCursorState);
        let foreign_stop = foreign.stop();
        let foreign_before = foreign.usage().clone();
        assert!(ParseEvidence::partition_failed(&mut input, &mut foreign).is_err());
        assert_eq!(input.as_ref().unwrap().records_pointer(), pointer);
        assert_eq!(foreign.stop(), foreign_stop);
        assert_eq!(foreign.usage(), &foreign_before);
        assert_eq!(c.usage(), &before);

        c.fail_invariant(Invariant::InputCursorState);
        let first = c.stop();
        let stopped = c.usage().clone();
        let failure = ParseEvidence::partition_failed(&mut input, &mut c)
            .unwrap()
            .unwrap();
        assert!(input.is_none());
        assert_eq!(failure.records_pointer(), pointer);
        assert_eq!(failure.len(), 1);
        let (record, location) = failure.retained_record(0).unwrap();
        assert_eq!(
            record.kind,
            DiagnosticKind::Tokenizer(DiagnosticCode::UnexpectedNullCharacter)
        );
        assert_eq!(location.source, source.range(0, 1).unwrap());
        assert_eq!(c.stop(), first);
        assert_eq!(
            c.usage().resource(HtmlParseResource::WorkCredits),
            stopped.resource(HtmlParseResource::WorkCredits)
        );
        assert_eq!(
            c.usage()
                .resource(HtmlParseResource::SourceMapEntries)
                .current,
            1
        );
        assert!(
            c.usage()
                .resource(HtmlParseResource::ManagedCapacityBytes)
                .current
                < stopped
                    .resource(HtmlParseResource::ManagedCapacityBytes)
                    .current
        );
        failure.release(&mut c).unwrap();
        assert_eq!(c.stop(), first);
        assert_released(&c);
        assert_released(&foreign);
    }

    #[test]
    fn paired_diagnostic_and_location_caps_never_publish_half_a_record() {
        let source = HtmlSource::utf8("\0\0");
        for resource in [
            HtmlParseResource::Diagnostics,
            HtmlParseResource::SourceMapEntries,
        ] {
            for cap in [0, 1, 2] {
                let b = limits(100).into_builder();
                let selected = match resource {
                    HtmlParseResource::Diagnostics => b.diagnostics(cap),
                    _ => b.source_map_entries(cap),
                }
                .finish()
                .unwrap();
                let (mut c, mut e) = owner(&source, selected);
                for offset in 0..2 {
                    let result = append(&mut e, &mut c, &source, offset);
                    if offset as u64 >= cap {
                        assert!(
                            matches!(result,Err(ParseStop::LimitExceeded {resource:r,..}) if r==resource)
                        );
                        assert_eq!(e.len(), offset);
                        assert_eq!(e.locations.len(), offset);
                        assert_eq!(
                            c.usage().resource(HtmlParseResource::Diagnostics).current,
                            offset as u64
                        );
                        assert_eq!(
                            c.usage()
                                .resource(HtmlParseResource::SourceMapEntries)
                                .current,
                            offset as u64
                        );
                        break;
                    }
                    let id = result.unwrap();
                    let (_, range) = e.get(&mut c, &id).unwrap();
                    assert_eq!((range.start(), range.end()), (offset, offset + 1));
                }
                let polls = c.owned_span_observation().polls;
                let stopped = c.stop().is_some();
                e.release(&mut c).unwrap();
                assert_released(&c);
                if stopped {
                    assert_eq!(c.owned_span_observation().polls, polls);
                }
            }
        }
    }

    #[test]
    fn actual_evidence_initializer_exact_and_one_short_atomic_layout_reclaim_before_pull() {
        let source = HtmlSource::utf8("\0");
        let bytes = mem::size_of::<Option<ParseEvidence>>() as u64;
        assert!(bytes > 0);
        for short in [false, true] {
            let selected = limits(100)
                .into_builder()
                .atomic_layout_bytes(bytes - u64::from(short))
                .finish()
                .unwrap();
            let mut c = ParseControl::new(selected, HtmlParseCancellation::new());
            c.admit_source(&source).unwrap();
            match ParseEvidence::prepare(&mut c) {
                Ok(evidence) => {
                    assert!(!short);
                    assert_eq!((evidence.len(), evidence.locations.len()), (0, 0));
                    evidence.release(&mut c).unwrap();
                }
                Err(SessionError::Stop(stop)) => {
                    assert!(short);
                    assert_eq!(
                        stop,
                        ParseStop::LimitExceeded {
                            resource: HtmlParseResource::AtomicLayoutBytes,
                            current: 0,
                            requested_delta: bytes,
                            limit: bytes - 1,
                        }
                    );
                    assert_eq!(c.stop(), Some(stop));
                }
                Err(other) => panic!("unexpected initializer refusal {other:?}"),
            }
            assert_released(&c);
        }
    }

    #[test]
    fn capacity_and_transient_one_short_cuts_retain_only_prepared_storage() {
        let source = HtmlSource::utf8("\0");
        let (mut probe, mut e) = owner(&source, limits(100));
        append(&mut e, &mut probe, &source, 0).unwrap();
        let capacity = probe
            .usage()
            .resource(HtmlParseResource::ManagedCapacityBytes)
            .high_water;
        let peak = probe
            .usage()
            .resource(HtmlParseResource::PeakTransientBytes)
            .high_water;
        e.release(&mut probe).unwrap();
        for (resource, actual) in [
            (HtmlParseResource::ManagedCapacityBytes, capacity),
            (HtmlParseResource::PeakTransientBytes, peak),
        ] {
            assert!(actual > 0);
            for short in [false, true] {
                let b = limits(100).into_builder();
                let cap = actual - u64::from(short);
                let selected = match resource {
                    HtmlParseResource::ManagedCapacityBytes => b.managed_capacity_bytes(cap),
                    _ => b.peak_transient_bytes(cap),
                }
                .finish()
                .unwrap();
                let (mut c, mut e) = owner(&source, selected);
                let result = append(&mut e, &mut c, &source, 0);
                if short {
                    assert!(
                        matches!(result,Err(ParseStop::LimitExceeded {resource:r,..}) if r==resource)
                    );
                    assert_eq!((e.len(), e.locations.len()), (0, 0));
                    assert_eq!(
                        c.usage().resource(HtmlParseResource::Diagnostics).current,
                        0
                    );
                    assert_eq!(
                        c.usage()
                            .resource(HtmlParseResource::SourceMapEntries)
                            .current,
                        0
                    );
                } else {
                    assert!(result.is_ok());
                    assert_eq!((e.len(), e.locations.len()), (1, 1));
                }
                let work = c.usage().resource(HtmlParseResource::WorkCredits);
                let stopped = c.stop().is_some();
                e.release(&mut c).unwrap();
                assert_released(&c);
                if stopped {
                    assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
                }
            }
        }
    }

    #[test]
    fn actual_second_append_exact_and_one_short_work_preserve_committed_prefix() {
        let source = HtmlSource::utf8("\0\0");
        let selected = limits(100);
        let (mut probe, mut e) = owner(&source, selected);
        append(&mut e, &mut probe, &source, 0).unwrap();
        let before = probe
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        append(&mut e, &mut probe, &source, 1).unwrap();
        let cost = probe
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - before;
        e.release(&mut probe).unwrap();
        for short in [false, true] {
            let (mut c, mut e) = owner(&source, selected);
            append(&mut e, &mut c, &source, 0).unwrap();
            let available = selected.work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(WorkCategory::StateStep, available - cost + u64::from(short))
                .unwrap();
            let result = append(&mut e, &mut c, &source, 1);
            if short {
                assert!(matches!(
                    result,
                    Err(ParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    })
                ));
                assert_eq!((e.len(), e.locations.len()), (1, 1));
                let first = c.stop();
                assert!(append(&mut e, &mut c, &source, 1).is_err());
                assert_eq!(c.stop(), first);
            } else {
                assert!(result.is_ok());
                assert_eq!((e.len(), e.locations.len()), (2, 2));
            }
            let (record, range) = e.retained_record(0).unwrap();
            assert_eq!(
                record.kind,
                DiagnosticKind::Tokenizer(DiagnosticCode::UnexpectedNullCharacter)
            );
            assert_eq!((range.start(), range.end()), (0, 1));
            e.release(&mut c).unwrap();
            assert_released(&c);
        }
    }

    #[test]
    fn cancelled_append_and_foreign_source_leave_the_real_prefix_unchanged() {
        let source = HtmlSource::utf8("\0\0");
        for cancel in [false, true] {
            let cancellation = HtmlParseCancellation::new();
            let mut c = ParseControl::new(limits(100), cancellation.clone());
            c.admit_source(&source).unwrap();
            let mut e = ParseEvidence::prepare(&mut c).unwrap();
            append(&mut e, &mut c, &source, 0).unwrap();
            let other = HtmlSource::utf8("\0\0");
            if cancel {
                cancellation.cancel();
            }
            let result = append(&mut e, &mut c, if cancel { &source } else { &other }, 1);
            assert_eq!(
                result.err(),
                Some(if cancel {
                    ParseStop::Cancelled
                } else {
                    ParseStop::InternalInvariant(Invariant::ForeignSource)
                })
            );
            assert_eq!((e.len(), e.locations.len()), (1, 1));
            e.release(&mut c).unwrap();
            assert_released(&c);
        }
    }

    #[test]
    fn actual_active_to_frozen_move_and_alias_refusal_preserve_the_same_store() {
        let request =
            crate::HtmlDocumentParseRequest::offline(HtmlSource::utf8("<p/>A"), limits(100));
        let mut driver = crate::parser::tree_builder::entry::initialized_driver(&request);
        for _ in 0..1000 {
            if driver.step().unwrap() == crate::parser::tree_builder::head::HeadStep::Complete {
                break;
            }
        }
        let mut input = Some(driver);
        let (mut c, mut session, _) =
            crate::parser::tree_builder::StandaloneDocumentDriver::take_standalone_fixture(
                &mut input,
            )
            .unwrap();
        let pointer = session.retained_evidence().unwrap().records_pointer();
        let mut frozen = session.freeze(&mut c).unwrap();
        assert_eq!(
            frozen
                .storage
                .as_ref()
                .unwrap()
                .evidence
                .as_ref()
                .unwrap()
                .records_pointer(),
            pointer
        );
        let alias = frozen.document().unwrap().clone();
        assert!(frozen.release(&mut c).is_err());
        let first = c.stop();
        assert_eq!(
            frozen
                .storage
                .as_ref()
                .unwrap()
                .evidence
                .as_ref()
                .unwrap()
                .records_pointer(),
            pointer
        );
        drop(alias);
        let work = c.usage().resource(HtmlParseResource::WorkCredits);
        let failure =
            ParseEvidence::partition_failed(&mut frozen.storage.as_mut().unwrap().evidence, &mut c)
                .unwrap()
                .unwrap();
        assert_eq!(failure.records_pointer(), pointer);
        assert!(frozen.storage.as_ref().unwrap().evidence.is_none());
        frozen.release(&mut c).unwrap();
        assert_eq!(c.stop(), first);
        assert_eq!(c.usage().resource(HtmlParseResource::WorkCredits), work);
        failure.release(&mut c).unwrap();
        assert_eq!(c.stop(), first);
        assert_released(&c);
    }

    #[test]
    fn same_controller_different_store_cannot_resolve_a_notification_id() {
        let source = HtmlSource::utf8("\0");
        let (mut c, mut first) = owner(&source, limits(100));
        let mut other = ParseEvidence::prepare(&mut c).unwrap();
        let id = append(&mut first, &mut c, &source, 0).unwrap();
        append(&mut other, &mut c, &source, 0).unwrap();
        assert!(matches!(
            other.get(&mut c, &id),
            Err(ParseStop::InternalInvariant(Invariant::ForeignEvidence))
        ));
        assert_eq!(first.len(), 1);
        assert_eq!(other.len(), 1);
        first.release(&mut c).unwrap();
        other.release(&mut c).unwrap();
        assert_released(&c);
    }

    #[test]
    fn foreign_controller_cannot_read_or_extend_an_actual_store() {
        let source = HtmlSource::utf8("\0\0");
        let (mut c, mut e) = owner(&source, limits(100));
        let id = append(&mut e, &mut c, &source, 0).unwrap();
        let before = c.usage().clone();
        let mut foreign = ParseControl::new(limits(100), HtmlParseCancellation::new());
        foreign.admit_source(&source).unwrap();
        assert!(matches!(
            e.get(&mut foreign, &id),
            Err(ParseStop::InternalInvariant(Invariant::ForeignControl))
        ));
        assert!(append(&mut e, &mut foreign, &source, 1).is_err());
        assert_eq!((e.len(), e.locations.len()), (1, 1));
        assert_eq!(c.usage(), &before);
        e.release(&mut c).unwrap();
        assert_released(&c);
    }
}
