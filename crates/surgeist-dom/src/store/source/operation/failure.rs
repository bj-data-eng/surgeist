//! First native failure custody belongs to the original Source operation.
use super::*;

impl DomSourceCleanupCause {
    pub(super) fn into_error(self, operation: DomSourceOperationId) -> DomError {
        match self {
            Self::InvariantFailed(failure) => {
                DomError::SourceInvariantFailed { operation, failure }
            }
            Self::SvgScriptProcessingUnavailable => DomError::SvgScriptProcessingUnavailable,
            Self::ReceiverRetired {
                subject,
                action: DomSourceStoppedAction::Dialog(action),
            } => DomError::SourceReceiverRetired {
                operation,
                subject,
                action,
            },
            Self::ReceiverRetired {
                subject,
                action: DomSourceStoppedAction::CloseWatcher(action),
            } => DomError::SourceCloseWatcherReceiverRetired {
                operation,
                subject,
                action,
            },
        }
    }
}
impl Operation {
    pub(super) fn can_close_svg_failure(
        &self,
        dom: &Dom,
        locked_cssom: Option<&surgeist_cssom::CssomStore>,
    ) -> bool {
        self.published
            && dom.state.incomplete_source == Some(self.id)
            && matches!(self.commands, OperationRoot::Edit(_))
            && !self.event_exposed
            && self.event_custody.is_none()
            && self.child.is_none()
            && self.hosted_global.is_none()
            && self.request_close.is_none()
            && self.inline_source.is_none()
            && self.meta_policy.is_none()
            && self.pending_meta.is_none()
            && self.meta_receipt.is_none()
            && self.draft.method.is_none()
            && self.draft.compound_failure().is_none()
            && self
                .svg_failure_style()
                .is_ok_and(|style| style.is_none() || locked_cssom.is_some())
    }

    fn svg_failure_style(&self) -> Result<Option<StyleAcknowledgment<'_>>, DomError> {
        match (&self.request, &self.receipt, &self.owner_binding) {
            (None, None, None) => Ok(None),
            (
                Some(request),
                Some(Receipt::Cssom {
                    revision,
                    sheet: Some(sheet),
                    applied: true,
                }),
                Some(binding),
            ) if request.action() == DomStyleAction::Update(Action::CreateSheet)
                && request.source().operation() == self.id
                && binding.matches(request) =>
            {
                Ok(Some(StyleAcknowledgment {
                    request,
                    revision,
                    sheet,
                    binding,
                }))
            }
            _ => Err(DomError::StyleReceiptMismatch),
        }
    }

    pub(super) fn remember_failure(&mut self, cause: DomSourceCleanupCause) {
        if self.failure.is_none() {
            self.failure = Some(cause);
        }
    }
}

struct StyleAcknowledgment<'a> {
    request: &'a DomStyleRequest,
    revision: &'a surgeist_cssom::CssomRevision,
    sheet: &'a surgeist_cssom::CssomSheetId,
    binding: &'a DomStyleCssomOwnerBinding,
}

impl StyleAcknowledgment<'_> {
    fn stage(
        self,
        dom: &Dom,
        original: &Draft,
        clean: &mut Draft,
        store: &surgeist_cssom::CssomStore,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use transaction::ordered_members::{Mode, OrderView};

        original.source_step(cancel)?;
        let snapshot = store.snapshot();
        let sheet = snapshot
            .sheet(self.sheet)
            .map_err(|_| DomError::StyleReceiptMismatch)?;
        if snapshot.revision() != self.revision
            || sheet.inputs().owner.as_deref() != Some(self.binding.owner())
            || &sheet.inputs().version != self.binding.version()
        {
            return Err(DomError::StyleReceiptMismatch);
        }
        let source = self.request.source();
        let key = Key::Existing(source.element());
        let association = self
            .request
            .association()
            .ok_or(DomError::StyleReceiptMismatch)?;
        let canonical = dom.state.node(dom.owner, source.element())?;
        let DomNodeKind::Element(element) = canonical.kind() else {
            return Err(DomError::StyleReceiptMismatch);
        };
        let style = element
            .style_state()
            .ok_or(DomError::StyleReceiptMismatch)?;
        if canonical.document() != source.document()
            || style.sheet.is_some()
            || association.element() != source.element()
            || OrderView::state(&dom.state, dom.owner)
                .root(key, Mode::Ordinary, &mut || original.source_step(cancel))?
                != Key::Existing(source.root())
        {
            return Err(DomError::StyleReceiptMismatch);
        }
        original.source_step(cancel)?;
        let staged = original.node(key)?;
        let transaction::KindRead::Element(element) = staged.kind() else {
            return Err(DomError::StyleReceiptMismatch);
        };
        let sheet = element
            .language()
            .style_state()
            .and_then(|style| style.sheet.as_ref())
            .ok_or(DomError::StyleReceiptMismatch)?;
        if sheet.id() != self.sheet
            || sheet.revision() != self.revision
            || sheet.root() != source.root()
            || sheet.association() != association
        {
            return Err(DomError::StyleReceiptMismatch);
        }
        stage_cssom_acknowledgment(
            clean,
            key,
            self.request,
            self.revision,
            Some(self.sheet),
            cancel,
        )
    }
}

impl DomSourceLease<'_> {
    pub(super) fn finish_source_failure(
        &mut self,
        op: &mut Operation,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, DomError> {
        let cause = op.failure.ok_or(DomError::InvalidSourceOperation)?;
        if op.request_close.as_ref().is_some_and(|call| !call.settled) {
            return Err(DomError::InvalidSourceOperation);
        }
        // Only the already-settled original operation releases its own marker.
        // This publication never resolves a dead receiver or runs its method.
        op.draft.source_set_method(None, cancel)?;
        op.publication(self.dom, false, PublicationTarget::None, None, cancel)?;
        op.receipt = None;
        self.complete = true;
        Err(cause.into_error(op.id))
    }

    pub(super) fn finish_svg_failure(
        &mut self,
        op: &mut Operation,
        locked_cssom: Option<&surgeist_cssom::CssomStore>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, DomError> {
        if !op.can_close_svg_failure(self.dom, locked_cssom) {
            return Err(DomError::InvalidSourceOperation);
        }
        // Only the already accepted CSSOM creation, if any, crosses the
        // closing fence. The Script and rejected suffix remain private.
        let mut clean = Draft::new_source_failure_close(self.dom, &op.draft, cancel)?;
        if let Some(style) = op.svg_failure_style()? {
            let store = locked_cssom.ok_or(DomError::StyleReceiptMismatch)?;
            style.stage(self.dom, &op.draft, &mut clean, store, cancel)?;
        }
        let rejected = std::mem::replace(&mut op.draft, dialog::OperationDraft::Segment(clean));
        if let Err(error) = op.publication(self.dom, false, PublicationTarget::None, None, cancel) {
            op.draft = rejected;
            return Err(error);
        }
        self.complete = true;
        Err(DomError::SvgScriptProcessingUnavailable)
    }
}
