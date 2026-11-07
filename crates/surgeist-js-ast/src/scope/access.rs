use super::analysis::{PreparedScopes, ScopeSession};
use super::{
    BindingLocator, BindingLocatorError, FunctionScopes, IdentifierReference, Scope,
    ScopeAdmission, ScopeAnalysisError, ScopeHandle, ScopeStateError, ScopeWork,
};
use surgeist_js_string::JsString;

/// Immutable scope metadata suitable for a prepared code constant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ScopeDisplay {
    /// The effective runtime scope index.
    pub index: u32,
    /// The effective number of bindings.
    pub bindings: u32,
}

/// A borrowed policy and scope view, never retained by a prepared artifact.
pub struct ScopeAccess<'a, E> {
    pub(crate) session: ScopeSession<'a, E>,
}

impl<'a, E> ScopeAccess<'a, E> {
    pub(crate) fn borrowed(
        store: &'a mut PreparedScopes,
        admission: ScopeAdmission<'a, E>,
    ) -> Self {
        Self {
            session: ScopeSession::borrowed(store, admission),
        }
    }

    /// Uses published scopes under a temporary caller policy.
    ///
    /// # Errors
    /// Returns the operation's error, including admission and scope-state errors.
    pub fn with_live<T, F>(
        root: &Scope,
        admission: ScopeAdmission<'_, E>,
        operation: impl FnOnce(&mut ScopeAccess<'_, E>) -> Result<T, F>,
    ) -> Result<T, F>
    where
        F: From<ScopeAnalysisError<E>>,
    {
        let mut store = PreparedScopes::live(root);
        store.with_access(admission, operation)
    }

    fn scope(&self, handle: &ScopeHandle) -> Result<Scope, ScopeAnalysisError<E>> {
        handle
            .scope_for(self.session.owner.as_ref())
            .cloned()
            .map_err(ScopeAnalysisError::State)
    }

    pub(crate) fn bind_root(&mut self, root: &Scope) -> Result<ScopeHandle, ScopeAnalysisError<E>> {
        if let Some(existing) = &self.session.root {
            if !std::rc::Rc::ptr_eq(&existing.inner, &root.inner) {
                return Err(ScopeAnalysisError::State(
                    ScopeStateError::AssociationMismatch,
                ));
            }
        } else {
            self.session.root = Some(root.clone());
        }
        Ok(self.session.root_handle())
    }

    pub(crate) fn has_admission(&self) -> bool {
        self.session.has_admission()
    }

    pub(crate) fn admit(&mut self, work: ScopeWork) -> Result<(), ScopeAnalysisError<E>> {
        self.session.admit(work)
    }

    pub(crate) fn finish(&mut self) -> Result<(), ScopeAnalysisError<E>> {
        self.session.finish()
    }

    /// Gets the root stamped with this view's association.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn root_handle(&mut self) -> Result<ScopeHandle, ScopeAnalysisError<E>> {
        self.session.admit(ScopeWork::ScopeProbe)?;
        if self.session.root.is_none() {
            return Err(ScopeAnalysisError::State(ScopeStateError::Unprepared));
        }
        Ok(self.session.root_handle())
    }

    /// Associates an accessible scope with this prepared view.
    ///
    /// # Errors
    /// Returns an error for an unrelated pending or aborted handle.
    pub fn associate(
        &mut self,
        handle: &ScopeHandle,
    ) -> Result<ScopeHandle, ScopeAnalysisError<E>> {
        let scope = self.scope(handle)?;
        self.session.admit(ScopeWork::ScopeProbe)?;
        if handle.associated_with(self.session.owner.as_ref()) {
            Ok(handle.clone())
        } else {
            Ok(self.session.bind(scope, None))
        }
    }

