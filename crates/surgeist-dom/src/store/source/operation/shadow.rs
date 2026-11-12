//! Synchronous Source attachment and retirement custody for its root output.

use super::*;
use crate::{DomNodeRetirementInbox, DomNodeRetirementLimits, DomNodeRetirementWatch};

pub(super) enum SynchronousAttachment<'a> {
    ReturnNode,
    Watch(AttachmentWatch<'a>),
}

pub(super) struct AttachmentWatch<'a> {
    inbox: &'a DomNodeRetirementInbox,
    custody_key: usize,
    limits: DomNodeRetirementLimits,
    output: Option<(NodeId, DomNodeRetirementWatch)>,
}

impl AttachmentWatch<'_> {
    pub(super) fn arm(
        &mut self,
        dom: &mut Dom,
        segment: &SegmentPublication,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let [Some(node)] = segment.historical.as_slice() else {
            return Err(DomError::UnavailableCreation);
        };
        let watch = dom.watch_prepared_source_node(
            &segment.prepared,
            *node,
            self.inbox,
            self.custody_key,
            self.limits,
            cancel,
        )?;
        self.output = Some((*node, watch));
        Ok(())
    }
}

impl Dom {
    /// Attach or reuse a ShadowRoot through a single complete Source publication.
    /// Operations reaching an external service require the ordinary Source lease.
    pub fn attach_shadow_synchronously(
        &mut self,
        host: NodeId,
        init: crate::DomShadowRootInit,
        source_limits: DomSourceLimits,
        cancel: &AtomicBool,
    ) -> Result<(DomCommit, NodeId), DomError> {
        let mut execution = Execution::disabled();
        execution.shadow_attachment = Some(SynchronousAttachment::ReturnNode);
        let completed =
            self.finish_shadow_attachment(host, init, source_limits, cancel, &mut execution)?;
        let Some(DomSourceOutput::Node(root)) = completed.output() else {
            return Err(DomError::InvalidSourceOperation);
        };
        Ok((completed.commit, root))
    }

    /// Attach or reuse a ShadowRoot and arm its exact retirement watch in the
    /// same Source publication. Refusal leaves the original host relation and
    /// retirement index intact.
    pub fn attach_shadow_with_node_retirement_watch(
        &mut self,
        host: NodeId,
        init: crate::DomShadowRootInit,
        inbox: &DomNodeRetirementInbox,
        custody_key: usize,
        (source_limits, watch_limits): (DomSourceLimits, DomNodeRetirementLimits),
        cancel: &AtomicBool,
    ) -> Result<crate::DomWatchedNode, DomError> {
        let mut execution = Execution::disabled();
        execution.shadow_attachment = Some(SynchronousAttachment::Watch(AttachmentWatch {
            inbox,
            custody_key,
            limits: watch_limits,
            output: None,
        }));
        let completion =
            self.finish_shadow_attachment(host, init, source_limits, cancel, &mut execution)?;
        let Some(SynchronousAttachment::Watch(AttachmentWatch {
            output: Some((node, watch)),
            ..
        })) = execution.shadow_attachment
        else {
            return Err(DomError::UnavailableCreation);
        };
        Ok(crate::DomWatchedNode::new(completion.commit, node, watch))
    }

    fn finish_shadow_attachment(
        &mut self,
        host: NodeId,
        init: crate::DomShadowRootInit,
        source_limits: DomSourceLimits,
        cancel: &AtomicBool,
        execution: &mut Execution<'_>,
    ) -> Result<DomSourceCompletion, DomError> {
        let mut lease = self.begin_source_edit(
            DomSourceEdit::attach_shadow(host, init),
            source_limits,
            cancel,
        )?;
        match lease.advance_execution(execution, None, cancel) {
            Ok(DomSourceProgress::Complete(done)) if !done.cancellation_requested() => Ok(done),
            Ok(DomSourceProgress::Complete(_)) => Err(DomError::Cancelled),
            Ok(DomSourceProgress::Failed(failure)) => Err(failure.into_parts().1),
            Ok(DomSourceProgress::Request(_)) => Err(DomError::IncompleteSourceOperation),
            Err(AdvanceError::Dom(error)) => Err(error),
            Err(_) => Err(DomError::InvalidSourceOperation),
        }
    }
}
