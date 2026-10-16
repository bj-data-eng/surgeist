//! DOM3071 native Document creators in the one canonical transaction owner.
use super::*;

/// One actual createDocument command tail beneath its insertion effects. Keys
/// retain private/canonical identities across publication; namespace is the
/// original argument occurrence, not the optional root's derived namespace.
#[derive(Clone)]
pub(super) struct XmlDocumentFrame {
    document: Key,
    associated: Key,
    root: Option<Key>,
    doctype: Option<Key>,
    namespace: crate::Namespace,
    stage: u8,
}
impl XmlDocumentFrame {
    pub(super) fn retained_payload(
        &self,
        mut step: impl FnMut() -> Result<(), DomError>,
    ) -> Result<style::storage::Payload, DomError> {
        step()?;
        let mut payload = style::storage::Payload::default();
        if let Some(uri) = self.namespace.uri() {
            payload.string(uri)?;
        }
        Ok(payload)
    }
}

impl Draft {
    pub(super) fn document_environment(
        &self,
        document: Key,
    ) -> Result<crate::DomDocumentEnvironment, DomError> {
        let DomNodeKind::Document(metadata) = &self.node(document)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        Ok(metadata.document_environment())
    }
    pub(super) fn create_xml_document(
        &mut self,
        associated: Key,
        namespace: &crate::Namespace,
        qualified_name: &DocumentString,
        doctype: Option<&NodeOperand>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        let input = self.document_environment(associated)?.input_platform();
        admit(
            self.nodes
                .checked_add(1)
                .ok_or(DomError::RepresentationExhausted)?,
            limits.nodes,
            DomResource::ManagedNodes,
        )?;
        admit(
            self.units
                .checked_add(NATIVE_DOCUMENT_STRING_UNITS)
                .ok_or(DomError::RepresentationExhausted)?,
            limits.units,
            DomResource::StringCodeUnits,
        )?;
        let formation = style::formation::Formation::new(self, cancel)?;
        let metadata =
            DomDocumentMetadata::native(crate::DomDocumentEnvironment::new(input), &mut |text| {
                style::formation::string(text, formation.as_ref(), cancel)
            })?;
        let document = Key::Created(self.created.len());
        let document = self.allocate(DomNodeKind::Document(metadata), document, limits, cancel)?;
        drop(formation);
        self.source_step(cancel)?;
        let root = if qualified_name.is_empty() {
            None
        } else {
            let formation = style::formation::Formation::new(self, cancel)?;
            let element = DomElement::native_ns(
                namespace,
                qualified_name,
                self.document_environment(document)?,
                cancel,
                formation.as_ref(),
            )?;
            let root = self.allocate(DomNodeKind::Element(element), document, limits, cancel)?;
            drop(formation);
            self.initialize_native_element(root, limits, cancel)?;
            Some(root)
        };
        // Root name validation precedes any doctype operand/effect. The real
        // pre-insert path retains identity and performs adoption and hooks.
        let doctype = doctype
            .map(|operand| {
                self.source_step(cancel)?;
                let key = self.resolve(operand.clone())?;
                if !matches!(self.node(key)?.kind, DomNodeKind::DocumentType(_)) {
                    return Err(DomError::WrongNodeKind);
                }
                Ok(key)
            })
            .transpose()?;
        #[cfg(test)]
        let namespace_before = xml_observation::before(self, cancel)?;
        let (namespace, payload) = if let Some(uri) = namespace.uri() {
            let (uri, payload) = style::formation::shared_string(self, uri, cancel)?;
            (crate::Namespace::from_uri(uri), payload)
        } else {
            (crate::Namespace::none(), None)
        };
        #[cfg(test)]
        if let Some(uri) = namespace.uri() {
            xml_observation::retained(namespace_before, uri, cancel);
        }
        self.push_effect_retained(
            select::Effect::XmlDocument(XmlDocumentFrame {
                document,
                associated,
                root,
                doctype,
                namespace,
                stage: 0,
            }),
            self.effect_scoped,
            payload,
            cancel,
        )?;
        self.run_effects(limits, cancel)
    }
    pub(super) fn xml_document_effect(
        &mut self,
        mut frame: XmlDocumentFrame,
        payload: Option<style::checkpoint::Charge>,
        limits: DomLimits,
        cancel: &AtomicBool,
    ) -> Result<(), DomError> {
        self.source_step(cancel)?;
        match frame.stage {
            0 | 1 => {
                let child = if frame.stage == 0 {
                    frame.doctype
                } else {
                    frame.root
                };
                let document = frame.document;
                frame.stage += 1;
                self.push_effect_retained(
                    select::Effect::XmlDocument(frame),
                    self.effect_scoped,
                    payload,
                    cancel,
                )?;
                if let Some(child) = child {
                    self.pre_insert(child, document, None, limits, cancel)?;
                }
                Ok(())
            }
            2 => {
                let DomNodeKind::Document(source) = &self.node(frame.associated)?.kind else {
                    unreachable!("qualified associated Document")
                };
                let origin = source.origin().clone();
                let DomNodeKind::Document(metadata) = &mut self.node_mut(frame.document).kind
                else {
                    unreachable!("created XML Document")
                };
                metadata.finish_xml_origin(origin);
                self.source_step(cancel)?;
                let content_type = match frame.namespace.uri() {
                    Some(uri)
                        if self.source_ascii(uri, "http://www.w3.org/1999/xhtml", cancel)? =>
                    {
                        "application/xhtml+xml"
                    }
                    Some(uri)
                        if self.source_ascii(uri, "http://www.w3.org/2000/svg", cancel)? =>
                    {
                        "image/svg+xml"
                    }
                    _ => "application/xml",
                };
                let DomNodeKind::Document(metadata) = &self.node(frame.document)?.kind else {
                    unreachable!("created XML Document")
                };
                let units = self
                    .units
                    .checked_sub(metadata.content_type().len())
                    .and_then(|units| units.checked_add(content_type.len()))
                    .ok_or(DomError::RepresentationExhausted)?;
                admit(units, limits.units, DomResource::StringCodeUnits)?;
                #[cfg(test)]
                let mime_before = xml_observation::before(self, cancel)?;
                let formation = style::formation::Formation::new(self, cancel)?;
                let content_type =
                    style::formation::string(content_type, formation.as_ref(), cancel)?;
                let DomNodeKind::Document(metadata) = &mut self.node_mut(frame.document).kind
                else {
                    unreachable!("created XML Document")
                };
                #[cfg(test)]
                xml_observation::retained(mime_before, &content_type, cancel);
                metadata.finish_xml_content_type(content_type);
                self.units = units;
                drop(formation);
                // The namespace occurrence remains live until its original
                // frame drops. Ticket exposure is the final source action.
                self.register_ticket(frame.document, cancel)?;
                drop(frame);
                drop(payload);
                self.source_admit(0, cancel)
            }
            _ => unreachable!("XML creator tail stage"),
        }
    }
    pub(super) fn create_html_document(
        &mut self,
        associated_document: Key,
        title: Option<&DocumentString>,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<Key, DomError> {
        check_cancel(cancelled)?;
        let DomNodeKind::Document(source) = &self.node(associated_document)?.kind else {
            return Err(DomError::WrongNodeKind);
        };
        // Origin follows the source operation; the selected immutable native
        // environment also remains established. Other metadata uses defaults;
        // no browsing context or host/resource capability is implicitly copied.
        let formation = style::formation::Formation::new(self, cancelled)?;
        let metadata = DomDocumentMetadata::native_html(
            source.origin().clone(),
            source.document_environment(),
            &mut |text| style::formation::string(text, formation.as_ref(), cancelled),
        )?;
        let document = Key::Created(self.created.len());
        let document =
            self.allocate(DomNodeKind::Document(metadata), document, limits, cancelled)?;
        drop(formation);
        let operands = style::formation::Formation::new(self, cancelled)?;
        let name = style::formation::string("html", operands.as_ref(), cancelled)?;
        let public = style::formation::string("", operands.as_ref(), cancelled)?;
        let system = style::formation::string("", operands.as_ref(), cancelled)?;
        let formation = style::formation::Formation::new(self, cancelled)?;
        let doctype = DomDocumentType::new(&name, &public, &system, formation.as_ref(), cancelled)?;
        drop((name, public, system));
        drop(operands);
        let doctype = self.allocate(
            DomNodeKind::DocumentType(doctype),
            document,
            limits,
            cancelled,
        )?;
        drop(formation);
        self.pre_insert(doctype, document, None, limits, cancelled)?;
        let html = self.html_skeleton_element(document, "html", limits, cancelled)?;
        self.pre_insert(html, document, None, limits, cancelled)?;
        let head = self.html_skeleton_element(document, "head", limits, cancelled)?;
        self.pre_insert(head, html, None, limits, cancelled)?;
        if let Some(title) = title {
            let element = self.html_skeleton_element(document, "title", limits, cancelled)?;
            self.pre_insert(element, head, None, limits, cancelled)?;
            let (title, payload) = style::formation::shared_string(self, title, cancelled)?;
            let text = self.allocate(DomNodeKind::Text(title), document, limits, cancelled)?;
            drop(payload);
            self.pre_insert(text, element, None, limits, cancelled)?;
        }
        let body = self.html_skeleton_element(document, "body", limits, cancelled)?;
        self.pre_insert(body, html, None, limits, cancelled)?;
        Ok(document)
    }
    fn html_skeleton_element(
        &mut self,
        document: Key,
        local_name: &str,
        limits: DomLimits,
        cancelled: &AtomicBool,
    ) -> Result<Key, DomError> {
        check_cancel(cancelled)?;
        let DomNodeKind::Document(metadata) = &self.node(document)?.kind else {
            unreachable!("canonical skeleton Document")
        };
        let operand = style::formation::Formation::new(self, cancelled)?;
        let local_name = style::formation::string(local_name, operand.as_ref(), cancelled)?;
        let formation = style::formation::Formation::new(self, cancelled)?;
        let element = DomElement::native(&local_name, metadata, cancelled, formation.as_ref())?;
        drop(local_name);
        drop(operand);
        let key = self.allocate(DomNodeKind::Element(element), document, limits, cancelled)?;
        drop(formation);
        Ok(key)
    }
    pub(super) fn html_element_in_html_document(&self, key: Key) -> Result<bool, DomError> {
        let node = self.node(key)?;
        let DomNodeKind::Element(element) = &node.kind else {
            return Err(DomError::WrongNodeKind);
        };
        let DomNodeKind::Document(document) = &self.node(node.document)?.kind else {
            unreachable!("canonical node document")
        };
        Ok(crate::names::in_html_namespace(element.name())
            && document.class() == crate::DomDocumentClass::Html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn owner() -> (Dom, DocumentId) {
        let mut dom = Dom::new(DomLimits::new(32, 4096, 64, 0, 0, 8, 8192).unwrap()).unwrap();
        let mut batch = DomBatch::new();
        let source = batch.create_document();
        let source = dom
            .apply(batch, &AtomicBool::new(false))
            .unwrap()
            .document(&source)
            .unwrap();
        (dom, source)
    }
    #[test]
    fn cancellation_after_complete_html_preparation_publishes_no_skeleton_origin_or_history() {
        let (mut dom, source) = owner();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        let created = batch.create_html_document(source, Some("title".into()));
        let prepared = dom.prepare(batch, &AtomicBool::new(false)).unwrap();
        assert_eq!(prepared.state.nodes, 8);
        let document = prepared.created[created.index].unwrap();
        let node = prepared.state.node(dom.owner, document).unwrap();
        let DomNodeKind::Document(metadata) = &node.kind else {
            panic!("prepared Document")
        };
        assert_eq!(metadata.class(), crate::DomDocumentClass::Html);
        assert_eq!(
            metadata.origin(),
            before.document(source).unwrap().metadata().origin()
        );
        assert_eq!(
            dom.publish(prepared, &AtomicBool::new(true)).unwrap_err(),
            DomError::Cancelled
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
    }
    #[test]
    fn exhausted_revision_rejects_complete_html_publication_without_consuming_ids() {
        let (mut dom, source) = owner();
        dom.revision = DomRevision(u64::MAX);
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut batch = DomBatch::new();
        batch.create_html_document(source, Some("".into()));
        assert_eq!(
            dom.apply(batch, &AtomicBool::new(false)).unwrap_err(),
            DomError::RevisionExhausted
        );
        assert_eq!(*dom.state, *before.state);
        assert_eq!(dom.snapshot().stamp(), before.stamp());
        assert_eq!(dom.usage(), usage);
        assert!(
            dom.changes_since(before.stamp())
                .unwrap()
                .changes()
                .is_empty()
        );
    }
}

/// Scalar observation at actual retained namespace/MIME occurrences. Disabled
/// outside these owning tests; no alternate allocation or quota controller.
#[cfg(test)]
mod xml_observation {
    use super::*;
    use std::cell::RefCell;
    #[derive(Default)]
    pub(super) struct State {
        pub bytes: usize,
        pub units: usize,
        pub exceeded: bool,
        pub reached: usize,
        pub cancel_at_mime: bool,
    }
    thread_local! { static STATE: RefCell<Option<State>> = const { RefCell::new(None) }; }
    pub(super) struct Binding;
    impl Binding {
        pub fn new(cancel_at_mime: bool) -> Self {
            STATE.with(|state| {
                assert!(state.borrow().is_none());
                *state.borrow_mut() = Some(State {
                    cancel_at_mime,
                    ..State::default()
                });
            });
            Self
        }
        pub fn finish(self) -> State {
            STATE.with(|state| state.borrow_mut().take().unwrap())
        }
    }
    impl Drop for Binding {
        fn drop(&mut self) {
            STATE.with(|state| {
                state.borrow_mut().take();
            });
        }
    }
    pub(super) fn before(
        draft: &Draft,
        cancel: &AtomicBool,
    ) -> Result<Option<(usize, usize, usize, usize)>, DomError> {
        if !STATE.with(|state| state.borrow().is_some()) {
            return Ok(None);
        }
        let limits = draft.source_limits.expect("actual Source fixture");
        Ok(Some((
            draft.source_storage_bytes(cancel)?
                + draft.source_other_bytes
                + draft.source_formation_bytes.load(Ordering::Relaxed),
            draft.source_storage_units(cancel)?
                + draft.source_other_units
                + draft.source_temporary_units.load(Ordering::Relaxed),
            limits.retained_bytes,
            limits.retained_units,
        )))
    }
    pub(super) fn retained(
        before: Option<(usize, usize, usize, usize)>,
        actual: &DocumentString,
        cancel: &AtomicBool,
    ) {
        let Some((bytes, units, byte_limit, unit_limit)) = before else {
            return;
        };
        let capacity = actual.shared_code_units().capacity();
        STATE.with(|state| {
            let mut state = state.borrow_mut();
            let state = state.as_mut().expect("bound observer");
            let required_bytes = bytes + size_of::<Vec<u16>>() + capacity * size_of::<u16>();
            let required_units = units + capacity;
            state.bytes = state.bytes.max(required_bytes);
            state.units = state.units.max(required_units);
            state.exceeded |= required_bytes > byte_limit || required_units > unit_limit;
            state.reached += 1;
            if state.cancel_at_mime && state.reached == 2 {
                cancel.store(true, Ordering::Relaxed);
            }
        });
    }
}

#[cfg(test)]
mod xml_tests {
    use super::*;
    fn run(
        bytes: usize,
        units: usize,
        cancelled_tail: bool,
    ) -> (Result<(), DomError>, xml_observation::State) {
        let cancel = AtomicBool::new(false);
        let mut dom = Dom::new(DomLimits::new(32, 100_000, 64, 0, 0, 8, 100_000).unwrap()).unwrap();
        let mut batch = DomBatch::new();
        let associated = batch.create_document();
        let associated = dom
            .apply(batch, &cancel)
            .unwrap()
            .document(&associated)
            .unwrap();
        let before = dom.snapshot();
        let usage = dom.usage();
        let mut uri = Vec::with_capacity(65536);
        uri.extend("http://www.w3.org/1999/xhtml".encode_utf16());
        let mut batch = DomBatch::new();
        let document = batch.create_xml_document(
            associated,
            crate::Namespace::from_uri(DocumentString::from_utf16(uri)),
            "",
            None,
        );
        let observation = xml_observation::Binding::new(cancelled_tail);
        let result = (|| {
            let mut lease = dom.begin_style_batch(
                batch,
                crate::DomStyleEditLimits::new(bytes, units, 100_000_000, 64, 4096, 4096),
                &cancel,
            )?;
            let crate::DomStyleProgress::Complete(done) = lease.advance(&cancel)? else {
                panic!("empty root needs no service")
            };
            let created = done.commit().document(&document).unwrap();
            assert_eq!(
                lease
                    .snapshot()
                    .document(created)
                    .unwrap()
                    .metadata()
                    .content_type(),
                &DocumentString::from("application/xhtml+xml")
            );
            assert_eq!(
                lease
                    .snapshot()
                    .document(created)
                    .unwrap()
                    .metadata()
                    .origin(),
                before.document(associated).unwrap().metadata().origin()
            );
            Ok(())
        })();
        let observed = observation.finish();
        if result.is_err() {
            assert_eq!(dom.snapshot().stamp(), before.stamp());
            assert_eq!(dom.usage(), usage);
            assert!(dom.pending_style_edit().is_none());
        }
        (result, observed)
    }
    #[test]
    fn source_xml_namespace_and_late_mime_admit_actual_backings_at_exact_and_one_short_bounds() {
        let (result, peak) = run(16_000_000, 1_000_000, false);
        result.unwrap();
        assert_eq!(peak.reached, 2);
        assert!(!peak.exceeded);
        for resource in [
            DomResource::StyleRetainedBytes,
            DomResource::StyleRetainedUnits,
        ] {
            let bytes = if resource == DomResource::StyleRetainedBytes {
                peak.bytes
            } else {
                16_000_000
            };
            let units = if resource == DomResource::StyleRetainedUnits {
                peak.units
            } else {
                1_000_000
            };
            let (result, exact) = run(bytes, units, false);
            result.unwrap();
            assert_eq!(exact.reached, 2);
            assert!(!exact.exceeded);
            let (result, short) = run(
                bytes - usize::from(resource == DomResource::StyleRetainedBytes),
                units - usize::from(resource == DomResource::StyleRetainedUnits),
                false,
            );
            assert!(
                matches!(result,Err(DomError::ResourceLimit { resource: actual, .. }) if actual == resource)
            );
            assert!(
                !short.exceeded,
                "actual retained graph must never exceed its parent allowance"
            );
        }
    }
    #[test]
    fn cancelled_xml_mime_tail_never_publishes_partial_origin_or_ticket() {
        let (result, observed) = run(16_000_000, 1_000_000, true);
        assert_eq!(result.unwrap_err(), DomError::Cancelled);
        assert_eq!(observed.reached, 2);
        assert!(!observed.exceeded);
    }
}
