//! One-operation publication and explicitly staged object edits.
use crate::*;
use surgeist_css::CssRecoveryDiagnostic;

/// A successful CSSOM publication and its standard return value.
///
/// Owner effects still require synchronous application by the binding. A host
/// failure after this result is returned does not roll back CSSOM publication.
/// Dropping the result releases abandoned effect reservations; `must_use`
/// warns about discarded obligations but cannot enforce host delivery.
#[must_use = "complete or explicitly abandon the returned owner effects"]
#[derive(Debug)]
pub struct CssomOperationOutcome<T> {
    value: T,
    publication: CssomPublication,
    diagnostics: Vec<CssRecoveryDiagnostic>,
    effects: Vec<CssomOwnerEffect>,
}
impl<T> CssomOperationOutcome<T> {
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn publication(&self) -> &CssomPublication {
        &self.publication
    }
    pub fn diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        &self.diagnostics
    }
    pub fn owner_effects(&self) -> &[CssomOwnerEffect] {
        &self.effects
    }
    /// Consume the outcome so a binding can finish host work before returning
    /// the standard value. Effects retain their existing single-use lifecycle.
    pub fn into_parts(
        self,
    ) -> (
        T,
        CssomPublication,
        Vec<CssRecoveryDiagnostic>,
        Vec<CssomOwnerEffect>,
    ) {
        (self.value, self.publication, self.diagnostics, self.effects)
    }
    /// Existing multi-phase owners may wrap their already completed publication
    /// without staging or committing a second operation.
    pub(crate) fn from_parts(
        value: T,
        publication: CssomPublication,
        diagnostics: Vec<CssRecoveryDiagnostic>,
        effects: Vec<CssomOwnerEffect>,
    ) -> Self {
        Self {
            value,
            publication,
            diagnostics,
            effects,
        }
    }
}

/// An operation staged in a caller-owned product batch, not a publication.
///
/// Its value is provisional until the caller commits the batch. Creation
/// operations use batch tickets here, never tentative live object handles.
#[must_use = "this operation is staged; commit or abandon the owning batch"]
#[derive(Debug)]
pub struct CssomStagedOperation<T> {
    value: T,
    diagnostics: Vec<CssRecoveryDiagnostic>,
}
impl<T> CssomStagedOperation<T> {
    pub fn value(&self) -> &T {
        &self.value
    }
    pub fn diagnostics(&self) -> &[CssRecoveryDiagnostic] {
        &self.diagnostics
    }
    pub fn into_parts(self) -> (T, Vec<CssRecoveryDiagnostic>) {
        (self.value, self.diagnostics)
    }
    pub(crate) fn new(value: T, diagnostics: Vec<CssRecoveryDiagnostic>) -> Self {
        Self { value, diagnostics }
    }
    pub(crate) fn map<U>(self, map: impl FnOnce(T) -> U) -> CssomStagedOperation<U> {
        CssomStagedOperation::new(map(self.value), self.diagnostics)
    }
}

struct StagingGuard<'a> {
    batch: &'a mut CssomBatch,
    completed: bool,
}
impl Drop for StagingGuard<'_> {
    fn drop(&mut self) {
        if !self.completed {
            self.batch.poisoned = true;
        }
    }
}
/// Shared typed staging boundary. Unlike the historical low-level batch API,
/// a caught unwind from a supplied object resolver cannot leave a committable batch.
pub(crate) fn stage<T>(
    batch: &mut CssomBatch,
    operation: impl FnOnce(&mut CssomBatch) -> Result<CssomStagedOperation<T>, CssomError>,
) -> Result<CssomStagedOperation<T>, CssomError> {
    let mut guard = StagingGuard {
        batch,
        completed: false,
    };
    let result = guard.batch.apply(operation);
    guard.completed = result.is_ok();
    result
}
/// Construct every fallible result and diagnostic allocation before publication.
pub(crate) fn execute<T>(
    store: &mut CssomStore,
    operation: impl FnOnce(&mut CssomBatch) -> Result<CssomStagedOperation<T>, CssomError>,
) -> Result<CssomOperationOutcome<T>, CssomError> {
    let mut batch = store.batch(CssomEditGuard::from_snapshot(&store.snapshot()))?;
    let staged = stage(&mut batch, operation)?;
    let (publication, effects) = store.commit(batch)?.into_operation_parts();
    Ok(CssomOperationOutcome::from_parts(
        staged.value,
        publication,
        staged.diagnostics,
        effects,
    ))
}

macro_rules! no_op_css_text {
    ($handle:ident) => {
        impl $handle {
            /// The selected base CSSRule setter intentionally performs no edit.
            pub fn set_css_text(
                &self,
                store: &mut CssomStore,
                _text: &str,
            ) -> Result<CssomOperationOutcome<()>, CssomError> {
                execute(store, |batch| {
                    Self::from_state(&batch.staged, self.id())?;
                    Ok(CssomStagedOperation::new((), Vec::new()))
                })
            }
        }
    };
}
crate::objects::for_rule_handles!(no_op_css_text);