    /// Gets the effective outer scope, preserving this view's ownership.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn outer(
        &mut self,
        handle: &ScopeHandle,
    ) -> Result<Option<ScopeHandle>, ScopeAnalysisError<E>> {
        self.scope(handle)?;
        self.session.admit(ScopeWork::ScopeHop)?;
        handle
            .outer()
            .map(|outer| self.associate(&outer))
            .transpose()
    }

    /// Checks whether a scope is a function scope.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn is_function(&mut self, handle: &ScopeHandle) -> Result<bool, ScopeAnalysisError<E>> {
        let scope = self.scope(handle)?;
        self.session.admit(ScopeWork::ScopeProbe)?;
        Ok(scope.is_function())
    }

    /// Checks whether a scope is global.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn is_global(&mut self, handle: &ScopeHandle) -> Result<bool, ScopeAnalysisError<E>> {
        let scope = self.scope(handle)?;
        self.session.admit(ScopeWork::ScopeProbe)?;
        Ok(scope.is_global())
    }

    /// Gets the effective runtime scope index.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn scope_index(&mut self, handle: &ScopeHandle) -> Result<u32, ScopeAnalysisError<E>> {
        let scope = self.scope(handle)?;
        self.session.scope_index(&scope)
    }

    /// Gets the effective binding count.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn num_bindings(&mut self, handle: &ScopeHandle) -> Result<u32, ScopeAnalysisError<E>> {
        let scope = self.scope(handle)?;
        self.session.num_bindings(&scope)
    }

    /// Captures effective immutable metadata for display.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn describe(
        &mut self,
        handle: &ScopeHandle,
    ) -> Result<ScopeDisplay, ScopeAnalysisError<E>> {
        Ok(ScopeDisplay {
            index: self.scope_index(handle)?,
            bindings: self.num_bindings(handle)?,
        })
    }

    /// Checks whether all bindings remain local.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn all_bindings_local(
        &mut self,
        handle: &ScopeHandle,
    ) -> Result<bool, ScopeAnalysisError<E>> {
        let scope = self.scope(handle)?;
        self.session.all_bindings_local(&scope)
    }

    /// Checks the effective escaped-this state.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn escaped_this(&mut self, handle: &ScopeHandle) -> Result<bool, ScopeAnalysisError<E>> {
        let scope = self.scope(handle)?;
        self.session.escaped_this(&scope)
    }

    /// Checks a current-scope binding.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn has_binding(
        &mut self,
        handle: &ScopeHandle,
        name: &JsString,
    ) -> Result<bool, ScopeAnalysisError<E>> {
        let scope = self.scope(handle)?;
        self.session.has_binding(&scope, name)
    }

    /// Checks a current-scope lexical binding.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn has_lex_binding(
        &mut self,
        handle: &ScopeHandle,
        name: &JsString,
    ) -> Result<bool, ScopeAnalysisError<E>> {
        let scope = self.scope(handle)?;
        self.session.has_lex_binding(&scope, name)
    }

    fn reference(
        &mut self,
        scope: &Scope,
        name: JsString,
        binding: &super::Binding,
    ) -> Result<IdentifierReference, ScopeAnalysisError<E>> {
        Ok(IdentifierReference::new(
            BindingLocator::declarative(
                name,
                self.session.scope_index(scope)?,
                binding.index,
                scope.unique_id(),
            ),
            binding.is_lex(),
            binding.escapes(),
        ))
    }

    /// Resolves a binding in this scope only.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn get_binding_reference(
        &mut self,
        handle: &ScopeHandle,
        name: &JsString,
    ) -> Result<Option<IdentifierReference>, ScopeAnalysisError<E>> {
        let scope = self.scope(handle)?;
        self.session
            .find(&scope, name)?
            .map(|(_, binding)| self.reference(&scope, name.clone(), &binding))
            .transpose()
    }

    /// Resolves a binding locator in this scope only.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn get_binding(
        &mut self,
        handle: &ScopeHandle,
        name: &JsString,
    ) -> Result<Option<BindingLocator>, ScopeAnalysisError<E>> {
        Ok(self
            .get_binding_reference(handle, name)?
            .map(|reference| reference.locator()))
    }

    /// Checks mutability in this scope or its outer scopes.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn is_binding_mutable(
        &mut self,
        handle: &ScopeHandle,
        name: &JsString,
    ) -> Result<Option<bool>, ScopeAnalysisError<E>> {
        let mut scope = self.scope(handle)?;
        loop {
            if let Some((_, binding)) = self.session.find(&scope, name)? {
                return Ok(Some(binding.is_mutable()));
            }
            let Some(outer) = scope.outer() else {
                return Ok(None);
            };
            self.session.admit(ScopeWork::ScopeHop)?;
            scope = outer.clone();
        }
    }

    /// Resolves an identifier through the effective outer chain.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn get_identifier_reference(
        &mut self,
        handle: &ScopeHandle,
        name: JsString,
    ) -> Result<IdentifierReference, ScopeAnalysisError<E>> {
        let mut scope = self.scope(handle)?;
        loop {
            if let Some((_, binding)) = self.session.find(&scope, &name)? {
                return self.reference(&scope, name, &binding);
            }
            let Some(outer) = scope.outer() else {
                return Ok(IdentifierReference::new(
                    BindingLocator::global(name),
                    false,
                    true,
                ));
            };
            self.session.admit(ScopeWork::ScopeHop)?;
            scope = outer.clone();
        }
    }

    /// Resolves an assignment while retaining ordinary binding failures separately.
    ///
    /// # Errors
    /// The outer result returns admission/state failures.
    pub fn set_mutable_binding(
        &mut self,
        handle: &ScopeHandle,
        name: JsString,
    ) -> Result<Result<IdentifierReference, BindingLocatorError>, ScopeAnalysisError<E>> {
        self.mutable_reference(handle, name, false)
    }

    /// Resolves an Annex B var assignment through function scopes.
    ///
    /// # Errors
    /// The outer result returns admission/state failures.
    #[cfg(feature = "annex-b")]
    pub fn set_mutable_binding_var(
        &mut self,
        handle: &ScopeHandle,
        name: JsString,
    ) -> Result<Result<IdentifierReference, BindingLocatorError>, ScopeAnalysisError<E>> {
        self.mutable_reference(handle, name, true)
    }

    fn mutable_reference(
        &mut self,
        handle: &ScopeHandle,
        name: JsString,
        function_only: bool,
    ) -> Result<Result<IdentifierReference, BindingLocatorError>, ScopeAnalysisError<E>> {
        let mut scope = self.scope(handle)?;
        loop {
            if (!function_only || scope.is_function())
                && let Some((_, binding)) = self.session.find(&scope, &name)?
            {
                return Ok(if binding.is_mutable() {
                    Ok(self.reference(&scope, name, &binding)?)
                } else if binding.is_strict() {
                    Err(BindingLocatorError::MutateImmutable)
                } else {
                    Err(BindingLocatorError::Silent)
                });
            }
            let Some(outer) = scope.outer() else {
                return Ok(Ok(IdentifierReference::new(
                    BindingLocator::global(name),
                    false,
                    true,
                )));
            };
            self.session.admit(ScopeWork::ScopeHop)?;
            scope = outer.clone();
        }
    }

    /// Creates a child with this view's canonical identity association.
    ///
    /// # Errors
    /// Returns admission, allocation, or identity failures.
    pub fn new_scope(
        &mut self,
        parent: ScopeHandle,
        function: bool,
    ) -> Result<ScopeHandle, ScopeAnalysisError<E>> {
        let raw_parent = self.scope(&parent)?;
        self.session.reserve_handle()?;
        let scope = self.session.new_scope(raw_parent, function)?;
        let handle = self.session.bind(scope, Some(parent));
        if self.session.staged {
            self.session.handles.push(handle.clone());
        }
        Ok(handle)
    }

    /// Creates a private module scope with mutable, nonlocal exported bindings.
    ///
    /// Returned locators preserve `exports` order. The caller owns duplicate-export
    /// validation; repeated names refer to the same binding. Only a staged scope
    /// owner supports this operation. Any failure invalidates that owner, so a
    /// partial module group cannot be published after an ignored error.
    ///
    /// # Errors
    /// Returns the original admission refusal, or a scope-state error.
    pub fn create_module_scope(
        &mut self,
        parent: &ScopeHandle,
        exports: &[JsString],
    ) -> Result<(ScopeHandle, Vec<BindingLocator>), ScopeAnalysisError<E>> {
        let result = (|| {
            if !self.session.staged {
                return Err(ScopeAnalysisError::State(ScopeStateError::Unprepared));
            }
            self.admit(ScopeWork::Capacity {
                elements: exports.len(),
            })?;
            let mut locators = Vec::new();
            locators
                .try_reserve(exports.len())
                .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
            let module = self.new_scope(parent.clone(), true)?;
            for name in exports {
                self.admit(ScopeWork::NamePreparation)?;
                if self.has_admission() {
                    for _ in name.iter() {
                        self.admit(ScopeWork::NameUnit)?;
                    }
                }
                let binding = self.create_mutable_binding(&module, name.clone(), false)?;
                self.admit(ScopeWork::Record)?;
                locators.push(binding);
            }
            self.escape_all_bindings(&module)?;
            Ok((module, locators))
        })();
        if result.is_err() {
            self.session.invalidate();
        }
        result
    }

    /// Checks mapped-argument access using the effective binding records.
    ///
    /// # Errors
    /// Returns an admission or scope-state error.
    pub fn arguments_object_accessed(
        &mut self,
        scopes: &FunctionScopes,
    ) -> Result<bool, ScopeAnalysisError<E>> {
        self.session.arguments_object_accessed(scopes)
    }
}

