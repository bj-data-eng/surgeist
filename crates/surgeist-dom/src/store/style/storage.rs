//! Checked retained payload storage for the actual synchronous source owner.
//! Shared immutable backings are conservatively charged per retained occurrence;
//! allocator metadata and Arc control-block implementation details are excluded.
use super::super::*;
use crate::element::NativeFamily;
use crate::{Attribute, QualifiedName};

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
    payload.add_bytes(size_of::<Vec<Attribute>>())?;
    payload.bank::<Attribute>(element.attributes().shared_attributes().capacity())?;
    for attribute in element.attributes().iter() {
        step()?;
        payload.name(attribute.name())?;
        payload.string(attribute.value())?;
    }
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
struct Related<'a, K> {
    base: Option<&'a crate::base::BaseState<K>>,
    base_document: Option<&'a crate::base::BaseDocument<K>>,
    hyperlink: Option<&'a crate::hyperlink::HyperlinkState<K>>,
    encoding_url_document: Option<&'a crate::hyperlink::EncodingUrlDocument>,
    image: Option<&'a crate::image::ImageState<K>>,
    form_past_names: Option<&'a crate::form::FormPastNames<K>>,
}
fn related<K>(
    related: Related<'_, K>,
    names_bank: bool,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    let mut payload = Payload::default();
    if let Some(base) = related.base
        && let Some(frozen) = &base.frozen
    {
        step()?;
        payload.producer(frozen.value.retained_storage_usage())?;
    }
    if let Some(document) = related.base_document
        && let Some(facts) = &document.facts
    {
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
    if let Some(names) = related.form_past_names {
        step()?;
        payload.add_bytes(size_of::<Vec<crate::form::FormPastName<K>>>())?;
        if names_bank {
            payload.bank::<crate::form::FormPastName<K>>(names.entries.capacity())?;
        }
        for entry in names.entries.iter() {
            step()?;
            payload.string(&entry.name)?;
        }
    }
    Ok(payload)
}

pub(in crate::store) fn node(
    node: &DraftNode,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    node_usage(node, true, &mut step)
}
/// Backings cloned into one candidate Node. The mapped past-name bank itself
/// is reserved separately by Formation; its Arc Vec header and names are not.
pub(in crate::store) fn candidate_node(
    node: &DraftNode,
    step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    node_usage(node, false, step)
}
fn node_usage(
    node: &DraftNode,
    names_bank: bool,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    let mut payload = kind(&node.kind, &mut step)?;
    payload.append(related(
        Related {
            base: node.base.as_ref(),
            base_document: node.base_document.as_ref(),
            hyperlink: node.hyperlink.as_ref(),
            encoding_url_document: node.encoding_url_document.as_ref(),
            image: node.image.as_ref(),
            form_past_names: node.form_past_names.as_ref(),
        },
        names_bank,
        step,
    )?)?;
    Ok(payload)
}
fn state_usage(
    state: &State,
    banks: bool,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    let mut payload = Payload::default();
    if banks {
        payload.add_bytes(size_of::<State>())?;
        payload.bank::<Slot>(state.slots.capacity())?;
        payload.bank::<NodeId>(state.signal_slots.capacity())?;
        payload.bank::<select::Task<NodeId>>(state.native_tasks.capacity())?;
        payload.bank::<crate::DomDetailsToggleIntention>(state.details_intentions.capacity())?;
    }
    for node in state.slots.iter().filter_map(|slot| slot.node.as_ref()) {
        step()?;
        payload.append(kind(&node.kind, &mut step)?)?;
        if banks {
            payload.bank::<NodeId>(node.children.capacity())?;
            payload.bank::<NodeId>(node.manual_nodes.capacity())?;
            payload.bank::<NodeId>(node.assigned_nodes.capacity())?;
            if let Some(crate::dialog::DocumentInteractiveState::Known {
                open_dialogs,
                top_layer,
            }) = &node.interactive_document
            {
                payload.bank::<NodeId>(open_dialogs.capacity())?;
                payload.bank::<NodeId>(top_layer.capacity())?;
            }
            if let Some(document) = &node.base_document {
                payload.bank::<NodeId>(document.render_blocking.capacity())?;
            }
        }
        payload.append(related(
            Related {
                base: node.base.as_ref(),
                base_document: node.base_document.as_ref(),
                hyperlink: node.hyperlink.as_ref(),
                encoding_url_document: node.encoding_url_document.as_ref(),
                image: node.image.as_ref(),
                form_past_names: node.form_past_names.as_ref(),
            },
            banks,
            &mut step,
        )?)?;
    }
    if banks {
        payload.bank::<crate::DomDocumentViewTransitions>(
            state.view_transitions.documents.capacity(),
        )?;
        payload.bank::<crate::view_transition::Transition>(
            state.view_transitions.transitions.capacity(),
        )?;
    }
    for transition in &state.view_transitions.transitions {
        step()?;
        if banks {
            payload.bank::<crate::view_transition::Record>(transition.records.capacity())?;
        }
        for record in &transition.records {
            step()?;
            payload.add_bytes(size_of::<surgeist_css::CssViewTransitionIdent>())?;
            payload.add_bytes(record.name.retained_buffer_bytes())?;
        }
    }
    Ok(payload)
}
pub(in crate::store) fn state(
    state: &State,
    step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    state_usage(state, true, step)
}
pub(in crate::store) fn state_payload(
    state: &State,
    step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    state_usage(state, false, step)
}
pub(in crate::store) fn commands(
    commands: &[Command],
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    let mut payload = Payload::default();
    for command in commands {
        step()?;
        match command {
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
            Command::CreateHtmlDocument {
                title: Some(value), ..
            }
            | Command::SetTextareaValue { value, .. }
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
            Command::Import { snapshot, .. } => {
                payload.append(state(&snapshot.state, &mut step)?)?
            }
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
    fn inline_hyperlink_snapshot_charges_only_its_out_of_line_backing() {
        let revision = HtmlBaseFactsRevision::new(HtmlBaseProducerId::fresh(), 1, 2, 3);
        let encoding = HtmlBaseEncoding::fresh();
        let base = HtmlBaseUrlRecord::observed(
            revision.clone(),
            "about:blank".into(),
            HtmlBaseUrlScheme::Other,
            true,
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
        let hyperlink = crate::hyperlink::HyperlinkState {
            parsed: Some(crate::hyperlink::Parsed {
                document: Key::Existing(0),
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
                base_document: None,
                hyperlink: Some(&hyperlink),
                encoding_url_document: None,
                image: None,
                form_past_names: None,
            },
            true,
            || Ok(()),
        )
        .unwrap();
        assert_eq!(payload.bytes, out_of_line);
        assert_eq!(payload.units, usage.utf16_extent());
        let cancel = AtomicBool::new(false);
        let limits =
            |bytes| crate::DomStyleEditLimits::new(bytes, usage.utf16_extent(), 1000, 1, 1, 1);
        let exact = super::super::initial::Initial::new(limits(out_of_line));
        exact.retain(payload, &cancel).unwrap();
        let short = super::super::initial::Initial::new(limits(out_of_line - 1));
        assert_eq!(
            short.retain(payload, &cancel),
            Err(DomError::ResourceLimit {
                resource: crate::DomResource::StyleRetainedBytes,
                limit: out_of_line - 1
            })
        );
    }
}
