//! Guest delivery through the original native element task bank.

use super::*;
use crate::element_tasks::{AttachedTaskRequest, ElementTaskRunOutcome};
use crate::{DomElementTaskProgress, DomElementTaskRunLimits, DomElementTaskSourceKind};

impl DocumentScriptHost {
    /// Execute or resume the attached Window's original task at the selected
    /// source's head. The native task bank owns event delivery and completion.
    ///
    /// This entry runs no guest microtask checkpoint.
    pub fn run_attached_element_task(
        &mut self,
        permit: &DocumentScriptPermit,
        source: DomElementTaskSourceKind,
        limits: DomElementTaskRunLimits,
    ) -> Result<DomElementTaskProgress, DocumentScriptEventError> {
        self.check_permit(permit)?;
        let attachment = self.attachment()?;
        attachment.realm.check_owner_active()?;
        let target = attachment.target;
        let events = self
            .guest_events()
            .map_err(DocumentScriptEventError::Guest)?;
        let owner = self.native.clone();
        let tasks = owner
            .session
            .try_borrow()
            .map_err(|_| DocumentScriptNativeError::Busy)?
            .element_task_execution();
        let mut expected = None;
        let mut usage = crate::dom_event::DispatchUsage::default();
        loop {
            let result = self.with_native(|dom, _, _| {
                let mut registry = events
                    .registry
                    .try_borrow_mut()
                    .map_err(|_| DocumentScriptNativeError::Busy)?;
                tasks
                    .run_attached(
                        dom,
                        &mut registry,
                        (source, limits),
                        AttachedTaskRequest {
                            target,
                            expected,
                            usage: &mut usage,
                        },
                        &owner.cancellation,
                    )
                    .map_err(DocumentScriptEventError::Task)
            })??;
            match result {
                ElementTaskRunOutcome::Native(progress) => return Ok(progress),
                ElementTaskRunOutcome::Guest(mut selected) => {
                    self.invoke_attached_listener(
                        &events,
                        selected.request.as_ref(),
                        &mut selected.invocation,
                    )?;
                    expected = Some(selected.id);
                }
            }
        }
    }
}
