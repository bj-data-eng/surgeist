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
    pub limits: crate::DomSourceLimits,
    pub closing_credit: u64,
}
impl Invocation {
    pub(in crate::store) fn retained_atoms(&self) -> Result<source::storage::Payload, DomError> {
        let mut payload = source::storage::Payload {
            bytes: size_of::<AtomicU64>() + 2 * size_of::<usize>(),
            units: 0,
        };
        let mut ancestor = self.ancestors.as_ref();
        while let Some(work) = ancestor {
            payload
                .add_bytes(size_of::<crate::source::event::EventWork>() + 2 * size_of::<usize>())?;
            payload.add_bytes(work.bytes.load(Ordering::Relaxed))?;
            payload.units = payload
                .units
                .checked_add(work.units.load(Ordering::Relaxed))
                .ok_or(DomError::RepresentationExhausted)?;
            ancestor = work.ancestors.as_ref();
        }
        Ok(payload)
    }
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
        for &key in self.fullscreen_raw_members(document, cancel)? {
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
                self.method = Some(method::Frame::FullscreenExit(Some(
                    DomFullscreenExitReturn::RejectedTypeError,
                )));
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
            current = self.node(container)?.document;
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
            self.method = Some(method::Frame::FullscreenExit(None));
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
            for old in &self.node(frame.document)?.fullscreen_events {
                self.source_step(cancel)?;
                found |= *old == event;
            }
            if !found {
                let mut count = 0usize;
                for node in self.existing.iter().chain(&self.created).flatten() {
                    self.source_step(cancel)?;
                    count = count
                        .checked_add(node.fullscreen_events.len())
                        .ok_or(DomError::RepresentationExhausted)?;
                }
                admit(
                    count
                        .checked_add(1)
                        .ok_or(DomError::RepresentationExhausted)?,
                    limits.fullscreen_events,
                    DomResource::FullscreenEvents,
                )?;
                let budget = source::formation::Formation::new(self, cancel)?;
                source::formation::reserve(
                    &mut self.node_mut(frame.document).fullscreen_events,
                    1,
                    budget.as_ref(),
                    cancel,
                )?;
                self.node_mut(frame.document).fullscreen_events.push(event);
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
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<Option<DomFullscreenExitId>, DomError> {
        for invocation in &self.fullscreen_exits {
            step()?;
            if Arc::ptr_eq(work, &invocation.work) {
                return Ok(Some(invocation.id));
            }
            let mut ancestor = invocation.ancestors.as_ref();
            while let Some(parent) = ancestor {
                step()?;
                if Arc::ptr_eq(work, &parent.work) {
                    return Ok(Some(invocation.id));
                }
                ancestor = parent.ancestors.as_ref();
            }
        }
        Ok(None)
    }
    pub(in crate::store) fn fullscreen_custody_bytes(&self) -> Result<usize, DomError> {
        let mut bytes = self
            .fullscreen_exits
            .capacity()
            .checked_mul(size_of::<Invocation>())
            .and_then(|bytes| bytes.checked_add(size_of::<Vec<Invocation>>() + size_of::<u64>()))
            .ok_or(DomError::RepresentationExhausted)?;
        for invocation in &self.fullscreen_exits {
            bytes = bytes
                .checked_add(invocation.retained_atoms()?.bytes)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        Ok(bytes)
    }
    pub(in crate::store) fn fullscreen_custody_units(&self) -> Result<usize, DomError> {
        let mut units = 0usize;
        for invocation in &self.fullscreen_exits {
            units = units
                .checked_add(invocation.retained_atoms()?.units)
                .ok_or(DomError::RepresentationExhausted)?;
        }
        Ok(units)
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
        draft.node_mut(key).fullscreen_events.clear();
        let prepared = draft.finish(self, cancel)?;
        let commit = self.publish(prepared, cancel)?;
        Ok(DomFullscreenEventBatch {
            basis,
            events: out,
            changes: commit.changes,
        })
    }
}
