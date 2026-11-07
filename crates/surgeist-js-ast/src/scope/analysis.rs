//! Sparse analysis overlay. Published scopes are changed only after admission ends.
use super::handle::{AssociationState, ScopeAssociation};
use super::*;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

/// One explicitly admitted unit of scope-analysis work.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeWork {
    /// Visit one syntax node.
    Node,
    /// Probe the sparse scope table.
    ScopeProbe,
    /// Follow one enclosing scope edge.
    ScopeHop,
    /// Inspect one binding candidate.
    BindingProbe,
    /// Compare or prepare one UTF-16 code unit.
    NameUnit,
    /// Begin preparing an identifier name.
    NamePreparation,
    /// Write one binding record.
    BindingWrite,
    /// Write scope metadata or allocate one scope identity.
    ScopeWrite,
    /// Record one sparse change.
    Record,
    /// Prepare capacity, including copying up to this many existing elements.
    Capacity { elements: usize },
    /// Validate the tracked scope-tree revisions.
    ContextValidation { contexts: usize },
    /// Publish one sparse record or identity counter.
    Publication,
}

/// A refusal or failure of controlled scope analysis.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ScopeAnalysisError<E> {
    /// The program violates a scope-analysis syntax requirement.
    Syntax(std::borrow::Cow<'static, str>),
    /// Caller admission refused work; its original reason is retained.
    Refused(E),
    /// Scope storage or identity state prevents publication.
    State(ScopeStateError),
}

impl<E: std::fmt::Display> std::fmt::Display for ScopeAnalysisError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Syntax(message) => f.write_str(message),
            Self::Refused(reason) => write!(f, "scope analysis refused: {reason}"),
            Self::State(state) => std::fmt::Display::fmt(state, f),
        }
    }
}
impl<E: std::error::Error + 'static> std::error::Error for ScopeAnalysisError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Refused(reason) => Some(reason),
            Self::State(state) => Some(state),
            Self::Syntax(_) => None,
        }
    }
}

/// Scope storage and identity failures independent of caller admission.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScopeStateError {
    /// A caller changed a relevant scope tree during analysis.
    Conflict,
    /// No further scope or binding identity can be represented.
    IdentityExhausted,
    /// Storage for an admitted operation could not be reserved.
    Allocation,
    /// Scope analysis has not completed for this syntax.
    Unprepared,
    /// A handle belongs to an aborted or different preparation.
    AssociationMismatch,
}

impl std::fmt::Display for ScopeStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Conflict => "scope state changed during analysis",
            Self::IdentityExhausted => "scope identity space exhausted",
            Self::Allocation => "scope analysis storage allocation failed",
            Self::Unprepared => "scope analysis is not prepared",
            Self::AssociationMismatch => "scope preparation association does not match",
        })
    }
}
impl std::error::Error for ScopeStateError {}

/// Optional caller admission for controlled scope work.
pub type ScopeAdmission<'a, E> = Option<&'a mut dyn FnMut(ScopeWork) -> Result<(), E>>;

#[derive(Debug)]
struct ContextEdit {
    context: Rc<ScopeContext>,
    revision: Option<u64>,
    next_id: u32,
}

#[derive(Debug)]
struct ScopeEdit {
    scope: Scope,
    index: Option<u32>,
    this_escaped: Option<bool>,
    bindings: Vec<(usize, Binding)>,
    binding_slots: HashMap<usize, usize>,
    appended: Vec<Binding>,
    projected: bool,
}

/// Durable private scope changes, independent of admission and engine state.
#[derive(Debug)]
pub struct PreparedScopes {
    pub(super) staged: bool,
    pub(super) root: Option<Scope>,
    pub(super) owner: Option<Rc<ScopeAssociation>>,
    contexts: Vec<ContextEdit>,
    context_slots: HashMap<usize, usize>,
    edits: Vec<ScopeEdit>,
    edit_slots: HashMap<usize, usize>,
    pub(super) handles: Vec<ScopeHandle>,
    extension: Option<ExtensionCheckpoint>,
    undo: Vec<ScopeUndo>,
    pub(super) project_declarations: bool,
}

#[derive(Debug)]
struct ExtensionCheckpoint {
    contexts: usize,
    edits: usize,
    handles: usize,
}

#[derive(Debug)]
enum ScopeUndo {
    NextId(usize, u32),
    Index(usize, Option<u32>),
    ThisEscaped(usize, Option<bool>),
    Binding(usize, usize, Binding),
    Appended(usize, usize, Binding),
    PopBinding(usize),
    PopAppended(usize),
}

impl PreparedScopes {
    fn empty() -> Self {
        Self {
            staged: false,
            root: None,
            owner: None,
            contexts: Vec::new(),
            context_slots: HashMap::new(),
            edits: Vec::new(),
            edit_slots: HashMap::new(),
            handles: Vec::new(),
            extension: None,
            undo: Vec::new(),
            project_declarations: false,
        }
    }

    /// Starts private staging over an existing live root.
    #[must_use]
    pub fn new(root: &Scope) -> Self {
        let mut store = Self::empty();
        let context = root.inner.context.clone();
        store.staged = true;
        store.root = Some(root.clone());
        store.owner = Some(ScopeAssociation::pending());
        store.context_slots.insert(Rc::as_ptr(&context) as usize, 0);
        store.contexts.push(ContextEdit {
            revision: context.revision.get(),
            next_id: context.unique_id_ctr.get(),
            context,
        });
        store
    }

    pub(crate) fn for_script(root: &Scope) -> Self {
        let mut store = Self::new(root);
        store.project_declarations = true;
        store
    }

    pub(crate) fn invalidate(&self) {
        if let Some(owner) = &self.owner
            && owner.state.get() == AssociationState::Pending
        {
            owner.state.set(AssociationState::Aborted);
        }
    }

    pub(crate) fn live(root: &Scope) -> Self {
        let mut store = Self::empty();
        store.root = Some(root.clone());
        store
    }

    fn validate_raw(&self) -> Result<(), ScopeStateError> {
        let mut conflict = false;
        for context in &self.contexts {
            conflict |=
                context.revision.is_none() || context.context.revision.get() != context.revision;
        }
        if conflict {
            Err(ScopeStateError::Conflict)
        } else {
            Ok(())
        }
    }

    /// Gets the root with this preparation's association, including an empty script.
    #[must_use]
    pub fn root_handle(&self) -> ScopeHandle {
        let Some(root) = &self.root else {
            return ScopeHandle::default();
        };
        self.bind(root.clone(), None)
    }

    pub(super) fn bind(&self, scope: Scope, outer: Option<ScopeHandle>) -> ScopeHandle {
        match &self.owner {
            Some(owner) => ScopeHandle::pending(scope, owner.clone(), outer),
            None => ScopeHandle::live(scope),
        }
    }
}

impl Drop for PreparedScopes {
    fn drop(&mut self) {
        if let Some(owner) = &self.owner
            && owner.state.get() == AssociationState::Pending
        {
            owner.state.set(AssociationState::Aborted);
        }
    }
}

