//! Checked retained payload storage for the actual synchronous source owner.
//! Shared immutable backings are conservatively charged per retained occurrence;
//! allocator metadata and Arc control-block implementation details are excluded.
use super::super::*;
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
        self.empty_string()?;
        self.bank::<u16>(capacity)?;
        self.add_units(capacity)
    }
    /// One actual fresh `DocumentString::from("")` still owns its Vec target.
    /// This prices it before construction without first allocating a sample.
    pub fn empty_string(&mut self) -> Result<(), DomError> {
        self.add_bytes(size_of::<Vec<u16>>())
    }
    pub fn bank<T>(&mut self, capacity: usize) -> Result<(), DomError> {
        self.add_bytes(
            capacity
                .checked_mul(size_of::<T>())
                .ok_or(DomError::RepresentationExhausted)?,
        )
    }
    pub(crate) fn name(
        &mut self,
        name: &(impl surgeist_html::HtmlNameInput + ?Sized),
    ) -> Result<(), DomError> {
        if let Some(uri) = name.namespace_uri() {
            self.string(uri)?;
        }
        if let Some(prefix) = name.prefix() {
            self.string(prefix)?;
        }
        self.string(name.local_name())
    }
    /// Out-of-line extent retained by an inline producer URL snapshot.
    pub(crate) fn hyperlink_snapshot(
        &mut self,
        value: &surgeist_html_semantics::HtmlHyperlinkUrlSnapshot,
    ) -> Result<(), DomError> {
        let usage = value.retained_storage_usage();
        self.add_bytes(
            usage
                .retained_bytes()
                .checked_sub(size_of::<surgeist_html_semantics::HtmlHyperlinkUrlSnapshot>())
                .ok_or(DomError::RepresentationExhausted)?,
        )?;
        self.add_units(usage.utf16_extent())
    }
    pub fn producer(
        &mut self,
        usage: surgeist_html_semantics::HtmlBaseRetainedStorageUsage,
    ) -> Result<(), DomError> {
        self.add_bytes(usage.retained_bytes())?;
        self.add_units(usage.utf16_extent())
    }
}
/// Checked shallow receipt of one exact typed target. Callers supply only its
/// directly owned strings/banks; independently materialized children stay out.
pub(crate) fn typed<T>(out_of_line: Payload) -> Result<custody::Usage, DomError> {
    custody::Usage::new(size_of::<super::super::payload::Payload<T>>(), 0)
        .checked_add(custody::Usage::new(out_of_line.bytes, out_of_line.units))
}

pub(crate) fn document_metadata(value: &DomDocumentMetadata) -> Result<custody::Usage, DomError> {
    typed::<DomDocumentMetadata>(Payload {
        bytes: value.retained_payload_bytes()?,
        units: value.retained_payload_units()?,
    })
}
pub(crate) fn encoding_url(
    value: &crate::hyperlink::EncodingUrlDocument,
) -> Result<custody::Usage, DomError> {
    let mut payload = Payload::default();
    if let Some(facts) = &value.facts {
        payload.producer(facts.retained_storage_usage())?;
    }
    typed::<crate::hyperlink::EncodingUrlDocument>(payload)
}
pub(crate) fn base<K>(value: &crate::base::BaseState<K>) -> Result<custody::Usage, DomError> {
    let mut payload = Payload::default();
    if let Some(frozen) = &value.frozen {
        payload.producer(frozen.value.retained_storage_usage())?;
    }
    typed::<crate::base::BaseState<K>>(payload)
}
pub(crate) fn hyperlink<K>(
    value: &crate::hyperlink::HyperlinkState<K>,
) -> Result<custody::Usage, DomError> {
    let mut payload = Payload::default();
    if let Some(parsed) = &value.parsed {
        payload.hyperlink_snapshot(&parsed.value)?;
    }
    typed::<crate::hyperlink::HyperlinkState<K>>(payload)
}
pub(crate) fn image<K>(value: &crate::image::ImageState<K>) -> Result<custody::Usage, DomError> {
    let mut payload = Payload::default();
    payload.string(value.native.current_request().current_url())?;
    typed::<crate::image::ImageState<K>>(payload)
}
pub(crate) fn processing_instruction(
    value: &crate::DomProcessingInstruction,
    step: impl FnMut() -> Result<(), DomError>,
) -> Result<custody::Usage, DomError> {
    typed::<crate::DomProcessingInstruction>(value.retained_payload(step)?)
}
pub(crate) fn document_type(value: &crate::DomDocumentType) -> Result<custody::Usage, DomError> {
    let mut payload = Payload::default();
    for string in [value.name(), value.public_id(), value.system_id()] {
        payload.string(string)?;
    }
    typed::<crate::DomDocumentType>(payload)
}

