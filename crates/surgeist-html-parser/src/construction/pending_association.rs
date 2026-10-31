//! Bounded original occurrences and immutable inputs; no CSSOM execution.
use super::*;
use history::{FrozenHistory, HistoryGroup};

/// An original occurrence, qualified by its immutable construction result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OccurrenceId {
    result: HtmlNodeId,
    slot: usize,
}
/// An original context frontier. This is not resource-completion order.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextEpochId {
    result: HtmlNodeId,
    slot: usize,
}
/// A deliberately unbound input at one exact original context frontier.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextSlotId {
    epoch: ContextEpochId,
    role: ContextRole,
}
impl ContextSlotId {
    pub fn epoch(&self) -> &ContextEpochId {
        &self.epoch
    }
    pub fn role(&self) -> ContextRole {
        self.role
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRole {
    DocumentUrl,
    FallbackBase,
    Encoding,
    InitialPolicy,
    ExternalEnvironment,
    BodyPresence,
    DocumentEligibility,
}
/// Pending external input is distinct from lost original evidence.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContextRequirement {
    Known,
    OfflineUnbound,
    DeferredCaptured,
    Unavailable(MissingKind),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MissingKind {
    UnsupportedLinkStylesheetHistory,
    ExtensionLinkHistory,
    OriginalContext,
    PolicyHistory,
}
/// Completeness of capture, never completeness of CSSOM processing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CaptureDisposition {
    PendingComplete,
    Unavailable(MissingKind),
}
/// Closed original-controller operations. None asserts successful sheet addition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OccurrenceKind {
    StyleCreated,
    StyleParserCreated,
    StyleUpdate(surgeist_html::HtmlStyleTrigger),
    StyleOwnerAttributes,
    StyleDocumentChanged,
    ChangePreferred,
    BaseChanged,
    UnsupportedLink,
    UnsupportedLinkExtension,
}

pub(super) struct DocumentCapture {
    document: DocumentRef,
    count: usize,
    obligations: usize,
    last: Option<usize>,
    missing: Option<MissingKind>,
    missing_at: Option<usize>,
}
pub(super) struct CapturedContext {
    value: surgeist_html_semantics::HtmlDocumentEncodingUrlContext,
    usage: associated::AssociatedStorageUsage,
}
pub(super) struct Inputs {
    state: surgeist_html::HtmlStyleState,
    parser_document: Option<DocumentRef>,
    connected: bool,
    type_eligible: bool,
    document_tree: bool,
    attributes: [Option<OriginGroupRef>; 5],
    text: OwnedVec<OriginGroupRef>,
    order: OwnedVec<usize>,
}
pub(super) struct Context {
    input_document: usize,
    mode: HtmlDocumentMode,
    class: crate::HtmlDocumentClass,
    body: bool,
    base_document: Option<usize>,
    first_base: Option<usize>,
    base_input: Option<OriginGroupRef>,
    base_observation: Option<usize>,
    captured: Option<usize>,
    policy_missing: bool,
    csp: Option<usize>,
    missing: bool,
}
pub(super) struct Occurrence {
    subject: usize,
    document: DocumentRef,
    root: usize,
    previous_document: Option<DocumentRef>,
    prior_document: Option<usize>,
    prior_style: Option<usize>,
    cause: Option<provenance::ConstructionCauseRef>,
    kind: OccurrenceKind,
    inputs: Option<Inputs>,
    preferred: Option<OriginGroupRef>,
    base_changed_input: Option<OriginGroupRef>,
    context: Context,
    requires_processing: bool,
    may_have_sheet: bool,
}

/// The subject of a style occurrence row: one that carries style inputs.
#[cfg(test)]
pub(super) fn style_occurrence_subject(row: &RetainedPayload) -> Option<usize> {
    match row {
        RetainedPayload::AssociationOccurrence(values) => {
            let first = &values.as_slice()[0];
            first.inputs.is_some().then_some(first.subject)
        }
        _ => None,
    }
}

