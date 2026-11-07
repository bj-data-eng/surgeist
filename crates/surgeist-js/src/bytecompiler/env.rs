use super::ByteCompiler;
use crate::JsResult;
use surgeist_js_ast::scope::ScopeHandle;

impl ByteCompiler<'_, '_> {
    /// Push either a new declarative or function scope on the environment stack.
    pub(crate) fn push_scope(&mut self, scope: &ScopeHandle) -> JsResult<u32> {
        self.current_open_environments_count += 1;

        let index = self.push_scope_constant(scope)?;

        if self.scope_is_function(scope)? {
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
        let Some(scope) = scope else {
            return Ok(None);
        };
        let mut scope = scope.clone();
        if !self.scope_all_bindings_local(&scope)? {
            self.current_open_environments_count += 1;
            let index = self.push_scope_constant(&scope)?;
            self.bytecode.emit_push_scope(index.into());
        }
        std::mem::swap(&mut self.lexical_scope, &mut scope);
        Ok(Some(scope))
    }

    /// Pop a declarative scope.
    pub(crate) fn pop_declarative_scope(&mut self, scope: Option<ScopeHandle>) -> JsResult<()> {
        if let Some(mut scope) = scope {
            std::mem::swap(&mut self.lexical_scope, &mut scope);
            if !self.scope_all_bindings_local(&scope)? {
                self.current_open_environments_count -= 1;
                self.bytecode.emit_pop_environment();
            }
        }
        Ok(())
    }

    /// Pops the top scope.
    pub(crate) fn pop_scope(&mut self) {
        self.current_open_environments_count -= 1;
    }
}
