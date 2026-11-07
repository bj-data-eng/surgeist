//! Private preparation for dynamic callables and module source.

use super::Module;
use crate::{
    function::FunctionExpression,
    scope::{PreparedScopes, Scope, ScopeAdmission, ScopeAnalysisError},
    scope_analyzer::{
        analyze_binding_escapes_at, analyze_binding_escapes_in_session, collect_bindings_at,
        collect_bindings_in_session, optimize_scope_indices_at,
        optimize_scope_indices_function_constructor_in_session,
    },
};
use surgeist_js_interner::Interner;

/// Dynamic function syntax and its unpublished scope analysis.
#[derive(Debug)]
pub struct PreparedFunction {
    tree: FunctionExpression,
    scopes: PreparedScopes,
}

/// Disjoint dynamic-function compilation input and scope owner.
pub struct FunctionParts<'a> {
    /// Immutable syntax, preserving parameters, source and scope annotations.
    pub tree: &'a FunctionExpression,
    /// Private analysis and compiler-extension storage.
    pub scopes: &'a mut PreparedScopes,
}

impl FunctionExpression {
    /// Analyzes a dynamic constructor's function without publishing any scope changes.
    ///
    /// The caller supplies the constructor's strictness and enclosing scope.
    /// Function-family runtime policy remains with the caller.
    ///
    /// # Errors
    /// Returns syntax errors, the original admission refusal, or a scope-state error.
    pub fn try_prepare<E>(
        mut self,
        strict: bool,
        root: &Scope,
        interner: &Interner,
        admission: ScopeAdmission<'_, E>,
    ) -> Result<PreparedFunction, ScopeAnalysisError<E>> {
        let mut scopes = PreparedScopes::new(root);
        scopes.with_access(admission, |access| {
            collect_bindings_in_session(&mut self, strict, false, root, interner, access)?;
            analyze_binding_escapes_in_session(&mut self, false, root.clone(), interner, access)?;
            optimize_scope_indices_function_constructor_in_session(&mut self, root, access)
        })?;
        Ok(PreparedFunction { tree: self, scopes })
    }
}

impl PreparedFunction {
    /// Gets the analyzed syntax.
    #[must_use]
    pub const fn tree(&self) -> &FunctionExpression {
        &self.tree
    }

    /// Gets the scope owner for freshness validation.
    #[must_use]
    pub const fn scopes(&self) -> &PreparedScopes {
        &self.scopes
    }

    /// Borrows the immutable syntax and mutable private scope storage separately.
    pub const fn parts(&mut self) -> FunctionParts<'_> {
        FunctionParts {
            tree: &self.tree,
            scopes: &mut self.scopes,
        }
    }
}

