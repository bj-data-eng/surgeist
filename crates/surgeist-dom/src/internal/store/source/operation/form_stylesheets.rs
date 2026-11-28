//! Style removals inside an accepted, still-private Form reset.

use super::*;
use crate::internal::hosted::DomHostedError;
use surgeist_cssom::{CssomCommit, CssomCommitRetirement, CssomEditGuard, CssomError};

/// The original Form Source phase with stylesheet removals and no hosted handoffs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomSourceFormStylesheetsRequest {
    operation: DomSourceOperationId,
}

/// Refusal before any Form control, stylesheet or hosted handoff has published.
#[derive(Debug)]
pub enum DomFormResetPublicationError {
    /// The canonical Source or DOM refused the reset body.
    Dom(DomError),
    /// The hosted issuer or handoff refused a reached child removal.
    Hosted(DomHostedError),
    /// The original stylesheet owner refused its portion of the reset body.
    Cssom(CssomError),
}

impl std::fmt::Display for DomFormResetPublicationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dom(error) => error.fmt(f),
            Self::Hosted(error) => error.fmt(f),
            Self::Cssom(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for DomFormResetPublicationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Dom(error) => Some(error),
            Self::Hosted(error) => Some(error),
            Self::Cssom(error) => Some(error),
        }
    }
}

impl From<DomError> for DomFormResetPublicationError {
    fn from(error: DomError) -> Self {
        Self::Dom(error)
    }
}

impl From<DomHostedError> for DomFormResetPublicationError {
    fn from(error: DomHostedError) -> Self {
        Self::Hosted(error)
    }
}

#[derive(Default)]
pub(super) struct Stylesheets {
    sheets: Vec<DomStyleSheet>,
    pending: bool,
    charge: Option<checkpoint::Charge>,
}

impl Stylesheets {
    pub(super) fn is_empty(&self) -> bool {
        self.sheets.is_empty()
    }
}

impl Operation {
    pub(super) fn collect_form_stylesheet(&mut self, cancel: &AtomicBool) -> Result<(), DomError> {
        let mut frame = self.frame()?;
        if frame.cursor.action() != DomStyleAction::Update(Action::RemoveAssociatedSheet)
            || self.draft.shadow_connected(frame.key, cancel)?
        {
            return Err(DomError::StyleCompositionRequired(
                Action::RemoveAssociatedSheet,
            ));
        }
        let element = self
            .draft
            .published_id(frame.key)
            .ok_or(DomError::StyleReceiptMismatch)?;
        let sheet = self
            .draft
            .element(frame.key)?
            .language()
            .style_state()
            .and_then(|state| state.sheet.as_ref())
            .filter(|sheet| sheet.association().element() == element)
            .ok_or(DomError::StyleReceiptMismatch)?
            .clone();
        frame
            .cursor
            .reply(Reply::Applied)
            .map_err(|_| DomError::StyleProtocol)?;
        // Both writes are in the private form journal. A refused write restores
        // the reached cursor and its original association before any retry.
        let mut state = self.draft.style_state_mut(frame.key, cancel)?;
        state.sheet = None;
        state.commit(cancel)?;
        self.draft.source_set_style_frame(frame, cancel)?;
        let formation = formation::Formation::new(&self.draft, cancel)?;
        let stylesheets = &mut self
            .reset_body
            .as_mut()
            .ok_or(DomError::InvalidSourceOperation)?
            .stylesheets;
        formation::reserve(&mut stylesheets.sheets, 1, formation.as_ref(), cancel)?;
        formation::merge_charge(
            &mut stylesheets.charge,
            formation.map(formation::Formation::into_charge),
        );
        stylesheets.sheets.push(sheet);
        Ok(())
    }

    pub(super) fn form_stylesheets_request(&mut self) -> Option<DomSourceFormStylesheetsRequest> {
        let stylesheets = &mut self.reset_body.as_mut()?.stylesheets;
        if stylesheets.is_empty() {
            return None;
        }
        stylesheets.pending = true;
        Some(DomSourceFormStylesheetsRequest { operation: self.id })
    }

    pub(super) fn pending_form_stylesheets(&self) -> Option<DomSourceFormStylesheetsRequest> {
        self.reset_body
            .as_ref()?
            .stylesheets
            .pending
            .then_some(DomSourceFormStylesheetsRequest { operation: self.id })
    }