impl SessionStorage {
    fn association_document(
        &mut self,
        c: &mut ParseControl,
        document: &DocumentRef,
    ) -> Result<usize, SessionError> {
        for &slot in self.payloads.kind_list(KindList::AssociationDocument) {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if matches!(self.payloads.values.get(slot), Some(RetainedPayload::AssociationDocument(value)) if &value.document == document)
            {
                return Ok(slot);
            }
        }
        let document = document.try_clone(c)?;
        payload_write::<DocumentCapture>(c, 1)?;
        self.prepare_payload_slot_for(c, Some(KindList::AssociationDocument))?;
        let slot = self.payloads.values.len();
        self.payloads
            .push_row(RetainedPayload::AssociationDocument(DocumentCapture {
                document,
                count: 0,
                obligations: 0,
                last: None,
                missing: None,
                missing_at: None,
            }))?;
        Ok(slot)
    }
    /// A reference never adds a strong alias to mutable/reclaimable backing.
    fn association_string(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
    ) -> Result<OriginGroupRef, SessionError> {
        let RetainedPayload::String { value, origins, .. } = &self.payloads.values[slot] else {
            return Err(SessionError::WrongToken);
        };
        if let Some(group) = origins {
            return self
                .evidence
                .as_ref()
                .ok_or(SessionError::WrongToken)?
                .copy_group_reference(c, group);
        }
        let units = value.len();
        let evidence = self.evidence.as_mut().ok_or(SessionError::WrongToken)?;
        let group = evidence.source_independent_group(c, units)?;
        evidence.bind_leaf_value(c, &group, slot)?;
        let retained = evidence.copy_group_reference(c, &group)?;
        payload_write::<Option<OriginGroupRef>>(c, 1)?;
        let RetainedPayload::String { origins, .. } = &mut self.payloads.values[slot] else {
            unreachable!()
        };
        *origins = Some(retained);
        Ok(group)
    }
    fn association_attribute(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        local: &str,
    ) -> Result<Option<OriginGroupRef>, SessionError> {
        let mut position = None;
        let HtmlNodeKind::Element(element) = self
            .construction
            .node_at(node)
            .ok_or(SessionError::ForeignNode)?
            .kind()
        else {
            return Ok(None);
        };
        for (index, attr) in element.attributes().as_slice().iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if attr.name().namespace().uri().is_none()
                && exact_ascii(c, Some(attr.name().local_name()), local)?
            {
                position = Some(index);
                break;
            }
        }
        let Some(position) = position else {
            return Ok(None);
        };
        let container = self.element_attribute_container(c, node)?;
        let slot = self.field_slot(c, StringLocation::AttributeValue(container, position))?;
        self.association_string(c, slot).map(Some)
    }
    fn association_style_inputs(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
    ) -> Result<Option<Inputs>, SessionError> {
        if !self.html_local(c, node, "style")? {
            return Ok(None);
        }
        let mut state = None;
        if let Some(slot) = self.payloads.joined(c, node, Join::Style)? {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let Some(RetainedPayload::Style {
                node: owner,
                state: value,
            }) = self.payloads.values.get(slot)
                && *owner == node
            {
                state = Some((
                    value.state,
                    value
                        .parser_document
                        .as_ref()
                        .map(|d| d.try_clone(c))
                        .transpose()?,
                ));
            }
        }
        let (state, parser_document) = state.ok_or(SessionError::WrongToken)?;
        let connected = self.shadow_including_connected(c, node, detached)?;
        let type_eligible = surgeist_html::style_type_allows_css_with(
            self.form_attribute(c, node, "type")?,
            || c.charge(WorkCategory::RecordVisit, 1),
        )?;
        let root = self.ordinary_root(c, node, detached)?;
        let document_tree = matches!(
            self.construction
                .node_at(root)
                .ok_or(SessionError::WrongToken)?
                .kind(),
            HtmlNodeKind::Document(_)
        );
        let mut text = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let mut order = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
        let result = (|| {
            let mut attributes = [None, None, None, None, None];
            for (index, local) in ["type", "media", "title", "blocking", "nonce"]
                .into_iter()
                .enumerate()
            {
                attributes[index] = self.association_attribute(c, node, local)?;
            }
            // Internal nonce is separate from the potentially hidden authored attribute.
            if let Some(slot) = self.payloads.joined(c, node, Join::Nonce)? {
                c.charge(WorkCategory::RecordVisit, 1)?;
                if let Some(RetainedPayload::Nonce {
                    node: Some(owner),
                    raw,
                }) = self.payloads.values.get(slot)
                    && *owner == node
                {
                    let raw = *raw;
                    attributes[4] = Some(self.association_string(c, raw)?);
                }
            }
            let mut next = self
                .construction
                .node_at(node)
                .ok_or(SessionError::ForeignNode)?
                .child_at(0)
                .map(|n| n.position());
            while let Some(child) = next {
                c.charge(WorkCategory::RecordVisit, 1)?;
                let record = self
                    .construction
                    .node_at(child)
                    .ok_or(SessionError::ForeignNode)?;
                next = self.next_sibling(c, child)?;
                if detached == Some(child) {
                    continue;
                }
                if matches!(record.kind(), HtmlNodeKind::Text(_)) {
                    let slot = self.field_slot(c, StringLocation::Text(child))?;
                    let group = self.association_string(c, slot)?;
                    text.try_push(c, group)?;
                }
            }
            let mut next = Some(root);
            while let Some(candidate) = next {
                if self.html_local(c, candidate, "style")? {
                    order.try_push(c, candidate)?;
                }
                next = self.next_view_node(c, root, candidate, detached)?;
            }
            Ok::<_, SessionError>(attributes)
        })();
        match result {
            Ok(attributes) => Ok(Some(Inputs {
                state,
                parser_document,
                connected,
                type_eligible,
                document_tree,
                attributes,
                text,
                order,
            })),
            Err(error) => {
                text.release(c)?;
                order.release(c)?;
                Err(error)
            }
        }
    }
    fn association_captured_context(
        &mut self,
        c: &mut ParseControl,
        document: &crate::HtmlCaptureDocumentId,
    ) -> Result<Option<usize>, SessionError> {
        for (index, row) in self.payloads.values.iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::AssociationCapturedContext(value) = row
                && value.value.document() == document
            {
                return Ok(Some(index));
            }
        }
        let mut selected = None;
        for row in &self.payloads.values {
            c.charge(WorkCategory::RecordVisit, 1)?;
            if let RetainedPayload::AssociatedInputs { inputs, .. } = row {
                for value in &inputs.captured {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    if value.document() == document {
                        selected = Some(value);
                        break;
                    }
                }
            }
        }
        let Some(value) = selected else {
            return Ok(None);
        };
        let retained = value.retained_storage_usage();
        let mut usage = associated::AssociatedStorageUsage::default();
        let bytes = retained
            .retained_bytes()
            .checked_sub(mem::size_of::<
                surgeist_html_semantics::HtmlDocumentEncodingUrlContext,
            >())
            .ok_or(SessionError::WrongToken)?;
        usage.add(bytes, retained.utf16_extent(), 0, c)?;
        usage.acquire(c)?;
        let value = value.clone();
        if let Err(error) = self.prepare_payload_slot_owned(c) {
            drop(value);
            usage.release(c, &self.control);
            return Err(error);
        }
        let index = self.payloads.values.len();
        self.payloads
            .values
            .push(RetainedPayload::AssociationCapturedContext(
                CapturedContext { value, usage },
            ));
        Ok(Some(index))
    }
    fn association_context(
        &mut self,
        c: &mut ParseControl,
        document: &DocumentRef,
        detached: Option<usize>,
        input_document: usize,
    ) -> Result<Context, SessionError> {
        let mut context = Context {
            input_document,
            mode: HtmlDocumentMode::NoQuirks,
            class: crate::HtmlDocumentClass::Unavailable,
            body: false,
            base_document: None,
            first_base: None,
            base_input: None,
            base_observation: None,
            captured: None,
            policy_missing: false,
            csp: None,
            missing: false,
        };
        match document {
            DocumentRef::Captured(document) => {
                let capture = self
                    .fragment
                    .as_ref()
                    .ok_or(SessionError::ContextUnavailable)?
                    .context
                    .capture();
                let node = capture
                    .node(document.subject())
                    .map_err(|_| SessionError::ContextUnavailable)?;
                let crate::HtmlCapturedNodeKind::Document(facts) = node.kind() else {
                    return Err(SessionError::WrongToken);
                };
                context.mode = facts.mode();
                context.class = facts.document_class();
                context.captured = self.association_captured_context(c, document)?;
            }
            DocumentRef::Owned(document) => {
                let HtmlNodeKind::Document(mode) = self
                    .construction
                    .node_at(*document)
                    .ok_or(SessionError::WrongToken)?
                    .kind()
                else {
                    return Err(SessionError::WrongToken);
                };
                context.mode = *mode;
                let slot = self.document_slot(c, *document)?;
                let RetainedPayload::Document { class, .. } = &self.payloads.values[slot] else {
                    unreachable!()
                };
                context.class = *class;
                // First document Element and only its direct body/frameset child.
                let mut child = self
                    .construction
                    .node_at(*document)
                    .ok_or(SessionError::WrongToken)?
                    .child_at(0)
                    .map(|n| n.position());
                while let Some(index) = child {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    let node = self
                        .construction
                        .node_at(index)
                        .ok_or(SessionError::WrongToken)?;
                    child = self.next_sibling(c, index)?;
                    if detached == Some(index) {
                        continue;
                    }
                    if let HtmlNodeKind::Element(element) = node.kind() {
                        if surgeist_html::document_body_role_with(element.name(), |_| {
                            c.charge(WorkCategory::RecordVisit, 1)
                        })? == surgeist_html::HtmlDocumentBodyRole::Html
                        {
                            let mut body = node.child_at(0).map(|n| n.position());
                            while let Some(index) = body {
                                c.charge(WorkCategory::RecordVisit, 1)?;
                                let node = self
                                    .construction
                                    .node_at(index)
                                    .ok_or(SessionError::WrongToken)?;
                                body = self.next_sibling(c, index)?;
                                if detached == Some(index) {
                                    continue;
                                }
                                if let HtmlNodeKind::Element(element) = node.kind()
                                    && surgeist_html::document_body_role_with(
                                        element.name(),
                                        |_| c.charge(WorkCategory::RecordVisit, 1),
                                    )? == surgeist_html::HtmlDocumentBodyRole::Body
                                {
                                    context.body = true;
                                    break;
                                }
                            }
                        }
                        break;
                    }
                }
                context.policy_missing = self.meta_policy_missing(c, *document)?.is_some();
                if let Some(slot) = self.payloads.joined(c, *document, Join::BaseDocument)? {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    if let Some(RetainedPayload::BaseDocument { node, first, .. }) =
                        self.payloads.values.get(slot)
                        && node == document
                    {
                        context.base_document = Some(slot);
                        context.first_base = *first;
                    }
                }
                if let Some(first) = context.first_base {
                    context.base_input = self.association_attribute(c, first, "href")?;
                    let slot = self.base_slot(c, first)?;
                    let RetainedPayload::Base { frozen, .. } = &self.payloads.values[slot] else {
                        unreachable!()
                    };
                    if let base::BaseValue::Frozen(value) = frozen {
                        let Some(facts_slot) = context.base_document else {
                            return Err(SessionError::WrongToken);
                        };
                        let RetainedPayload::BaseDocument { facts, .. } =
                            &self.payloads.values[facts_slot]
                        else {
                            unreachable!()
                        };
                        let raw = self
                            .form_attribute(c, first, "href")?
                            .ok_or(SessionError::WrongToken)?;
                        let fallback = facts
                            .document()
                            .fallback()
                            .map_err(|_| SessionError::WrongToken)?;
                        let encoding = facts
                            .document()
                            .encoding()
                            .ok_or(SessionError::WrongToken)?;
                        for (index, observation) in facts.observations().iter().enumerate() {
                            c.charge(WorkCategory::RecordVisit, 1)?;
                            if observation.inputs_match(
                                fallback,
                                encoding,
                                facts.document().revision(),
                            ) && exact_string_equal(c, observation.raw(), raw)?
                                && facts.same_owner(&value.environment)
                            {
                                context.base_observation = Some(index);
                                break;
                            }
                        }
                        context.missing = context.base_observation.is_none();
                    } else {
                        context.missing = true;
                    }
                }
            }
        }
        if let DocumentRef::Owned(document) = document {
            self.ensure_csp_policy(c, *document)?;
            context.csp = self.csp_policy_slot(c, *document)?;
        }
        Ok(context)
    }
    pub(super) fn record_association(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        kind: OccurrenceKind,
        detached: Option<usize>,
        previous_document: Option<DocumentRef>,
        preferred: bool,
    ) -> Result<(), SessionError> {
        // Generic native-only sessions cannot issue original-controller evidence.
        if self.evidence.is_none() {
            return Ok(());
        }
        let document = self.node_document_ref(c, node)?;
        let document_slot = self.association_document(c, &document)?;
        let context = self.association_context(c, &document, detached, document_slot)?;
        let preferred = if preferred {
            self.association_attribute(c, node, "content")?
        } else {
            None
        };
        let base_changed_input = if kind == OccurrenceKind::BaseChanged {
            self.association_attribute(c, node, "href")?
        } else {
            None
        };
        let inputs = self.association_style_inputs(c, node, detached)?;
        let result = (|| {
            let root = self.ordinary_root(c, node, detached)?;
            let prior_style = self.payloads.joined(c, node, Join::StyleOccurrence)?;
            let cause = self.associated_cause(c)?;
            let prior_sheet = prior_style.is_some_and(|slot| matches!(self.payloads.values.get(slot), Some(RetainedPayload::AssociationOccurrence(values)) if values.as_slice().first().is_some_and(|first| first.may_have_sheet)));
            let RetainedPayload::AssociationDocument(summary) =
                &self.payloads.values[document_slot]
            else {
                unreachable!()
            };
            let count = summary
                .count
                .checked_add(1)
                .ok_or(SessionError::WrongToken)?;
            let mut obligation = matches!(
                kind,
                OccurrenceKind::ChangePreferred
                    | OccurrenceKind::UnsupportedLink
                    | OccurrenceKind::UnsupportedLinkExtension
            );
            let mut may_have_sheet = prior_sheet;
            if let Some(inputs) = &inputs {
                // Removal is conditional on the prior real association, never parser None.
                obligation |= match kind {
                    OccurrenceKind::StyleUpdate(_) => {
                        may_have_sheet = inputs.connected && inputs.type_eligible;
                        prior_sheet || may_have_sheet
                    }
                    OccurrenceKind::StyleOwnerAttributes => prior_sheet,
                    _ => false,
                };
            }
            let obligations = summary
                .obligations
                .checked_add(usize::from(obligation))
                .ok_or(SessionError::WrongToken)?;
            let prior_document = summary.last;
            let creating = matches!(kind, OccurrenceKind::StyleUpdate(_)) && may_have_sheet;
            let missing = if creating && context.policy_missing {
                Some(MissingKind::PolicyHistory)
            } else if creating && context.missing {
                Some(MissingKind::OriginalContext)
            } else if kind == OccurrenceKind::UnsupportedLink {
                Some(MissingKind::UnsupportedLinkStylesheetHistory)
            } else if kind == OccurrenceKind::UnsupportedLinkExtension {
                Some(MissingKind::ExtensionLinkHistory)
            } else {
                None
            };
            payload_write::<DocumentCapture>(c, 1)?;
            Ok::<_, SessionError>((
                root,
                cause,
                prior_style,
                prior_document,
                count,
                obligations,
                missing,
                obligation,
                may_have_sheet,
            ))
        })();
        match result {
            Ok((
                root,
                cause,
                prior_style,
                prior_document,
                count,
                obligations,
                missing,
                requires_processing,
                may_have_sheet,
            )) => {
                let styled = inputs.is_some();
                let value = Occurrence {
                    subject: node,
                    document,
                    root,
                    previous_document,
                    prior_document,
                    prior_style,
                    cause,
                    kind,
                    inputs,
                    preferred,
                    base_changed_input,
                    context,
                    requires_processing,
                    may_have_sheet,
                };
                let mut values = OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?;
                if let Err((stop, value)) = values.try_push_recover(c, value) {
                    release_inputs(value, c);
                    values.release(c)?;
                    return Err(SessionError::Stop(stop));
                }
                if let Err(error) = self.prepare_payload_slot_owned(c) {
                    release_occurrence(values, c);
                    return Err(error);
                }
                let slot = self.payloads.values.len();
                self.payloads
                    .values
                    .push(RetainedPayload::AssociationOccurrence(values));
                if styled && let Some(cell) = self.payloads.node_joins.get_mut(node) {
                    cell.set_latest(Join::StyleOccurrence, slot);
                }
                let RetainedPayload::AssociationDocument(summary) =
                    &mut self.payloads.values[document_slot]
                else {
                    unreachable!()
                };
                summary.count = count;
                summary.obligations = obligations;
                summary.last = Some(slot);
                if summary.missing.is_none() {
                    summary.missing = missing;
                    summary.missing_at = missing.map(|_| slot);
                }
                Ok(())
            }
            Err(error) => {
                if let Some(inputs) = inputs {
                    inputs.text.release(c)?;
                    inputs.order.release(c)?;
                }
                Err(error)
            }
        }
    }
    pub(super) fn association_style_trigger(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        trigger: surgeist_html::HtmlStyleTrigger,
        detached: Option<usize>,
    ) -> Result<(), SessionError> {
        if self.html_local(c, node, "style")? && surgeist_html::style_update_is_required(trigger) {
            self.record_association(
                c,
                node,
                OccurrenceKind::StyleUpdate(trigger),
                detached,
                None,
                false,
            )?;
        }
        Ok(())
    }
    pub(super) fn association_tree_hook(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        detached: Option<usize>,
        removal: bool,
        old_parent: Option<usize>,
    ) -> Result<(), SessionError> {
        if self.html_local(c, node, "style")? {
            let Some(slot) = self.style_slot(c, node)? else {
                return Ok(());
            };
            let RetainedPayload::Style { state, .. } = &self.payloads.values[slot] else {
                unreachable!()
            };
            let on_parser_stack = state.state.on_parser_stack();
            let connected = self.shadow_including_connected(c, node, detached)?;
            let old_connected = match old_parent {
                Some(parent) => self.shadow_including_connected(c, parent, None)?,
                None => false,
            };
            let trigger = if removal {
                surgeist_html::HtmlStyleTrigger::Removal {
                    disconnected: old_connected && !connected,
                    on_parser_stack,
                }
            } else {
                surgeist_html::HtmlStyleTrigger::Insertion {
                    connected,
                    on_parser_stack,
                }
            };
            self.association_style_trigger(c, node, trigger, detached)?;
        }
        if !removal {
            self.association_link(c, node)?;
        }
        Ok(())
    }
    pub(super) fn association_children(
        &mut self,
        c: &mut ParseControl,
        parent: usize,
        detached: Option<usize>,
    ) -> Result<(), SessionError> {
        let Some(slot) = self.style_slot(c, parent)? else {
            return Ok(());
        };
        let RetainedPayload::Style { state, .. } = &self.payloads.values[slot] else {
            unreachable!()
        };
        self.association_style_trigger(
            c,
            parent,
            surgeist_html::HtmlStyleTrigger::ChildrenChanged {
                on_parser_stack: state.state.on_parser_stack(),
            },
            detached,
        )
    }
    pub(super) fn association_attribute_hook(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        position: usize,
    ) -> Result<(), SessionError> {
        if self.evidence.is_none() {
            return Ok(());
        }
        let HtmlNodeKind::Element(element) = self
            .construction
            .node_at(node)
            .ok_or(SessionError::WrongToken)?
            .kind()
        else {
            return Ok(());
        };
        let attr = element
            .attributes()
            .as_slice()
            .get(position)
            .ok_or(SessionError::WrongToken)?;
        let effect = surgeist_html::KnownHtmlAttribute::from_name(attr.name())
            .map(|name| surgeist_html::style_attribute_effect(attr.name().namespace(), name));
        if effect == Some(surgeist_html::HtmlStyleAttributeEffect::OwnerAttributes)
            && self.html_local(c, node, "style")?
        {
            self.record_association(
                c,
                node,
                OccurrenceKind::StyleOwnerAttributes,
                None,
                None,
                false,
            )?;
        }
        self.association_link(c, node)
    }
    pub(super) fn association_initial_attributes(
        &mut self,
        c: &mut ParseControl,
        node: usize,
    ) -> Result<(), SessionError> {
        if self.evidence.is_none() || !self.html_local(c, node, "style")? {
            return Ok(());
        }
        let HtmlNodeKind::Element(element) = self
            .construction
            .node_at(node)
            .ok_or(SessionError::WrongToken)?
            .kind()
        else {
            return Ok(());
        };
        let count = element.attributes().len();
        for position in 0..count {
            self.association_attribute_hook(c, node, position)?;
        }
        Ok(())
    }
    pub(super) fn prepare_association_documents(
        &mut self,
        c: &mut ParseControl,
    ) -> Result<(), SessionError> {
        if self.evidence.is_none() {
            return Ok(());
        }
        let extent = self.payloads.values.len();
        for index in 0..extent {
            c.charge(WorkCategory::RecordVisit, 1)?;
            let document = match &self.payloads.values[index] {
                RetainedPayload::Document {
                    node: Some(node), ..
                } => Some(DocumentRef::Owned(*node)),
                RetainedPayload::NodeDocument {
                    document: DocumentRef::Captured(id),
                    ..
                } => {
                    c.charge(WorkCategory::ProviderCall, 1)?;
                    Some(DocumentRef::Captured(id.clone()))
                }
                _ => None,
            };
            if let Some(document) = document {
                self.association_document(c, &document)?;
            }
        }
        Ok(())
    }
    fn association_link(&mut self, c: &mut ParseControl, node: usize) -> Result<(), SessionError> {
        if self.evidence.is_none()
            || !self.html_local(c, node, "link")?
            || !self.shadow_including_connected(c, node, None)?
            || self.form_attribute(c, node, "href")?.is_none()
        {
            return Ok(());
        }
        let Some(rel) = self.form_attribute(c, node, "rel")? else {
            return Ok(());
        };
        let mut cursor = bounded::HtmlSpaceTokenCursor::new(rel.code_units());
        let mut stylesheet = false;
        let mut extension = false;
        loop {
            c.charge(
                WorkCategory::RecordVisit,
                bounded::HtmlSpaceTokenCursor::STEP_WORK as u64,
            )?;
            match cursor.step() {
                bounded::HtmlSpaceTokenStep::Continue => {}
                bounded::HtmlSpaceTokenStep::Done => break,
                bounded::HtmlSpaceTokenStep::Token { start, end } => {
                    struct Meter<'a>(&'a mut ParseControl);
                    impl bounded::HtmlRelMeter for Meter<'_> {
                        type Error = SessionError;
                        fn work(&mut self, units: usize) -> Result<(), Self::Error> {
                            self.0
                                .charge(WorkCategory::RecordVisit, units as u64)
                                .map_err(Into::into)
                        }
                        fn reserve(
                            &mut self,
                            _: &mut bounded::HtmlRelReservation<'_>,
                        ) -> Result<(), Self::Error> {
                            Err(SessionError::WrongToken)
                        }
                    }
                    let (keyword, _) =
                        bounded::classify_rel(&rel.code_units()[start..end], &mut Meter(c))
                            .map_err(|error| match error {
                                bounded::HtmlRelActionError::Admission(error) => error,
                                bounded::HtmlRelActionError::Protocol(_) => {
                                    SessionError::WrongToken
                                }
                            })?;
                    stylesheet |= keyword == Some(surgeist_html::HtmlRelKeyword::Stylesheet);
                    extension |= keyword.is_none();
                }
            }
        }
        if stylesheet {
            self.record_association(c, node, OccurrenceKind::UnsupportedLink, None, None, false)?;
        } else if extension {
            self.record_association(
                c,
                node,
                OccurrenceKind::UnsupportedLinkExtension,
                None,
                None,
                false,
            )?;
        }
        Ok(())
    }
}

