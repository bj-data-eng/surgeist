//! Ordinary native Meta composition on the existing association owner.
use super::*;
use std::cell::Cell;
use std::rc::Rc;
use surgeist_dom::{
    DocumentString, DomBatch, DomMetaAtomicLimits, DomMetaInlineBatch, DomMetaOrdinaryCommit,
    DomMetaOrdinaryPreparation, DomMetaRuntimePrefix, DomMetaRuntimeUsage,
};

/// Ordinary native Meta preparation failed before any owner published.
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
pub struct DomStyleMetaPreference {
    meta: surgeist_dom::DomMetaSubject,
    observation: DomStyleSheetSetObservation,
    root_commit_index: usize,
}
impl DomStyleMetaPreference {
    /// Exact existing generation or opaque original batch-qualified subject.
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
    limits: DomMetaAtomicLimits,
}
impl Backing {
    fn check(&self) -> Result<(), DomStyleRuntimeError> {
        for (actual, limit, resource) in [
            (
                self.bytes - self.dom_shared_bytes,
                self.limits.retained_bytes(),
                surgeist_dom::DomResource::MetaAtomicRetainedBytes,
            ),
            (
                self.units - self.dom_shared_units,
                self.limits.retained_units(),
                surgeist_dom::DomResource::MetaAtomicRetainedUnits,
            ),
            (
                self.outputs,
                self.limits.outputs(),
                surgeist_dom::DomResource::MetaAtomicOutputs,
            ),
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
        preparation.admit_runtime(self.usage()?, steps.cancel)?;
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
        preparation.admit_runtime(self.usage()?, steps.cancel)?;
        Ok(())
    }
    fn usage(&self) -> Result<DomMetaRuntimeUsage, DomStyleRuntimeError> {
        Ok(DomMetaRuntimeUsage::new(
            self.bytes - self.dom_shared_bytes,
            self.units - self.dom_shared_units,
            self.outputs,
        )?)
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
            limits,
        };
        backing.check()?;
        let mut capabilities = Vec::new();
        let mut references = Vec::new();
        backing.reserve(&mut capabilities, state.roots.len())?;
        backing.reserve(&mut references, state.roots.len())?;
        for root in &state.roots {
            StyleState::step_counter(&state.work, cancel, prefix_maximum)?;
            capabilities.push(root.store.clone());
        }
        for capability in &capabilities {
            StyleState::step_counter(&state.work, cancel, prefix_maximum)?;
            references.push(capability.as_ref());
        }
        StyleState::step_counter(&state.work, cancel, prefix_maximum)?;
        let formation_work = state.work.get() - initial_work;
        // Both banks are outside every DOM loan and remain stable until all
        // returned producer guards have been extracted and released.
        let prefix = DomMetaRuntimePrefix::new(backing.usage()?, formation_work, limits)
            .map_err(DomStyleRuntimeError::from)?;
        let mut preparation =
            dom.raw_mut()
                .prepare_meta_batch(batch, &references, inline, limits, prefix, cancel)?;
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
                    .admit_runtime(backing.usage()?, cancel)
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
}
