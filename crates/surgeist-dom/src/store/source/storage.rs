//! Checked retained payload storage for the actual synchronous source owner.
//! Shared immutable backings are conservatively charged per retained occurrence;
//! allocator metadata and Arc control-block implementation details are excluded.
use super::super::*;
use crate::QualifiedName;
use crate::element::NativeFamily;

#[derive(Clone, Copy, Default)]
pub(crate) struct Payload {
    pub bytes: usize,
    pub units: usize,
}
impl Payload {
    pub fn add_bytes(&mut self, bytes: usize) -> Result<(), DomError> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(())
    }
    pub fn add_units(&mut self, units: usize) -> Result<(), DomError> {
        self.units = self
            .units
            .checked_add(units)
            .ok_or(DomError::RepresentationExhausted)?;
        Ok(())
    }
    pub fn append(&mut self, other: Self) -> Result<(), DomError> {
        self.add_bytes(other.bytes)?;
        self.add_units(other.units)
    }
    pub fn string(&mut self, string: &DocumentString) -> Result<(), DomError> {
        let capacity = string.shared_code_units().capacity();
        self.add_bytes(size_of::<Vec<u16>>())?;
        self.bank::<u16>(capacity)?;
        self.add_units(capacity)
    }
    pub fn bank<T>(&mut self, capacity: usize) -> Result<(), DomError> {
        self.add_bytes(
            capacity
                .checked_mul(size_of::<T>())
                .ok_or(DomError::RepresentationExhausted)?,
        )
    }
    pub(crate) fn name(&mut self, name: &QualifiedName) -> Result<(), DomError> {
        if let Some(uri) = name.namespace().uri() {
            self.string(uri)?;
        }
        if let Some(prefix) = name.prefix() {
            self.string(prefix)?;
        }
        self.string(name.local_name())
    }
    pub fn producer(
        &mut self,
        usage: surgeist_html_semantics::HtmlBaseRetainedStorageUsage,
    ) -> Result<(), DomError> {
        self.add_bytes(usage.retained_bytes())?;
        self.add_units(usage.utf16_extent())
    }
}
pub(crate) fn element(
    element: &DomElement,
    family: &NativeFamily,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    let mut payload = Payload::default();
    step()?;
    payload.name(element.name())?;
    payload.append(element.attributes_payload()?)?;
    for string in [element.saved_is(), element.nonce()].into_iter().flatten() {
        step()?;
        payload.string(string)?;
    }
    payload.append(family_payload(family, step)?)?;
    Ok(payload)
}
pub(crate) fn family_payload(
    family: &NativeFamily,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    let mut payload = Payload::default();
    match family {
        NativeFamily::Fieldset(state) => payload.string(&state.control.message)?,
        NativeFamily::Button(state) => payload.string(&state.control.message)?,
        NativeFamily::Input(state) => {
            payload.add_bytes(size_of::<crate::DomInputState>())?;
            payload.string(&state.current)?;
            payload.string(&state.control.message)?;
            if let surgeist_html::HtmlInputImageHistory::Last {
                source: Some(source),
                ..
            } = &state.image
            {
                payload.string(source)?;
            }
        }
        NativeFamily::Textarea(state) => {
            payload.add_bytes(size_of::<crate::DomTextareaState>())?;
            payload.string(&state.raw)?;
            payload.string(&state.control.message)?;
        }
        NativeFamily::Output(state) => {
            payload.add_bytes(size_of::<crate::DomOutputState>())?;
            payload.string(&state.control.message)?;
            if let Some(value) = &state.default_override {
                payload.string(value)?;
            }
        }
        NativeFamily::Select(state) => {
            payload.add_bytes(size_of::<crate::DomSelectState>())?;
            payload.string(&state.control.message)?;
        }
        NativeFamily::Dialog(state) => {
            payload.add_bytes(size_of::<crate::dialog::DomDialogState>())?;
            payload.string(state.return_value())?;
            if let Some(value) = &state.request_return_value {
                payload.string(value)?;
            }
        }
        NativeFamily::Style(_) => payload.add_bytes(size_of::<crate::DomStyleState>())?,
        NativeFamily::Canvas(state) => {
            payload.add_bytes(size_of::<surgeist_html::HtmlCanvasNoneState>())?;
            for dimension in [state.width(), state.height()] {
                step()?;
                if let surgeist_html::HtmlCanvasBitmapDimension::Parsed(value) = dimension {
                    payload.add_bytes(value.retained_buffer_bytes())?;
                    payload.add_units(value.retained_buffer_bytes())?;
                }
            }
        }
        _ => {}
    }
    Ok(payload)
}
pub(in crate::store) fn kind(
    kind: &DomNodeKind,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    step()?;
    let mut payload = Payload::default();
    match kind {
        DomNodeKind::Document(metadata) => {
            payload.add_bytes(metadata.retained_payload_bytes()?)?;
            payload.add_units(metadata.retained_payload_units()?)?;
        }
        DomNodeKind::Element(element) => payload.append(element.retained_payload(&mut step)?)?,
        DomNodeKind::Text(string) | DomNodeKind::Comment(string) => payload.string(string)?,
        DomNodeKind::ProcessingInstruction(pi) => {
            payload.append(pi.retained_payload(&mut step)?)?
        }
        DomNodeKind::DocumentType(doctype) => {
            for string in [doctype.name(), doctype.public_id(), doctype.system_id()] {
                payload.string(string)?;
            }
        }
        DomNodeKind::DocumentFragment | DomNodeKind::ShadowRoot(_) => {}
    }
    Ok(payload)
}
#[cfg(test)]
struct Related<'a, K> {
    base: Option<&'a crate::base::BaseState<K>>,
    base_facts: Option<&'a surgeist_html_semantics::HtmlParseBaseFacts>,
    hyperlink: Option<&'a crate::hyperlink::HyperlinkState<K>>,
    encoding_url_document: Option<&'a crate::hyperlink::EncodingUrlDocument>,
    image: Option<&'a crate::image::ImageState<K>>,
}
#[cfg(test)]
fn related<K>(
    related: Related<'_, K>,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    let mut payload = Payload::default();
    if let Some(base) = related.base
        && let Some(frozen) = &base.frozen
    {
        step()?;
        payload.producer(frozen.value.retained_storage_usage())?;
    }
    if let Some(facts) = related.base_facts {
        step()?;
        payload.producer(facts.retained_storage_usage())?;
    }
    if let Some(hyperlink) = related.hyperlink
        && let Some(parsed) = &hyperlink.parsed
    {
        step()?;
        // The Node/DraftNode bank already contains this value's inline header.
        // The producer receipt includes Self for independently held snapshots.
        let usage = parsed.value.retained_storage_usage();
        payload.add_bytes(
            usage
                .retained_bytes()
                .checked_sub(size_of::<surgeist_html_semantics::HtmlHyperlinkUrlSnapshot>())
                .ok_or(DomError::RepresentationExhausted)?,
        )?;
        payload.add_units(usage.utf16_extent())?;
    }
    if let Some(document) = related.encoding_url_document
        && let Some(facts) = &document.facts
    {
        step()?;
        payload.producer(facts.retained_storage_usage())?;
    }
    if let Some(image) = related.image {
        step()?;
        payload.string(image.native.current_request().current_url())?;
    }
    Ok(payload)
}

