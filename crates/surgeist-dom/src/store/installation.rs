//! Fresh-native immutable HTML adaptation; one canonical Draft and publication.
use super::*;
use crate::{
    DomDocumentStamp, DomNativeHtmlAssociation as Association, DomNativeHtmlBinding,
    DomNativeHtmlInstallError as Error, DomNativeHtmlInstallLimits,
    DomNativeHtmlInstallResource as Resource, DomNativeHtmlInstallation,
};
use surgeist_html::{
    HtmlFragment, HtmlFragmentOwner, HtmlNodeId, HtmlNodeKind, HtmlTemplateContentsKind,
};

struct Visit {
    source: HtmlNodeId,
    parent: Option<Key>,
    existing: Option<Key>,
    owner: Option<HtmlNodeId>,
}

pub(super) struct Row {
    pub source: HtmlNodeId,
    pub key: Key,
}
pub(super) struct NativeInstallation {
    pub rows: Vec<Row>,
    pub rows_payload: Option<source::checkpoint::Charge>,
    pub bindings: Vec<DomNativeHtmlBinding>,
    pub bindings_payload: Option<source::checkpoint::Charge>,
    pub destination: DomDocumentStamp,
    pub source_root: HtmlNodeId,
    pub wrapper: Key,
    pub parent: Key,
    pub before: Option<Key>,
    pub limits: DomNativeHtmlInstallLimits,
    pub work: usize,
    pub bytes: usize,
}
impl NativeInstallation {
    /// The immutable native close budget and real output capacity must exist
    /// before Source can publish any externally observable phase. The ordinary
    /// installation path retains its original finish tariff below.
    pub(super) fn prepare_source_closing(
        &mut self,
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<(), Error> {
        let mut control = Control {
            limits: self.limits,
            work: self.work,
            bytes: self.bytes,
            cancel,
            source_work: None,
        };
        control.charge(self.rows.len())?;
        control.reserve_owned(
            draft,
            &mut self.bindings,
            &mut self.bindings_payload,
            self.rows.len(),
        )?;
        self.work = control.work;
        self.bytes = control.bytes;
        Ok(())
    }
}
pub(super) struct StagedInstallation {
    pub draft: Draft,
    pub continuation: NativeInstallation,
}
struct PreparedInstallation {
    prepared: Prepared,
    destination: DomDocumentStamp,
    source_root: HtmlNodeId,
    root: NodeId,
    bindings: Vec<DomNativeHtmlBinding>,
}
struct Control<'a> {
    limits: DomNativeHtmlInstallLimits,
    work: usize,
    bytes: usize,
    cancel: &'a AtomicBool,
    // Work only: storage admission always uses a fresh current-Draft Formation.
    source_work: Option<source::formation::Formation>,
}
impl Control<'_> {
    fn check(&self) -> Result<(), Error> {
        if self.cancel.load(Ordering::Acquire) {
            Err(Error::Cancelled)
        } else {
            Ok(())
        }
    }
    fn charge(&mut self, amount: usize) -> Result<(), Error> {
        self.check()?;
        self.work = self
            .work
            .checked_add(amount)
            .ok_or(Error::RepresentationExhausted)?;
        if self.work > self.limits.work {
            return Err(Error::ResourceLimit {
                resource: Resource::Work,
                limit: self.limits.work,
            });
        }
        if let Some(source) = &self.source_work {
            source.step(self.cancel).map_err(Error::Target)?;
        }
        Ok(())
    }
    fn retain(&mut self, bytes: usize) -> Result<(), Error> {
        self.check()?;
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or(Error::RepresentationExhausted)?;
        if self.bytes > self.limits.scratch_bytes {
            return Err(Error::ResourceLimit {
                resource: Resource::ScratchBytes,
                limit: self.limits.scratch_bytes,
            });
        }
        Ok(())
    }
    fn reserve<T>(&mut self, values: &mut Vec<T>, additional: usize) -> Result<(), Error> {
        let desired = values
            .len()
            .checked_add(additional)
            .ok_or(Error::RepresentationExhausted)?;
        if desired <= values.capacity() {
            self.check()?;
            return Ok(());
        }
        let old = values.capacity();
        let bytes = (desired - old)
            .checked_mul(size_of::<T>())
            .ok_or(Error::RepresentationExhausted)?;
        self.retain(bytes)?;
        values
            .try_reserve_exact(additional)
            .map_err(|_| Error::AllocationFailed)?;
        let extra = (values.capacity() - desired)
            .checked_mul(size_of::<T>())
            .ok_or(Error::RepresentationExhausted)?;
        self.retain(extra)
    }
    fn release<T>(&mut self, values: Vec<T>) {
        self.bytes -= values.capacity() * size_of::<T>();
        drop(values);
    }
    fn reserve_owned<T>(
        &mut self,
        draft: &Draft,
        values: &mut Vec<T>,
        payload: &mut Option<source::checkpoint::Charge>,
        additional: usize,
    ) -> Result<(), Error> {
        if draft.source_limits.is_none() {
            return self.reserve(values, additional);
        }
        self.check()?;
        let desired = values
            .len()
            .checked_add(additional)
            .ok_or(Error::RepresentationExhausted)?;
        if desired <= values.capacity() {
            draft.source_step(self.cancel).map_err(Error::Target)?;
            return Ok(());
        }
        let old = values.capacity();
        let net = (desired - old)
            .checked_mul(size_of::<T>())
            .ok_or(Error::RepresentationExhausted)?;
        // Preserve the original native domain's first-stop priority and exact
        // reservation policy. Source additionally admits the whole replacement.
        self.retain(net)?;
        let formation =
            source::formation::Formation::new(draft, self.cancel).map_err(Error::Target)?;
        let budget = formation.as_ref().expect("Source staging");
        budget
            .retain_bytes(
                desired
                    .checked_mul(size_of::<T>())
                    .ok_or(Error::RepresentationExhausted)?,
                self.cancel,
            )
            .map_err(Error::Target)?;
        values
            .try_reserve_exact(additional)
            .map_err(|_| Error::AllocationFailed)?;
        let extra = (values.capacity() - desired)
            .checked_mul(size_of::<T>())
            .ok_or(Error::RepresentationExhausted)?;
        self.retain(extra)?;
        budget
            .retain_bytes(extra, self.cancel)
            .map_err(Error::Target)?;
        // The old allocation has now actually gone. Replace its singular debit
        // with the new bank, without an unrepresented ownership interval.
        let previous = payload.replace(formation.expect("Source staging").into_charge());
        drop(previous);
        Ok(())
    }
    fn map_row(
        &mut self,
        draft: &Draft,
        rows: &mut Vec<Row>,
        payload: &mut Option<source::checkpoint::Charge>,
        source: HtmlNodeId,
        key: Key,
    ) -> Result<(), Error> {
        self.admit_node(rows.len())?;
        for row in rows.iter() {
            self.charge(1)?;
            if row.source == source {
                return Err(Error::UnsupportedAssociation {
                    subject: source,
                    association: Association::TemplateOwnership,
                });
            }
        }
        self.charge(1)?;
        self.reserve_owned(draft, rows, payload, 1)?;
        rows.push(Row { source, key });
        Ok(())
    }
    fn admit_node(&self, previous: usize) -> Result<(), Error> {
        let count = previous
            .checked_add(1)
            .ok_or(Error::RepresentationExhausted)?;
        if count > self.limits.mapped_nodes {
            return Err(Error::ResourceLimit {
                resource: Resource::MappedNodes,
                limit: self.limits.mapped_nodes,
            });
        }
        Ok(())
    }
    fn name(&mut self, name: &crate::QualifiedName) -> Result<(), Error> {
        self.charge(crate::names::name_units(name).ok_or(Error::RepresentationExhausted)?)
    }
}
fn source_error(subject: &HtmlNodeId, error: DomError) -> Error {
    Error::Source {
        subject: subject.clone(),
        error,
    }
}
fn association(subject: &HtmlNodeId, kind: Association) -> Error {
    Error::UnsupportedAssociation {
        subject: subject.clone(),
        association: kind,
    }
}

