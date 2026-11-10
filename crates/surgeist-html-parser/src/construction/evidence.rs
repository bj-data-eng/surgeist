//! HTML 877d Chapter 13 error invocations, retained at actual emission.
//! Flat committed diagnostics remain safe after native tree reclamation.

use super::*;
use crate::parser::{
    control::HtmlParseInvariant,
    tokenizer::{
        states::HtmlTokenizerState,
        token::{HtmlTokenizerDiagnostic, HtmlTokenizerDiagnosticPhase},
    },
    tree_builder::{
        head::{HtmlInsertionMode, HtmlTreeDiagnostic},
        prologue::{HtmlPrologueDiagnostic, HtmlPrologueMode},
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseDiagnosticKind {
    Tokenizer(HtmlTokenizerDiagnostic),
    Prologue(HtmlPrologueDiagnostic),
    Tree(HtmlTreeDiagnostic),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseDiagnosticContext {
    Tokenizer {
        phase: HtmlTokenizerDiagnosticPhase,
        state: HtmlTokenizerState,
    },
    Prologue(HtmlPrologueMode),
    Tree(HtmlInsertionMode),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseDiagnosticSeverity {
    Recoverable,
}
/// Selected HTML anchor plus intentional engine branch within that algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlParseDiagnosticRule {
    pub(crate) anchor: &'static str,
    pub(crate) branch: &'static str,
}
pub struct HtmlParseDiagnostic {
    pub(crate) kind: HtmlParseDiagnosticKind,
    pub(crate) context: HtmlParseDiagnosticContext,
    pub(crate) severity: HtmlParseDiagnosticSeverity,
    pub(crate) rule: HtmlParseDiagnosticRule,
    location: usize,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseTokenFamily {
    Text,
    Tag,
    Comment,
    Doctype,
    ProcessingInstruction,
    Eof,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParseDiagnosticLocationRole {
    InputFrontier,
    TokenEnvelope,
    TextOrigin,
    BufferedTableCharacter,
}
pub struct HtmlParseDiagnosticLocation {
    pub(crate) source: crate::HtmlSourceRange,
    pub(crate) role: HtmlParseDiagnosticLocationRole,
    pub(crate) token: Option<HtmlParseTokenFamily>,
}
impl std::ops::Deref for HtmlParseDiagnosticLocation {
    type Target = crate::HtmlSourceRange;
    fn deref(&self) -> &Self::Target {
        &self.source
    }
}
pub struct DiagnosticLocationInput<'a> {
    pub(crate) source: &'a crate::HtmlSourceRange,
    pub(crate) role: HtmlParseDiagnosticLocationRole,
    pub(crate) token: Option<HtmlParseTokenFamily>,
}
pub(super) struct EvidenceIdentity;
#[cfg(test)]
thread_local! {
    /// The work credits in use right after each drop was counted, in order, on
    /// this thread. A test clears it, parses, then reads where each drop landed.
    pub(crate) static DROP_MARKS: std::cell::RefCell<Vec<u64>> =
        const { std::cell::RefCell::new(Vec::new()) };
}
/// What became of one emitted diagnostic. The `Diagnostics` cap bounds
/// retention, so a diagnostic past it is counted and dropped, not refused.
pub(crate) enum DiagnosticAdmission {
    /// The diagnostic is the next retained record; the identifier names it.
    Retained(DiagnosticId),
    /// The cap was already reached: the diagnostic was counted and not kept.
    Dropped,
    /// The parse did not select diagnostics: nothing was counted or kept.
    Unselected,
}
pub(crate) struct DiagnosticId {
    index: usize,
    owner: ControlIdentity,
    evidence: Arc<EvidenceIdentity>,
}
#[cfg(test)]
pub(crate) struct TokenizerDiagnosticView<'a> {
    pub(crate) code: HtmlTokenizerDiagnostic,
    pub(crate) phase: HtmlTokenizerDiagnosticPhase,
    pub(crate) state: HtmlTokenizerState,
    pub(crate) source: &'a crate::HtmlSourceRange,
}
pub(crate) struct ParseEvidence {
    pub(super) identity: Arc<EvidenceIdentity>,
    identity_allocation: SharedAllocation<EvidenceIdentity>,
    records: OwnedVec<HtmlParseDiagnostic>,
    locations: OwnedVec<HtmlParseDiagnosticLocation>,
    /// Diagnostics emitted after `records` reached the `Diagnostics` cap.
    dropped: u64,
    pub(super) native: super::provenance::NativeProvenance,
    diagnostics_taken: bool,
    /// The detail the parse selected: what this owner records.
    detail: crate::DetailSet,
}
/// The terminal consumer receives only committed diagnostics and their actual
/// issuer. No native association, construction cause or source-map draft can
/// survive this partition as a usable parse result.
pub(crate) struct FailureDiagnosticEvidence {
    _identity: Arc<EvidenceIdentity>,
    _identity_allocation: Option<SharedAllocation<EvidenceIdentity>>,
    records: OwnedVec<HtmlParseDiagnostic>,
    locations: OwnedVec<HtmlParseDiagnosticLocation>,
    dropped: u64,
    /// Whether the parse selected diagnostics.
    selected: bool,
}
/// Fixed borrowed lender of committed diagnostics. It carries no execution authority,
/// allocation ticket or native storage, and each observation uses constant work.
///
/// The `Diagnostics` limit bounds retention, not completion. The retained
/// records are the first diagnostics the parse emitted, up to that limit, in
/// the order it emitted them: the order of emission, not of source offset, since
/// delayed recovery can report an earlier offset after a later one. A
/// diagnostic emitted after the limit was reached is counted by
/// [`Self::dropped`] and kept nowhere, so `len() + dropped()` is the number of
/// diagnostics a parse of the same input retains under a limit that fits them
/// all, and the retained records are that parse's first `len()` records.
pub struct HtmlParseDiagnostics<'a> {
    records: &'a [HtmlParseDiagnostic],
    locations: &'a [HtmlParseDiagnosticLocation],
    dropped: u64,
    /// Whether the parse selected diagnostics.
    pub(crate) selected: bool,
}
impl HtmlParseDiagnostics<'_> {
    /// The number of retained diagnostics: the first ones the parse emitted.
    pub fn len(&self) -> usize {
        self.records.len()
    }
    /// The number of diagnostics emitted after the `Diagnostics` limit was
    /// reached and therefore not retained. Zero when every diagnostic fits.
    ///
    /// A caller that needs every diagnostic sets the limit high enough and
    /// checks that this is zero.
    pub fn dropped(&self) -> u64 {
        self.dropped
    }
    /// Whether no diagnostic is retained. This reflects retained records only:
    /// it is true when the `Diagnostics` limit is zero even though
    /// [`Self::dropped`] counts every diagnostic the parse emitted. Check
    /// `dropped() == 0` as well to learn that the parse emitted none.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    /// The committed record at `index` with the location it names, or `None`
    /// past the last record. Every committed record names a committed
    /// location; a record that did not is a fault in the report.
    pub fn retained_record(
        &self,
        index: usize,
    ) -> Result<
        Option<(&HtmlParseDiagnostic, &HtmlParseDiagnosticLocation)>,
        read::HtmlParseUnresolvedReference,
    > {
        let Some(record) = self.records.get(index) else {
            return Ok(None);
        };
        self.locations
            .get(record.location)
            .map(|location| Some((record, location)))
            .ok_or(read::HtmlParseUnresolvedReference::new(
                read::HtmlParseUnresolvedKind::Diagnostic,
            ))
    }
    #[cfg(test)]
    pub fn records_pointer(&self) -> *const HtmlParseDiagnostic {
        self.records.as_ptr()
    }
}
impl FailureDiagnosticEvidence {
    pub(crate) fn diagnostics(&self) -> HtmlParseDiagnostics<'_> {
        HtmlParseDiagnostics {
            records: self.records.as_slice(),
            locations: self.locations.as_slice(),
            dropped: self.dropped,
            selected: self.selected,
        }
    }

    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.records.len()
    }
    #[cfg(test)]
    pub(crate) fn retained_record(
        &self,
        index: usize,
    ) -> Option<(&HtmlParseDiagnostic, &HtmlParseDiagnosticLocation)> {
        self.records
            .as_slice()
            .get(index)
            .map(|record| (record, &self.locations.as_slice()[record.location]))
    }
    #[cfg(test)]
    pub(crate) fn records_pointer(&self) -> *const HtmlParseDiagnostic {
        self.records.as_slice().as_ptr()
    }
    #[cfg(test)]
    pub(crate) fn release(self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        self.records.release(c)?;
        self.locations.release(c)?;
        if let Some(allocation) = self._identity_allocation {
            c.release_zero_sized_shared(self._identity, allocation)
                .map_err(|error| error.into_parts().0)
        } else {
            drop(self._identity);
            Ok(())
        }
    }
}
#[cfg(test)]
impl ParseEvidence {
    /// Names a location that is not committed as the location of one
    /// diagnostic record. Whether the record exists.
    pub(super) fn corrupt_diagnostic_location_for_test(&mut self, index: usize) -> bool {
        match self.records.as_mut_slice_for_test().get_mut(index) {
            Some(record) => {
                record.location = usize::MAX;
                true
            }
            None => false,
        }
    }
}
impl ParseEvidence {
    pub(super) fn retains_diagnostics(&self) -> bool {
        !self.diagnostics_taken
    }
    pub(crate) fn diagnostics(&self) -> HtmlParseDiagnostics<'_> {
        HtmlParseDiagnostics {
            records: self.records.as_slice(),
            locations: self.locations.as_slice(),
            dropped: self.dropped,
            selected: self.records_diagnostics(),
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
    pub(crate) fn new(c: &mut ParseControl) -> Result<Self, HtmlParseStop> {
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
            dropped: 0,
            native,
            diagnostics_taken: false,
            detail: c.detail(),
        })
    }
    /// Whether the parse selected diagnostics. A parse that did not records
    /// none, charges none and keeps no storage for them.
    pub(crate) fn records_diagnostics(&self) -> bool {
        self.detail.contains(crate::DetailSet::DIAGNOSTICS)
    }
    /// Whether the parse selected the source map.
    pub(crate) fn records_source_map(&self) -> bool {
        self.detail.contains(crate::DetailSet::SOURCE_MAP)
    }
    /// Whether the parse selected the lexical records.
    pub(crate) fn records_lexical(&self) -> bool {
        self.detail.contains(crate::DetailSet::LEXICAL)
    }
    /// Whether the parse records the origin and cause graph.
    pub(crate) fn records_graph(&self) -> bool {
        self.detail.records_graph()
    }
    pub(crate) fn append(
        &mut self,
        c: &mut ParseControl,
        kind: HtmlParseDiagnosticKind,
        context: HtmlParseDiagnosticContext,
        rule: HtmlParseDiagnosticRule,
        input: DiagnosticLocationInput<'_>,
    ) -> Result<DiagnosticAdmission, HtmlParseStop> {
        if !self.records_diagnostics() {
            return Ok(DiagnosticAdmission::Unselected);
        }
        let source = input.source;
        // Ownership precedes every charge and the cap read, so a foreign
        // controller is refused whether or not its own cap is reached.
        self.records.require_owner(c)?;
        self.locations.require_owner(c)?;
        c.require_source_range(source)?;
        // The `Diagnostics` cap bounds retention only. Past it the diagnostic
        // is counted, with the count charged before it is written, and nothing
        // is allocated, so no other resource is touched either.
        if c.diagnostics_retention_full()? {
            c.charge_dropped_diagnostic()?;
            // Every drop was charged at least one work credit, and work credits
            // never exceed `u64::MAX`, so the count cannot saturate.
            self.dropped = self.dropped.saturating_add(1);
            #[cfg(test)]
            DROP_MARKS.with(|marks| {
                marks
                    .borrow_mut()
                    .push(c.usage().resource(HtmlParseResource::WorkCredits).current)
            });
            return Ok(DiagnosticAdmission::Dropped);
        }
        c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let index = self.records.len();
        let location = self.locations.len();
        // Handle and descriptor preparation precedes the atomic pair commit.
        c.charge(
            HtmlParseWorkCategory::CopyByte,
            const {
                (mem::size_of::<ControlIdentity>() + mem::size_of::<Arc<EvidenceIdentity>>()) as u64
            },
        )?;
        let owner = c.identity();
        let evidence = self.identity.clone();
        c.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<crate::HtmlSourceRange>() as u64,
        )?;
        let source = HtmlParseDiagnosticLocation {
            source: source.clone(),
            role: input.role,
            token: input.token,
        };
        // The slot is taken first: a cap that fills after the check above, in a
        // scope the parse shares, drops the diagnostic as the check does.
        if !c.try_retain_diagnostic()? {
            c.charge_dropped_diagnostic()?;
            self.dropped = self.dropped.saturating_add(1);
            return Ok(DiagnosticAdmission::Dropped);
        }
        let pushed = self.records.try_push_pair_held(
            &mut self.locations,
            c,
            HtmlParseDiagnostic {
                kind,
                context,
                severity: HtmlParseDiagnosticSeverity::Recoverable,
                rule,
                location,
            },
            source,
        );
        if let Err(stop) = pushed {
            c.release_unretained_diagnostic();
            return Err(stop);
        }
        Ok(DiagnosticAdmission::Retained(DiagnosticId {
            index,
            owner,
            evidence,
        }))
    }
    pub(crate) fn get<'a>(
        &'a self,
        c: &mut ParseControl,
        id: &DiagnosticId,
    ) -> Result<(&'a HtmlParseDiagnostic, &'a HtmlParseDiagnosticLocation), HtmlParseStop> {
        c.require_identity(&id.owner)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        c.charge(
            HtmlParseWorkCategory::InspectByte,
            mem::size_of::<Arc<EvidenceIdentity>>() as u64,
        )?;
        if !Arc::ptr_eq(&self.identity, &id.evidence) {
            return Err(c.fail_invariant(HtmlParseInvariant::ForeignEvidence));
        }
        self.records.require_disposal_owner(c)?;
        self.locations.require_disposal_owner(c)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
        let record = self
            .records
            .as_slice()
            .get(id.index)
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::RegistryRow))?;
        let location = self
            .locations
            .as_slice()
            .get(record.location)
            .ok_or_else(|| c.fail_invariant(HtmlParseInvariant::RegistryRow))?;
        Ok((record, location))
    }
    #[cfg(test)]
    pub(crate) fn len(&self) -> usize {
        self.records.len()
    }
    /// Fixed borrowed observations of already committed, self-contained data.
    /// Used by the stopped private consumer; this does not resume parse work.
    #[cfg(test)]
    pub(crate) fn retained_record(
        &self,
        index: usize,
    ) -> Option<(&HtmlParseDiagnostic, &HtmlParseDiagnosticLocation)> {
        self.records
            .as_slice()
            .get(index)
            .map(|record| (record, &self.locations.as_slice()[record.location]))
    }
    /// The number of diagnostics emitted past the `Diagnostics` cap so far.
    #[cfg(test)]
    pub(crate) fn dropped(&self) -> u64 {
        self.dropped
    }

    #[cfg(test)]
    pub(crate) fn tokenizer_observation(&self, id: &DiagnosticId) -> TokenizerDiagnosticView<'_> {
        assert!(
            Arc::ptr_eq(&self.identity, &id.evidence),
            "actual issuer-qualified ID"
        );
        let (r, source) = self.retained_record(id.index).expect("real emitted ID");
        let HtmlParseDiagnosticKind::Tokenizer(code) = r.kind else {
            panic!("lexer ID")
        };
        let HtmlParseDiagnosticContext::Tokenizer { phase, state } = r.context else {
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
    pub(crate) fn records_pointer(&self) -> *const HtmlParseDiagnostic {
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
        let Some(evidence) = input.take() else {
            return Ok(None);
        };
        // Every refusal gives the evidence back to its slot untouched.
        let preflight = (|| {
            evidence.records.require_disposal_owner(c)?;
            evidence.locations.require_disposal_owner(c)?;
            evidence.native.preflight_failed_disposal(c)?;
            Ok::<(), HtmlParseStop>(())
        })();
        if let Err(stop) = preflight {
            *input = Some(evidence);
            return Err(stop.into());
        }
        let Self {
            identity,
            identity_allocation,
            records,
            locations,
            dropped,
            native,
            diagnostics_taken: _,
            detail,
        } = evidence;
        // Same-owner flat vectors, exact checked capacities and their aggregate
        // logical/capacity totals were proved above. No ledger mutation intervenes,
        // so each cleanup subtraction is covered by that aggregate proof. The
        // controller is already stopped, so a refusal here would be secondary
        // (latched by its owner); the diagnostics are handed on regardless.
        let _ = native.release(c);
        Ok(Some(FailureDiagnosticEvidence {
            _identity: identity,
            _identity_allocation: Some(identity_allocation),
            records,
            locations,
            dropped,
            selected: detail.contains(crate::DetailSet::DIAGNOSTICS),
        }))
    }
    pub(crate) fn release(self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        self.native.release(c)?;
        self.records.release(c)?;
        self.locations.release(c)?;
        c.release_zero_sized_shared(self.identity, self.identity_allocation)
            .map_err(|error| error.into_parts().0)
    }
}
impl ParserArenaSession {
    #[cfg(test)]
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
        // The evidence is retained before it is initialized, so a refusal
        // leaves it with the session for cleanup.
        storage
            .evidence
            .insert(ParseEvidence::prepare(c)?)
            .initialize_native_source(c, source)?;
        let slots = storage.prepare_node_causes(c, 1)?;
        let [first, _] = slots;
        if let Some(slot) = first {
            let RetainedPayload::ConstructionCause { node, .. } =
                storage.payloads.row_mut(c, slot)?
            else {
                return Err(latch_registry_row(c));
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
    #[cfg(test)]
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
        c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        payload_write::<FailureDiagnosticEvidence>(c, 1)?;
        payload_write::<bool>(c, 1)?;
        let identity = evidence.identity.clone();
        let records = mem::replace(&mut evidence.records, records);
        let locations = mem::replace(&mut evidence.locations, locations);
        evidence.diagnostics_taken = true;
        Ok(FailureDiagnosticEvidence {
            _identity: identity,
            _identity_allocation: None,
            records,
            locations,
            dropped: mem::take(&mut evidence.dropped),
            selected: evidence.records_diagnostics(),
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

impl HtmlParseDiagnostic {
    pub fn kind(&self) -> HtmlParseDiagnosticKind {
        self.kind
    }
    pub fn context(&self) -> HtmlParseDiagnosticContext {
        self.context
    }
    pub fn severity(&self) -> HtmlParseDiagnosticSeverity {
        self.severity
    }
    pub fn rule(&self) -> HtmlParseDiagnosticRule {
        self.rule
    }
}
impl HtmlParseDiagnosticRule {
    pub fn anchor(&self) -> &'static str {
        self.anchor
    }
    pub fn branch(&self) -> &'static str {
        self.branch
    }
}
impl HtmlParseDiagnosticLocation {
    pub fn source_range(&self) -> &crate::HtmlSourceRange {
        &self.source
    }
    pub fn role(&self) -> HtmlParseDiagnosticLocationRole {
        self.role
    }
    pub fn token(&self) -> Option<HtmlParseTokenFamily> {
        self.token
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
    ) -> Result<DiagnosticAdmission, HtmlParseStop> {
        e.append(
            c,
            HtmlParseDiagnosticKind::Tokenizer(HtmlTokenizerDiagnostic::UnexpectedNullCharacter),
            HtmlParseDiagnosticContext::Tokenizer {
                phase: HtmlTokenizerDiagnosticPhase::Tokenizer,
                state: HtmlTokenizerState::Data,
            },
            HtmlParseDiagnosticRule {
                anchor: "parse-error-unexpected-null-character",
                branch: "unexpected-null-character",
            },
            DiagnosticLocationInput {
                source: &source.range(offset, offset + 1).unwrap(),
                role: HtmlParseDiagnosticLocationRole::InputFrontier,
                token: None,
            },
        )
    }
    /// The identifier of a diagnostic the test knows the cap retains.
    fn retained_id(admission: Result<DiagnosticAdmission, HtmlParseStop>) -> DiagnosticId {
        match admission.unwrap() {
            DiagnosticAdmission::Retained(id) => id,
            DiagnosticAdmission::Dropped => panic!("the diagnostic was dropped"),
            DiagnosticAdmission::Unselected => panic!("the diagnostic was not selected"),
        }
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
    fn a_record_identifier_past_the_retained_records_is_a_registry_row_invariant() {
        let source = HtmlSource::utf8("\0");
        let (mut c, mut evidence) = owner(&source, limits(100));
        evidence.initialize_native_source(&mut c, &source).unwrap();
        let mut id = retained_id(append(&mut evidence, &mut c, &source, 0));
        id.index = evidence.len();
        let stop = evidence.get(&mut c, &id).err().unwrap();
        assert_eq!(
            stop,
            HtmlParseStop::InternalInvariant(HtmlParseInvariant::RegistryRow)
        );
        assert_eq!(c.stop(), Some(stop));
        evidence.release(&mut c).unwrap();
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
        assert_eq!(c.usage(), before);

        let mut foreign = ParseControl::new(limits(100), HtmlParseCancellation::new());
        foreign.fail_invariant(HtmlParseInvariant::InputCursorState);
        let foreign_stop = foreign.stop();
        let foreign_before = foreign.usage().clone();
        assert!(ParseEvidence::partition_failed(&mut input, &mut foreign).is_err());
        assert_eq!(input.as_ref().unwrap().records_pointer(), pointer);
        assert_eq!(foreign.stop(), foreign_stop);
        assert_eq!(foreign.usage(), foreign_before);
        assert_eq!(c.usage(), before);

        c.fail_invariant(HtmlParseInvariant::InputCursorState);
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
            HtmlParseDiagnosticKind::Tokenizer(HtmlTokenizerDiagnostic::UnexpectedNullCharacter)
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
                        // Only the source-map cap is terminal. The diagnostics
                        // cap bounds retention, so the diagnostic is counted.
                        if resource == HtmlParseResource::Diagnostics {
                            assert!(matches!(result, Ok(DiagnosticAdmission::Dropped)));
                            assert_eq!(e.dropped, (offset as u64 - cap) + 1);
                        } else {
                            assert!(
                                matches!(result, Err(HtmlParseStop::LimitExceeded { resource: r, .. }) if r == resource)
                            );
                        }
                        assert_eq!(e.len(), cap.min(offset as u64) as usize);
                        assert_eq!(e.locations.len(), e.len());
                        assert_eq!(
                            c.usage().resource(HtmlParseResource::Diagnostics).current,
                            e.len() as u64
                        );
                        assert_eq!(
                            c.usage()
                                .resource(HtmlParseResource::SourceMapEntries)
                                .current,
                            e.len() as u64
                        );
                        if resource == HtmlParseResource::SourceMapEntries {
                            break;
                        }
                        continue;
                    }
                    let id = retained_id(result);
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
                        HtmlParseStop::LimitExceeded {
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
                        matches!(result,Err(HtmlParseStop::LimitExceeded {resource:r,..}) if r==resource)
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
            c.charge(
                HtmlParseWorkCategory::StateStep,
                available - cost + u64::from(short),
            )
            .unwrap();
            let result = append(&mut e, &mut c, &source, 1);
            if short {
                assert!(matches!(
                    result,
                    Err(HtmlParseStop::LimitExceeded {
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
                HtmlParseDiagnosticKind::Tokenizer(
                    HtmlTokenizerDiagnostic::UnexpectedNullCharacter
                )
            );
            assert_eq!((range.start(), range.end()), (0, 1));
            e.release(&mut c).unwrap();
            assert_released(&c);
        }
    }

    #[test]
    fn a_diagnostic_past_the_cap_is_counted_with_its_work_charged_before_the_count() {
        let source = HtmlSource::utf8("\0\0\0");
        let selected = limits(100).into_builder().diagnostics(1).finish().unwrap();
        // Probe the exact cost of the dropped append on a control of its own.
        let (mut probe, mut e) = owner(&source, selected);
        retained_id(append(&mut e, &mut probe, &source, 0));
        let before = probe.usage().clone();
        assert!(matches!(
            append(&mut e, &mut probe, &source, 1),
            Ok(DiagnosticAdmission::Dropped)
        ));
        let after = probe.usage().clone();
        let cost = after.resource(HtmlParseResource::WorkCredits).current
            - before.resource(HtmlParseResource::WorkCredits).current;
        // The range check (a visit and a provider call), then the visit of the
        // retention state and the write of the counter.
        assert_eq!(cost, 4);
        assert_eq!(e.dropped, 1);
        assert_eq!((e.len(), e.locations.len()), (1, 1));
        // Nothing is reserved or retained for the dropped diagnostic.
        for resource in [
            HtmlParseResource::Diagnostics,
            HtmlParseResource::SourceMapEntries,
            HtmlParseResource::ManagedCapacityBytes,
            HtmlParseResource::PeakTransientBytes,
            HtmlParseResource::AtomicLayoutBytes,
        ] {
            assert_eq!(after.resource(resource), before.resource(resource));
        }
        assert!(matches!(
            append(&mut e, &mut probe, &source, 2),
            Ok(DiagnosticAdmission::Dropped)
        ));
        assert_eq!(e.dropped, 2);
        e.release(&mut probe).unwrap();
        assert_released(&probe);

        // One short of that cost refuses the drop before it is counted.
        for short in [false, true] {
            let (mut c, mut e) = owner(&source, selected);
            retained_id(append(&mut e, &mut c, &source, 0));
            let available = selected.work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                HtmlParseWorkCategory::StateStep,
                available - cost + u64::from(short),
            )
            .unwrap();
            let result = append(&mut e, &mut c, &source, 1);
            if short {
                assert!(matches!(
                    result,
                    Err(HtmlParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    })
                ));
                assert_eq!(e.dropped, 0);
            } else {
                assert!(matches!(result, Ok(DiagnosticAdmission::Dropped)));
                assert_eq!(e.dropped, 1);
            }
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
                    HtmlParseStop::Cancelled
                } else {
                    HtmlParseStop::InternalInvariant(HtmlParseInvariant::ForeignSource)
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
        let id = retained_id(append(&mut first, &mut c, &source, 0));
        append(&mut other, &mut c, &source, 0).unwrap();
        assert!(matches!(
            other.get(&mut c, &id),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignEvidence
            ))
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
        let id = retained_id(append(&mut e, &mut c, &source, 0));
        let before = c.usage().clone();
        let mut foreign = ParseControl::new(limits(100), HtmlParseCancellation::new());
        foreign.admit_source(&source).unwrap();
        assert!(matches!(
            e.get(&mut foreign, &id),
            Err(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl
            ))
        ));
        assert!(append(&mut e, &mut foreign, &source, 1).is_err());
        assert_eq!((e.len(), e.locations.len()), (1, 1));
        assert_eq!(c.usage(), before);
        e.release(&mut c).unwrap();
        assert_released(&c);
    }

    #[test]
    fn a_foreign_controller_at_its_diagnostics_cap_is_refused_not_counted() {
        let source = HtmlSource::utf8("\0\0");
        let (mut c, mut e) = owner(&source, limits(100));
        retained_id(append(&mut e, &mut c, &source, 0));
        let before = c.usage().clone();
        // The foreign controller's own cap is already reached, so a retained
        // append would not be attempted: only the owner check can refuse it.
        let zero = limits(100).into_builder().diagnostics(0).finish().unwrap();
        let mut foreign = ParseControl::new(zero, HtmlParseCancellation::new());
        foreign.admit_source(&source).unwrap();
        let foreign_before = foreign.usage().clone();
        assert_eq!(
            append(&mut e, &mut foreign, &source, 1).err(),
            Some(HtmlParseStop::InternalInvariant(
                HtmlParseInvariant::ForeignControl
            ))
        );
        assert_eq!(e.dropped, 0);
        assert_eq!((e.len(), e.locations.len()), (1, 1));
        assert_eq!(c.usage(), before);
        assert_eq!(foreign.usage(), foreign_before);
        // The owner still counts its own drops afterwards.
        e.release(&mut c).unwrap();
        assert_released(&c);
    }

    #[test]
    fn fifth_diagnostic_pair_admits_the_doubled_capacity_of_both_actual_vectors() {
        let source = HtmlSource::utf8("\0\0\0\0\0");
        let record = mem::size_of::<HtmlParseDiagnostic>() as u64;
        let location = mem::size_of::<HtmlParseDiagnosticLocation>() as u64;
        // At four pairs both vectors are full. Each doubles to eight slots, the records first:
        // the pair then retains eight of each, and the larger of the two replacements overlaps
        // the old and the new layout of the vector it replaces.
        let capacity = 8 * (record + location);
        let transient =
            (4 * (record + location) + 8 * record).max(8 * record + 4 * location + 8 * location);
        for physical in [true, false] {
            // At the limit the fifth pair is admitted; one byte below it is refused with all
            // four earlier pairs retained.
            for refused in [false, true] {
                let builder = limits(100).into_builder();
                let (limit, resource) = if physical {
                    (capacity, HtmlParseResource::ManagedCapacityBytes)
                } else {
                    (transient, HtmlParseResource::PeakTransientBytes)
                };
                let limit = limit - u64::from(refused);
                let selected = if physical {
                    builder.managed_capacity_bytes(limit)
                } else {
                    builder.peak_transient_bytes(limit)
                }
                .finish()
                .unwrap();
                let (mut c, mut evidence) = owner(&source, selected);
                for offset in 0..4 {
                    append(&mut evidence, &mut c, &source, offset).unwrap();
                }
                let result = append(&mut evidence, &mut c, &source, 4);
                if refused {
                    assert!(matches!(
                        result,
                        Err(HtmlParseStop::LimitExceeded { resource: stopped, limit: refusing, .. })
                            if stopped == resource && refusing == limit
                    ));
                    assert_eq!((evidence.len(), evidence.locations.len()), (4, 4));
                    for offset in 0..4 {
                        let (record, location) = evidence.retained_record(offset).unwrap();
                        assert_eq!(record.location, offset);
                        assert_eq!((location.start(), location.end()), (offset, offset + 1));
                    }
                } else {
                    let id = retained_id(result);
                    assert_eq!(id.index, 4);
                    assert_eq!(c.stop(), None);
                    assert_eq!((evidence.len(), evidence.locations.len()), (5, 5));
                    assert_eq!(
                        (evidence.records.capacity(), evidence.locations.capacity()),
                        (8, 8)
                    );
                    if physical {
                        assert_eq!(c.usage().actual_capacity_bytes, capacity);
                    } else {
                        assert_eq!(
                            c.usage()
                                .resource(HtmlParseResource::PeakTransientBytes)
                                .high_water,
                            transient
                        );
                    }
                }
                evidence.release(&mut c).unwrap();
                assert_released(&c);
            }
        }
    }

    #[test]
    fn fifth_pair_one_short_work_keeps_original_prefix_and_prepared_first_capacity() {
        let source = HtmlSource::utf8("\0\0\0\0\0");
        let selected = limits(100);
        let (mut probe, mut evidence) = owner(&source, selected);
        for offset in 0..4 {
            append(&mut evidence, &mut probe, &source, offset).unwrap();
        }
        let before = probe
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        append(&mut evidence, &mut probe, &source, 4).unwrap();
        let cost = probe
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current
            - before;
        evidence.release(&mut probe).unwrap();
        assert_released(&probe);
        for short in [false, true] {
            let (mut c, mut evidence) = owner(&source, selected);
            for offset in 0..4 {
                append(&mut evidence, &mut c, &source, offset).unwrap();
            }
            let available = selected.work_credits()
                - c.usage().resource(HtmlParseResource::WorkCredits).current;
            c.charge(
                HtmlParseWorkCategory::StateStep,
                available - cost + u64::from(short),
            )
            .unwrap();
            let result = append(&mut evidence, &mut c, &source, 4);
            if short {
                assert!(matches!(
                    result,
                    Err(HtmlParseStop::LimitExceeded {
                        resource: HtmlParseResource::WorkCredits,
                        ..
                    })
                ));
                assert_eq!((evidence.len(), evidence.locations.len()), (4, 4));
                assert!(evidence.records.capacity() > 4);
                assert_eq!(evidence.locations.capacity(), 4);
                let first = c.stop();
                let snapshot = c.first_stop_usage().unwrap().clone();
                assert!(append(&mut evidence, &mut c, &source, 4).is_err());
                assert_eq!(c.stop(), first);
                assert_eq!(c.first_stop_usage(), Some(&snapshot));
                let mut slot = Some(evidence);
                let failed = ParseEvidence::partition_failed(&mut slot, &mut c)
                    .unwrap()
                    .unwrap();
                assert!(slot.is_none());
                for offset in 0..4 {
                    let (record, location) = failed.retained_record(offset).unwrap();
                    assert_eq!(record.location, offset);
                    assert_eq!((location.start(), location.end()), (offset, offset + 1));
                }
                failed.release(&mut c).unwrap();
                assert_eq!(c.stop(), first);
                assert_eq!(c.first_stop_usage(), Some(&snapshot));
            } else {
                assert!(result.is_ok());
                assert_eq!((evidence.len(), evidence.locations.len()), (5, 5));
                evidence.release(&mut c).unwrap();
            }
            assert_released(&c);
        }
    }

    #[test]
    fn second_location_one_short_capacity_keeps_the_prepared_record_and_original_prefix() {
        let source = HtmlSource::utf8("\0\0\0\0\0");
        let record_bytes = mem::size_of::<HtmlParseDiagnostic>() as u64;
        let location_bytes = mem::size_of::<HtmlParseDiagnosticLocation>() as u64;
        for prefix in [0_usize, 4] {
            // Both vectors reach the greater of one more slot and double their capacity: one
            // slot when empty, eight when four are full.
            let slots = (prefix as u64 + 1).max(2 * prefix as u64);
            let required = slots * (record_bytes + location_bytes);
            let selected = limits(100)
                .into_builder()
                .managed_capacity_bytes(required - 1)
                .finish()
                .unwrap();
            let (mut c, mut evidence) = owner(&source, selected);
            for offset in 0..prefix {
                append(&mut evidence, &mut c, &source, offset).unwrap();
            }
            let result = append(&mut evidence, &mut c, &source, prefix);
            let first_capacity = slots * record_bytes + prefix as u64 * location_bytes;
            assert_eq!(
                result.err(),
                Some(HtmlParseStop::LimitExceeded {
                    resource: HtmlParseResource::ManagedCapacityBytes,
                    current: first_capacity,
                    requested_delta: (slots - prefix as u64) * location_bytes,
                    limit: required - 1,
                })
            );
            assert_eq!((evidence.len(), evidence.locations.len()), (prefix, prefix));
            assert_eq!(evidence.records.capacity() as u64, slots);
            assert_eq!(evidence.locations.capacity(), prefix);
            assert_eq!(c.usage().actual_capacity_bytes, first_capacity);
            assert_eq!(
                c.usage().resource(HtmlParseResource::Diagnostics).current,
                prefix as u64
            );
            assert_eq!(
                c.usage()
                    .resource(HtmlParseResource::SourceMapEntries)
                    .current,
                prefix as u64
            );
            let stop = c.stop();
            let snapshot = c.first_stop_usage().unwrap().clone();
            for offset in 0..prefix {
                let (record, location) = evidence.retained_record(offset).unwrap();
                assert_eq!(record.location, offset);
                assert_eq!((location.start(), location.end()), (offset, offset + 1));
            }
            let mut slot = Some(evidence);
            let failure = ParseEvidence::partition_failed(&mut slot, &mut c)
                .unwrap()
                .unwrap();
            assert!(slot.is_none());
            assert_eq!(failure.len(), prefix);
            failure.release(&mut c).unwrap();
            assert_released(&c);
            assert_eq!(c.stop(), stop);
            assert_eq!(c.first_stop_usage(), Some(&snapshot));
        }
    }

    #[test]
    fn fifth_pair_second_logical_refusal_retains_spare_capacity_but_no_new_pair() {
        let source = HtmlSource::utf8("\0\0\0\0\0");
        let selected = limits(100)
            .into_builder()
            .source_map_entries(4)
            .finish()
            .unwrap();
        let (mut c, mut evidence) = owner(&source, selected);
        for offset in 0..4 {
            append(&mut evidence, &mut c, &source, offset).unwrap();
        }
        assert_eq!(
            append(&mut evidence, &mut c, &source, 4).err(),
            Some(HtmlParseStop::LimitExceeded {
                resource: HtmlParseResource::SourceMapEntries,
                current: 4,
                requested_delta: 1,
                limit: 4,
            })
        );
        assert_eq!((evidence.len(), evidence.locations.len()), (4, 4));
        assert_eq!(evidence.records.capacity(), 8);
        assert_eq!(evidence.locations.capacity(), 4);
        assert_eq!(
            c.usage().resource(HtmlParseResource::Diagnostics).current,
            4
        );
        let snapshot = c.first_stop_usage().unwrap().clone();
        let first = c.stop();
        evidence.release(&mut c).unwrap();
        assert_released(&c);
        assert_eq!(c.stop(), first);
        assert_eq!(c.first_stop_usage(), Some(&snapshot));
    }

    #[test]
    fn cancellation_or_foreign_control_before_fifth_pair_preserves_both_actual_buffers() {
        let source = HtmlSource::utf8("\0\0\0\0\0");
        for cancel in [true, false] {
            let cancellation = HtmlParseCancellation::new();
            let mut c = ParseControl::new(limits(100), cancellation.clone());
            c.admit_source(&source).unwrap();
            let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
            for offset in 0..4 {
                append(&mut evidence, &mut c, &source, offset).unwrap();
            }
            let before = c.usage().clone();
            let mut foreign = ParseControl::new(limits(100), HtmlParseCancellation::new());
            foreign.admit_source(&source).unwrap();
            if cancel {
                cancellation.cancel();
            }
            let active = if cancel { &mut c } else { &mut foreign };
            assert_eq!(
                append(&mut evidence, active, &source, 4).err(),
                Some(if cancel {
                    HtmlParseStop::Cancelled
                } else {
                    HtmlParseStop::InternalInvariant(HtmlParseInvariant::ForeignControl)
                })
            );
            assert_eq!((evidence.len(), evidence.locations.len()), (4, 4));
            assert_eq!(
                (evidence.records.capacity(), evidence.locations.capacity()),
                (4, 4)
            );
            assert_eq!(c.usage(), before);
            for offset in 0..4 {
                let (record, location) = evidence.retained_record(offset).unwrap();
                assert_eq!(record.location, offset);
                assert_eq!((location.start(), location.end()), (offset, offset + 1));
            }
            let first = active_stop(&c, &foreign, cancel);
            evidence.release(&mut c).unwrap();
            assert_released(&c);
            assert_released(&foreign);
            assert_eq!(active_stop(&c, &foreign, cancel), first);
        }
        fn active_stop(
            c: &ParseControl,
            foreign: &ParseControl,
            cancel: bool,
        ) -> Option<HtmlParseStop> {
            if cancel { c.stop() } else { foreign.stop() }
        }
    }

    #[test]
    fn tokenizer_null_comment_diagnostics_retain_bounded_pair_layouts() {
        use super::super::provenance::{LexicalOccurrence, LexicalPart};
        use crate::parser::tokenizer::{
            Tokenizer,
            token::{
                Event, HtmlParseOriginKind, HtmlParseTextOrigin, Text, Token, TokenEvent, TokenKind,
            },
        };

        // HTML 877d comment state: one error and FFFD for each NULL. The real
        // closed comment is followed by actual EOF, not a synthetic diagnostic
        // append or ordinary-tree fixture. All limits precede observations.
        // R=N+64; the quadratic term admits existing exact pair/comment/part
        // submissions, while linear/fixed room admits lexical and owner fees.
        // These finite fixture allowances are not an HTML work coefficient,
        // allocator/RSS claim or the archived 100M threshold.
        let per_record = mem::size_of::<HtmlParseDiagnostic>()
            + mem::size_of::<HtmlParseDiagnosticLocation>()
            + mem::size_of::<HtmlParseTextOrigin>()
            + mem::size_of::<LexicalOccurrence>()
            + mem::size_of::<LexicalPart>()
            + mem::size_of::<crate::HtmlSourceRange>()
            + mem::size_of::<u16>();
        let fixed = mem::size_of::<ParseControl>()
            + mem::size_of::<ParseEvidence>()
            + mem::size_of::<Tokenizer<'_>>()
            + mem::size_of::<TokenEvent>()
            + mem::size_of::<Token>()
            + mem::size_of::<Text>()
            + 8 * mem::size_of::<Option<Event>>();
        let exact_growth = mem::size_of::<HtmlParseDiagnostic>()
            + mem::size_of::<HtmlParseDiagnosticLocation>()
            + mem::size_of::<LexicalPart>()
            + mem::size_of::<u16>();
        let mut observations = Vec::new(); // Caller-owned observations only.
        for n in [0_usize, 1, 8, 32, 128, 512] {
            let room = (n + 64) as u64;
            let physical = 64 * room * per_record as u64 + 16 * fixed as u64;
            let work = 4 * room * room * exact_growth as u64
                + 1024 * room * per_record as u64
                + 32 * fixed as u64;
            let selected = crate::HtmlParseLimits::builder()
                .source_units((n + 7) as u64)
                .buffered_utf16_units(64 * room)
                .token_attributes(0)
                .auxiliary_entries(64 * room)
                .active_formatting_entries(64 * room)
                .managed_nodes(0)
                .managed_edges(0)
                .managed_attributes(0)
                .managed_utf16_units(64 * room)
                .source_map_entries(128 * room)
                .source_map_payload_units(64 * room)
                .diagnostics((n + 8) as u64)
                .diagnostic_payload_units(0)
                .context_effects(0)
                .open_elements(0)
                .tree_depth(0)
                .managed_capacity_bytes(physical)
                .peak_transient_bytes(2 * physical)
                .atomic_layout_bytes(physical)
                .work_credits(work)
                .finish()
                .unwrap();
            let source = HtmlSource::utf8(format!("<!--{}-->", "\0".repeat(n)));
            assert_eq!(source.len(), n + 7);
            let (mut c, mut evidence) = owner(&source, selected);
            let mut lexer =
                Tokenizer::new(&source, HtmlTokenizerState::Data, None, &mut c).unwrap();
            let mut record_slots = 0;
            let mut location_slots = 0;
            let mut record_reservations = 0;
            let mut location_reservations = 0;
            let mut diagnostics = 0;
            let mut comments = 0;
            let mut eof = 0;
            let mut exhausted = false;
            for _ in 0..(source.len() + 8) {
                let old_records = evidence.records.capacity();
                let old_locations = evidence.locations.capacity();
                let event = lexer.next(&mut c, false, &mut evidence).unwrap();
                let records = evidence.records.capacity();
                let locations = evidence.locations.capacity();
                if records != old_records {
                    record_slots += old_records + records;
                    record_reservations += 1;
                }
                if locations != old_locations {
                    location_slots += old_locations + locations;
                    location_reservations += 1;
                }
                let event = match event {
                    crate::parser::tokenizer::TokenizerPull::Event(event) => event,
                    crate::parser::tokenizer::TokenizerPull::Exhausted => {
                        exhausted = true;
                        break;
                    }
                };
                match &event {
                    Event::Diagnostic(id) => {
                        assert_eq!((comments, eof), (0, 0));
                        assert!(diagnostics < n);
                        assert_eq!(id.index, diagnostics);
                        assert!(Arc::ptr_eq(&id.evidence, &evidence.identity));
                        let (record, location) = evidence.get(&mut c, id).unwrap();
                        assert_eq!(
                            record.kind,
                            HtmlParseDiagnosticKind::Tokenizer(
                                HtmlTokenizerDiagnostic::UnexpectedNullCharacter
                            )
                        );
                        assert_eq!(
                            record.context,
                            HtmlParseDiagnosticContext::Tokenizer {
                                phase: HtmlTokenizerDiagnosticPhase::Tokenizer,
                                state: HtmlTokenizerState::Comment,
                            }
                        );
                        assert_eq!(record.severity, HtmlParseDiagnosticSeverity::Recoverable);
                        assert_eq!(record.rule.anchor, "parse-error-unexpected-null-character");
                        assert_eq!(record.rule.branch, "unexpected-null-character");
                        assert_eq!(record.location, diagnostics);
                        assert_eq!(
                            location.role,
                            HtmlParseDiagnosticLocationRole::InputFrontier
                        );
                        assert_eq!(location.token, None);
                        assert_eq!(location.source.source(), source.identity());
                        assert_eq!(
                            (location.source.start(), location.source.end()),
                            (4 + diagnostics, 5 + diagnostics)
                        );
                        diagnostics += 1;
                    }
                    Event::Token(value) => {
                        assert_eq!(diagnostics, n);
                        let token = value.token_for_tests();
                        assert_eq!(token.source.source(), source.identity());
                        match &token.kind {
                            TokenKind::Comment(text) => {
                                assert_eq!((comments, eof), (0, 0));
                                assert_eq!((token.source.start(), token.source.end()), (0, n + 7));
                                assert_eq!(text.value.text().code_units(), vec![0xfffd; n]);
                                assert_eq!(text.origins.len(), n);
                                for (i, origin) in text.origins.as_slice().iter().enumerate() {
                                    assert_eq!(origin.kind, HtmlParseOriginKind::NullReplacement);
                                    assert_eq!((origin.start, origin.end), (i, i + 1));
                                    assert_eq!(origin.source.source(), source.identity());
                                    assert_eq!(
                                        (origin.source.start(), origin.source.end()),
                                        (4 + i, 5 + i)
                                    );
                                }
                                comments += 1;
                            }
                            TokenKind::Eof => {
                                assert_eq!((comments, eof), (1, 0));
                                assert_eq!(
                                    (token.source.start(), token.source.end()),
                                    (n + 7, n + 7)
                                );
                                eof += 1;
                            }
                            _ => panic!("unexpected token in the literal NULL comment"),
                        }
                    }
                    Event::Preprocessing { .. } => panic!("no preprocessing in the literal source"),
                }
                assert_eq!(evidence.records.len(), diagnostics);
                assert_eq!(evidence.locations.len(), diagnostics);
                event.release(&mut c).unwrap();
            }
            assert!(exhausted, "actual lexical exhaustion for N={n}");
            assert_eq!((diagnostics, comments, eof), (n, 1, 1));
            assert_eq!((evidence.records.len(), evidence.locations.len()), (n, n));
            let final_records = evidence.records.capacity();
            let final_locations = evidence.locations.capacity();
            let parse_reserve = c.usage().work(HtmlParseWorkCategory::ReserveByte);
            let submitted_pair_bytes = record_slots * mem::size_of::<HtmlParseDiagnostic>()
                + location_slots * mem::size_of::<HtmlParseDiagnosticLocation>();
            assert!(parse_reserve >= submitted_pair_bytes as u64);
            let retained = evidence.diagnostics();
            assert_eq!(retained.len(), n);
            for i in 0..n {
                let (record, location) = retained.retained_record(i).unwrap().unwrap();
                assert_eq!(record.location, i);
                assert_eq!(
                    record.kind,
                    HtmlParseDiagnosticKind::Tokenizer(
                        HtmlTokenizerDiagnostic::UnexpectedNullCharacter
                    )
                );
                assert_eq!(location.source.source(), source.identity());
                assert_eq!(
                    (location.source.start(), location.source.end()),
                    (4 + i, 5 + i)
                );
            }
            assert!(retained.retained_record(n).unwrap().is_none());
            assert_eq!(c.stop(), None);
            let before_cleanup = c.usage().clone();
            lexer.release(&mut c).unwrap();
            evidence.release(&mut c).unwrap();
            assert_released(&c);
            assert_eq!(c.usage().actual_capacity_bytes, 0);
            for resource in [
                HtmlParseResource::BufferedUtf16Units,
                HtmlParseResource::TokenAttributes,
                HtmlParseResource::AuxiliaryEntries,
                HtmlParseResource::ManagedEdges,
                HtmlParseResource::ManagedAttributes,
                HtmlParseResource::ManagedUtf16Units,
                HtmlParseResource::SourceMapEntries,
                HtmlParseResource::SourceMapPayloadUnits,
                HtmlParseResource::Diagnostics,
                HtmlParseResource::DiagnosticPayloadUnits,
                HtmlParseResource::ContextEffects,
                HtmlParseResource::OpenElements,
                HtmlParseResource::ManagedCapacityBytes,
                HtmlParseResource::PeakTransientBytes,
            ] {
                assert_eq!(
                    c.usage().resource(resource).current,
                    0,
                    "N={n} {resource:?}"
                );
                assert!(
                    c.usage().resource(resource).high_water
                        >= before_cleanup.resource(resource).high_water
                );
            }
            assert!(c.usage().work(HtmlParseWorkCategory::ReserveByte) >= parse_reserve);
            assert_eq!(c.stop(), None);
            eprintln!(
                "Pair witness N={n} record_reservations={record_reservations} record_slots={record_slots} final_records={final_records} compiled_record_bytes={} location_reservations={location_reservations} location_slots={location_slots} final_locations={final_locations} compiled_location_bytes={} parse_all_writer_reserve_bytes={parse_reserve} cleanup_actual_bytes={}",
                mem::size_of::<HtmlParseDiagnostic>(),
                mem::size_of::<HtmlParseDiagnosticLocation>(),
                c.usage().actual_capacity_bytes
            );
            observations.push((n, record_slots, location_slots));
        }
        // All six literal diagnostic/token/source/issuer/EOF/cleanup cases were
        // checked before either growth assertion. These per-writer envelopes
        // neither attribute whole ReserveByte to the pair nor prescribe a
        // blanket Vec policy, physical copying or HTML linearity coefficient.
        for (n, record_slots, location_slots) in observations {
            assert!(
                record_slots <= 8 * n,
                "DiagnosticRecord layout submissions for N={n}: {record_slots} > {}",
                8 * n
            );
            assert!(
                location_slots <= 8 * n,
                "DiagnosticLocation layout submissions for N={n}: {location_slots} > {}",
                8 * n
            );
        }
    }

    #[test]
    fn a_retained_record_naming_a_missing_location_is_reported() {
        let record = HtmlParseDiagnostic {
            kind: HtmlParseDiagnosticKind::Tokenizer(
                HtmlTokenizerDiagnostic::UnexpectedNullCharacter,
            ),
            context: HtmlParseDiagnosticContext::Tokenizer {
                phase: HtmlTokenizerDiagnosticPhase::Tokenizer,
                state: HtmlTokenizerState::Data,
            },
            severity: HtmlParseDiagnosticSeverity::Recoverable,
            rule: HtmlParseDiagnosticRule {
                anchor: "parse-error-unexpected-null-character",
                branch: "unexpected-null-character",
            },
            location: 3,
        };
        let records = [record];
        let view = HtmlParseDiagnostics {
            records: &records,
            locations: &[],
            dropped: 0,
            selected: true,
        };
        assert_eq!(view.len(), 1);
        assert_eq!(
            view.retained_record(0).err(),
            Some(read::HtmlParseUnresolvedReference::new(
                read::HtmlParseUnresolvedKind::Diagnostic
            ))
        );
        assert!(matches!(view.retained_record(1), Ok(None)));
    }

    /// A parse sharing a host scope takes the last `Diagnostics` slot between
    /// the retention check and the acquisition. Retention bounds what is kept
    /// and never ends a parse, so the diagnostic is dropped and counted.
    #[test]
    fn a_host_slot_taken_after_the_retention_check_drops_the_diagnostic_and_the_parse_goes_on() {
        use surgeist_html::accounting::{HtmlLimits, HtmlResource};
        use surgeist_ledger::{Change, Operation, Tariffs};
        let host_limits = HtmlLimits::from_fn(
            |_| u64::MAX,
            |resource| {
                if resource == HtmlResource::Diagnostics {
                    1
                } else {
                    u64::MAX
                }
            },
        );
        let ledger = Operation::new("host", host_limits, 8).unwrap().share();
        let root = ledger.root();
        let sibling = ledger
            .child(
                root,
                "sibling",
                HtmlLimits::unlimited(),
                Tariffs::identity(),
            )
            .unwrap();
        let request = crate::HtmlParseAccounting::host(
            surgeist_html::accounting::HtmlHostScope::new(ledger.clone(), root),
        )
        .into_request();
        let source = HtmlSource::utf8("\0");
        let mut c = ParseControl::with_accounting(
            limits(100),
            HtmlParseCancellation::new(),
            request.lease(),
            crate::DetailSet::ALL,
        );
        c.admit_source(&source).unwrap();
        let mut evidence = ParseEvidence::prepare(&mut c).unwrap();
        evidence.initialize_native_source(&mut c, &source).unwrap();
        let taker = ledger.clone();
        c.after_next_retention_check(move || {
            taker
                .apply(sibling, Change::Acquire(HtmlResource::Diagnostics, 1))
                .unwrap();
        });
        assert!(matches!(
            append(&mut evidence, &mut c, &source, 0),
            Ok(DiagnosticAdmission::Dropped)
        ));
        assert_eq!(c.stop(), None);
        assert_eq!(evidence.dropped(), 1);
        assert_eq!(evidence.len(), 0);
        evidence.release(&mut c).unwrap();
        assert_released(&c);
    }
}
