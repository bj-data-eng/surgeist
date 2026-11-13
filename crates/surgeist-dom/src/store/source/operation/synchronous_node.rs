//! Synchronous Source creation and retirement custody for its node output.

use super::*;
use crate::{DomNodeRetirementInbox, DomNodeRetirementLimits, DomNodeRetirementWatch};

/// Retirement admission bound to the exact node output of one live operation.
pub struct DomSourceNodeOutputWatch<'a> {
    inbox: &'a DomNodeRetirementInbox,
    key: usize,
    limits: DomNodeRetirementLimits,
    operation: Option<DomSourceOperationId>,
    watch: Option<(NodeId, DomNodeRetirementWatch)>,
}

impl<'a> DomSourceNodeOutputWatch<'a> {
    pub const fn new(
        inbox: &'a DomNodeRetirementInbox,
        key: usize,
        limits: DomNodeRetirementLimits,
    ) -> Self {
        Self {
            inbox,
            key,
            limits,
            operation: None,
            watch: None,
        }
    }

    pub fn into_watch(self) -> Option<(NodeId, DomNodeRetirementWatch)> {
        self.watch
    }

    pub(super) fn admission(
        &mut self,
        operation: DomSourceOperationId,
    ) -> Result<NodeOutputWatchAdmission<'_>, DomError> {
        if self.watch.is_some() || self.operation.is_some_and(|owner| owner != operation) {
            return Err(DomError::InvalidSourceOperation);
        }
        self.operation = Some(operation);
        Ok(NodeOutputWatchAdmission {
            inbox: self.inbox,
            key: self.key,
            limits: self.limits,
            watch: &mut self.watch,
        })
    }
}

pub(super) struct NodeOutputWatchAdmission<'a> {
    inbox: &'a DomNodeRetirementInbox,
    key: usize,
    limits: DomNodeRetirementLimits,
    watch: &'a mut Option<(NodeId, DomNodeRetirementWatch)>,
}

impl NodeOutputWatchAdmission<'_> {
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
            self.key,
            self.limits,
            cancel,
        )?;
        *self.watch = Some((*node, watch));
        Ok(())
    }
}

pub(super) enum SynchronousNode<'a> {
    ReturnNode,
    Watch(NodeWatch<'a>),
    ElementFactory(NodeWatch<'a>),
    ReturnedCustomElement(&'a mut bool),
    CustomElementResult(NodeWatch<'a>),
}

/// The exact registration and its retained disconnection callback lookup.
/// The lookup must use the original definition identity and cannot enter guest code.
pub struct DomShadowReactionRoute<'a> {
    registration: &'a crate::DomDocumentScriptRegistration,
    queue: Option<crate::DomSynchronousReactionQueue>,
    has_callback: &'a mut dyn FnMut(crate::DomRegistryDefinitionId) -> Result<bool, DomError>,
}

impl<'a> DomShadowReactionRoute<'a> {
    pub fn new(
        registration: &'a crate::DomDocumentScriptRegistration,
        queue: Option<crate::DomSynchronousReactionQueue>,
        has_callback: &'a mut dyn FnMut(crate::DomRegistryDefinitionId) -> Result<bool, DomError>,
    ) -> Self {
        Self {
            registration,
            queue,
            has_callback,
        }
    }
    fn reborrow(&mut self) -> DomShadowReactionRoute<'_> {
        DomShadowReactionRoute {
            registration: self.registration,
            queue: self.queue,
            has_callback: &mut *self.has_callback,
        }
    }
}

pub(super) struct ShadowReactionRoute<'a> {
    pub(super) route: DomShadowReactionRoute<'a>,
    pub(super) published: &'a mut bool,
}

impl ShadowReactionRoute<'_> {
    pub(super) fn prepare(
        &mut self,
        reactions: &mut super::super::reactions::Publication,
        formation: &formation::Formation,
        cancel: &AtomicBool,
    ) -> Result<bool, DomError> {
        let present = reactions.route_shadow_removal(
            self.route.registration,
            self.route.queue,
            &mut self.route.has_callback,
            formation,
            cancel,
        )?;
        *self.published = present;
        Ok(present)
    }
}