pub(super) struct Budgets {
    pub native: DomNativeHtmlInstallLimits,
    pub source: Option<crate::DomSourceLimits>,
}
impl Dom {
    /// Installs exact immutable fields using fresh native creation, then actual
    /// fragment transfer, in one canonical publication. Passing a bare parsed
    /// fragment deliberately selects this native cause; it does not transfer a
    /// parse report's context, metadata, source provenance or effects.
    ///
    /// The stamp must match the exact live destination. Source Template owned
    /// contents reuse actual native auxiliary ownership. Declarative/retained
    /// associations need their own established facts and are refused precisely.
    pub fn install_native_html_fragment(
        &mut self,
        basis: DomDocumentStamp,
        parent: NodeId,
        before: Option<NodeId>,
        source: &HtmlFragment,
        limits: DomNativeHtmlInstallLimits,
        cancelled: &AtomicBool,
    ) -> Result<DomNativeHtmlInstallation, Error> {
        self.require_source_idle().map_err(Error::Target)?;
        let prepared = self
            .prepare_native_html_installation(basis, parent, before, source, limits, cancelled)?;
        self.publish_native_html_installation(prepared, cancelled)
    }

    pub(super) fn stage_native_html_installation(
        &self,
        basis: DomDocumentStamp,
        parent: NodeId,
        before: Option<NodeId>,
        source: &HtmlFragment,
        budgets: Budgets,
        cancelled: &AtomicBool,
    ) -> Result<StagedInstallation, Error> {
        let Budgets {
            native: limits,
            source: source_limits,
        } = budgets;
        let mut control = Control {
            limits,
            work: 0,
            bytes: 0,
            cancel: cancelled,
            source_work: None,
        };
        control.check()?;
        let current = self.snapshot();
        if basis.store().owner() != self.owner {
            return Err(Error::Target(DomError::ForeignOwner));
        }
        if basis.store() != current.stamp() {
            return Err(Error::StaleDestination {
                expected: basis,
                actual: current.stamp(),
            });
        }
        let document = current.document(basis.document()).map_err(Error::Target)?;
        let target = document.node(parent).map_err(Error::Target)?;
        if !tree::parent_can_have_children(target.kind()) {
            return Err(Error::Target(DomError::HierarchyRequest));
        }
        if let Some(before) = before {
            let child = current.node(before).map_err(Error::Target)?;
            if child.parent() != Some(parent) {
                return Err(Error::Target(DomError::NotFound));
            }
        }
        let mut draft = match source_limits {
            Some(limits) => Draft::new_source(
                self,
                Arc::new(BatchIdentity),
                &Vec::new(),
                limits,
                cancelled,
            )
            .map(|(draft, _, _)| draft)
            .map_err(Error::Target)?,
            None => Draft::new(self, Arc::new(BatchIdentity), cancelled).map_err(Error::Target)?,
        };
        control.source_work =
            source::formation::Formation::new(&draft, cancelled).map_err(Error::Target)?;
        let document = draft
            .resolve(basis.document().into())
            .map_err(Error::Target)?;
        let parent = draft.resolve(parent.into()).map_err(Error::Target)?;
        let before = before
            .map(|node| draft.resolve(node.into()))
            .transpose()
            .map_err(Error::Target)?;
        let wrapper = draft
            .allocate(
                DomNodeKind::DocumentFragment,
                document,
                self.limits,
                cancelled,
            )
            .map_err(Error::Target)?;
        let source_root = source.root().id();
        let pending_header =
            source::formation::Formation::new(&draft, cancelled).map_err(Error::Target)?;
        if let Some(budget) = &pending_header {
            budget
                .retain_bytes(size_of::<Vec<Visit>>(), cancelled)
                .map_err(Error::Target)?;
        }
        let mut pending_payload = None;
        let mut pending = Vec::new();
        #[cfg(test)]
        let pending_growth =
            staging_observation::before(&draft, &pending, pending_payload.is_some(), cancelled)
                .map_err(Error::Target)?;
        control.reserve_owned(&draft, &mut pending, &mut pending_payload, 1)?;
        #[cfg(test)]
        staging_observation::grew(pending_growth, &pending);
        pending.push(Visit {
            source: source_root.clone(),
            parent: None,
            existing: Some(wrapper),
            owner: None,
        });
        let mut rows_payload = None;
        let mut rows = Vec::new();
        while let Some(visit) = pending.pop() {
            control.charge(1)?;
            control.admit_node(rows.len())?;
            let original = source
                .node(&visit.source)
                .map_err(|error| Error::SourceLookup {
                    subject: visit.source.clone(),
                    error,
                })?;
            // All association observations are explicit, even on childless roots.
            control.charge(5)?;
            let shadow = original.shadow_properties();
            let declarative = original.declarative_shadow_root();
            let owner = original.fragment_owner();
            let contents_kind = original.template_contents_kind();
            let contents = original.template_contents();
            if shadow.is_some() || declarative.is_some() {
                return Err(association(&visit.source, Association::DeclarativeShadow));
            }
            match (owner, &visit.owner) {
                (None, None) => {}
                (Some(HtmlFragmentOwner::Template(actual)), Some(expected)) => {
                    control.charge(1)?;
                    if actual != expected {
                        return Err(association(&visit.source, Association::TemplateOwnership));
                    }
                }
                _ => return Err(association(&visit.source, Association::HostedSourceRoot)),
            }
            if contents_kind == Some(HtmlTemplateContentsKind::ShadowReference) {
                return Err(association(
                    &visit.source,
                    Association::TemplateShadowReference,
                ));
            }
            let key = if let Some(existing) = visit.existing {
                if !matches!(original.kind(), HtmlNodeKind::Fragment) {
                    return Err(source_error(&visit.source, DomError::WrongNodeKind));
                }
                existing
            } else {
                let parent = visit
                    .parent
                    .expect("ordinary source child has a staged parent");
                let document = draft.node(parent).map_err(Error::Target)?.document;
                let formation =
                    source::formation::Formation::new(&draft, cancelled).map_err(Error::Target)?;
                let kind = match original.kind() {
                    HtmlNodeKind::Element(element) => {
                        control.name(element.name())?;
                        DomNodeKind::Element(
                            DomElement::native_components(
                                element.name(),
                                draft
                                    .document_environment(document)
                                    .map_err(Error::Target)?,
                                cancelled,
                                formation.as_ref(),
                            )
                            .map_err(|error| source_error(&visit.source, error))?,
                        )
                    }
                    HtmlNodeKind::Text(value) => {
                        control.charge(value.len())?;
                        DomNodeKind::Text(
                            source::formation::share(value, formation.as_ref(), cancelled)
                                .map_err(|error| source_error(&visit.source, error))?,
                        )
                    }
                    HtmlNodeKind::Comment(value) => {
                        control.charge(value.len())?;
                        DomNodeKind::Comment(
                            source::formation::share(value, formation.as_ref(), cancelled)
                                .map_err(|error| source_error(&visit.source, error))?,
                        )
                    }
                    HtmlNodeKind::ProcessingInstruction(value) => {
                        control.charge(
                            value
                                .target()
                                .len()
                                .checked_add(value.data().len())
                                .ok_or(Error::RepresentationExhausted)?,
                        )?;
                        DomNodeKind::ProcessingInstruction(
                            crate::DomProcessingInstruction::new(
                                value.target(),
                                value.data(),
                                formation.as_ref(),
                                cancelled,
                            )
                            .map_err(|error| source_error(&visit.source, error))?,
                        )
                    }
                    HtmlNodeKind::Doctype(value) => {
                        control.charge(
                            value
                                .name()
                                .len()
                                .checked_add(value.public_id().map_or(0, DocumentString::len))
                                .and_then(|n| {
                                    n.checked_add(value.system_id().map_or(0, DocumentString::len))
                                })
                                .ok_or(Error::RepresentationExhausted)?,
                        )?;
                        let public = value
                            .public_id()
                            .is_none()
                            .then(|| source::formation::string("", formation.as_ref(), cancelled))
                            .transpose()
                            .map_err(Error::Target)?;
                        let system = value
                            .system_id()
                            .is_none()
                            .then(|| source::formation::string("", formation.as_ref(), cancelled))
                            .transpose()
                            .map_err(Error::Target)?;
                        let kind = crate::DomDocumentType::new(
                            value.name(),
                            value
                                .public_id()
                                .or(public.as_ref())
                                .expect("actual public ID"),
                            value
                                .system_id()
                                .or(system.as_ref())
                                .expect("actual system ID"),
                            formation.as_ref(),
                            cancelled,
                        )
                        .map_err(|error| source_error(&visit.source, error))?;
                        for temporary in [public, system].into_iter().flatten() {
                            let mut payload = source::storage::Payload::default();
                            payload.string(&temporary).map_err(Error::Target)?;
                            drop(temporary);
                            if let Some(budget) = &formation {
                                budget.release_payload(payload);
                            }
                        }
                        DomNodeKind::DocumentType(kind)
                    }
                    HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment => {
                        return Err(source_error(&visit.source, DomError::HierarchyRequest));
                    }
                };
                let key = draft
                    .allocate(kind, document, self.limits, cancelled)
                    .map_err(|error| source_error(&visit.source, error))?;
                drop(formation);
                draft
                    .initialize_native_element(key, self.limits, cancelled)
                    .map_err(|error| source_error(&visit.source, error))?;
                key
            };
            control.map_row(
                &draft,
                &mut rows,
                &mut rows_payload,
                visit.source.clone(),
                key,
            )?;
            if let HtmlNodeKind::Element(element) = original.kind() {
                for attribute in element.attributes().iter() {
                    control.charge(1)?;
                    control.name(attribute.name())?;
                    control.charge(attribute.value().len())?;
                    crate::names::validate_attribute_components(attribute.name(), &mut || {
                        draft.source_step(cancelled)
                    })
                    .map_err(|error| source_error(&visit.source, error))?;
                    draft
                        .append_attribute(
                            key,
                            attribute,
                            attributes::AttributeCause::Change,
                            self.limits,
                            cancelled,
                        )
                        .map_err(|error| source_error(&visit.source, error))?;
                }
            }
            if let Some(parent) = visit.parent {
                draft
                    .pre_insert(key, parent, None, self.limits, cancelled)
                    .map_err(|error| source_error(&visit.source, error))?;
            }
            for child in original.children().rev() {
                control.charge(1)?;
                #[cfg(test)]
                let pending_growth = staging_observation::before(
                    &draft,
                    &pending,
                    pending_payload.is_some(),
                    cancelled,
                )
                .map_err(Error::Target)?;
                control.reserve_owned(&draft, &mut pending, &mut pending_payload, 1)?;
                #[cfg(test)]
                staging_observation::grew(pending_growth, &pending);
                pending.push(Visit {
                    source: child,
                    parent: Some(key),
                    existing: None,
                    owner: None,
                });
            }
            match (contents, contents_kind) {
                (None, None) => {}
                (Some(contents), Some(HtmlTemplateContentsKind::Owned)) => {
                    control.charge(1)?;
                    let target = draft
                        .node(key)
                        .map_err(Error::Target)?
                        .template_contents
                        .ok_or_else(|| {
                            association(&visit.source, Association::TemplateOwnership)
                        })?;
                    #[cfg(test)]
                    let pending_growth = staging_observation::before(
                        &draft,
                        &pending,
                        pending_payload.is_some(),
                        cancelled,
                    )
                    .map_err(Error::Target)?;
                    control.reserve_owned(&draft, &mut pending, &mut pending_payload, 1)?;
                    #[cfg(test)]
                    staging_observation::grew(pending_growth, &pending);
                    pending.push(Visit {
                        source: contents,
                        parent: None,
                        existing: Some(target),
                        owner: Some(visit.source),
                    });
                }
                _ => return Err(association(&visit.source, Association::TemplateOwnership)),
            }
        }
        control.release(pending);
        drop(pending_payload);
        drop(pending_header);
        Ok(StagedInstallation {
            draft,
            continuation: NativeInstallation {
                rows,
                rows_payload,
                bindings: Vec::new(),
                bindings_payload: None,
                destination: basis,
                source_root,
                wrapper,
                parent,
                before,
                limits,
                work: control.work,
                bytes: control.bytes,
            },
        })
    }