/// Freshness of a pending preparation at the beginning of a new attempt.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BasisStatus {
    /// The observed scope trees still match.
    Current,
    /// A previous operation changed the observed scope trees.
    Stale,
}

impl PreparedScopes {
    /// Borrows the existing scope algorithms under a temporary policy.
    ///
    /// # Errors
    /// Returns the operation's error without changing its error domain.
    pub fn with_access<T, E, F>(
        &mut self,
        admission: ScopeAdmission<'_, E>,
        operation: impl FnOnce(&mut super::ScopeAccess<'_, E>) -> Result<T, F>,
    ) -> Result<T, F>
    where
        F: From<ScopeAnalysisError<E>>,
    {
        if let Some(owner) = &self.owner
            && owner.state.get() != AssociationState::Pending
        {
            return Err(F::from(ScopeAnalysisError::State(
                ScopeStateError::AssociationMismatch,
            )));
        }
        match admission {
            Some(callback) => {
                // This forwarding closure shortens the policy borrow independently
                // of the retained store. No callback enters the returned artifact.
                let mut forward = |work| callback(work);
                let mut access = super::ScopeAccess::borrowed(self, Some(&mut forward));
                operation(&mut access)
            }
            None => {
                let mut access = super::ScopeAccess::borrowed(self, None);
                operation(&mut access)
            }
        }
    }

    /// Validates a pending basis without rejecting a pre-existing stale snapshot.
    ///
    /// # Errors
    /// Returns refusal or a callback-induced conflict.
    pub fn validate_basis<E>(
        &self,
        mut admission: ScopeAdmission<'_, E>,
    ) -> Result<BasisStatus, ScopeAnalysisError<E>> {
        if admission.is_none() {
            return Ok(if self.validate_raw().is_ok() {
                BasisStatus::Current
            } else {
                BasisStatus::Stale
            });
        }
        // The first admission establishes this new attempt's observation boundary.
        // Changes made by that callback make the old basis stale. No published
        // state is touched; a second admission is checked against the observation.
        if let Some(callback) = &mut admission {
            callback(ScopeWork::Capacity {
                elements: self.contexts.len(),
            })
            .map_err(ScopeAnalysisError::Refused)?;
            callback(ScopeWork::ContextValidation {
                contexts: self.contexts.len(),
            })
            .map_err(ScopeAnalysisError::Refused)?;
        }
        let mut before = Vec::new();
        before
            .try_reserve(self.contexts.len())
            .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
        for item in &self.contexts {
            before.push(item.context.revision.get());
        }
        if let Some(callback) = &mut admission {
            callback(ScopeWork::ContextValidation {
                contexts: self.contexts.len(),
            })
            .map_err(ScopeAnalysisError::Refused)?;
        }
        let mut stale = false;
        let mut conflict = false;
        for (context, revision) in self.contexts.iter().zip(before) {
            conflict |= context.context.revision.get() != revision;
            stale |= revision.is_none() || revision != context.revision;
        }
        if conflict {
            return Err(ScopeAnalysisError::State(ScopeStateError::Conflict));
        }
        Ok(if stale {
            BasisStatus::Stale
        } else {
            BasisStatus::Current
        })
    }

    /// Begins a private compiler extension, retaining no admission callback.
    ///
    /// # Errors
    /// Returns refusal, an unsuitable preparation, or an active-extension error.
    pub fn begin_extension<E>(
        &mut self,
        admission: ScopeAdmission<'_, E>,
    ) -> Result<ScopeExtension<'_>, ScopeAnalysisError<E>> {
        if self.extension.is_some() || !self.staged {
            return Err(ScopeAnalysisError::State(ScopeStateError::Unprepared));
        }
        self.with_access(admission, |access| access.admit(ScopeWork::Record))?;
        self.extension = Some(ExtensionCheckpoint {
            contexts: self.contexts.len(),
            edits: self.edits.len(),
            handles: self.handles.len(),
        });
        Ok(ScopeExtension {
            scopes: self,
            committed: false,
        })
    }

    fn rollback_extension(&mut self) {
        let Some(checkpoint) = self.extension.take() else {
            return;
        };
        while let Some(undo) = self.undo.pop() {
            match undo {
                ScopeUndo::NextId(slot, old) => self.contexts[slot].next_id = old,
                ScopeUndo::Index(slot, old) => self.edits[slot].index = old,
                ScopeUndo::ThisEscaped(slot, old) => self.edits[slot].this_escaped = old,
                ScopeUndo::Binding(slot, position, old) => {
                    self.edits[slot].bindings[position].1 = old
                }
                ScopeUndo::Appended(slot, position, old) => {
                    self.edits[slot].appended[position] = old
                }
                ScopeUndo::PopBinding(slot) => {
                    if let Some((position, _)) = self.edits[slot].bindings.pop() {
                        self.edits[slot].binding_slots.remove(&position);
                    }
                }
                ScopeUndo::PopAppended(slot) => {
                    self.edits[slot].appended.pop();
                }
            }
        }
        for handle in self.handles.drain(checkpoint.handles..) {
            handle.invalidate();
        }
        for edit in self.edits.drain(checkpoint.edits..) {
            self.edit_slots
                .remove(&(Rc::as_ptr(&edit.scope.inner) as usize));
        }
        for context in self.contexts.drain(checkpoint.contexts..) {
            self.context_slots
                .remove(&(Rc::as_ptr(&context.context) as usize));
        }
    }

    /// Admits every publication operation and freezes the preparation.
    ///
    /// # Errors
    /// Returns refusal, conflicting state, or allocation failure.
    pub fn prepare_publication<E>(
        &mut self,
        admission: ScopeAdmission<'_, E>,
    ) -> Result<PublicationPermit<'_>, ScopeAnalysisError<E>> {
        if self.extension.is_some() {
            return Err(ScopeAnalysisError::State(ScopeStateError::Unprepared));
        }
        self.with_access(admission, |access| access.session.prepare_publication())?;
        Ok(PublicationPermit { scopes: self })
    }

    fn commit_store(&mut self) -> Result<InstalledScopes, ScopeStateError> {
        self.validate_raw()?;
        let root = self.root.clone().ok_or(ScopeStateError::Unprepared)?;
        for edit in std::mem::take(&mut self.edits) {
            if let Some(index) = edit.index {
                edit.scope.inner.index.set(index);
            }
            if let Some(escaped) = edit.this_escaped {
                edit.scope.inner.this_escaped.set(escaped);
            }
            let mut bindings = edit.scope.inner.bindings.borrow_mut();
            for (position, binding) in edit.bindings {
                bindings[position] = binding;
            }
            bindings.extend(edit.appended);
            edit.scope.inner.context.changed();
        }
        for context in std::mem::take(&mut self.contexts) {
            if context.next_id != context.context.unique_id_ctr.get() {
                context.context.unique_id_ctr.set(context.next_id);
                context.context.changed();
            }
        }
        if let Some(owner) = &self.owner {
            owner.state.set(AssociationState::Installed);
        }
        Ok(InstalledScopes {
            association: self.owner.clone(),
            root,
        })
    }
}

/// A compiler's private savepoint over the same prepared scope store.
pub struct ScopeExtension<'a> {
    scopes: &'a mut PreparedScopes,
    committed: bool,
}