pub(in crate::store) fn canonical_slot(
    slot: &Slot,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    step()?;
    let mut payload = Payload {
        bytes: size_of::<Slot>(),
        units: 0,
    };
    if let Some(node) = &slot.node {
        payload.add_bytes(node.receipts.payload.bytes())?;
        payload.add_units(node.receipts.payload.units())?;
    }
    Ok(payload)
}
#[cfg(test)]
pub(in crate::store) fn refresh_node_receipts(node: &mut Node) -> Result<(), DomError> {
    let payload = canonical_node(node, true, || Ok(()))?;
    node.receipts.payload = custody::Usage::new(payload.bytes, payload.units);
    node.receipts.fields = transaction::NodeFields::from_canonical(node).usage(|| Ok(()))?;
    Ok(())
}

#[cfg(test)]
pub(in crate::store) fn canonical_node(
    node: &Node,
    banks: bool,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    let mut payload = Payload::default();
    step()?;
    payload.append(kind(&node.kind, &mut step)?)?;
    if banks {
        payload.add_bytes(node.fullscreen_events.retained_bytes()?)?;
        payload.add_bytes(node.children.retained_bytes()?)?;
        payload.add_bytes(node.manual_nodes.retained_bytes()?)?;
        payload.add_bytes(node.assigned_nodes.retained_bytes()?)?;
        if let Some(canonical::Autofocus::Known { candidates, .. }) = &node.autofocus_document {
            payload.add_bytes(candidates.retained_bytes()?)?;
        }
        if let Some(canonical::Interactive::Known {
            open_dialogs,
            top_layer,
            pending_top_layer_removals,
        }) = &node.interactive_document
        {
            payload.add_bytes(open_dialogs.retained_bytes()?)?;
            payload.add_bytes(top_layer.retained_bytes()?)?;
            payload.add_bytes(pending_top_layer_removals.retained_bytes()?)?;
        }
        if let Some(document) = &node.base_document {
            payload.add_bytes(document.render_blocking.retained_bytes()?)?;
        }
    }
    payload.append(related(
        Related {
            base: node.base.as_ref(),
            base_facts: node
                .base_document
                .as_ref()
                .and_then(|document| document.facts.as_ref()),
            hyperlink: node.hyperlink.as_ref(),
            encoding_url_document: node.encoding_url_document.as_ref(),
            image: node.image.as_ref(),
        },
        &mut step,
    )?)?;
    Ok(payload)
}
pub(in crate::store) fn commands(
    commands: &[Command],
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    let mut payload = Payload::default();
    for command in commands {
        step()?;
        match command {
            Command::AssignSlot { incoming, .. } => {
                payload.bank::<NodeOperand>(incoming.capacity())?;
            }
            Command::BindBaseFacts { facts, .. } => {
                payload.producer(facts.retained_storage_usage())?
            }
            Command::BindEncodingUrlFacts { facts, .. } => {
                payload.producer(facts.retained_storage_usage())?
            }
            Command::CreateXmlDocument {
                namespace,
                qualified_name,
                ..
            } => {
                if let Some(uri) = namespace.uri() {
                    payload.string(uri)?;
                }
                payload.string(qualified_name)?;
            }
            Command::RequestCloseDialog {
                result: Some(value),
                ..
            }
            | Command::CloseDialog {
                result: Some(value),
                ..
            }
            | Command::CreateHtmlDocument {
                title: Some(value), ..
            }
            | Command::SetTextareaValue { value, .. }
            | Command::SetDesignMode { value, .. }
            | Command::SetTextareaDefaultValue { value, .. }
            | Command::SetInputValue { value, .. }
            | Command::SetCustomValidity { value, .. }
            | Command::SetDialogReturnValue { value, .. }
            | Command::SetNonce { value, .. } => payload.string(value)?,
            Command::Select {
                edit: crate::select::Edit::Value(value),
                ..
            }
            | Command::Output {
                edit: crate::output::Edit::Value(value) | crate::output::Edit::DefaultValue(value),
                ..
            } => payload.string(value)?,
            Command::Create { kind, .. } => match kind {
                Creation::Element(value) | Creation::Text(value) | Creation::Comment(value) => {
                    payload.string(value)?
                }
                Creation::ElementNs { namespace, name } => {
                    if let Some(uri) = namespace.uri() {
                        payload.string(uri)?;
                    }
                    payload.string(name)?;
                }
                Creation::ProcessingInstruction { target, data } => {
                    payload.string(target)?;
                    payload.string(data)?;
                }
                Creation::DocumentType {
                    name,
                    public_id,
                    system_id,
                } => {
                    payload.string(name)?;
                    payload.string(public_id)?;
                    payload.string(system_id)?;
                }
                Creation::Document | Creation::Fragment => {}
            },
            Command::SetAttribute {
                namespace,
                name,
                value,
                ..
            } => {
                if let Some(uri) = namespace.as_ref().and_then(crate::Namespace::uri) {
                    payload.string(uri)?;
                }
                payload.string(name)?;
                payload.string(value)?;
            }
            Command::RemoveAttribute {
                namespace, name, ..
            } => {
                if let Some(uri) = namespace.as_ref().and_then(crate::Namespace::uri) {
                    payload.string(uri)?;
                }
                payload.string(name)?;
            }
            Command::ReplaceData { data, .. } => payload.string(data)?,
            // A whole-snapshot Import retains its actual missing controlled
            // frontier through Source InputCapture, not a second payload sum.
            Command::Import { .. } => {}
            Command::ViewTransition {
                edit: crate::view_transition::Edit::Append { name, .. },
                ..
            } => {
                payload.add_bytes(size_of::<surgeist_css::CssViewTransitionIdent>())?;
                payload.add_bytes(name.retained_buffer_bytes())?;
            }
            _ => {}
        }
    }
    Ok(payload)
}

