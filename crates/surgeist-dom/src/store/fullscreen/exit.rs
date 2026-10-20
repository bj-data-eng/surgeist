//! Fullscreen7c38 exit prefix. The live bank retains no mutable canonical Draft.
use super::*;
use crate::{
    DomFullscreenEvent, DomFullscreenEventBatch, DomFullscreenEventKind, DomFullscreenExitFact,
    DomFullscreenExitId, DomFullscreenExitReturn, DomFullscreenExitStatus,
};

#[derive(Clone, Copy)]
pub(in crate::store) struct Frame {
    pub document: Key,
    pub resize: bool,
    pub disconnected: Option<Key>,
    pub traversable: crate::DomHostedNavigableId,
    pub global: crate::DomHostedGlobalId,
    pub host: crate::DomHostedHostId,
    pub id: Option<DomFullscreenExitId>,
    pub ordinal: Option<u64>,
}
pub(in crate::store) struct Invocation {
    pub id: DomFullscreenExitId,
    pub document: DocumentId,
    pub resize: bool,
    pub operation: crate::DomSourceOperationId,
    pub occurrence: crate::source::watcher::HookOccurrence,
    pub traversable: crate::DomHostedNavigableId,
    pub global: crate::DomHostedGlobalId,
    pub host: crate::DomHostedHostId,
    pub work: Arc<AtomicU64>,
    pub ancestors: Option<Arc<crate::source::event::EventWork>>,
    pub work_custody: source::formation::SourceWorkCustody,
    pub retained_ancestors: Option<Arc<source::formation::EventWorkCustody>>,
    pub limits: crate::DomSourceLimits,
    pub closing_credit: u64,
}
impl Invocation {
    pub(in crate::store) fn step(&self, cancel: &AtomicBool) -> Result<(), DomError> {
        check_cancel(cancel)?;
        let next = self
            .work
            .load(Ordering::Relaxed)
            .checked_add(1)
            .ok_or(DomError::RepresentationExhausted)?;
        if next > self.limits.work {
            return Err(DomError::SourceWorkLimit {
                limit: self.limits.work,
            });
        }
        self.work.store(next, Ordering::Relaxed);
        crate::source::event::step_ancestors(self.ancestors.as_ref(), cancel)?;
        Ok(())
    }
    pub(in crate::store) fn status(&self) -> DomFullscreenExitStatus {
        DomFullscreenExitStatus {
            id: self.id,
            document: self.document,
            resize: self.resize,
            operation: self.operation,
            command: self.occurrence.command,
            occurrence: self.occurrence.ordinal,
        }
    }
}
impl Draft {
    fn fullscreen_simple(&self, document: Key, cancel: &AtomicBool) -> Result<bool, DomError> {
        let mut count = 0usize;
        for key in self.fullscreen_raw_members(document, cancel)?.iter() {
            self.source_step(cancel)?;
            count += usize::from(self.element(key)?.fullscreen_flag());
            if count > 1 {
                return Ok(false);
            }
        }
        Ok(count == 1)
    }
    fn fullscreen_composition(&self, document: Key, fact: DomFullscreenExitFact) -> DomError {
        DomError::FullscreenExitCompositionUnsupported {
            document: DocumentId(
                self.published_id(document)
                    .expect("active canonical Document"),
            ),
            fact,
        }
    }
    fn fullscreen_node_navigable(
        &self,
        document: Key,
        cancel: &AtomicBool,
    ) -> Result<crate::DomHostedNavigableId, DomError> {
        let id = DocumentId(
            self.published_id(document)
                .ok_or(DomError::HostedSourceUnavailable)?,
        );
        self.hosted_history
            .as_deref()
            .ok_or_else(|| {
                self.fullscreen_composition(document, DomFullscreenExitFact::NodeNavigable)
            })?
            .active_navigable_for_document_with(id, || self.source_step(cancel).map_err(Into::into))
            .map_err(|e| match e {
                crate::DomHostedError::Dom(e) => e,
                _ => self.fullscreen_composition(document, DomFullscreenExitFact::NodeNavigable),
            })?
            .ok_or_else(|| {
                self.fullscreen_composition(document, DomFullscreenExitFact::NodeNavigable)
            })
    }
    pub(in crate::store) fn exit_fullscreen_boundary(
        &mut self,
        document: Key,
        root: bool,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        if root && matches!(self.method, Some(method::Frame::FullscreenExit(Some(_)))) {
            return Ok(());
        }
        if self.fullscreen_exit.is_some() {
            return Err(DomError::FullscreenExitHandoffRequired);
        }
        // Source order: inactivity rejects without querying raw membership.
        if !self.document_fully_active(document, cancel)?
            || self.fullscreen_model_target(document, cancel)?.is_none()
        {
            if root {
                self.source_set_method(
                    Some(method::Frame::FullscreenExit(Some(
                        DomFullscreenExitReturn::RejectedTypeError,
                    ))),
                    cancel,
                )?;
            }
            return Ok(());
        }
        let mut documents = self.effect_vec();
        self.select_reserve(&mut documents, 1, cancel)?;
        documents.push(document);
        let graph = self
            .hosted_history
            .as_deref()
            .ok_or(DomError::HostedSourceUnavailable)?;
        let mut current = document;
        loop {
            if self.fullscreen_model_target(current, cancel)?.is_none() {
                return Err(DomError::FullscreenExitAssertionFailed {
                    document: DocumentId(
                        self.published_id(current)
                            .ok_or(DomError::HostedSourceUnavailable)?,
                    ),
                });
            }
            if !self.fullscreen_simple(current, cancel)? {
                break;
            }
            let nav = self.fullscreen_node_navigable(current, cancel)?;
            let nav = graph
                .navigable_with(nav, || self.source_step(cancel).map_err(Into::into))
                .map_err(|e| match e {
                    crate::DomHostedError::Dom(e) => e,
                    _ => self.fullscreen_composition(current, DomFullscreenExitFact::NodeNavigable),
                })?;
            let Some(container) = nav.container else {
                break;
            };
            let container = self.resolve(container.into()).map_err(|_| {
                self.fullscreen_composition(current, DomFullscreenExitFact::Container)
            })?;
            let element = self.element(container)?;
            if element.child_navigable() != Some(Some(nav.id)) {
                return Err(self.fullscreen_composition(current, DomFullscreenExitFact::Container));
            }
            if element.iframe_fullscreen_flag() == Some(true) {
                break;
            }
            current = self.node(container)?.document();
            let mut seen = false;
            for old in documents.iter() {
                self.source_step(cancel)?;
                seen |= *old == current;
            }
            if seen {
                return Err(self.fullscreen_composition(current, DomFullscreenExitFact::Container));
            }
            self.select_reserve(&mut documents, 1, cancel)?;
            documents.push(current);
        }
        let nav = self.fullscreen_node_navigable(document, cancel)?;
        let nav = graph
            .navigable_with(nav, || self.source_step(cancel).map_err(Into::into))
            .map_err(|e| match e {
                crate::DomHostedError::Dom(e) => e,
                _ => self.fullscreen_composition(document, DomFullscreenExitFact::TopLevelDocument),
            })?;
        let top = graph
            .navigable_with(nav.traversable, || {
                self.source_step(cancel).map_err(Into::into)
            })
            .map_err(|e| match e {
                crate::DomHostedError::Dom(e) => e,
                _ => self.fullscreen_composition(document, DomFullscreenExitFact::TopLevelDocument),
            })?;
        let top_document = graph
            .entry_with(top.active, || self.source_step(cancel).map_err(Into::into))
            .map_err(|e| match e {
                crate::DomHostedError::Dom(e) => e,
                _ => self.fullscreen_composition(document, DomFullscreenExitFact::TopLevelDocument),
            })?
            .document
            .ok_or_else(|| {
                self.fullscreen_composition(document, DomFullscreenExitFact::TopLevelDocument)
            })?;
        let top_key = self.resolve(top_document.into())?;
        let mut included = false;
        for &doc in documents.iter() {
            self.source_step(cancel)?;
            included |= doc == top_key;
        }
        let resize = included && self.fullscreen_simple(top_key, cancel)?;
        let selected = if resize { top_key } else { document };
        let selected_nav = self.fullscreen_node_navigable(selected, cancel)?;
        let selected_nav = graph
            .navigable_with(selected_nav, || {
                self.source_step(cancel).map_err(Into::into)
            })
            .map_err(|e| match e {
                crate::DomHostedError::Dom(e) => e,
                _ => self.fullscreen_composition(selected, DomFullscreenExitFact::NodeNavigable),
            })?;
        let target = self.fullscreen_model_target(selected, cancel)?.ok_or(
            DomError::FullscreenExitAssertionFailed {
                document: DocumentId(
                    self.published_id(selected)
                        .ok_or(DomError::HostedSourceUnavailable)?,
                ),
            },
        )?;
        let frame = Frame {
            document: selected,
            resize,
            disconnected: (!self.shadow_connected(target, cancel)?).then_some(target),
            traversable: nav.traversable,
            global: selected_nav.global,
            host: selected_nav.host,
            id: None,
            ordinal: None,
        };
        drop(documents);
        if root {
            self.source_set_method(Some(method::Frame::FullscreenExit(None)), cancel)?;
        }
        self.fullscreen_exit = Some(frame);
        Err(DomError::FullscreenExitHandoffRequired)
    }
    pub(in crate::store) fn fullscreen_exit_prefix(
        &mut self,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        let frame = self
            .fullscreen_exit
            .ok_or(DomError::InvalidSourceOperation)?;
        if let Some(subject) = frame.disconnected {
            let event = DomFullscreenEvent {
                document: DocumentId(
                    self.published_id(frame.document)
                        .ok_or(DomError::UnavailableCreation)?,
                ),
                subject: self
                    .published_id(subject)
                    .ok_or(DomError::UnavailableCreation)?,
                kind: DomFullscreenEventKind::Change,
            };
            let mut found = false;
            for old in self.node(frame.document)?.fullscreen_events().iter() {
                self.source_step(cancel)?;
                found |= old == event;
            }
            if !found {
                admit(
                    self.counts
                        .fullscreen_events
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?,
                    limits.fullscreen_events,
                    DomResource::FullscreenEvents,
                )?;
                self.fullscreen_events_push(frame.document, event, cancel)?;
            }
            self.unfullscreen_element_model(subject, cancel)?;
        }
        Ok(())
    }
}
impl Dom {
    pub(in crate::store) fn fullscreen_work_custody(
        &self,
        work: &Arc<AtomicU64>,
        step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<DomFullscreenExitId>, DomError> {
        self.fullscreen_exits.work_custody(work, step)
    }
    pub(in crate::store) fn fullscreen_custody_bytes(&self) -> Result<usize, DomError> {
        // Records, index paths and aliased work atoms are already represented
        // once by the common controlled ledger. This external envelope contains
        // only the owner field headers, never another logical root total.
        size_of::<super::custody::Invocations>()
            .checked_add(size_of::<u64>())
            .ok_or(DomError::RepresentationExhausted)
    }
    pub(in crate::store) fn fullscreen_custody_units(&self) -> Result<usize, DomError> {
        Ok(0)
    }
    pub fn fullscreen_exit_status(
        &self,
        id: DomFullscreenExitId,
    ) -> Result<DomFullscreenExitStatus, DomError> {
        if id.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        self.fullscreen_exits
            .get(id.slot)
            .filter(|entry| entry.id == id)
            .map(Invocation::status)
            .ok_or(DomError::StaleFullscreenExit)
    }
    pub fn take_fullscreen_events(
        &mut self,
        document: DocumentId,
        max_output: usize,
        cancel: &AtomicBool,
    ) -> Result<DomFullscreenEventBatch, DomError> {
        self.require_source_idle()?;
        check_cancel(cancel)?;
        if max_output > isize::MAX as usize / size_of::<DomFullscreenEvent>() {
            return Err(DomError::InvalidLimit(DomResource::FullscreenEvents));
        }
        let snapshot = self.snapshot();
        let view = snapshot.document(document)?;
        let basis = view.stamp();
        let events = view.pending_fullscreen_events()?;
        admit(events.len(), max_output, DomResource::FullscreenEvents)?;
        let mut out = Vec::new();
        out.try_reserve_exact(events.len())
            .map_err(|_| DomError::AllocationFailed)?;
        for event in events {
            check_cancel(cancel)?;
            out.push(*event);
        }
        let mut draft = Draft::new(self, DomBatch::new().owner, cancel)?;
        let key = draft.resolve(document.into())?;
        draft.fullscreen_events_clear(key, cancel)?;
        let prepared = draft.finish(self, cancel)?;
        let commit = self.publish(prepared, cancel)?;
        Ok(DomFullscreenEventBatch {
            basis,
            events: out,
            changes: commit.changes,
        })
    }
}
