//! Eval preparation over explicit caller scopes.

use super::Script;
use crate::{
    scope::{
        PreparedScopes, Scope, ScopeAccess, ScopeAdmission, ScopeAnalysisError, ScopeHandle,
        ScopeStateError, ScopeWork,
    },
    scope_analyzer::{
        EvalDeclarationBindings, analyze_binding_escapes_at, collect_bindings_at,
        eval_declaration_instantiation_in_session, optimize_scope_indices_at,
    },
};
use surgeist_js_interner::{Interner, Sym};

/// Eval source, declaration metadata and its unpublished scope analysis.
#[derive(Debug)]
pub struct PreparedEval {
    tree: Script,
    strict: bool,
    annex_b_function_names: Vec<Sym>,
    bindings: EvalDeclarationBindings,
    variable_scope: ScopeHandle,
    lexical_scope: ScopeHandle,
    scopes: PreparedScopes,
    ready: bool,
}

/// Disjoint eval compilation input and private scope owner.
pub struct EvalParts<'a> {
    /// Immutable analyzed eval syntax.
    pub tree: &'a Script,
    /// Prepared declaration locators, retained for a later compilation retry.
    pub bindings: &'a EvalDeclarationBindings,
    /// Effective variable scope; strict eval uses its private lexical child.
    pub variable_scope: &'a ScopeHandle,
    /// The private eval lexical child of the caller's lexical parent.
    pub lexical_scope: &'a ScopeHandle,
    /// Canonical private analysis and compiler-extension storage.
    pub scopes: &'a mut PreparedScopes,
}

fn analyze_eval<E>(
    tree: &mut Script,
    strict: bool,
    variable_parent: &Scope,
    annex_b_function_names: &[Sym],
    interner: &Interner,
    access: &mut ScopeAccess<'_, E>,
) -> Result<(EvalDeclarationBindings, ScopeHandle, ScopeHandle), ScopeAnalysisError<E>> {
    let parent = access.root_handle()?;
    let lexical = access.new_scope(parent, strict)?;
    let variable = if strict {
        lexical.clone()
    } else {
        access.associate(&variable_parent.handle())?
    };
    let bindings = eval_declaration_instantiation_in_session(
        tree,
        strict,
        &variable,
        &lexical,
        annex_b_function_names,
        interner,
        access,
    )?;
    collect_bindings_at(tree, strict, true, lexical.clone(), interner, access)?;
    analyze_binding_escapes_at(tree, true, lexical.clone(), interner, access)?;
    access.escape_all_bindings(&variable)?;
    access.escape_all_bindings(&lexical)?;
    access.reorder_binding_indices(&variable)?;
    access.reorder_binding_indices(&lexical)?;
    optimize_scope_indices_at(tree, &lexical, access)?;
    Ok((bindings, variable, lexical))
}

impl Script {
    /// Prepares eval scopes without modifying caller bindings or shared identities.
    ///
    /// The caller supplies effective strictness, both enclosing scope authorities and
    /// eligible Annex B function names. Environment poisoning and guest effects belong
    /// to evaluation after admitted publication.
    ///
    /// # Errors
    /// Returns syntax errors, the original admission refusal, or a scope-state error.
    pub fn try_prepare_eval<E>(
        mut self,
        strict: bool,
        variable_parent: &Scope,
        lexical_parent: &Scope,
        annex_b_function_names: &[Sym],
        interner: &Interner,
        admission: ScopeAdmission<'_, E>,
    ) -> Result<PreparedEval, ScopeAnalysisError<E>> {
        let mut scopes = PreparedScopes::new(lexical_parent);
        let (names, bindings, variable_scope, lexical_scope) =
            scopes.with_access(admission, |access| {
                access.admit(ScopeWork::Capacity {
                    elements: annex_b_function_names.len(),
                })?;
                let mut names = Vec::new();
                names
                    .try_reserve(annex_b_function_names.len())
                    .map_err(|_| ScopeAnalysisError::State(ScopeStateError::Allocation))?;
                for name in annex_b_function_names {
                    access.admit(ScopeWork::Record)?;
                    names.push(*name);
                }
                let (bindings, variable, lexical) =
                    analyze_eval(&mut self, strict, variable_parent, &names, interner, access)?;
                Ok::<_, ScopeAnalysisError<E>>((names, bindings, variable, lexical))
            })?;
        Ok(PreparedEval {
            tree: self,
            strict,
            annex_b_function_names: names,
            bindings,
            variable_scope,
            lexical_scope,
            scopes,
            ready: true,
        })
    }
}