pub(crate) fn fieldset_state(value: &crate::DomFieldsetState) -> Result<custody::Usage, DomError> {
    let mut payload = Payload::default();
    payload.string(&value.control.message)?;
    typed::<crate::DomFieldsetState>(payload)
}
pub(crate) fn button_state(value: &crate::DomButtonState) -> Result<custody::Usage, DomError> {
    let mut payload = Payload::default();
    payload.string(&value.control.message)?;
    typed::<crate::DomButtonState>(payload)
}
pub(crate) fn input_state(value: &crate::DomInputState) -> Result<custody::Usage, DomError> {
    let mut payload = Payload::default();
    payload.string(&value.current)?;
    payload.string(&value.control.message)?;
    if let surgeist_html::HtmlInputImageHistory::Last {
        source: Some(source),
        ..
    } = &value.image
    {
        payload.string(source)?;
    }
    typed::<crate::DomInputState>(payload)
}
pub(crate) fn textarea_state(value: &crate::DomTextareaState) -> Result<custody::Usage, DomError> {
    let mut payload = Payload::default();
    payload.string(&value.raw)?;
    payload.string(&value.control.message)?;
    typed::<crate::DomTextareaState>(payload)
}
pub(crate) fn output_state(value: &crate::DomOutputState) -> Result<custody::Usage, DomError> {
    let mut payload = Payload::default();
    payload.string(&value.control.message)?;
    if let Some(value) = &value.default_override {
        payload.string(value)?;
    }
    typed::<crate::DomOutputState>(payload)
}
pub(crate) fn select_state(value: &crate::DomSelectState) -> Result<custody::Usage, DomError> {
    let mut payload = Payload::default();
    payload.string(&value.control.message)?;
    typed::<crate::DomSelectState>(payload)
}
pub(crate) fn dialog_state(
    value: &crate::dialog::DomDialogState,
) -> Result<custody::Usage, DomError> {
    let mut payload = Payload::default();
    payload.string(value.return_value())?;
    if let Some(value) = &value.request_return_value {
        payload.string(value)?;
    }
    typed::<crate::dialog::DomDialogState>(payload)
}
pub(crate) fn style_state(_value: &crate::DomStyleState) -> Result<custody::Usage, DomError> {
    typed::<crate::DomStyleState>(Payload::default())
}