    fn prepare_native_html_installation(
        &self,
        basis: DomDocumentStamp,
        parent: NodeId,
        before: Option<NodeId>,
        source: &HtmlFragment,
        limits: DomNativeHtmlInstallLimits,
        cancelled: &AtomicBool,
    ) -> Result<PreparedInstallation, Error> {
        let StagedInstallation {
            mut draft,
            continuation,
        } = self.stage_native_html_installation(
            basis,
            parent,
            before,
            source,
            Budgets {
                native: limits,
                source: None,
            },
            cancelled,
        )?;
        let NativeInstallation {
            rows,
            source_root,
            wrapper,
            parent,
            before,
            work,
            bytes,
            ..
        } = continuation;
        let mut control = Control {
            limits,
            work,
            bytes,
            cancel: cancelled,
            source_work: None,
        };
        draft
            .pre_insert(wrapper, parent, before, self.limits, cancelled)
            .map_err(Error::Target)?;
        // Admit and materialize every output while the canonical state is still
        // private. Exact final IDs come from the existing finish map, not tickets.
        let count = rows.len();
        control.charge(count)?;
        let mapped_bytes = count
            .checked_mul(size_of::<NodeId>())
            .ok_or(Error::RepresentationExhausted)?;
        control.retain(mapped_bytes)?;
        let (prepared, mapped) = draft
            .finish_with_mapping(self, cancelled, rows.iter().map(|row| row.key))
            .map_err(Error::Target)?;
        let extra = mapped
            .capacity()
            .checked_sub(count)
            .and_then(|n| n.checked_mul(size_of::<NodeId>()))
            .ok_or(Error::RepresentationExhausted)?;
        control.retain(extra)?;
        let root = mapped[0];
        let mut bindings = Vec::new();
        control.reserve(&mut bindings, count)?;
        for (row, node) in rows.iter().zip(mapped.iter()) {
            control.charge(1)?;
            bindings.push(DomNativeHtmlBinding {
                source: row.source.clone(),
                node: *node,
            });
        }
        control.release(rows);
        control.release(mapped);
        control.check()?;
        Ok(PreparedInstallation {
            prepared,
            destination: basis,
            source_root,
            root,
            bindings,
        })
    }

