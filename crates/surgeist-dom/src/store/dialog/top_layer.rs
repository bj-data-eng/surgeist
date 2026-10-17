//! Shared canonical top-layer algorithms and the coherent ShowModal Draft phase.
use super::*;

impl Draft {
    pub(super) fn add_top_layer(
        &mut self,
        subject: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        self.element(subject)?;
        let document = self.node(subject)?.document;
        if self.dialog_list_contains(document, subject, DialogList::Raw, cancel)? {
            if !self.dialog_list_contains(document, subject, DialogList::Pending, cancel)? {
                return Err(DomError::TopLayerAlreadyActive {
                    subject: self
                        .published_id(subject)
                        .ok_or(DomError::UnavailableCreation)?,
                    document: DocumentId(
                        self.published_id(document)
                            .ok_or(DomError::UnavailableCreation)?,
                    ),
                });
            }
            self.remove_top_layer_immediately(subject, cancel)?;
        }
        self.dialog_list_append(document, subject, DialogList::Raw, cancel)
    }
    pub(super) fn request_top_layer_removal(
        &mut self,
        subject: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        self.element(subject)?;
        let document = self.node(subject)?.document;
        if !self.dialog_list_contains(document, subject, DialogList::Raw, cancel)?
            || self.dialog_list_contains(document, subject, DialogList::Pending, cancel)?
        {
            return Ok(());
        }
        self.dialog_list_append(document, subject, DialogList::Pending, cancel)
    }
    pub(super) fn remove_top_layer_immediately(
        &mut self,
        subject: Key,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        self.element(subject)?;
        let document = self.node(subject)?.document;
        self.dialog_list_remove(document, subject, DialogList::Raw, cancel)?;
        self.dialog_list_remove(document, subject, DialogList::Pending, cancel)
    }
    pub(super) fn set_dialog_modal(
        &mut self,
        subject: Key,
        modal: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if self.dialog_state(subject)?.is_modal() == modal {
            return Ok(());
        }
        let formation = source::formation::Formation::new(self, cancel)?;
        self.admit_dialog_state_copy(subject, formation.as_ref(), false, true, cancel)?;
        self.source_step(cancel)?;
        let mut state = self.dialog_state(subject)?.clone();
        state.is_modal = modal;
        self.element_mut(subject).set_dialog_state(state);
        drop(formation);
        self.source_admit(0, cancel)
    }
    pub(super) fn dialog_modal_phase(
        &mut self,
        frame: &mut MethodFrame,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let subject = frame.subject;
        loop {
            self.source_step(cancel)?;
            let reply = match frame.cursor.action() {
                Action::SetModal => {
                    self.set_dialog_modal(subject, true, cancel)?;
                    Reply::Applied
                }
                Action::BlockNodeDocument => {
                    // Blocking is defined by raw Dialog order. Its canonical
                    // list and this flag stay private until this phase ends.
                    self.dialog_list(self.node(subject)?.document, DialogList::Raw)?;
                    Reply::Applied
                }
                Action::ReadTopLayerContains => {
                    Reply::TopLayerContains(self.dialog_list_contains(
                        self.node(subject)?.document,
                        subject,
                        DialogList::Raw,
                        cancel,
                    )?)
                }
                Action::AddTopLayer => {
                    self.add_top_layer(subject, cancel)?;
                    Reply::Applied
                }
                Action::RememberFocusedElement => {
                    return Err(DomError::DialogProcessingUnavailable {
                        action: Action::RememberFocusedElement,
                    });
                }
                _ => return Err(DomError::InvalidSourceOperation),
            };
            frame
                .cursor
                .reply(reply)
                .map_err(DomError::DialogProtocol)?;
        }
    }
}

#[cfg(test)]
mod tests;
