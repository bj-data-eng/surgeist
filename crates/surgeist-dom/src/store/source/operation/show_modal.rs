//! Standard ShowModal prefix through its actual canonical watcher assertion.
use super::*;
use surgeist_html::{HtmlDialogAction, HtmlDialogOperation, HtmlDialogReply};

impl Operation {
    pub(super) fn is_show_modal_method(&self) -> bool {
        matches!(
            self.commands,
            OperationRoot::Edit([Command::ShowModalDialog { .. }])
        )
    }
    pub(super) fn observe_show_modal_assertion(
        &mut self,
        dom: &Dom,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut frame = self
            .draft
            .dialog_method
            .ok_or(DomError::InvalidSourceOperation)?;
        if !self.is_show_modal_method()
            || frame.cursor.action() != HtmlDialogAction::AssertCloseWatcherPresent
            || !matches!(frame.state, crate::store::dialog::MethodState::Ready)
            || self.draft.effects.len() != self.draft.effect_floor
            || dom.state.incomplete_source != Some(self.id)
        {
            return Err(DomError::InvalidSourceOperation);
        }
        self.draft.source_step(cancel)?;
        let subject = self
            .draft
            .published_id(frame.subject)
            .ok_or(DomError::UnavailableCreation)?;
        let node = dom.state.node(dom.owner, subject)?;
        let DomNodeKind::Element(element) = &node.kind else {
            return Err(DomError::DialogInapplicable);
        };
        let state = element.dialog_state().ok_or(DomError::DialogInapplicable)?;
        let Some(watcher) = state.watcher else {
            self.remember_failure(DomSourceCleanupCause::InvariantFailed(
                DomSourceInvariantFailure {
                    dialog_operation: HtmlDialogOperation::ShowModal,
                    subject,
                    action: HtmlDialogAction::AssertCloseWatcherPresent,
                    basis: dom.snapshot().stamp(),
                    reason: DomSourceInvariantReason::MissingDialogCloseWatcher,
                },
            ));
            return Ok(());
        };
        // The closed canonical association was made by an affine original Hook
        // receipt. Validate its original provenance, independently of adoption
        // and of Runtime's still-registering readback custody.
        if watcher.subject() != subject
            || watcher.operation.owner != dom.owner
            || watcher.operation.serial >= dom.next_source_operation
            || watcher.global.source().owner != dom.owner
            || watcher.window_document().node().owner() != dom.owner
            || node.relevant_global != Some(watcher.global())
            || (watcher.operation == self.id
                && (watcher.command != self.index
                    || watcher.occurrence >= self.draft.source_invocation))
        {
            return Err(DomError::InvalidSourceOperation);
        }
        dom.hosted_source
            .as_ref()
            .ok_or(DomError::HostedSourceUnavailable)?
            .validate_issuer(watcher.global.source())
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        frame
            .cursor
            .reply(HtmlDialogReply::Applied)
            .map_err(DomError::DialogProtocol)?;
        self.draft.dialog_method = Some(frame);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn original_source_failure_keeps_its_first_assertion_basis_and_exact_terminal_value() {
        // Isolate the failure-custody owner. No active Window, watcher receipt
        // or modal language trace is fabricated by this private storage test.
        let cancel = AtomicBool::new(false);
        let mut dom =
            Dom::new(DomLimits::new(32, 100_000, 64, 0, 0, 16, 1_000_000).unwrap()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let subject = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
        let other = batch.create_element_ns(&document, crate::Namespace::html(), "dialog");
        let commit = dom.apply(batch, &cancel).unwrap();
        let subject = commit.created(&subject).unwrap().unwrap();
        let other = commit.created(&other).unwrap().unwrap();
        let basis = dom.snapshot().stamp();
        let lease = dom
            .begin_source_edit(
                DomSourceEdit::show_modal_dialog(subject),
                DomSourceLimits::new(32_000_000, 1_000_000, 10_000_000, 64, 4096, 4096, 4),
                &cancel,
            )
            .unwrap();
        let id = lease.operation();
        let failure = DomSourceInvariantFailure {
            dialog_operation: surgeist_html::HtmlDialogOperation::ShowModal,
            subject,
            action: surgeist_html::HtmlDialogAction::AssertCloseWatcherPresent,
            basis,
            reason: DomSourceInvariantReason::MissingDialogCloseWatcher,
        };
        let original = DomSourceCleanupCause::InvariantFailed(failure);
        let operation = lease.dom.source_operation.as_mut().unwrap();
        operation.remember_failure(original);
        operation.remember_failure(DomSourceCleanupCause::ReceiverRetired {
            subject: other,
            action: DomSourceStoppedAction::Dialog(surgeist_html::HtmlDialogAction::ReadOpen),
        });
        assert_eq!(lease.status().cleanup_cause(), Some(original));
        assert_eq!(
            failure.dialog_operation(),
            surgeist_html::HtmlDialogOperation::ShowModal
        );
        assert_eq!(failure.subject(), subject);
        assert_eq!(
            failure.action(),
            surgeist_html::HtmlDialogAction::AssertCloseWatcherPresent
        );
        assert_eq!(failure.basis(), basis);
        assert_eq!(
            failure.reason(),
            DomSourceInvariantReason::MissingDialogCloseWatcher
        );
        assert_eq!(
            original.into_error(id),
            DomError::SourceInvariantFailed {
                operation: id,
                failure
            }
        );
        // Unpublished private custody has no incurred external tail to retain.
        drop(lease);
        assert!(dom.pending_source_edit().is_none());
        assert_eq!(dom.snapshot().stamp(), basis);
    }
}
