//! Document stylesheet-set history beside the actual association owner.
use super::*;
use surgeist_dom::DocumentString;

pub(super) enum RootSets {
    Document(State),
    Shadow,
}
#[derive(Clone)]
pub(super) struct State {
    preferred: DocumentString,
    last_selected: Option<DocumentString>,
    revision: u64,
}
impl RootSets {
    pub(super) fn new(document: bool) -> Self {
        if document {
            Self::Document(State {
                preferred: DocumentString::default(),
                last_selected: None,
                revision: 0,
            })
        } else {
            Self::Shadow
        }
    }
    pub(super) fn state(&self) -> Option<&State> {
        match self {
            Self::Document(state) => Some(state),
            Self::Shadow => None,
        }
    }
    pub(super) fn publish(&mut self, state: State) {
        match self {
            Self::Document(current) => *current = state,
            Self::Shadow => unreachable!("qualified Document transition"),
        }
    }
    pub(super) fn backing_bytes(&self) -> Result<usize, DomStyleRuntimeError> {
        self.state()
            .map_or(Ok(0), |state| state.additional_backing(None))
    }
}
impl State {
    fn names(&self) -> impl Iterator<Item = &DocumentString> {
        std::iter::once(&self.preferred).chain(self.last_selected.iter())
    }
    // Candidate aliases of old names retain the same allocation. Charge new
    // immutable backing once within this exact Document's old/new state union.
    pub(super) fn additional_backing(
        &self,
        old: Option<&Self>,
    ) -> Result<usize, DomStyleRuntimeError> {
        let mut bytes = 0usize;
        for (index, name) in self.names().enumerate() {
            let allocation = name.shared_code_units();
            if old.is_some_and(|old| {
                old.names()
                    .any(|n| Arc::ptr_eq(n.shared_code_units(), allocation))
            }) || self
                .names()
                .take(index)
                .any(|n| Arc::ptr_eq(n.shared_code_units(), allocation))
            {
                continue;
            }
            bytes = allocation
                .capacity()
                .checked_mul(size_of::<u16>())
                .and_then(|n| n.checked_add(size_of::<Vec<u16>>()))
                .and_then(|n| n.checked_add(bytes))
                .ok_or(DomStyleRuntimeError::Admission)?;
        }
        Ok(bytes)
    }
}
impl StyleState {
    fn same_set_name(
        &self,
        left: &[u16],
        right: &[u16],
        cancel: &AtomicBool,
    ) -> Result<bool, DomStyleRuntimeError> {
        self.step(cancel)?;
        if left.len() != right.len() {
            return Ok(false);
        }
        for (a, b) in left.iter().zip(right) {
            self.step(cancel)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn enable_named_sheets(
        &self,
        snapshot: &CssomSnapshot,
        batch: &mut CssomBatch,
        name: &[u16],
        cancel: &AtomicBool,
    ) -> Result<(), DomStyleRuntimeError> {
        for id in snapshot.sheets() {
            self.step(cancel)?;
            let title = &snapshot.sheet(id)?.inputs().title;
            if title.is_empty() {
                continue;
            }
            let enabled = !name.is_empty() && self.same_set_name(title, name, cancel)?;
            batch.set_disabled(id, !enabled)?;
        }
        Ok(())
    }
    pub(super) fn prepare_added_sheet_sets(
        &self,
        index: usize,
        snapshot: &CssomSnapshot,
        batch: &mut CssomBatch,
        inputs: &mut CssomSheetInputs,
        cancel: &AtomicBool,
    ) -> Result<Option<State>, DomStyleRuntimeError> {
        let Some(current) = self.roots[index].sheet_sets.state() else {
            return Ok(None);
        };
        self.step(cancel)?;
        // The add-sheet early return preserves both preferred history and flags.
        if inputs.disabled {
            return Ok(None);
        }
        let mut candidate = None;
        if !inputs.title.is_empty() && !inputs.alternate && current.preferred.is_empty() {
            let mut next = current.clone();
            next.preferred = DocumentString::from_shared_code_units(inputs.title.clone());
            next.revision = next
                .revision
                .checked_add(1)
                .ok_or(DomStyleRuntimeError::Admission)?;
            self.admit(
                size_of::<State>()
                    .checked_add(next.additional_backing(Some(current))?)
                    .ok_or(DomStyleRuntimeError::Admission)?,
                cancel,
            )?;
            if next.last_selected.is_none() {
                self.enable_named_sheets(snapshot, batch, next.preferred.code_units(), cancel)?;
            }
            candidate = Some(next);
        }
        let selected = candidate.as_ref().unwrap_or(current);
        let enabled = inputs.title.is_empty()
            || match &selected.last_selected {
                None => {
                    self.same_set_name(&inputs.title, selected.preferred.code_units(), cancel)?
                }
                Some(name) => self.same_set_name(&inputs.title, name.code_units(), cancel)?,
            };
        inputs.disabled = !enabled;
        Ok(candidate)
    }
}

/// Current Document preference history joined to its actual stylesheet owner.
/// Immutable returned names are caller-owned observations, not retained history
/// in Runtime's current-bank storage budget.
#[derive(Clone, Debug)]
pub struct DomStyleSheetSetObservation {
    stamp: surgeist_dom::DomStamp,
    document: NodeId,
    revision: u64,
    cssom_revision: surgeist_cssom::CssomRevision,
    preferred: DocumentString,
    last_selected: Option<DocumentString>,
}
impl DomStyleSheetSetObservation {
    /// Canonical basis actually qualified.
    pub fn stamp(&self) -> surgeist_dom::DomStamp {
        self.stamp
    }
    /// Exact Document root generation; a ShadowRoot never grants this state.
    pub fn document(&self) -> NodeId {
        self.document
    }
    /// Association-owned preference revision, independent of sheet ID issuance.
    pub fn revision(&self) -> u64 {
        self.revision
    }
    /// Actual producer owner/revision inspected with this preference state.
    pub fn cssom_revision(&self) -> &surgeist_cssom::CssomRevision {
        &self.cssom_revision
    }
    /// Exact preferred UTF-16 name; initially empty.
    pub fn preferred_name(&self) -> &DocumentString {
        &self.preferred
    }
    /// Last explicitly selected name. None differs from Some(empty).
    pub fn last_selected_name(&self) -> Option<&DocumentString> {
        self.last_selected.as_ref()
    }
}
/// One actual CSSOM commit and the preference state published with it.
/// Producer-private receipt backing retains CSSOM's independent accounting.
pub struct DomStyleSheetSetCommit {
    cssom: CssomCommit,
    observation: DomStyleSheetSetObservation,
}
impl DomStyleSheetSetCommit {
    /// Actual producer commit, including an authentic Unchanged publication.
    pub fn cssom(&self) -> &CssomCommit {
        &self.cssom
    }
    /// Preference state at this publication, with its actual producer revision.
    pub fn observation(&self) -> &DomStyleSheetSetObservation {
        &self.observation
    }
    /// Transfer both actual outputs once.
    pub fn into_parts(self) -> (CssomCommit, DomStyleSheetSetObservation) {
        (self.cssom, self.observation)
    }
}
#[derive(Clone, Copy)]
enum Action {
    Enable,
    Select,
    Preferred,
}
struct Prepared {
    document: NodeId,
    stamp: surgeist_dom::DomStamp,
    revision: u64,
    owner: std::rc::Rc<DomStyleCssomRoot>,
    next: State,
    batch: Option<CssomBatch>,
}
impl StyleState {
    fn sheet_set_root(
        &self,
        snapshot: &DomSnapshot,
        document: NodeId,
        cancel: &AtomicBool,
    ) -> Result<usize, DomStyleRuntimeError> {
        self.step(cancel)?;
        if !matches!(
            snapshot.node(document)?.kind(),
            surgeist_dom::DomNodeKind::Document(_)
        ) {
            return Err(DomStyleRuntimeError::SheetSetDocumentRequired { root: document });
        }
        for (index, root) in self.roots.iter().enumerate() {
            self.step(cancel)?;
            if root.node == document {
                return Ok(index);
            }
        }
        Err(DomStyleRuntimeError::SheetSetRootUnavailable { root: document })
    }
    fn require_sheet_set_idle(
        &self,
        dom: &DomEventDom<'_>,
    ) -> Result<DomSnapshot, DomStyleRuntimeError> {
        if let Some(status) = dom.pending_source_edit() {
            return Err(DomStyleRuntimeError::SourcePending(status.operation()));
        }
        if self.staged.is_some() || self.publication.is_some() {
            return Err(DomStyleRuntimeError::SheetSetPublicationPending);
        }
        let snapshot = dom.snapshot();
        if snapshot.incomplete_source_operation().is_some()
            || snapshot.incomplete_hosted_operation().is_some()
        {
            return Err(DomError::IncompleteSourceOperation.into());
        }
        Ok(snapshot)
    }
    fn observe_sheet_sets(
        &mut self,
        dom: &DomEventDom<'_>,
        document: NodeId,
        cancel: &AtomicBool,
    ) -> Result<DomStyleSheetSetObservation, DomStyleRuntimeError> {
        self.step(cancel)?;
        let snapshot = self.require_sheet_set_idle(dom)?;
        self.prune_retired_roots(&snapshot, cancel)?;
        let index = self.sheet_set_root(&snapshot, document, cancel)?;
        self.admit(size_of::<DomStyleSheetSetObservation>(), cancel)?;
        let root = &self.roots[index];
        let current = root.store.snapshot()?;
        let state = root.sheet_sets.state().expect("qualified Document root");
        Ok(DomStyleSheetSetObservation {
            stamp: snapshot.stamp(),
            document,
            revision: state.revision,
            cssom_revision: current.revision().clone(),
            preferred: state.preferred.clone(),
            last_selected: state.last_selected.clone(),
        })
    }
    fn prepare_sheet_set_change(
        &mut self,
        dom: &DomEventDom<'_>,
        basis: &DomStyleSheetSetObservation,
        name: &DocumentString,
        action: Action,
        max_output: usize,
        cancel: &AtomicBool,
    ) -> Result<Prepared, DomStyleRuntimeError> {
        self.step(cancel)?;
        let snapshot = self.require_sheet_set_idle(dom)?;
        self.prune_retired_roots(&snapshot, cancel)?;
        let index = self.sheet_set_root(&snapshot, basis.document, cancel)?;
        let root = &self.roots[index];
        let current = root.store.snapshot()?;
        let state = root.sheet_sets.state().expect("qualified Document root");
        if snapshot.stamp() != basis.stamp
            || state.revision != basis.revision
            || current.revision() != &basis.cssom_revision
        {
            return Err(DomStyleRuntimeError::StaleSheetSetObservation);
        }
        if max_output == 0 {
            return Err(DomStyleRuntimeError::SheetSetOutputLimit);
        }
        let mut next = state.clone();
        let enable = match action {
            Action::Enable => true,
            Action::Select => {
                let same = match &state.last_selected {
                    Some(old) => self.same_set_name(old.code_units(), name.code_units(), cancel)?,
                    None => false,
                };
                if !same {
                    next.last_selected = Some(name.clone());
                    next.revision = next
                        .revision
                        .checked_add(1)
                        .ok_or(DomStyleRuntimeError::Admission)?;
                }
                true
            }
            Action::Preferred => {
                let changed =
                    !self.same_set_name(state.preferred.code_units(), name.code_units(), cancel)?;
                if changed {
                    next.preferred = name.clone();
                    next.revision = next
                        .revision
                        .checked_add(1)
                        .ok_or(DomStyleRuntimeError::Admission)?;
                }
                changed && state.last_selected.is_none()
            }
        };
        // Current names remain live during this new state and result wrapper.
        // Shared immutable names are aliases, not a second payload allocation.
        let backing = next.additional_backing(Some(state))?;
        self.admit(
            size_of::<Prepared>()
                .checked_add(size_of::<DomStyleSheetSetCommit>())
                .and_then(|n| n.checked_add(backing))
                .ok_or(DomStyleRuntimeError::Admission)?,
            cancel,
        )?;
        let mut batch = root.store.batch(CssomEditGuard::from_snapshot(&current))?;
        if enable {
            self.enable_named_sheets(&current, &mut batch, name.code_units(), cancel)?;
        }
        self.step(cancel)?;
        Ok(Prepared {
            document: basis.document,
            stamp: snapshot.stamp(),
            revision: state.revision,
            owner: root.store.clone(),
            next,
            batch: Some(batch),
        })
    }
    fn commit_sheet_set_change(
        &mut self,
        snapshot: &DomSnapshot,
        mut prepared: Prepared,
        cancel: &AtomicBool,
    ) -> Result<DomStyleSheetSetCommit, DomStyleRuntimeError> {
        let index = self.sheet_set_root(snapshot, prepared.document, cancel)?;
        let root = &self.roots[index];
        if snapshot.stamp() != prepared.stamp
            || !std::rc::Rc::ptr_eq(&root.store, &prepared.owner)
            || root.sheet_sets.state().expect("Document").revision != prepared.revision
        {
            return Err(DomStyleRuntimeError::StaleSheetSetObservation);
        }
        self.step(cancel)?;
        let preferred = prepared.next.preferred.clone();
        let last_selected = prepared.next.last_selected.clone();
        let revision = prepared.next.revision;
        // Final fallible producer step. The old Root is untouched on refusal;
        // all names, wrappers, output and work have already been admitted.
        let cssom = prepared.owner.commit(&mut prepared.batch)?;
        let cssom_revision = match cssom.publication() {
            surgeist_cssom::CssomPublication::Changed(summary) => summary.to(),
            surgeist_cssom::CssomPublication::Unchanged { revision } => revision,
        }
        .clone();
        self.roots[index].sheet_sets.publish(prepared.next);
        Ok(DomStyleSheetSetCommit {
            cssom,
            observation: DomStyleSheetSetObservation {
                stamp: prepared.stamp,
                document: prepared.document,
                revision,
                cssom_revision,
                preferred,
                last_selected,
            },
        })
    }
}
impl DomStyleRuntime {
    /// Read genuine current names from an actual bound Document owner, including
    /// an empty list. Standalone preference fronts require idle Source custody.
    pub fn sheet_sets(
        &self,
        dom: &DomEventDom<'_>,
        document: NodeId,
        cancel: &AtomicBool,
    ) -> Result<DomStyleSheetSetObservation, DomStyleRuntimeError> {
        let owner = self.qualified(dom)?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        state.observe_sheet_sets(dom, document, cancel)
    }
    /// Enable exactly the case-sensitive named set, leaving preferred and last
    /// names unchanged. Empty disables named sheets; unnamed sheets are untouched.
    pub fn enable_sheet_set(
        &mut self,
        dom: &DomEventDom<'_>,
        basis: &DomStyleSheetSetObservation,
        name: &DocumentString,
        max_output: usize,
        cancel: &AtomicBool,
    ) -> Result<DomStyleSheetSetCommit, DomStyleRuntimeError> {
        self.mutate_sheet_sets(dom, basis, name, Action::Enable, max_output, cancel)
    }
    /// Enable the named set and remember Some(name), including an empty name.
    pub fn select_sheet_set(
        &mut self,
        dom: &DomEventDom<'_>,
        basis: &DomStyleSheetSetObservation,
        name: &DocumentString,
        max_output: usize,
        cancel: &AtomicBool,
    ) -> Result<DomStyleSheetSetCommit, DomStyleRuntimeError> {
        self.mutate_sheet_sets(dom, basis, name, Action::Select, max_output, cancel)
    }
    /// Store the exact preferred name even with no sheets. Enable it only when
    /// the name changed and no explicit selection has been made. A stale basis
    /// or actual pending Source/publication refuses before producer publication.
    pub fn change_preferred_sheet_set_name(
        &mut self,
        dom: &DomEventDom<'_>,
        basis: &DomStyleSheetSetObservation,
        name: &DocumentString,
        max_output: usize,
        cancel: &AtomicBool,
    ) -> Result<DomStyleSheetSetCommit, DomStyleRuntimeError> {
        self.mutate_sheet_sets(dom, basis, name, Action::Preferred, max_output, cancel)
    }
    fn mutate_sheet_sets(
        &mut self,
        dom: &DomEventDom<'_>,
        basis: &DomStyleSheetSetObservation,
        name: &DocumentString,
        action: Action,
        max_output: usize,
        cancel: &AtomicBool,
    ) -> Result<DomStyleSheetSetCommit, DomStyleRuntimeError> {
        let owner = self.qualified(dom)?;
        let mut state = owner
            .try_borrow_mut()
            .map_err(|_| DomStyleRuntimeError::Binding)?;
        let prepared =
            state.prepare_sheet_set_change(dom, basis, name, action, max_output, cancel)?;
        state.commit_sheet_set_change(&dom.snapshot(), prepared, cancel)
    }
}
