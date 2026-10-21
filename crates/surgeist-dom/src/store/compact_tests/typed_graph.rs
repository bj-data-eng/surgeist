//! Independent enumeration of actual typed semantic targets and owned backing.
//! Expected extents never read Payload.own_usage, a Debit or a cached summary.
use super::*;
use crate::element::NativeFamily;
use crate::store::payload::Payload as Target;

#[derive(Default)]
struct Raw {
    bytes: usize,
    units: usize,
}
impl Raw {
    fn bytes(&mut self, bytes: usize) {
        self.bytes = self.bytes.checked_add(bytes).unwrap();
    }
    fn units(&mut self, units: usize) {
        self.units = self.units.checked_add(units).unwrap();
    }
    fn text(&mut self, value: &DocumentString) {
        let capacity = value.shared_code_units().capacity();
        self.bytes(size_of::<Vec<u16>>() + capacity * size_of::<u16>());
        self.units(capacity);
    }
    fn sequence<T>(&mut self, value: &SharedVec<T>) {
        if value.has_backing() {
            self.bytes(size_of::<Vec<T>>() + value.capacity() * size_of::<T>());
        }
    }
    fn producer(&mut self, value: surgeist_html_semantics::HtmlBaseRetainedStorageUsage) {
        self.bytes(value.retained_bytes());
        self.units(value.utf16_extent());
    }
}
struct Walker<'a> {
    graph: Option<&'a mut AllocationGraph>,
    objects: BTreeMap<custody::MaterializationId, custody::Usage>,
}
impl Walker<'_> {
    fn object<T>(&mut self, value: &Arc<Target<T>>, extra: Raw, label: &'static str) {
        let expected = custody::Usage::new(size_of::<Target<T>>() + extra.bytes, extra.units);
        assert_eq!(
            value.own_usage, expected,
            "actual shallow receipt for {label}"
        );
        if let Some(old) = self.objects.insert(value.identity, expected) {
            assert_eq!(old, expected);
        }
        if let Some(graph) = &mut self.graph {
            graph.target(value, label);
        }
    }
    fn fixed<T>(&mut self, value: &Arc<Target<T>>, label: &'static str) {
        self.object(value, Raw::default(), label);
    }
    fn root<T>(&mut self, value: &Arc<Target<T>>, label: &'static str) {
        assert_eq!(
            value.own_usage,
            custody::Usage::new(size_of::<Target<T>>(), 0),
            "actual shallow root extent for {label}"
        );
        if let Some(graph) = &mut self.graph {
            graph.target(value, label);
        }
        // The root is an actual allocation/ID, but its body shares the Slot's
        // affine receipt. It is not an independently registered child object.
    }
    fn text(&mut self, out: &mut Raw, value: &DocumentString) {
        out.text(value);
        if let Some(graph) = &mut self.graph {
            graph.string(value);
        }
    }
    fn sequence<T>(&mut self, out: &mut Raw, value: &SharedVec<T>) {
        out.sequence(value);
        if let Some(graph) = &mut self.graph {
            value.inspect_raw_allocations(graph);
        }
    }
    fn atom(&mut self, value: &crate::DomNameAtom) {
        let mut raw = Raw::default();
        self.text(&mut raw, value.text());
        let expected =
            custody::Usage::new(size_of::<crate::name::NameRecord>() + raw.bytes, raw.units);
        if let Some(old) = self.objects.insert(value.identity(), expected) {
            assert_eq!(old, expected);
        }
        if let Some(graph) = &mut self.graph {
            graph.target(value.record(), "name.record");
        }
    }
    fn qualified(&mut self, out: &mut Raw, value: &crate::DomQualifiedName) {
        if let Some(namespace) = value.namespace() {
            self.text(out, namespace.text());
            self.atom(namespace);
        }
        if let Some(prefix) = value.prefix() {
            self.text(out, prefix);
        }
        self.text(out, value.local_name());
        self.atom(value.local_name_atom());
    }
    fn container<K>(&mut self, value: &Option<Arc<Target<canonical::ContainerChildren<K>>>>) {
        if let Some(value) = value {
            let mut raw = Raw::default();
            self.sequence(&mut raw, &value.value.values);
            self.object(value, raw, "node.children");
        }
    }
    fn metadata(&mut self, value: &Arc<Target<DomDocumentMetadata>>) {
        let metadata = &value.value;
        let mut raw = Raw::default();
        for text in [
            Some(metadata.content_type()),
            Some(metadata.encoding()),
            Some(metadata.url()),
            metadata.meta_state().and_then(|v| v.pragma_language()),
        ]
        .into_iter()
        .flatten()
        {
            self.text(&mut raw, text);
        }
        if let Some(Some(selected)) = &metadata.meta_color_scheme {
            let report = selected.report.retained_storage();
            raw.bytes(report.bytes());
            raw.units(report.utf16_units());
        }
        if let crate::DomDocumentCspState::Associated(association) =
            metadata.policy_container().csp()
        {
            // Ask the actual immutable producer graph, not SelectedCsp's cached
            // layout or this metadata target's receipt. Its enclosing wrapper
            // adds only its fixed layout descriptor beside the association.
            let layout = association.retained_layout().unwrap();
            raw.bytes(layout.storage().bytes());
            raw.bytes(size_of::<surgeist_html::HtmlCspArtifactLayout>());
            raw.units(layout.storage().utf16_units());
        }
        if let crate::DomDocumentCreationCause::ParsedOffline { url, .. } =
            metadata.creation_cause()
        {
            raw.producer(url.retained_storage_usage());
        }
        if metadata.document_environment().hosted.is_some() {
            raw.bytes(size_of::<crate::hosted::DocumentAssociation>());
        }
        if let Some(inputs) = &metadata.hosted_inputs {
            raw.bytes(inputs.payload_bytes().unwrap());
            raw.units(inputs.payload_units().unwrap());
        }
        self.object(value, raw, "document.data");
        // Native small-fixture physical scope; producer-owned parsed/hosted
        // opaque allocations require their own explicit physical descriptors.
        if let Some(graph) = &mut self.graph {
            metadata.inspect_compact_baseline(graph);
        }
    }
    fn element(&mut self, value: &Arc<Target<DomElement>>) {
        let element = &value.value;
        self.fixed(value, "element.data");
        let name = element.name_payload();
        let mut raw = Raw::default();
        self.qualified(&mut raw, &name.value.value);
        self.object(name, raw, "element.name");
        if let Some(binding) = element.attribute_binding() {
            let mut text = Raw::default();
            let storage = &binding.value.storage;
            for attribute in storage.entries() {
                self.qualified(&mut text, attribute.name());
                self.text(&mut text, attribute.value());
            }
            self.object(binding, text, "element.attributes");
            if let Some(bank) = storage.payload() {
                let mut raw = Raw::default();
                raw.bytes(bank.value.capacity() * size_of::<crate::DomAttribute>());
                self.object(bank, raw, "attributes.target");
                if let Some(graph) = &mut self.graph {
                    storage.inspect_allocations(graph);
                }
            }
        }
        if let Some(custom) = element.custom_payload() {
            let mut raw = Raw::default();
            if let Some(text) = &custom.value.saved_is {
                self.text(&mut raw, text);
            }
            self.object(custom, raw, "element.custom");
        }
        if let Some(nonce) = element.nonce_payload() {
            let mut raw = Raw::default();
            self.text(&mut raw, &nonce.value.value);
            self.object(nonce, raw, "element.nonce");
        }
        if let Some(focus) = element.focus_payload() {
            self.fixed(focus, "element.focus");
        }
        if let Some(hosted) = element.hosted_payload() {
            self.fixed(hosted, "element.hosted");
        }
        self.family(element.native_family());
    }
    fn family(&mut self, family: &NativeFamily) {
        match family {
            NativeFamily::Input(value) => {
                let mut raw = Raw::default();
                self.text(&mut raw, &value.value.current);
                self.text(&mut raw, &value.value.control.message);
                if let surgeist_html::HtmlInputImageHistory::Last {
                    source: Some(source),
                    ..
                } = &value.value.image
                {
                    self.text(&mut raw, source);
                }
                self.object(value, raw, "native.input");
            }
            NativeFamily::Textarea(value) => {
                let mut raw = Raw::default();
                self.text(&mut raw, &value.value.raw);
                self.text(&mut raw, &value.value.control.message);
                self.object(value, raw, "native.textarea");
            }
            NativeFamily::Output(value) => {
                let mut raw = Raw::default();
                self.text(&mut raw, &value.value.control.message);
                if let Some(text) = &value.value.default_override {
                    self.text(&mut raw, text);
                }
                self.object(value, raw, "native.output");
            }
            NativeFamily::Select(value) => {
                let mut raw = Raw::default();
                self.text(&mut raw, &value.value.control.message);
                self.object(value, raw, "native.select");
            }
            NativeFamily::Fieldset(value) => {
                let mut raw = Raw::default();
                self.text(&mut raw, &value.value.control.message);
                self.object(value, raw, "native.fieldset");
            }
            NativeFamily::Button(value) => {
                let mut raw = Raw::default();
                self.text(&mut raw, &value.value.control.message);
                self.object(value, raw, "native.button");
            }
            NativeFamily::Dialog(value) => {
                let mut raw = Raw::default();
                self.text(&mut raw, value.value.return_value());
                if let Some(text) = &value.value.request_return_value {
                    self.text(&mut raw, text);
                }
                self.object(value, raw, "native.dialog");
            }
            NativeFamily::Style(value) => self.fixed(value, "native.style"),
            NativeFamily::Canvas(value) => {
                let mut raw = Raw::default();
                for dimension in [value.value.width(), value.value.height()] {
                    if let surgeist_html::HtmlCanvasBitmapDimension::Parsed(digits) = dimension {
                        let bytes = digits.retained_buffer_bytes();
                        raw.bytes(bytes);
                        raw.units(bytes);
                        if bytes != 0
                            && let Some(graph) = &mut self.graph
                        {
                            graph.allocation(
                                digits.decimal_digits().as_ptr() as usize,
                                "native.canvas.digits",
                                bytes,
                            );
                        }
                    }
                }
                self.object(value, raw, "native.canvas");
            }
            _ => {}
        }
    }
    fn history<K>(&mut self, value: &crate::form::FormPastNames<K>) {
        if let Some(target) = value.payload() {
            let mut raw = Raw::default();
            raw.bytes(target.value.entries.capacity() * size_of::<crate::form::FormPastName<K>>());
            for entry in &target.value.entries {
                self.text(&mut raw, &entry.name);
            }
            self.object(target, raw, "form.names.target");
            if let Some(graph) = &mut self.graph {
                value.inspect_allocations(graph);
            }
        }
    }
    fn association(&mut self, value: &canonical::ElementAssociations<NodeId>) {
        use canonical::ElementAssociations as A;
        match value {
            A::None => {}
            A::FormEmpty(names) => {
                assert!(
                    names.payload().is_none(),
                    "inline FormEmpty has no retained target"
                );
                assert!(names.entries().is_empty());
                assert_eq!(names.capacity(), 0);
            }
            A::Template(v) => self.fixed(v, "node.template"),
            A::Control(v) => self.fixed(v, "node.control"),
            A::Option(v) => self.fixed(v, "node.option"),
            A::Script(v) => self.fixed(v, "node.script"),
            A::Style(v) => self.fixed(v, "node.style"),
            A::Link(v) => self.fixed(v, "node.link"),
            A::Slot(v) => {
                let mut raw = Raw::default();
                self.sequence(&mut raw, &v.value.manual_nodes);
                self.sequence(&mut raw, &v.value.assigned_nodes);
                self.object(v, raw, "node.slot");
            }
            A::Form(v) => {
                self.fixed(v, "node.form");
                self.history(&v.value.past_names);
            }
            A::Base(v) => {
                let mut raw = Raw::default();
                if let Some(frozen) = &v.value.frozen {
                    raw.producer(frozen.value.retained_storage_usage());
                }
                self.object(v, raw, "node.base");
            }
            A::Hyperlink(v) => {
                let mut raw = Raw::default();
                if let Some(parsed) = &v.value.parsed {
                    let usage = parsed.value.retained_storage_usage();
                    raw.bytes(
                        usage.retained_bytes()
                            - size_of::<surgeist_html_semantics::HtmlHyperlinkUrlSnapshot>(),
                    );
                    raw.units(usage.utf16_extent());
                }
                self.object(v, raw, "node.hyperlink");
            }
            A::Image(v) => {
                self.fixed(v, "node.image.association");
                let mut raw = Raw::default();
                self.text(
                    &mut raw,
                    v.value.image.value.native.current_request().current_url(),
                );
                self.object(&v.value.image, raw, "node.image");
            }
        }
    }
    fn node(&mut self, node: &Node) {
        match node {
            Node::Document(v) => {
                self.root(v, "node.document");
                let v = &v.value;
                self.metadata(&v.metadata);
                self.container(&v.children);
                if let Some(events) = &v.fullscreen_events {
                    let mut raw = Raw::default();
                    self.sequence(&mut raw, &events.value.values);
                    self.object(events, raw, "document.fullscreen");
                }
                if let canonical::BaseDocumentState::Known(v) = &v.base_document {
                    let mut raw = Raw::default();
                    self.sequence(&mut raw, &v.value.render_blocking);
                    if let Some(facts) = &v.value.facts {
                        raw.producer(facts.retained_storage_usage());
                    }
                    self.object(v, raw, "document.base");
                }
                if let canonical::EncodingUrl::Known(v) = &v.encoding_url_document {
                    let mut raw = Raw::default();
                    if let Some(facts) = &v.value.facts {
                        raw.producer(facts.retained_storage_usage());
                    }
                    self.object(v, raw, "document.encoding");
                }
                if let canonical::Autofocus::Known(v) = &v.autofocus_document {
                    let mut raw = Raw::default();
                    self.sequence(&mut raw, &v.value.candidates);
                    self.object(v, raw, "document.autofocus");
                }
                if let canonical::Interactive::Known(v) = &v.interactive_document {
                    let mut raw = Raw::default();
                    self.sequence(&mut raw, &v.value.open_dialogs);
                    self.sequence(&mut raw, &v.value.top_layer);
                    self.sequence(&mut raw, &v.value.pending_top_layer_removals);
                    self.object(v, raw, "document.interactive");
                }
            }
            Node::Element(v) => {
                self.root(v, "node.element");
                let v = &v.value;
                self.element(&v.element);
                self.container(&v.children);
                if let Some(v) = &v.slottable {
                    self.fixed(v, "node.slottable");
                }
                if let Some(v) = &v.shadow {
                    self.fixed(v, "node.shadow.host");
                }
                self.association(&v.associations);
            }
            Node::DocumentFragment(v) => {
                self.root(v, "node.fragment");
                self.container(&v.value.children);
            }
            Node::ShadowRoot(v) => {
                self.root(v, "node.shadow");
                self.fixed(&v.value.state, "shadow.data");
                self.container(&v.value.children);
            }
            Node::Text(v) => {
                self.root(v, "node.text");
                let mut raw = Raw::default();
                self.text(&mut raw, &v.value.data.value.value);
                self.object(&v.value.data, raw, "character.data");
                if let Some(v) = &v.value.slottable {
                    self.fixed(v, "node.slottable");
                }
            }
            Node::Comment(v) => {
                self.root(v, "node.comment");
                let mut raw = Raw::default();
                self.text(&mut raw, &v.value.data.value.value);
                self.object(&v.value.data, raw, "character.data");
            }
            Node::DocumentType(v) => {
                self.root(v, "node.doctype");
                let mut raw = Raw::default();
                for text in [
                    v.value.data.value.name(),
                    v.value.data.value.public_id(),
                    v.value.data.value.system_id(),
                ] {
                    self.text(&mut raw, text);
                }
                self.object(&v.value.data, raw, "doctype.data");
            }
            Node::ProcessingInstruction(v) => {
                self.root(v, "node.pi");
                let value = &v.value.data.value;
                let mut raw = Raw::default();
                self.text(&mut raw, value.target());
                self.text(&mut raw, value.data());
                for (name, value) in value.attributes() {
                    self.text(&mut raw, name);
                    self.text(&mut raw, value);
                }
                let mut actual = AllocationGraph::default();
                value.inspect_compact_baseline(&mut actual);
                if let Some((_, bytes)) = actual.summary().get("pi.attributes.target") {
                    raw.bytes(*bytes);
                }
                if let Some((_, bytes)) = actual.summary().get("pi.attributes.buffer") {
                    raw.bytes(*bytes);
                }
                if let Some(graph) = &mut self.graph {
                    graph.merge(&actual);
                }
                self.object(&v.value.data, raw, "pi.data");
            }
        }
    }
}

