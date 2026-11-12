//! Retirement custody for a root created by the native Source attach command.

use super::*;
use crate::{DomNodeRetirementInbox, DomNodeRetirementLimits, DomNodeRetirementWatch};

pub(super) struct CreationWatch<'a> {
    inbox: &'a DomNodeRetirementInbox,
    custody_key: usize,
    limits: DomNodeRetirementLimits,
    output: Option<(NodeId, DomNodeRetirementWatch)>,
}

impl CreationWatch<'_> {
    pub(super) fn arm(
        &mut self,
        dom: &mut Dom,
        segment: &SegmentPublication,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let [Some(node)] = segment.historical.as_slice() else {
            return Err(DomError::UnavailableCreation);
        };
        let watch = dom.watch_prepared_source_creation(
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
    /// Attach a native ShadowRoot and arm its exact retirement watch in the
    /// original Source publication. Refusal before publication leaves both
    /// the host relation and the retirement index unchanged.
    pub fn attach_shadow_with_node_retirement_watch(
        &mut self,
        host: NodeId,
        init: crate::DomShadowRootInit,
        inbox: &DomNodeRetirementInbox,
        custody_key: usize,
        (source_limits, watch_limits): (DomSourceLimits, DomNodeRetirementLimits),
        cancel: &AtomicBool,
    ) -> Result<crate::DomWatchedNode, DomError> {
        let mut lease = self.begin_source_edit(
            DomSourceEdit::attach_shadow(host, init),
            source_limits,
            cancel,
        )?;
        let mut execution = Execution::disabled();
        execution.creation_watch = Some(CreationWatch {
            inbox,
            custody_key,
            limits: watch_limits,
            output: None,
        });
        let completion = match lease.advance_execution(&mut execution, None, cancel) {
            Ok(DomSourceProgress::Complete(done)) if !done.cancellation_requested() => done,
            Ok(DomSourceProgress::Complete(_)) => return Err(DomError::Cancelled),
            Ok(DomSourceProgress::Failed(failure)) => return Err(failure.into_parts().1),
            Err(AdvanceError::Dom(error)) => return Err(error),
            _ => return Err(DomError::InvalidSourceOperation),
        };
        let (node, watch) = execution
            .creation_watch
            .and_then(|watch| watch.output)
            .ok_or(DomError::UnavailableCreation)?;
        Ok(crate::DomWatchedNode::new(completion.commit, node, watch))
    }
}
