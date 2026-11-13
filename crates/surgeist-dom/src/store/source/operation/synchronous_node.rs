//! Synchronous Source creation and retirement custody for its node output.

use super::*;
use crate::{DomNodeRetirementInbox, DomNodeRetirementLimits, DomNodeRetirementWatch};

pub(super) enum SynchronousNode<'a> {
    ReturnNode,
    Watch(NodeWatch<'a>),
}

/// A loan of the exact canonical name prepared for a detached Element. Work
/// on this loan belongs to the original Source operation and its observers.
pub struct DomPreparedElementName<'a> {
    name: &'a crate::DomQualifiedName,
    formation: &'a formation::Formation,
    cancel: &'a AtomicBool,
}

impl DomPreparedElementName<'_> {
    pub const fn name(&self) -> &crate::DomQualifiedName {
        self.name
    }

    pub fn step(&self) -> Result<(), DomError> {
        self.formation.step(self.cancel)
    }
}

type PrepareElement<'a> = dyn FnMut(DomPreparedElementName<'_>) -> bool + 'a;

pub(super) struct NodeWatch<'a> {
    inbox: &'a DomNodeRetirementInbox,
    custody_key: usize,
    limits: DomNodeRetirementLimits,
    prepare_element: Option<&'a mut PrepareElement<'a>>,
    output: Option<(NodeId, DomNodeRetirementWatch)>,
}

impl NodeWatch<'_> {
    pub(super) fn arm(
        &mut self,
        dom: &mut Dom,
        segment: &SegmentPublication,
        cancel: &AtomicBool,
    ) -> Result<(), AdvanceError> {
        let [Some(node)] = segment.historical.as_slice() else {
            return Err(DomError::UnavailableCreation.into());
        };
        if let Some(prepare) = self.prepare_element.as_deref_mut() {
            let formation = segment
                .prepared
                ._formation
                .as_ref()
                .ok_or(DomError::InvalidSourceOperation)?;
            let element = segment
                .prepared
                .state
                .node_with(dom.owner, *node, &mut || formation.step(cancel))?
                .element()
                .ok_or(DomError::WrongNodeKind)?;
            let ready = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                prepare(DomPreparedElementName {
                    name: element.name(),
                    formation,
                    cancel,
                })
            }))
            .map_err(AdvanceError::Unwind)?;
            if !ready {
                return Err(DomError::UnavailableCreation.into());
            }
        }
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
    /// Create a detached Element using canonical name formation. The host may
    /// form private output from the prepared name before the retirement watch
    /// and Element publish together. Either refusal drops the private output.
    /// The preparation closure cannot run guest code that needs this DOM's
    /// exclusive mutation authority. Its output cannot borrow the private name.
    pub fn create_detached_element_with_node_retirement_watch<T, E: From<DomError>>(
        &mut self,
        document: crate::DocumentId,
        (name, options): (
            crate::DomElementCreationName,
            crate::DomElementCreationOptions,
        ),
        (inbox, custody_key): (&DomNodeRetirementInbox, usize),
        limits: (DomSourceLimits, DomNodeRetirementLimits),
        cancel: &AtomicBool,
        prepare: impl FnOnce(DomPreparedElementName<'_>) -> Result<T, E>,
    ) -> Result<(crate::DomWatchedNode, T), E> {
        let edit = DomSourceEdit::create_element_with_options(document.node(), name, options);
        let mut prepare = Some(prepare);
        let mut output = None;
        let watched = self.publish_watched_source_node(
            edit,
            (inbox, custody_key),
            limits,
            cancel,
            Some(&mut |name| {
                output = prepare.take().map(|prepare| prepare(name));
                matches!(output, Some(Ok(_)))
            }),
        );
        match output {
            Some(Ok(output)) => Ok((watched?, output)),
            Some(Err(error)) => Err(error),
            None => Err(watched
                .err()
                .unwrap_or(DomError::UnavailableCreation)
                .into()),
        }
    }

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
        execution.synchronous_node = Some(SynchronousNode::ReturnNode);
        let completed = self.finish_synchronous_node(
            DomSourceEdit::attach_shadow(host, init),
            source_limits,
            cancel,
            &mut execution,
        )?;
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
        self.publish_watched_source_node(
            DomSourceEdit::attach_shadow(host, init),
            (inbox, custody_key),
            (source_limits, watch_limits),
            cancel,
            None,
        )
    }

    /// Create a detached node and arm its retirement watch in the same Source
    /// publication, including during authenticated Source callback reentry.
    /// Refusal leaves the canonical tree and retirement index unchanged.
    pub fn create_detached_with_node_retirement_watch(
        &mut self,
        document: crate::DocumentId,
        creation: DomDetachedNodeCreation,
        (inbox, custody_key): (&DomNodeRetirementInbox, usize),
        limits: (DomSourceLimits, DomNodeRetirementLimits),
        cancel: &AtomicBool,
    ) -> Result<crate::DomWatchedNode, DomError> {
        self.publish_watched_source_node(
            creation.into_edit(document),
            (inbox, custody_key),
            limits,
            cancel,
            None,
        )
    }

    fn publish_watched_source_node<'a>(
        &mut self,
        edit: DomSourceEdit,
        (inbox, custody_key): (&'a DomNodeRetirementInbox, usize),
        (source_limits, watch_limits): (DomSourceLimits, DomNodeRetirementLimits),
        cancel: &AtomicBool,
        prepare_element: Option<&'a mut PrepareElement<'a>>,
    ) -> Result<crate::DomWatchedNode, DomError> {
        let mut execution = Execution::disabled();
        execution.synchronous_node = Some(SynchronousNode::Watch(NodeWatch {
            inbox,
            custody_key,
            limits: watch_limits,
            prepare_element,
            output: None,
        }));
        let completion =
            self.finish_synchronous_node(edit, source_limits, cancel, &mut execution)?;
        let Some(SynchronousNode::Watch(NodeWatch {
            output: Some((node, watch)),
            ..
        })) = execution.synchronous_node
        else {
            return Err(DomError::UnavailableCreation);
        };
        Ok(crate::DomWatchedNode::new(completion.commit, node, watch))
    }

    fn finish_synchronous_node(
        &mut self,
        edit: DomSourceEdit,
        source_limits: DomSourceLimits,
        cancel: &AtomicBool,
        execution: &mut Execution<'_>,
    ) -> Result<DomSourceCompletion, DomError> {
        let mut lease = self.begin_source_edit(edit, source_limits, cancel)?;
        match lease.advance_execution(execution, None, cancel) {
            Ok(DomSourceProgress::Complete(done)) if !done.cancellation_requested() => Ok(done),
            Ok(DomSourceProgress::Complete(_)) => Err(DomError::Cancelled),
            Ok(DomSourceProgress::Failed(failure)) => Err(failure.into_parts().1),
            Ok(DomSourceProgress::Request(_)) => Err(DomError::IncompleteSourceOperation),
            Err(AdvanceError::Dom(error)) => Err(error),
            Err(AdvanceError::Unwind(payload)) => {
                drop(lease);
                std::panic::resume_unwind(payload);
            }
            Err(_) => Err(DomError::InvalidSourceOperation),
        }
    }
}
