//! Ordinary native Meta composition on the existing association owner.
use super::*;
pub(super) mod parsed;
use std::cell::Cell;
use std::rc::Rc;
use surgeist_dom::{
    DocumentString, DomBatch, DomMetaAtomicLimits, DomMetaInlineBatch, DomMetaOrdinaryCommit,
    DomMetaOrdinaryPreparation, DomMetaRuntimePrefix, DomMetaRuntimeUsage, DomSourceProgress,
};

/// Native Meta preparation failed before its joined owners published.
/// Retains the original Runtime refusal or root-qualified native producer error.
#[derive(Debug)]
pub enum DomStyleMetaError {
    /// Runtime qualification, preference, storage or work admission failed.
    Runtime(DomStyleRuntimeError),
    /// Canonical or producer preparation refused the actual native batch.
    Meta(surgeist_dom::DomMetaAtomicError),
}
impl std::fmt::Display for DomStyleMetaError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => write!(f, "native Meta Runtime preparation refused: {error}"),
            Self::Meta(error) => write!(f, "native Meta preparation refused: {error}"),
        }
    }
}
impl std::error::Error for DomStyleMetaError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Runtime(error) => Some(error),
            Self::Meta(error) => Some(error),
        }
    }
}
impl From<DomStyleRuntimeError> for DomStyleMetaError {
    fn from(error: DomStyleRuntimeError) -> Self {
        Self::Runtime(error)
    }
}
impl From<surgeist_dom::DomMetaAtomicError> for DomStyleMetaError {
    fn from(error: surgeist_dom::DomMetaAtomicError) -> Self {
        Self::Meta(error)
    }
}

/// One reached insertion-only preferred-name operation, in invocation order.
/// All observations for a Document report its one genuine final producer commit;
/// their preference revisions still distinguish the actual ordered transitions.
#[derive(Debug)]
pub struct DomStyleMetaPreference {
    meta: surgeist_dom::DomMetaSubject,
    observation: DomStyleSheetSetObservation,
    root_commit_index: usize,
}
impl DomStyleMetaPreference {
    /// Exact existing generation or opaque original phase-qualified subject.
    /// A subsequently destroyed created Meta need not receive a canonical ID;
    /// cloned/imported descendants need not have caller creation tickets.
    pub fn meta(&self) -> &surgeist_dom::DomMetaSubject {
        &self.meta
    }
    /// Actual preference transition with the joined canonical/producer bases.
    pub fn observation(&self) -> &DomStyleSheetSetObservation {
        &self.observation
    }
}
/// Complete ordinary canonical, producer and preference publication.
/// Immutable outputs belong to the caller; Runtime retains only current names.
#[must_use]
pub struct DomStyleMetaOrdinaryOutcome {
    commit: DomMetaOrdinaryCommit,
    preferences: Vec<DomStyleMetaPreference>,
}
impl DomStyleMetaOrdinaryOutcome {
    /// Authentic canonical, root CSSOM and optional inline outputs.
    pub fn commit(&self) -> &DomMetaOrdinaryCommit {
        &self.commit
    }
    /// Actual reached operations, including repeated names and removed Metas.
    pub fn preferences(&self) -> &[DomStyleMetaPreference] {
        &self.preferences
    }
    /// Move the authentic outputs once, without acknowledgement or replay.
    pub fn into_parts(self) -> (DomMetaOrdinaryCommit, Vec<DomStyleMetaPreference>) {
        (self.commit, self.preferences)
    }
}

/// One original Source advance with authentic joined owner evidence.
/// Without a reached Meta occurrence, the original Source/inline progress moves
/// unchanged and both root commits and preference observations are empty.
#[must_use]
pub struct DomStyleMetaSourceOutcome {
    source: DomSourceProgress,
    roots: Vec<surgeist_dom::DomMetaRootCommit>,
    inline: Option<surgeist_dom::DomInlineStyleSourcePublication>,
    preferences: Vec<DomStyleMetaPreference>,
}
impl std::fmt::Debug for DomStyleMetaSourceOutcome {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomStyleMetaSourceOutcome")
            .field("source", &self.source)
            .field("root_count", &self.roots.len())
            .field("inline", &self.inline)
            .field("preferences", &self.preferences)
            .finish()
    }
}
impl DomStyleMetaSourceOutcome {
    /// Exact next unserviced request or completed original operation.
    pub fn source(&self) -> &DomSourceProgress {
        &self.source
    }
    /// Genuine final producer outputs in first-occurrence Document order.
    pub fn roots(&self) -> &[surgeist_dom::DomMetaRootCommit] {
        &self.roots
    }
    /// Authentic paired inline publication, when the original operation made one.
    pub fn inline(&self) -> Option<&surgeist_dom::DomInlineStyleSourcePublication> {
        self.inline.as_ref()
    }
    /// Reached operations, including repeated names and semantic no-ops.
    pub fn preferences(&self) -> &[DomStyleMetaPreference] {
        &self.preferences
    }
    /// Move all original outputs once; no acknowledgement or replay follows.
    pub fn into_parts(
        self,
    ) -> (
        DomSourceProgress,
        Vec<surgeist_dom::DomMetaRootCommit>,
        Option<surgeist_dom::DomInlineStyleSourcePublication>,
        Vec<DomStyleMetaPreference>,
    ) {
        (self.source, self.roots, self.inline, self.preferences)
    }
}

