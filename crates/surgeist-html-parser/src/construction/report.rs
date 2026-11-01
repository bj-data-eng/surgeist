//! Qualified immutable publication storage over the canonical frozen owner.
//! Metadata inventory examines one real slot; no lookup hides a registry scan.

use super::*;
use history::{FrozenHistory, PreparedHistory, Unpublishable};

#[cfg(test)]
impl FrozenArena {
    /// Names, as the policy of one Document's Meta history, a row the arena
    /// does not hold. Whether a Document was found to name it.
    pub(crate) fn corrupt_meta_policy_for_test(&mut self) -> bool {
        let Some(storage) = self.storage.as_mut() else {
            return false;
        };
        for row in &mut storage.payloads.values {
            if let RetainedPayload::Document { meta, .. } = row {
                meta.csp = Some(usize::MAX);
                return true;
            }
        }
        false
    }
}

/// A kind of metadata row that publication must find bound to a node.
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum UnboundRow {
    Textarea,
    Output,
    Input,
    InputSource,
    FormAssociation,
    SelectFamily,
    Script,
    Template,
    LookupRegistry,
    NodeDocument,
    SvgScript,
}
#[cfg(test)]
impl UnboundRow {
    pub(crate) const ALL: [Self; 11] = [
        Self::Textarea,
        Self::Output,
        Self::Input,
        Self::InputSource,
        Self::FormAssociation,
        Self::SelectFamily,
        Self::Script,
        Self::Template,
        Self::LookupRegistry,
        Self::NodeDocument,
        Self::SvgScript,
    ];
    /// A source that creates a row of this kind.
    pub(crate) fn literal(self) -> &'static str {
        match self {
            Self::Textarea => "<!doctype html><textarea>t</textarea>",
            Self::Output => "<!doctype html><output>a</output>",
            Self::Input | Self::InputSource | Self::FormAssociation => {
                "<!doctype html><form><input value=x></form>"
            }
            Self::SelectFamily => "<!doctype html><select><option>a</select>",
            Self::Script => "<!doctype html><script>a</script>",
            Self::Template => "<!doctype html><template>a</template>",
            Self::LookupRegistry | Self::NodeDocument => "<!doctype html><p>a",
            Self::SvgScript => "<!doctype html><svg><script/></svg>",
        }
    }
    /// Leaves the first row of this kind bound to no node. Whether one was found.
    pub(super) fn unbind(self, row: &mut RetainedPayload) -> bool {
        let node = match (self, row) {
            (Self::Textarea, RetainedPayload::Textarea { node, .. })
            | (Self::Output, RetainedPayload::Output { node, .. })
            | (Self::Input, RetainedPayload::Input { node, .. })
            | (Self::InputSource, RetainedPayload::InputSource { node, .. })
            | (Self::FormAssociation, RetainedPayload::FormAssociation { node, .. })
            | (Self::SelectFamily, RetainedPayload::SelectFamily { node, .. })
            | (Self::Script, RetainedPayload::Script { node, .. })
            | (Self::Template, RetainedPayload::Template { node, .. })
            | (Self::LookupRegistry, RetainedPayload::LookupRegistry { node, .. })
            | (Self::NodeDocument, RetainedPayload::NodeDocument { node, .. })
            | (Self::SvgScript, RetainedPayload::SvgScript { node, .. }) => node,
            _ => return false,
        };
        *node = None;
        true
    }
}
#[cfg(test)]
impl FrozenArena {
    /// Leaves the first row of `kind` bound to no node. Whether one was found.
    pub(crate) fn unbind_row_for_test(&mut self, kind: UnboundRow) -> bool {
        let Some(storage) = self.storage.as_mut() else {
            return false;
        };
        storage
            .payloads
            .values
            .iter_mut()
            .any(|row| kind.unbind(row))
    }
}