impl Module {
    /// Analyzes one member using the group's canonical private scope authority.
    ///
    /// The explicit root may belong to a different realm from earlier members.
    /// Pending identities and edits remain in `scopes` until one group publication.
    /// Existing annotations are reset, preserving the source and template metadata,
    /// so retained source may be analyzed in a fresh group after a failed attempt.
    /// A failed member invalidates the group; discard it without publication.
    ///
    /// # Errors
    /// Returns syntax errors, the original admission refusal, or a scope-state error.
    pub fn try_prepare_in<E>(
        &mut self,
        root: &Scope,
        scopes: &mut PreparedScopes,
        interner: &Interner,
        admission: ScopeAdmission<'_, E>,
    ) -> Result<(), ScopeAnalysisError<E>> {
        let result = scopes.with_access(admission, |access| {
            super::reset::reset_scopes(self, access)?;
            let root = access.associate(&root.handle())?;
            collect_bindings_at(self, true, false, root.clone(), interner, access)?;
            analyze_binding_escapes_at(self, false, root, interner, access)?;
            let module_scope = self.scope().clone();
            optimize_scope_indices_at(self, &module_scope, access)
        });
        if let Err(error) = result {
            scopes.invalidate();
            return Err(error);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        LinearPosition, ModuleItem, Position, Span, Statement, StatementList,
        declaration::{VarDeclaration, Variable, VariableList},
        expression::Identifier,
        function::{FormalParameter, FormalParameterList, FunctionBody},
        scope::{BasisStatus, ScopeStateError, ScopeWork},
    };
    use surgeist_js_macros::utf16;
    use surgeist_js_string::JsString;

    fn variable(name: surgeist_js_interner::Sym) -> Statement {
        Statement::Var(VarDeclaration(
            VariableList::new(Box::new([Variable::from_identifier(
                Identifier::new(name, Span::new((1, 1), (1, 2))),
                None,
            )]))
            .expect("one variable"),
        ))
    }

    fn module(interner: &mut Interner) -> Module {
        Module::new(
            vec![ModuleItem::StatementListItem(
                variable(interner.get_or_intern_static("local", utf16!("local"))).into(),
            )]
            .into(),
        )
    }

    fn function(interner: &mut Interner) -> FunctionExpression {
        let parameter = interner.get_or_intern_static("parameter", utf16!("parameter"));
        let local = interner.get_or_intern_static("local", utf16!("local"));
        let span = Span::new(Position::new(1, 1), Position::new(1, 31));
        FunctionExpression::new(
            None,
            FormalParameterList::from_parameters(vec![FormalParameter::new(
                Variable::from_identifier(
                    Identifier::new(parameter, Span::new((1, 1), (1, 10))),
                    None,
                ),
                false,
            )]),
            FunctionBody::new(
                StatementList::new(vec![variable(local).into()], LinearPosition::new(31), false),
                span,
            ),
            None,
            false,
            span,
        )
    }

    #[test]
    fn unpublished_function_retains_parameters_and_spends_no_shared_identity_on_drop() {
        let root = Scope::new_global();
        let mut interner = Interner::default();
        let syntax = function(&mut interner);
        let parameters = syntax.parameters().clone();
        let prepared = syntax
            .try_prepare::<()>(false, &root, &interner, None)
            .expect("prepare");
        assert_eq!(prepared.tree().parameters(), &parameters);
        assert_eq!(root.num_bindings(), 0);
        let handle = prepared.tree().scopes().function_scope().clone();
        assert_eq!(
            handle.installed_scopes().expect_err("unpublished"),
            ScopeStateError::Unprepared
        );
        drop(prepared);
        assert_eq!(
            handle.installed_scopes().expect_err("aborted"),
            ScopeStateError::AssociationMismatch
        );
        assert_eq!(Scope::new(root, false).unique_id(), 1);
    }

    #[test]
    fn prepared_function_publication_installs_parameter_and_body_bindings() {
        let root = Scope::new_global();
        let mut interner = Interner::default();
        let mut prepared = function(&mut interner)
            .try_prepare::<()>(false, &root, &interner, None)
            .expect("prepare");
        let handle = prepared.tree().scopes().function_scope().clone();
        let installed = prepared
            .parts()
            .scopes
            .prepare_publication::<()>(None)
            .expect("admit")
            .commit()
            .expect("publish");
        let scope = handle.resolve_live(&installed).expect("installed child");
        assert!(scope.has_binding(&JsString::from("parameter")));
        assert!(scope.has_binding(&JsString::from("local")));
        assert_eq!(root.num_bindings(), 0);
    }

    #[test]
    fn module_refusal_preserves_callback_edits_and_identity_counter() {
        let root = Scope::new_global();
        let mut interner = Interner::default();
        let mut scopes = PreparedScopes::new(&root);
        let mut refuse = |work| {
            if work == ScopeWork::ScopeWrite {
                drop(root.create_mutable_binding(JsString::from("callback"), false));
                return Err("original stop");
            }
            Ok(())
        };
        let result =
            module(&mut interner).try_prepare_in(&root, &mut scopes, &interner, Some(&mut refuse));
        assert!(matches!(
            result,
            Err(ScopeAnalysisError::Refused("original stop"))
        ));
        assert!(root.has_binding(&JsString::from("callback")));
        assert!(!root.has_binding(&JsString::from("local")));
        assert!(matches!(
            scopes.prepare_publication::<()>(None),
            Err(ScopeAnalysisError::State(
                ScopeStateError::AssociationMismatch
            ))
        ));
        assert_eq!(Scope::new(root, false).unique_id(), 1);
    }

    #[test]
    fn module_group_uses_distinct_pending_identities_and_one_publication() {
        let root = Scope::new_global();
        let mut interner = Interner::default();
        let mut scopes = PreparedScopes::new(&root);
        let mut first = module(&mut interner);
        first
            .try_prepare_in::<()>(&root, &mut scopes, &interner, None)
            .expect("first");
        let mut second = module(&mut interner);
        second
            .try_prepare_in::<()>(&root, &mut scopes, &interner, None)
            .expect("second");
        scopes
            .with_access(
                None::<&mut dyn FnMut(ScopeWork) -> Result<(), ()>>,
                |access| {
                    let first_binding = access
                        .get_binding(first.scope(), &JsString::from("local"))?
                        .expect("first local");
                    let second_binding = access
                        .get_binding(second.scope(), &JsString::from("local"))?
                        .expect("second local");
                    assert_ne!(
                        first_binding.unique_scope_id(),
                        second_binding.unique_scope_id()
                    );
                    Ok::<_, ScopeAnalysisError<()>>(())
                },
            )
            .expect("inspect group");
        assert!(first.scope().installed_scopes().is_err());
        assert!(second.scope().installed_scopes().is_err());
        let installed = scopes
            .prepare_publication::<()>(None)
            .expect("admit")
            .commit()
            .expect("publish group");
        assert!(
            first
                .scope()
                .resolve_live(&installed)
                .expect("first installed")
                .has_binding(&JsString::from("local"))
        );
        assert!(
            second
                .scope()
                .resolve_live(&installed)
                .expect("second installed")
                .has_binding(&JsString::from("local"))
        );
        assert_eq!(Scope::new(root, false).unique_id(), 3);
    }

    #[test]
    fn group_conflict_keeps_external_effects_and_retained_module_reprepares() {
        let first_root = Scope::new_global();
        let second_root = Scope::new_global();
        let mut interner = Interner::default();
        let mut scopes = PreparedScopes::new(&first_root);
        let mut first = module(&mut interner);
        first
            .try_prepare_in::<()>(&first_root, &mut scopes, &interner, None)
            .expect("first");
        let mut second = module(&mut interner);
        second
            .try_prepare_in::<()>(&second_root, &mut scopes, &interner, None)
            .expect("second realm");
        let old_first = first.scope().clone();
        let old_second = second.scope().clone();
        let permit = scopes.prepare_publication::<()>(None).expect("admit");
        drop(second_root.create_mutable_binding(JsString::from("external"), false));
        assert!(matches!(permit.commit(), Err(ScopeStateError::Conflict)));
        assert_eq!(scopes.validate_basis::<()>(None), Ok(BasisStatus::Stale));
        drop(scopes);
        let mut fresh = PreparedScopes::new(&first_root);
        first
            .try_prepare_in::<()>(&first_root, &mut fresh, &interner, None)
            .expect("reprepare first");
        second
            .try_prepare_in::<()>(&second_root, &mut fresh, &interner, None)
            .expect("reprepare second");
        assert_eq!(
            old_first.installed_scopes().expect_err("old first"),
            ScopeStateError::AssociationMismatch
        );
        assert_eq!(
            old_second.installed_scopes().expect_err("old second"),
            ScopeStateError::AssociationMismatch
        );
        let installed = fresh
            .prepare_publication::<()>(None)
            .expect("admit")
            .commit()
            .expect("publish");
        assert_eq!(
            first
                .scope()
                .resolve_live(&installed)
                .expect("first")
                .unique_id(),
            1
        );
        assert_eq!(
            second
                .scope()
                .resolve_live(&installed)
                .expect("second")
                .unique_id(),
            1
        );
        assert!(second_root.has_binding(&JsString::from("external")));
        assert_eq!(first_root.num_bindings(), 0);
        assert_eq!(second_root.num_bindings(), 1);
    }
}