struct Candidate {
    root_index: usize,
    root_commit_index: usize,
    next: Option<sheet_sets::State>,
}
struct NameRow<'a> {
    destination: &'a mut sheet_sets::State,
    next: Option<sheet_sets::State>,
}
impl NameRow<'_> {
    fn install(&mut self, retired: &mut Vec<sheet_sets::State>) {
        // Capacity and the exact destination were qualified before publication.
        // Keep every displaced immutable name alive across the whole fence.
        retired.push(std::mem::replace(
            self.destination,
            self.next.take().expect("one prepared name installation"),
        ));
    }
}

// This counter refers to the original Runtime work cell, not another work owner.
struct Steps<'a> {
    work: &'a Cell<u64>,
    maximum: u64,
    cancel: &'a AtomicBool,
}

// Source pays the real Operation/ancestor/native owners before Runtime work.
// This is not the ordinary numeric prefix or a second work bank.
struct SourceSteps<'a> {
    work: &'a Cell<u64>,
    maximum: u64,
    cancel: &'a AtomicBool,
}
impl SourceSteps<'_> {
    fn runtime(&self) -> Result<(), DomStyleRuntimeError> {
        StyleState::step_counter(self.work, self.cancel, self.maximum)
    }
    fn prefix(
        &self,
        hold: &surgeist_dom::DomMetaSourceRuntimeHold<'_, '_>,
    ) -> Result<(), DomStyleRuntimeError> {
        hold.step(self.cancel)?;
        self.runtime()
    }
    fn joined(
        &self,
        preparation: &surgeist_dom::DomMetaSourcePreparation<'_, '_, '_>,
    ) -> Result<(), DomStyleRuntimeError> {
        preparation.step(self.cancel)?;
        self.runtime()
    }
    fn same(
        &self,
        preparation: &surgeist_dom::DomMetaSourcePreparation<'_, '_, '_>,
        left: &[u16],
        right: &[u16],
    ) -> Result<bool, DomStyleRuntimeError> {
        self.joined(preparation)?;
        if left.len() != right.len() {
            return Ok(false);
        }
        for (a, b) in left.iter().zip(right) {
            self.joined(preparation)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

fn source_storage(
    state: &StyleState,
    hold: &mut surgeist_dom::DomMetaSourceRuntimeHold<'_, '_>,
    backing: &Backing,
    steps: &SourceSteps<'_>,
) -> Result<usize, DomStyleRuntimeError> {
    let headers = state.storage_headers(None, || steps.prefix(hold))?;
    let remaining = steps.maximum.saturating_sub(steps.work.get());
    let mut fault = None;
    let mut admit = |action| {
        if fault.is_some() {
            return false;
        }
        let result = match action {
            surgeist_css::CssSourceRetentionAction::Step => steps.prefix(hold),
            surgeist_css::CssSourceRetentionAction::Scratch { live_bytes } => (|| {
                steps.prefix(hold)?;
                let bytes = backing
                    .bytes
                    .checked_add(live_bytes)
                    .ok_or(DomStyleRuntimeError::Admission)?;
                hold.admit_runtime(
                    DomMetaRuntimeUsage::new(bytes, backing.units, backing.outputs)?,
                    steps.cancel,
                )?;
                Ok(())
            })(),
        };
        match result {
            Ok(()) => true,
            Err(error) => {
                if fault.is_none() {
                    fault = Some(error);
                }
                false
            }
        }
    };
    let measured = (|| {
        let mut sources = CssSourceRetention::with_admission(
            usize::MAX,
            usize::try_from(remaining).unwrap_or(usize::MAX),
            usize::try_from(remaining).unwrap_or(usize::MAX),
            &mut admit,
        )
        .map_err(|_| DomStyleRuntimeError::Admission)?;
        state.processing(None, |value| {
            sources
                .charge_step()
                .map_err(|_| DomStyleRuntimeError::Admission)?;
            for diagnostic in value.diagnostics() {
                sources
                    .visit(diagnostic)
                    .map_err(|_| DomStyleRuntimeError::Admission)?;
            }
            Ok(())
        })?;
        sources
            .complete_payload_bytes_admitted()
            .map_err(|_| DomStyleRuntimeError::Admission)
    })();
    // The collector and its bank are already destroyed before the typed caller
    // refusal is recovered or its external hold charge can be released.
    if let Some(error) = fault {
        return Err(error);
    }
    headers
        .checked_add(measured?)
        .ok_or(DomStyleRuntimeError::Admission)
}
impl Steps<'_> {
    fn runtime(&self) -> Result<(), DomStyleRuntimeError> {
        StyleState::step_counter(self.work, self.cancel, self.maximum)
    }
    fn joined(
        &self,
        preparation: &DomMetaOrdinaryPreparation<'_, '_>,
    ) -> Result<(), DomStyleRuntimeError> {
        self.runtime()?;
        preparation.step(self.cancel)?;
        Ok(())
    }
    fn same(
        &self,
        preparation: &DomMetaOrdinaryPreparation<'_, '_>,
        left: &[u16],
        right: &[u16],
    ) -> Result<bool, DomStyleRuntimeError> {
        self.joined(preparation)?;
        if left.len() != right.len() {
            return Ok(false);
        }
        for (a, b) in left.iter().zip(right) {
            self.joined(preparation)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
}

// A private normalized capacity policy, with the diagnostics of its actual
// public domain. Native and parsed owners retain distinct public envelopes.
struct BackingLimits {
    bytes: usize,
    units: usize,
    outputs: usize,
    resources: [surgeist_dom::DomResource; 3],
}
impl BackingLimits {
    fn native(limits: DomMetaAtomicLimits) -> Self {
        Self {
            bytes: limits.retained_bytes(),
            units: limits.retained_units(),
            outputs: limits.outputs(),
            resources: [
                surgeist_dom::DomResource::MetaAtomicRetainedBytes,
                surgeist_dom::DomResource::MetaAtomicRetainedUnits,
                surgeist_dom::DomResource::MetaAtomicOutputs,
            ],
        }
    }
    fn parsed(limits: surgeist_dom::DomParsedHtmlAssociationLimits) -> Self {
        Self {
            bytes: limits.retained_bytes(),
            units: limits.retained_units(),
            outputs: limits.outputs(),
            resources: [
                surgeist_dom::DomResource::ParsedAssociationProcessingRetainedBytes,
                surgeist_dom::DomResource::ParsedAssociationProcessingRetainedUnits,
                surgeist_dom::DomResource::ParsedAssociationProcessingOutputs,
            ],
        }
    }
}

// Actual temporary/output backing beside the unchanged canonical Runtime bank.
// Producer-private graphs remain charged by DOM/CSSOM, not a payload tariff here.
struct Backing {
    bytes: usize,
    units: usize,
    dom_shared_bytes: usize,
    dom_shared_units: usize,
    outputs: usize,
    current: usize,
    runtime_limit: usize,
    limits: BackingLimits,
}
impl Backing {
    fn check(&self) -> Result<(), DomStyleRuntimeError> {
        for (actual, limit, resource) in [
            (
                self.bytes - self.dom_shared_bytes,
                self.limits.bytes,
                self.limits.resources[0],
            ),
            (
                self.units - self.dom_shared_units,
                self.limits.units,
                self.limits.resources[1],
            ),
            (self.outputs, self.limits.outputs, self.limits.resources[2]),
        ] {
            if actual > limit {
                return Err(DomError::ResourceLimit { resource, limit }.into());
            }
        }
        if self
            .current
            .checked_add(self.bytes)
            .is_none_or(|n| n > self.runtime_limit)
        {
            return Err(DomStyleRuntimeError::Admission);
        }
        Ok(())
    }
    fn add(&mut self, bytes: usize) -> Result<(), DomStyleRuntimeError> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.check()
    }
    fn retain_name(
        &mut self,
        bytes: usize,
        units: usize,
        shared_dom: bool,
    ) -> Result<(), DomStyleRuntimeError> {
        let next_bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or(DomStyleRuntimeError::Admission)?;
        let next_units = self
            .units
            .checked_add(units)
            .ok_or(DomStyleRuntimeError::Admission)?;
        let shared_bytes = self
            .dom_shared_bytes
            .checked_add(if shared_dom { bytes } else { 0 })
            .ok_or(DomStyleRuntimeError::Admission)?;
        let shared_units = self
            .dom_shared_units
            .checked_add(if shared_dom { units } else { 0 })
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.bytes = next_bytes;
        self.units = next_units;
        self.dom_shared_bytes = shared_bytes;
        self.dom_shared_units = shared_units;
        self.check()
    }
    fn reserve<T>(
        &mut self,
        values: &mut Vec<T>,
        count: usize,
    ) -> Result<(), DomStyleRuntimeError> {
        // Every bank starts empty and is reserved once; no hidden replacement.
        let requested = count
            .checked_mul(size_of::<T>())
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.add(requested)?;
        values
            .try_reserve_exact(count)
            .map_err(|_| DomStyleRuntimeError::Admission)?;
        let actual = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.add(
            actual
                .checked_sub(requested)
                .ok_or(DomStyleRuntimeError::Admission)?,
        )
    }
    fn reserve_joined<T>(
        &mut self,
        values: &mut Vec<T>,
        count: usize,
        preparation: &mut DomMetaOrdinaryPreparation<'_, '_>,
        steps: &Steps<'_>,
    ) -> Result<(), DomStyleRuntimeError> {
        let requested = count
            .checked_mul(size_of::<T>())
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.add(requested)?;
        steps.runtime()?;
        preparation.admit_runtime(self.native_usage()?, steps.cancel)?;
        values
            .try_reserve_exact(count)
            .map_err(|_| DomStyleRuntimeError::Admission)?;
        let actual = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.add(
            actual
                .checked_sub(requested)
                .ok_or(DomStyleRuntimeError::Admission)?,
        )?;
        steps.runtime()?;
        preparation.admit_runtime(self.native_usage()?, steps.cancel)?;
        Ok(())
    }
    fn native_usage(&self) -> Result<DomMetaRuntimeUsage, DomStyleRuntimeError> {
        Ok(DomMetaRuntimeUsage::new(
            self.bytes - self.dom_shared_bytes,
            self.units - self.dom_shared_units,
            self.outputs,
        )?)
    }
    fn reserve_prefix<T>(
        &mut self,
        values: &mut Vec<T>,
        count: usize,
        hold: &mut surgeist_dom::DomMetaSourceRuntimeHold<'_, '_>,
        steps: &SourceSteps<'_>,
    ) -> Result<(), DomStyleRuntimeError> {
        let requested = count
            .checked_mul(size_of::<T>())
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.add(requested)?;
        steps.prefix(hold)?;
        hold.admit_runtime(self.native_usage()?, steps.cancel)?;
        values
            .try_reserve_exact(count)
            .map_err(|_| DomStyleRuntimeError::Admission)?;
        let actual = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.add(
            actual
                .checked_sub(requested)
                .ok_or(DomStyleRuntimeError::Admission)?,
        )?;
        steps.prefix(hold)?;
        hold.admit_runtime(self.native_usage()?, steps.cancel)?;
        Ok(())
    }
    fn reserve_source<T>(
        &mut self,
        values: &mut Vec<T>,
        count: usize,
        preparation: &mut surgeist_dom::DomMetaSourcePreparation<'_, '_, '_>,
        steps: &SourceSteps<'_>,
    ) -> Result<(), DomStyleRuntimeError> {
        let requested = count
            .checked_mul(size_of::<T>())
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.add(requested)?;
        steps.joined(preparation)?;
        preparation.admit_runtime(self.native_usage()?, steps.cancel)?;
        values
            .try_reserve_exact(count)
            .map_err(|_| DomStyleRuntimeError::Admission)?;
        let actual = values
            .capacity()
            .checked_mul(size_of::<T>())
            .ok_or(DomStyleRuntimeError::Admission)?;
        self.add(
            actual
                .checked_sub(requested)
                .ok_or(DomStyleRuntimeError::Admission)?,
        )?;
        steps.joined(preparation)?;
        preparation.admit_runtime(self.native_usage()?, steps.cancel)?;
        Ok(())
    }
}

impl DomStyleRuntime {
    /// Execute actual native Meta insertion operations with all reached Document
    /// CSSOM owners, optional inline edits and preference history atomically.
    /// Raw context-free DOM fronts retain their reached unavailable refusal.
    /// A failure occurs before any producer or canonical name publication; an
    /// ordinary failed input is consumed. Each ordered preference observation
    /// consumes one native output in addition to the lower actual outputs.
    /// This does not consume parsed Pending
    /// history, supply Source permission, or acknowledge earlier Style custody.
    #[expect(
        clippy::result_large_err,
        reason = "Retains the exact root and original producer failure without allocating on the native refusal path"
    )]
    pub fn apply_meta(
        &self,
        dom: &mut DomEventDom<'_>,
        batch: DomBatch,
        inline: Option<DomMetaInlineBatch<'_>>,
        limits: DomMetaAtomicLimits,
        cancel: &AtomicBool,
    ) -> Result<DomStyleMetaOrdinaryOutcome, DomStyleMetaError> {
        let owner = self.qualified(dom)?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        // The ordinary front shares the real standalone exclusion boundary.
        // Source composition will use its own original phase, not this front.
        let stamp = state.require_sheet_set_idle(dom)?.stamp();
        let initial_work = state.work.get();
        let prefix_maximum = initial_work
            .checked_add(limits.work())
            .ok_or(DomStyleRuntimeError::Dom(DomError::RepresentationExhausted))?
            .min(state.limits.work);
        let current = state.storage(None, cancel, prefix_maximum)?;
        let mut backing = Backing {
            bytes: size_of::<DomStyleMetaOrdinaryOutcome>()
                + size_of::<Vec<Rc<DomStyleCssomRoot>>>()
                + size_of::<Vec<&DomStyleCssomRoot>>()
                + size_of::<Vec<Candidate>>()
                + size_of::<Vec<NameRow<'_>>>()
                + size_of::<Vec<sheet_sets::State>>()
                + size_of::<sheet_sets::State>()
                + size_of::<DocumentString>()
                + size_of::<DomStyleMetaPreference>(),
            units: 0,
            dom_shared_bytes: 0,
            dom_shared_units: 0,
            outputs: 0,
            current,
            runtime_limit: state.limits.storage_bytes,
            limits: BackingLimits::native(limits),
        };
        backing.check()?;
        let mut capabilities = Vec::new();
        let mut references = Vec::new();
        backing.reserve(&mut capabilities, state.roots.len())?;
        backing.reserve(&mut references, state.roots.len())?;
        for root in &state.roots {
            StyleState::step_counter(&state.work, cancel, prefix_maximum)?;
            if let RootResource::Bound(owner) = &root.resource {
                capabilities.push(owner.clone());
            }
        }
        for capability in &capabilities {
            StyleState::step_counter(&state.work, cancel, prefix_maximum)?;
            references.push(capability.as_ref());
        }
        StyleState::step_counter(&state.work, cancel, prefix_maximum)?;
        let formation_work = state.work.get() - initial_work;
        // Both banks are outside every DOM loan and remain stable until all
        // returned producer guards have been extracted and released.
        let prefix = DomMetaRuntimePrefix::new(backing.native_usage()?, formation_work, limits)
            .map_err(DomStyleRuntimeError::from)?;
        let mut preparation = dom
            .raw_mut()
            .prepare_meta_batch(batch, &references, inline, limits, prefix, cancel)
            .map_err(|error| {
                if let surgeist_dom::DomMetaAtomicError::MissingOwner { root } = &error
                    && let Err(reason) = state.require_bound_if_registered(*root, cancel)
                {
                    return DomStyleMetaError::Runtime(reason);
                }
                DomStyleMetaError::Meta(error)
            })?;
        let StyleState {
            roots,
            work,
            limits: runtime_limits,
            ..
        } = &mut *state;
        let steps = Steps {
            work,
            maximum: runtime_limits.work,
            cancel,
        };
        let count = preparation.occurrences().len();
        backing.outputs = count;
        backing.check()?;
        let mut candidates: Vec<Candidate> = Vec::new();
        let mut preferences: Vec<DomStyleMetaPreference> = Vec::new();
        let mut names = Vec::new();
        let mut retired = Vec::new();
        backing.reserve_joined(&mut candidates, count, &mut preparation, &steps)?;
        backing.reserve_joined(&mut preferences, count, &mut preparation, &steps)?;
        backing.reserve_joined(&mut names, count, &mut preparation, &steps)?;
        backing.reserve_joined(&mut retired, count, &mut preparation, &steps)?;
        for occurrence_index in 0..count {
            steps.joined(&preparation)?;
            let occurrence = &preparation.occurrences()[occurrence_index];
            let document = occurrence.document();
            let meta = occurrence.meta().clone();
            let content = occurrence.content().clone();
            let mut candidate_index = None;
            for (index, candidate) in candidates.iter().enumerate() {
                steps.joined(&preparation)?;
                if roots[candidate.root_index].node == document {
                    candidate_index = Some(index);
                    break;
                }
            }
            let index = if let Some(index) = candidate_index {
                index
            } else {
                let mut root_index = None;
                for (index, root) in roots.iter().enumerate() {
                    steps.joined(&preparation)?;
                    if root.node == document {
                        root_index = Some(index);
                        break;
                    }
                }
                let root_index = root_index
                    .ok_or(DomStyleRuntimeError::SheetSetRootUnavailable { root: document })?;
                let current = roots[root_index]
                    .sheet_sets
                    .state()
                    .ok_or(DomStyleRuntimeError::SheetSetDocumentRequired { root: document })?;
                let index = candidates.len();
                candidates.push(Candidate {
                    root_index,
                    root_commit_index: index,
                    next: Some(current.clone()),
                });
                index
            };
            let current = candidates[index].next.as_ref().expect("prepared candidate");
            let (next, enable) =
                current.transition(&content, sheet_sets::Action::Preferred, |left, right| {
                    steps.same(&preparation, left, right)
                })?;
            // Only successor/output names actually retained by Runtime belong
            // to its old/new union. Native recording already owns any alias in
            // the occurrence bank; exclude that backing from the shared native
            // envelope while still checking Runtime's own storage limit.
            for (name_index, name) in next.names().enumerate() {
                steps.joined(&preparation)?;
                let allocation = name.shared_code_units();
                let mut retained = false;
                for root in roots.iter() {
                    steps.joined(&preparation)?;
                    if let Some(current) = root.sheet_sets.state() {
                        for old in current.names() {
                            steps.joined(&preparation)?;
                            retained |= Arc::ptr_eq(allocation, old.shared_code_units());
                        }
                    }
                }
                for previous in &preferences {
                    steps.joined(&preparation)?;
                    for old in std::iter::once(previous.observation.preferred_name())
                        .chain(previous.observation.last_selected_name())
                    {
                        steps.joined(&preparation)?;
                        retained |= Arc::ptr_eq(allocation, old.shared_code_units());
                    }
                }
                for old in next.names().take(name_index) {
                    steps.joined(&preparation)?;
                    retained |= Arc::ptr_eq(allocation, old.shared_code_units());
                }
                if retained {
                    continue;
                }
                let mut shared_dom = false;
                for original in preparation.occurrences() {
                    steps.joined(&preparation)?;
                    shared_dom |= Arc::ptr_eq(allocation, original.content().shared_code_units());
                }
                let units = allocation.capacity();
                let bytes = units
                    .checked_mul(size_of::<u16>())
                    .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
                    .ok_or(DomStyleRuntimeError::Admission)?;
                backing.retain_name(bytes, units, shared_dom)?;
                steps.runtime()?;
                preparation
                    .admit_runtime(backing.native_usage()?, cancel)
                    .map_err(DomStyleRuntimeError::from)?;
            }
            steps.runtime()?;
            let cssom = preparation.root_snapshot(document, cancel)?;
            let producer_revision = cssom.revision().clone();
            let sheet_count = cssom.sheets().len();
            if enable {
                for sheet_index in 0..sheet_count {
                    steps.joined(&preparation)?;
                    steps.runtime()?;
                    let cssom = preparation.root_snapshot(document, cancel)?;
                    let sheet = cssom.sheets()[sheet_index].clone();
                    let title = &cssom
                        .sheet(&sheet)
                        .map_err(DomStyleRuntimeError::from)?
                        .inputs()
                        .title;
                    let disabled = sheet_sets::named_sheet_disabled(
                        title,
                        content.code_units(),
                        |left, right| steps.same(&preparation, left, right),
                    )?;
                    if let Some(disabled) = disabled {
                        // Always edit the genuine staged batch: a later B -> A
                        // occurrence must undo B even when A equals base flags.
                        steps.runtime()?;
                        preparation.set_disabled(document, &sheet, disabled, cancel)?;
                    }
                }
            }
            preferences.push(DomStyleMetaPreference {
                meta,
                observation: DomStyleSheetSetObservation::from_state(
                    stamp,
                    document,
                    &next,
                    producer_revision,
                ),
                root_commit_index: candidates[index].root_commit_index,
            });
            candidates[index].next = Some(next);
        }
        // Split only already-qualified exact destinations. No root lookup,
        // variant match or RefCell borrowing remains after producer publication.
        for (index, root) in roots.iter_mut().enumerate() {
            steps.joined(&preparation)?;
            for candidate in &mut candidates {
                steps.joined(&preparation)?;
                if candidate.root_index == index {
                    names.push(NameRow {
                        destination: root
                            .sheet_sets
                            .state_mut()
                            .expect("qualified Document destination"),
                        next: candidate.next.take(),
                    });
                    break;
                }
            }
        }
        steps.runtime()?;
        let published = preparation.commit(cancel)?;
        // This suffix contains only preadmitted canonical moves and output
        // completion. Producer guards stay held through every name install.
        for row in &mut names {
            row.install(&mut retired);
        }
        let (commit, retirement) = published.into_parts();
        for preference in &mut preferences {
            let actual = commit.roots()[preference.root_commit_index].cssom();
            let revision = match actual.publication() {
                surgeist_cssom::CssomPublication::Changed(summary) => summary.to(),
                surgeist_cssom::CssomPublication::Unchanged { revision } => revision,
            };
            preference
                .observation
                .finish_publication(commit.dom().changes().to(), revision.clone());
        }
        let outcome = DomStyleMetaOrdinaryOutcome {
            commit,
            preferences,
        };
        // These potentially last-owner disposals occur after the complete fence.
        drop(names);
        drop(candidates);
        drop(retired);
        drop(retirement);
        Ok(outcome)
    }
    /// Advance the exact original Source operation through its native Meta join.
    /// Prior Style publication must already have been acknowledged. Original
    /// profiles and producer limits remain with the operation; only its resolver
    /// is borrowed for this execution. Failed attempts retain original spent
    /// Source/ancestor/native work, policy and pending publication custody.
    #[expect(
        clippy::result_large_err,
        reason = "Source composition retains the actual root and original producer failure without allocating"
    )]
    pub fn advance_meta(
        &self,
        lease: &mut DomSourceLease<'_>,
        resolver: Option<&mut surgeist_dom::DomMetaInlineResolver<'_>>,
        limits: DomMetaAtomicLimits,
        cancel: &AtomicBool,
    ) -> Result<DomStyleMetaSourceOutcome, DomStyleMetaError> {
        let owner = self.original()?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        let view = lease.snapshot();
        if state.dom != view.owner() {
            return Err(DomStyleRuntimeError::WrongSession.into());
        }
        if state.staged.is_some() || state.publication.is_some() {
            return Err(DomStyleRuntimeError::SheetSetPublicationPending.into());
        }
        let stamp = view.stamp();
        drop(view);
        // Declaration order is custody order: every represented bank is formed
        // after this affine hold and is destroyed before its charge on failure.
        let mut hold = lease
            .begin_meta_runtime(limits, cancel)
            .map_err(DomStyleRuntimeError::from)?;
        let steps = SourceSteps {
            work: &state.work,
            maximum: state.limits.work,
            cancel,
        };
        let mut backing = Backing {
            bytes: size_of::<DomStyleMetaSourceOutcome>()
                + size_of::<Vec<Rc<DomStyleCssomRoot>>>()
                + size_of::<Vec<&DomStyleCssomRoot>>()
                + size_of::<Vec<Candidate>>()
                + size_of::<Vec<NameRow<'_>>>()
                + size_of::<Vec<sheet_sets::State>>()
                + size_of::<sheet_sets::State>()
                + size_of::<DocumentString>()
                + size_of::<DomStyleMetaPreference>(),
            units: 0,
            dom_shared_bytes: 0,
            dom_shared_units: 0,
            outputs: 0,
            current: 0,
            runtime_limit: state.limits.storage_bytes,
            limits: BackingLimits::native(limits),
        };
        steps.prefix(&hold)?;
        hold.admit_runtime(backing.native_usage()?, cancel)
            .map_err(DomStyleRuntimeError::from)?;
        // Persistent Runtime storage excludes this temporary collector, as in
        // the existing owner contract. Its actual scratch is additionally
        // prepaid by the native/Source owners before allocation or traversal.
        backing.current = source_storage(&state, &mut hold, &backing, &steps)?;
        backing.check()?;
        let mut capabilities = Vec::new();
        let mut references = Vec::new();
        backing.reserve_prefix(&mut capabilities, state.roots.len(), &mut hold, &steps)?;
        backing.reserve_prefix(&mut references, state.roots.len(), &mut hold, &steps)?;
        for root in &state.roots {
            steps.prefix(&hold)?;
            if let RootResource::Bound(owner) = &root.resource {
                capabilities.push(owner.clone());
            }
        }
        for capability in &capabilities {
            steps.prefix(&hold)?;
            references.push(capability.as_ref());
        }
        steps.prefix(&hold)?;
        let mut preparation =
            match hold
                .prepare(&references, resolver, cancel)
                .map_err(|error| {
                    if let surgeist_dom::DomMetaAtomicError::MissingOwner { root } = &error
                        && let Err(reason) = state.require_bound_if_registered(*root, cancel)
                    {
                        return DomStyleMetaError::Runtime(reason);
                    }
                    DomStyleMetaError::Meta(error)
                })? {
                surgeist_dom::DomMetaSourceStep::Prepared(preparation) => preparation,
                surgeist_dom::DomMetaSourceStep::Progress(progress) => {
                    let (source, inline) = progress.into_parts();
                    let outcome = DomStyleMetaSourceOutcome {
                        source,
                        inline,
                        roots: Vec::new(),
                        preferences: Vec::new(),
                    };
                    // This is the infallible original-output ownership frontier.
                    // Reverse declaration order disposes both external banks before
                    // the hold releases their actual Source charges.
                    return Ok(outcome);
                }
            };
        let StyleState {
            roots,
            work,
            limits: runtime_limits,
            ..
        } = &mut *state;
        let steps = SourceSteps {
            work,
            maximum: runtime_limits.work,
            cancel,
        };
        let count = preparation.occurrences().len();
        backing.outputs = count;
        backing.check()?;
        let mut candidates: Vec<Candidate> = Vec::new();
        let mut preferences: Vec<DomStyleMetaPreference> = Vec::new();
        let mut names = Vec::new();
        let mut retired = Vec::new();
        backing.reserve_source(&mut candidates, count, &mut preparation, &steps)?;
        backing.reserve_source(&mut preferences, count, &mut preparation, &steps)?;
        backing.reserve_source(&mut names, count, &mut preparation, &steps)?;
        backing.reserve_source(&mut retired, count, &mut preparation, &steps)?;
        for occurrence_index in 0..count {
            steps.joined(&preparation)?;
            let occurrence = &preparation.occurrences()[occurrence_index];
            let document = occurrence.document();
            let meta = occurrence.meta().clone();
            let content = occurrence.content().clone();
            let mut candidate_index = None;
            for (index, candidate) in candidates.iter().enumerate() {
                steps.joined(&preparation)?;
                if roots[candidate.root_index].node == document {
                    candidate_index = Some(index);
                    break;
                }
            }
            let index = if let Some(index) = candidate_index {
                index
            } else {
                let mut root_index = None;
                for (index, root) in roots.iter().enumerate() {
                    steps.joined(&preparation)?;
                    if root.node == document {
                        root_index = Some(index);
                        break;
                    }
                }
                let root_index = root_index
                    .ok_or(DomStyleRuntimeError::SheetSetRootUnavailable { root: document })?;
                let current = roots[root_index]
                    .sheet_sets
                    .state()
                    .ok_or(DomStyleRuntimeError::SheetSetDocumentRequired { root: document })?;
                let index = candidates.len();
                candidates.push(Candidate {
                    root_index,
                    root_commit_index: index,
                    next: Some(current.clone()),
                });
                index
            };
            let current = candidates[index].next.as_ref().expect("prepared candidate");
            let (next, enable) =
                current.transition(&content, sheet_sets::Action::Preferred, |left, right| {
                    steps.same(&preparation, left, right)
                })?;
            // Only successor/output names actually retained by Runtime belong
            // to its old/new union. Native recording already owns any alias in
            // the occurrence bank; exclude that backing from the shared native
            // envelope while still checking Runtime's own storage limit.
            for (name_index, name) in next.names().enumerate() {
                steps.joined(&preparation)?;
                let allocation = name.shared_code_units();
                let mut retained = false;
                for root in roots.iter() {
                    steps.joined(&preparation)?;
                    if let Some(current) = root.sheet_sets.state() {
                        for old in current.names() {
                            steps.joined(&preparation)?;
                            retained |= Arc::ptr_eq(allocation, old.shared_code_units());
                        }
                    }
                }
                for previous in &preferences {
                    steps.joined(&preparation)?;
                    for old in std::iter::once(previous.observation.preferred_name())
                        .chain(previous.observation.last_selected_name())
                    {
                        steps.joined(&preparation)?;
                        retained |= Arc::ptr_eq(allocation, old.shared_code_units());
                    }
                }
                for old in next.names().take(name_index) {
                    steps.joined(&preparation)?;
                    retained |= Arc::ptr_eq(allocation, old.shared_code_units());
                }
                if retained {
                    continue;
                }
                let mut shared_dom = false;
                for original in preparation.occurrences() {
                    steps.joined(&preparation)?;
                    shared_dom |= Arc::ptr_eq(allocation, original.content().shared_code_units());
                }
                let units = allocation.capacity();
                let bytes = units
                    .checked_mul(size_of::<u16>())
                    .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
                    .ok_or(DomStyleRuntimeError::Admission)?;
                backing.retain_name(bytes, units, shared_dom)?;
                steps.joined(&preparation)?;
                preparation
                    .admit_runtime(backing.native_usage()?, cancel)
                    .map_err(DomStyleRuntimeError::from)?;
            }
            steps.joined(&preparation)?;
            let cssom = preparation.root_snapshot(document, cancel)?;
            let producer_revision = cssom.revision().clone();
            let sheet_count = cssom.sheets().len();
            if enable {
                for sheet_index in 0..sheet_count {
                    steps.joined(&preparation)?;
                    steps.joined(&preparation)?;
                    let cssom = preparation.root_snapshot(document, cancel)?;
                    let sheet = cssom.sheets()[sheet_index].clone();
                    let title = &cssom
                        .sheet(&sheet)
                        .map_err(DomStyleRuntimeError::from)?
                        .inputs()
                        .title;
                    let disabled = sheet_sets::named_sheet_disabled(
                        title,
                        content.code_units(),
                        |left, right| steps.same(&preparation, left, right),
                    )?;
                    if let Some(disabled) = disabled {
                        // Always edit the genuine staged batch: a later B -> A
                        // occurrence must undo B even when A equals base flags.
                        steps.joined(&preparation)?;
                        preparation.set_disabled(document, &sheet, disabled, cancel)?;
                    }
                }
            }
            preferences.push(DomStyleMetaPreference {
                meta,
                observation: DomStyleSheetSetObservation::from_state(
                    stamp,
                    document,
                    &next,
                    producer_revision,
                ),
                root_commit_index: candidates[index].root_commit_index,
            });
            candidates[index].next = Some(next);
        }
        // Split only already-qualified exact destinations. No root lookup,
        // variant match or RefCell borrowing remains after producer publication.
        for (index, root) in roots.iter_mut().enumerate() {
            steps.joined(&preparation)?;
            for candidate in &mut candidates {
                steps.joined(&preparation)?;
                if candidate.root_index == index {
                    names.push(NameRow {
                        destination: root
                            .sheet_sets
                            .state_mut()
                            .expect("qualified Document destination"),
                        next: candidate.next.take(),
                    });
                    break;
                }
            }
        }
        steps.joined(&preparation)?;
        let published = preparation.commit(cancel)?;
        // No fallible service, metering, callback or allocation follows the
        // first producer publication. Names install while all guards are held.
        for row in &mut names {
            row.install(&mut retired);
        }
        let (commit, retirement) = published.into_parts();
        let stamp = commit.stamp();
        for preference in &mut preferences {
            let actual = commit.roots()[preference.root_commit_index].cssom();
            let revision = match actual.publication() {
                surgeist_cssom::CssomPublication::Changed(summary) => summary.to(),
                surgeist_cssom::CssomPublication::Unchanged { revision } => revision,
            };
            preference
                .observation
                .finish_publication(stamp, revision.clone());
        }
        let (source, roots, inline) = commit.into_parts();
        let outcome = DomStyleMetaSourceOutcome {
            source,
            roots,
            inline,
            preferences,
        };
        drop(names);
        drop(candidates);
        drop(retired);
        drop(retirement);
        drop(references);
        drop(capabilities);
        drop(hold);
        Ok(outcome)
    }
}
