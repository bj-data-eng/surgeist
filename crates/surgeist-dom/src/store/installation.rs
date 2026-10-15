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

struct Row {
    source: HtmlNodeId,
    key: Key,
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
    fn map_row(&mut self, rows: &mut Vec<Row>, source: HtmlNodeId, key: Key) -> Result<(), Error> {
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
        self.reserve(rows, 1)?;
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
        let prepared = self
            .prepare_native_html_installation(basis, parent, before, source, limits, cancelled)?;
        self.publish_native_html_installation(prepared, cancelled)
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
        let mut control = Control {
            limits,
            work: 0,
            bytes: 0,
            cancel: cancelled,
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
        let mut draft =
            Draft::new(self, Arc::new(BatchIdentity), cancelled).map_err(Error::Target)?;
        let document = draft
            .resolve(basis.document().into())
            .map_err(Error::Target)?;
        let parent = draft.resolve(parent.into()).map_err(Error::Target)?;
        let before = before
            .map(|node| draft.resolve(node.into()))
            .transpose()
            .map_err(Error::Target)?;
        let wrapper = draft
            .allocate(DomNodeKind::DocumentFragment, document, self.limits)
            .map_err(Error::Target)?;
        let source_root = source.root().id();
        let mut pending = Vec::new();
        control.reserve(&mut pending, 1)?;
        pending.push(Visit {
            source: source_root.clone(),
            parent: None,
            existing: Some(wrapper),
            owner: None,
        });
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
                let kind = match original.kind() {
                    HtmlNodeKind::Element(element) => {
                        control.name(element.name())?;
                        DomNodeKind::Element(
                            DomElement::native_components(
                                element.name().clone(),
                                draft.native_environment(document).map_err(Error::Target)?,
                                cancelled,
                            )
                            .map_err(|error| source_error(&visit.source, error))?,
                        )
                    }
                    HtmlNodeKind::Text(value) => {
                        control.charge(value.len())?;
                        DomNodeKind::Text(value.clone())
                    }
                    HtmlNodeKind::Comment(value) => {
                        control.charge(value.len())?;
                        DomNodeKind::Comment(value.clone())
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
                                value.target().clone(),
                                value.data().clone(),
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
                        DomNodeKind::DocumentType(
                            crate::DomDocumentType::new(
                                value.name().clone(),
                                value.public_id().cloned().unwrap_or_default(),
                                value.system_id().cloned().unwrap_or_default(),
                            )
                            .map_err(|error| source_error(&visit.source, error))?,
                        )
                    }
                    HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment => {
                        return Err(source_error(&visit.source, DomError::HierarchyRequest));
                    }
                };
                let key = draft
                    .allocate(kind, document, self.limits)
                    .map_err(|error| source_error(&visit.source, error))?;
                draft
                    .initialize_native_element(key, self.limits, cancelled)
                    .map_err(|error| source_error(&visit.source, error))?;
                key
            };
            control.map_row(&mut rows, visit.source.clone(), key)?;
            if let HtmlNodeKind::Element(element) = original.kind() {
                for attribute in element.attributes().iter() {
                    control.charge(1)?;
                    control.name(attribute.name())?;
                    control.charge(attribute.value().len())?;
                    crate::names::validate_attribute_components(attribute.name(), cancelled)
                        .map_err(|error| source_error(&visit.source, error))?;
                    draft
                        .append_attribute(
                            key,
                            attribute.clone(),
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
                control.reserve(&mut pending, 1)?;
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
                    control.reserve(&mut pending, 1)?;
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
            commit,
            bindings: pending.bindings,
        })
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