    /// Hold the registered writers until all prepared owners have published.
    /// The supplied private continuation does every remaining fallible step
    /// before publishing its DOM segment and any reserved hosted handoffs.
    pub(super) fn with_form_stylesheets<T>(
        &mut self,
        dom: &mut Dom,
        cancel: &AtomicBool,
        publish: impl FnOnce(&mut Self, &mut Dom) -> Result<T, DomFormResetPublicationError>,
    ) -> Result<T, DomFormResetPublicationError> {
        let sheets = self
            .reset_body
            .as_ref()
            .map(|body| body.stylesheets.sheets.as_slice())
            .unwrap_or_default();
        if sheets.is_empty() {
            return publish(self, dom);
        }
        let budget = formation::Formation::new(&self.draft, cancel)?;
        let mut roots = Vec::<DomStyleCssomRoot>::new();
        for sheet in sheets {
            let mut found = false;
            for owner in &roots {
                self.draft.source_step(cancel)?;
                if owner.root() == sheet.root() {
                    found = true;
                    break;
                }
            }
            if !found {
                self.draft.source_step(cancel)?;
                let owner = dom
                    .style_cssom_roots
                    .get(sheet.root())
                    .ok_or(DomError::StyleReceiptMismatch)?;
                formation::reserve(&mut roots, 1, budget.as_ref(), cancel)?;
                roots.push(owner.duplicate());
            }
        }
        let mut writers = Vec::new();
        formation::reserve(&mut writers, roots.len(), budget.as_ref(), cancel)?;
        for owner in &roots {
            self.draft.source_step(cancel)?;
            writers.push(
                owner
                    .lock()
                    .map_err(DomStyleCssomRootError::into_dom_error)?,
            );
        }

        let mut prepared = Vec::new();
        formation::reserve(&mut prepared, roots.len(), budget.as_ref(), cancel)?;
        let mut retirement = Vec::<(CssomCommit, CssomCommitRetirement)>::new();
        formation::reserve(&mut retirement, roots.len(), budget.as_ref(), cancel)?;
        for (owner, store) in roots.iter().zip(&mut writers) {
            let root = owner.root();
            let snapshot = store.snapshot();
            let mut batch = store
                .batch(CssomEditGuard::from_snapshot(&snapshot))
                .map_err(DomFormResetPublicationError::Cssom)?;
            for sheet in sheets {
                self.draft.source_step(cancel)?;
                if sheet.root() == root {
                    if sheet.revision().owner() != snapshot.owner() {
                        return Err(DomError::StyleReceiptMismatch.into());
                    }
                    batch
                        .remove_sheet(sheet.id())
                        .map_err(DomFormResetPublicationError::Cssom)?;
                }
            }
            prepared.push(
                store
                    .prepare_commit(batch)
                    .map_err(DomFormResetPublicationError::Cssom)?,
            );
        }
        let result = publish(self, dom)?;
        for edit in prepared.drain(..) {
            retirement.push(edit.publish());
        }
        drop(prepared);
        drop(writers);
        drop(retirement);
        Ok(result)
    }

    pub(super) fn complete_form_reset(&mut self, commit: DomCommit) -> DomSourceCompletion {
        DomSourceCompletion {
            operation: self.id,
            cancelled: self.cancelled,
            published: std::mem::take(&mut self.historical),
            commit,
            installation: None,
            custom_element_disposition: None,
            fullscreen_exit_return: None,
            design_mode_range_disposition: None,
            dialog_toggle_disposition: None,
        }
    }
}

impl DomSourceLease<'_> {
    /// Publish the original Form's final DOM segment and all reached sheets.
    /// Binding and producer admission precede the final cancellation check.
    pub fn publish_form_stylesheets(
        &mut self,
        request: DomSourceFormStylesheetsRequest,
        cancel: &AtomicBool,
    ) -> Result<DomSourceCompletion, DomFormResetPublicationError> {
        let mut operation = self
            .dom
            .source_operation
            .take()
            .ok_or(DomError::InvalidSourceOperation)?;
        let result = (|| {
            if self.complete
                || request.operation != self.id
                || operation.cancelled
                || operation.pending_form_stylesheets() != Some(request)
            {
                return Err(DomError::InvalidSourceOperation.into());
            }
            operation.refresh_admission(cancel)?;
            let input_bytes = operation.input_bytes()?;
            let checkpoint = operation.draft.source_checkpoint(input_bytes, 0, cancel)?;
            let result = operation.with_form_stylesheets(self.dom, cancel, |op, dom| {
                let segment = op
                    .prepare_segment(dom, false, PublicationTarget::None, None, cancel)
                    .map_err(|error| match error {
                        DomHostedError::Dom(error) => DomFormResetPublicationError::Dom(error),
                        error => DomFormResetPublicationError::Hosted(error),
                    })?;
                if let Some(recording) = segment.prepared.inline_recording.as_ref() {
                    recording.require_unreached(|| op.draft.source_step(cancel))?;
                }
                op.draft.source_step(cancel)?;
                let commit = segment.publish(op, dom).commit;
                Ok(op.complete_form_reset(commit))
            });
            if result.is_err() {
                checkpoint.restore(&mut operation.draft);
            }
            result
        })();
        if result.is_ok() {
            self.complete = true;
            self.dom.source_operation = self.dom.source_parents.pop();
            #[cfg(test)]
            self.dom.observe_source_lifecycle();
        } else {
            self.dom.source_operation = Some(operation);
        }
        result
    }
}
