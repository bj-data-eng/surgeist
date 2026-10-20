//! Called Dialog focusing continuation, retained by the original method frame.
use super::*;
mod delegate;
use crate::store::focus::AllowFocusPermission;
use surgeist_html::{
    HtmlDialogFocusControl as Control, HtmlDialogFocusingAction as A, HtmlDialogFocusingCursor,
    HtmlDialogFocusingReply as R,
};

#[derive(Clone)]
pub(in crate::store) struct FocusingFrame {
    pub cursor: HtmlDialogFocusingCursor,
    pub delegate: Option<Key>,
    pub top_document: Option<Key>,
    pub invocation: source::Invocation,
    delegate_call: Option<delegate::DelegateFrame>,
}
#[derive(Clone, Copy)]
pub(in crate::store) struct PendingFocusing {
    pub delegate: Option<delegate::PendingDelegate>,
    pub subject: Key,
    pub action: A,
    pub invocation: source::Invocation,
}
impl Draft {
    /// Read-only queue preflight uses the same complete called protocol. Its
    /// temporary bank has no invocation or effect authority and is released
    /// before queue admission. An unavailable query proves no executable tail.
    pub(in crate::store) fn dialog_delegate_null_available(
        &self,
        subject: Key,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.dialog_state(subject)?;
        let mut call = delegate::DelegateFrame::new(self, subject);
        Ok(call.advance(self, subject, cancel)? == Some(false))
    }
    pub(in crate::store) fn pending_focusing(&self) -> Option<PendingFocusing> {
        let method::Frame::Dialog(frame) = self.method.as_ref()? else {
            return None;
        };
        if frame.cursor.action() != Action::RunDialogFocusingSteps {
            return None;
        }
        let called = frame.focusing.as_ref()?;
        Some(PendingFocusing {
            delegate: called
                .delegate_call
                .as_ref()
                .and_then(|call| call.pending()),
            subject: frame.subject,
            action: called.cursor.action(),
            invocation: called.invocation,
        })
    }
    pub(in crate::store) fn dialog_allow_focus_permission(
        &self,
        subject: Key,
        cancel: &AtomicBool,
    ) -> Result<AllowFocusPermission, DomError> {
        self.source_step(cancel)?;
        self.document_allow_focus_permission(self.node(subject)?.document(), cancel)
    }
    fn selected_focus_control(
        &self,
        subject: Key,
        called: &FocusingFrame,
        control: Control,
    ) -> Option<Key> {
        match control {
            Control::Subject => Some(subject),
            Control::Delegate => called.delegate,
        }
    }
    pub(in crate::store) fn dialog_subject_focus_no_change(
        &self,
        subject: Key,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        self.dialog_state(subject)?;
        let own_inert = self.focus_boolean(
            subject,
            surgeist_html::HtmlBooleanAttribute::Inert,
            KnownHtmlAttribute::Inert,
            cancel,
        )?;
        self.source_step(cancel)?;
        let document = self.node(subject)?.document();
        let mut is_document_element = false;
        for child in self.node(document)?.children().iter() {
            self.source_step(cancel)?;
            if matches!(self.node(child)?.kind(), transaction::KindRead::Element(_)) {
                is_document_element = child == subject;
                break;
            }
        }
        self.source_step(cancel)?;
        let delegates = match self.node(subject)?.shadow_root() {
            None => false,
            Some(root) => {
                self.source_step(cancel)?;
                let transaction::KindRead::ShadowRoot(state) = &self.node(root)?.kind() else {
                    return Err(DomError::WrongNodeKind);
                };
                state.properties().delegates_focus()
            }
        };
        Ok(surgeist_html::dialog_subject_focusing_decision(
            surgeist_html::HtmlDialogSubjectFocusFacts {
                own_inert: Some(own_inert),
                is_document_element: Some(is_document_element),
                has_delegating_focus_root: Some(delegates),
            },
        ) == surgeist_html::HtmlDialogSubjectFocusingDecision::NoChange)
    }
    pub(in crate::store) fn dialog_control_top_document(
        &self,
        control: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        self.source_step(cancel)?;
        self.focus_top_document(self.node(control)?.document(), cancel)
    }
    pub(super) fn dialog_focusing_boundary(
        &mut self,
        frame: &mut MethodFrame,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut called = frame.focusing.take().unwrap_or(FocusingFrame {
            cursor: HtmlDialogFocusingCursor::new(),
            delegate: None,
            top_document: None,
            invocation: source::Invocation::Unissued,
            delegate_call: None,
        });
        let result = (|| {
            loop {
                self.source_step(cancel)?;
                let action = called.cursor.action();
                let reply = match action {
                    A::ReadAllowFocus => {
                        match self.dialog_allow_focus_permission(frame.subject, cancel)? {
                            AllowFocusPermission::Allowed
                                if surgeist_html::allow_focus_decision(Some(true), None)
                                    == surgeist_html::HtmlAllowFocusDecision::Allowed =>
                            {
                                Some(R::AllowFocus(true))
                            }
                            _ => None,
                        }
                    }
                    A::FindFocusDelegate => {
                        let delegate = called.delegate_call.get_or_insert_with(|| {
                            delegate::DelegateFrame::new(self, frame.subject)
                        });
                        match delegate.advance(self, frame.subject, cancel)? {
                            Some(present) => {
                                called.delegate_call = None;
                                Some(R::FocusDelegatePresent(present))
                            }
                            None => None,
                        }
                    }
                    A::ReadAutofocus => Some(R::Autofocus(self.focus_boolean(
                        frame.subject,
                        surgeist_html::HtmlBooleanAttribute::Autofocus,
                        KnownHtmlAttribute::Autofocus,
                        cancel,
                    )?)),
                    A::RunFocusingSteps {
                        control: Control::Subject,
                    } if self.dialog_subject_focus_no_change(frame.subject, cancel)? => {
                        Some(R::Applied)
                    }
                    A::CaptureControlTopDocument { control } => {
                        match self.selected_focus_control(frame.subject, &called, control) {
                            Some(control) => {
                                match self.dialog_control_top_document(control, cancel)? {
                                    Some(top) => {
                                        called.top_document = Some(top);
                                        Some(R::Applied)
                                    }
                                    None => None,
                                }
                            }
                            None => None,
                        }
                    }
                    A::ReadSameOrigin { control } => match (
                        self.selected_focus_control(frame.subject, &called, control),
                        called.top_document,
                    ) {
                        (Some(control), Some(top)) => {
                            self.source_step(cancel)?;
                            let document = self.node(control)?.document();
                            self.source_step(cancel)?;
                            Some(R::SameOrigin(
                                self.document_metadata(document)?.origin()
                                    == self.document_metadata(top)?.origin(),
                            ))
                        }
                        _ => None,
                    },
                    A::EmptyAutofocusCandidates => {
                        let top = called
                            .top_document
                            .ok_or(DomError::InvalidSourceOperation)?;
                        match self.autofocus_state(top) {
                            Ok(_) => {
                                self.autofocus_clear(top, cancel)?;
                                Some(R::Applied)
                            }
                            Err(DomError::AutofocusDocumentStateUnavailable) => None,
                            Err(error) => return Err(error),
                        }
                    }
                    A::SetAutofocusProcessed => {
                        self.autofocus_set_processed(
                            called
                                .top_document
                                .ok_or(DomError::InvalidSourceOperation)?,
                            cancel,
                        )?;
                        Some(R::Applied)
                    }
                    A::Complete => {
                        frame
                            .cursor
                            .reply(Reply::Applied)
                            .map_err(DomError::DialogProtocol)?;
                        return Ok(());
                    }
                    _ => None,
                };
                if let Some(reply) = reply {
                    called
                        .cursor
                        .reply(reply)
                        .map_err(DomError::DialogFocusingProtocol)?;
                    called.invocation = source::Invocation::Unissued;
                } else {
                    return Err(DomError::DialogFocusingUnavailable { action });
                }
            }
        })();
        if result.is_err() || called.cursor.action() != A::Complete {
            frame.focusing = Some(called);
        }
        result
    }
}

#[cfg(test)]
mod tests;
