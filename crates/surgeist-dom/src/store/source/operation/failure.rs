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
        // A settled handoff keeps its job and closing credit in the fullscreen
        // bank. Unprepared Inline and Meta inputs belong to the rejected Draft.
        // A prepared Meta phase still belongs to Source.
        self.published
            && dom.state.incomplete_source == Some(self.id)
            && matches!(self.commands, OperationRoot::Edit(_))
            && self.observation().is_none()
            && self.event_custody.is_none()
            && self.draft.fullscreen_exit.is_none()
            && self.child.is_none()
            && self.draft.hosted_pending.is_none()
            && self.hosted_global.is_none()
            && self.request_close.is_none()
            && self.pending_meta.is_none()
            && self.draft.method.is_none()
            && self.draft.compound_failure().is_none()
            && self.svg_failure_style().is_ok_and(|style| {
                !matches!(style, Some(StyleAcknowledgment::Cssom(_))) || locked_cssom.is_some()
            })
    }

    fn svg_failure_style(&self) -> Result<Option<StyleAcknowledgment<'_>>, DomError> {
        let Some(request) = &self.request else {
            return if self.receipt.is_none() && self.owner_binding.is_none() {
                Ok(None)
            } else {
                Err(DomError::StyleReceiptMismatch)
            };
        };
        if request.source().operation() != self.id {
            return Err(DomError::StyleReceiptMismatch);
        }
        let (revision, effect) = match (request.action(), &self.receipt, &self.owner_binding) {
            (
                DomStyleAction::Update(Action::CheckInlineCsp),
                Some(Receipt::Inline {
                    observation,
                    applied: true,
                    ..
                }),
                None,
            ) if !observation.allowed && matches_inline_csp_source(request, observation) => {
                return Ok(Some(StyleAcknowledgment::CspDenied));
            }
            (
                DomStyleAction::Update(Action::CreateSheet),
                Some(Receipt::Cssom {
                    revision,
                    sheet: Some(sheet),
                    applied: true,
                    ..
                }),
                Some(binding),
            ) if binding.matches(request) => (revision, CssomEffect::Created(sheet)),
            (
                DomStyleAction::Update(Action::RemoveAssociatedSheet),
                Some(Receipt::Cssom {
                    revision,
                    sheet: None,
                    applied: true,
                    ..
                }),
                None,
            ) => (
                revision,
                CssomEffect::Removed(request.sheet().ok_or(DomError::StyleReceiptMismatch)?),
            ),
            _ => return Err(DomError::StyleReceiptMismatch),
        };
        Ok(Some(StyleAcknowledgment::Cssom(CssomAcknowledgment {
            request,
            revision,
            effect,
        })))
    }

    pub(super) fn remember_failure(&mut self, cause: DomSourceCleanupCause) {
        if self.failure.is_none() {
            self.failure = Some(cause);
        }
    }
}

enum StyleAcknowledgment<'a> {
    CspDenied,
    Cssom(CssomAcknowledgment<'a>),
}

enum CssomEffect<'a> {
    Created(&'a surgeist_cssom::CssomSheetId),
    Removed(&'a DomStyleSheet),
}

struct CssomAcknowledgment<'a> {
    request: &'a DomStyleRequest,
    revision: &'a surgeist_cssom::CssomRevision,
    effect: CssomEffect<'a>,
}

impl CssomAcknowledgment<'_> {
    fn stage(
        self,
        dom: &Dom,
        original: &Draft,
        clean: &mut Draft,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        use transaction::ordered_members::{Mode, OrderView};

        original.source_step(cancel)?;
        let source = self.request.source();
        let key = Key::Existing(source.element());
        let canonical = dom.state.node(dom.owner, source.element())?;
        let DomNodeKind::Element(element) = canonical.kind() else {
            return Err(DomError::StyleReceiptMismatch);
        };
        let style = element
            .style_state()
            .ok_or(DomError::StyleReceiptMismatch)?;
        if canonical.document() != source.document()
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
        let staged_style = element
            .language()
            .style_state()
            .ok_or(DomError::StyleReceiptMismatch)?;
        let sheet = match self.effect {
            CssomEffect::Created(sheet) => {
                let associated = staged_style
                    .sheet
                    .as_ref()
                    .ok_or(DomError::StyleReceiptMismatch)?;
                if style.sheet.is_some()
                    || associated.id() != sheet
                    || associated.revision() != self.revision
                    || associated.root() != source.root()
                    || Some(associated.association()) != self.request.association()
                    || associated.association().element() != source.element()
                {
                    return Err(DomError::StyleReceiptMismatch);
                }
                Some(sheet)
            }
            CssomEffect::Removed(old) => {
                if style.sheet.as_ref() != Some(old)
                    || old.association().element() != source.element()
                    || staged_style.sheet.is_some()
                    || staged_style.next_association != style.next_association
                {
                    return Err(DomError::StyleReceiptMismatch);
                }
                None
            }
        };
        stage_cssom_acknowledgment(clean, key, self.request, self.revision, sheet, cancel)
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
        // The producer effect was qualified under the still-held receipt lock.
        // Only its acknowledgment crosses the fence. The suffix stays private.
        let mut clean = Draft::new_source_failure_close(self.dom, &op.draft, cancel)?;
        if let Some(StyleAcknowledgment::Cssom(style)) = op.svg_failure_style()? {
            style.stage(self.dom, &op.draft, &mut clean, cancel)?;
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