impl ScopeExtension<'_> {
    /// Borrows this extension under a temporary policy.
    ///
    /// # Errors
    /// Returns the operation's original error.
    pub fn with_access<T, E, F>(
        &mut self,
        admission: ScopeAdmission<'_, E>,
        operation: impl FnOnce(&mut super::ScopeAccess<'_, E>) -> Result<T, F>,
    ) -> Result<T, F>
    where
        F: From<ScopeAnalysisError<E>>,
    {
        self.scopes.with_access(admission, operation)
    }

    /// Retains successful private additions without publishing them.
    pub fn commit_private(mut self) {
        self.scopes.undo.clear();
        self.scopes.extension = None;
        self.committed = true;
    }
}

impl Drop for ScopeExtension<'_> {
    fn drop(&mut self) {
        if !self.committed {
            self.scopes.rollback_extension();
        }
    }
}

/// Already-admitted scope publication, containing no policy callback.
pub struct PublicationPermit<'a> {
    scopes: &'a mut PreparedScopes,
}

impl PublicationPermit<'_> {
    /// Validates a pending handle's exact owner before publication.
    ///
    /// This check does not call admission, inspect scope records or expose the
    /// pending scope. Live, foreign, aborted and unprepared handles are rejected.
    ///
    /// # Errors
    /// Returns a scope-state error when the handle is not valid for this permit.
    pub fn authorize_handle(&self, handle: &ScopeHandle) -> Result<(), ScopeStateError> {
        let owner = self
            .scopes
            .owner
            .as_ref()
            .ok_or(ScopeStateError::Unprepared)?;
        if owner.state.get() != AssociationState::Pending || !handle.associated_with(Some(owner)) {
            return Err(ScopeStateError::AssociationMismatch);
        }
        handle.scope_for(Some(owner)).map(|_| ())
    }

    /// Validates and publishes the canonical state without calling admission.
    ///
    /// # Errors
    /// Returns a conflicting basis without publishing anything.
    pub fn commit(self) -> Result<InstalledScopes, ScopeStateError> {
        self.scopes.commit_store()
    }
}

/// A short-lived admission policy borrowing the sole owning scope store.
pub(crate) struct ScopeSession<'a, E> {
    admission: ScopeAdmission<'a, E>,
    store: &'a mut PreparedScopes,
}

impl<E> Deref for ScopeSession<'_, E> {
    type Target = PreparedScopes;
    fn deref(&self) -> &Self::Target {
        self.store
    }
}

impl<E> DerefMut for ScopeSession<'_, E> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.store
    }
}

impl<'a, E> ScopeSession<'a, E> {
    pub(super) fn borrowed(
        store: &'a mut PreparedScopes,
        admission: ScopeAdmission<'a, E>,
    ) -> Self {
        Self { admission, store }
    }

    fn record_undo(&mut self, undo: ScopeUndo) -> Result<(), ScopeAnalysisError<E>> {
        if self.extension.is_none() {
            return Ok(());
        }
        self.admit(ScopeWork::Record)?;
        self.admit(ScopeWork::Capacity {
            elements: self.undo.len(),
        })?;
        self.undo
            .try_reserve(1)
            .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
        self.undo.push(undo);
        Ok(())
    }

    pub(super) fn reserve_handle(&mut self) -> Result<(), ScopeAnalysisError<E>> {
        if self.staged {
            self.admit(ScopeWork::Record)?;
            self.admit(ScopeWork::Capacity {
                elements: self.handles.len(),
            })?;
            self.handles
                .try_reserve(1)
                .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
        }
        Ok(())
    }

    pub(crate) fn has_admission(&self) -> bool {
        self.admission.is_some()
    }

    fn validate(&self) -> Result<(), ScopeAnalysisError<E>> {
        self.validate_raw().map_err(ScopeAnalysisError::State)
    }

    pub(crate) fn admit(&mut self, work: ScopeWork) -> Result<(), ScopeAnalysisError<E>> {
        if !self.staged {
            return self.admission.as_deref_mut().map_or(Ok(()), |callback| {
                callback(work).map_err(ScopeAnalysisError::Refused)
            });
        }
        // There is never a RefCell borrow across either caller invocation.
        let contexts = self.contexts.len().saturating_mul(2);
        if let Some(callback) = &mut self.admission {
            callback(ScopeWork::ContextValidation { contexts })
                .map_err(ScopeAnalysisError::Refused)?;
        }
        let before = self.validate();
        if before.is_err() {
            // Both full scans were admitted, even though work itself cannot run.
            let _ = self.validate();
            return before;
        }
        let admitted = if let Some(callback) = &mut self.admission {
            callback(work).map_err(ScopeAnalysisError::Refused)
        } else {
            Ok(())
        };
        // Complete the admitted scan even after refusal. The caller's original
        // refusal still takes precedence over any mutation it performed.
        let after = self.validate();
        admitted?;
        after
    }

    fn track(&mut self, scope: &Scope) -> Result<(), ScopeAnalysisError<E>> {
        if !self.staged {
            return Ok(());
        }
        let key = Rc::as_ptr(&scope.inner.context) as usize;
        if self.context_slots.contains_key(&key) {
            return Ok(());
        }
        // Capture before calling admission so callbacks cannot silently change a newly seen tree.
        let revision = scope.inner.context.revision.get();
        let next_id = scope.inner.context.unique_id_ctr.get();
        self.admit(ScopeWork::Capacity {
            elements: self.contexts.len(),
        })?;
        if scope.inner.context.revision.get() != revision || revision.is_none() {
            return Err(ScopeAnalysisError::State(ScopeStateError::Conflict));
        }
        self.contexts
            .try_reserve(1)
            .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
        self.context_slots
            .try_reserve(1)
            .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
        let slot = self.contexts.len();
        self.context_slots.insert(key, slot);
        self.contexts.push(ContextEdit {
            context: scope.inner.context.clone(),
            revision,
            next_id,
        });
        Ok(())
    }

    fn slot(&mut self, scope: &Scope) -> Result<Option<usize>, ScopeAnalysisError<E>> {
        self.track(scope)?;
        self.admit(ScopeWork::ScopeProbe)?;
        Ok(self
            .edit_slots
            .get(&(Rc::as_ptr(&scope.inner) as usize))
            .copied())
    }

    fn edit(&mut self, scope: &Scope) -> Result<usize, ScopeAnalysisError<E>> {
        if let Some(slot) = self.slot(scope)? {
            return Ok(slot);
        }
        self.admit(ScopeWork::Record)?;
        self.admit(ScopeWork::Capacity {
            elements: self.edits.len(),
        })?;
        self.edits
            .try_reserve(1)
            .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
        self.edit_slots
            .try_reserve(1)
            .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
        let slot = self.edits.len();
        self.edit_slots
            .insert(Rc::as_ptr(&scope.inner) as usize, slot);
        self.edits.push(ScopeEdit {
            scope: scope.clone(),
            index: None,
            this_escaped: None,
            bindings: Vec::new(),
            binding_slots: HashMap::new(),
            appended: Vec::new(),
            projected: false,
        });
        Ok(slot)
    }

