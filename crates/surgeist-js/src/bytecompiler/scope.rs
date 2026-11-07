//! Scope authority borrowed by one compiler and its nested compilers.
use super::ByteCompiler;
use crate::{JsResult, JsString};
use std::fmt;
use surgeist_js_ast::scope::{
    BindingLocatorError, FunctionScopes, IdentifierReference, Scope, ScopeAccess, ScopeAdmission,
    ScopeExtension, ScopeHandle,
};
use surgeist_js_parser::frontend::FrontendLimit;

/// The actual pending or published scope authority for a compilation.
pub(crate) enum CompilerScopes<'compile, 'store> {
    Pending(&'compile mut ScopeExtension<'store>),
    Live(Scope),
}

impl fmt::Debug for CompilerScopes<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Pending(_) => "PendingScopes",
            Self::Live(_) => "LiveScopes",
        })
    }
}

impl<'store> CompilerScopes<'_, 'store> {
    pub(crate) fn reborrow(&mut self) -> CompilerScopes<'_, 'store> {
        match self {
            Self::Pending(extension) => CompilerScopes::Pending(extension),
            Self::Live(root) => CompilerScopes::Live(root.clone()),
        }
    }
}

pub(super) struct CompilerAdmission<'a>(pub(super) ScopeAdmission<'a, FrontendLimit>);

impl fmt::Debug for CompilerAdmission<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CompilerAdmission")
            .field(&self.0.is_some())
            .finish()
    }
}

impl CompilerAdmission<'_> {
    pub(super) fn reborrow(&mut self) -> ScopeAdmission<'_, FrontendLimit> {
        self.0.as_mut().map(|callback| &mut **callback as _)
    }
}

