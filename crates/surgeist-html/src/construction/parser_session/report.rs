//! Qualified immutable publication storage over the canonical frozen owner.
//! Metadata inventory examines one real slot; no lookup hides a registry scan.

use super::*;
use history::{FrozenHistory, PreparedHistory};

pub(crate) struct ReportStorage {
    history: FrozenHistory,
}
pub(crate) struct PreparedReportStorage<'a> {
    history: PreparedHistory<'a>,
}
impl<'a> PreparedReportStorage<'a> {
    pub(crate) fn prepare(
        input: &'a mut Option<FrozenArena>,
        c: &mut ParseControl,
    ) -> Result<Self, SessionError> {
        let frozen = input.as_ref().ok_or(SessionError::Consumed)?;
        let storage = frozen.storage.as_ref().ok_or(SessionError::Consumed)?;
        c.require_identity(&storage.control)?;
        let arena = arena(storage);
        for payload in &storage.payloads.values {
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
                    if state.flags.kind() == crate::HtmlInputKind::File
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
                    if state.flags.kind() == crate::HtmlInputKind::Image
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
                RetainedPayload::Textarea { state, .. } => {
                    checked_raw(storage, state.raw, c)?;
                }
                RetainedPayload::FormAssociation { state, .. } => {
                    if let Some(owner) = state.owner {
                        checked_node(arena, owner, c)?;
                    }
                }
                RetainedPayload::LookupRegistry {
                    creation: Some(state),
                    ..
                } => {
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
                    checked_document(storage, *document, c)?
                }
                RetainedPayload::Script { metadata, .. } => {
                    for document in [metadata.parser_document, metadata.preparation_document]
                        .into_iter()
                        .flatten()
                    {
                        checked_document(storage, document, c)?;
                    }
                }
                RetainedPayload::Document {
                    state: template::DocumentState::Ordinary { inert: Some(inert) },
                    ..
                } => checked_node(arena, *inert, c)?,
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
            frozen.prepare_fragment_effects(c)?;
        }
        payload_write::<Self>(c, 1)?;
        payload_write::<ReportStorage>(c, 1)?;
        Ok(Self {
            history: PreparedHistory::prepare(input, c)?,
        })
    }
    pub(crate) fn commit(self) -> ReportStorage {
        ReportStorage {
            history: self.history.commit(),
        }
    }
}
fn checked_document(
    storage: &FrozenStorage,
    document: DocumentRef,
    c: &mut ParseControl,
) -> Result<(), SessionError> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    match document {
        DocumentRef::Owned(index) => {
            if !matches!(
                arena(storage).records.get(index).map(|record| &record.kind),
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
                .document_at(index)
                .map_err(|_| SessionError::NodeDocumentUnavailable)?;
        }
    }
    Ok(())
}
fn arena(storage: &FrozenStorage) -> &Arena {
    match &storage.view {
        FrozenView::Document(view) => &view.arena,
        FrozenView::Fragment(view) => &view.arena,
    }
}
fn checked_node(arena: &Arena, node: usize, c: &mut ParseControl) -> Result<(), SessionError> {
    c.charge(WorkCategory::RecordVisit, 1)?;
    if node >= arena.records.len() {
        return Err(SessionError::ForeignNode);
    }
    Ok(())
}
fn checked_select(arena: &Arena, node: usize, c: &mut ParseControl) -> Result<(), SessionError> {
    checked_node(arena, node, c)?;
    c.charge(WorkCategory::RecordVisit, 1)?;
    let HtmlNodeKind::Element(element) = &arena.records[node].kind else {
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
        RetainedPayload::StandaloneDocumentCompletion { node, .. } => Some(*node),
        RetainedPayload::LookupRegistry { node, .. }
        | RetainedPayload::PiAttributes { node, .. }
        | RetainedPayload::NodeDocument { node, .. }
        | RetainedPayload::Document { node, .. }
        | RetainedPayload::Template { node, .. }
        | RetainedPayload::SvgScript { node, .. }
        | RetainedPayload::Script { node, .. }
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
            storage,
            arena: arena(storage),
            next: 0,
        }
    }
}
pub struct MetadataCursor<'a> {
    storage: &'a FrozenStorage,
    arena: &'a Arena,
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
    storage: &'a FrozenStorage,
    arena: &'a Arena,
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
impl<'a> MetadataRef<'a> {
    pub fn node(&self) -> HtmlNode<'a> {
        HtmlNode {
            arena: self.arena,
            index: metadata_node(self.payload).expect("issued metadata node"),
        }
    }
    pub(super) fn raw(&self, slot: usize) -> &'a DocumentString {
        let RetainedPayload::String { value, .. } = &self.storage.payloads.values[slot] else {
            unreachable!("qualified raw field");
        };
        value
    }
    fn native(&self, index: usize) -> HtmlNode<'a> {
        HtmlNode {
            arena: self.arena,
            index,
        }
    }
    fn document_ref(&self, document: DocumentRef) -> MetadataDocument<'a> {
        match document {
            DocumentRef::Owned(index) => MetadataDocument::Owned(self.native(index)),
            DocumentRef::Captured(index) => MetadataDocument::Captured(
                self.storage
                    .fragment
                    .as_ref()
                    .expect("qualified captured document")
                    .context
                    .capture()
                    .document_at(index)
                    .expect("qualified captured document"),
            ),
        }
    }
    pub fn node_document(&self) -> Option<MetadataDocument<'a>> {
        match self.payload {
            RetainedPayload::NodeDocument { document, .. } => Some(self.document_ref(*document)),
            _ => None,
        }
    }
    pub fn script_documents(
        &self,
    ) -> Option<(Option<MetadataDocument<'a>>, Option<MetadataDocument<'a>>)> {
        match self.payload {
            RetainedPayload::Script { metadata, .. } => Some((
                metadata.parser_document.map(|d| self.document_ref(d)),
                metadata.preparation_document.map(|d| self.document_ref(d)),
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
            RetainedPayload::FormAssociation { state, .. } => Some((
                state,
                state.owner.map(|index| HtmlNode {
                    arena: self.arena,
                    index,
                }),
            )),
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
        HtmlNode {
            arena: self.contents.arena,
            index: self.contents.index,
        }
    }
    pub fn target(&self) -> Option<HtmlNode<'_>> {
        self.target.as_ref().map(|n| HtmlNode {
            arena: n.arena,
            index: n.index,
        })
    }
    pub fn start(&self) -> Option<HtmlNode<'_>> {
        self.start.as_ref().map(|n| HtmlNode {
            arena: n.arena,
            index: n.index,
        })
    }
    pub fn end(&self) -> Option<HtmlNode<'_>> {
        self.end.as_ref().map(|n| HtmlNode {
            arena: n.arena,
            index: n.index,
        })
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
                HtmlElement {
                    name: QualifiedName::new(Namespace::html(), None, local),
                    attributes: OrderedAttributes::default(),
                },
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