    fn binding_count(&mut self, scope: &Scope) -> Result<usize, ScopeAnalysisError<E>> {
        let slot = self.slot(scope)?;
        Ok(scope.inner.bindings.borrow().len() + slot.map_or(0, |s| self.edits[s].appended.len()))
    }

    fn binding(
        &mut self,
        scope: &Scope,
        position: usize,
    ) -> Result<Binding, ScopeAnalysisError<E>> {
        let slot = self.slot(scope)?;
        self.admit(ScopeWork::BindingProbe)?;
        let published = scope.inner.bindings.borrow();
        if let Some(slot) = slot {
            let edit = &self.edits[slot];
            if position >= published.len() {
                return Ok(edit.appended[position - published.len()].clone());
            }
            if let Some(record) = edit.binding_slots.get(&position) {
                return Ok(edit.bindings[*record].1.clone());
            }
        }
        Ok(published[position].clone())
    }

    fn names_equal(&mut self, a: &JsString, b: &JsString) -> Result<bool, ScopeAnalysisError<E>> {
        if !self.has_admission() {
            return Ok(a == b);
        }
        if a.len() != b.len() {
            return Ok(false);
        }
        for (a, b) in a.iter().zip(b.iter()) {
            self.admit(ScopeWork::NameUnit)?;
            if a != b {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(super) fn find(
        &mut self,
        scope: &Scope,
        name: &JsString,
    ) -> Result<Option<(usize, Binding)>, ScopeAnalysisError<E>> {
        let len = self.binding_count(scope)?;
        let published = scope.inner.bindings.borrow().len();
        let projected = self
            .slot(scope)?
            .is_some_and(|slot| self.edits[slot].projected);
        // Projected source declarations shadow the live basis without reusing its slots.
        let ranges = if projected {
            [published..len, 0..published]
        } else {
            [0..len, 0..0]
        };
        for position in ranges.into_iter().flatten() {
            let binding = self.binding(scope, position)?;
            if self.names_equal(&binding.name, name)? {
                return Ok(Some((position, binding)));
            }
        }
        Ok(None)
    }

    fn write_binding(
        &mut self,
        scope: &Scope,
        position: usize,
        binding: Binding,
    ) -> Result<(), ScopeAnalysisError<E>> {
        self.admit(ScopeWork::BindingWrite)?;
        if !self.staged {
            scope.inner.bindings.borrow_mut()[position] = binding;
            scope.inner.context.changed();
            return Ok(());
        }
        let slot = self.edit(scope)?;
        let published = scope.inner.bindings.borrow().len();
        if position >= published {
            self.record_undo(ScopeUndo::Appended(
                slot,
                position - published,
                self.edits[slot].appended[position - published].clone(),
            ))?;
            self.edits[slot].appended[position - published] = binding;
            return Ok(());
        }
        if let Some(record) = self.edits[slot].binding_slots.get(&position).copied() {
            self.record_undo(ScopeUndo::Binding(
                slot,
                record,
                self.edits[slot].bindings[record].1.clone(),
            ))?;
            self.edits[slot].bindings[record].1 = binding;
            return Ok(());
        }
        self.admit(ScopeWork::Record)?;
        self.admit(ScopeWork::Capacity {
            elements: self.edits[slot].bindings.len(),
        })?;
        let edit = &mut self.edits[slot];
        edit.bindings
            .try_reserve(1)
            .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
        edit.binding_slots
            .try_reserve(1)
            .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
        self.record_undo(ScopeUndo::PopBinding(slot))?;
        let edit = &mut self.edits[slot];
        edit.binding_slots.insert(position, edit.bindings.len());
        edit.bindings.push((position, binding));
        Ok(())
    }

    fn append(&mut self, scope: &Scope, binding: Binding) -> Result<(), ScopeAnalysisError<E>> {
        self.admit(ScopeWork::BindingWrite)?;
        if !self.staged {
            scope.inner.bindings.borrow_mut().push(binding);
            scope.inner.context.changed();
            return Ok(());
        }
        let slot = self.edit(scope)?;
        self.admit(ScopeWork::Record)?;
        self.admit(ScopeWork::Capacity {
            elements: self.edits[slot].appended.len(),
        })?;
        self.edits[slot]
            .appended
            .try_reserve(1)
            .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
        self.record_undo(ScopeUndo::PopAppended(slot))?;
        self.edits[slot].appended.push(binding);
        Ok(())
    }

    pub(crate) fn new_scope(
        &mut self,
        parent: Scope,
        function: bool,
    ) -> Result<Scope, ScopeAnalysisError<E>> {
        self.track(&parent)?;
        self.admit(ScopeWork::ScopeWrite)?;
        if !self.staged {
            return Ok(Scope::new(parent, function));
        }
        let slot = self.context_slots[&(Rc::as_ptr(&parent.inner.context) as usize)];
        let id = self.contexts[slot]
            .next_id
            .checked_add(1)
            .ok_or(ScopeAnalysisError::State(
                ScopeStateError::IdentityExhausted,
            ))?;
        let index = self
            .scope_index(&parent)?
            .checked_add(1)
            .filter(|index| *index < u32::MAX)
            .ok_or(ScopeAnalysisError::State(
                ScopeStateError::IdentityExhausted,
            ))?;
        self.admit(ScopeWork::Capacity { elements: 1 })?;
        self.record_undo(ScopeUndo::NextId(slot, self.contexts[slot].next_id))?;
        self.contexts[slot].next_id = id;
        Ok(Scope {
            inner: Rc::new(Inner {
                unique_id: id,
                outer: Some(parent.clone()),
                index: Cell::new(index),
                bindings: RefCell::default(),
                function,
                this_escaped: Cell::new(false),
                context: parent.inner.context.clone(),
            }),
        })
    }

    pub(crate) fn has_binding(
        &mut self,
        scope: &Scope,
        name: &JsString,
    ) -> Result<bool, ScopeAnalysisError<E>> {
        Ok(self.find(scope, name)?.is_some())
    }

    pub(crate) fn has_lex_binding(
        &mut self,
        scope: &Scope,
        name: &JsString,
    ) -> Result<bool, ScopeAnalysisError<E>> {
        Ok(self.find(scope, name)?.is_some_and(|(_, b)| b.is_lex()))
    }

    pub(crate) fn num_bindings(&mut self, scope: &Scope) -> Result<u32, ScopeAnalysisError<E>> {
        u32::try_from(self.binding_count(scope)?)
            .map_err(|_| ScopeAnalysisError::State(ScopeStateError::IdentityExhausted))
    }

    pub(crate) fn scope_index(&mut self, scope: &Scope) -> Result<u32, ScopeAnalysisError<E>> {
        let slot = self.slot(scope)?;
        Ok(slot
            .and_then(|s| self.edits[s].index)
            .unwrap_or_else(|| scope.scope_index()))
    }

    pub(crate) fn escaped_this(&mut self, scope: &Scope) -> Result<bool, ScopeAnalysisError<E>> {
        let slot = self.slot(scope)?;
        Ok(slot
            .and_then(|s| self.edits[s].this_escaped)
            .unwrap_or_else(|| scope.escaped_this()))
    }

    pub(crate) fn set_index(
        &mut self,
        scope: &Scope,
        index: u32,
    ) -> Result<(), ScopeAnalysisError<E>> {
        self.track(scope)?;
        self.admit(ScopeWork::ScopeWrite)?;
        if index == u32::MAX {
            return Err(ScopeAnalysisError::State(
                ScopeStateError::IdentityExhausted,
            ));
        }
        if self.staged {
            let slot = self.edit(scope)?;
            self.record_undo(ScopeUndo::Index(slot, self.edits[slot].index))?;
            self.edits[slot].index = Some(index);
        } else {
            scope.set_index(index);
        }
        Ok(())
    }

    pub(super) fn project_binding(
        &mut self,
        scope: &Scope,
        name: JsString,
        mutable: bool,
    ) -> Result<(), ScopeAnalysisError<E>> {
        let index = self.num_bindings(scope)?;
        if index == u32::MAX {
            return Err(ScopeAnalysisError::State(
                ScopeStateError::IdentityExhausted,
            ));
        }
        let slot = self.edit(scope)?;
        self.edits[slot].projected = true;
        let mut flags = BindingFlags::LEX | BindingFlags::ESCAPES;
        flags.set(BindingFlags::MUTABLE, mutable);
        flags.set(BindingFlags::STRICT, !mutable);
        self.append(scope, Binding { name, index, flags })
    }

    pub(crate) fn create_mutable_binding(
        &mut self,
        scope: &Scope,
        name: JsString,
        function_scope: bool,
    ) -> Result<BindingLocator, ScopeAnalysisError<E>> {
        let binding = if let Some((_, binding)) = self.find(scope, &name)? {
            binding
        } else {
            let index = self.num_bindings(scope)?;
            if index == u32::MAX {
                return Err(ScopeAnalysisError::State(
                    ScopeStateError::IdentityExhausted,
                ));
            }
            let mut flags = BindingFlags::MUTABLE;
            flags.set(BindingFlags::LEX, !function_scope);
            flags.set(BindingFlags::ESCAPES, scope.is_global());
            let binding = Binding {
                name: name.clone(),
                index,
                flags,
            };
            self.append(scope, binding.clone())?;
            binding
        };
        Ok(BindingLocator::declarative(
            name,
            self.scope_index(scope)?,
            binding.index,
            scope.unique_id(),
        ))
    }

    pub(crate) fn create_immutable_binding(
        &mut self,
        scope: &Scope,
        name: JsString,
        strict: bool,
    ) -> Result<(), ScopeAnalysisError<E>> {
        if self.find(scope, &name)?.is_some() {
            return Ok(());
        }
        let index = self.num_bindings(scope)?;
        if index == u32::MAX {
            return Err(ScopeAnalysisError::State(
                ScopeStateError::IdentityExhausted,
            ));
        }
        let mut flags = BindingFlags::LEX;
        flags.set(BindingFlags::STRICT, strict);
        flags.set(BindingFlags::ESCAPES, scope.is_global());
        self.append(scope, Binding { name, index, flags })
    }

    pub(crate) fn all_bindings_local(
        &mut self,
        scope: &Scope,
    ) -> Result<bool, ScopeAnalysisError<E>> {
        for position in 0..self.binding_count(scope)? {
            if self.binding(scope, position)?.escapes() {
                return Ok(false);
            }
        }
        Ok(true)
    }

    pub(crate) fn escape_all_bindings(
        &mut self,
        scope: &Scope,
    ) -> Result<(), ScopeAnalysisError<E>> {
        for position in 0..self.binding_count(scope)? {
            let mut binding = self.binding(scope, position)?;
            binding.flags.insert(BindingFlags::ESCAPES);
            self.write_binding(scope, position, binding)?;
        }
        Ok(())
    }

    pub(crate) fn reorder_binding_indices(
        &mut self,
        scope: &Scope,
    ) -> Result<(), ScopeAnalysisError<E>> {
        let mut index = 0;
        for position in 0..self.binding_count(scope)? {
            let mut binding = self.binding(scope, position)?;
            binding.index = if binding.escapes() {
                let current = index;
                index += 1;
                current
            } else {
                0
            };
            self.write_binding(scope, position, binding)?;
        }
        Ok(())
    }

    pub(crate) fn access_binding(
        &mut self,
        scope: &Scope,
        name: &JsString,
        eval_or_with: bool,
    ) -> Result<(), ScopeAnalysisError<E>> {
        let mut current = scope.clone();
        let mut crossed = false;
        loop {
            if let Some((position, mut binding)) = self.find(&current, name)? {
                binding.flags.insert(BindingFlags::ACCESSED);
                let escapes = binding.escapes() || crossed || eval_or_with;
                binding.flags.set(BindingFlags::ESCAPES, escapes);
                return self.write_binding(&current, position, binding);
            }
            let Some(outer) = current.outer() else {
                return Ok(());
            };
            self.admit(ScopeWork::ScopeHop)?;
            crossed |= current.is_function();
            current = outer.clone();
        }
    }

    pub(crate) fn escape_this_in_enclosing_function_scope(
        &mut self,
        scope: &Scope,
    ) -> Result<(), ScopeAnalysisError<E>> {
        let mut current = scope.clone();
        let mut crossed = false;
        loop {
            self.track(&current)?;
            if crossed && current.is_function() {
                self.admit(ScopeWork::ScopeWrite)?;
                if self.staged {
                    let slot = self.edit(&current)?;
                    self.record_undo(ScopeUndo::ThisEscaped(slot, self.edits[slot].this_escaped))?;
                    self.edits[slot].this_escaped = Some(true);
                } else {
                    current.inner.this_escaped.set(true);
                    current.inner.context.changed();
                }
                return Ok(());
            }
            let Some(outer) = current.outer() else {
                return Ok(());
            };
            self.admit(ScopeWork::ScopeHop)?;
            crossed |= current.is_function();
            current = outer.clone();
        }
    }

    pub(crate) fn arguments_object_accessed(
        &mut self,
        scopes: &FunctionScopes,
    ) -> Result<bool, ScopeAnalysisError<E>> {
        for scope in [
            Some(&scopes.function_scope),
            scopes.parameters_eval_scope.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            let scope = scope
                .scope_for(self.owner.as_ref())
                .map_err(ScopeAnalysisError::State)?
                .clone();
            if self.binding_count(&scope)? > 0 {
                let b = self.binding(&scope, 0)?;
                if self.names_equal(&b.name, &JsString::from("arguments"))? && b.is_accessed() {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    pub(crate) fn escape_function_bindings(
        &mut self,
        scopes: &FunctionScopes,
    ) -> Result<(), ScopeAnalysisError<E>> {
        for scope in [
            Some(&scopes.function_scope),
            scopes.parameters_eval_scope.as_ref(),
            scopes.parameters_scope.as_ref(),
            scopes.lexical_scope.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            let scope = scope
                .scope_for(self.owner.as_ref())
                .map_err(ScopeAnalysisError::State)?
                .clone();
            self.escape_all_bindings(&scope)?;
        }
        Ok(())
    }

    pub(crate) fn reorder_function_bindings(
        &mut self,
        scopes: &FunctionScopes,
    ) -> Result<(), ScopeAnalysisError<E>> {
        for scope in [
            Some(&scopes.function_scope),
            scopes.parameters_eval_scope.as_ref(),
            scopes.parameters_scope.as_ref(),
            scopes.lexical_scope.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            let scope = scope
                .scope_for(self.owner.as_ref())
                .map_err(ScopeAnalysisError::State)?
                .clone();
            self.reorder_binding_indices(&scope)?;
        }
        Ok(())
    }

    pub(super) fn prepare_publication(&mut self) -> Result<(), ScopeAnalysisError<E>> {
        if !self.staged {
            return Ok(());
        }
        // Admit every write and capacity operation before making any semantic changes.
        for slot in 0..self.edits.len() {
            self.admit(ScopeWork::Publication)?;
            let count = self.edits[slot].bindings.len() + self.edits[slot].appended.len();
            for _ in 0..count {
                self.admit(ScopeWork::Publication)?;
            }
            if !self.edits[slot].appended.is_empty() {
                let scope = self.edits[slot].scope.clone();
                let elements =
                    scope.inner.bindings.borrow().len() + self.edits[slot].appended.len();
                self.admit(ScopeWork::Capacity { elements })?;
                scope
                    .inner
                    .bindings
                    .borrow_mut()
                    .try_reserve(self.edits[slot].appended.len())
                    .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
            }
        }
        for slot in 0..self.contexts.len() {
            self.admit(ScopeWork::ScopeProbe)?;
            if self.contexts[slot].next_id != self.contexts[slot].context.unique_id_ctr.get() {
                self.admit(ScopeWork::Publication)?;
            }
        }
        self.admit(ScopeWork::Publication)?;
        self.admit(ScopeWork::ContextValidation {
            contexts: self.contexts.len(),
        })?;
        Ok(())
    }

    pub(crate) fn finish(&mut self) -> Result<(), ScopeAnalysisError<E>> {
        self.prepare_publication()?;
        self.commit_store()
            .map(|_| ())
            .map_err(ScopeAnalysisError::State)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pending_freshness_admits_both_context_scans_and_detects_reentry() {
        let root = Scope::new_global();
        let other = Scope::new_global();
        let mut store = PreparedScopes::new(&root);
        store
            .with_access(
                None::<&mut dyn FnMut(ScopeWork) -> Result<(), ()>>,
                |access| access.has_binding(&other.handle(), &JsString::from("absent")),
            )
            .expect("observe another tree");
        let mut events = Vec::new();
        let mut scans = 0;
        let mut admission = |work| {
            events.push(work);
            if matches!(work, ScopeWork::ContextValidation { .. }) {
                scans += 1;
                if scans == 2 {
                    drop(other.create_mutable_binding(JsString::from("callback"), false));
                }
            }
            Ok::<_, ()>(())
        };
        assert_eq!(
            store.validate_basis(Some(&mut admission)),
            Err(ScopeAnalysisError::State(ScopeStateError::Conflict))
        );
        assert_eq!(
            events,
            [
                ScopeWork::Capacity { elements: 2 },
                ScopeWork::ContextValidation { contexts: 2 },
                ScopeWork::ContextValidation { contexts: 2 }
            ]
        );
        assert!(other.has_binding(&JsString::from("callback")));
        assert_eq!(root.num_bindings(), 0);
        assert_eq!(store.validate_basis::<()>(None), Ok(BasisStatus::Stale));
    }

    #[test]
    fn compiler_abort_restores_existing_sparse_records_and_appends() {
        let root = Scope::new(Scope::new_global(), true);
        drop(root.create_mutable_binding(JsString::from("existing"), false));
        let mut store = PreparedScopes::new(&root);
        let handle = store.root_handle();
        store
            .with_access(
                None::<&mut dyn FnMut(ScopeWork) -> Result<(), ()>>,
                |access| {
                    drop(access.create_mutable_binding(
                        &handle,
                        JsString::from("private"),
                        false,
                    )?);
                    Ok::<_, ScopeAnalysisError<()>>(())
                },
            )
            .expect("private binding before savepoint");
        let abandoned;
        {
            let mut extension = store.begin_extension::<()>(None).expect("savepoint");
            abandoned = extension
                .with_access(
                    None::<&mut dyn FnMut(ScopeWork) -> Result<(), ()>>,
                    |access| {
                        access.access_binding(&handle, &JsString::from("existing"), true)?;
                        access.set_index(&handle, 7)?;
                        drop(access.create_mutable_binding(
                            &handle,
                            JsString::from("new"),
                            false,
                        )?);
                        access.new_scope(handle.clone(), false)
                    },
                )
                .expect("compiler's private edits");
        }
        store
            .with_access(
                None::<&mut dyn FnMut(ScopeWork) -> Result<(), ()>>,
                |access| {
                    assert_eq!(access.scope_index(&handle)?, 1);
                    assert_eq!(access.num_bindings(&handle)?, 2);
                    assert!(!access.has_binding(&handle, &JsString::from("new"))?);
                    let existing = access
                        .get_binding_reference(&handle, &JsString::from("existing"))?
                        .expect("existing binding");
                    assert!(existing.local());
                    assert!(access.scope_index(&abandoned).is_err());
                    Ok::<_, ScopeAnalysisError<()>>(())
                },
            )
            .expect("restored view");
        let installed = store
            .prepare_publication::<()>(None)
            .expect("admit commit")
            .commit()
            .expect("publish");
        assert_eq!(
            handle
                .resolve_live(&installed)
                .expect("canonical")
                .num_bindings(),
            2
        );
        assert_eq!(root.num_bindings(), 2);
        assert_eq!(Scope::new(root, false).unique_id(), 2);
    }

    #[test]
    fn publication_permit_authorizes_only_valid_handles_from_its_pending_owner() {
        let root = Scope::new_global();
        let mut scopes = PreparedScopes::new(&root);
        let parent = scopes.root_handle();
        let valid = scopes
            .with_access(
                None::<&mut dyn FnMut(ScopeWork) -> Result<(), ()>>,
                |access| access.new_scope(parent.clone(), true),
            )
            .expect("pending child");
        let abandoned = {
            let mut extension = scopes.begin_extension::<()>(None).expect("savepoint");
            extension
                .with_access(
                    None::<&mut dyn FnMut(ScopeWork) -> Result<(), ()>>,
                    |access| access.new_scope(parent.clone(), true),
                )
                .expect("abandoned child")
        };
        let foreign = PreparedScopes::new(&root);
        let foreign_handle = foreign.root_handle();
        let aborted = {
            let aborted = PreparedScopes::new(&root);
            aborted.root_handle()
        };
        let mut published = PreparedScopes::new(&root);
        let published_handle = published.root_handle();
        published
            .prepare_publication::<()>(None)
            .expect("admit other owner")
            .commit()
            .expect("publish other owner");
        let mut calls = 0;
        let mut admission = |_| {
            calls += 1;
            Ok::<_, ()>(())
        };
        let permit = scopes
            .prepare_publication(Some(&mut admission))
            .expect("admit");
        let admitted = calls;
        assert_eq!(permit.authorize_handle(&parent), Ok(()));
        assert_eq!(permit.authorize_handle(&valid), Ok(()));
        for invalid in [
            root.handle(),
            foreign_handle,
            aborted,
            abandoned,
            published_handle,
            ScopeHandle::default(),
        ] {
            assert_eq!(
                permit.authorize_handle(&invalid),
                Err(ScopeStateError::AssociationMismatch)
            );
        }
        assert_eq!(calls, admitted);
        let installed = permit.commit().expect("publish canonical owner");
        assert!(valid.resolve_live(&installed).is_ok());
        assert_eq!(calls, admitted);
    }

    #[test]
    fn publication_permit_checks_its_basis_without_invoking_policy_again() {
        let root = Scope::new_global();
        let mut store = PreparedScopes::new(&root);
        let handle = store.root_handle();
        store
            .with_access(
                None::<&mut dyn FnMut(ScopeWork) -> Result<(), ()>>,
                |access| access.project_binding(&handle, JsString::from("private"), true),
            )
            .expect("stage source declaration");
        let mut calls = 0;
        let mut admission = |_| {
            calls += 1;
            Ok::<_, ()>(())
        };
        let permit = store
            .prepare_publication(Some(&mut admission))
            .expect("admit");
        let admitted = calls;
        drop(root.create_mutable_binding(JsString::from("external"), false));
        assert!(matches!(permit.commit(), Err(ScopeStateError::Conflict)));
        assert_eq!(calls, admitted);
        assert_eq!(root.num_bindings(), 1);
        assert!(!root.has_binding(&JsString::from("private")));
        assert_eq!(
            handle.installed_scopes().expect_err("not activated"),
            ScopeStateError::Unprepared
        );
    }

    #[test]
    fn denied_publication_preserves_bindings_and_identity_counter() {
        let root = Scope::new_global();
        let mut admission = |work| {
            if work == ScopeWork::Publication {
                Err("stop")
            } else {
                Ok(())
            }
        };
        let mut session_store = PreparedScopes::new(&root);
        let mut session = ScopeSession::borrowed(&mut session_store, Some(&mut admission));
        let _ = session
            .create_mutable_binding(&root, JsString::from("staged"), false)
            .expect("stage binding");
        let _ = session
            .new_scope(root.clone(), false)
            .expect("stage identity");
        assert_eq!(root.num_bindings(), 0);
        assert_eq!(session.finish(), Err(ScopeAnalysisError::Refused("stop")));
        assert_eq!(root.num_bindings(), 0);
        assert_eq!(Scope::new(root, false).unique_id(), 1);
    }

    #[test]
    fn dropped_empty_child_still_spends_its_identity_on_success() {
        let root = Scope::new_global();
        let mut session_store = PreparedScopes::new(&root);
        let mut session = ScopeSession::<()>::borrowed(&mut session_store, None);
        drop(session.new_scope(root.clone(), false).expect("stage child"));
        session.finish().expect("publish");
        assert_eq!(Scope::new(root, false).unique_id(), 2);
    }

    #[test]
    fn successful_callback_mutation_conflicts_and_keeps_its_own_effect() {
        let root = Scope::new(Scope::new_global(), true);
        let _ = root.create_mutable_binding(JsString::from("visible"), false);
        let mut admission = |work| {
            if work == ScopeWork::BindingProbe {
                root.escape_all_bindings();
            }
            Ok::<_, ()>(())
        };
        let mut session_store = PreparedScopes::new(&root);
        let mut session = ScopeSession::borrowed(&mut session_store, Some(&mut admission));
        assert_eq!(
            session.has_binding(&root, &JsString::from("visible")),
            Err(ScopeAnalysisError::State(ScopeStateError::Conflict))
        );
        assert!(!root.all_bindings_local());
    }

    #[test]
    fn successful_callback_identity_allocation_conflicts() {
        let root = Scope::new_global();
        let mut admission = |work| {
            if work == ScopeWork::ScopeWrite {
                drop(Scope::new(root.clone(), false));
            }
            Ok::<_, ()>(())
        };
        let mut session_store = PreparedScopes::new(&root);
        let mut session = ScopeSession::borrowed(&mut session_store, Some(&mut admission));
        assert!(matches!(
            session.new_scope(root.clone(), false),
            Err(ScopeAnalysisError::State(ScopeStateError::Conflict))
        ));
        assert_eq!(Scope::new(root, false).unique_id(), 2);
    }

    #[test]
    fn callback_refusal_retains_precedence_over_its_mutation() {
        let root = Scope::new_global();
        let mut admission = |work| {
            if work == ScopeWork::ScopeWrite {
                let _ = root.create_mutable_binding(JsString::from("caller"), false);
                return Err("caller reason");
            }
            Ok(())
        };
        let mut session_store = PreparedScopes::new(&root);
        let mut session = ScopeSession::borrowed(&mut session_store, Some(&mut admission));
        assert_eq!(
            session.set_index(&root, 4),
            Err(ScopeAnalysisError::Refused("caller reason"))
        );
        assert!(root.has_binding(&JsString::from("caller")));
        assert_eq!(root.scope_index(), 0);
    }

    #[test]
    fn successful_publication_updates_the_canonical_scope_handle() {
        let root = Scope::new_global();
        let canonical = root.clone();
        let mut session_store = PreparedScopes::new(&root);
        let mut session = ScopeSession::<()>::borrowed(&mut session_store, None);
        let locator = session
            .create_mutable_binding(&root, JsString::from("value"), false)
            .expect("stage binding");
        assert_eq!(root.num_bindings(), 0);
        assert!(
            session
                .has_binding(&root, &JsString::from("value"))
                .expect("overlay read")
        );
        session.finish().expect("publish");
        assert_eq!(
            canonical.get_binding(&JsString::from("value")),
            Some(locator)
        );
    }

    #[test]
    fn revision_tracking_does_not_change_structural_scope_equality() {
        let a = Scope::new_global();
        let b = Scope::new_global();
        a.inner.context.changed();
        assert_eq!(a, b);
    }

    #[test]
    fn callback_mutation_of_an_additional_observed_tree_conflicts() {
        let root = Scope::new_global();
        let other = Scope::new_global();
        let mut admission = |work| {
            if work == ScopeWork::ScopeProbe {
                let _ = other.create_mutable_binding(JsString::from("caller"), false);
            }
            Ok::<_, ()>(())
        };
        let mut session_store = PreparedScopes::new(&root);
        let mut session = ScopeSession::borrowed(&mut session_store, Some(&mut admission));
        assert_eq!(
            session.has_binding(&other, &JsString::from("absent")),
            Err(ScopeAnalysisError::State(ScopeStateError::Conflict))
        );
        assert!(other.has_binding(&JsString::from("caller")));
        assert_eq!(root.num_bindings(), 0);
    }

    #[test]
    fn exhausted_identity_is_refused_without_publishing_changes() {
        let root = Scope::new_global();
        root.inner.context.unique_id_ctr.set(u32::MAX);
        let mut session_store = PreparedScopes::new(&root);
        let mut session = ScopeSession::<()>::borrowed(&mut session_store, None);
        let _ = session
            .create_mutable_binding(&root, JsString::from("pending"), false)
            .expect("stage binding");
        assert!(matches!(
            session.new_scope(root.clone(), false),
            Err(ScopeAnalysisError::State(
                ScopeStateError::IdentityExhausted
            ))
        ));
        assert_eq!(root.num_bindings(), 0);
        assert_eq!(root.inner.context.unique_id_ctr.get(), u32::MAX);
    }

    #[test]
    fn revision_admission_mutation_prevents_requested_work() {
        let root = Scope::new_global();
        let mut requested_work_called = false;
        let mut admission = |work| {
            if matches!(work, ScopeWork::ContextValidation { .. }) {
                let _ = root.create_mutable_binding(JsString::from("caller"), false);
            } else {
                requested_work_called = true;
            }
            Ok::<_, ()>(())
        };
        let mut session_store = PreparedScopes::new(&root);
        let mut session = ScopeSession::borrowed(&mut session_store, Some(&mut admission));
        assert_eq!(
            session.admit(ScopeWork::Node),
            Err(ScopeAnalysisError::State(ScopeStateError::Conflict))
        );
        assert!(!requested_work_called);
        assert!(root.has_binding(&JsString::from("caller")));
    }

    #[test]
    fn denied_final_publication_preserves_existing_physical_binding_slots_and_retry_publishes() {
        fn caller_scope() -> Scope {
            let caller = Scope::new(Scope::new_global(), true);
            let _ = caller.create_mutable_binding(JsString::from("local"), false);
            let mut store = PreparedScopes::live(&caller);
            ScopeSession::<()>::borrowed(&mut store, None)
                .create_immutable_binding(&caller, JsString::from("strict"), true)
                .expect("seed strict binding");
            let _ = caller.create_mutable_binding(JsString::from("tail"), false);
            caller.reorder_binding_indices();
            caller
        }

        fn stage_changes<E>(
            session: &mut ScopeSession<'_, E>,
            caller: &Scope,
        ) -> Result<Scope, ScopeAnalysisError<E>> {
            let child = session.new_scope(caller.clone(), true)?;
            drop(session.new_scope(caller.clone(), false)?);
            session.access_binding(caller, &JsString::from("local"), false)?;
            session.access_binding(&child, &JsString::from("strict"), false)?;
            session.access_binding(&child, &JsString::from("tail"), false)?;
            session.reorder_binding_indices(caller)?;
            session.set_index(caller, 7)?;
            session.escape_this_in_enclosing_function_scope(&child)?;
            Ok(child)
        }

        // Locate the final publication fault point through the actual admission
        // stream. Expected binding state below comes from the seeded declarations
        // and their accesses, independently of the number of publication events.
        let calibration_caller = caller_scope();
        let mut publications = 0;
        let mut accept = |work| {
            if work == ScopeWork::Publication {
                publications += 1;
            }
            Ok::<_, ()>(())
        };
        let mut calibration_store = PreparedScopes::new(&calibration_caller);
        let mut calibration = ScopeSession::borrowed(&mut calibration_store, Some(&mut accept));
        drop(stage_changes(&mut calibration, &calibration_caller).expect("stage calibration"));
        calibration.finish().expect("complete calibration");
        assert!(publications > 0);

        let caller = caller_scope();
        let canonical_parent = caller.clone();
        let before_bindings = caller.inner.bindings.borrow().clone();
        assert_eq!(before_bindings.len(), 3);
        assert_eq!(
            before_bindings.iter().map(|b| b.index).collect::<Vec<_>>(),
            [0, 0, 0]
        );
        assert_eq!(
            before_bindings[0].flags,
            BindingFlags::MUTABLE | BindingFlags::LEX
        );
        assert_eq!(
            before_bindings[1].flags,
            BindingFlags::LEX | BindingFlags::STRICT
        );
        assert_eq!(
            before_bindings[2].flags,
            BindingFlags::MUTABLE | BindingFlags::LEX
        );
        assert_eq!(caller.scope_index(), 1);
        assert!(!caller.escaped_this());
        assert_eq!(caller.inner.context.unique_id_ctr.get(), 1);

        let mut publication = 0;
        let mut deny_final = |work| {
            if work == ScopeWork::Publication {
                publication += 1;
                if publication == publications {
                    return Err("final publication refused");
                }
            }
            Ok(())
        };
        let mut denied_store = PreparedScopes::new(&caller);
        let mut denied = ScopeSession::borrowed(&mut denied_store, Some(&mut deny_final));
        let denied_child = stage_changes(&mut denied, &caller).expect("stage existing bindings");
        assert_eq!(denied_child.unique_id(), 2);
        assert_eq!(denied.scope_index(&caller), Ok(7));
        assert_eq!(denied.escaped_this(&caller), Ok(true));
        assert_eq!(
            denied.finish(),
            Err(ScopeAnalysisError::Refused("final publication refused"))
        );
        assert_eq!(publication, publications);
        assert_eq!(*caller.inner.bindings.borrow(), before_bindings);
        assert_eq!(caller.scope_index(), 1);
        assert!(!caller.escaped_this());
        assert_eq!(caller.unique_id(), 1);
        assert_eq!(caller.inner.context.unique_id_ctr.get(), 1);

        let mut retry_store = PreparedScopes::new(&caller);
        let mut retry = ScopeSession::<()>::borrowed(&mut retry_store, None);
        let child = stage_changes(&mut retry, &caller).expect("stage retry");
        assert_eq!(child.unique_id(), 2);
        retry.finish().expect("publish retry");
        assert!(Rc::ptr_eq(&caller.inner, &canonical_parent.inner));
        assert!(Rc::ptr_eq(
            &child.outer().expect("canonical parent").inner,
            &canonical_parent.inner
        ));
        assert_eq!(canonical_parent.scope_index(), 7);
        assert!(canonical_parent.escaped_this());
        let bindings = canonical_parent.inner.bindings.borrow();
        assert_eq!(
            bindings.iter().map(|b| b.index).collect::<Vec<_>>(),
            [0, 0, 1]
        );
        assert_eq!(
            bindings[0].flags,
            BindingFlags::MUTABLE | BindingFlags::LEX | BindingFlags::ACCESSED
        );
        assert_eq!(
            bindings[1].flags,
            BindingFlags::LEX
                | BindingFlags::STRICT
                | BindingFlags::ACCESSED
                | BindingFlags::ESCAPES
        );
        assert_eq!(
            bindings[2].flags,
            BindingFlags::MUTABLE
                | BindingFlags::LEX
                | BindingFlags::ACCESSED
                | BindingFlags::ESCAPES
        );
        assert_eq!(
            bindings.iter().map(|b| b.name.clone()).collect::<Vec<_>>(),
            [
                JsString::from("local"),
                JsString::from("strict"),
                JsString::from("tail")
            ]
        );
        drop(bindings);
        assert_eq!(caller.inner.context.unique_id_ctr.get(), 3);
        assert_eq!(Scope::new(caller, false).unique_id(), 4);
    }
}
