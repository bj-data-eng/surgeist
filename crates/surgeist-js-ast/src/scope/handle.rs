use super::{Scope, ScopeStateError};
use std::{cell::Cell, fmt, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum AssociationState {
    Pending,
    Installed,
    Aborted,
}

#[derive(Debug)]
pub(super) struct ScopeAssociation {
    pub(super) state: Cell<AssociationState>,
}

impl ScopeAssociation {
    pub(super) fn pending() -> Rc<Self> {
        Rc::new(Self {
            state: Cell::new(AssociationState::Pending),
        })
    }
}

struct AnalyzedHandle {
    scope: Scope,
    association: Option<Rc<ScopeAssociation>>,
    valid: Cell<bool>,
    outer: Option<ScopeHandle>,
}

enum HandleData {
    Unanalyzed,
    Analyzed(AnalyzedHandle),
}

thread_local! {
    static UNANALYZED_SCOPE: Rc<HandleData> = Rc::new(HandleData::Unanalyzed);
}

/// An opaque reference to an AST scope and its preparation association.
///
/// Pending references can be inspected only through their owning prepared view.
/// Equality compares installed scope values structurally, matching [`Scope`].
/// Pending values compare only within the same owner and canonical scope. This
/// phase-sensitive equality does not express global identity and is not hashable.
/// Equal values do not grant access through another preparation's association.
#[derive(Clone)]
pub struct ScopeHandle(Rc<HandleData>);

impl Default for ScopeHandle {
    fn default() -> Self {
        UNANALYZED_SCOPE.with(|scope| Self(scope.clone()))
    }
}

impl ScopeHandle {
    fn data(&self) -> Option<&AnalyzedHandle> {
        match &*self.0 {
            HandleData::Unanalyzed => None,
            HandleData::Analyzed(data) => Some(data),
        }
    }

    pub(super) fn live(scope: Scope) -> Self {
        Self(Rc::new(HandleData::Analyzed(AnalyzedHandle {
            scope,
            association: None,
            valid: Cell::new(true),
            outer: None,
        })))
    }

    pub(super) fn pending(
        scope: Scope,
        association: Rc<ScopeAssociation>,
        outer: Option<Self>,
    ) -> Self {
        Self(Rc::new(HandleData::Analyzed(AnalyzedHandle {
            scope,
            association: Some(association),
            valid: Cell::new(true),
            outer,
        })))
    }

    pub(super) fn associated_with(&self, owner: Option<&Rc<ScopeAssociation>>) -> bool {
        match (
            self.data().and_then(|data| data.association.as_ref()),
            owner,
        ) {
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        }
    }

    pub(super) fn invalidate(&self) {
        if let Some(data) = self.data() {
            data.valid.set(false);
        }
    }

    pub(super) fn scope_for(
        &self,
        owner: Option<&Rc<ScopeAssociation>>,
    ) -> Result<&Scope, ScopeStateError> {
        let data = self.data().ok_or(ScopeStateError::Unprepared)?;
        if !data.valid.get() {
            return Err(ScopeStateError::AssociationMismatch);
        }
        if let Some(association) = &data.association {
            match association.state.get() {
                AssociationState::Installed => {}
                AssociationState::Pending
                    if owner.is_some_and(|owner| Rc::ptr_eq(owner, association)) => {}
                AssociationState::Pending => {
                    return Err(if owner.is_some() {
                        ScopeStateError::AssociationMismatch
                    } else {
                        ScopeStateError::Unprepared
                    });
                }
                AssociationState::Aborted => return Err(ScopeStateError::AssociationMismatch),
            }
        }
        Ok(&data.scope)
    }

    pub(crate) fn raw(&self) -> &Scope {
        &self.data().expect("analyzed scope handle").scope
    }

    /// Obtains the installed association for an already published handle.
    ///
    /// # Errors
    /// Returns a state error for an unprepared or aborted handle.
    pub fn installed_scopes(&self) -> Result<InstalledScopes, ScopeStateError> {
        let scope = self.scope_for(None)?;
        Ok(InstalledScopes {
            association: self.data().and_then(|data| data.association.clone()),
            root: scope.clone(),
        })
    }

    /// Resolves an installed handle through its matching association.
    ///
    /// # Errors
    /// Returns a state error when either the handle or association is unsuitable.
    pub fn resolve_live(&self, installed: &InstalledScopes) -> Result<Scope, ScopeStateError> {
        let scope = self.scope_for(None)?;
        let data = self.data().ok_or(ScopeStateError::Unprepared)?;
        let matches = match (&data.association, &installed.association) {
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            (None, None) => Rc::ptr_eq(&scope.inner.context, &installed.root.inner.context),
            _ => false,
        };
        if !matches {
            return Err(ScopeStateError::AssociationMismatch);
        }
        Ok(scope.clone())
    }

    pub(crate) fn is_function(&self) -> bool {
        self.raw().is_function()
    }

    pub(crate) fn outer(&self) -> Option<Self> {
        let data = self.data()?;
        if let Some(outer) = &data.outer {
            return Some(outer.clone());
        }
        data.scope.outer().cloned().map(|scope| {
            data.association.as_ref().map_or_else(
                || Self::live(scope.clone()),
                |owner| Self::pending(scope.clone(), owner.clone(), None),
            )
        })
    }
}

impl fmt::Debug for ScopeHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let state = self.data().map(|data| {
            if !data.valid.get() {
                AssociationState::Aborted
            } else {
                data.association
                    .as_ref()
                    .map_or(AssociationState::Installed, |owner| owner.state.get())
            }
        });
        f.debug_struct("ScopeHandle")
            .field("state", &state)
            .finish()
    }
}

impl PartialEq for ScopeHandle {
    fn eq(&self, other: &Self) -> bool {
        match (self.data(), other.data()) {
            (Some(a), Some(b)) => {
                if !a.valid.get() || !b.valid.get() {
                    return Rc::ptr_eq(&self.0, &other.0);
                }
                let installed = |data: &AnalyzedHandle| {
                    data.association
                        .as_ref()
                        .is_none_or(|owner| owner.state.get() == AssociationState::Installed)
                };
                match (installed(a), installed(b)) {
                    (true, true) => a.scope == b.scope,
                    (false, false) => {
                        Rc::ptr_eq(&a.scope.inner, &b.scope.inner)
                            && match (&a.association, &b.association) {
                                (Some(a), Some(b)) => Rc::ptr_eq(a, b),
                                _ => false,
                            }
                    }
                    _ => false,
                }
            }
            (None, None) => true,
            _ => false,
        }
    }
}

#[cfg(feature = "arbitrary")]
impl<'a> arbitrary::Arbitrary<'a> for ScopeHandle {
    fn arbitrary(_u: &mut arbitrary::Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(Self::default())
    }
}

/// Proof that a scope preparation has published its canonical metadata.
#[derive(Clone, Debug)]
pub struct InstalledScopes {
    pub(super) association: Option<Rc<ScopeAssociation>>,
    pub(super) root: Scope,
}

impl Scope {
    /// Obtains an opaque reference to this live scope.
    #[must_use]
    pub fn handle(&self) -> ScopeHandle {
        ScopeHandle::live(self.clone())
    }

    /// Obtains the live association of an existing scope tree.
    #[must_use]
    pub fn installed_scopes(&self) -> InstalledScopes {
        InstalledScopes {
            association: None,
            root: self.clone(),
        }
    }
}