impl ByteCompiler<'_, '_> {
    pub(crate) fn with_scopes<T>(
        &mut self,
        operation: impl FnOnce(&mut ScopeAccess<'_, FrontendLimit>) -> JsResult<T>,
    ) -> JsResult<T> {
        let admission = self.scope_admission.reborrow();
        match &mut self.scopes {
            CompilerScopes::Pending(extension) => extension.with_access(admission, operation),
            CompilerScopes::Live(root) => ScopeAccess::with_live(root, admission, operation),
        }
    }

    pub(super) fn push_scope_constant(&mut self, handle: &ScopeHandle) -> JsResult<u32> {
        let (handle, display) = self.with_scopes(|access| {
            let handle = access.associate(handle)?;
            let display = crate::vm::ScopeDisplay {
                index: access.scope_index(&handle)?,
                bindings: access.num_bindings(&handle)?,
            };
            Ok((handle, display))
        })?;
        let index = self.constants.len() as u32;
        self.constants
            .push(crate::vm::Constant::Scope(handle, display));
        Ok(index)
    }

    pub(super) fn scope_is_function(&mut self, scope: &ScopeHandle) -> JsResult<bool> {
        self.with_scopes(|access| Ok(access.is_function(scope)?))
    }

    pub(super) fn scope_all_bindings_local(&mut self, scope: &ScopeHandle) -> JsResult<bool> {
        self.with_scopes(|access| Ok(access.all_bindings_local(scope)?))
    }

    pub(super) fn scope_arguments_object_accessed(
        &mut self,
        scopes: &FunctionScopes,
    ) -> JsResult<bool> {
        self.with_scopes(|access| Ok(access.arguments_object_accessed(scopes)?))
    }

    pub(super) fn scope_binding_reference(
        &mut self,
        scope: &ScopeHandle,
        name: &JsString,
    ) -> JsResult<Option<IdentifierReference>> {
        self.with_scopes(|access| Ok(access.get_binding_reference(scope, name)?))
    }

    pub(super) fn scope_identifier_reference(
        &mut self,
        scope: &ScopeHandle,
        name: JsString,
    ) -> JsResult<IdentifierReference> {
        self.with_scopes(|access| Ok(access.get_identifier_reference(scope, name)?))
    }

    pub(super) fn scope_is_binding_mutable(
        &mut self,
        scope: &ScopeHandle,
        name: &JsString,
    ) -> JsResult<Option<bool>> {
        self.with_scopes(|access| Ok(access.is_binding_mutable(scope, name)?))
    }

    pub(super) fn scope_set_mutable_binding(
        &mut self,
        scope: &ScopeHandle,
        name: JsString,
    ) -> JsResult<Result<IdentifierReference, BindingLocatorError>> {
        self.with_scopes(|access| Ok(access.set_mutable_binding(scope, name)?))
    }

    #[cfg(feature = "annex-b")]
    pub(super) fn scope_set_mutable_binding_var(
        &mut self,
        scope: &ScopeHandle,
        name: JsString,
    ) -> JsResult<Result<IdentifierReference, BindingLocatorError>> {
        self.with_scopes(|access| Ok(access.set_mutable_binding_var(scope, name)?))
    }

    pub(super) fn lexical_identifier_reference(
        &mut self,
        name: JsString,
    ) -> JsResult<IdentifierReference> {
        self.scope_identifier_reference(&self.lexical_scope.clone(), name)
    }

    pub(super) fn variable_identifier_reference(
        &mut self,
        name: JsString,
    ) -> JsResult<IdentifierReference> {
        self.scope_identifier_reference(&self.variable_scope.clone(), name)
    }

    pub(super) fn lexical_set_mutable_binding(
        &mut self,
        name: JsString,
    ) -> JsResult<Result<IdentifierReference, BindingLocatorError>> {
        self.scope_set_mutable_binding(&self.lexical_scope.clone(), name)
    }

    #[cfg(feature = "annex-b")]
    pub(super) fn variable_set_mutable_binding_var(
        &mut self,
        name: JsString,
    ) -> JsResult<Result<IdentifierReference, BindingLocatorError>> {
        self.scope_set_mutable_binding_var(&self.variable_scope.clone(), name)
    }

    pub(super) fn lexical_is_binding_mutable(&mut self, name: &JsString) -> JsResult<Option<bool>> {
        self.scope_is_binding_mutable(&self.lexical_scope.clone(), name)
    }

    pub(super) fn new_scope(
        &mut self,
        parent: ScopeHandle,
        function: bool,
    ) -> JsResult<ScopeHandle> {
        self.with_scopes(|access| Ok(access.new_scope(parent, function)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{JsError, SpannedSourceText, js_string, vm::source_info::SourcePath};
    use surgeist_js_ast::scope::{PreparedScopes, ScopeAnalysisError, ScopeWork};
    use surgeist_js_interner::Interner;

    #[test]
    fn refused_existing_binding_lookup_preserves_the_limit_with_a_live_register() {
        let root = Scope::new_global();
        let original = root.create_mutable_binding(js_string!("visible"), false);
        let mut store = PreparedScopes::new(&root);
        let handle = store.root_handle();
        let mut extension = store
            .begin_extension::<FrontendLimit>(None)
            .expect("extension");
        let mut interner = Interner::new();
        let refusal = FrontendLimit::Scope(17);
        let mut deny = |work| {
            if work == ScopeWork::BindingProbe {
                Err(refusal)
            } else {
                Ok(())
            }
        };
        let result = {
            let mut compiler = ByteCompiler::new(
                js_string!("test"),
                false,
                false,
                handle.clone(),
                handle,
                false,
                false,
                &mut interner,
                false,
                SpannedSourceText::new_empty(),
                SourcePath::None,
                CompilerScopes::Pending(&mut extension),
                Some(&mut deny),
            );
            let _temporary = compiler.register_allocator.alloc();
            compiler.lexical_identifier_reference(js_string!("visible"))
        };
        assert_eq!(
            result,
            Err(JsError::from(ScopeAnalysisError::Refused(refusal)))
        );
        drop(extension);
        assert_eq!(root.num_bindings(), 1);
        assert_eq!(root.get_binding(&js_string!("visible")), Some(original));
    }

    #[test]
    fn refused_compiler_extension_preserves_ids_and_retry_uses_the_canonical_root() {
        fn compile_default_scope(
            store: &mut PreparedScopes,
            mut admission: ScopeAdmission<'_, FrontendLimit>,
        ) -> JsResult<crate::vm::CodeBlock> {
            let handle = store.root_handle();
            let mut extension = store.begin_extension::<FrontendLimit>(None)?;
            let mut interner = Interner::new();
            let controlled = admission.is_some();
            let mut forward = |work| match admission.as_mut() {
                Some(admit) => admit(work),
                None => Ok(()),
            };
            let callback: ScopeAdmission<'_, FrontendLimit> =
                if controlled { Some(&mut forward) } else { None };
            let mut compiler = ByteCompiler::new(
                js_string!("default"),
                true,
                false,
                handle.clone(),
                handle,
                false,
                false,
                &mut interner,
                false,
                SpannedSourceText::new_empty(),
                SourcePath::None,
                CompilerScopes::Pending(&mut extension),
                callback,
            );
            let temporary = compiler.register_allocator.alloc();
            let scope = compiler.new_scope(compiler.lexical_scope.clone(), true)?;
            let _ = compiler.push_scope(&scope)?;
            compiler.register_allocator.dealloc(temporary);
            let code = compiler.finish()?;
            extension.commit_private();
            Ok(code)
        }

        let root = Scope::new_global();
        let mut store = PreparedScopes::new(&root);
        let refusal = FrontendLimit::Scope(29);
        let mut deny = |work| {
            if work == ScopeWork::ScopeWrite {
                Err(refusal)
            } else {
                Ok(())
            }
        };
        assert_eq!(
            compile_default_scope(&mut store, Some(&mut deny)).expect_err("scope creation refused"),
            JsError::from(ScopeAnalysisError::Refused(refusal)),
        );
        assert_eq!(root, Scope::new_global());

        let code = compile_default_scope(&mut store, None).expect("retry compilation");
        assert!(code.scope_association.installed_scopes().is_err());
        let installed = store
            .prepare_publication::<FrontendLimit>(None)
            .expect("publication admission")
            .commit()
            .expect("publication");
        let associated_root = code
            .scope_association
            .resolve_live(&installed)
            .expect("installed root");
        let _ = associated_root.create_mutable_binding(js_string!("canonical"), false);
        assert!(root.has_binding(&js_string!("canonical")));
        assert_eq!(Scope::new(root, false).unique_id(), 2);
    }
}
