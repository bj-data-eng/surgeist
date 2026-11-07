//! Sparse analysis overlay. Published scopes are changed only after admission ends.
use super::*;
use std::collections::HashMap;

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
    Syntax(&'static str),
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
}

impl std::fmt::Display for ScopeStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Conflict => "scope state changed during analysis",
            Self::IdentityExhausted => "scope identity space exhausted",
            Self::Allocation => "scope analysis storage allocation failed",
        })
    }
}
impl std::error::Error for ScopeStateError {}

/// Optional caller admission for controlled scope work.
pub type ScopeAdmission<'a, E> = Option<&'a mut dyn FnMut(ScopeWork) -> Result<(), E>>;

struct ContextEdit {
    context: Rc<ScopeContext>,
    revision: Option<u64>,
    next_id: u32,
}

struct ScopeEdit {
    scope: Scope,
    index: Option<u32>,
    this_escaped: Option<bool>,
    bindings: Vec<(usize, Binding)>,
    binding_slots: HashMap<usize, usize>,
    appended: Vec<Binding>,
}

/// One analysis, with direct legacy and staged controlled execution modes.
pub(crate) struct ScopeSession<'a, E> {
    admission: ScopeAdmission<'a, E>,
    staged: bool,
    contexts: Vec<ContextEdit>,
    context_slots: HashMap<usize, usize>,
    edits: Vec<ScopeEdit>,
    edit_slots: HashMap<usize, usize>,
}

impl<'a, E> ScopeSession<'a, E> {
    pub(crate) fn direct() -> Self {
        Self {
            admission: None,
            staged: false,
            contexts: Vec::new(),
            context_slots: HashMap::new(),
            edits: Vec::new(),
            edit_slots: HashMap::new(),
        }
    }

    pub(crate) fn new(root: &Scope, admission: ScopeAdmission<'a, E>) -> Self {
        let context = root.inner.context.clone();
        let key = Rc::as_ptr(&context) as usize;
        Self {
            admission,
            staged: true,
            contexts: vec![ContextEdit {
                revision: context.revision.get(),
                next_id: context.unique_id_ctr.get(),
                context,
            }],
            context_slots: HashMap::from([(key, 0)]),
            edits: Vec::new(),
            edit_slots: HashMap::new(),
        }
    }

    fn validate(&self) -> Result<(), ScopeAnalysisError<E>> {
        // Full scans make ContextValidation's admitted probe count exact, including
        // when an earlier tree already conflicts.
        let mut conflict = false;
        for context in &self.contexts {
            let current = context.context.revision.get();
            conflict |= context.revision.is_none() || current != context.revision;
        }
        if conflict {
            Err(ScopeAnalysisError::State(ScopeStateError::Conflict))
        } else {
            Ok(())
        }
    }

    pub(crate) fn admit(&mut self, work: ScopeWork) -> Result<(), ScopeAnalysisError<E>> {
        // There is never a RefCell borrow across either caller invocation.
        if let Some(callback) = &mut self.admission {
            callback(ScopeWork::ContextValidation {
                contexts: self.contexts.len().saturating_mul(2),
            })
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
        self.context_slots.insert(key, self.contexts.len());
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

    fn find(
        &mut self,
        scope: &Scope,
        name: &JsString,
    ) -> Result<Option<(usize, Binding)>, ScopeAnalysisError<E>> {
        let len = self.binding_count(scope)?;
        for position in 0..len {
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
            self.edits[slot].appended[position - published] = binding;
            return Ok(());
        }
        if let Some(record) = self.edits[slot].binding_slots.get(&position).copied() {
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
            self.edits[slot].index = Some(index);
        } else {
            scope.set_index(index);
        }
        Ok(())
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
            if self.binding_count(scope)? > 0 {
                let b = self.binding(scope, 0)?;
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
            self.escape_all_bindings(scope)?;
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
            self.reorder_binding_indices(scope)?;
        }
        Ok(())
    }

    pub(crate) fn finish(mut self) -> Result<(), ScopeAnalysisError<E>> {
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
        // Callback-free commit: capacity was prepared above and canonical Rc identities survive.
        for edit in self.edits {
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
        for context in self.contexts {
            if context.next_id != context.context.unique_id_ctr.get() {
                context.context.unique_id_ctr.set(context.next_id);
                context.context.changed();
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        let mut session = ScopeSession::new(&root, Some(&mut admission));
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
        let mut session = ScopeSession::<()>::new(&root, None);
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
        let mut session = ScopeSession::new(&root, Some(&mut admission));
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
        let mut session = ScopeSession::new(&root, Some(&mut admission));
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
        let mut session = ScopeSession::new(&root, Some(&mut admission));
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
        let mut session = ScopeSession::<()>::new(&root, None);
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
        let mut session = ScopeSession::new(&root, Some(&mut admission));
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
        let mut session = ScopeSession::<()>::new(&root, None);
        let _ = session
            .create_mutable_binding(&root, JsString::from("pending"), false)
            .expect("stage binding");
        assert!(matches!(
            session.new_scope(root.clone(), false),
            Err(ScopeAnalysisError::State(
                ScopeStateError::IdentityExhausted
            ))
        ));
        drop(session);
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
        let mut session = ScopeSession::new(&root, Some(&mut admission));
        assert_eq!(
            session.admit(ScopeWork::Node),
            Err(ScopeAnalysisError::State(ScopeStateError::Conflict))
        );
        drop(session);
        assert!(!requested_work_called);
        assert!(root.has_binding(&JsString::from("caller")));
    }
}