/// Aggregate diagnostics for a language value, never a new language-root debit.
/// Each actual stable child supplies its original shallow occurrence receipt.
pub(crate) fn element(
    element: &DomElement,
    family: &NativeFamily,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    step()?;
    let name = element.name_payload().own_usage;
    let mut payload = Payload {
        bytes: name.bytes(),
        units: name.units(),
    };
    if let Some(value) = element.attribute_binding() {
        payload.add_bytes(value.own_usage.bytes())?;
        payload.add_units(value.own_usage.units())?;
        payload.add_bytes(value.value.storage.own_usage().bytes())?;
    }
    for usage in [
        element.custom_payload().map(|v| v.own_usage),
        element.nonce_payload().map(|v| v.own_usage),
        element.focus_payload().map(|v| v.own_usage),
        element.hosted_payload().map(|v| v.own_usage),
    ]
    .into_iter()
    .flatten()
    {
        step()?;
        payload.add_bytes(usage.bytes())?;
        payload.add_units(usage.units())?;
    }
    payload.append(family_payload(family, step)?)?;
    Ok(payload)
}
pub(crate) fn family_payload(
    family: &NativeFamily,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    step()?;
    let usage = match family {
        NativeFamily::Fieldset(value) => value.own_usage,
        NativeFamily::Button(value) => value.own_usage,
        NativeFamily::Input(value) => value.own_usage,
        NativeFamily::Textarea(value) => value.own_usage,
        NativeFamily::Output(value) => value.own_usage,
        NativeFamily::Select(value) => value.own_usage,
        NativeFamily::Dialog(value) => value.own_usage,
        NativeFamily::Style(value) => value.own_usage,
        NativeFamily::Canvas(value) => value.own_usage,
        _ => custody::Usage::ZERO,
    };
    Ok(Payload {
        bytes: usage.bytes(),
        units: usage.units(),
    })
}
/// Raw caller-owned creation input, before any exact-kind target is formed.
/// This borrows the genuine ingress operand and shares the diagnostic inventory.
#[cfg(test)]
pub(in crate::store) fn new_kind(
    value: &NewNode,
    step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    let value = match value {
        NewNode::Document(value) => DomNodeKind::Document(value),
        NewNode::Element(value) => DomNodeKind::Element(value),
        NewNode::DocumentFragment => DomNodeKind::DocumentFragment,
        NewNode::ShadowRoot(value) => DomNodeKind::ShadowRoot(value),
        NewNode::Text(value) => DomNodeKind::Text(value),
        NewNode::Comment(value) => DomNodeKind::Comment(value),
        NewNode::ProcessingInstruction(value) => DomNodeKind::ProcessingInstruction(value),
        NewNode::DocumentType(value) => DomNodeKind::DocumentType(value),
    };
    kind(&value, step)
}
pub(in crate::store) fn kind_read(
    kind: transaction::KindRead<'_>,
    step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    let kind = match kind {
        transaction::KindRead::Document(value) => DomNodeKind::Document(value),
        transaction::KindRead::Element(value) => DomNodeKind::Element(value.language()),
        transaction::KindRead::DocumentFragment => DomNodeKind::DocumentFragment,
        transaction::KindRead::ShadowRoot(value) => DomNodeKind::ShadowRoot(value),
        transaction::KindRead::Text(value) => DomNodeKind::Text(value),
        transaction::KindRead::Comment(value) => DomNodeKind::Comment(value),
        transaction::KindRead::ProcessingInstruction(value) => {
            DomNodeKind::ProcessingInstruction(value)
        }
        transaction::KindRead::DocumentType(value) => DomNodeKind::DocumentType(value),
    };
    self::kind(&kind, step)
}
pub(in crate::store) fn kind(
    kind: &DomNodeKind<'_>,
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
        payload.hyperlink_snapshot(&parsed.value)?;
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

/// One Slot controls its own target and its exclusively nested typed root.
/// Independently materialized language and structural children stay separate.
pub(in crate::store) fn slot_usage(slot: &Slot) -> Result<custody::Usage, DomError> {
    custody::Usage::new(size_of::<Slot>(), 0).checked_add(
        slot.node
            .as_ref()
            .map_or(custody::Usage::ZERO, super::super::Node::own_usage),
    )
}

pub(in crate::store) fn canonical_slot(
    slot: &Slot,
    mut step: impl FnMut() -> Result<(), DomError>,
) -> Result<Payload, DomError> {
    step()?;
    let usage = slot_usage(slot)?;
    Ok(Payload {
        bytes: usage.bytes(),
        units: usage.units(),
    })
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
                payload.add_bytes(size_of::<
                    surgeist_css::values::view_transition::CssViewTransitionIdent,
                >())?;
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
        let mut dom = Dom::new(
            DomLimits::new(
                8,
                4096,
                64,
                0,
                0,
                16,
                1_000_000,
                crate::DomNameLimits::new(16_777_216, 16_384, 4_000_000, 1_048_576).unwrap(),
            )
            .unwrap(),
        )
        .unwrap();
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