impl PreparedEval {
    /// Gets analyzed eval syntax.
    ///
    /// # Errors
    /// Returns Unprepared after an interrupted refresh.
    pub fn tree(&self) -> Result<&Script, ScopeStateError> {
        if self.ready {
            Ok(&self.tree)
        } else {
            Err(ScopeStateError::Unprepared)
        }
    }

    /// Gets the caller-supplied effective eval strictness.
    #[must_use]
    pub const fn strict(&self) -> bool {
        self.strict
    }

    /// Gets eligible Annex B names in their original order.
    #[must_use]
    pub fn annex_b_function_names(&self) -> &[Sym] {
        &self.annex_b_function_names
    }

    /// Gets the scope owner for freshness validation.
    #[must_use]
    pub const fn scopes(&self) -> &PreparedScopes {
        &self.scopes
    }

    /// Borrows immutable eval compilation input and mutable private scope storage.
    ///
    /// # Errors
    /// Returns Unprepared after an interrupted refresh.
    pub fn parts(&mut self) -> Result<EvalParts<'_>, ScopeStateError> {
        if !self.ready {
            return Err(ScopeStateError::Unprepared);
        }
        Ok(EvalParts {
            tree: &self.tree,
            bindings: &self.bindings,
            variable_scope: &self.variable_scope,
            lexical_scope: &self.lexical_scope,
            scopes: &mut self.scopes,
        })
    }

    /// Reanalyzes retained eval source against explicit current caller authorities.
    ///
    /// Refusal invalidates prior annotations and retains source for another refresh.
    ///
    /// # Errors
    /// Returns syntax errors, the original admission refusal, or a scope-state error.
    pub fn refresh<E>(
        &mut self,
        variable_parent: &Scope,
        lexical_parent: &Scope,
        interner: &Interner,
        admission: ScopeAdmission<'_, E>,
    ) -> Result<(), ScopeAnalysisError<E>> {
        self.ready = false;
        self.scopes.invalidate();
        self.scopes = PreparedScopes::new(lexical_parent);
        let (bindings, variable, lexical) = self.scopes.with_access(admission, |access| {
            super::reset::reset_scopes(&mut self.tree, access)?;
            analyze_eval(
                &mut self.tree,
                self.strict,
                variable_parent,
                &self.annex_b_function_names,
                interner,
                access,
            )
        })?;
        self.bindings = bindings;
        self.variable_scope = variable;
        self.lexical_scope = lexical;
        self.ready = true;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Declaration, LinearPosition, Span, Statement, StatementList,
        declaration::{LexicalDeclaration, VarDeclaration, Variable, VariableList},
        expression::Identifier,
    };
    use surgeist_js_string::JsString;

    fn eval_source(interner: &mut Interner) -> Script {
        let var = interner.get_or_intern("declared");
        let lexical = interner.get_or_intern("lexical");
        let variable = |name| {
            VariableList::new(Box::new([Variable::from_identifier(
                Identifier::new(name, Span::new((1, 1), (1, 2))),
                None,
            )]))
            .expect("one declaration")
        };
        Script::new(StatementList::new(
            vec![
                Statement::Var(VarDeclaration(variable(var))).into(),
                Declaration::Lexical(LexicalDeclaration::Let(variable(lexical))).into(),
            ],
            LinearPosition::new(31),
            false,
        ))
    }

    #[test]
    fn strict_eval_keeps_variable_bindings_in_its_private_child() {
        let root = Scope::new(Scope::new_global(), true);
        let mut interner = Interner::default();
        let mut prepared = eval_source(&mut interner)
            .try_prepare_eval::<()>(true, &root, &root, &[], &interner, None)
            .expect("prepare strict eval");
        assert_eq!(root.num_bindings(), 0);
        let parts = prepared.parts().expect("analyzed");
        assert_eq!(parts.variable_scope, parts.lexical_scope);
        let child = parts.lexical_scope.clone();
        assert_eq!(parts.bindings.declared_var_names.len(), 1);
        let installed = parts
            .scopes
            .prepare_publication::<()>(None)
            .expect("admit")
            .commit()
            .expect("publish");
        let child = child.resolve_live(&installed).expect("installed");
        assert!(child.has_binding(&JsString::from("declared")));
        assert!(child.has_binding(&JsString::from("lexical")));
        assert_eq!(root.num_bindings(), 0);
    }

    #[test]
    fn sloppy_eval_publishes_only_var_bindings_to_its_caller() {
        let root = Scope::new(Scope::new_global(), true);
        let mut interner = Interner::default();
        let mut prepared = eval_source(&mut interner)
            .try_prepare_eval::<()>(false, &root, &root, &[], &interner, None)
            .expect("prepare sloppy eval");
        assert_eq!(root.num_bindings(), 0);
        let parts = prepared.parts().expect("analyzed");
        let child = parts.lexical_scope.clone();
        let installed = parts
            .scopes
            .prepare_publication::<()>(None)
            .expect("admit")
            .commit()
            .expect("publish");
        assert!(root.has_binding(&JsString::from("declared")));
        assert!(!root.has_binding(&JsString::from("lexical")));
        assert!(!root.all_bindings_local());
        assert!(
            child
                .resolve_live(&installed)
                .expect("installed")
                .has_binding(&JsString::from("lexical"))
        );
    }

    #[test]
    fn every_analysis_refusal_leaves_caller_bindings_and_identities_unspent() {
        let mut interner = Interner::default();
        let source = eval_source(&mut interner);
        let calibration = Scope::new(Scope::new_global(), true);
        let mut events = 0;
        let mut accept = |_| {
            events += 1;
            Ok::<_, usize>(())
        };
        drop(
            source
                .clone()
                .try_prepare_eval(
                    false,
                    &calibration,
                    &calibration,
                    &[],
                    &interner,
                    Some(&mut accept),
                )
                .expect("calibrate"),
        );
        assert!(events > 0);
        for denied in 0..events {
            let root = Scope::new(Scope::new_global(), true);
            let mut seen = 0;
            let mut refuse = |_| {
                let index = seen;
                seen += 1;
                if index == denied { Err(denied) } else { Ok(()) }
            };
            assert!(matches!(source.clone().try_prepare_eval(
                false, &root, &root, &[], &interner, Some(&mut refuse),
            ), Err(ScopeAnalysisError::Refused(reason)) if reason == denied));
            assert_eq!(root.num_bindings(), 0);
            assert_eq!(Scope::new(root, false).unique_id(), 2);
        }
    }

    #[test]
    fn eval_conflict_retains_the_original_identifier_diagnostic() {
        let root = Scope::new_global();
        drop(root.create_mutable_binding(JsString::from("declared"), false));
        let mut interner = Interner::default();
        let result = eval_source(&mut interner).try_prepare_eval::<()>(
            false,
            &root,
            &root,
            &[],
            &interner,
            None,
        );
        assert!(matches!(result, Err(ScopeAnalysisError::Syntax(reason))
            if reason == "duplicate lexical declaration declared"));
        assert_eq!(root.num_bindings(), 1);
        assert_eq!(Scope::new(root, false).unique_id(), 1);
    }

    #[test]
    fn stale_eval_refresh_invalidates_old_child_and_preserves_callback_binding() {
        let root = Scope::new(Scope::new_global(), true);
        let mut interner = Interner::default();
        let mut prepared = eval_source(&mut interner)
            .try_prepare_eval::<()>(false, &root, &root, &[], &interner, None)
            .expect("prepare");
        let old = prepared.parts().expect("parts").lexical_scope.clone();
        drop(root.create_mutable_binding(JsString::from("external"), false));
        prepared
            .refresh::<()>(&root, &root, &interner, None)
            .expect("refresh");
        assert_eq!(
            old.installed_scopes().expect_err("invalidated"),
            ScopeStateError::AssociationMismatch
        );
        prepared
            .parts()
            .expect("fresh parts")
            .scopes
            .prepare_publication::<()>(None)
            .expect("admit")
            .commit()
            .expect("publish");
        assert!(root.has_binding(&JsString::from("external")));
        assert!(root.has_binding(&JsString::from("declared")));
        assert!(!root.has_binding(&JsString::from("lexical")));
    }
}
