//! Qualified immutable publication storage over the canonical frozen owner.
//! Metadata inventory examines one real slot; no lookup hides a registry scan.

use super::*;
use history::{FrozenHistory, PreparedHistory};

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
impl<'a> PreparedReportStorage<'a> {
    pub(crate) fn prepare(
        input: &'a mut Option<FrozenArena>,
        c: &mut ParseControl,
        base_facts: Option<&crate::HtmlParseBaseFacts>,
        closed_common: bool,
    ) -> Result<Self, SessionError> {
        let frozen = input.as_ref().ok_or(SessionError::Consumed)?;
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
                    associated_usage.checked_sum(state.diagnostic_usage(), c)?;
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
                    for p in &storage.payloads.values {
                        c.charge(WorkCategory::RecordVisit, 1)?;
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
                    {
                        return Err(SessionError::WrongToken);
                    }
                    if let Some(first) = first {
                        checked_node(arena, *first, c)?;
                    }
                }
                RetainedPayload::Base { node, frozen, .. } => {
                    let node = node.ok_or(SessionError::WrongToken)?;
                    if !base::is_base(arena.node_at(node).expect("qualified metadata node")) {
                        return Err(SessionError::WrongToken);
                    }
                    if let base::BaseValue::Unavailable { missing, .. } = frozen {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if !matches!(
                            storage.payloads.values.get(*missing),
                            Some(RetainedPayload::MetaMissing {
                                kind: meta::MetaMissingKind::Csp,
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
                    checked_node(arena, node, c)?;
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let HtmlNodeKind::Element(element) =
                        arena.node_at(node).expect("qualified metadata node").kind()
                    else {
                        return Err(SessionError::WrongToken);
                    };
                    if element.link_flags().is_none() {
                        return Err(SessionError::WrongToken);
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
                    checked_node(arena, node, c)?;
                    c.charge(
                        WorkCategory::InspectByte,
                        surgeist_html::bounded::HTML_NONCE_INSPECTION_BOUND as u64,
                    )?;
                    let HtmlNodeKind::Element(element) =
                        arena.node_at(node).expect("qualified metadata node").kind()
                    else {
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
                            _ => 0,
                        };
                        if family != 0 {
                            let mut count = 0usize;
                            for candidate in &storage.payloads.values {
                                c.charge(WorkCategory::RecordVisit, 1)?;
                                if matches!((family, candidate),
                                    (1, RetainedPayload::Hyperlink { node, .. })
                                    | (2, RetainedPayload::Image {node, ..})
                                    | (3, RetainedPayload::Style {node, ..}) if *node == subject)
                                {
                                    count += 1;
                                }
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
                    node, state, meta, ..
                } => {
                    let document = node.ok_or(SessionError::WrongToken)?;
                    if let template::DocumentState::Ordinary { inert: Some(inert) } = state {
                        checked_node(arena, *inert, c)?;
                    }
                    if let Some(raw) = meta.language {
                        checked_raw(storage, raw, c)?;
                    }
                    let mut earliest = None;
                    for domain in meta::MetaDomain::ALL {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if let Some(gap) = meta.gaps[domain.index()] {
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
    pub(crate) fn commit(self) -> ReportStorage {
        ReportStorage {
            history: self.history.commit(),
            closed_common: self.closed_common,
            associated_slot: self.associated_slot,
        }
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
fn checked_node(
    arena: &HtmlConstructionResult,
    node: usize,
    c: &mut ParseControl,
) -> Result<(), SessionError> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    if node >= arena.len() {
        return Err(SessionError::ForeignNode);
    }
    Ok(())
}
fn checked_select(
    arena: &HtmlConstructionResult,
    node: usize,
    c: &mut ParseControl,
) -> Result<(), SessionError> {
    checked_node(arena, node, c)?;
    c.charge(WorkCategory::RecordVisit, 1)?;
    let HtmlNodeKind::Element(element) =
        arena.node_at(node).expect("qualified metadata node").kind()
    else {
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
        | RetainedPayload::Style { node, .. } => Some(*node),
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
    pub(crate) fn release_execution_inputs(&mut self, c: &mut ParseControl) {
        if let Some(slot) = self.associated_slot.take() {
            let storage = self
                .history
                .arena_mut()
                .storage
                .as_mut()
                .expect("qualified report");
            let payload =
                mem::replace(&mut storage.payloads.values[slot], RetainedPayload::Retired);
            let RetainedPayload::AssociatedInputs { usage, .. } = &payload else {
                unreachable!("qualified execution bank");
            };
            let resources = usage.resources();
            drop(payload);
            c.commit_prepared_release(&resources);
        }
    }
    pub(crate) fn fragment_parser_document<'a>(
        &'a self,
        source: &'a crate::HtmlSource,
    ) -> Option<fragment_document::FragmentParserDocument<'a>> {
        fragment_document::FragmentParserDocument::new(&self.history, source)
    }
    pub(crate) fn history(&self) -> &FrozenHistory {
        &self.history
    }
    pub(crate) fn evidence(&self) -> Result<&ParseEvidence, SessionError> {
        self.history.arena().retained_evidence()
    }
    pub(crate) fn effects(&self) -> Result<context::FrozenFragmentEffects<'_>, SessionError> {
        self.history.arena().frozen_fragment_effects()
    }
    pub(crate) fn metadata(&self) -> MetadataCursor<'_> {
        let storage = self
            .history
            .arena()
            .storage
            .as_ref()
            .expect("qualified report storage");
        MetadataCursor {
            history: &self.history,
            closed_common: self.closed_common,
            storage,
            arena: arena(storage),
            next: 0,
        }
    }
}
pub struct MetadataCursor<'a> {
    closed_common: bool,
    history: &'a FrozenHistory,
    storage: &'a FrozenStorage,
    arena: &'a HtmlConstructionResult,
    next: usize,
}
pub enum MetadataStep<'a> {
    Record(MetadataRef<'a>),
    Skipped,
    Complete,
}
impl<'a> MetadataCursor<'a> {
    /// One actual canonical slot, never a next-match loop or materialization.
    pub fn advance(&mut self) -> MetadataStep<'a> {
        let Some(payload) = self.storage.payloads.values.get(self.next) else {
            return MetadataStep::Complete;
        };
        self.next += 1;
        if metadata_node(payload).is_some() {
            MetadataStep::Record(MetadataRef {
                history: self.history,
                closed_common: self.closed_common,
                storage: self.storage,
                arena: self.arena,
                payload,
            })
        } else {
            MetadataStep::Skipped
        }
    }
}
/// Issued only by the qualified report's fixed slot observation. No caller can
/// supply a raw payload index or mix this lender with another result's arena.
pub struct MetadataRef<'a> {
    closed_common: bool,
    history: &'a FrozenHistory,
    storage: &'a FrozenStorage,
    arena: &'a HtmlConstructionResult,
    payload: &'a RetainedPayload,
}
pub enum MetadataDocument<'a> {
    Owned(HtmlNode<'a>),
    Captured(crate::HtmlCapturedNode<'a>),
}
pub struct MetadataTemplate<'a> {
    pub(crate) contents: HtmlNode<'a>,
    pub(crate) target: Option<HtmlNode<'a>>,
    pub(crate) start: Option<HtmlNode<'a>>,
    pub(crate) end: Option<HtmlNode<'a>>,
}
/// Qualified current Base state; old snapshots are historical, never defaults.
pub enum MetadataBaseState<'a> {
    Unfrozen,
    Frozen(MetadataBase<'a>),
    Unavailable {
        missing: MetadataMetaMissing<'a>,
        previous: Option<MetadataBase<'a>>,
    },
}
/// A completed intrinsic/external domain or its actual first missing operation.
pub enum MetadataMetaDisposition<'a> {
    Complete,
    Unavailable(MetadataMetaMissing<'a>),
}
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
pub struct MetadataMetaMissing<'a> {
    history: &'a FrozenHistory,
    storage: &'a FrozenStorage,
    arena: &'a HtmlConstructionResult,
    slot: usize,
}
impl<'a> MetadataMetaMissing<'a> {
    fn row(
        &self,
    ) -> (
        &'a usize,
        &'a usize,
        &'a meta::MetaMissingKind,
        &'a Option<provenance::ConstructionCauseRef>,
    ) {
        let RetainedPayload::MetaMissing {
            document,
            subject,
            kind,
            cause,
        } = &self.storage.payloads.values[self.slot]
        else {
            unreachable!("qualified missing row")
        };
        (document, subject, kind, cause)
    }
    pub fn document(&self) -> HtmlNode<'a> {
        self.arena
            .node_at(*self.row().0)
            .expect("qualified Document")
            .as_node()
    }
    pub fn subject(&self) -> HtmlNode<'a> {
        self.arena
            .node_at(*self.row().1)
            .expect("qualified Meta subject")
            .as_node()
    }
    pub fn kind(&self) -> &'a meta::MetaMissingKind {
        self.row().2
    }
    pub fn cause(&self) -> Option<read::HtmlParseCause<'a>> {
        self.row().3.as_ref().map(|cause| {
            read::HtmlParseCause::qualified(
                self.history,
                self.storage
                    .evidence
                    .as_ref()
                    .and_then(|e| e.retained_cause(cause))
                    .expect("qualified Meta cause"),
            )
        })
    }
}
pub struct MetadataDocumentMeta<'a> {
    history: &'a FrozenHistory,
    storage: &'a FrozenStorage,
    arena: &'a HtmlConstructionResult,
    document: usize,
    meta: &'a meta::MetaHistory,
}
impl<'a> MetadataDocumentMeta<'a> {
    pub fn document(&self) -> HtmlNode<'a> {
        self.arena
            .node_at(self.document)
            .expect("qualified Document")
            .as_node()
    }
    pub fn profile(&self) -> MetadataMetaProfile {
        MetadataMetaProfile::OfflineDoNothingNoUi
    }
    /// Known absent and Known tag: both are original-controller current values.
    pub fn pragma_language(&self) -> Option<&'a DocumentString> {
        self.meta.language.map(|slot| {
            let RetainedPayload::String { value, .. } = &self.storage.payloads.values[slot] else {
                unreachable!("qualified current language")
            };
            value
        })
    }
    pub fn referrer_policy(&self) -> HtmlReferrerPolicy {
        self.meta.referrer
    }
    fn missing(&self, slot: usize) -> MetadataMetaMissing<'a> {
        MetadataMetaMissing {
            history: self.history,
            storage: self.storage,
            arena: self.arena,
            slot,
        }
    }
    pub fn refresh(&self) -> MetadataMetaRefresh<'a> {
        match self.meta.gaps[meta::MetaDomain::Refresh.index()] {
            Some(slot) => MetadataMetaRefresh::Unavailable(self.missing(slot)),
            None => MetadataMetaRefresh::Known(self.meta.refresh),
        }
    }
    pub fn disposition(&self, domain: meta::MetaDomain) -> MetadataMetaDisposition<'a> {
        match self.meta.gaps[domain.index()] {
            Some(slot) => MetadataMetaDisposition::Unavailable(self.missing(slot)),
            None => MetadataMetaDisposition::Complete,
        }
    }
    pub fn first_missing(&self) -> Option<MetadataMetaMissing<'a>> {
        self.meta.first.map(|domain| {
            self.missing(self.meta.gaps[domain.index()].expect("qualified first gap"))
        })
    }
}
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
pub enum MetadataHyperlinkState<'a> {
    AbsentHref,
    Observed(&'a surgeist_html_semantics::HtmlHyperlinkUrlSnapshot),
    Unavailable(&'a associated::HyperlinkUnavailable),
}
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
pub struct MetadataImage<'a> {
    history: &'a FrozenHistory,
    storage: &'a FrozenStorage,
    arena: &'a HtmlConstructionResult,
    node: usize,
    state: &'a associated::ImageMetadata,
}
impl<'a> MetadataImage<'a> {
    pub fn state(&self) -> &'a surgeist_html::HtmlImgFreshState {
        &self.state.fresh
    }
    pub fn dimension_attribute_source(&self) -> HtmlNode<'a> {
        self.arena
            .node_at(self.state.dimension_source)
            .expect("qualified Img source")
            .as_node()
    }
    pub fn latest_started(&self) -> u64 {
        self.state.latest
    }
    pub fn cause(&self) -> Option<read::HtmlParseCause<'a>> {
        self.state.cause.as_ref().map(|cause| {
            read::HtmlParseCause::qualified(
                self.history,
                self.storage
                    .evidence
                    .as_ref()
                    .and_then(|e| e.retained_cause(cause))
                    .expect("qualified Img cause"),
            )
        })
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
        self.arena
            .node_at(self.node)
            .expect("qualified Img")
            .as_node()
    }
}
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
    /// Exact creation subject in a completed ordinary offline or contextual
    /// no-execution/no-interaction profile with actual Null lookup registry.
    /// The source scope is retained at actual creation;
    /// report preparation validates it through HTML's one bounded classifier.
    /// This is current closed-state qualification, not native-fresh evidence.
    pub fn common_state(&self) -> Option<surgeist_html::HtmlCommonState> {
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
        Some(surgeist_html::HtmlCommonState::closed(state.common_scope))
    }

    pub fn style(&self) -> Option<MetadataStyle<'a>> {
        let RetainedPayload::Style { state, .. } = self.payload else {
            return None;
        };
        Some(MetadataStyle {
            state: state.state,
            parser_document: state
                .parser_document
                .as_ref()
                .map(|document| self.document_ref(document)),
            cause: state.cause.as_ref().map(|cause| {
                read::HtmlParseCause::qualified(
                    self.history,
                    self.storage
                        .evidence
                        .as_ref()
                        .and_then(|e| e.retained_cause(cause))
                        .expect("qualified Style cause"),
                )
            }),
        })
    }
    pub fn hyperlink(&self) -> Option<MetadataHyperlink<'a>> {
        let RetainedPayload::Hyperlink { state, .. } = self.payload else {
            return None;
        };
        Some(MetadataHyperlink {
            document: self.document_ref(&state.document),
            cause: state.cause.as_ref().map(|cause| {
                read::HtmlParseCause::qualified(
                    self.history,
                    self.storage
                        .evidence
                        .as_ref()
                        .and_then(|e| e.retained_cause(cause))
                        .expect("qualified URL cause"),
                )
            }),
            raw: state.raw.map(|i| self.raw(i)),
            state: match &state.value {
                associated::HyperlinkValue::Absent => MetadataHyperlinkState::AbsentHref,
                associated::HyperlinkValue::Observed(value) => {
                    MetadataHyperlinkState::Observed(value)
                }
                associated::HyperlinkValue::Unavailable(error) => {
                    MetadataHyperlinkState::Unavailable(error)
                }
            },
        })
    }
    pub fn image(&self) -> Option<MetadataImage<'a>> {
        let RetainedPayload::Image { node, state } = self.payload else {
            return None;
        };
        Some(MetadataImage {
            history: self.history,
            storage: self.storage,
            arena: self.arena,
            node: *node,
            state,
        })
    }

    /// Wrong row is None. Unfrozen and unavailable current state are distinct;
    /// a genuine prior frozen value remains historical inside Unavailable.
    pub fn base_state(&self) -> Option<MetadataBaseState<'a>> {
        let RetainedPayload::Base { frozen, .. } = self.payload else {
            return None;
        };
        Some(match frozen {
            base::BaseValue::Unfrozen => MetadataBaseState::Unfrozen,
            base::BaseValue::Frozen(value) => MetadataBaseState::Frozen(self.base_view(value)),
            base::BaseValue::Unavailable { missing, previous } => MetadataBaseState::Unavailable {
                missing: self.missing_view(*missing),
                previous: previous.as_ref().map(|value| self.base_view(value)),
            },
        })
    }
    fn base_view(&self, value: &'a base::FrozenBase) -> MetadataBase<'a> {
        MetadataBase {
            value: &value.value,
            document: self.native(value.document),
            revision: value.environment.document().revision(),
            responded: value.response == base::Response::Finished,
        }
    }
    fn missing_view(&self, slot: usize) -> MetadataMetaMissing<'a> {
        MetadataMetaMissing {
            history: self.history,
            storage: self.storage,
            arena: self.arena,
            slot,
        }
    }
    /// Actual per-Document history. No getter traverses the tree or replays Meta.
    pub fn document_meta(&self) -> Option<MetadataDocumentMeta<'a>> {
        let RetainedPayload::Document {
            node: Some(document),
            meta,
            ..
        } = self.payload
        else {
            return None;
        };
        Some(MetadataDocumentMeta {
            history: self.history,
            storage: self.storage,
            arena: self.arena,
            document: *document,
            meta,
        })
    }
    pub fn document_base_facts(&self) -> Option<&'a crate::HtmlParseBaseFacts> {
        match self.payload {
            RetainedPayload::BaseDocument { facts, .. } => Some(facts),
            _ => None,
        }
    }
    pub fn document_first_base(&self) -> Option<Option<HtmlNode<'a>>> {
        match self.payload {
            RetainedPayload::BaseDocument { first, .. } => {
                Some(first.map(|node| self.native(node)))
            }
            _ => None,
        }
    }
    /// Actual link flags and creating-parser document. The current node document
    /// is exposed by its separate canonical NodeDocument metadata record.
    pub fn link_parser_document(&self) -> Option<Option<MetadataDocument<'a>>> {
        match self.payload {
            RetainedPayload::Link {
                parser_document, ..
            } => Some(parser_document.as_ref().map(|d| self.document_ref(d))),
            _ => None,
        }
    }
    pub fn link_flags(&self) -> Option<crate::HtmlLinkFlags> {
        if !matches!(self.payload, RetainedPayload::Link { .. }) {
            return None;
        }
        match self.node().kind() {
            HtmlNodeKind::Element(e) => e.link_flags(),
            _ => None,
        }
    }
    /// Ordinary borrowed readback of actual flags and the current node-document
    /// join. It scans retained metadata without allocating; it has no separate
    /// aggregate work/cancellation certificate.
    pub fn link_history(&self) -> Option<crate::HtmlLinkHistory> {
        let RetainedPayload::Link {
            node: Some(node),
            parser_document,
        } = self.payload
        else {
            return None;
        };
        let current = self.storage.payloads.values.iter().find_map(|p| match p {
            RetainedPayload::NodeDocument {
                node: Some(owner),
                document,
            } if owner == node => Some(document),
            _ => None,
        })?;
        Some(crate::HtmlLinkHistory::new(
            self.link_flags()?,
            parser_document.as_ref() == Some(current),
        ))
    }
    pub fn node(&self) -> HtmlNode<'a> {
        self.native(metadata_node(self.payload).expect("issued metadata node"))
    }
    pub(super) fn nonce(&self) -> Option<&'a DocumentString> {
        match self.payload {
            RetainedPayload::Nonce { raw, .. } => Some(self.raw(*raw)),
            _ => None,
        }
    }
    pub(super) fn raw(&self, slot: usize) -> &'a DocumentString {
        let RetainedPayload::String { value, .. } = &self.storage.payloads.values[slot] else {
            unreachable!("qualified raw field");
        };
        value
    }
    fn native(&self, index: usize) -> HtmlNode<'a> {
        self.arena
            .node_at(index)
            .expect("qualified metadata node")
            .as_node()
    }
    fn document_ref(&self, document: &DocumentRef) -> MetadataDocument<'a> {
        match document {
            DocumentRef::Owned(index) => MetadataDocument::Owned(self.native(*index)),
            DocumentRef::Captured(index) => MetadataDocument::Captured(
                self.storage
                    .fragment
                    .as_ref()
                    .expect("qualified captured document")
                    .context
                    .capture()
                    .node(index.subject())
                    .expect("qualified captured document"),
            ),
        }
    }
    pub fn node_document(&self) -> Option<MetadataDocument<'a>> {
        match self.payload {
            RetainedPayload::NodeDocument { document, .. } => Some(self.document_ref(document)),
            _ => None,
        }
    }
    pub fn script_documents(
        &self,
    ) -> Option<(Option<MetadataDocument<'a>>, Option<MetadataDocument<'a>>)> {
        match self.payload {
            RetainedPayload::Script { metadata, .. } => Some((
                metadata
                    .parser_document
                    .as_ref()
                    .map(|d| self.document_ref(d)),
                metadata
                    .preparation_document
                    .as_ref()
                    .map(|d| self.document_ref(d)),
            )),
            _ => None,
        }
    }
    pub fn template(&self) -> Option<MetadataTemplate<'a>> {
        match self.payload {
            RetainedPayload::Template { state, .. } => Some(MetadataTemplate {
                contents: self.native(state.contents),
                target: state.target.map(|i| self.native(i)),
                start: state.start.map(|i| self.native(i)),
                end: state.end.map(|i| self.native(i)),
            }),
            _ => None,
        }
    }
    pub fn creation(&self) -> Option<(crate::HtmlCustomElementState, Option<&'a DocumentString>)> {
        match self.payload {
            RetainedPayload::LookupRegistry {
                creation: Some(state),
                ..
            } => Some((state.state, state.saved_is.map(|i| self.raw(i)))),
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
        match self.payload {
            RetainedPayload::Document {
                state: template::DocumentState::Ordinary { inert },
                class,
                ..
            } => Some((*class, false, inert.map(|index| self.native(index)))),
            RetainedPayload::Document {
                state: template::DocumentState::InertTemplate,
                class,
                ..
            } => Some((*class, true, None)),
            _ => None,
        }
    }
    pub(crate) fn input(&self) -> Option<(&'a InputState, &'a DocumentString)> {
        match self.payload {
            RetainedPayload::Input { state, raw, .. } => Some((state, self.raw(*raw))),
            _ => None,
        }
    }
    pub(crate) fn textarea(&self) -> Option<(&'a TextareaState, &'a DocumentString)> {
        match self.payload {
            RetainedPayload::Textarea { state, .. } => Some((state, self.raw(state.raw))),
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
        match self.payload {
            RetainedPayload::FormAssociation { state, .. } => {
                Some((state, state.owner.map(|index| self.native(index))))
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
    /// option's null cache. A nonnull node was qualified before publication.
    pub fn option_nearest_select(&self) -> Option<Option<HtmlNode<'a>>> {
        self.option().map(|state| {
            state
                .cached_nearest_select()
                .map(|index| self.native(index))
        })
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
