use super::{
    ByteCompiler,
    control::{CompilerWork, size_overflow},
};
use crate::JsResult;
use surgeist_js_ast::scope::ScopeHandle;

impl ByteCompiler<'_, '_> {
    /// Push either a new declarative or function scope on the environment stack.
    pub(crate) fn push_scope(&mut self, scope: &ScopeHandle) -> JsResult<u32> {
        self.control.admit(CompilerWork::MetadataVisit)?;
        let count = self
            .current_open_environments_count
            .checked_add(1)
            .ok_or_else(size_overflow)?;
        let index = self.push_scope_constant(scope)?;
        let function = self.scope_is_function(scope)?;
        self.control.admit(CompilerWork::MetadataWrite)?;
        self.current_open_environments_count = count;
        if function {
            self.variable_scope = scope.clone();
        }

        self.lexical_scope = scope.clone();

        Ok(index)
    }

    /// Push a declarative scope.
    ///
    /// Returns the outer scope.
    pub(crate) fn push_declarative_scope(
        &mut self,
        scope: Option<&ScopeHandle>,
    ) -> JsResult<Option<ScopeHandle>> {
        self.control.admit(CompilerWork::MetadataVisit)?;
        let Some(scope) = scope else {
            return Ok(None);
        };
        let mut scope = scope.clone();
        if !self.scope_all_bindings_local(&scope)? {
            let count = self
                .current_open_environments_count
                .checked_add(1)
                .ok_or_else(size_overflow)?;
            let index = self.push_scope_constant(&scope)?;
            self.control.admit(CompilerWork::MetadataWrite)?;
            self.bytecode
                .emit_push_scope(index.into(), &mut self.control)?;
            self.current_open_environments_count = count;
        } else {
            self.control.admit(CompilerWork::MetadataWrite)?;
        }
        std::mem::swap(&mut self.lexical_scope, &mut scope);
        Ok(Some(scope))
    }

    /// Pop a declarative scope.
    pub(crate) fn pop_declarative_scope(&mut self, scope: Option<ScopeHandle>) -> JsResult<()> {
        self.control.admit(CompilerWork::MetadataVisit)?;
        if let Some(mut scope) = scope {
            let current = self.lexical_scope.clone();
            let local = self.scope_all_bindings_local(&current)?;
            let count = if local {
                self.current_open_environments_count
            } else {
                self.current_open_environments_count
                    .checked_sub(1)
                    .ok_or_else(size_overflow)?
            };
            self.control.admit(CompilerWork::MetadataWrite)?;
            if !local {
                self.bytecode.emit_pop_environment(&mut self.control)?;
            }
            self.current_open_environments_count = count;
            std::mem::swap(&mut self.lexical_scope, &mut scope);
        }
        Ok(())
    }

    /// Pops the top scope.
    pub(crate) fn pop_scope(&mut self) -> JsResult<()> {
        self.control.admit(CompilerWork::MetadataVisit)?;
        let count = self
            .current_open_environments_count
            .checked_sub(1)
            .ok_or_else(size_overflow)?;
        self.control.admit(CompilerWork::MetadataWrite)?;
        self.current_open_environments_count = count;
        Ok(())
    }
}