/// Result of advancing a detached Element factory with retirement custody.
/// A reached constructor retains its original Source operation in the DOM.
pub enum DomWatchedElementCreation<T> {
    Created(crate::DomWatchedNode, T),
    Constructor(DomSourceCustomElementRequest),
}

/// Final output of an autonomous element factory with wrapper custody.
pub enum DomWatchedCustomElementResult<T> {
    Returned(NodeId),
    Fallback {
        node: NodeId,
        watch: DomNodeRetirementWatch,
        projection: T,
        disposition: crate::DomSourceCustomElementDisposition,
    },
}

/// A loan of the exact canonical name prepared for a detached Element. Work
/// on this loan belongs to the original Source operation and its observers.
pub struct DomPreparedElementName<'a> {
    name: &'a crate::DomQualifiedName,
    formation: &'a formation::Formation,
    cancel: &'a AtomicBool,
    custom_element_disposition: Option<crate::DomSourceCustomElementDisposition>,
}

impl DomPreparedElementName<'_> {
    pub const fn name(&self) -> &crate::DomQualifiedName {
        self.name
    }

    pub fn step(&self) -> Result<(), DomError> {
        self.formation.step(self.cancel)
    }

    pub const fn custom_element_disposition(
        &self,
    ) -> Option<crate::DomSourceCustomElementDisposition> {
        self.custom_element_disposition
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
        custom_element_disposition: Option<crate::DomSourceCustomElementDisposition>,
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
                    custom_element_disposition,
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
    /// A reached constructor returns before calling the preparation closure.
    /// The caller can resume the request or discard it before opening its scope.
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
    ) -> Result<DomWatchedElementCreation<T>, E> {
        let edit = DomSourceEdit::create_element_with_options(document.node(), name, options);
        let mut prepare = Some(prepare);
        let mut output = None;
        let mut prepare_element = |name: DomPreparedElementName<'_>| {
            output = prepare.take().map(|prepare| prepare(name));
            matches!(output, Some(Ok(_)))
        };
        let mut execution = Execution::disabled();
        execution.synchronous_node = Some(SynchronousNode::ElementFactory(NodeWatch {
            inbox,
            custody_key,
            limits: limits.1,
            prepare_element: Some(&mut prepare_element),
            output: None,
        }));
        let mut lease = self.begin_source_edit(edit, limits.0, cancel)?;
        let progress = lease.advance_node(&mut execution, cancel);
        let Some(SynchronousNode::ElementFactory(watch)) = execution.synchronous_node.take() else {
            return Err(DomError::InvalidSourceOperation.into());
        };
        match (progress, watch.output, output) {
            (Ok(DomSourceProgress::Complete(done)), Some((node, watch)), Some(Ok(output))) => {
                Ok(DomWatchedElementCreation::Created(
                    crate::DomWatchedNode::new(done.commit, node, watch),
                    output,
                ))
            }
            (
                Ok(DomSourceProgress::Request(DomSourceRequest::CustomElementConstruction(
                    request,
                ))),
                None,
                None,
            ) => Ok(DomWatchedElementCreation::Constructor(request)),
            (_, _, Some(Err(error))) => Err(error),
            (Err(error), _, _) => Err(error.into()),
            _ => Err(DomError::IncompleteSourceOperation.into()),
        }
    }

    /// Attach or reuse a ShadowRoot through a single complete Source publication.
    /// Operations reaching an external service require the ordinary Source lease.
    /// If removal creates custom-element reactions, the original active script
    /// registration is required. The result indicates whether it received a
    /// backup reaction segment.
    pub fn attach_shadow_synchronously(
        &mut self,
        host: NodeId,
        init: crate::DomShadowRootInit,
        source_limits: DomSourceLimits,
        mut route: Option<DomShadowReactionRoute<'_>>,
        cancel: &AtomicBool,
    ) -> Result<(DomCommit, NodeId, bool), DomError> {
        let mut reactions = false;
        let mut execution = Execution::disabled();
        execution.synchronous_node = Some(SynchronousNode::ReturnNode);
        execution.shadow_reactions = route.as_mut().map(|route| ShadowReactionRoute {
            route: route.reborrow(),
            published: &mut reactions,
        });
        let completed = self.finish_synchronous_node(
            DomSourceEdit::attach_shadow(host, init),
            source_limits,
            cancel,
            &mut execution,
        )?;
        let Some(DomSourceOutput::Node(root)) = completed.output() else {
            return Err(DomError::InvalidSourceOperation);
        };
        Ok((completed.commit, root, reactions))
    }

    /// Attach or reuse a ShadowRoot and arm its exact retirement watch in the
    /// same Source publication. Refusal leaves the original host relation and
    /// retirement index intact. A reaction-producing reuse requires the
    /// original active script registration and reports whether it received reactions.
    pub fn attach_shadow_with_node_retirement_watch(
        &mut self,
        host: NodeId,
        init: crate::DomShadowRootInit,
        watch: (&DomNodeRetirementInbox, usize),
        (source_limits, watch_limits): (DomSourceLimits, DomNodeRetirementLimits),
        route: Option<DomShadowReactionRoute<'_>>,
        cancel: &AtomicBool,
    ) -> Result<(crate::DomWatchedNode, bool), DomError> {
        self.publish_watched_source_node(
            DomSourceEdit::attach_shadow(host, init),
            watch,
            (source_limits, watch_limits),
            route,
            cancel,
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
            None,
            cancel,
        )
        .map(|(watched, _)| watched)
    }

    fn publish_watched_source_node(
        &mut self,
        edit: DomSourceEdit,
        (inbox, custody_key): (&DomNodeRetirementInbox, usize),
        (source_limits, watch_limits): (DomSourceLimits, DomNodeRetirementLimits),
        mut route: Option<DomShadowReactionRoute<'_>>,
        cancel: &AtomicBool,
    ) -> Result<(crate::DomWatchedNode, bool), DomError> {
        let mut reactions = false;
        let mut execution = Execution::disabled();
        execution.shadow_reactions = route.as_mut().map(|route| ShadowReactionRoute {
            route: route.reborrow(),
            published: &mut reactions,
        });
        execution.synchronous_node = Some(SynchronousNode::Watch(NodeWatch {
            inbox,
            custody_key,
            limits: watch_limits,
            prepare_element: None,
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
        Ok((
            crate::DomWatchedNode::new(completion.commit, node, watch),
            reactions,
        ))
    }

    fn finish_synchronous_node(
        &mut self,
        edit: DomSourceEdit,
        source_limits: DomSourceLimits,
        cancel: &AtomicBool,
        execution: &mut Execution<'_>,
    ) -> Result<DomSourceCompletion, DomError> {
        let mut lease = self.begin_source_edit(edit, source_limits, cancel)?;
        match lease.advance_node(execution, cancel)? {
            DomSourceProgress::Complete(done) => Ok(done),
            DomSourceProgress::Request(_) => Err(DomError::IncompleteSourceOperation),
            DomSourceProgress::Failed(failure) => Err(failure.into_parts().1),
        }
    }
}

impl DomSourceLease<'_> {
    /// Settle a returned autonomous receiver without preparing fallback custody.
    /// `None` leaves the original accepted result pending for watched fallback.
    pub fn finish_returned_custom_element(
        &mut self,
        request: DomSourceCustomElementRequest,
        cancel: &AtomicBool,
    ) -> Result<Option<NodeId>, DomError> {
        self.qualify_autonomous_result(request)?;
        let mut fallback = false;
        let mut execution = Execution::disabled();
        execution.synchronous_node = Some(SynchronousNode::ReturnedCustomElement(&mut fallback));
        let progress = self.advance_node(&mut execution, cancel);
        if fallback {
            return Ok(None);
        }
        match progress? {
            DomSourceProgress::Complete(done)
                if done.custom_element_disposition()
                    == Some(crate::DomSourceCustomElementDisposition::Constructed) =>
            {
                match done.output() {
                    Some(DomSourceOutput::Node(node)) => Ok(Some(node)),
                    _ => Err(DomError::InvalidSourceOperation),
                }
            }
            _ => Err(DomError::IncompleteSourceOperation),
        }
    }

    /// Finish the original autonomous factory. Only a distinct fallback requires a new
    /// wrapper and watch. Any preparation refusal leaves the accepted result
    /// in the original operation.
    pub fn finish_custom_element_with_watch<T, E: From<DomError>>(
        &mut self,
        request: DomSourceCustomElementRequest,
        (inbox, custody_key): (&DomNodeRetirementInbox, usize),
        limits: DomNodeRetirementLimits,
        cancel: &AtomicBool,
        prepare: impl FnOnce(DomPreparedElementName<'_>) -> Result<T, E>,
    ) -> Result<DomWatchedCustomElementResult<T>, E> {
        self.qualify_autonomous_result(request)?;
        let mut prepare = Some(prepare);
        let mut output = None;
        let mut prepare_element = |name: DomPreparedElementName<'_>| {
            output = prepare.take().map(|prepare| prepare(name));
            matches!(output, Some(Ok(_)))
        };
        let mut execution = Execution::disabled();
        execution.synchronous_node = Some(SynchronousNode::CustomElementResult(NodeWatch {
            inbox,
            custody_key,
            limits,
            prepare_element: Some(&mut prepare_element),
            output: None,
        }));
        let progress = self.advance_node(&mut execution, cancel);
        let Some(SynchronousNode::CustomElementResult(NodeWatch {
            output: watched, ..
        })) = execution.synchronous_node.take()
        else {
            return Err(DomError::InvalidSourceOperation.into());
        };
        if let Some(Err(error)) = output {
            return Err(error);
        }
        let DomSourceProgress::Complete(done) = progress? else {
            return Err(DomError::IncompleteSourceOperation.into());
        };
        match (
            done.custom_element_disposition(),
            done.output(),
            watched,
            output,
        ) {
            (
                Some(crate::DomSourceCustomElementDisposition::Constructed),
                Some(DomSourceOutput::Node(node)),
                None,
                None,
            ) => Ok(DomWatchedCustomElementResult::Returned(node)),
            (
                Some(disposition),
                Some(DomSourceOutput::Node(result)),
                Some((node, watch)),
                Some(Ok(projection)),
            ) if node == result => Ok(DomWatchedCustomElementResult::Fallback {
                node,
                watch,
                projection,
                disposition,
            }),
            _ => Err(DomError::InvalidSourceOperation.into()),
        }
    }

    fn qualify_autonomous_result(
        &self,
        request: DomSourceCustomElementRequest,
    ) -> Result<(), DomError> {
        if self
            .dom
            .source_operation
            .as_deref()
            .filter(|operation| operation.id == self.id)
            .and_then(|operation| operation.custom_element.as_ref())
            .is_some_and(|pending| pending.accepted_autonomous(request))
        {
            Ok(())
        } else {
            Err(DomError::InvalidSourceOperation)
        }
    }

    fn advance_node(
        &mut self,
        execution: &mut Execution<'_>,
        cancel: &AtomicBool,
    ) -> Result<DomSourceProgress, DomError> {
        match self.advance_execution(execution, None, cancel) {
            Ok(DomSourceProgress::Complete(done)) if done.cancellation_requested() => {
                Err(DomError::Cancelled)
            }
            Ok(DomSourceProgress::Failed(failure)) => Err(failure.into_parts().1),
            Ok(progress) => Ok(progress),
            Err(AdvanceError::Dom(error)) => Err(error),
            Err(AdvanceError::Unwind(payload)) => std::panic::resume_unwind(payload),
            Err(_) => Err(DomError::InvalidSourceOperation),
        }
    }
}
