use super::Script;
use crate::{
    scope::{
        PreparedScopes, Scope, ScopeAccess, ScopeAdmission, ScopeAnalysisError, ScopeStateError,
        ScopeWork,
    },
    scope_analyzer::{
        analyze_binding_escapes_in_session, collect_bindings_in_session,
        optimize_scope_indices_in_session,
    },
};
use std::ops::{Deref, DerefMut};
use surgeist_js_interner::Interner;
use surgeist_js_string::JsString;

/// The source's original global declaration names, retained across preparation.
#[derive(Debug, Default)]
pub struct ScriptDeclarations {
    pub(crate) lexical: Vec<JsString>,
    pub(crate) var: Vec<JsString>,
    pub(crate) annex_b: Vec<JsString>,
}

impl ScriptDeclarations {
    /// Gets the source's ordered lexical declaration names.
    #[must_use]
    pub fn lexical_names(&self) -> &[JsString] {
        &self.lexical
    }
    /// Gets unique original var declaration names in first-declaration order.
    ///
    /// Function declaration names are included; repeated names occur only once.
    /// This order is retained for observable global declaration hooks.
    #[must_use]
    pub fn var_names(&self) -> &[JsString] {
        &self.var
    }

    /// Gets original eligible Annex B block-function candidates, in collector order.
    ///
    /// Repeated names are retained; live global eligibility belongs to evaluation.
    #[must_use]
    pub fn annex_b_candidates(&self) -> &[JsString] {
        &self.annex_b
    }

    /// Checks declaration conflicts at evaluation, without installing bindings.
    ///
    /// # Errors
    /// Returns an ordinary syntax conflict or admission/state failure.
    pub fn check_against_live<E>(
        &self,
        root: &Scope,
        admission: ScopeAdmission<'_, E>,
    ) -> Result<(), ScopeAnalysisError<E>> {
        ScopeAccess::with_live(root, admission, |access| {
            let handle = access.root_handle()?;
            for name in &self.lexical {
                access.admit(ScopeWork::Node)?;
                if access.has_binding(&handle, name)? {
                    return Err(ScopeAnalysisError::Syntax(
                        "duplicate lexical declaration".into(),
                    ));
                }
            }
            for name in &self.var {
                access.admit(ScopeWork::Node)?;
                if access.has_lex_binding(&handle, name)? {
                    return Err(ScopeAnalysisError::Syntax(
                        "duplicate lexical declaration".into(),
                    ));
                }
            }
            Ok(())
        })
    }
}

/// Syntax and privately analyzed scopes, with no retained caller policy.
#[derive(Debug)]
pub struct PreparedScript {
    tree: Script,
    declarations: ScriptDeclarations,
    scopes: PreparedScopes,
    ready: bool,
}

/// Disjoint compiler input and its private scope owner.
pub struct PreparedParts<'a> {
    /// Immutable syntax with opaque scope annotations.
    pub tree: &'a Script,
    /// Original declarations for evaluation-time validation.
    pub declarations: &'a ScriptDeclarations,
    /// The same private scope owner used by analysis and compilation.
    pub scopes: &'a mut PreparedScopes,
}

impl Script {
    /// Prepares syntax without reserving live global bindings.
    ///
    /// # Errors
    /// Returns scope analysis refusal, source errors, or scope state errors.
    pub fn try_prepare<E>(
        mut self,
        root: &Scope,
        interner: &Interner,
        admission: ScopeAdmission<'_, E>,
    ) -> Result<PreparedScript, ScopeAnalysisError<E>> {
        let mut scopes = PreparedScopes::for_script(root);
        let declarations = scopes.with_access(admission, |access| {
            let declarations = crate::scope_analyzer::script_declarations(&self, interner, access)?;
            analyze(&mut self, root, interner, access)?;
            Ok::<_, ScopeAnalysisError<E>>(declarations)
        })?;
        Ok(PreparedScript {
            tree: self,
            declarations,
            scopes,
            ready: true,
        })
    }
}

fn analyze<E>(
    tree: &mut Script,
    root: &Scope,
    interner: &Interner,
    access: &mut ScopeAccess<'_, E>,
) -> Result<(), ScopeAnalysisError<E>> {
    let strict = tree.strict();
    collect_bindings_in_session(tree, strict, false, root, interner, access)?;
    analyze_binding_escapes_in_session(tree, false, root.clone(), interner, access)?;
    optimize_scope_indices_in_session(tree, root, access)
}

impl PreparedScript {
    /// Gets analyzed immutable syntax.
    ///
    /// # Errors
    /// Returns Unprepared after mutable access or failed refresh.
    pub fn tree(&self) -> Result<&Script, ScopeStateError> {
        if self.ready {
            Ok(&self.tree)
        } else {
            Err(ScopeStateError::Unprepared)
        }
    }
    /// Gets original source declarations.
    #[must_use]
    pub fn declarations(&self) -> &ScriptDeclarations {
        &self.declarations
    }
    /// Transfers original declaration metadata while releasing syntax and private storage.
    #[must_use]
    pub fn into_declarations(self) -> ScriptDeclarations {
        self.declarations
    }
    /// Gets the preparation's scope owner.
    #[must_use]
    pub fn scopes(&self) -> &PreparedScopes {
        &self.scopes
    }

    /// Borrows disjoint compilation input.
    ///
    /// # Errors
    /// Returns Unprepared after mutable access or failed refresh.
    pub fn parts(&mut self) -> Result<PreparedParts<'_>, ScopeStateError> {
        if !self.ready {
            return Err(ScopeStateError::Unprepared);
        }
        Ok(PreparedParts {
            tree: &self.tree,
            declarations: &self.declarations,
            scopes: &mut self.scopes,
        })
    }

    /// Grants mutable syntax access and invalidates its previous pending analysis.
    ///
    /// # Errors
    /// Returns Unprepared unless the current analysis is complete.
    pub fn edit_tree(&mut self) -> Result<ScriptEdit<'_>, ScopeStateError> {
        if !self.ready {
            return Err(ScopeStateError::Unprepared);
        }
        self.ready = false;
        self.scopes.invalidate();
        Ok(ScriptEdit {
            tree: &mut self.tree,
        })
    }

    /// Resets annotations and analyzes against a fresh live basis.
    ///
    /// Refusal retains syntax in a state which can be refreshed again.
    ///
    /// # Errors
    /// Returns admission, syntax, or scope-state errors; never checks live declarations.
    pub fn refresh<E>(
        &mut self,
        root: &Scope,
        interner: &Interner,
        admission: ScopeAdmission<'_, E>,
    ) -> Result<(), ScopeAnalysisError<E>> {
        self.ready = false;
        self.scopes.invalidate();
        self.scopes = PreparedScopes::for_script(root);
        self.scopes.with_access(admission, |access| {
            super::reset::reset_scopes(&mut self.tree, access)?;
            analyze(&mut self.tree, root, interner, access)
        })?;
        self.ready = true;
        Ok(())
    }
}

/// An explicit syntax mutation capability; previous pending annotations are invalid.
pub struct ScriptEdit<'a> {
    tree: &'a mut Script,
}
impl Deref for ScriptEdit<'_> {
    type Target = Script;
    fn deref(&self) -> &Script {
        self.tree
    }
}
impl DerefMut for ScriptEdit<'_> {
    fn deref_mut(&mut self) -> &mut Script {
        self.tree
    }
}
