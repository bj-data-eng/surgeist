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

/// Actual checked candidates and the context captured during their preparation.
/// The resolver receives no store, batch or host-write capability.
/// Resolver work should be pure; CSSOM does not roll back side effects on
/// external values a caller chooses to capture in its closure.
#[derive(Debug)]
pub struct CssomAdmissionRequest<'a> {
    candidates: &'a [CssomDeclarationCandidate],
    context: &'a CssomContext,
}
impl CssomAdmissionRequest<'_> {
    pub fn candidates(&self) -> &[CssomDeclarationCandidate] {
        self.candidates
    }
    pub fn context(&self) -> &CssomContext {
        self.context
    }
}

/// Supplied decisions and their independently current input versions.
///
/// Collect current versions after decision work. The owning declaration engine
/// validates completeness, identity, role and freshness when applying this packet;
/// construction alone does not assert that it belongs to a particular request.
#[derive(Debug)]
pub struct CssomAdmission {
    decisions: Vec<CssomDeclarationDecision>,
    current_inputs: Vec<CssomInputVersion>,
}
impl CssomAdmission {
    pub fn new(
        decisions: Vec<CssomDeclarationDecision>,
        current_inputs: Vec<CssomInputVersion>,
    ) -> Self {
        Self {
            decisions,
            current_inputs,
        }
    }
    pub fn decisions(&self) -> &[CssomDeclarationDecision] {
        &self.decisions
    }
    pub fn current_inputs(&self) -> &[CssomInputVersion] {
        &self.current_inputs
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

pub(crate) fn admit_prepared(
    batch: &mut CssomBatch,
    prepared: CssomPreparedDeclaration,
    resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>,
) -> Result<CssomStagedOperation<CssomDeclarationEditResult>, CssomError> {
    stage(batch, |batch| {
        let mut diagnostics = Vec::new();
        diagnostics
            .try_reserve_exact(prepared.diagnostics().len())
            .map_err(|_| CssomError::Limit {
                resource: "object operation diagnostics capacity",
                maximum: prepared.diagnostics().len(),
            })?;
        diagnostics.extend_from_slice(prepared.diagnostics());
        let admission = if prepared.candidates().is_empty() {
            let mut current_inputs = Vec::new();
            current_inputs
                .try_reserve_exact(prepared.captured_context().inputs.len())
                .map_err(|_| CssomError::Limit {
                    resource: "object operation input inventory capacity",
                    maximum: prepared.captured_context().inputs.len(),
                })?;
            current_inputs.extend(
                prepared
                    .captured_context()
                    .inputs
                    .iter()
                    .filter(|input| input.version.role == CssomInputRole::Support)
                    .map(|input| input.version.clone()),
            );
            CssomAdmission::new(Vec::new(), current_inputs)
        } else {
            resolver(&CssomAdmissionRequest {
                candidates: prepared.candidates(),
                context: prepared.captured_context(),
            })?
        };
        let value = batch.apply_declaration_with_inputs(
            prepared,
            &admission.decisions,
            &admission.current_inputs,
        )?;
        Ok(CssomStagedOperation::new(value, diagnostics))
    })
}

fn stage_remove(
    batch: &mut CssomBatch,
    object: &CssStyleDeclaration,
    name: &str,
    limits: CssomDeclarationRequestLimits,
) -> Result<CssomStagedOperation<String>, CssomError> {
    stage(batch, |batch| {
        CssStyleDeclaration::from_state(&batch.staged, object.id())?;
        Ok(CssomStagedOperation::new(
            batch.remove_property(object.id(), name, limits)?,
            Vec::new(),
        ))
    })
}
fn stage_set(
    batch: &mut CssomBatch,
    object: &CssStyleDeclaration,
    name: &str,
    value: &str,
    priority: Option<&str>,
    limits: CssomDeclarationRequestLimits,
    resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>,
) -> Result<CssomStagedOperation<()>, CssomError> {
    stage(batch, |batch| {
        CssStyleDeclaration::from_state(&batch.staged, object.id())?;
        let prepared =
            batch.prepare_set_property(object.id(), name, value, priority.unwrap_or(""), limits)?;
        Ok(admit_prepared(batch, prepared, resolver)?.map(|_| ()))
    })
}

impl CssStyleDeclaration {
    pub fn remove_property(
        &self,
        store: &mut CssomStore,
        name: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<CssomOperationOutcome<String>, CssomError> {
        execute(store, |batch| stage_remove(batch, self, name, limits))
    }
    /// Resolve only candidates requiring whole-declaration usability decisions.
    /// `None` priority is the standard empty priority string.
    /// The owner remains exclusively borrowed during admission:
    /// ```compile_fail
    /// use surgeist_cssom::*;
    /// fn reenter(object: &CssStyleDeclaration, store: &mut CssomStore) {
    ///     let _ = object.set_property(store, "width", "2px", None,
    ///         CssomDeclarationRequestLimits::default(), |_| {
    ///             let _ = store.snapshot();
    ///             Ok(CssomAdmission::new(Vec::new(), Vec::new()))
    ///         });
    /// }
    /// ```
    pub fn set_property(
        &self,
        store: &mut CssomStore,
        name: &str,
        value: &str,
        priority: Option<&str>,
        limits: CssomDeclarationRequestLimits,
        resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>,
    ) -> Result<CssomOperationOutcome<()>, CssomError> {
        execute(store, |batch| {
            stage_set(batch, self, name, value, priority, limits, resolver)
        })
    }
    /// Advanced product editing. This stages against the supplied batch and
    /// never automatically commits a standard object call.
    pub fn edit<'batch>(
        &self,
        batch: &'batch mut CssomBatch,
    ) -> Result<CssomDeclarationEdit<'batch>, CssomError> {
        let _ = stage(batch, |batch| {
            CssStyleDeclaration::from_state(&batch.staged, self.id())?;
            Ok(CssomStagedOperation::new((), Vec::new()))
        })?;
        Ok(CssomDeclarationEdit {
            object: self.clone(),
            batch,
        })
    }
}
/// A typed declaration editor borrowing an explicit caller-owned batch.
pub struct CssomDeclarationEdit<'batch> {
    pub(crate) object: CssStyleDeclaration,
    pub(crate) batch: &'batch mut CssomBatch,
}
impl CssomDeclarationEdit<'_> {
    pub fn remove_property(
        &mut self,
        name: &str,
        limits: CssomDeclarationRequestLimits,
    ) -> Result<CssomStagedOperation<String>, CssomError> {
        stage_remove(self.batch, &self.object, name, limits)
    }
    pub fn set_property(
        &mut self,
        name: &str,
        value: &str,
        priority: Option<&str>,
        limits: CssomDeclarationRequestLimits,
        resolver: impl FnOnce(&CssomAdmissionRequest<'_>) -> Result<CssomAdmission, CssomError>,
    ) -> Result<CssomStagedOperation<()>, CssomError> {
        stage_set(
            self.batch,
            &self.object,
            name,
            value,
            priority,
            limits,
            resolver,
        )
    }
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