pub(crate) struct ReportStorage {
    history: FrozenHistory,
    closed_common: bool,
    associated_slot: Option<usize>,
}
pub(crate) struct PreparedReportStorage<'a> {
    history: PreparedHistory<'a>,
    closed_common: bool,
    associated_slot: Option<usize>,
}
pub(crate) struct SharedReportStorage<'a, 'c> {
    storage: PreparedReportStorage<'a>,
    sharing: super::value_sharing::Sharing<'c>,
}
impl SharedReportStorage<'_, '_> {
    pub(crate) fn control(&mut self) -> &mut ParseControl {
        self.sharing.control()
    }
    pub(crate) fn commit(self) -> Result<ReportStorage, PublishFault> {
        self.sharing.commit()?;
        Ok(self.storage.commit()?)
    }
}
/// Why a prepared report could not be published.
pub(crate) enum PublishFault {
    /// The stop is already the controller's first.
    Stop(ParseStop),
    /// The prepared history held no custody of what the report needs.
    Unpublishable,
}
impl From<ParseStop> for PublishFault {
    fn from(stop: ParseStop) -> Self {
        Self::Stop(stop)
    }
}
impl From<Unpublishable> for PublishFault {
    fn from(_: Unpublishable) -> Self {
        Self::Unpublishable
    }
}
impl PublishFault {
    /// The refusal as the session error, with a custody fault latched as the
    /// controller's first stop.
    pub(crate) fn latch(self, c: &mut ParseControl) -> SessionError {
        match self {
            Self::Stop(stop) => SessionError::Stop(stop),
            Self::Unpublishable => latch_registry_row(c),
        }
    }
}
impl<'a> PreparedReportStorage<'a> {
    pub(crate) fn share_values<'c>(
        mut self,
        c: &'c mut ParseControl,
        #[cfg(test)] mode: crate::request::ValueSharingMode,
    ) -> Result<SharedReportStorage<'a, 'c>, SessionError> {
        let sharing = self.history.share_values(
            c,
            #[cfg(test)]
            mode,
        )?;
        Ok(SharedReportStorage {
            storage: self,
            sharing,
        })
    }
    pub(crate) fn prepare(
        input: &'a mut Option<FrozenArena>,
        c: &mut ParseControl,
        base_facts: Option<&crate::HtmlParseBaseFacts>,
        closed_common: bool,
    ) -> Result<Self, SessionError> {
        let frozen = input.as_ref().ok_or(SessionError::Consumed)?;
        frozen.require_original_transfer(c)?;
        let storage = frozen.storage.as_ref().ok_or(SessionError::Consumed)?;
        c.require_identity(&storage.control)?;
        let arena = arena(storage);
        let mut associated_slot = None;
        let mut associated_usage = associated::AssociatedStorageUsage::default();
        for (payload_index, payload) in storage.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(
                WorkCategory::InspectByte,
                mem::size_of::<RetainedPayload>() as u64,
            )?;
            let node = metadata_node(payload);
            if let Some(node) = node {
                checked_node(arena, node, c)?;
            }
            match payload {
                RetainedPayload::AssociatedInputs { inputs, usage } => {
                    if associated_slot.replace(payload_index).is_some() {
                        return Err(SessionError::AssociatedFacts);
                    }
                    // Prepay final row/request Arc drops and every actual bank
                    // handle destruction before the publication checkpoint.
                    // Unused Vec capacity needs no per-element destruction.
                    payload_write::<RetainedPayload>(c, 1)?;
                    c.charge(WorkCategory::ProviderCall, 2)?;
                    for _ in &inputs.encoding_urls {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        c.charge(WorkCategory::ProviderCall, 1)?;
                    }
                    if inputs.template.is_some() {
                        c.charge(WorkCategory::ProviderCall, 1)?;
                    }
                    if inputs.parser_document.is_some() {
                        c.charge(WorkCategory::ProviderCall, 1)?;
                    }
                    for _ in &inputs.captured {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        c.charge(WorkCategory::ProviderCall, 1)?;
                    }
                    for (resource, amount) in usage.resources() {
                        if c.usage().resource(resource).current < amount {
                            return Err(SessionError::AssociatedFacts);
                        }
                    }
                }
                RetainedPayload::Hyperlink { node, state } => {
                    checked_document(storage, &state.document, c)?;
                    if !matches!(
                        arena.node_at(*node).map(|n| n.kind()),
                        Some(HtmlNodeKind::Element(_))
                    ) {
                        return Err(SessionError::WrongToken);
                    }
                    if let Some(raw) = state.raw {
                        checked_raw(storage, raw, c)?;
                    }
                    if let Some(cause) = &state.cause {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if storage
                            .evidence
                            .as_ref()
                            .and_then(|e| e.retained_cause(cause))
                            .is_none()
                        {
                            return Err(SessionError::WrongToken);
                        }
                    }
                    associated_usage.checked_sum(state.diagnostic_usage(c)?, c)?;
                }
                RetainedPayload::Dialog { node, state } => {
                    checked_raw(storage, state.value, c)?;
                    let HtmlNodeKind::Element(element) =
                        arena.node_at(*node).ok_or(SessionError::WrongToken)?.kind()
                    else {
                        return Err(SessionError::WrongToken);
                    };
                    if !exact_ascii(
                        c,
                        element.name().namespace().uri(),
                        "http://www.w3.org/1999/xhtml",
                    )? || !exact_ascii(c, Some(element.name().local_name()), "dialog")?
                    {
                        return Err(SessionError::WrongToken);
                    }
                    if let Some(missing) = &state.first {
                        checked_dialog_missing(storage, missing, c)?;
                        if missing.subject != *node {
                            return Err(SessionError::WrongToken);
                        }
                    }
                }
                RetainedPayload::Style { node, state } => {
                    let HtmlNodeKind::Element(element) =
                        arena.node_at(*node).ok_or(SessionError::WrongToken)?.kind()
                    else {
                        return Err(SessionError::WrongToken);
                    };
                    if !exact_ascii(
                        c,
                        element.name().namespace().uri(),
                        "http://www.w3.org/1999/xhtml",
                    )? || !exact_ascii(c, Some(element.name().local_name()), "style")?
                    {
                        return Err(SessionError::WrongToken);
                    }
                    if let Some(document) = &state.parser_document {
                        checked_document(storage, document, c)?;
                    }
                    if state.state.created_by_document_parser() != state.parser_document.is_some()
                        || state.state.on_parser_stack()
                    {
                        return Err(SessionError::AssociatedFacts);
                    }
                    if let Some(cause) = &state.cause {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if storage
                            .evidence
                            .as_ref()
                            .and_then(|e| e.retained_cause(cause))
                            .is_none()
                        {
                            return Err(SessionError::WrongToken);
                        }
                    }
                }
                RetainedPayload::Image { node, state } => {
                    checked_node(arena, state.dimension_source, c)?;
                    if *node != state.dimension_source || state.latest == 0 {
                        return Err(SessionError::AssociatedFacts);
                    }
                    if let Some(cause) = &state.cause {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if storage
                            .evidence
                            .as_ref()
                            .and_then(|e| e.retained_cause(cause))
                            .is_none()
                        {
                            return Err(SessionError::WrongToken);
                        }
                    }
                    associated_usage.checked_sum(state.usage, c)?;
                }
                RetainedPayload::BaseDocument { node, facts, first } => {
                    checked_document(storage, &DocumentRef::Owned(*node), c)?;
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    let mut inert = false;
                    let mut auxiliary = false;
                    for p in &storage.payloads.values {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if let RetainedPayload::AssociatedInputs { inputs, .. } = p {
                            c.charge(WorkCategory::RecordVisit, 1)?;
                            if *node == 0
                                && storage.fragment.is_some()
                                && inputs
                                    .parser_document
                                    .as_ref()
                                    .is_some_and(|original| facts.same_owner(original))
                            {
                                auxiliary = true;
                            }
                        }
                        if let RetainedPayload::AssociatedInputs { inputs, .. } = p
                            && inputs
                                .template
                                .as_ref()
                                .is_some_and(|original| facts.same_owner(original))
                        {
                            for candidate in &storage.payloads.values {
                                c.charge(WorkCategory::RecordVisit, 1)?;
                                if matches!(candidate, RetainedPayload::Document { node: Some(bound), state: template::DocumentState::InertTemplate, .. } if bound == node)
                                {
                                    inert = true;
                                    break;
                                }
                            }
                            break;
                        }
                    }
                    if (*node != 0 || base_facts.is_none_or(|original| !facts.same_owner(original)))
                        && !inert
                        && !auxiliary
                    {
                        return Err(SessionError::WrongToken);
                    }
                    if let Some(first) = first {
                        checked_node(arena, *first, c)?;
                    }
                }
                RetainedPayload::Base { node, frozen, .. } => {
                    let node = node.ok_or(SessionError::WrongToken)?;
                    // The node was checked against the result with the row's other nodes.
                    if !base::is_base(arena.node_at(node).ok_or(SessionError::ForeignNode)?) {
                        return Err(SessionError::WrongToken);
                    }
                    if let base::BaseValue::Unavailable { missing, .. } = frozen {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if !matches!(
                            storage.payloads.values.get(*missing),
                            Some(RetainedPayload::MetaMissing {
                                kind: meta::MetaMissingKind::Csp
                                    | meta::MetaMissingKind::CspProcessing(_),
                                ..
                            })
                        ) {
                            return Err(SessionError::WrongToken);
                        }
                    }
                    if let Some(frozen) = frozen.previous() {
                        checked_document(storage, &DocumentRef::Owned(frozen.document), c)?;
                        if frozen.response == base::Response::Pending {
                            return Err(SessionError::WrongToken);
                        }
                    }
                }
                RetainedPayload::Link {
                    node,
                    parser_document,
                } => {
                    let node = node.ok_or(SessionError::WrongToken)?;
                    let record = checked_node(arena, node, c)?;
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let HtmlNodeKind::Element(element) = record.kind() else {
                        return Err(SessionError::WrongToken);
                    };
                    if element.link_flags().is_none() {
                        return Err(SessionError::WrongToken);
                    }
                    // Every created node has its node-document row, which the
                    // link history joins to the parser Document.
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let slot = storage.payloads.joined(c, node, Join::NodeDocument)?;
                    if !matches!(
                        slot.and_then(|slot| storage.payloads.values.get(slot)),
                        Some(RetainedPayload::NodeDocument { node: Some(owner), .. }) if *owner == node
                    ) {
                        return Err(SessionError::NodeDocumentUnavailable);
                    }
                    if let Some(document) = parser_document {
                        checked_document(storage, document, c)?;
                    }
                }
                RetainedPayload::Details { node, .. } => {
                    // Construction eligibility is private, but its bound node
                    // must still belong to this immutable result.
                    checked_node(arena, node.ok_or(SessionError::WrongToken)?, c)?;
                }
                RetainedPayload::Input {
                    raw,
                    complete,
                    state,
                    ..
                } => {
                    if !complete {
                        return Err(SessionError::InputMetadataUnavailable);
                    }
                    checked_raw(storage, *raw, c)?;
                    c.charge(WorkCategory::RecordVisit, 2)?;
                    if state.flags.input_type() == crate::HtmlInputType::File
                        && !state.selected_files_empty
                    {
                        return Err(SessionError::InputMetadataUnavailable);
                    }
                    if let input_image::ImageState::Last {
                        source: Some(source),
                        ..
                    } = state.image
                    {
                        checked_raw(storage, source, c)?;
                    }
                    if state.flags.input_type() == crate::HtmlInputType::Image
                        && matches!(state.image, input_image::ImageState::NeverEntered)
                    {
                        return Err(SessionError::InputMetadataUnavailable);
                    }
                }
                RetainedPayload::SelectFamily {
                    state: SelectMetadata::Option(state),
                    ..
                } => {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    c.charge(
                        WorkCategory::InspectByte,
                        mem::size_of::<Option<usize>>() as u64,
                    )?;
                    if let Some(select) = state.cached_nearest_select() {
                        checked_select(arena, select, c)?;
                    }
                }
                RetainedPayload::Nonce { node, raw } => {
                    let node = node.ok_or(SessionError::NonceMetadataUnavailable)?;
                    let record = checked_node(arena, node, c)?;
                    c.charge(
                        WorkCategory::InspectByte,
                        surgeist_html::bounded::HTML_NONCE_INSPECTION_BOUND as u64,
                    )?;
                    let HtmlNodeKind::Element(element) = record.kind() else {
                        return Err(SessionError::NonceMetadataUnavailable);
                    };
                    if !surgeist_html::bounded::is_nonce_element(element.name()) {
                        return Err(SessionError::NonceMetadataUnavailable);
                    }
                    checked_raw(storage, *raw, c)?;
                }
                RetainedPayload::Textarea { state, .. } => {
                    checked_raw(storage, state.raw, c)?;
                }
                RetainedPayload::FormAssociation { state, .. } => {
                    if let Some(owner) = state.owner {
                        checked_node(arena, owner, c)?;
                    }
                }
                RetainedPayload::LookupRegistry {
                    node,
                    registry,
                    creation: Some(state),
                } => {
                    if closed_common
                        && storage.fragment.is_none()
                        && !matches!(registry, context::RegistryRef::Null)
                    {
                        return Err(SessionError::ContextUnavailable);
                    }
                    if closed_common && matches!(registry, context::RegistryRef::Null) {
                        let HtmlNodeKind::Element(element) = arena
                            .node_at(node.ok_or(SessionError::WrongToken)?)
                            .ok_or(SessionError::WrongToken)?
                            .kind()
                        else {
                            return Err(SessionError::WrongToken);
                        };
                        let class = surgeist_html::bounded::element_classification_with(
                            element.name(),
                            |step| match step {
                                surgeist_html::bounded::HtmlElementNameRead::NamespaceHeader
                                | surgeist_html::bounded::HtmlElementNameRead::LocalNameHeader => {
                                    c.charge(WorkCategory::RecordVisit, 1)
                                }
                                surgeist_html::bounded::HtmlElementNameRead::NamespacePair
                                | surgeist_html::bounded::HtmlElementNameRead::LocalNamePair => {
                                    c.charge(WorkCategory::InspectByte, 3)
                                }
                            },
                        )?;
                        if class.common_scope() != state.common_scope {
                            return Err(SessionError::WrongToken);
                        }
                        let subject = node.ok_or(SessionError::WrongToken)?;
                        let family = match class {
                            surgeist_html::HtmlElementClass::KnownHtml(
                                surgeist_html::KnownHtmlElement::A
                                | surgeist_html::KnownHtmlElement::Area,
                            ) => 1,
                            surgeist_html::HtmlElementClass::FormAssociated(
                                surgeist_html::HtmlNativeFormAssociationKind::Image,
                            ) => 2,
                            surgeist_html::HtmlElementClass::KnownHtml(
                                surgeist_html::KnownHtmlElement::Style,
                            ) => 3,
                            surgeist_html::HtmlElementClass::KnownHtml(
                                surgeist_html::KnownHtmlElement::Dialog,
                            ) => 4,
                            _ => 0,
                        };
                        if family != 0 {
                            let scan = |c: &mut ParseControl, charged: bool| {
                                let mut count = 0usize;
                                for candidate in &storage.payloads.values {
                                    if charged {
                                        c.charge(WorkCategory::RecordVisit, 1)?;
                                    }
                                    if matches!((family, candidate),
                                        (1, RetainedPayload::Hyperlink { node, .. })
                                        | (2, RetainedPayload::Image {node, ..})
                                        | (3, RetainedPayload::Style {node, ..})
                                        | (4, RetainedPayload::Dialog {node, ..}) if *node == subject)
                                    {
                                        // At most one per registry row.
                                        count = count.saturating_add(1);
                                    }
                                }
                                Ok::<_, SessionError>(count)
                            };
                            let join = match family {
                                1 => Join::Hyperlink,
                                2 => Join::Image,
                                3 => Join::Style,
                                _ => Join::Dialog,
                            };
                            // The join names the only row unless a second row of a
                            // unique family ever joined a record; only then count by scan.
                            let count = match storage.payloads.unique_row_count(c, subject, join)? {
                                Some(count) => count,
                                None => scan(c, true)?,
                            };
                            #[cfg(test)]
                            {
                                // An uncharged scan, so the check stays out of the work.
                                assert_eq!(
                                    count,
                                    scan(c, false)?,
                                    "join count of record {subject}"
                                );
                            }
                            if count != 1 {
                                return Err(SessionError::AssociatedFacts);
                            }
                        }
                    }
                    if let Some(saved) = state.saved_is {
                        checked_raw(storage, saved, c)?;
                    }
                }
                RetainedPayload::Template { state, .. } => {
                    checked_node(arena, state.contents, c)?;
                    for node in [state.target, state.start, state.end].into_iter().flatten() {
                        checked_node(arena, node, c)?;
                    }
                }
                RetainedPayload::NodeDocument { document, .. } => {
                    checked_document(storage, document, c)?
                }
                RetainedPayload::Script { metadata, .. } => {
                    for document in [
                        metadata.parser_document.as_ref(),
                        metadata.preparation_document.as_ref(),
                    ]
                    .into_iter()
                    .flatten()
                    {
                        checked_document(storage, document, c)?;
                    }
                }
                RetainedPayload::Document {
                    node,
                    state,
                    meta,
                    interaction,
                    ..
                } => {
                    let document = node.ok_or(SessionError::WrongToken)?;
                    if let Some(missing) = &interaction.first {
                        checked_dialog_missing(storage, missing, c)?;
                        if missing.document != DocumentRef::Owned(document) {
                            return Err(SessionError::WrongToken);
                        }
                    }
                    if let template::DocumentState::Ordinary { inert: Some(inert) } = state {
                        checked_node(arena, *inert, c)?;
                    }
                    if let Some(raw) = meta.language {
                        checked_raw(storage, raw, c)?;
                    }
                    // `MetadataDocumentMeta` reads these rows without a controller.
                    for slot in [meta.csp, meta.base_csp].into_iter().flatten() {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if !matches!(
                            storage.payloads.values.get(slot),
                            Some(RetainedPayload::CspPolicy(_))
                        ) {
                            return Err(latch_registry_row(c));
                        }
                    }
                    if let Some(scheme) = &meta.scheme {
                        scheme.require(c)?;
                        let Some(HtmlNodeKind::Document(mode)) =
                            arena.node_at(document).map(|node| node.kind())
                        else {
                            return Err(SessionError::WrongToken);
                        };
                        let expected = surgeist_css::CssParserContext::new(
                            if *mode == HtmlDocumentMode::Quirks {
                                surgeist_css::CssParserMode::Quirks
                            } else {
                                surgeist_css::CssParserMode::Standards
                            },
                        );
                        if scheme.report().result().is_err()
                            || scheme.report().source().context() != expected
                        {
                            return Err(SessionError::WrongToken);
                        }
                    }
                    let mut earliest = None;
                    for domain in meta::MetaDomain::ALL {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if let Some(gap) = meta.gaps.copy(domain) {
                            let Some(RetainedPayload::MetaMissing {
                                document: owner,
                                kind,
                                ..
                            }) = storage.payloads.values.get(gap)
                            else {
                                return Err(SessionError::WrongToken);
                            };
                            if *owner != document || kind.domain() != domain {
                                return Err(SessionError::WrongToken);
                            }
                            if earliest.is_none_or(|(index, _)| gap < index) {
                                earliest = Some((gap, domain));
                            }
                        }
                    }
                    if earliest.map(|(_, domain)| domain) != meta.first {
                        return Err(SessionError::WrongToken);
                    }
                }
                RetainedPayload::MetaMissing {
                    document,
                    subject,
                    cause,
                    ..
                } => {
                    checked_document(storage, &DocumentRef::Owned(*document), c)?;
                    checked_node(arena, *subject, c)?;
                    if let Some(cause) = cause {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if storage
                            .evidence
                            .as_ref()
                            .and_then(|e| e.retained_cause(cause))
                            .is_none()
                        {
                            return Err(SessionError::WrongToken);
                        }
                    }
                }
                RetainedPayload::InputSource {
                    token,
                    value_attribute,
                    ..
                } => {
                    c.require_source_range(token)?;
                    if let Some(attribute) = value_attribute {
                        c.require_source_range(attribute)?;
                        if attribute.start() < token.start() || attribute.end() > token.end() {
                            return Err(SessionError::WrongToken);
                        }
                    }
                }
                RetainedPayload::PiAttributes { ready, .. } if !ready => {
                    return Err(SessionError::ProcessingInstructionMapUnavailable);
                }
                _ => {}
            }
        }
        pending_association::qualify(storage, c)?;
        if storage.fragment.is_some() {
            fragment_document::qualify(storage, c)?;
            frozen.prepare_fragment_effects(c)?;
        }
        payload_write::<Self>(c, 1)?;
        payload_write::<ReportStorage>(c, 1)?;
        Ok(Self {
            history: PreparedHistory::prepare(input, c)?,
            closed_common,
            associated_slot,
        })
    }
    pub(crate) fn commit(self) -> Result<ReportStorage, Unpublishable> {
        Ok(ReportStorage {
            history: self.history.commit()?,
            closed_common: self.closed_common,
            associated_slot: self.associated_slot,
        })
    }
}
fn checked_document(
    storage: &FrozenStorage,
    document: &DocumentRef,
    c: &mut ParseControl,
) -> Result<(), SessionError> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    match document {
        DocumentRef::Owned(index) => {
            if !matches!(
                arena(storage).node_at(*index).map(|record| record.kind()),
                Some(HtmlNodeKind::Document(_))
            ) {
                return Err(SessionError::NodeDocumentUnavailable);
            }
        }
        DocumentRef::Captured(index) => {
            storage
                .fragment
                .as_ref()
                .ok_or(SessionError::ContextUnavailable)?
                .context
                .capture()
                .node(index.subject())
                .map_err(|_| SessionError::NodeDocumentUnavailable)?;
        }
    }
    Ok(())
}
fn arena(storage: &FrozenStorage) -> &HtmlConstructionResult {
    &storage.view
}
/// The record `node` names, which must belong to this result.
fn checked_node<'a>(
    arena: &'a HtmlConstructionResult,
    node: usize,
    c: &mut ParseControl,
) -> Result<crate::bounded::HtmlConstructionNode<'a>, SessionError> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    arena.node_at(node).ok_or(SessionError::ForeignNode)
}
fn checked_select(
    arena: &HtmlConstructionResult,
    node: usize,
    c: &mut ParseControl,
) -> Result<(), SessionError> {
    let record = checked_node(arena, node, c)?;
    c.charge(WorkCategory::RecordVisit, 1)?;
    let HtmlNodeKind::Element(element) = record.kind() else {
        return Err(SessionError::SelectMetadataUnavailable);
    };
    if !exact_ascii(
        c,
        element.name().namespace().uri(),
        "http://www.w3.org/1999/xhtml",
    )? || !exact_ascii(c, Some(element.name().local_name()), "select")?
    {
        return Err(SessionError::SelectMetadataUnavailable);
    }
    Ok(())
}
fn checked_raw(
    storage: &FrozenStorage,
    slot: usize,
    c: &mut ParseControl,
) -> Result<(), SessionError> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    let Some(RetainedPayload::String {
        value, allocation, ..
    }) = storage.payloads.values.get(slot)
    else {
        return Err(SessionError::WrongToken);
    };
    c.require_document_string(allocation, value)?;
    Ok(())
}
fn checked_dialog_missing(
    storage: &FrozenStorage,
    missing: &dialog::Missing,
    c: &mut ParseControl,
) -> Result<(), SessionError> {
    checked_node(arena(storage), missing.subject, c)?;
    checked_document(storage, &missing.document, c)?;
    c.charge(WorkCategory::RecordVisit, 1)?;
    if let Some(cause) = &missing.cause
        && storage
            .evidence
            .as_ref()
            .and_then(|e| e.retained_cause(cause))
            .is_none()
    {
        return Err(SessionError::WrongToken);
    }
    Ok(())
}
fn metadata_node(payload: &RetainedPayload) -> Option<usize> {
    match payload {
        // Notification eligibility is a parser proof, not authored state.
        // Details name/open remain canonical native attributes.
        RetainedPayload::Details { .. } => None,
        RetainedPayload::Shadow { root, .. } => *root,
        RetainedPayload::StandaloneDocumentCompletion { node, .. }
        | RetainedPayload::BaseDocument { node, .. }
        | RetainedPayload::Hyperlink { node, .. }
        | RetainedPayload::Image { node, .. }
        | RetainedPayload::Style { node, .. }
        | RetainedPayload::Dialog { node, .. } => Some(*node),
        RetainedPayload::Base { node, .. }
        | RetainedPayload::Nonce { node, .. }
        | RetainedPayload::LookupRegistry { node, .. }
        | RetainedPayload::PiAttributes { node, .. }
        | RetainedPayload::NodeDocument { node, .. }
        | RetainedPayload::Document { node, .. }
        | RetainedPayload::Template { node, .. }
        | RetainedPayload::SvgScript { node, .. }
        | RetainedPayload::Script { node, .. }
        | RetainedPayload::Link { node, .. }
        | RetainedPayload::FormAssociation { node, .. }
        | RetainedPayload::Textarea { node, .. }
        | RetainedPayload::Output { node, .. }
        | RetainedPayload::Input { node, .. }
        | RetainedPayload::InputSource { node, .. }
        | RetainedPayload::SelectFamily { node, .. } => *node,
        _ => None,
    }
}
impl ReportStorage {
    pub(crate) fn release_execution_inputs(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<(), ParseStop> {
        let Some(slot) = self.associated_slot else {
            return Ok(());
        };
        let row = self.history.storage_mut().payloads.values.get_mut(slot);
        let Some(row) = row else {
            return Err(c.fail_invariant(crate::parser::control::Invariant::RegistryRow));
        };
        match mem::replace(row, RetainedPayload::Retired) {
            RetainedPayload::AssociatedInputs { inputs, usage } => {
                self.associated_slot = None;
                let resources = usage.resources();
                drop((inputs, usage));
                c.commit_prepared_release(&resources)?;
                Ok(())
            }
            // Another kind of row is put back where it was.
            other => {
                *row = other;
                Err(c.fail_invariant(crate::parser::control::Invariant::RegistryRow))
            }
        }
    }
    pub(crate) fn fragment_parser_document<'a>(
        &'a self,
        source: &'a crate::HtmlSource,
    ) -> Result<fragment_document::FragmentParserDocument<'a>, read::UnresolvedReference> {
        fragment_document::FragmentParserDocument::new(&self.history, source)
    }
    pub(crate) fn history(&self) -> &FrozenHistory {
        &self.history
    }
    /// The retained history, for tests that corrupt what a report holds.
    #[cfg(test)]
    pub(crate) fn history_mut(&mut self) -> &mut FrozenHistory {
        &mut self.history
    }
    pub(crate) fn evidence(&self) -> &ParseEvidence {
        self.history.evidence()
    }
    /// The effects of a fragment report; a Document report holds none.
    pub(crate) fn effects(&self) -> Option<context::FrozenFragmentEffects<'_>> {
        self.history.storage().fragment_effects()
    }
    pub(crate) fn metadata(&self) -> MetadataCursor<'_> {
        let storage = self.history.storage();
        MetadataCursor {
            history: &self.history,
            closed_common: self.closed_common,
            storage,
            arena: arena(storage),
            next: 0,
            fault: None,
        }
    }
}
pub struct MetadataCursor<'a> {
    closed_common: bool,
    history: &'a FrozenHistory,
    storage: &'a FrozenStorage,
    arena: &'a HtmlConstructionResult,
    next: usize,
    fault: Option<read::UnresolvedReference>,
}
#[expect(
    clippy::large_enum_variant,
    reason = "The record holds what it resolved, and is lent by value so the read path allocates nothing; boxing it would add an allocation where no controller exists to charge it"
)]
pub enum MetadataStep<'a> {
    Record(MetadataRef<'a>),
    /// A slot that holds no metadata record.
    Skipped,
    Complete,
    /// The record of this slot names a reference the report cannot resolve.
    /// The cursor stays faulted, so a caller that continues is told again.
    Fault(read::UnresolvedReference),
}
impl<'a> MetadataCursor<'a> {
    /// One actual canonical slot, never a next-match loop or materialization.
    pub fn advance(&mut self) -> MetadataStep<'a> {
        if let Some(fault) = self.fault {
            return MetadataStep::Fault(fault);
        }
        let Some(payload) = self.storage.payloads.values.get(self.next) else {
            return MetadataStep::Complete;
        };
        // `next` named a registry row, so it is below the address space.
        self.next = self.next.saturating_add(1);
        let Some(node) = metadata_node(payload) else {
            return MetadataStep::Skipped;
        };
        let resolver = Resolver {
            history: self.history,
            storage: self.storage,
            arena: self.arena,
        };
        match MetadataRef::issue(resolver, self.closed_common, node, payload) {
            Ok(record) => MetadataStep::Record(record),
            Err(fault) => {
                self.fault = Some(fault);
                MetadataStep::Fault(fault)
            }
        }
    }
}
/// The tables a record's references are resolved against. A record is issued
/// only when every reference it holds resolves, so what it lends cannot fail.
#[derive(Clone, Copy)]
struct Resolver<'a> {
    history: &'a FrozenHistory,
    storage: &'a FrozenStorage,
    arena: &'a HtmlConstructionResult,
}
type Fault = read::UnresolvedReference;
fn unresolved(kind: read::UnresolvedKind) -> Fault {
    read::UnresolvedReference::new(kind)
}
impl<'a> Resolver<'a> {
    fn native(&self, index: usize) -> Result<HtmlNode<'a>, Fault> {
        self.arena
            .node_at(index)
            .map(|node| node.as_node())
            .ok_or(unresolved(read::UnresolvedKind::Node))
    }
    fn optional_native(&self, index: Option<usize>) -> Result<Option<HtmlNode<'a>>, Fault> {
        index.map(|index| self.native(index)).transpose()
    }
    fn raw(&self, slot: usize) -> Result<&'a DocumentString, Fault> {
        match self.storage.payloads.values.get(slot) {
            Some(RetainedPayload::String { value, .. }) => Ok(value),
            _ => Err(unresolved(read::UnresolvedKind::Fact)),
        }
    }
    fn policy(&self, slot: usize) -> Result<Option<&'a surgeist_html::HtmlCspAssociation>, Fault> {
        match self.storage.payloads.values.get(slot) {
            Some(RetainedPayload::CspPolicy(value)) => Ok(value.value.as_ref()),
            _ => Err(unresolved(read::UnresolvedKind::Fact)),
        }
    }
    fn cause(
        &self,
        cause: &Option<provenance::ConstructionCauseRef>,
    ) -> Result<Option<read::HtmlParseCause<'a>>, Fault> {
        cause
            .as_ref()
            .map(|cause| {
                self.history
                    .evidence()
                    .retained_cause(cause)
                    .map(|cause| read::HtmlParseCause::qualified(self.history, cause))
                    .ok_or(unresolved(read::UnresolvedKind::Cause))
            })
            .transpose()
    }
    fn document(&self, document: &DocumentRef) -> Result<MetadataDocument<'a>, Fault> {
        match document {
            DocumentRef::Owned(index) => self.native(*index).map(MetadataDocument::Owned),
            DocumentRef::Captured(index) => self
                .storage
                .fragment
                .as_ref()
                .ok_or(unresolved(read::UnresolvedKind::Fragment))?
                .context
                .capture()
                .node(index.subject())
                .map(MetadataDocument::Captured)
                .map_err(|_| unresolved(read::UnresolvedKind::CapturedNode)),
        }
    }
    fn optional_document(
        &self,
        document: Option<&DocumentRef>,
    ) -> Result<Option<MetadataDocument<'a>>, Fault> {
        document.map(|document| self.document(document)).transpose()
    }
    fn dialog_missing(
        &self,
        missing: &dialog::Missing,
    ) -> Result<MetadataDialogMissing<'a>, Fault> {
        Ok(MetadataDialogMissing {
            subject: self.native(missing.subject)?,
            document: self.document(&missing.document)?,
            action: missing.action,
            cause: self.cause(&missing.cause)?,
        })
    }
    fn disposition(
        &self,
        first: Option<&dialog::Missing>,
    ) -> Result<MetadataDialogDisposition<'a>, Fault> {
        Ok(match first {
            None => MetadataDialogDisposition::Closed,
            Some(missing) => MetadataDialogDisposition::Unavailable(self.dialog_missing(missing)?),
        })
    }
    fn meta_missing(&self, slot: usize) -> Result<MetadataMetaMissing<'a>, Fault> {
        let Some(RetainedPayload::MetaMissing {
            document,
            subject,
            kind,
            cause,
        }) = self.storage.payloads.values.get(slot)
        else {
            return Err(unresolved(read::UnresolvedKind::Fact));
        };
        Ok(MetadataMetaMissing {
            document: self.native(*document)?,
            subject: self.native(*subject)?,
            kind,
            cause: self.cause(cause)?,
        })
    }
    fn base(&self, value: &'a base::FrozenBase) -> Result<MetadataBase<'a>, Fault> {
        Ok(MetadataBase {
            value: &value.value,
            document: self.native(value.document)?,
            revision: value.environment.document().revision(),
            responded: value.response == base::Response::Finished,
        })
    }
    fn document_meta(
        &self,
        document: usize,
        meta: &'a meta::MetaHistory,
    ) -> Result<MetadataDocumentMeta<'a>, Fault> {
        let mut fault = None;
        let gaps = meta.gaps.map(|slot| {
            let slot = slot?;
            match self.meta_missing(slot) {
                Ok(missing) => Some(missing),
                Err(error) => {
                    fault.get_or_insert(error);
                    None
                }
            }
        });
        if let Some(fault) = fault {
            return Err(fault);
        }
        // The policy of a domain with a gap is not an answer.
        let csp = match (meta.gaps.get(meta::MetaDomain::Csp), meta.csp) {
            (None, Some(slot)) => self.policy(slot)?,
            _ => None,
        };
        let base_policy = match meta.base_csp {
            Some(slot) => self.policy(slot)?,
            None => None,
        };
        let first = meta
            .first
            .map(|domain| {
                gaps.copy(domain)
                    .ok_or(unresolved(read::UnresolvedKind::Fact))
            })
            .transpose()?;
        Ok(MetadataDocumentMeta {
            document: self.native(document)?,
            meta,
            csp,
            base_policy,
            language: meta.language.map(|slot| self.raw(slot)).transpose()?,
            gaps,
            first,
        })
    }
}
/// What a record's references resolved to, by the kind of row it was issued for.
#[derive(Clone, Copy)]
#[expect(
    clippy::large_enum_variant,
    reason = "Only a Document row holds its Meta history; the record is lent by value so the read path allocates nothing, and boxing it would add an allocation where no controller exists to charge it"
)]
enum Refs<'a> {
    None,
    Raw(&'a DocumentString),
    Input {
        raw: &'a DocumentString,
        image_source: Option<&'a DocumentString>,
    },
    Creation(Option<&'a DocumentString>),
    Owner(Option<HtmlNode<'a>>),
    Dialog(MetadataDialog<'a>),
    Document {
        interaction: MetadataDocumentInteraction<'a>,
        meta: MetadataDocumentMeta<'a>,
        inert: Option<HtmlNode<'a>>,
    },
    Style(MetadataStyle<'a>),
    Hyperlink(MetadataHyperlink<'a>),
    Image(MetadataImage<'a>),
    Base(MetadataBaseState<'a>),
    FirstBase(Option<HtmlNode<'a>>),
    Link {
        parser_document: Option<MetadataDocument<'a>>,
        flags: crate::HtmlLinkFlags,
        history: crate::HtmlLinkHistory,
    },
    NodeDocument(MetadataDocument<'a>),
    Script(Option<MetadataDocument<'a>>, Option<MetadataDocument<'a>>),
    Template(MetadataTemplate<'a>),
}
impl<'a> Refs<'a> {
    fn resolve(
        r: Resolver<'a>,
        payload: &'a RetainedPayload,
        node: HtmlNode<'a>,
    ) -> Result<Self, Fault> {
        Ok(match payload {
            RetainedPayload::Nonce { raw, .. } => Self::Raw(r.raw(*raw)?),
            RetainedPayload::Textarea { state, .. } => Self::Raw(r.raw(state.raw)?),
            RetainedPayload::Input { raw, state, .. } => Self::Input {
                raw: r.raw(*raw)?,
                image_source: match state.image {
                    input_image::ImageState::Last {
                        source: Some(source),
                        ..
                    } => Some(r.raw(source)?),
                    _ => None,
                },
            },
            RetainedPayload::LookupRegistry {
                creation: Some(state),
                ..
            } => Self::Creation(state.saved_is.map(|slot| r.raw(slot)).transpose()?),
            RetainedPayload::FormAssociation { state, .. } => {
                Self::Owner(r.optional_native(state.owner)?)
            }
            RetainedPayload::SelectFamily {
                state: SelectMetadata::Option(state),
                ..
            } => Self::Owner(r.optional_native(state.cached_nearest_select())?),
            RetainedPayload::Dialog { state, .. } => Self::Dialog(MetadataDialog {
                value: r.raw(state.value)?,
                disposition: r.disposition(state.first.as_ref())?,
            }),
            RetainedPayload::Document {
                node: Some(document),
                state,
                meta,
                interaction,
                ..
            } => Self::Document {
                interaction: MetadataDocumentInteraction {
                    document: r.native(*document)?,
                    disposition: r.disposition(interaction.first.as_ref())?,
                },
                meta: r.document_meta(*document, meta)?,
                inert: match state {
                    template::DocumentState::Ordinary { inert } => r.optional_native(*inert)?,
                    template::DocumentState::InertTemplate => None,
                },
            },
            RetainedPayload::Style { state, .. } => Self::Style(MetadataStyle {
                state: state.state,
                parser_document: r.optional_document(state.parser_document.as_ref())?,
                cause: r.cause(&state.cause)?,
            }),
            RetainedPayload::Hyperlink { state, .. } => Self::Hyperlink(MetadataHyperlink {
                document: r.document(&state.document)?,
                cause: r.cause(&state.cause)?,
                raw: state.raw.map(|slot| r.raw(slot)).transpose()?,
                state: match &state.value {
                    associated::HyperlinkValue::Absent => MetadataHyperlinkState::AbsentHref,
                    associated::HyperlinkValue::Observed(value) => {
                        MetadataHyperlinkState::Observed(value)
                    }
                    associated::HyperlinkValue::Unavailable(error) => {
                        MetadataHyperlinkState::Unavailable(error)
                    }
                },
            }),
            RetainedPayload::Image { state, .. } => Self::Image(MetadataImage {
                state,
                dimension_source: r.native(state.dimension_source)?,
                cause: r.cause(&state.cause)?,
                subject: node,
            }),
            RetainedPayload::Base { frozen, .. } => Self::Base(match frozen {
                base::BaseValue::Unfrozen => MetadataBaseState::Unfrozen,
                base::BaseValue::Frozen(value) => MetadataBaseState::Frozen(r.base(value)?),
                base::BaseValue::Unavailable { missing, previous } => {
                    MetadataBaseState::Unavailable {
                        missing: r.meta_missing(*missing)?,
                        previous: previous.as_ref().map(|value| r.base(value)).transpose()?,
                    }
                }
            }),
            RetainedPayload::BaseDocument { first, .. } => {
                Self::FirstBase(r.optional_native(*first)?)
            }
            RetainedPayload::Link {
                node: link,
                parser_document,
            } => {
                let HtmlNodeKind::Element(element) = node.kind() else {
                    return Err(unresolved(read::UnresolvedKind::Node));
                };
                let flags = element
                    .link_flags()
                    .ok_or(unresolved(read::UnresolvedKind::Node))?;
                // The row the join names must be this link's own.
                let link = link.ok_or(unresolved(read::UnresolvedKind::Node))?;
                let current = match r
                    .storage
                    .payloads
                    .joined_uncharged(link, Join::NodeDocument)
                    .and_then(|slot| r.storage.payloads.values.get(slot))
                {
                    Some(RetainedPayload::NodeDocument {
                        node: Some(owner),
                        document,
                    }) if *owner == link => document,
                    _ => return Err(unresolved(read::UnresolvedKind::Fact)),
                };
                Self::Link {
                    parser_document: r.optional_document(parser_document.as_ref())?,
                    flags,
                    history: crate::HtmlLinkHistory::new(
                        flags,
                        parser_document.as_ref() == Some(current),
                    ),
                }
            }
            RetainedPayload::NodeDocument { document, .. } => {
                Self::NodeDocument(r.document(document)?)
            }
            RetainedPayload::Script { metadata, .. } => Self::Script(
                r.optional_document(metadata.parser_document.as_ref())?,
                r.optional_document(metadata.preparation_document.as_ref())?,
            ),
            RetainedPayload::Template { state, .. } => Self::Template(MetadataTemplate {
                contents: r.native(state.contents)?,
                target: r.optional_native(state.target)?,
                start: r.optional_native(state.start)?,
                end: r.optional_native(state.end)?,
            }),
            _ => Self::None,
        })
    }
}
/// Issued only by the qualified report's fixed slot observation. No caller can
/// supply a raw payload index or mix this lender with another result's arena.
/// Every reference the record holds was resolved when it was issued.
pub struct MetadataRef<'a> {
    node: HtmlNode<'a>,
    closed_common: bool,
    payload: &'a RetainedPayload,
    refs: Refs<'a>,
}
#[derive(Clone, Copy)]
pub enum MetadataDocument<'a> {
    Owned(HtmlNode<'a>),
    Captured(crate::HtmlCapturedNode<'a>),
}
/// Current source-qualified closed-services prefix, or its first real gap.
/// Closed does not assert that the Dialog lacks an open content attribute.
#[derive(Clone, Copy)]
pub enum MetadataDialogDisposition<'a> {
    Closed,
    Unavailable(MetadataDialogMissing<'a>),
}
#[derive(Clone, Copy)]
pub struct MetadataDialogMissing<'a> {
    subject: HtmlNode<'a>,
    document: MetadataDocument<'a>,
    action: surgeist_html::HtmlDialogAction,
    cause: Option<read::HtmlParseCause<'a>>,
}
impl<'a> MetadataDialogMissing<'a> {
    pub fn subject(&self) -> HtmlNode<'a> {
        self.subject
    }
    pub fn document(&self) -> MetadataDocument<'a> {
        self.document
    }
    pub fn action(&self) -> surgeist_html::HtmlDialogAction {
        self.action
    }
    pub fn cause(&self) -> Option<read::HtmlParseCause<'a>> {
        self.cause
    }
}
#[derive(Clone, Copy)]
pub struct MetadataDialog<'a> {
    value: &'a DocumentString,
    disposition: MetadataDialogDisposition<'a>,
}
impl<'a> MetadataDialog<'a> {
    pub fn return_value(&self) -> &'a DocumentString {
        self.value
    }
    pub fn disposition(&self) -> MetadataDialogDisposition<'a> {
        self.disposition
    }
}
#[derive(Clone, Copy)]
pub struct MetadataDocumentInteraction<'a> {
    document: HtmlNode<'a>,
    disposition: MetadataDialogDisposition<'a>,
}
impl<'a> MetadataDocumentInteraction<'a> {
    pub fn document(&self) -> HtmlNode<'a> {
        self.document
    }
    pub fn disposition(&self) -> MetadataDialogDisposition<'a> {
        self.disposition
    }
}
#[derive(Clone, Copy)]
pub struct MetadataTemplate<'a> {
    pub(crate) contents: HtmlNode<'a>,
    pub(crate) target: Option<HtmlNode<'a>>,
    pub(crate) start: Option<HtmlNode<'a>>,
    pub(crate) end: Option<HtmlNode<'a>>,
}
/// Qualified current Base state; old snapshots are historical, never defaults.
#[derive(Clone, Copy)]
pub enum MetadataBaseState<'a> {
    Unfrozen,
    Frozen(MetadataBase<'a>),
    Unavailable {
        missing: MetadataMetaMissing<'a>,
        previous: Option<MetadataBase<'a>>,
    },
}
/// A completed intrinsic/external domain or its actual first missing operation.
#[derive(Clone, Copy)]
pub enum MetadataMetaDisposition<'a> {
    Complete,
    Unavailable(MetadataMetaMissing<'a>),
}
#[derive(Clone, Copy)]
pub enum MetadataMetaRefresh<'a> {
    Known(bool),
    Unavailable(MetadataMetaMissing<'a>),
}
/// Original-controller closed offline policy. This value is not a native
/// initialization token and conveys no canonical target UI association proof.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataMetaProfile {
    OfflineDoNothingNoUi,
}
#[derive(Clone, Copy)]
pub struct MetadataMetaMissing<'a> {
    document: HtmlNode<'a>,
    subject: HtmlNode<'a>,
    kind: &'a meta::MetaMissingKind,
    cause: Option<read::HtmlParseCause<'a>>,
}
impl<'a> MetadataMetaMissing<'a> {
    pub fn document(&self) -> HtmlNode<'a> {
        self.document
    }
    pub fn subject(&self) -> HtmlNode<'a> {
        self.subject
    }
    pub fn kind(&self) -> &'a meta::MetaMissingKind {
        self.kind
    }
    pub fn cause(&self) -> Option<read::HtmlParseCause<'a>> {
        self.cause
    }
}
#[derive(Clone, Copy)]
pub struct MetadataDocumentMeta<'a> {
    document: HtmlNode<'a>,
    meta: &'a meta::MetaHistory,
    csp: Option<&'a surgeist_html::HtmlCspAssociation>,
    base_policy: Option<&'a surgeist_html::HtmlCspAssociation>,
    language: Option<&'a DocumentString>,
    gaps: meta::MetaDomainTable<Option<MetadataMetaMissing<'a>>>,
    first: Option<MetadataMetaMissing<'a>>,
}
impl<'a> MetadataDocumentMeta<'a> {
    /// Exact original construction policy; absent means no protocol qualification.
    pub fn csp_association(&self) -> Option<&'a surgeist_html::HtmlCspAssociation> {
        self.csp
    }

    /// Original policy explicitly paired with the supplied initial Base facts.
    /// A later policy never substitutes for this historical qualification.
    pub fn base_policy_association(&self) -> Option<&'a surgeist_html::HtmlCspAssociation> {
        self.base_policy
    }

    pub fn document(&self) -> HtmlNode<'a> {
        self.document
    }
    pub fn profile(&self) -> MetadataMetaProfile {
        MetadataMetaProfile::OfflineDoNothingNoUi
    }
    /// Known absent and Known tag: both are original-controller current values.
    pub fn pragma_language(&self) -> Option<&'a DocumentString> {
        self.language
    }
    /// Last completed original-controller selection, or its reached missing dependency.
    pub fn theme_color(&self) -> Result<Option<surgeist_color::Color>, MetadataMetaMissing<'a>> {
        match self.gaps.copy(meta::MetaDomain::ThemeColor) {
            Some(missing) => Err(missing),
            None => Ok(self.meta.theme),
        }
    }
    /// The original immutable CSS report and context, without reparsing authored attributes.
    pub fn color_scheme(
        &self,
    ) -> Option<&'a Arc<surgeist_css::values::color_adjust::CssColorSchemeParseReport>> {
        self.meta
            .scheme
            .as_ref()
            .map(crate::parser::control::CssSchemeValue::report)
    }
    pub fn referrer_policy(&self) -> HtmlReferrerPolicy {
        self.meta.referrer
    }
    pub fn refresh(&self) -> MetadataMetaRefresh<'a> {
        match self.gaps.copy(meta::MetaDomain::Refresh) {
            Some(missing) => MetadataMetaRefresh::Unavailable(missing),
            None => MetadataMetaRefresh::Known(self.meta.refresh),
        }
    }
    pub fn disposition(&self, domain: meta::MetaDomain) -> MetadataMetaDisposition<'a> {
        match self.gaps.copy(domain) {
            Some(missing) => MetadataMetaDisposition::Unavailable(missing),
            None => MetadataMetaDisposition::Complete,
        }
    }
    pub fn first_missing(&self) -> Option<MetadataMetaMissing<'a>> {
        self.first
    }
}
#[derive(Clone, Copy)]
pub struct MetadataBase<'a> {
    value: &'a crate::HtmlBaseUrlRecord,
    document: HtmlNode<'a>,
    revision: &'a crate::HtmlBaseFactsRevision,
    responded: bool,
}
impl<'a> MetadataBase<'a> {
    pub fn value(&self) -> &'a crate::HtmlBaseUrlRecord {
        self.value
    }
    pub fn document(&self) -> HtmlNode<'a> {
        self.document
    }
    pub fn revision(&self) -> &'a crate::HtmlBaseFactsRevision {
        self.revision
    }
    pub fn responded(&self) -> bool {
        self.responded
    }
}
#[derive(Clone, Copy)]
pub enum MetadataHyperlinkState<'a> {
    AbsentHref,
    Observed(&'a surgeist_html_semantics::HtmlHyperlinkUrlSnapshot),
    Unavailable(&'a associated::HyperlinkUnavailable),
}
#[derive(Clone, Copy)]
pub struct MetadataHyperlink<'a> {
    document: MetadataDocument<'a>,
    state: MetadataHyperlinkState<'a>,
    cause: Option<read::HtmlParseCause<'a>>,
    raw: Option<&'a DocumentString>,
}
impl<'a> MetadataHyperlink<'a> {
    pub fn document(&self) -> &MetadataDocument<'a> {
        &self.document
    }
    pub fn state(&self) -> &MetadataHyperlinkState<'a> {
        &self.state
    }
    pub fn cause(&self) -> Option<&read::HtmlParseCause<'a>> {
        self.cause.as_ref()
    }
    pub fn raw(&self) -> Option<&'a DocumentString> {
        self.raw
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MetadataImageUpdate {
    Waiting {
        generation: u64,
        flags: surgeist_html::HtmlImageUpdateFlags,
    },
    UnavailableActivity {
        latest_started: u64,
    },
}
#[derive(Clone, Copy)]
pub struct MetadataImage<'a> {
    state: &'a associated::ImageMetadata,
    dimension_source: HtmlNode<'a>,
    cause: Option<read::HtmlParseCause<'a>>,
    subject: HtmlNode<'a>,
}
impl<'a> MetadataImage<'a> {
    pub fn state(&self) -> &'a surgeist_html::HtmlImgFreshState {
        &self.state.fresh
    }
    pub fn dimension_attribute_source(&self) -> HtmlNode<'a> {
        self.dimension_source
    }
    pub fn latest_started(&self) -> u64 {
        self.state.latest
    }
    pub fn cause(&self) -> Option<read::HtmlParseCause<'a>> {
        self.cause
    }

    pub fn update(&self) -> MetadataImageUpdate {
        match self.state.waiting {
            Some(flags) => MetadataImageUpdate::Waiting {
                generation: self.state.latest,
                flags,
            },
            None => MetadataImageUpdate::UnavailableActivity {
                latest_started: self.state.latest,
            },
        }
    }
    pub fn subject(&self) -> HtmlNode<'a> {
        self.subject
    }
}
#[derive(Clone, Copy)]
pub struct MetadataStyle<'a> {
    state: surgeist_html::HtmlStyleState,
    parser_document: Option<MetadataDocument<'a>>,
    cause: Option<read::HtmlParseCause<'a>>,
}
impl<'a> MetadataStyle<'a> {
    pub fn state(&self) -> surgeist_html::HtmlStyleState {
        self.state
    }
    pub fn creating_parser_document(&self) -> Option<&MetadataDocument<'a>> {
        self.parser_document.as_ref()
    }
    pub fn cause(&self) -> Option<&read::HtmlParseCause<'a>> {
        self.cause.as_ref()
    }
    pub fn association(&self) -> style::StyleAssociation {
        style::StyleAssociation::Pending
    }
}
impl<'a> MetadataRef<'a> {
    /// Every reference the record holds is resolved here, so that the readers
    /// of the record cannot fail. A reference the report cannot resolve is the
    /// fault.
    fn issue(
        resolver: Resolver<'a>,
        closed_common: bool,
        node: usize,
        payload: &'a RetainedPayload,
    ) -> Result<Self, Fault> {
        let node = resolver.native(node)?;
        Ok(Self {
            node,
            closed_common,
            payload,
            refs: Refs::resolve(resolver, payload, node)?,
        })
    }
    /// Exact creation subject in a completed ordinary offline or contextual
    /// no-execution/no-interaction profile with actual Null lookup registry.
    /// The source scope is retained at actual creation;
    /// report preparation validates it through HTML's one bounded classifier.
    /// This qualifies the profile's current null-only references, not native freshness.
    pub fn common_state(
        &self,
    ) -> Option<surgeist_html::HtmlCommonState<core::convert::Infallible>> {
        if !self.closed_common {
            return None;
        }
        let RetainedPayload::LookupRegistry {
            registry: context::RegistryRef::Null,
            creation: Some(state),
            ..
        } = self.payload
        else {
            return None;
        };
        Some(surgeist_html::HtmlCommonState::new(state.common_scope))
    }