pub(in crate::store) fn inspect_node(node: &Node, graph: &mut AllocationGraph) {
    Walker {
        graph: Some(graph),
        objects: BTreeMap::new(),
    }
    .node(node);
}
pub(in crate::store) fn assert_node_receipts(node: &Node) {
    Walker {
        graph: None,
        objects: BTreeMap::new(),
    }
    .node(node);
}
/// Actual Slot and exact-kind root layouts, without cached semantic receipts.
pub(in crate::store) fn slot_payload(slot: &Slot) -> custody::Usage {
    let root = slot.node.as_ref().map_or(0, |node| match node {
        Node::Document(_) => size_of::<Target<canonical::DocumentNode<NodeId>>>(),
        Node::Element(_) => size_of::<Target<canonical::ElementNode<NodeId>>>(),
        Node::DocumentFragment(_) => size_of::<Target<canonical::FragmentNode<NodeId>>>(),
        Node::ShadowRoot(_) => size_of::<Target<canonical::ShadowNode<NodeId>>>(),
        Node::Text(_) => size_of::<Target<canonical::TextNode<NodeId>>>(),
        Node::Comment(_) => size_of::<Target<canonical::CommentNode<NodeId>>>(),
        Node::ProcessingInstruction(_) => {
            size_of::<Target<canonical::ProcessingInstructionNode<NodeId>>>()
        }
        Node::DocumentType(_) => size_of::<Target<canonical::DocumentTypeNode<NodeId>>>(),
    });
    custody::Usage::new(size_of::<Slot>() + root, 0)
}
pub(in crate::store) fn materializations(
    state: &State,
) -> BTreeMap<custody::MaterializationId, custody::Usage> {
    let mut walker = Walker {
        graph: None,
        objects: BTreeMap::new(),
    };
    for (_, slot) in state.slots.iter() {
        if let Some(node) = &slot.node {
            walker.node(node);
        }
    }
    let graph = state
        .slots
        .inspect_graph(|slot| {
            let raw = slot_payload(slot);
            Ok(bank::Totals {
                bytes: raw.bytes(),
                units: raw.units(),
                occurrences: 1,
            })
        })
        .unwrap();
    for (id, raw) in graph.materializations() {
        assert!(
            walker.objects.insert(id, raw).is_none(),
            "Slot/page identity is distinct from its controlled children"
        );
    }
    walker.objects
}
