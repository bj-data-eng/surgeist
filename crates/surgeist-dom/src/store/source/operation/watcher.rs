//! Watcher results reuse the original operation receipt and Hook checkpoint.
use super::*;
use crate::{
    DomCloseWatcherId, DomDialogWatcherPreparation, DomDialogWatcherPublication, DomHostedSource,
};
use surgeist_html::{HtmlDialogAction, HtmlDialogReply};
impl Operation {
    pub(super) fn external_receipt_applied(&self) -> bool {
        matches!(
            self.receipt,
            Some(
                Receipt::Watcher { applied: true, .. }
                    | Receipt::DialogFocus { applied: true, .. }
                    | Receipt::CloseWatcher { applied: true, .. }
            )
        )
    }
    pub(super) fn restore_external_receipt(&mut self, value: bool) {
        if let Some(
            Receipt::Watcher { applied, .. }
            | Receipt::DialogFocus { applied, .. }
            | Receipt::CloseWatcher { applied, .. },
        ) = &mut self.receipt
        {
            *applied = value;
        }
    }

    fn watcher_frame(
        &self,
        request: &DomSourceDialogRequest,
    ) -> Result<crate::store::dialog::HookFrame, DomError> {
        if request.operation != self.id || request.hook().is_none() {
            return Err(DomError::InvalidSourceOperation);
        }
        let Some(select::Effect::DialogHook(frame)) =
            self.draft.effects.last().map(|frame| &frame.effect)
        else {
            return Err(DomError::InvalidSourceOperation);
        };
        let hook = request.hook().expect("qualified hook");
        if hook.command != self.index
            || !matches!(frame.invocation, Invocation::Exposed(actual) if actual == hook.ordinal)
            || self.draft.published_id(frame.subject) != Some(request.subject)
            || frame.cursor.action() != request.action
            || !matches!(
                request.action,
                HtmlDialogAction::EstablishCloseWatcher | HtmlDialogAction::DestroyCloseWatcher
            )
        {
            return Err(DomError::InvalidSourceOperation);
        }
        Ok(*frame)
    }
    pub(super) fn apply_watcher_receipt(
        &mut self,
        request: DomSourceDialogRequest,
        watcher: DomCloseWatcherId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut frame = self.watcher_frame(&request)?;
        if request.action == HtmlDialogAction::EstablishCloseWatcher {
            self.draft
                .set_dialog_watcher(frame.subject, Some(watcher), cancel)?;
        } else if self.draft.dialog_state(frame.subject)?.watcher != Some(watcher) {
            return Err(DomError::InvalidSourceOperation);
        }
        frame
            .cursor
            .reply(HtmlDialogReply::Applied)
            .map_err(DomError::DialogProtocol)?;
        self.draft.source_step(cancel)?;
        self.draft
            .effects
            .last_mut()
            .expect("qualified Hook")
            .effect = select::Effect::DialogHook(frame);
        Ok(())
    }
}
impl DomSourceLease<'_> {
    /// Qualify the actual Hook occurrence and original Window. Inactive/no-Global
    /// establishment has no positive service; destruction uses its stored ID.
    pub fn prepare_dialog_watcher(
        &self,
        request: &DomSourceDialogRequest,
        source: &DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<Option<DomDialogWatcherPreparation>, DomError> {
        self.dom
            .validate_hosted_source(source)
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        let op = self
            .dom
            .source_operation
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        let frame = op.watcher_frame(request)?;
        if op.receipt.is_some() {
            return Err(DomError::InvalidSourceOperation);
        }
        op.draft.source_step(cancel)?;
        // The actual association ElementState and final Source close must still
        // fit before the external manager effect can become irreversible.
        if op
            .outputs
            .checked_add(2)
            .is_none_or(|outputs| outputs > op.limits.outputs)
        {
            return Err(DomError::SourceOutputLimit);
        }
        if op
            .phases
            .checked_add(1)
            .is_none_or(|phases| phases > op.limits.phases)
        {
            return Err(DomError::SourcePublicationLimit);
        }
        let watcher = if request.action == HtmlDialogAction::EstablishCloseWatcher {
            let Some(global) = self
                .dom
                .state
                .node(self.dom.owner, request.subject)?
                .relevant_global
            else {
                return Ok(None);
            };
            let document = self
                .dom
                .snapshot()
                .global_document(global)
                .map_err(|_| DomError::HostedSourceUnavailable)?;
            let document_key = op.draft.resolve(document.node().into())?;
            if !op.draft.document_fully_active(document_key, cancel)? {
                return Ok(None);
            }
            let hook = request.hook().expect("qualified occurrence");
            DomCloseWatcherId {
                operation: op.id,
                command: hook.command,
                occurrence: hook.ordinal,
                target: request.subject,
                global,
                window_document: document,
            }
        } else {
            let watcher = op
                .draft
                .dialog_state(frame.subject)?
                .watcher
                .ok_or(DomError::InvalidSourceOperation)?;
            if watcher.global.source() != source.id() {
                return Err(DomError::HostedSourceUnavailable);
            }
            if self
                .dom
                .snapshot()
                .node(request.subject)?
                .as_dialog()
                .ok_or(DomError::WrongNodeKind)?
                .close_watcher()
                != Some(watcher)
            {
                return Err(DomError::InvalidSourceOperation);
            }
            watcher
        };
        Ok(Some(DomDialogWatcherPreparation {
            request: request.clone(),
            watcher,
        }))
    }
    pub fn accept_dialog_watcher(
        &mut self,
        request: &DomSourceDialogRequest,
        publication: &mut Option<DomDialogWatcherPublication>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let op = self
            .dom
            .source_operation
            .as_mut()
            .ok_or(DomError::InvalidSourceOperation)?;
        op.watcher_frame(request)?;
        op.draft.source_step(cancel)?;
        let actual = publication
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        if &actual.request != request || op.receipt.is_some() {
            return Err(DomError::InvalidSourceOperation);
        }
        op.refresh_admission(cancel)?;
        let actual = publication.take().expect("checked actual result");
        op.receipt = Some(Receipt::Watcher {
            request: actual.request,
            watcher: actual.watcher,
            applied: false,
        });
        Ok(())
    }
    pub fn retain_hosted_runtime_payload(
        &mut self,
        source: &DomHostedSource,
        bytes: usize,
        cancel: &AtomicBool,
    ) -> Result<crate::DomHostedRuntimeRetention, DomError> {
        source.retain_runtime_payload(self.dom, bytes, cancel)
    }
    /// Admit real hosted Runtime work and bounded post-advance readback/moves
    /// before a child can complete.
    /// This creates no execution result and retains the original cumulative work.
    pub fn admit_hosted_runtime_work(
        &self,
        steps: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if self.complete
            || self
                .dom
                .source_operation
                .as_ref()
                .is_none_or(|operation| operation.id != self.id)
        {
            return Err(DomError::InvalidSourceOperation);
        }
        self.dom.admit_hosted_runtime_work(steps, cancel)
    }
}

impl Dom {
    pub(in crate::store) fn admit_hosted_runtime_work(
        &self,
        steps: usize,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(op) = self.source_operation.as_ref() else {
            check_cancel(cancel)?;
            return Ok(());
        };
        for _ in 0..steps {
            if let Some(work) = op.observed_event_work() {
                work.step(cancel)?;
            } else {
                op.draft.source_step(cancel)?;
            }
        }
        Ok(())
    }
}