/// One actual payload slot per step; no hidden next-match traversal.
pub struct Cursor<'a> {
    history: &'a FrozenHistory,
    storage: &'a FrozenStorage,
    next: usize,
}
pub enum Step<'a> {
    Document(DocumentView<'a>),
    Occurrence(OccurrenceView<'a>),
    Skipped,
    Complete,
}
impl<'a> Cursor<'a> {
    pub(crate) fn new(history: &'a FrozenHistory) -> Self {
        Self {
            history,
            storage: history.arena().storage.as_ref().expect("qualified report"),
            next: 0,
        }
    }
    pub fn advance(&mut self) -> Step<'a> {
        let slot = self.next;
        let Some(row) = self.storage.payloads.values.get(slot) else {
            return Step::Complete;
        };
        self.next += 1;
        match row {
            RetainedPayload::AssociationDocument(value) => Step::Document(DocumentView {
                history: self.history,
                storage: self.storage,
                value,
            }),
            RetainedPayload::AssociationOccurrence(value) => Step::Occurrence(OccurrenceView {
                history: self.history,
                storage: self.storage,
                slot,
                value: &value.as_slice()[0],
            }),
            _ => Step::Skipped,
        }
    }
}
fn native(storage: &FrozenStorage, index: usize) -> HtmlNode<'_> {
    storage
        .view
        .node_at(index)
        .expect("qualified original subject")
        .as_node()
}
fn document<'a>(storage: &'a FrozenStorage, value: &DocumentRef) -> report::MetadataDocument<'a> {
    match value {
        DocumentRef::Owned(node) => report::MetadataDocument::Owned(native(storage, *node)),
        DocumentRef::Captured(id) => report::MetadataDocument::Captured(
            storage
                .fragment
                .as_ref()
                .expect("qualified capture")
                .context
                .capture()
                .node(id.subject())
                .expect("qualified Document"),
        ),
    }
}
fn identity(storage: &FrozenStorage, slot: usize) -> OccurrenceId {
    OccurrenceId {
        result: native(storage, 0).id(),
        slot,
    }
}
fn context_epoch(storage: &FrozenStorage, slot: usize) -> ContextEpochId {
    ContextEpochId {
        result: native(storage, 0).id(),
        slot,
    }
}
fn group<'a>(history: &'a FrozenHistory, value: &'a OriginGroupRef) -> HistoryGroup<'a> {
    history
        .view()
        .expect("qualified history")
        .group(value.index)
        .expect("qualified snapshot group")
}
pub struct DocumentView<'a> {
    history: &'a FrozenHistory,
    storage: &'a FrozenStorage,
    value: &'a DocumentCapture,
}
impl<'a> DocumentView<'a> {
    pub fn document(&self) -> report::MetadataDocument<'a> {
        document(self.storage, &self.value.document)
    }
    pub fn disposition(&self) -> CaptureDisposition {
        self.value.missing.map_or(
            CaptureDisposition::PendingComplete,
            CaptureDisposition::Unavailable,
        )
    }
    pub fn occurrence_count(&self) -> usize {
        self.value.count
    }
    pub fn has_obligations(&self) -> bool {
        self.value.obligations != 0
    }
    pub fn last_occurrence(&self) -> Option<OccurrenceId> {
        self.value.last.map(|slot| identity(self.storage, slot))
    }
    pub fn first_missing(&self) -> Option<OccurrenceId> {
        self.value
            .missing_at
            .map(|slot| identity(self.storage, slot))
    }
    pub fn history(&self) -> crate::construction::read::HtmlParseHistory<'a> {
        crate::construction::read::HtmlParseHistory::new(self.history)
    }
}
pub struct OccurrenceView<'a> {
    history: &'a FrozenHistory,
    storage: &'a FrozenStorage,
    slot: usize,
    value: &'a Occurrence,
}
impl<'a> OccurrenceView<'a> {
    pub fn id(&self) -> OccurrenceId {
        identity(self.storage, self.slot)
    }
    pub fn kind(&self) -> OccurrenceKind {
        self.value.kind
    }
    pub fn requires_processing(&self) -> bool {
        self.value.requires_processing
    }
    pub fn subject(&self) -> HtmlNode<'a> {
        native(self.storage, self.value.subject)
    }
    pub fn document(&self) -> report::MetadataDocument<'a> {
        document(self.storage, &self.value.document)
    }
    pub fn root(&self) -> HtmlNode<'a> {
        native(self.storage, self.value.root)
    }
    pub fn previous_document(&self) -> Option<report::MetadataDocument<'a>> {
        matches!(self.value.kind, OccurrenceKind::StyleDocumentChanged)
            .then(|| document(self.storage, &self.value.document))
    }
    /// Adoption is captured before replacing the old Document relation.
    pub fn next_document(&self) -> Option<report::MetadataDocument<'a>> {
        self.value
            .previous_document
            .as_ref()
            .map(|d| document(self.storage, d))
    }
    /// Prior original invocation in this Document. This records causal source
    /// order, not successful sheet addition or asynchronous resource completion.
    pub fn prior_document(&self) -> Option<OccurrenceId> {
        self.value
            .prior_document
            .map(|slot| identity(self.storage, slot))
    }
    /// Prior lifecycle invocation for this Style. Consumers preserve the HTML
    /// synchronous prefix and genuine resource dependencies; this edge does not
    /// impose lexical completion order on outstanding resource operations.
    pub fn prior_style(&self) -> Option<OccurrenceId> {
        self.value
            .prior_style
            .map(|slot| identity(self.storage, slot))
    }
    pub fn cause(&self) -> Option<read::HtmlParseCause<'a>> {
        self.value.cause.as_ref().map(|cause| {
            read::HtmlParseCause::qualified(
                self.history,
                self.storage
                    .evidence
                    .as_ref()
                    .and_then(|e| e.retained_cause(cause))
                    .expect("qualified occurrence cause"),
            )
        })
    }
    pub fn preferred_name(&self) -> Option<HistoryGroup<'a>> {
        self.value
            .preferred
            .as_ref()
            .map(|value| group(self.history, value))
    }
    pub fn base_changed_input(&self) -> Option<HistoryGroup<'a>> {
        self.value
            .base_changed_input
            .as_ref()
            .map(|value| group(self.history, value))
    }
    pub fn style(&self) -> Option<StyleInputs<'a>> {
        self.value.inputs.as_ref().map(|value| StyleInputs {
            history: self.history,
            storage: self.storage,
            value,
        })
    }
    pub fn context(&self) -> ContextView<'a> {
        ContextView {
            history: self.history,
            storage: self.storage,
            slot: self.slot,
            document: &self.value.document,
            value: &self.value.context,
        }
    }
}
pub struct StyleInputs<'a> {
    history: &'a FrozenHistory,
    storage: &'a FrozenStorage,
    value: &'a Inputs,
}
impl<'a> StyleInputs<'a> {
    pub fn state(&self) -> surgeist_html::HtmlStyleState {
        self.value.state
    }
    pub fn creating_parser_document(&self) -> Option<report::MetadataDocument<'a>> {
        self.value
            .parser_document
            .as_ref()
            .map(|d| document(self.storage, d))
    }
    pub fn connected(&self) -> bool {
        self.value.connected
    }
    pub fn type_eligible(&self) -> bool {
        self.value.type_eligible
    }
    pub fn in_document_tree(&self) -> bool {
        self.value.document_tree
    }
    pub fn text_len(&self) -> usize {
        self.value.text.len()
    }
    pub fn text(&self, index: usize) -> Option<HistoryGroup<'a>> {
        self.value
            .text
            .as_slice()
            .get(index)
            .map(|value| group(self.history, value))
    }
    pub fn sheet_order_len(&self) -> usize {
        self.value.order.len()
    }
    pub fn sheet_order(&self, index: usize) -> Option<HtmlNode<'a>> {
        self.value
            .order
            .as_slice()
            .get(index)
            .map(|node| native(self.storage, *node))
    }
    pub fn attribute(
        &self,
        attribute: surgeist_html::KnownHtmlAttribute,
    ) -> Option<HistoryGroup<'a>> {
        let index = match attribute {
            surgeist_html::KnownHtmlAttribute::Type => 0,
            surgeist_html::KnownHtmlAttribute::Media => 1,
            surgeist_html::KnownHtmlAttribute::Title => 2,
            surgeist_html::KnownHtmlAttribute::Blocking => 3,
            surgeist_html::KnownHtmlAttribute::Nonce => 4,
            _ => return None,
        };
        self.value.attributes[index]
            .as_ref()
            .map(|value| group(self.history, value))
    }
}
pub struct ContextView<'a> {
    history: &'a FrozenHistory,
    storage: &'a FrozenStorage,
    slot: usize,
    document: &'a DocumentRef,
    value: &'a Context,
}
impl<'a> ContextView<'a> {
    /// The exact association applicable at this occurrence, never final policy.
    pub fn csp_association(&self) -> Option<&'a surgeist_html::HtmlCspAssociation> {
        if self.value.policy_missing {
            return None;
        }
        let slot = self.value.csp?;
        let RetainedPayload::CspPolicy(value) = &self.storage.payloads.values[slot] else {
            unreachable!("qualified policy")
        };
        value.value.as_ref()
    }
    pub fn epoch(&self) -> ContextEpochId {
        context_epoch(self.storage, self.slot)
    }
    /// Immutable initial Document input lineage for URL, fallback, encoding and
    /// initial policy; other roles retain their original occurrence frontier.
    /// This never identifies a derived selected Base result: original Base
    /// inputs, selection epochs and known providers remain occurrence-specific.
    pub fn input_epoch(&self, role: ContextRole) -> ContextEpochId {
        if matches!(
            role,
            ContextRole::DocumentUrl
                | ContextRole::FallbackBase
                | ContextRole::Encoding
                | ContextRole::InitialPolicy
        ) {
            context_epoch(self.storage, self.value.input_document)
        } else {
            self.epoch()
        }
    }
    pub fn document_mode(&self) -> HtmlDocumentMode {
        self.value.mode
    }
    pub fn document_class(&self) -> crate::HtmlDocumentClass {
        self.value.class
    }
    pub fn body_present(&self) -> Option<bool> {
        matches!(self.document, DocumentRef::Owned(_)).then_some(self.value.body)
    }
    pub fn requirement(&self, role: ContextRole) -> ContextRequirement {
        if self.value.missing {
            return ContextRequirement::Unavailable(MissingKind::OriginalContext);
        }
        if role == ContextRole::InitialPolicy && self.value.policy_missing {
            return ContextRequirement::Unavailable(MissingKind::PolicyHistory);
        }
        if role == ContextRole::InitialPolicy && self.value.csp.is_some() {
            return ContextRequirement::Known;
        }
        if matches!(self.document, DocumentRef::Captured(_)) {
            if (self.value.captured.is_some()
                && matches!(role, ContextRole::FallbackBase | ContextRole::Encoding))
                || (role == ContextRole::DocumentEligibility
                    && self.value.class != crate::HtmlDocumentClass::Unavailable)
            {
                ContextRequirement::Known
            } else {
                ContextRequirement::DeferredCaptured
            }
        } else if let Some(facts) = self.base_facts() {
            let known = match role {
                ContextRole::DocumentUrl => facts.document().document_url().is_some(),
                ContextRole::FallbackBase => self.selected_base().is_ok(),
                ContextRole::Encoding => facts.document().encoding().is_some(),
                ContextRole::BodyPresence | ContextRole::DocumentEligibility => true,
                _ => false,
            };
            if known {
                ContextRequirement::Known
            } else {
                ContextRequirement::OfflineUnbound
            }
        } else if matches!(
            role,
            ContextRole::BodyPresence | ContextRole::DocumentEligibility
        ) {
            ContextRequirement::Known
        } else {
            ContextRequirement::OfflineUnbound
        }
    }
    pub fn slot(&self, role: ContextRole) -> Option<ContextSlotId> {
        (self.requirement(role) == ContextRequirement::OfflineUnbound).then(|| ContextSlotId {
            epoch: self.input_epoch(role),
            role,
        })
    }
    /// Exact original capture authority for a deferred observation. This lender
    /// is a request basis, not permission to replace it with current host facts.
    pub fn captured_observation(&self, role: ContextRole) -> Option<CapturedObservation<'a>> {
        if self.requirement(role) != ContextRequirement::DeferredCaptured {
            return None;
        }
        let DocumentRef::Captured(document) = self.document else {
            return None;
        };
        Some(CapturedObservation {
            epoch: self.input_epoch(role),
            role,
            document,
            context: &self
                .storage
                .fragment
                .as_ref()
                .expect("qualified original context")
                .context,
        })
    }
    pub fn base_facts(&self) -> Option<&'a crate::HtmlParseBaseFacts> {
        self.value
            .base_document
            .map(|slot| match &self.storage.payloads.values[slot] {
                RetainedPayload::BaseDocument { facts, .. } => facts,
                _ => unreachable!("qualified base provider"),
            })
    }
    pub fn captured_context(
        &self,
    ) -> Option<&'a surgeist_html_semantics::HtmlDocumentEncodingUrlContext> {
        self.value
            .captured
            .map(|slot| match &self.storage.payloads.values[slot] {
                RetainedPayload::AssociationCapturedContext(value) => &value.value,
                _ => unreachable!("qualified captured provider"),
            })
    }
    pub fn first_base(&self) -> Option<HtmlNode<'a>> {
        self.value.first_base.map(|node| native(self.storage, node))
    }
    pub fn base_input(&self) -> Option<HistoryGroup<'a>> {
        self.value
            .base_input
            .as_ref()
            .map(|value| group(self.history, value))
    }
    pub fn base_observation(&self) -> Option<usize> {
        self.value.base_observation
    }
    pub fn selected_base(&self) -> Result<&'a crate::HtmlBaseUrlRecord, crate::HtmlBaseMissing> {
        if let Some(context) = self.captured_context() {
            return Ok(context.base_url().value());
        }
        let facts = self
            .base_facts()
            .ok_or(crate::HtmlBaseMissing::DocumentFacts)?;
        if self.value.first_base.is_some() {
            let index = self
                .value
                .base_observation
                .ok_or(crate::HtmlBaseMissing::FrozenBase)?;
            let fallback = facts.document().fallback()?;
            crate::select_frozen_base(fallback, facts.observations()[index].outcome())
                .map(|(value, _)| value)
        } else {
            surgeist_html_semantics::select_document_base(
                Some(facts.document()),
                surgeist_html_semantics::HtmlDocumentFirstBase::Absent,
            )
        }
    }
}
pub struct CapturedObservation<'a> {
    epoch: ContextEpochId,
    role: ContextRole,
    document: &'a crate::HtmlCaptureDocumentId,
    context: &'a crate::HtmlFragmentContext,
}
impl CapturedObservation<'_> {
    pub fn epoch(&self) -> &ContextEpochId {
        &self.epoch
    }
    pub fn role(&self) -> ContextRole {
        self.role
    }
    pub fn document(&self) -> &crate::HtmlCaptureDocumentId {
        self.document
    }
    pub fn context(&self) -> &crate::HtmlFragmentContext {
        self.context
    }
}

