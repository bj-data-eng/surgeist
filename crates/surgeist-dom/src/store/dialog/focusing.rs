//! Called Dialog focusing continuation, retained by the original method frame.
use super::*;
use crate::source::FocusQualification;
use surgeist_html::{
    HtmlDialogFocusControl as Control, HtmlDialogFocusingAction as A, HtmlDialogFocusingCursor,
    HtmlDialogFocusingReply as R,
};

#[derive(Clone, Copy)]
pub(in crate::store) struct FocusingFrame {
    pub cursor: HtmlDialogFocusingCursor,
    pub delegate: Option<Key>,
    pub top_document: Option<Key>,
    pub invocation: source::Invocation,
}
#[derive(Clone, Copy)]
pub(in crate::store) struct PendingFocusing {
    pub subject: Key,
    pub action: A,
    pub invocation: source::Invocation,
}
pub(in crate::store) enum AllowFocusPermission {
    Allowed,
    NeedsWindow(FocusQualification),
    Unavailable,
}
impl Draft {
    pub(in crate::store) fn pending_focusing(&self) -> Option<PendingFocusing> {
        let frame = self.dialog_method?;
        if frame.cursor.action() != Action::RunDialogFocusingSteps {
            return None;
        }
        let called = frame.focusing?;
        Some(PendingFocusing {
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
        let document = self.node(subject)?.document;
        self.source_step(cancel)?;
        let environment = self.document_metadata(document)?.document_environment();
        let Some(association) = environment.hosted.as_ref() else {
            return Ok(AllowFocusPermission::Unavailable);
        };
        let retired = matches!(
            association.phase,
            crate::hosted::DocumentPhase::Retired { .. }
        );
        if environment.browsing_context() == surgeist_html::HtmlBrowsingContextPresence::Present {
            if !matches!(
                association.phase,
                crate::hosted::DocumentPhase::InitialEntry { .. }
            ) {
                return Ok(AllowFocusPermission::Unavailable);
            }
            if self.document_fully_active(document, cancel)? {
                self.source_step(cancel)?;
                if association.policy.inherited_focus_without_user_activation()
                    == crate::DomPermissionsPolicyValue::Enabled
                {
                    return Ok(AllowFocusPermission::Allowed);
                }
            }
        } else if !retired {
            return Ok(AllowFocusPermission::Unavailable);
        }
        let document = DocumentId(
            self.published_id(document)
                .ok_or(DomError::UnavailableCreation)?,
        );
        Ok(AllowFocusPermission::NeedsWindow(if retired {
            FocusQualification::RetiredDocument {
                document,
                global: association.global,
                host: association.host,
            }
        } else {
            FocusQualification::DeniedDocument {
                document,
                global: association.global,
                host: association.host,
            }
        }))
    }
    fn dialog_focus_boolean(
        &self,
        subject: Key,
        selected: surgeist_html::HtmlBooleanAttribute,
        attribute: KnownHtmlAttribute,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        let element = self.element(subject)?;
        let mut raw = None;
        for item in element.attributes().iter() {
            self.source_step(cancel)?;
            if item.name().namespace().uri().is_none()
                && self.source_ascii(item.name().local_name(), attribute.local_name(), cancel)?
            {
                raw = Some(item);
                break;
            }
        }
        let formation = source::formation::Formation::new(self, cancel)?;
        if let Some(formation) = &formation {
            let mut payload = source::storage::Payload::default();
            payload.name(element.name())?;
            if let Some(raw) = raw {
                payload.name(raw.name())?;
                payload.string(raw.value())?;
            }
            formation.retain_payload(payload, cancel)?;
        }
        // The existing interpreter clones only its exact report inputs. Bound
        // its fixed vocabulary/namespace applicability and Boolean spelling
        // comparisons before invocation, independently of report retention.
        for _ in 0..4 {
            self.source_step(cancel)?;
            for _ in 0..=element
                .name()
                .namespace()
                .uri()
                .map_or(0, DocumentString::len)
            {
                self.source_step(cancel)?;
            }
            for known in surgeist_html::KnownHtmlElement::ALL {
                self.source_step(cancel)?;
                for _ in 0..=known.local_name().len() {
                    self.source_step(cancel)?;
                }
            }
        }
        if let Some(raw) = raw {
            for _ in 0..=raw.name().local_name().len() {
                self.source_step(cancel)?;
            }
            for _ in 0..=raw.value().len() {
                self.source_step(cancel)?;
            }
        }
        let report = surgeist_html::HtmlValueInterpreter::new(raw.map_or(0, |a| a.value().len()))
            .boolean_attribute(selected, element.name(), raw)
            .expect("actual admitted Boolean raw extent");
        let value = report
            .intrinsic()
            .copied()
            .ok_or(DomError::DialogInapplicable)?;
        drop(report);
        drop(formation);
        Ok(value)
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
    fn dialog_subject_focus_no_change(
        &self,
        subject: Key,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        self.dialog_state(subject)?;
        let own_inert = self.dialog_focus_boolean(
            subject,
            surgeist_html::HtmlBooleanAttribute::Inert,
            KnownHtmlAttribute::Inert,
            cancel,
        )?;
        self.source_step(cancel)?;
        let document = self.node(subject)?.document;
        let mut is_document_element = false;
        for child in &self.node(document)?.children {
            self.source_step(cancel)?;
            if matches!(self.node(*child)?.kind, DomNodeKind::Element(_)) {
                is_document_element = *child == subject;
                break;
            }
        }
        self.source_step(cancel)?;
        let delegates = match self.node(subject)?.shadow_root {
            None => false,
            Some(root) => {
                self.source_step(cancel)?;
                let DomNodeKind::ShadowRoot(state) = &self.node(root)?.kind else {
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
    fn dialog_control_top_document(
        &self,
        control: Key,
        cancel: &AtomicBool,
    ) -> Result<Option<Key>, DomError> {
        self.source_step(cancel)?;
        let document = self.node(control)?.document;
        self.source_step(cancel)?;
        let environment = self.document_metadata(document)?.document_environment();
        let Some(association) = environment.hosted.as_ref() else {
            return Ok(None);
        };
        let crate::hosted::DocumentPhase::InitialEntry { navigable, .. } = association.phase else {
            return Ok(None);
        };
        association
            .validate_live()
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        let graph = self
            .hosted_history
            .as_deref()
            .ok_or(DomError::HostedSourceUnavailable)?;
        let mut step = || {
            self.source_step(cancel)
                .map_err(crate::DomHostedError::from)
        };
        let node = graph
            .navigable_with(navigable, &mut step)
            .map_err(|e| match e {
                crate::DomHostedError::Dom(e) => e,
                _ => DomError::HostedSourceUnavailable,
            })?;
        let top = graph
            .navigable_with(node.traversable, &mut step)
            .map_err(|e| match e {
                crate::DomHostedError::Dom(e) => e,
                _ => DomError::HostedSourceUnavailable,
            })?;
        let active = graph
            .entry_with(top.active, &mut step)
            .map_err(|e| match e {
                crate::DomHostedError::Dom(e) => e,
                _ => DomError::HostedSourceUnavailable,
            })?
            .document;
        let Some(active) = active else {
            return Ok(None);
        };
        let key = self.resolve(active.node().into())?;
        self.source_step(cancel)?;
        let environment = self.document_metadata(key)?.document_environment();
        let Some(association) = environment.hosted.as_ref() else {
            return Ok(None);
        };
        association
            .validate_live()
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        if !matches!(association.phase, crate::hosted::DocumentPhase::InitialEntry { navigable,.. } if navigable==top.id)
            || association.global != top.global
        {
            return Err(DomError::HostedSourceUnavailable);
        }
        Ok(Some(key))
    }
    pub(super) fn dialog_focusing_boundary(
        &mut self,
        mut frame: MethodFrame,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut called = frame.focusing.unwrap_or(FocusingFrame {
            cursor: HtmlDialogFocusingCursor::new(),
            delegate: None,
            top_document: None,
            invocation: source::Invocation::Unissued,
        });
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
                A::ReadAutofocus => Some(R::Autofocus(self.dialog_focus_boolean(
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
                        Some(control) => match self.dialog_control_top_document(control, cancel)? {
                            Some(top) => {
                                called.top_document = Some(top);
                                Some(R::Applied)
                            }
                            None => None,
                        },
                        None => None,
                    }
                }
                A::ReadSameOrigin { control } => match (
                    self.selected_focus_control(frame.subject, &called, control),
                    called.top_document,
                ) {
                    (Some(control), Some(top)) => {
                        self.source_step(cancel)?;
                        let document = self.node(control)?.document;
                        self.source_step(cancel)?;
                        Some(R::SameOrigin(
                            self.document_metadata(document)?.origin()
                                == self.document_metadata(top)?.origin(),
                        ))
                    }
                    _ => None,
                },
                A::Complete => {
                    frame
                        .cursor
                        .reply(Reply::Applied)
                        .map_err(DomError::DialogProtocol)?;
                    frame.focusing = None;
                    self.dialog_method = Some(frame);
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
                frame.focusing = Some(called);
                self.dialog_method = Some(frame);
                return Err(DomError::DialogFocusingUnavailable { action });
            }
        }
    }
}

#[cfg(test)]
mod tests;
