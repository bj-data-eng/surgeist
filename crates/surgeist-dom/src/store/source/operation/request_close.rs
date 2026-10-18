//! One original requestClose continuation; Runtime owns effects and custody only.
use super::*;
use surgeist_html::{
    HtmlCloseWatcherRequestAction as A, HtmlCloseWatcherRequestCursor,
    HtmlCloseWatcherRequestReply as R, HtmlDialogAction as D, HtmlDialogReply,
};

pub(super) struct Call {
    pub(super) outer: crate::store::dialog::MethodFrame,
    pub(super) cursor: HtmlCloseWatcherRequestCursor,
    pub(super) watcher: crate::DomCloseWatcherId,
    pub(super) invocation: DomSourceCloseWatcherInvocationId,
    pub(super) occurrence: Option<u64>,
    pub(super) window: DocumentId,
    pub(super) nested: bool,
    pub(super) settled: bool,
    pub(super) result: Option<bool>,
}
impl Call {
    pub(super) fn retry_copy(&self) -> Result<Self, DomError> {
        Ok(Self {
            outer: self.outer.before_focusing()?,
            cursor: self.cursor,
            watcher: self.watcher,
            invocation: self.invocation,
            occurrence: self.occurrence,
            window: self.window,
            nested: self.nested,
            settled: self.settled,
            result: self.result,
        })
    }
    fn step(&self, failure: Option<DomSourceCleanupCause>) -> DomSourceCloseWatcherStep {
        if failure.is_some() || self.result.is_some() {
            DomSourceCloseWatcherStep::SettleInvocation
        } else {
            DomSourceCloseWatcherStep::Action(self.cursor.action())
        }
    }
    fn request(
        &self,
        failure: Option<DomSourceCleanupCause>,
    ) -> Result<DomSourceCloseWatcherRequest, DomError> {
        Ok(DomSourceCloseWatcherRequest {
            invocation: self.invocation,
            watcher: self.watcher,
            step: self.step(failure),
            occurrence: self.occurrence.ok_or(DomError::InvalidSourceOperation)?,
            cause: failure,
        })
    }
}
impl Operation {
    fn qualify_close_watcher(
        &self,
        request: &DomSourceCloseWatcherRequest,
    ) -> Result<&Call, DomError> {
        let call = self
            .request_close
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        if call.nested
            || call.settled
            || call.request(self.failure)? != *request
            || request.operation() != self.id
        {
            return Err(DomError::InvalidSourceOperation);
        }
        Ok(call)
    }
    pub(super) fn apply_close_watcher_receipt(
        &mut self,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let Some(Receipt::CloseWatcher {
            publication,
            applied,
        }) = &self.receipt
        else {
            return Err(DomError::InvalidSourceOperation);
        };
        if *applied {
            return Ok(());
        }
        let mut call = self
            .qualify_close_watcher(&publication.request)?
            .retry_copy()?;
        self.draft.source_step(cancel)?;
        if let Some(reply) = publication.reply {
            call.cursor
                .reply(reply)
                .map_err(DomError::CloseWatcherProtocol)?;
        } else if publication.request.step == DomSourceCloseWatcherStep::SettleInvocation {
            call.settled = true;
        } else {
            return Err(DomError::InvalidSourceOperation);
        }
        call.occurrence = None;
        self.request_close = Some(call);
        if let Some(Receipt::CloseWatcher { applied, .. }) = &mut self.receipt {
            *applied = true;
        }
        Ok(())
    }
    pub(super) fn active_close_result(&self) -> Option<&DocumentString> {
        if self.request_close.as_ref().is_some_and(|call| call.nested) {
            self.request_close_result.as_ref()
        } else {
            match self.commands.get(self.index) {
                Some(Command::CloseDialog { result, .. }) => result.as_ref(),
                _ => None,
            }
        }
    }
    pub(super) fn fail_request_close(&mut self, cause: DomSourceCleanupCause) {
        self.remember_failure(cause);
        let call = self
            .request_close
            .as_mut()
            .expect("original requestClose call");
        call.nested = false;
        call.occurrence = None;
        self.draft.method = None;
        self.receipt = None;
    }
}
impl DomSourceLease<'_> {
    pub fn prepare_close_watcher_request(
        &self,
        request: &DomSourceCloseWatcherRequest,
        source: &crate::DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<crate::DomCloseWatcherRequestPreparation, DomError> {
        self.dom
            .validate_hosted_source(source)
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        let op = self
            .dom
            .source_operation
            .as_ref()
            .filter(|op| op.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        let call = op.qualify_close_watcher(request)?;
        if request.watcher.global().source() != source.id() || op.receipt.is_some() {
            return Err(DomError::InvalidSourceOperation);
        }
        op.draft.source_step(cancel)?;
        op.refresh_close_watcher_admission(cancel)?;
        Ok(crate::DomCloseWatcherRequestPreparation {
            request: request.clone(),
            cursor: call.cursor,
        })
    }
    pub fn accept_close_watcher_request(
        &mut self,
        request: &DomSourceCloseWatcherRequest,
        publication: &mut Option<crate::DomCloseWatcherRequestPublication>,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let op = self
            .dom
            .source_operation
            .as_mut()
            .filter(|op| op.id == self.id)
            .ok_or(DomError::InvalidSourceOperation)?;
        op.qualify_close_watcher(request)?;
        let actual = publication
            .as_ref()
            .ok_or(DomError::InvalidSourceOperation)?;
        if actual.request != *request || op.receipt.is_some() {
            return Err(DomError::InvalidSourceOperation);
        }
        op.draft.source_step(cancel)?;
        op.refresh_admission(cancel)?;
        let actual = publication.take().expect("qualified original publication");
        op.receipt = Some(Receipt::CloseWatcher {
            publication: actual,
            applied: false,
        });
        Ok(())
    }
    /// Fresh actual canonical getEnabledState, including the request's literal force flag.
    pub fn close_watcher_enabled(
        &self,
        request: &DomSourceCloseWatcherRequest,
        source: &crate::DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        let prepared = self.prepare_close_watcher_request(request, source, cancel)?;
        if prepared.request.step != DomSourceCloseWatcherStep::Action(A::ReadEnabled) {
            return Err(DomError::InvalidSourceOperation);
        }
        let op = self
            .dom
            .source_operation
            .as_ref()
            .expect("qualified operation");
        let key = op.draft.resolve(request.watcher.subject().into())?;
        let state = op.draft.dialog_state(key)?;
        if state.force_watcher_enabled {
            return Ok(surgeist_html::dialog_close_watcher_enabled(
                true,
                surgeist_html::HtmlClosedByState::Auto,
                state.is_modal(),
            ));
        }
        let element = op.draft.element(key)?;
        let mut raw = None;
        for attr in element.attributes().iter() {
            op.draft.source_step(cancel)?;
            if attr.name().namespace().uri().is_none()
                && op.draft.source_ascii(
                    attr.name().local_name(),
                    surgeist_html::KnownHtmlAttribute::Closedby.local_name(),
                    cancel,
                )?
            {
                raw = Some(attr);
                break;
            }
        }
        let bound = raw.map_or(0, |attr| attr.value().len());
        for _ in 0..bound {
            op.draft.source_step(cancel)?;
        }
        let formation = formation::Formation::new(&op.draft, cancel)?;
        // Enumerated report allocation is bounded by the producer's owning envelope.
        if let Some(budget) = &formation {
            let mut payload = storage::Payload::default();
            payload.name(element.name())?;
            if let Some(raw) = raw {
                payload.name(raw.name())?;
                payload.string(raw.value())?;
            }
            budget.retain_payload(payload, cancel)?;
            for _ in 0..2 {
                for _ in 0..=element
                    .name()
                    .namespace()
                    .uri()
                    .map_or(0, DocumentString::len)
                {
                    budget.step(cancel)?;
                }
            }
            for known in surgeist_html::KnownHtmlElement::ALL {
                for _ in 0..=known.local_name().len() {
                    budget.step(cancel)?;
                }
            }
            if let Some(raw) = raw {
                for _ in 0..=raw.name().local_name().len() {
                    budget.step(cancel)?;
                }
                for keyword in ["any", "closerequest", "none"] {
                    for _ in 0..=raw.value().len() {
                        budget.step(cancel)?;
                    }
                    for _ in 0..=keyword.len() {
                        budget.step(cancel)?;
                    }
                }
            }
        }
        let report = surgeist_html::HtmlValueInterpreter::new(bound)
            .closedby(element.name(), raw)
            .map_err(|_| DomError::InvalidSourceOperation)?;
        let closedby = *report.intrinsic().ok_or(DomError::DialogInapplicable)?;
        Ok(surgeist_html::dialog_close_watcher_enabled(
            state.force_watcher_enabled,
            closedby,
            state.is_modal(),
        ))
    }
    /// Fresh activity of the retained original Window's actual associated Document.
    pub fn close_watcher_window_fully_active(
        &self,
        request: &DomSourceCloseWatcherRequest,
        source: &crate::DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        let prepared = self.prepare_close_watcher_request(request, source, cancel)?;
        if prepared.request.step != DomSourceCloseWatcherStep::Action(A::ReadWindowFullyActive) {
            return Err(DomError::InvalidSourceOperation);
        }
        let op = self
            .dom
            .source_operation
            .as_ref()
            .expect("qualified operation");
        let call = op.qualify_close_watcher(request)?;
        let graph = op
            .draft
            .hosted_history
            .as_ref()
            .ok_or(DomError::HostedSourceUnavailable)?;
        let navigable = graph
            .active_navigable_for_document_with(call.window, || {
                op.draft
                    .source_step(cancel)
                    .map_err(crate::DomHostedError::from)
            })
            .map_err(|error| match error {
                crate::DomHostedError::Dom(error) => error,
                _ => DomError::HostedSourceUnavailable,
            })?;
        if navigable.is_none() {
            return Ok(false);
        }
        let key = op.draft.resolve(call.window.node().into())?;
        let environment = op.draft.document_metadata(key)?.document_environment();
        let association = environment
            .hosted
            .as_ref()
            .ok_or(DomError::HostedSourceUnavailable)?;
        if association.global != call.watcher.global() {
            return Err(DomError::HostedSourceUnavailable);
        }
        op.draft.document_fully_active(key, cancel)
    }
    pub(super) fn begin_request_close(
        &mut self,
        op: &mut Operation,
        require_history_action_activation: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        if op.request_close.is_some() {
            return Err(DomError::InvalidSourceOperation);
        }
        let Some(crate::store::method::Frame::Dialog(outer)) = op.draft.method.as_ref() else {
            return Err(DomError::InvalidSourceOperation);
        };
        let outer = outer.before_focusing()?;
        let crate::store::dialog::MethodLocals::RequestClose {
            watcher: Some(watcher),
        } = outer.locals
        else {
            return Err(DomError::InvalidSourceOperation);
        };
        let window = watcher.window_document();
        let method = op
            .draft
            .source_invocation
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if !op.command_admitted {
            op.admit_current_command(self.dom, cancel)?;
        }
        op.publication(self.dom, true, PublicationTarget::None, cancel)?;
        op.draft.source_invocation = method;
        op.request_close = Some(Call {
            outer,
            cursor: HtmlCloseWatcherRequestCursor::new(require_history_action_activation),
            watcher,
            invocation: DomSourceCloseWatcherInvocationId {
                operation: op.id,
                method,
            },
            occurrence: None,
            window,
            nested: false,
            settled: false,
            result: None,
        });
        op.started = true;
        op.receipt = None;
        Ok(())
    }
    pub(super) fn advance_request_close(
        &mut self,
        op: &mut Operation,
        cancel: &AtomicBool,
    ) -> Result<Option<DomSourceProgress>, DomError> {
        loop {
            op.refresh_admission(cancel)?;
            if op.receipt.is_some() {
                op.apply_receipt(cancel)?;
            }
            let mut call = op
                .request_close
                .as_ref()
                .ok_or(DomError::InvalidSourceOperation)?
                .retry_copy()?;
            if call.settled {
                if op.failure.is_none()
                    && matches!(
                        self.dom.state.node(self.dom.owner, call.watcher.subject()),
                        Err(DomError::StaleNode)
                    )
                {
                    op.remember_failure(DomSourceCleanupCause::ReceiverRetired {
                        subject: call.watcher.subject(),
                        action: DomSourceStoppedAction::Dialog(D::SetForceCloseWatcherEnabled(
                            false,
                        )),
                    });
                }
                if op.failure.is_some() {
                    return self.finish_source_failure(op, cancel).map(Some);
                }
                call.outer
                    .cursor
                    .reply(HtmlDialogReply::WatcherRequestResult(
                        call.result.ok_or(DomError::InvalidSourceOperation)?,
                    ))
                    .map_err(DomError::DialogProtocol)?;
                op.draft.method = Some(crate::store::method::Frame::Dialog(call.outer));
                op.request_close = None;
                op.receipt = None;
                return Ok(None);
            }
            if op.failure.is_none() && call.result.is_none() {
                let action = call.cursor.action();
                if matches!(action, A::ReadEnabled | A::RunCloseAction)
                    && matches!(
                        self.dom.state.node(self.dom.owner, call.watcher.subject()),
                        Err(DomError::StaleNode)
                    )
                {
                    op.fail_request_close(DomSourceCleanupCause::ReceiverRetired {
                        subject: call.watcher.subject(),
                        action: DomSourceStoppedAction::CloseWatcher(action),
                    });
                    continue;
                }
                match action {
                    A::CaptureWindow => {
                        op.draft.source_step(cancel)?;
                        call.cursor
                            .reply(R::Applied)
                            .map_err(DomError::CloseWatcherProtocol)?;
                        op.request_close = Some(call);
                        continue;
                    }
                    A::RunCancelAction { can_prevent_close } => {
                        return op
                            .observe_request_close_cancel(self.dom, can_prevent_close, cancel)
                            .map(Some);
                    }
                    A::RunCloseAction => {
                        let key = op.draft.resolve(call.watcher.subject().into())?;
                        let state = op.draft.dialog_state(key)?;
                        if state.request_source.is_some() {
                            return Err(DomError::DialogProcessingUnavailable {
                                action: D::RequestWatcherClose {
                                    require_history_action_activation: false,
                                },
                            });
                        }
                        let formation = formation::Formation::new(&op.draft, cancel)?;
                        let result = state
                            .request_return_value
                            .as_ref()
                            .map(|value| formation::share(value, formation.as_ref(), cancel))
                            .transpose()?;
                        op.request_close_result = result;
                        call.nested = true;
                        call.occurrence = None;
                        op.draft.method = Some(crate::store::method::Frame::Dialog(
                            crate::store::dialog::MethodFrame {
                                subject: key,
                                cursor: surgeist_html::HtmlDialogCursor::new(
                                    surgeist_html::HtmlDialogOperation::Close {
                                        result_present: op.request_close_result.is_some(),
                                    },
                                ),
                                state: crate::store::dialog::MethodState::Ready,
                                invocation: Invocation::Unissued,
                                locals: crate::store::dialog::MethodLocals::Close,
                                focusing: None,
                            },
                        ));
                        op.request_close = Some(call);
                        op.receipt = None;
                        drop(formation);
                        op.refresh_admission(cancel)?;
                        return Ok(None);
                    }
                    A::Complete(result) => {
                        call.result = Some(result);
                        call.occurrence = None;
                        op.request_close = Some(call);
                        continue;
                    }
                    _ => {}
                }
            }
            if call.occurrence.is_some() {
                return call.request(op.failure).map(|request| {
                    Some(DomSourceProgress::Request(DomSourceRequest::CloseWatcher(
                        request,
                    )))
                });
            }
            let occurrence = op
                .draft
                .source_invocation
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?;
            op.publication(self.dom, true, PublicationTarget::None, cancel)?;
            op.draft.source_invocation = occurrence;
            call.occurrence = Some(occurrence);
            let request = call.request(op.failure)?;
            op.request_close = Some(call);
            op.receipt = None;
            return Ok(request).map(|request| {
                Some(DomSourceProgress::Request(DomSourceRequest::CloseWatcher(
                    request,
                )))
            });
        }
    }
}
impl Operation {
    fn refresh_close_watcher_admission(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        self.draft.source_admit(0, cancel)?;
        self.draft.source_admit_units(0, cancel)
    }
}

impl Draft {
    fn request_close_shadow_next(
        &self,
        root: Key,
        mut current: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        self.source_step(cancel)?;
        let node = self.node(current)?;
        if let Some(shadow) = node.shadow_root {
            return Ok(Some(shadow));
        }
        if let Some(first) = node.children.first() {
            return Ok(Some(*first));
        }
        loop {
            self.source_step(cancel)?;
            if current == root {
                return Ok(None);
            }
            let node = self.node(current)?;
            if let Some(host) = node.host {
                if let Some(first) = self.node(host)?.children.first() {
                    return Ok(Some(*first));
                }
                current = host;
                continue;
            }
            let parent = node.parent.ok_or(DomError::HostedSourceUnavailable)?;
            let mut found = false;
            for sibling in &self.node(parent)?.children {
                self.source_step(cancel)?;
                if found {
                    return Ok(Some(*sibling));
                }
                found = *sibling == current;
            }
            if !found {
                return Err(DomError::HostedSourceUnavailable);
            }
            current = parent;
        }
    }
}
impl DomSourceLease<'_> {
    /// Capture actual history-consumption Windows using the original Window's
    /// navigable and HTML's shadow-including inclusive descendant algorithm.
    pub fn close_watcher_history_windows(
        &mut self,
        request: &DomSourceCloseWatcherRequest,
        source: &crate::DomHostedSource,
        cancel: &AtomicBool,
    ) -> Result<crate::DomCloseWatcherHistoryWindows, DomError> {
        let prepared = self.prepare_close_watcher_request(request, source, cancel)?;
        if prepared.request.step
            != DomSourceCloseWatcherStep::Action(A::ConsumeHistoryActionActivation)
        {
            return Err(DomError::InvalidSourceOperation);
        }
        let op = self
            .dom
            .source_operation
            .as_ref()
            .expect("qualified original operation");
        let call = op.qualify_close_watcher(request)?;
        let graph = op
            .draft
            .hosted_history
            .as_ref()
            .ok_or(DomError::HostedSourceUnavailable)?;
        let step = || {
            op.draft
                .source_step(cancel)
                .map_err(crate::DomHostedError::from)
        };
        let map = |error| match error {
            crate::DomHostedError::Dom(error) => error,
            _ => DomError::HostedSourceUnavailable,
        };
        let navigable = graph
            .active_navigable_for_document_with(call.window, step)
            .map_err(map)?;
        let formation = formation::Formation::new(&op.draft, cancel)?;
        let mut windows = Vec::new();
        // Each frame is one actual active Document traversal, retained while its
        // child active Document is visited in source order. No parent-edge closure.
        let mut pending: Vec<(Key, Option<Key>)> = Vec::new();
        if let Some(navigable) = navigable {
            let nav = graph.navigable_with(navigable, step).map_err(map)?;
            let top = graph.navigable_with(nav.traversable, step).map_err(map)?;
            let top_document = graph
                .entry_with(top.active, step)
                .map_err(map)?
                .document
                .ok_or(DomError::HostedSourceUnavailable)?;
            let key = op.draft.resolve(top_document.node().into())?;
            formation::reserve(&mut pending, 1, formation.as_ref(), cancel)?;
            pending.push((key, Some(key)));
        }
        while let Some((root, current)) = pending.last().copied() {
            op.draft.source_step(cancel)?;
            let Some(current) = current else {
                pending.pop();
                continue;
            };
            let node = op.draft.node(current)?;
            let mut child_document = None;
            if current == root {
                let environment = op.draft.document_metadata(root)?.document_environment();
                let association = environment
                    .hosted
                    .as_ref()
                    .ok_or(DomError::HostedSourceUnavailable)?;
                let crate::hosted::DocumentPhase::InitialEntry { navigable, .. } =
                    association.phase
                else {
                    return Err(DomError::HostedSourceUnavailable);
                };
                let row = graph.navigable_with(navigable, step).map_err(map)?;
                let active = graph
                    .entry_with(row.active, step)
                    .map_err(map)?
                    .document
                    .ok_or(DomError::HostedSourceUnavailable)?;
                let active_key = op.draft.resolve(active.node().into())?;
                let active_environment = op
                    .draft
                    .document_metadata(active_key)?
                    .document_environment();
                let active_association = active_environment
                    .hosted
                    .as_ref()
                    .ok_or(DomError::HostedSourceUnavailable)?;
                if active_association.global.source() != source.id() {
                    return Err(DomError::HostedSourceUnavailable);
                }
                formation::reserve(&mut windows, 1, formation.as_ref(), cancel)?;
                windows.push(active_association.global);
            } else if let DomNodeKind::Element(element) = &node.kind
                && let Some(Some(child)) = element.child_navigable()
            {
                let row = graph.navigable_with(child, step).map_err(map)?;
                child_document = graph.entry_with(row.active, step).map_err(map)?.document;
            }
            let next = op.draft.request_close_shadow_next(root, current, cancel)?;
            pending.last_mut().expect("original traversal frame").1 = next;
            if let Some(document) = child_document {
                let key = op.draft.resolve(document.node().into())?;
                formation::reserve(&mut pending, 1, formation.as_ref(), cancel)?;
                pending.push((key, Some(key)));
            }
        }
        let bytes = windows
            .capacity()
            .checked_mul(size_of::<crate::DomHostedGlobalId>())
            .and_then(|n| n.checked_add(size_of::<crate::DomCloseWatcherHistoryWindows>()))
            .ok_or(DomError::RepresentationExhausted)?;
        let retained = source.retain_runtime_payload(self.dom, bytes, cancel)?;
        drop(pending);
        drop(formation);
        Ok(crate::DomCloseWatcherHistoryWindows {
            windows,
            navigable,
            _retained: retained,
        })
    }
}