    fn publish_native_html_installation(
        &mut self,
        pending: PreparedInstallation,
        cancelled: &AtomicBool,
    ) -> Result<DomNativeHtmlInstallation, Error> {
        let commit = self
            .publish(pending.prepared, cancelled)
            .map_err(Error::Target)?;
        // Publication has succeeded. Only infallible moves and an Arc snapshot
        // clone remain; no lookup, allocation, callback or cancellation failure.
        Ok(DomNativeHtmlInstallation {
            destination: pending.destination,
            source_root: pending.source_root,
            root: pending.root,
            snapshot: self.snapshot(),
            changes: commit.changes.clone(),
            bindings: pending.bindings,
        })
    }
}

#[cfg(test)]
mod staging_observation {
    use super::*;
    use std::cell::Cell;
    #[derive(Clone, Copy, Debug)]
    pub(super) struct Growth {
        pub required: usize,
        pub limit: usize,
    }
    thread_local! {
        static ACTIVE: Cell<bool> = const { Cell::new(false) };
        static LAST: Cell<Option<Growth>> = const { Cell::new(None) };
    }
    pub(super) fn start() {
        ACTIVE.set(true);
        LAST.set(None);
    }
    pub(super) fn finish() -> Option<Growth> {
        ACTIVE.set(false);
        LAST.take()
    }
    pub(super) fn before(
        draft: &Draft,
        pending: &Vec<Visit>,
        registered: bool,
        cancel: &AtomicBool,
    ) -> Result<Option<Growth>, DomError> {
        if !ACTIVE.get() || pending.len() < pending.capacity() {
            return Ok(None);
        }
        let Some(limits) = draft.source_limits else {
            return Ok(None);
        };
        let old = if registered {
            0
        } else {
            pending.capacity() * size_of::<Visit>()
        };
        Ok(Some(Growth {
            required: draft
                .source_storage_bytes(cancel)?
                .checked_add(draft.source_other_bytes)
                .and_then(|n| n.checked_add(draft.source_formation_bytes.load(Ordering::Relaxed)))
                .and_then(|n| n.checked_add(old))
                .ok_or(DomError::RepresentationExhausted)?,
            limit: limits.retained_bytes,
        }))
    }
    pub(super) fn grew(before: Option<Growth>, pending: &Vec<Visit>) {
        if let Some(mut before) = before {
            before.required += pending.capacity() * size_of::<Visit>();
            if LAST
                .get()
                .is_none_or(|previous| previous.required < before.required)
            {
                LAST.set(Some(before));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Dom, crate::DocumentId, NodeId) {
        let mut dom = Dom::new(DomLimits::new(64, 4096, 128, 64, 64, 8, 65536).unwrap()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let parent = batch.create_element_ns(&document, crate::Namespace::html(), "div");
        batch.insert_before(&parent, &document, None);
        let commit = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let document = commit.document(&document).unwrap();
        let parent = commit.created(&parent).unwrap().unwrap();
        (dom, document, parent)
    }

    fn source(with_template: bool) -> HtmlFragment {
        let mut builder = HtmlFragment::builder(surgeist_html::HtmlConstructionLimits::new(
            16, 16, 16, 256, 16,
        ))
        .unwrap();
        if with_template {
            let template = builder
                .append_element(
                    &builder.root(),
                    crate::QualifiedName::new(crate::Namespace::html(), None, "template"),
                    crate::OrderedAttributes::default(),
                )
                .unwrap();
            let contents = builder.template_contents(&template).unwrap().unwrap();
            let script = builder
                .append_element(
                    &contents,
                    crate::QualifiedName::new(crate::Namespace::html(), None, "script"),
                    crate::OrderedAttributes::default(),
                )
                .unwrap();
            builder.append_text(&script, "x").unwrap();
        }
        builder.finish()
    }

    fn actual_source_pending_growth(bytes: usize) -> Option<staging_observation::Growth> {
        let cancel = AtomicBool::new(false);
        let mut dom =
            Dom::new(DomLimits::new(2048, 100_000, 4096, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut setup = DomBatch::new();
        let document = setup.create_document();
        let parent = setup.create_element_ns(&document, crate::Namespace::html(), "div");
        setup.insert_before(&parent, &document, None);
        let created = dom.apply(setup, &cancel).unwrap();
        let document = created.document(&document).unwrap();
        let parent = created.created(&parent).unwrap().unwrap();
        let mut builder = HtmlFragment::builder(surgeist_html::HtmlConstructionLimits::new(
            2048, 2048, 2048, 4096, 16,
        ))
        .unwrap();
        for _ in 0..1000 {
            builder.append_text(&builder.root(), "x").unwrap();
        }
        let source = builder.finish();
        let before = dom.snapshot();
        staging_observation::start();
        let result = dom.begin_native_installation(
            crate::DomNativeHtmlInstallTarget::new(
                before.document(document).unwrap().stamp(),
                parent,
                None,
            ),
            &source,
            crate::DomSourceNativeHtmlInstallLimits::new(
                DomNativeHtmlInstallLimits::new(4096, 10_000_000, 10_000_000),
                crate::DomSourceLimits::new(bytes, 1_000_000, 100_000_000, 64, 4096, 4096, 1),
            ),
            &cancel,
        );
        let actual = staging_observation::finish();
        drop(result);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(*dom.state, *before.state);
        assert!(dom.pending_source_edit().is_none());
        actual
    }

    #[test]
    fn source_native_staging_admits_actual_pending_replacement_before_allocation() {
        let high = actual_source_pending_growth(16_000_000).expect("real pending visit growth");
        let exact = actual_source_pending_growth(high.required)
            .expect("exact allowance reaches actual pending bank overlap");
        assert_eq!(exact.required, high.required);
        if let Some(actual) = actual_source_pending_growth(high.required - 1) {
            assert!(
                actual.required <= actual.limit,
                "native staging banks retained {} bytes under {}",
                actual.required,
                actual.limit
            );
        }
    }

    #[test]
    fn empty_installation_admits_the_exact_retained_buffer_peak() {
        // One pending visit and one row coexist. Later the row, exact final-ID
        // output and binding coexist; no canonical Draft allocation is included.
        let peak = (size_of::<Visit>() + size_of::<Row>())
            .max(size_of::<Row>() + size_of::<NodeId>() + size_of::<DomNativeHtmlBinding>());
        for (bytes, succeeds) in [(peak - 1, false), (peak, true)] {
            let (mut dom, document, parent) = setup();
            let before = dom.snapshot();
            let usage = dom.usage();
            let result = dom.install_native_html_fragment(
                before.document(document).unwrap().stamp(),
                parent,
                None,
                &source(false),
                DomNativeHtmlInstallLimits::new(1, 9, bytes),
                &AtomicBool::new(false),
            );
            if succeeds {
                assert_eq!(result.unwrap().bindings().len(), 1);
            } else {
                assert!(
                    matches!(result, Err(Error::ResourceLimit { resource: Resource::ScratchBytes, limit }) if limit == bytes)
                );
                assert_eq!(*dom.state, *before.state);
                assert_eq!(dom.usage(), usage);
                assert_eq!(dom.snapshot().stamp(), before.stamp());
            }
        }
    }

    #[test]
    fn prepared_identity_map_and_auxiliary_state_do_not_escape_failed_publication() {
        for exhausted in [false, true] {
            let (mut dom, document, parent) = setup();
            if exhausted {
                dom.revision = DomRevision(u64::MAX);
            }
            let before = dom.snapshot();
            let usage = dom.usage();
            let source = source(true);
            let cancel = AtomicBool::new(false);
            let pending = dom
                .prepare_native_html_installation(
                    before.document(document).unwrap().stamp(),
                    parent,
                    None,
                    &source,
                    DomNativeHtmlInstallLimits::new(16, 10000, 65536),
                    &cancel,
                )
                .unwrap();
            // Root, Template, real owned contents, Script and Text all have
            // final identities before publication; the inert Document is private.
            assert_eq!(pending.bindings.len(), 5);
            assert_eq!(pending.prepared.state.nodes, before.state.nodes + 6);
            for binding in &pending.bindings {
                assert!(before.node(binding.node()).is_err());
            }
            if !exhausted {
                cancel.store(true, Ordering::Release);
            }
            let error = dom
                .publish_native_html_installation(pending, &cancel)
                .unwrap_err();
            assert!(matches!(
                (exhausted, error),
                (true, Error::Target(DomError::RevisionExhausted))
                    | (false, Error::Target(DomError::Cancelled))
            ));
            assert_eq!(*dom.state, *before.state);
            assert_eq!(dom.usage(), usage);
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert!(
                dom.changes_since(before.stamp())
                    .unwrap()
                    .changes()
                    .is_empty()
            );
        }
    }
}
