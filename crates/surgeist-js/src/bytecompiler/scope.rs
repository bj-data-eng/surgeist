//! Scope authority borrowed by one compiler and its nested compilers.
use super::ByteCompiler;
use crate::{JsError, JsResult, JsString};
use surgeist_js_ast::scope::{
    BindingLocatorError, FunctionScopes, IdentifierReference, ScopeAccess, ScopeHandle,
};

impl ByteCompiler<'_, '_> {
    pub(crate) fn with_scopes<T>(
        &mut self,
        operation: impl FnOnce(&mut ScopeAccess<'_, JsError>) -> JsResult<T>,
    ) -> JsResult<T> {
        let scopes = &mut self.scopes;
        self.control
            .with_scope_admission(|admission| scopes.with_access(admission, operation))
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
        let index =
            u32::try_from(self.constants.len()).map_err(|_| super::control::size_overflow())?;
        super::control::thin_push(
            &mut self.constants,
            crate::vm::Constant::Scope(handle, display),
            &mut self.control,
        )?;
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
    use crate::bytecompiler::control::{
        CompilerAdmission, CompilerControl, CompilerDepth, CompilerWork,
    };
    use crate::{JsError, SpannedSourceText, js_string, vm::source_info::SourcePath};
    use surgeist_js_ast::scope::{
        PreparedScopes, Scope, ScopeAdmission, ScopeAnalysisError, ScopeWork,
    };
    use surgeist_js_interner::Interner;
    use surgeist_js_parser::frontend::FrontendLimit;

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
            if matches!(work, CompilerWork::Scope(ScopeWork::BindingProbe)) {
                Err(JsError::from(ScopeAnalysisError::Refused(refusal)))
            } else {
                Ok(())
            }
        };
        let depth = CompilerDepth::new();
        let result = (|| {
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
                &mut extension,
                CompilerControl::new(Some(&mut deny), &depth),
            )?;
            let _temporary = compiler.register_allocator.alloc(&mut compiler.control)?;
            compiler.lexical_identifier_reference(js_string!("visible"))
        })();
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
            let mut forward = |work| match (work, admission.as_mut()) {
                (CompilerWork::Scope(work), Some(admit)) => admit(work)
                    .map_err(|refusal| JsError::from(ScopeAnalysisError::Refused(refusal))),
                _ => Ok(()),
            };
            let callback: CompilerAdmission<'_> =
                if controlled { Some(&mut forward) } else { None };
            let depth = CompilerDepth::new();
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
                &mut extension,
                CompilerControl::new(callback, &depth),
            )?;
            let temporary = compiler.register_allocator.alloc(&mut compiler.control)?;
            let scope = compiler.new_scope(compiler.lexical_scope.clone(), true)?;
            let _ = compiler.push_scope(&scope)?;
            compiler
                .register_allocator
                .dealloc(temporary, &mut compiler.control)?;
            let code = compiler.finish(None)?;
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
