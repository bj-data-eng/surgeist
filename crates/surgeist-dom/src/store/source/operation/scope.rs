//! One original entry association, independent of Draft/checkpoint rebuilding.
use super::*;
use crate::{DomHostedHostId, DomHostedSource, DomSourceExecutionEntryScope};
#[derive(Clone, Copy)]
pub(super) enum EntryScope {
    NativeEntry,
    Unbound,
    FreshHosted {
        global: crate::DomHostedGlobalId,
        host: Option<DomHostedHostId>,
    },
    Hosted {
        global: crate::DomHostedGlobalId,
        host: Option<DomHostedHostId>,
    },
    ExplicitHost(DomHostedHostId),
}
impl EntryScope {
    pub(super) fn capture(
        dom: &Dom,
        node: Option<NodeId>,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<Self, DomError> {
        let Some(node) = node else {
            return Ok(Self::NativeEntry);
        };
        draft.source_step(cancel)?;
        // Entry metadata never replaces the command's original validation/error
        // order. An invalid operand is still diagnosed by that actual command.
        let Ok(node) = dom.state.node(dom.owner, node) else {
            return Ok(Self::NativeEntry);
        };
        let Some(global) = node.relevant_global() else {
            return Ok(Self::NativeEntry);
        };
        draft.source_step(cancel)?;
        let snapshot = dom.snapshot();
        let host = snapshot.global_document(global).ok().and_then(|document| {
            let document = snapshot.document(document).ok()?;
            let environment = document.metadata().document_environment();
            let association = environment.hosted.as_ref()?;
            (association.global == global).then_some(association.host)
        });
        Ok(Self::FreshHosted { global, host })
    }
    pub(super) fn seal(&mut self) {
        *self = match *self {
            Self::NativeEntry => Self::Unbound,
            Self::FreshHosted { global, host } => Self::Hosted { global, host },
            other => other,
        };
    }
    pub(super) fn observation(self) -> DomSourceExecutionEntryScope {
        match self {
            Self::NativeEntry => DomSourceExecutionEntryScope::NativeEntry,
            Self::Unbound => DomSourceExecutionEntryScope::Unbound,
            Self::FreshHosted { global, host } | Self::Hosted { global, host } => {
                DomSourceExecutionEntryScope::Hosted {
                    original_global: global,
                    original_host: host,
                }
            }
            Self::ExplicitHost(host) => DomSourceExecutionEntryScope::ExplicitHost(host),
        }
    }
}
impl OperationRoot {
    pub(super) fn entry_receiver(&self) -> Option<NodeId> {
        fn existing(node: &NodeOperand) -> Option<NodeId> {
            if let NodeOperand::Existing(id) = node {
                Some(*id)
            } else {
                None
            }
        }
        let command = match self {
            Self::SelectCompletion(target) => return Some(*target),
            Self::DialogTask(task) => return Some(task.id().target()),
            Self::NativeInstallation => return None,
            Self::Edit(commands) => commands.first(),
            #[cfg(test)]
            Self::Commands(commands) => commands.first(),
        }?;
        match command {
            Command::CreateRange { document, .. } | Command::SetActiveRange { document, .. } => {
                existing(document)
            }
            // A Range has no permanent receiver Document. Boundary operands
            // cannot supply an invented hosted entry association for it.
            Command::SetRangeStart { .. }
            | Command::SetRangeEnd { .. }
            | Command::RetireRange(_) => None,
            Command::BindEncodingUrlFacts { basis, .. }
            | Command::BindBaseFacts { basis, .. }
            | Command::ViewTransition { basis, .. } => Some(basis.document().node()),
            Command::Insert { parent, .. } | Command::ReplaceChild { parent, .. } => {
                existing(parent)
            }
            Command::Import { document, .. } | Command::Adopt { document, .. } => {
                existing(document)
            }
            Command::CreateXmlDocument {
                associated_document,
                ..
            }
            | Command::CreateHtmlDocument {
                associated_document,
                ..
            } => existing(associated_document),
            Command::Create { document, .. } => document.as_ref().and_then(existing),
            Command::AttachShadow { host, .. } => existing(host),
            Command::AssignSlot { slot, .. } => existing(slot),
            Command::Select { node, .. }
            | Command::SetStyleDisabled { node, .. }
            | Command::Output { node, .. }
            | Command::SetTextareaValue { node, .. }
            | Command::SetTextareaDefaultValue { node, .. }
            | Command::ResetTextarea { node }
            | Command::SetDesignMode { document: node, .. }
            | Command::ExitFullscreen { document: node }
            | Command::FullscreenElementModel { node }
            | Command::UnfullscreenElementModel { node }
            | Command::UnfullscreenDocumentModel { document: node }
            | Command::SetInputValue { node, .. }
            | Command::SetInputChecked { node, .. }
            | Command::SetInputIndeterminate { node, .. }
            | Command::ResetInput { node }
            | Command::SetReflectedTarget { node, .. }
            | Command::Detach(node)
            | Command::Destroy(node)
            | Command::SetAttribute { node, .. }
            | Command::RemoveAttribute { node, .. }
            | Command::SetCustomValidity { node, .. }
            | Command::ShowDialog { node }
            | Command::ShowModalDialog { node }
            | Command::CloseDialog { node, .. }
            | Command::RequestCloseDialog { node, .. }
            | Command::SetDialogReturnValue { node, .. }
            | Command::SetNonce { node, .. }
            | Command::SetMarqueeRunning { node, .. }
            | Command::ReplaceData { node, .. } => existing(node),
        }
    }
}
impl DomSourceLease<'_> {
    /// Historical original entry association, never execution authority.
    pub fn execution_entry_scope(&self) -> Result<DomSourceExecutionEntryScope, DomError> {
        self.dom
            .source_operation
            .as_ref()
            .filter(|operation| !self.complete && operation.id == self.id)
            .map(|operation| operation.entry_scope.observation())
            .ok_or(DomError::InvalidSourceOperation)
    }
    fn qualify_execution_host(
        &self,
        source: &DomHostedSource,
        host: DomHostedHostId,
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
        check_cancel(cancel)?;
        self.dom
            .validate_hosted_source(source)
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        source
            .observe()
            .validate(
                source.id(),
                crate::hosted::Role::Host,
                host.slot,
                host.generation,
            )
            .map_err(|_| DomError::HostedSourceUnavailable)?;
        if host.source != source.id() {
            return Err(DomError::HostedSourceUnavailable);
        }
        self.admit_hosted_runtime_work(1, cancel)
    }
    /// Bind a trusted exact Host once, before the original first advance.
    pub fn bind_execution_host(
        &mut self,
        source: &DomHostedSource,
        host: DomHostedHostId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.qualify_execution_host(source, host, cancel)?;
        let operation = self.dom.source_operation.as_mut().expect("live lease");
        operation.entry_scope = match operation.entry_scope {
            EntryScope::NativeEntry => EntryScope::ExplicitHost(host),
            EntryScope::FreshHosted {
                global,
                host: Some(original),
            } if original == host => EntryScope::Hosted {
                global,
                host: Some(original),
            },
            _ => return Err(DomError::InvalidSourceOperation),
        };
        Ok(())
    }
    /// Resume may confirm the exact original Host but never select a new scope.
    pub fn validate_execution_host(
        &self,
        source: &DomHostedSource,
        host: DomHostedHostId,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.qualify_execution_host(source, host, cancel)?;
        match self.execution_entry_scope()? {
            DomSourceExecutionEntryScope::ExplicitHost(original)
            | DomSourceExecutionEntryScope::Hosted {
                original_host: Some(original),
                ..
            } if original == host => Ok(()),
            _ => Err(DomError::InvalidSourceOperation),
        }
    }
}