    /// Fixed current Dialog state issued by the actual original controller.
    pub fn dialog(&self) -> Option<MetadataDialog<'a>> {
        match self.refs {
            Refs::Dialog(dialog) => Some(dialog),
            _ => None,
        }
    }
    /// Interaction history of this actual newly owned Document, not a captured
    /// target's whole list state and not native-constructor evidence.
    pub fn document_interaction(&self) -> Option<MetadataDocumentInteraction<'a>> {
        match self.refs {
            Refs::Document { interaction, .. } => Some(interaction),
            _ => None,
        }
    }
    pub fn style(&self) -> Option<MetadataStyle<'a>> {
        match self.refs {
            Refs::Style(style) => Some(style),
            _ => None,
        }
    }
    pub fn hyperlink(&self) -> Option<MetadataHyperlink<'a>> {
        match self.refs {
            Refs::Hyperlink(hyperlink) => Some(hyperlink),
            _ => None,
        }
    }
    pub fn image(&self) -> Option<MetadataImage<'a>> {
        match self.refs {
            Refs::Image(image) => Some(image),
            _ => None,
        }
    }

    /// Wrong row is None. Unfrozen and unavailable current state are distinct;
    /// a genuine prior frozen value remains historical inside Unavailable.
    pub fn base_state(&self) -> Option<MetadataBaseState<'a>> {
        match self.refs {
            Refs::Base(state) => Some(state),
            _ => None,
        }
    }
    /// Actual per-Document history. No getter traverses the tree or replays Meta.
    pub fn document_meta(&self) -> Option<MetadataDocumentMeta<'a>> {
        match self.refs {
            Refs::Document { meta, .. } => Some(meta),
            _ => None,
        }
    }
    pub fn document_base_facts(&self) -> Option<&'a crate::HtmlParseBaseFacts> {
        match self.payload {
            RetainedPayload::BaseDocument { facts, .. } => Some(facts),
            _ => None,
        }
    }
    pub fn document_first_base(&self) -> Option<Option<HtmlNode<'a>>> {
        match self.refs {
            Refs::FirstBase(first) => Some(first),
            _ => None,
        }
    }
    /// Actual link flags and creating-parser document. The current node document
    /// is exposed by its separate canonical NodeDocument metadata record.
    pub fn link_parser_document(&self) -> Option<Option<MetadataDocument<'a>>> {
        match self.refs {
            Refs::Link {
                parser_document, ..
            } => Some(parser_document),
            _ => None,
        }
    }
    /// The actual flags of a link record; `None` for any other record.
    pub fn link_flags(&self) -> Option<crate::HtmlLinkFlags> {
        match self.refs {
            Refs::Link { flags, .. } => Some(flags),
            _ => None,
        }
    }
    /// Ordinary borrowed readback of actual flags and the current node-document
    /// join, resolved when the record was issued; `None` for any other record.
    pub fn link_history(&self) -> Option<crate::HtmlLinkHistory> {
        match self.refs {
            Refs::Link { history, .. } => Some(history),
            _ => None,
        }
    }
    pub fn node(&self) -> HtmlNode<'a> {
        self.node
    }
    pub(super) fn nonce(&self) -> Option<&'a DocumentString> {
        match (self.payload, self.refs) {
            (RetainedPayload::Nonce { .. }, Refs::Raw(raw)) => Some(raw),
            _ => None,
        }
    }
    pub fn node_document(&self) -> Option<MetadataDocument<'a>> {
        match self.refs {
            Refs::NodeDocument(document) => Some(document),
            _ => None,
        }
    }
    pub fn script_documents(
        &self,
    ) -> Option<(Option<MetadataDocument<'a>>, Option<MetadataDocument<'a>>)> {
        match self.refs {
            Refs::Script(parser, preparation) => Some((parser, preparation)),
            _ => None,
        }
    }
    pub fn template(&self) -> Option<MetadataTemplate<'a>> {
        match self.refs {
            Refs::Template(template) => Some(template),
            _ => None,
        }
    }
    pub fn creation(&self) -> Option<(crate::HtmlCustomElementState, Option<&'a DocumentString>)> {
        match (self.payload, self.refs) {
            (
                RetainedPayload::LookupRegistry {
                    creation: Some(state),
                    ..
                },
                Refs::Creation(saved),
            ) => Some((state.state, saved)),
            _ => None,
        }
    }
    pub fn input_source(
        &self,
    ) -> Option<(
        &'a crate::HtmlSourceRange,
        Option<&'a crate::HtmlSourceRange>,
    )> {
        match self.payload {
            RetainedPayload::InputSource {
                token,
                value_attribute,
                ..
            } => Some((token, value_attribute.as_ref())),
            _ => None,
        }
    }
    /// The complete map is already owned and qualified. This returns exactly
    /// one original canonical pair, with no key lookup or copied map.
    pub fn pi_attribute_count(&self) -> Option<usize> {
        match self.payload {
            RetainedPayload::PiAttributes { values, .. } => Some(values.len()),
            _ => None,
        }
    }
    pub fn pi_attribute(&self, index: usize) -> Option<(&'a DocumentString, &'a DocumentString)> {
        match self.payload {
            RetainedPayload::PiAttributes { values, .. } => values
                .as_slice()
                .get(index)
                .map(|pair| (&pair.name, &pair.value)),
            _ => None,
        }
    }
    pub fn shadow(&self) -> Option<(bool, bool, bool)> {
        match self.payload {
            RetainedPayload::Shadow { state, .. } => Some((
                state.keep_registry_null,
                state.declarative,
                state.available_to_internals,
            )),
            _ => None,
        }
    }
    pub fn document_facts(&self) -> Option<(crate::HtmlDocumentClass, bool, Option<HtmlNode<'a>>)> {
        match (self.payload, self.refs) {
            (
                RetainedPayload::Document {
                    state: template::DocumentState::Ordinary { .. },
                    class,
                    ..
                },
                Refs::Document { inert, .. },
            ) => Some((*class, false, inert)),
            (
                RetainedPayload::Document {
                    state: template::DocumentState::InertTemplate,
                    class,
                    ..
                },
                _,
            ) => Some((*class, true, None)),
            _ => None,
        }
    }
    pub(crate) fn input(&self) -> Option<(&'a InputState, &'a DocumentString)> {
        match (self.payload, self.refs) {
            (RetainedPayload::Input { state, .. }, Refs::Input { raw, .. }) => Some((state, raw)),
            _ => None,
        }
    }
    /// The last source observed while Image, when an Input has one.
    pub(super) fn input_image_source_value(&self) -> Option<&'a DocumentString> {
        match self.refs {
            Refs::Input { image_source, .. } => image_source,
            _ => None,
        }
    }
    pub(crate) fn textarea(&self) -> Option<(&'a TextareaState, &'a DocumentString)> {
        match (self.payload, self.refs) {
            (RetainedPayload::Textarea { state, .. }, Refs::Raw(raw)) => Some((state, raw)),
            _ => None,
        }
    }
    pub(crate) fn output(&self) -> Option<&'a OutputState> {
        match self.payload {
            RetainedPayload::Output { state, .. } => Some(state),
            _ => None,
        }
    }
    pub(crate) fn form(&self) -> Option<(&'a FormAssociationState, Option<HtmlNode<'a>>)> {
        match (self.payload, self.refs) {
            (RetainedPayload::FormAssociation { state, .. }, Refs::Owner(owner)) => {
                Some((state, owner))
            }
            _ => None,
        }
    }
    pub(crate) fn script(&self) -> Option<&'a ScriptMetadata> {
        match self.payload {
            RetainedPayload::Script { metadata, .. } => Some(metadata),
            _ => None,
        }
    }
    pub(crate) fn svg_script(&self) -> Option<&'a SvgScriptState> {
        match self.payload {
            RetainedPayload::SvgScript { state, .. } => Some(state),
            _ => None,
        }
    }
    pub(crate) fn select(&self) -> Option<&'a select::SelectState> {
        match self.payload {
            RetainedPayload::SelectFamily {
                state: SelectMetadata::Select(state),
                ..
            } => Some(state),
            _ => None,
        }
    }
    pub(crate) fn option(&self) -> Option<&'a select::OptionState> {
        match self.payload {
            RetainedPayload::SelectFamily {
                state: SelectMetadata::Option(state),
                ..
            } => Some(state),
            _ => None,
        }
    }
    /// None is an inapplicable metadata record; Some(None) is an actual
    /// option's null cache. A nonnull node was resolved when the record was issued.
    pub fn option_nearest_select(&self) -> Option<Option<HtmlNode<'a>>> {
        match (self.option(), self.refs) {
            (Some(_), Refs::Owner(select)) => Some(select),
            _ => None,
        }
    }
    pub(crate) fn selectedcontent(&self) -> Option<&'a select::SelectedcontentState> {
        match self.payload {
            RetainedPayload::SelectFamily {
                state: SelectMetadata::Selectedcontent(state),
                ..
            } => Some(state),
            _ => None,
        }
    }
}