macro_rules! scope_operation {
    ($name:ident $(, $argument:ident : $ty:ty)*) => {
        pub(crate) fn $name(
            &mut self,
            handle: &ScopeHandle,
            $($argument: $ty,)*
        ) -> Result<(), ScopeAnalysisError<E>> {
            let scope = self.scope(handle)?;
            self.session.$name(&scope, $($argument,)*)
        }
    };
}

impl<E> ScopeAccess<'_, E> {
    scope_operation!(set_index, index: u32);
    scope_operation!(create_immutable_binding, name: JsString, strict: bool);
    scope_operation!(escape_all_bindings);
    scope_operation!(reorder_binding_indices);
    scope_operation!(access_binding, name: &JsString, eval_or_with: bool);
    scope_operation!(escape_this_in_enclosing_function_scope);

    pub(crate) fn projects_declarations(&self) -> bool {
        self.session.project_declarations
    }

    pub(crate) fn project_binding(
        &mut self,
        handle: &ScopeHandle,
        name: JsString,
        mutable: bool,
    ) -> Result<(), ScopeAnalysisError<E>> {
        let scope = self.scope(handle)?;
        self.session.project_binding(&scope, name, mutable)
    }

    pub(crate) fn create_mutable_binding(
        &mut self,
        handle: &ScopeHandle,
        name: JsString,
        function_scope: bool,
    ) -> Result<BindingLocator, ScopeAnalysisError<E>> {
        let scope = self.scope(handle)?;
        self.session
            .create_mutable_binding(&scope, name, function_scope)
    }

    pub(crate) fn escape_function_bindings(
        &mut self,
        scopes: &FunctionScopes,
    ) -> Result<(), ScopeAnalysisError<E>> {
        self.session.escape_function_bindings(scopes)
    }

    pub(crate) fn reorder_function_bindings(
        &mut self,
        scopes: &FunctionScopes,
    ) -> Result<(), ScopeAnalysisError<E>> {
        self.session.reorder_function_bindings(scopes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn module_constructor_preserves_export_order_and_installs_mutable_nonlocal_bindings() {
        let root = Scope::new_global();
        let exports = [
            JsString::from("second"),
            JsString::from("first"),
            JsString::from("second"),
        ];
        let mut scopes = PreparedScopes::new(&root);
        let (module, locators) = scopes
            .with_access(
                None::<&mut dyn FnMut(ScopeWork) -> Result<(), ()>>,
                |access| {
                    let parent = access.root_handle()?;
                    let (module, locators) = access.create_module_scope(&parent, &exports)?;
                    assert!(!access.all_bindings_local(&module)?);
                    assert_eq!(access.num_bindings(&module)?, 2);
                    Ok::<_, ScopeAnalysisError<()>>((module, locators))
                },
            )
            .expect("prepare exports");
        assert_eq!(locators[0], locators[2]);
        assert_eq!(locators[0].binding_index(), 0);
        assert_eq!(locators[1].binding_index(), 1);
        let permit = scopes.prepare_publication::<()>(None).expect("admit");
        permit
            .authorize_handle(&module)
            .expect("same pending owner");
        let installed = permit.commit().expect("publish");
        let module = module.resolve_live(&installed).expect("installed");
        assert!(module.is_function());
        assert_eq!(module.get_binding(&exports[0]), Some(locators[0].clone()));
        assert_eq!(module.get_binding(&exports[1]), Some(locators[1].clone()));
        assert!(module.set_mutable_binding(exports[0].clone()).is_ok());
        assert!(!module.all_bindings_local());
        assert_eq!(root.num_bindings(), 0);
    }

    #[test]
    fn every_module_constructor_refusal_aborts_the_group_without_spending_identities() {
        let exports = [JsString::from("exported")];
        let root = Scope::new_global();
        let mut calibration = PreparedScopes::new(&root);
        let mut events = 0;
        let mut accept = |_| {
            events += 1;
            Ok::<_, usize>(())
        };
        calibration
            .with_access(Some(&mut accept), |access| {
                let parent = access.session.root_handle();
                access.create_module_scope(&parent, &exports)
            })
            .expect("calibrate");
        drop(calibration);
        for denied in 0..events {
            let root = Scope::new_global();
            let mut scopes = PreparedScopes::new(&root);
            let mut seen = 0;
            let mut refuse = |_| {
                let index = seen;
                seen += 1;
                if index == denied { Err(denied) } else { Ok(()) }
            };
            let result = scopes.with_access(Some(&mut refuse), |access| {
                let parent = access.session.root_handle();
                access.create_module_scope(&parent, &exports)
            });
            assert!(matches!(result, Err(ScopeAnalysisError::Refused(reason)) if reason == denied));
            assert!(matches!(
                scopes.prepare_publication::<()>(None),
                Err(ScopeAnalysisError::State(
                    ScopeStateError::AssociationMismatch
                ))
            ));
            assert_eq!(root.num_bindings(), 0);
            assert_eq!(Scope::new(root, false).unique_id(), 1);
        }
    }

    #[test]
    fn live_module_constructor_rejects_before_allocating_or_installing_exports() {
        let root = Scope::new_global();
        let result = ScopeAccess::<()>::with_live(&root, None, |access| {
            let parent = access.root_handle()?;
            access.create_module_scope(&parent, &[JsString::from("exported")])
        });
        assert!(matches!(
            result,
            Err(ScopeAnalysisError::State(ScopeStateError::Unprepared))
        ));
        assert_eq!(root.num_bindings(), 0);
        assert_eq!(Scope::new(root, false).unique_id(), 1);
    }
}
