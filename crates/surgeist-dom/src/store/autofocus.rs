//! Original insertion continuation and canonical autofocus history transitions.
use super::focus::AllowFocusPermission;
use super::*;
use crate::autofocus::DocumentAutofocusState as State;
use surgeist_html::{
    HtmlAutofocusInsertionAction as A, HtmlAutofocusInsertionCursor,
    HtmlAutofocusInsertionReply as R,
};

#[derive(Clone, Copy)]
pub(in crate::store) struct Frame {
    pub subject: Key,
    pub target: Option<Key>,
    pub top: Option<Key>,
    pub cursor: HtmlAutofocusInsertionCursor,
    pub invocation: source::Invocation,
}
impl Draft {
    pub(in crate::store) fn autofocus_state(
        &self,
        document: Key,
    ) -> Result<(transaction::SequenceView<'_, NodeId, Key>, bool), DomError> {
        match self.node(document)?.autofocus_document {
            Some(transaction::AutofocusRead::Known {
                candidates,
                processed,
            }) => Ok((candidates, processed)),
            Some(transaction::AutofocusRead::Unavailable) => {
                Err(DomError::AutofocusDocumentStateUnavailable)
            }
            None => Err(DomError::WrongNodeKind),
        }
    }
    pub(in crate::store) fn autofocus_count(&self, cancel: &AtomicBool) -> Result<usize, DomError> {
        self.source_step(cancel)?;
        Ok(self.counts.autofocus_candidates)
    }
    pub(in crate::store) fn autofocus_remove(
        &mut self,
        document: Key,
        subject: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let mut index = 0;
        loop {
            self.source_step(cancel)?;
            let candidates = self.autofocus_state(document)?.0;
            if index == candidates.len() {
                break;
            }
            if candidates.get(index) != Some(subject) {
                index += 1;
                continue;
            }
            for _ in index..candidates.len() {
                self.source_step(cancel)?;
            }
            self.relations_remove(
                document,
                transaction::RelationField::AutofocusCandidates,
                index,
                1,
                cancel,
            )?;
        }
        Ok(())
    }
    pub(in crate::store) fn autofocus_append(
        &mut self,
        document: Key,
        subject: Key,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        self.element(subject)?;
        self.autofocus_state(document)?;
        let count = self
            .autofocus_count(cancel)?
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        admit(
            count,
            limits.autofocus_candidates,
            DomResource::AutofocusCandidates,
        )?;
        let length = self.autofocus_state(document)?.0.len();
        self.relations_insert(
            document,
            transaction::RelationField::AutofocusCandidates,
            length,
            &[subject],
            cancel,
        )?;
        Ok(())
    }
    pub(in crate::store) fn autofocus_clear(
        &mut self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let length = self.autofocus_state(document)?.0.len();
        self.relations_remove(
            document,
            transaction::RelationField::AutofocusCandidates,
            0,
            length,
            cancel,
        )?;
        Ok(())
    }
    pub(in crate::store) fn autofocus_set_processed(
        &mut self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        self.autofocus_state(document)?;
        self.with_node_fields(document, cancel, |fields| {
            let Some(State::Known { processed, .. }) = &mut fields.autofocus_document else {
                unreachable!()
            };
            *processed = true;
        })?;
        Ok(())
    }
    pub(in crate::store) fn retire_autofocus_candidate(
        &mut self,
        subject: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if !matches!(self.node(subject)?.kind, DomNodeKind::Element(_)) {
            return Ok(());
        }
        let owners = self.inbound_keys(subject, cancel)?;
        for key in owners.iter().copied() {
            self.source_step(cancel)?;
            if matches!(
                self.node(key)?.autofocus_document,
                Some(transaction::AutofocusRead::Known { .. })
            ) {
                self.autofocus_remove(key, subject, cancel)?;
                #[cfg(test)]
                tests::after_retirement_document(self);
            }
        }
        Ok(())
    }
    pub(in crate::store) fn pending_autofocus(&self) -> Option<Frame> {
        match self.effects.last().map(|e| &e.effect) {
            Some(select::Effect::AutofocusInsertion(frame)) => Some(*frame),
            _ => None,
        }
    }
    /// One coalesced change row for a currently executable same-origin tail.
    /// This preflights the newly admitted local domain before queue publication.
    pub(in crate::store) fn dialog_autofocus_tail_outputs(
        &self,
        subject: Key,
        cancel: &AtomicBool,
    ) -> Result<usize, DomError> {
        self.source_step(cancel)?;
        let document = self.node(subject)?.document;
        if !matches!(
            self.document_allow_focus_permission(document, cancel)?,
            AllowFocusPermission::Allowed
        ) {
            return Ok(0);
        }
        let autofocus = self.focus_boolean(
            subject,
            surgeist_html::HtmlBooleanAttribute::Autofocus,
            surgeist_html::KnownHtmlAttribute::Autofocus,
            cancel,
        )?;
        if !self.dialog_subject_focus_no_change(subject, cancel)?
            || (!autofocus && !self.dialog_delegate_null_available(subject, cancel)?)
        {
            return Ok(0);
        }
        let Some(top) = self.focus_top_document(document, cancel)? else {
            return Ok(0);
        };
        self.source_step(cancel)?;
        if self.document_metadata(document)?.origin() != self.document_metadata(top)?.origin() {
            return Ok(0);
        }
        match self.autofocus_state(top) {
            Ok((candidates, processed)) => Ok(usize::from(!candidates.is_empty() || !processed)),
            Err(DomError::AutofocusDocumentStateUnavailable) => Ok(0),
            Err(error) => Err(error),
        }
    }
    pub(in crate::store) fn autofocus_presence(
        &self,
        subject: Key,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        self.source_step(cancel)?;
        let element = self.element(subject)?;
        for attribute in element.attributes().iter() {
            self.source_step(cancel)?;
            // The producer qualifies the actual Element against three constant
            // URI lengths. Only equal-length comparisons inspect URI units;
            // no scan grows with an arbitrary URI length or reads raw values.
            if let Some(uri) = element.name().namespace().uri() {
                for width in [28, 26, 34] {
                    self.source_step(cancel)?;
                    if uri.len() == width {
                        for _ in 0..width {
                            self.source_step(cancel)?;
                        }
                    }
                }
            }
            // At most nine attribute-name units after the exact length gate.
            if attribute.name().namespace().uri().is_none()
                && attribute.name().local_name().len() == 9
            {
                for _ in 0..9 {
                    self.source_step(cancel)?;
                }
            }
            if surgeist_html::autofocus_presence(element.name(), Some(attribute)) {
                return Ok(true);
            }
        }
        Ok(false)
    }
    pub(super) fn autofocus_insertion(
        &mut self,
        subject: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if !matches!(self.node(subject)?.kind, DomNodeKind::Element(_)) {
            return Ok(());
        }
        self.push_effect(
            select::Effect::AutofocusInsertion(Frame {
                subject,
                target: None,
                top: None,
                cursor: HtmlAutofocusInsertionCursor::new(),
                invocation: source::Invocation::Unissued,
            }),
            self.effect_scoped,
            cancel,
        )
    }
    pub(super) fn autofocus_effect(
        &mut self,
        mut frame: Frame,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        loop {
            self.source_step(cancel)?;
            let action = frame.cursor.action();
            let reply = match action {
                A::ReadAutofocus => Some(R::Autofocus(
                    self.autofocus_presence(frame.subject, cancel)?,
                )),
                A::ReadInDocumentTree => {
                    let root = crate::slot::root_admitted(self, frame.subject, &mut || {
                        self.source_step(cancel)
                    })?;
                    Some(R::InDocumentTree(matches!(
                        self.node(root)?.kind,
                        DomNodeKind::Document(_)
                    )))
                }
                A::CaptureTargetDocument => {
                    frame.target = Some(self.node(frame.subject)?.document);
                    Some(R::Applied)
                }
                A::ReadFullyActive => Some(R::FullyActive(self.document_fully_active(
                    frame.target.ok_or(DomError::InvalidSourceOperation)?,
                    cancel,
                )?)),
                A::ReadSandboxAutomaticFeatures => {
                    let target = frame.target.ok_or(DomError::InvalidSourceOperation)?;
                    self.source_step(cancel)?;
                    self.document_metadata(target)?
                        .hosted_inputs
                        .as_ref()
                        .map(|i| {
                            R::SandboxAutomaticFeatures(
                                i.0.active_sandbox
                                    .contains(surgeist_html::HtmlSandboxFlag::AutomaticFeatures),
                            )
                        })
                }
                A::ReadAllowFocus => match self.document_allow_focus_permission(
                    frame.target.ok_or(DomError::InvalidSourceOperation)?,
                    cancel,
                )? {
                    AllowFocusPermission::Allowed
                        if surgeist_html::allow_focus_decision(Some(true), None)
                            == surgeist_html::HtmlAllowFocusDecision::Allowed =>
                    {
                        Some(R::AllowFocus(true))
                    }
                    AllowFocusPermission::NeedsWindow(_)
                        if self.headroom_external_hook(cancel)? =>
                    {
                        Some(R::AllowFocus(false))
                    }
                    _ => None,
                },
                A::CaptureTargetTopDocument => match self.focus_top_document(
                    frame.target.ok_or(DomError::InvalidSourceOperation)?,
                    cancel,
                )? {
                    Some(top) => {
                        frame.top = Some(top);
                        Some(R::Applied)
                    }
                    None => None,
                },
                A::ReadAutofocusProcessed => {
                    match self.autofocus_state(frame.top.ok_or(DomError::InvalidSourceOperation)?) {
                        Ok((_, processed)) => Some(R::AutofocusProcessed(processed)),
                        Err(DomError::AutofocusDocumentStateUnavailable) => None,
                        Err(error) => return Err(error),
                    }
                }
                A::RemoveCandidate => {
                    self.autofocus_remove(
                        frame.top.ok_or(DomError::InvalidSourceOperation)?,
                        frame.subject,
                        cancel,
                    )?;
                    Some(R::Applied)
                }
                A::AppendCandidate => {
                    self.autofocus_append(
                        frame.top.ok_or(DomError::InvalidSourceOperation)?,
                        frame.subject,
                        limits,
                        cancel,
                    )?;
                    Some(R::Applied)
                }
                A::Complete => return Ok(()),
            };
            if let Some(reply) = reply {
                frame
                    .cursor
                    .reply(reply)
                    .map_err(DomError::AutofocusInsertionProtocol)?;
                frame.invocation = source::Invocation::Unissued;
            } else {
                self.push_effect(
                    select::Effect::AutofocusInsertion(frame),
                    self.effect_scoped,
                    cancel,
                )?;
                return Err(DomError::AutofocusInsertionUnavailable { action });
            }
        }
    }
}

#[cfg(test)]
mod tests;