impl MetadataTemplate<'_> {
    pub fn contents(&self) -> HtmlNode<'_> {
        self.contents
    }
    pub fn target(&self) -> Option<HtmlNode<'_>> {
        self.target
    }
    pub fn start(&self) -> Option<HtmlNode<'_>> {
        self.start
    }
    pub fn end(&self) -> Option<HtmlNode<'_>> {
        self.end
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        HtmlElement, HtmlParseCancellation, HtmlParseLimits, HtmlParseResource, Namespace,
        OrderedAttributes, QualifiedName,
    };

    fn native_select(limits: HtmlParseLimits, local: &str) -> (ParseControl, FrozenArena, usize) {
        let mut c = ParseControl::new(limits, HtmlParseCancellation::new());
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let node = session
            .allocate_element(
                &mut c,
                HtmlElement::new(
                    QualifiedName::new(Namespace::html(), None, local),
                    OrderedAttributes::default(),
                ),
            )
            .unwrap();
        let index = node.index;
        drop(node);
        let frozen = session.freeze(&mut c).unwrap();
        (c, frozen, index)
    }

    fn registry_row() -> ParseStop {
        ParseStop::InternalInvariant(crate::parser::control::Invariant::RegistryRow)
    }
    fn published() -> crate::HtmlParseReport {
        crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<!doctype html><p>a"),
            super::super::tests::limits(),
        )
        .parse()
        .unwrap()
    }

    #[test]
    fn releasing_the_execution_bank_past_the_registry_reports_the_registry_row() {
        let mut report = published();
        let mut c = ParseControl::new(super::super::tests::limits(), HtmlParseCancellation::new());
        report.storage_mut().associated_slot = Some(usize::MAX / 2);
        assert_eq!(
            report.storage_mut().release_execution_inputs(&mut c),
            Err(registry_row())
        );
        assert_eq!(c.stop(), Some(registry_row()));
    }

    #[test]
    fn releasing_the_execution_bank_from_another_row_kind_keeps_that_row() {
        let mut report = published();
        let mut c = ParseControl::new(super::super::tests::limits(), HtmlParseCancellation::new());
        let row = report
            .storage_mut()
            .history_mut()
            .storage_mut()
            .payloads
            .values
            .iter()
            .position(|row| !matches!(row, RetainedPayload::Retired))
            .unwrap();
        let before = |report: &mut crate::HtmlParseReport| {
            mem::discriminant(
                &report
                    .storage_mut()
                    .history_mut()
                    .storage_mut()
                    .payloads
                    .values[row],
            )
        };
        let kept = before(&mut report);
        report.storage_mut().associated_slot = Some(row);
        assert_eq!(
            report.storage_mut().release_execution_inputs(&mut c),
            Err(registry_row())
        );
        assert_eq!(c.stop(), Some(registry_row()));
        assert_eq!(before(&mut report), kept);
    }

    #[test]
    fn cached_select_reference_qualification_has_finite_exact_cost_and_sticky_cleanup() {
        let limits = super::super::tests::limits();
        let (mut setup, mut frozen, node) = native_select(limits, "select");
        let base = setup
            .usage()
            .resource(HtmlParseResource::WorkCredits)
            .current;
        // Bounds visit + native-kind visit + two ASCII length visits =4.
        // URI28 and select6 actual units each admit three inspected bytes =102.
        // No ancestry or registry lookup participates in this fixed qualification.
        let cost = 4 + 3 * (28 + 6);
        assert_eq!(
            checked_select(arena(frozen.storage.as_ref().unwrap()), node, &mut setup),
            Ok(())
        );
        assert_eq!(
            setup
                .usage()
                .resource(HtmlParseResource::WorkCredits)
                .current,
            base + cost
        );
        frozen.release(&mut setup).unwrap();
        super::super::tests::storage_zero(&setup);
        for extra in [cost - 1, cost] {
            let selected = limits
                .into_builder()
                .work_credits(base + extra)
                .finish()
                .unwrap();
            let (mut c, mut frozen, node) = native_select(selected, "select");
            let result = checked_select(arena(frozen.storage.as_ref().unwrap()), node, &mut c);
            if extra == cost {
                result.unwrap();
            } else {
                let first = c.stop().unwrap();
                assert!(matches!(first, ParseStop::LimitExceeded {
                    resource: HtmlParseResource::WorkCredits, current, requested_delta: 3, ..
                } if current == base + cost - 3));
                assert_eq!(result, Err(SessionError::Stop(first)));
                assert_eq!(
                    checked_select(arena(frozen.storage.as_ref().unwrap()), node, &mut c),
                    Err(SessionError::Stop(first))
                );
                assert!(frozen.fragment().is_some());
            }
            if extra == cost {
                // Exact qualifier budget is exhausted; latch a real stop so
                // disposal of the admitted nonce backing uses terminal cleanup.
                c.charge(WorkCategory::StateStep, 1).unwrap_err();
            }
            let stop = c.stop();
            frozen.release(&mut c).unwrap();
            assert_eq!(c.stop(), stop);
            super::super::tests::storage_zero(&c);
        }
        let (mut c, mut frozen, node) = native_select(limits, "p");
        assert_eq!(
            checked_select(arena(frozen.storage.as_ref().unwrap()), node, &mut c),
            Err(SessionError::SelectMetadataUnavailable)
        );
        assert!(c.stop().is_none());
        assert_eq!(
            checked_select(arena(frozen.storage.as_ref().unwrap()), usize::MAX, &mut c),
            Err(SessionError::ForeignNode)
        );
        assert!(c.stop().is_none());
        assert!(frozen.fragment().is_some());
        frozen.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }
}