pub(super) fn qualify(storage: &FrozenStorage, c: &mut ParseControl) -> Result<(), SessionError> {
    for (slot, row) in storage.payloads.values.iter().enumerate() {
        c.charge(WorkCategory::RecordVisit, 1)?;
        match row {
            RetainedPayload::AssociationDocument(value) => {
                storage.validate_association_document(c, &value.document)?;
                if value
                    .last
                    .is_some_and(|index| index <= slot || index >= storage.payloads.values.len())
                {
                    return Err(SessionError::WrongToken);
                }
            }
            RetainedPayload::AssociationOccurrence(values) => {
                values.require_disposal_owner(c)?;
                if values.len() != 1 {
                    return Err(SessionError::WrongToken);
                }
                let value = &values.as_slice()[0];
                if value.context.input_document >= slot
                    || !matches!(
                        storage.payloads.values.get(value.context.input_document),
                        Some(RetainedPayload::AssociationDocument(summary)) if summary.document == value.document
                    )
                {
                    return Err(SessionError::WrongToken);
                }
                storage.validate_association_document(c, &value.document)?;
                if let Some(document) = &value.previous_document {
                    storage.validate_association_document(c, document)?;
                }
                if storage.view.node_at(value.subject).is_none()
                    || storage.view.node_at(value.root).is_none()
                {
                    return Err(SessionError::WrongToken);
                }
                for prior in [value.prior_document, value.prior_style]
                    .into_iter()
                    .flatten()
                {
                    if prior >= slot
                        || !matches!(
                            storage.payloads.values.get(prior),
                            Some(RetainedPayload::AssociationOccurrence(_))
                        )
                    {
                        return Err(SessionError::WrongToken);
                    }
                }
                let evidence = storage.evidence.as_ref().ok_or(SessionError::WrongToken)?;
                if let Some(cause) = &value.cause
                    && evidence.retained_cause(cause).is_none()
                {
                    return Err(SessionError::WrongToken);
                }
                for reference in [
                    value.preferred.as_ref(),
                    value.base_changed_input.as_ref(),
                    value.context.base_input.as_ref(),
                ]
                .into_iter()
                .flatten()
                {
                    evidence.require_group(reference)?;
                    c.charge(WorkCategory::RecordVisit, 1)?;
                }
                if let Some(inputs) = &value.inputs {
                    inputs.text.require_disposal_owner(c)?;
                    inputs.order.require_disposal_owner(c)?;
                    for node in inputs.order.as_slice() {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        if storage.view.node_at(*node).is_none() {
                            return Err(SessionError::WrongToken);
                        }
                    }
                    for reference in inputs
                        .attributes
                        .iter()
                        .flatten()
                        .chain(inputs.text.as_slice())
                    {
                        evidence.require_group(reference)?;
                        c.charge(WorkCategory::RecordVisit, 1)?;
                    }
                    if let Some(document) = &inputs.parser_document {
                        storage.validate_association_document(c, document)?;
                    }
                }
                if let Some(provider) = value.context.base_document
                    && !matches!((&value.document, storage.payloads.values.get(provider)), (DocumentRef::Owned(document), Some(RetainedPayload::BaseDocument { node, facts, .. })) if node == document && value.context.base_observation.is_none_or(|index| index < facts.observations().len()))
                {
                    return Err(SessionError::WrongToken);
                }
                if let Some(provider) = value.context.captured
                    && !matches!((&value.document, storage.payloads.values.get(provider)), (DocumentRef::Captured(document), Some(RetainedPayload::AssociationCapturedContext(context))) if context.value.document() == document)
                {
                    return Err(SessionError::WrongToken);
                }
            }
            _ => {}
        }
    }
    Ok(())
}
impl FrozenStorage {
    fn validate_association_document(
        &self,
        c: &mut ParseControl,
        document: &DocumentRef,
    ) -> Result<(), SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        match document {
            DocumentRef::Owned(node)
                if matches!(
                    self.view.node_at(*node).map(|n| n.kind()),
                    Some(HtmlNodeKind::Document(_))
                ) =>
            {
                Ok(())
            }
            DocumentRef::Captured(id)
                if self
                    .fragment
                    .as_ref()
                    .and_then(|f| f.context.capture().node(id.subject()).ok())
                    .is_some() =>
            {
                Ok(())
            }
            _ => Err(SessionError::WrongToken),
        }
    }
}
pub(super) fn release_occurrence(mut values: OwnedVec<Occurrence>, c: &mut ParseControl) {
    while let Some(value) = if c.stop().is_some() {
        values.cleanup_pop(c)
    } else {
        values.try_pop(c)
    }
    .unwrap_or_else(|_| {
        values
            .cleanup_pop(c)
            .expect("terminal association receiver disposal")
    }) {
        release_inputs(value, c);
    }
    values
        .release(c)
        .expect("owned association occurrence receiver");
}
fn release_inputs(value: Occurrence, c: &mut ParseControl) {
    if let Some(inputs) = value.inputs {
        inputs
            .text
            .release(c)
            .expect("owned association text inputs");
        inputs
            .order
            .release(c)
            .expect("owned association order inputs");
    }
}
pub(super) fn captured_usage(value: &CapturedContext) -> associated::AssociatedStorageUsage {
    value.usage
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::tokenizer::{states::State, tests::LexerFixture};

    fn lexical_node(
        session: &mut ParserArenaSession,
        c: &mut ParseControl,
        lexer: &mut LexerFixture<'_>,
    ) -> ParserNode {
        let TokenKind::Tag(tag) = super::super::tests::lexical_kind(lexer, c) else {
            panic!("literal tag")
        };
        session.allocate_html_tag(c, &mut Some(tag)).unwrap()
    }
    fn generated(c: &mut ParseControl, units: &[u16]) -> Text {
        let mut value = Utf16Buffer::new(c).unwrap();
        value.extend_units(units, c).unwrap();
        Text {
            value: value.freeze(c).unwrap(),
            origins: OwnedVec::new(HtmlParseResource::SourceMapEntries).unwrap(),
        }
    }
    fn read_group(
        session: &ParserArenaSession,
        c: &mut ParseControl,
        group: &OriginGroupRef,
    ) -> Vec<u16> {
        let storage = session.active().unwrap();
        let value = storage
            .evidence
            .as_ref()
            .unwrap()
            .materialize_group(c, &storage.payloads, group)
            .unwrap();
        let units = value.text().code_units().to_vec();
        value.release(c).unwrap();
        units
    }
    #[test]
    fn original_coalesced_text_retitle_removal_and_document_move_keep_old_exact_inputs() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<style title=A><i title=B>"),
            super::super::tests::limits(),
        );
        let (mut c, mut session, capability) = super::super::tests::fresh_script_owner(&request);
        let mut lexer = LexerFixture::new(request.source(), State::Data, None, &mut c).unwrap();
        let root = session.root(&mut c).unwrap();
        let style = lexical_node(&mut session, &mut c, &mut lexer);
        session
            .insert(&mut c, &root, &style, None, Some(&capability))
            .unwrap();
        for units in [&[0xd800, 0][..], &[0x41, 0xdfff][..], &[0x42][..]] {
            let mut input = Some(generated(&mut c, units));
            session
                .insert_text(&mut c, &style, None, &mut input, Some(&capability))
                .unwrap();
            assert!(input.is_none());
        }
        let TokenKind::Tag(mut tag) = super::super::tests::lexical_kind(&mut lexer, &mut c) else {
            panic!("literal replacement")
        };
        let mut attr = tag.attributes.try_pop(&mut c).unwrap();
        tag.name.release(&mut c).unwrap();
        tag.attributes.release(&mut c).unwrap();
        session
            .set_managed_attribute(
                &mut c,
                &style,
                &mut attr,
                &capability,
                mutation::ManagedFamily::Style,
            )
            .unwrap();
        assert!(attr.is_none());
        session
            .remove_managed_attribute(&mut c, &style, "title", mutation::ManagedFamily::Style)
            .unwrap();
        let document = session
            .active_mut()
            .unwrap()
            .appropriate_contents_document(&mut c, &DocumentRef::Owned(0))
            .unwrap();
        let next = session
            .active()
            .unwrap()
            .node(&mut c, document.owned().unwrap())
            .unwrap();
        session
            .insert(&mut c, &next, &style, None, Some(&capability))
            .unwrap();
        let storage = session.active().unwrap();
        let rows: Vec<_> = storage
            .payloads
            .values
            .iter()
            .filter_map(|row| match row {
                RetainedPayload::AssociationOccurrence(values)
                    if values.as_slice()[0].subject == style.index =>
                {
                    Some(&values.as_slice()[0])
                }
                _ => None,
            })
            .collect();
        let texts: Vec<_> = rows
            .iter()
            .filter(|row| {
                matches!(
                    row.kind,
                    OccurrenceKind::StyleUpdate(
                        surgeist_html::HtmlStyleTrigger::ChildrenChanged { .. }
                    )
                )
            })
            .collect();
        assert_eq!(texts.len(), 3);
        assert_eq!(
            read_group(
                &session,
                &mut c,
                &texts[0].inputs.as_ref().unwrap().text.as_slice()[0]
            ),
            [0xd800, 0]
        );
        assert_eq!(
            read_group(
                &session,
                &mut c,
                &texts[1].inputs.as_ref().unwrap().text.as_slice()[0]
            ),
            [0xd800, 0, 0x41, 0xdfff]
        );
        assert_eq!(
            read_group(
                &session,
                &mut c,
                &texts[2].inputs.as_ref().unwrap().text.as_slice()[0]
            ),
            [0xd800, 0, 0x41, 0xdfff, 0x42]
        );
        let attributes: Vec<_> = rows
            .iter()
            .filter(|row| row.kind == OccurrenceKind::StyleOwnerAttributes)
            .collect();
        assert!(attributes.iter().any(|row| {
            row.inputs.as_ref().unwrap().attributes[2]
                .as_ref()
                .is_some_and(|group| read_group(&session, &mut c, group) == [66])
        }));
        assert!(
            attributes
                .last()
                .unwrap()
                .inputs
                .as_ref()
                .unwrap()
                .attributes[2]
                .is_none()
        );
        let moved = rows
            .iter()
            .find(|row| row.kind == OccurrenceKind::StyleDocumentChanged)
            .unwrap();
        assert_eq!(moved.document, DocumentRef::Owned(0));
        assert_eq!(moved.previous_document.as_ref(), Some(&document));
        assert!(
            rows.iter().any(|row| row.document == document
                && matches!(row.kind, OccurrenceKind::StyleUpdate(_)))
        );
        lexer.release(&mut c).unwrap();
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }
    #[test]
    fn original_evidence_rejects_native_style_without_document_binding() {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(""),
            super::super::tests::limits(),
        );
        let (mut c, mut session, _) = super::super::tests::fresh_script_owner(&request);
        let storage = session.active_mut().unwrap();
        let style = storage
            .allocate_element(
                &mut c,
                HtmlElement::new(
                    QualifiedName::new(Namespace::html(), None, "style"),
                    OrderedAttributes::default(),
                ),
            )
            .unwrap();
        assert!(matches!(
            storage.initialize_style(&mut c, style.index),
            Err(SessionError::NodeDocumentUnavailable)
        ));
        assert!(
            !storage
                .payloads
                .values
                .iter()
                .any(|row| matches!(row, RetainedPayload::AssociationOccurrence(_)))
        );
        session.release(&mut c).unwrap();
        super::super::tests::storage_zero(&c);
    }
    #[test]
    fn association_reference_promotes_unmapped_backing_before_retirement_and_sticky_cancel_cleanup()
    {
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8(""),
            super::super::tests::limits(),
        );
        let (mut c, mut session, _) = super::super::tests::fresh_script_owner(&request);
        let mut raw = Utf16Buffer::new(&mut c).unwrap();
        raw.extend_units(&[0xd800, 0, 65], &mut c).unwrap();
        let raw = raw.freeze(&mut c).unwrap();
        let (handle, slot) = session
            .active_mut()
            .unwrap()
            .register_group_string_owned(&mut c, raw, None)
            .unwrap();
        drop(handle);
        let reference = session
            .active_mut()
            .unwrap()
            .association_string(&mut c, slot)
            .unwrap();
        session
            .active_mut()
            .unwrap()
            .retire_registered_string(&mut c, slot)
            .unwrap();
        assert!(matches!(
            session.active().unwrap().payloads.values[slot],
            RetainedPayload::String {
                origins: Some(_),
                ..
            }
        ));
        assert_eq!(read_group(&session, &mut c, &reference), [0xd800, 0, 65]);
        request.cancellation().cancel();
        let first = c.checkpoint().unwrap_err();
        assert_eq!(c.checkpoint(), Err(first));
        drop(reference);
        session.release(&mut c).unwrap();
        assert_eq!(c.stop(), Some(first));
        super::super::tests::storage_zero(&c);
    }
}