#[cfg(test)]
mod tests {
    use super::*;
    use surgeist_html_semantics::*;

    #[test]
    fn original_assign_slot_input_capacity_is_cached_once_with_the_commands() {
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let slot = batch.create_element_ns(&document, crate::Namespace::html(), "slot");
        let mut incoming = Vec::with_capacity(7);
        incoming.push(NodeOperand::from(&slot));
        batch.assign_slot(&slot, incoming);
        let before = commands(&batch.commands[..2], || Ok(())).unwrap();
        let mut visits = 0;
        let retained = commands(&batch.commands, || {
            visits += 1;
            Ok(())
        })
        .unwrap();
        assert_eq!(visits, 3);
        assert_eq!(retained.bytes - before.bytes, 7 * size_of::<NodeOperand>());
        assert_eq!(retained.units, before.units);
    }

    #[test]
    fn inline_hyperlink_snapshot_charges_only_its_out_of_line_backing() {
        let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
        let encoding = HtmlBaseEncoding::fresh();
        let base = HtmlBaseUrlRecord::observed(
            revision.clone(),
            "about:blank".into(),
            HtmlBaseUrlScheme::Other,
            true,
            None,
        )
        .unwrap();
        let mut raw = Vec::with_capacity(128);
        raw.push(0xd800);
        let raw = DocumentString::from_utf16(raw);
        let facts = HtmlEncodingUrlParseFacts::new(
            base.clone(),
            encoding.clone(),
            revision.clone(),
            vec![HtmlEncodingUrlParseObservation::new(
                raw.clone(),
                HtmlEncodingUrlParseOutcome::Failure,
            )],
            HtmlBaseFactsLimits::new(16, 1024, 10_000),
        )
        .unwrap();
        let snapshot = facts
            .resolve_hyperlink_with(
                &raw,
                &base,
                &encoding,
                &revision,
                || Ok::<_, ()>(()),
                |_| Ok(()),
            )
            .unwrap();
        let usage = snapshot.retained_storage_usage();
        let mut dom = Dom::new(DomLimits::new(8, 4096, 64, 0, 0, 16, 1_000_000).unwrap()).unwrap();
        let mut batch = DomBatch::new();
        let document = batch.create_document();
        let done = dom.apply(batch, &AtomicBool::new(false)).unwrap();
        let document = done.document(&document).unwrap();
        let hyperlink = crate::hyperlink::HyperlinkState {
            parsed: Some(crate::hyperlink::Parsed {
                document: Key::Existing(document.node()),
                value: snapshot,
            }),
        };
        // A Node/DraftNode embeds this public value, whereas the provider's
        // standalone snapshot receipt intentionally includes its inline header.
        let out_of_line = usage
            .retained_bytes()
            .checked_sub(size_of::<HtmlHyperlinkUrlSnapshot>())
            .unwrap();
        let payload = related(
            Related {
                base: None,
                base_facts: None,
                hyperlink: Some(&hyperlink),
                encoding_url_document: None,
                image: None,
            },
            || Ok(()),
        )
        .unwrap();
        assert_eq!(payload.bytes, out_of_line);
        assert_eq!(payload.units, usage.utf16_extent());
        let cancel = AtomicBool::new(false);
        let limits =
            |bytes| crate::DomSourceLimits::new(bytes, usage.utf16_extent(), 1000, 1, 1, 1, 1);
        let exact = super::super::initial::Initial::new(limits(out_of_line));
        exact.retain(payload, &cancel).unwrap();
        let short = super::super::initial::Initial::new(limits(out_of_line - 1));
        assert_eq!(
            short.retain(payload, &cancel),
            Err(DomError::ResourceLimit {
                resource: crate::DomResource::SourceRetainedBytes,
                limit: out_of_line - 1
            })
        );
    }
}
