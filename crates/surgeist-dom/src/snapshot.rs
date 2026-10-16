use std::sync::Arc;

use crate::store::{Node, State};
use crate::{DocumentId, DocumentString, DomError, DomOwnerId, DomRevision, DomStamp, NodeId};
use crate::{DomDocumentMetadata, DomDocumentType, DomElement, DomProcessingInstruction};

/// The complete node kinds supported by this contribution.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomNodeKind {
    Document(DomDocumentMetadata),
    Element(DomElement),
    DocumentFragment,
    ShadowRoot(crate::DomShadowRootState),
    Text(DocumentString),
    Comment(DocumentString),
    ProcessingInstruction(DomProcessingInstruction),
    DocumentType(DomDocumentType),
}

/// Owned immutable state, including detached nodes and all documents.
#[derive(Clone, Debug)]
pub struct DomSnapshot {
    pub(crate) owner: DomOwnerId,
    pub(crate) revision: DomRevision,
    pub(crate) state: Arc<State>,
}

impl DomSnapshot {
    pub fn incomplete_hosted_operation(&self) -> Option<crate::hosted::DomHostedOperationId> {
        self.state.incomplete_hosted
    }
    pub fn incomplete_source_operation(&self) -> Option<crate::DomSourceOperationId> {
        self.state.incomplete_source
    }
    pub(crate) fn require_complete_source(&self) -> Result<(), DomError> {
        if self.state.incomplete_source.is_some() || self.state.incomplete_hosted.is_some() {
            Err(DomError::IncompleteSourceOperation)
        } else {
            Ok(())
        }
    }
    pub fn owner(&self) -> DomOwnerId {
        self.owner
    }
    pub fn revision(&self) -> DomRevision {
        self.revision
    }
    pub fn stamp(&self) -> DomStamp {
        DomStamp {
            owner: self.owner,
            revision: self.revision,
        }
    }
    pub fn pending_details_toggle_intentions(&self) -> usize {
        self.state.details_intentions.len()
    }
    pub fn node(&self, id: NodeId) -> Result<DomNode<'_>, DomError> {
        let node = self.state.node(self.owner, id)?;
        Ok(DomNode {
            snapshot: self,
            id,
            node,
        })
    }
    pub fn document(&self, id: DocumentId) -> Result<DomDocument<'_>, DomError> {
        if !matches!(self.node(id.node())?.kind(), DomNodeKind::Document(_)) {
            return Err(DomError::WrongNodeKind);
        }
        Ok(DomDocument { snapshot: self, id })
    }
    /// Every managed node, including detached nodes. Arena order is unspecified.
    pub fn nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.state
            .slots
            .iter()
            .enumerate()
            .filter_map(|(slot, value)| {
                value.node.as_ref().map(|_| NodeId {
                    owner: self.owner,
                    slot,
                    generation: value.generation,
                })
            })
    }
}

/// A document-qualified view over one exact immutable store revision.
#[derive(Clone, Copy, Debug)]
pub struct DomDocument<'a> {
    pub(crate) snapshot: &'a DomSnapshot,
    pub(crate) id: DocumentId,
}

impl<'a> DomDocument<'a> {
    pub fn id(self) -> DocumentId {
        self.id
    }
    /// Activity derived from this immutable snapshot's actual published
    /// association. Later issuer retirement never changes this historical read.
    pub fn is_fully_active(self) -> Result<bool, crate::hosted::DomHostedError> {
        self.snapshot.require_complete_source()?;
        match &self.metadata().document_environment().hosted {
            Some(association) => association.historical_fully_active(),
            None => Ok(false),
        }
    }
    /// Canonical metadata belonging to this exact captured document revision.
    pub fn metadata(self) -> &'a DomDocumentMetadata {
        let DomNodeKind::Document(metadata) = self
            .snapshot
            .node(self.id.node())
            .expect("captured document")
            .kind()
        else {
            unreachable!("checked document view")
        };
        metadata
    }
    pub fn stamp(self) -> DomDocumentStamp {
        DomDocumentStamp {
            store: self.snapshot.stamp(),
            document: self.id,
        }
    }
    /// Validates membership at this captured revision. A Document belongs to
    /// itself, although its source-defined ownerDocument getter returns None.
    pub fn node(self, id: NodeId) -> Result<DomNode<'a>, DomError> {
        let node = self.snapshot.node(id)?;
        if node.node.document != self.id {
            return Err(DomError::WrongDocument);
        }
        Ok(node)
    }
}

/// Exact store, document and revision qualification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomDocumentStamp {
    store: DomStamp,
    document: DocumentId,
}
impl DomDocumentStamp {
    pub fn store(self) -> DomStamp {
        self.store
    }
    pub fn document(self) -> DocumentId {
        self.document
    }
}

/// A checked node read from an immutable snapshot.
#[derive(Clone, Copy, Debug)]
pub struct DomNode<'a> {
    pub(crate) snapshot: &'a DomSnapshot,
    id: NodeId,
    pub(crate) node: &'a Node,
}
impl<'a> DomNode<'a> {
    pub fn as_form(self) -> Option<crate::DomForm<'a>> {
        self.as_element()
            .and_then(DomElement::form_state)
            .map(|_| crate::DomForm { node: self })
    }
    pub fn as_fieldset(self) -> Option<crate::DomFieldset<'a>> {
        self.as_element()
            .and_then(DomElement::fieldset_state)
            .map(|_| crate::DomFieldset { node: self })
    }
    pub fn as_button(self) -> Option<crate::DomButton<'a>> {
        self.as_element()
            .and_then(DomElement::button_state)
            .map(|_| crate::DomButton { node: self })
    }
    pub fn as_input(self) -> Option<crate::DomInput<'a>> {
        self.as_element()?.input_state()?;
        Some(crate::DomInput { node: self })
    }
    pub fn as_details(self) -> Option<crate::DomDetails<'a>> {
        self.as_element()?
            .is_details()
            .then_some(crate::DomDetails { node: self })
    }
    pub fn as_select(self) -> Option<crate::DomSelect<'a>> {
        self.as_element()?.select_state()?;
        Some(crate::DomSelect { node: self })
    }
    pub fn as_option(self) -> Option<crate::DomOption<'a>> {
        self.as_element()?.option_state()?;
        Some(crate::DomOption { node: self })
    }
    pub fn as_selectedcontent(self) -> Option<crate::DomSelectedcontent<'a>> {
        self.as_element()?.selectedcontent_disabled()?;
        Some(crate::DomSelectedcontent { node: self })
    }
    pub fn as_output(self) -> Option<crate::DomOutput<'a>> {
        self.as_element()?.output_state()?;
        Some(crate::DomOutput { node: self })
    }
    pub fn as_textarea(self) -> Option<crate::DomTextarea<'a>> {
        self.as_element()?.textarea_state()?;
        Some(crate::DomTextarea { node: self })
    }

    /// Privileged canonical ownership, including closed roots. This does not
    /// implement the source's open-only Element.shadowRoot presentation.
    pub fn hosted_shadow_root(self) -> Option<NodeId> {
        self.node.shadow_root
    }
    /// Source Element.shadowRoot presentation: a closed root is not exposed.
    pub fn shadow_root(self) -> Option<NodeId> {
        let root = self.hosted_shadow_root()?;
        let state = self
            .snapshot
            .shadow_root(root)
            .expect("canonical root")
            .state();
        (state.mode() == crate::HtmlShadowRootMode::Open).then_some(root)
    }
    /// The exact ShadowRoot's host. Template contents instead use fragment_host().
    pub fn shadow_host(self) -> Option<NodeId> {
        matches!(self.kind(), DomNodeKind::ShadowRoot(_))
            .then_some(self.node.host)
            .flatten()
    }
    pub fn as_shadow_root(self) -> Option<DomShadowRoot<'a>> {
        matches!(self.kind(), DomNodeKind::ShadowRoot(_)).then_some(DomShadowRoot { node: self })
    }
    /// Ordinary tree identity, with its separate host if the root is shadow.
    pub fn tree_scope(self) -> DomTreeScope {
        let root = self.root();
        DomTreeScope {
            root,
            host: self
                .snapshot
                .node(root)
                .expect("captured root")
                .shadow_host(),
        }
    }
    /// This crosses only a ShadowRoot's host boundary, never a light-child or
    /// assignment projection. The root itself still has no ordinary parent.
    pub fn shadow_including_parent(self) -> Option<NodeId> {
        self.parent().or(self.shadow_host())
    }
    /// Host-including ancestry crosses ShadowRoot and owned Template fragment hosts.
    pub fn host_including_parent(self) -> Option<NodeId> {
        self.parent().or(self.node.host)
    }
    pub fn shadow_including_root(self) -> NodeId {
        let mut node = self;
        while let Some(parent) = node.shadow_including_parent() {
            node = self
                .snapshot
                .node(parent)
                .expect("captured shadow ancestor");
        }
        node.id()
    }
    pub fn is_shadow_including_inclusive_ancestor_of(
        self,
        other: NodeId,
    ) -> Result<bool, DomError> {
        let mut next = Some(other);
        while let Some(id) = next {
            let node = self.snapshot.node(id)?;
            if self.id == id {
                return Ok(true);
            }
            next = node.shadow_including_parent();
        }
        Ok(false)
    }
    pub fn is_host_including_inclusive_ancestor_of(self, other: NodeId) -> Result<bool, DomError> {
        let mut next = Some(other);
        while let Some(id) = next {
            let node = self.snapshot.node(id)?;
            if self.id == id {
                return Ok(true);
            }
            next = node.host_including_parent();
        }
        Ok(false)
    }
    /// Inclusive ordinary preorder. No allocation, recursion or shadow piercing.
    pub fn tree_order(self) -> DomTreeOrder<'a> {
        DomTreeOrder {
            snapshot: self.snapshot,
            root: self.id,
            next: Some(self.id),
            shadow: false,
        }
    }
    /// Inclusive preorder visiting each hosted root immediately after its host,
    /// before the host's ordinary children, independent of open/closed mode.
    pub fn shadow_including_tree_order(self) -> DomTreeOrder<'a> {
        DomTreeOrder {
            snapshot: self.snapshot,
            root: self.id,
            next: Some(self.id),
            shadow: true,
        }
    }
    pub fn id(self) -> NodeId {
        self.id
    }
    pub fn kind(self) -> &'a DomNodeKind {
        &self.node.kind
    }
    /// DOM nodeName readback; PI targets and doctype names preserve exact units.
    pub fn node_name(self) -> DocumentString {
        match &self.node.kind {
            DomNodeKind::Document(_) => "#document".into(),
            DomNodeKind::Element(_) => self.tag_name().expect("Element tag name"),
            DomNodeKind::DocumentFragment | DomNodeKind::ShadowRoot(_) => {
                "#document-fragment".into()
            }
            DomNodeKind::Text(_) => "#text".into(),
            DomNodeKind::Comment(_) => "#comment".into(),
            DomNodeKind::ProcessingInstruction(value) => value.target().clone(),
            DomNodeKind::DocumentType(value) => value.name().clone(),
        }
    }
    /// Checked document view, including for documents discovered by enumeration.
    pub fn as_document(self) -> Option<DomDocument<'a>> {
        matches!(self.node.kind, DomNodeKind::Document(_)).then_some(DomDocument {
            snapshot: self.snapshot,
            id: DocumentId(self.id),
        })
    }
    pub fn as_element(self) -> Option<&'a DomElement> {
        match &self.node.kind {
            DomNodeKind::Element(value) => Some(value),
            _ => None,
        }
    }
    /// Source tagName follows the actual current node-document. Raw namespace,
    /// prefix and local name remain unchanged by this presentation or adoption.
    pub fn tag_name(self) -> Option<DocumentString> {
        let element = self.as_element()?;
        let spelling = crate::names::qualified_spelling(element.name());
        Some(if self.html_element_in_html_document() {
            DocumentString::from_utf16(
                spelling
                    .code_units()
                    .iter()
                    .map(|&unit| {
                        if (0x61..=0x7a).contains(&unit) {
                            unit - 0x20
                        } else {
                            unit
                        }
                    })
                    .collect::<Vec<_>>(),
            )
        } else {
            spelling
        })
    }
    fn html_element_in_html_document(self) -> bool {
        self.as_element()
            .is_some_and(|element| crate::names::in_html_namespace(element.name()))
            && self
                .snapshot
                .document(self.node.document)
                .expect("canonical node document")
                .metadata()
                .class()
                == crate::DomDocumentClass::Html
    }
    /// DOM getAttribute lookup, including HTML input casing from this exact
    /// captured node-document. The intrinsic DomElement lookup stays exact.
    pub fn attribute(self, qualified_name: &DocumentString) -> Option<&'a DocumentString> {
        let element = self.as_element()?;
        let html = self.html_element_in_html_document();
        element
            .attributes()
            .iter()
            .find(|attribute| {
                if html {
                    crate::names::qualified_equals_lowercase_input(attribute.name(), qualified_name)
                } else {
                    crate::names::qualified_equals(attribute.name(), qualified_name)
                }
            })
            .map(|attribute| attribute.value())
    }
    /// Exact namespace/local-name lookup; HTML input casing does not apply.
    pub fn attribute_ns(
        self,
        namespace: &crate::Namespace,
        local_name: &DocumentString,
    ) -> Option<&'a DocumentString> {
        self.as_element()?.attribute_ns(namespace, local_name)
    }
    pub fn parent_element(self) -> Option<NodeId> {
        self.parent().filter(|id| {
            self.snapshot
                .node(*id)
                .expect("captured parent")
                .as_element()
                .is_some()
        })
    }
    pub fn element_children(self) -> impl DoubleEndedIterator<Item = NodeId> + 'a {
        self.node.children.iter().copied().filter(move |id| {
            self.snapshot
                .node(*id)
                .expect("captured child")
                .as_element()
                .is_some()
        })
    }
    pub fn previous_sibling(self) -> Option<NodeId> {
        let parent = self.snapshot.node(self.parent()?).expect("captured parent");
        let position = parent
            .children()
            .iter()
            .position(|id| *id == self.id)
            .expect("captured sibling");
        position
            .checked_sub(1)
            .map(|position| parent.children()[position])
    }
    pub fn next_sibling(self) -> Option<NodeId> {
        let parent = self.snapshot.node(self.parent()?).expect("captured parent");
        let position = parent
            .children()
            .iter()
            .position(|id| *id == self.id)
            .expect("captured sibling");
        parent.children().get(position + 1).copied()
    }
    pub fn previous_element_sibling(self) -> Option<NodeId> {
        let mut next = self.previous_sibling();
        while let Some(id) = next {
            let node = self.snapshot.node(id).expect("captured sibling");
            if node.as_element().is_some() {
                return Some(id);
            }
            next = node.previous_sibling();
        }
        None
    }
    pub fn next_element_sibling(self) -> Option<NodeId> {
        let mut next = self.next_sibling();
        while let Some(id) = next {
            let node = self.snapshot.node(id).expect("captured sibling");
            if node.as_element().is_some() {
                return Some(id);
            }
            next = node.next_sibling();
        }
        None
    }
    /// Zero-based Element-only sibling index; detached Elements have index 0.
    pub fn element_index(self) -> Option<usize> {
        self.as_element()?;
        let Some(parent) = self.parent() else {
            return Some(0);
        };
        self.snapshot
            .node(parent)
            .expect("captured parent")
            .element_children()
            .position(|id| id == self.id)
    }
    /// DOM's ownerDocument getter: None for Document, otherwise node document.
    pub fn owner_document(self) -> Option<DocumentId> {
        (!matches!(self.node.kind, DomNodeKind::Document(_))).then_some(self.node.document)
    }
    pub fn parent(self) -> Option<NodeId> {
        self.node.parent
    }
    pub fn children(self) -> &'a [NodeId] {
        &self.node.children
    }
    pub fn data(self) -> Option<&'a DocumentString> {
        match &self.node.kind {
            DomNodeKind::Text(data) | DomNodeKind::Comment(data) => Some(data),
            DomNodeKind::ProcessingInstruction(value) => Some(value.data()),
            _ => None,
        }
    }
    /// Ordinary tree root; a detached node's owner document is not its root.
    pub fn root(self) -> NodeId {
        let mut id = self.id;
        let mut node = self.node;
        while let Some(parent) = node.parent {
            id = parent;
            node = self
                .snapshot
                .state
                .node(self.snapshot.owner, parent)
                .expect("snapshot parent invariant");
        }
        id
    }
    pub fn is_connected(self) -> bool {
        matches!(
            self.snapshot
                .node(self.shadow_including_root())
                .expect("snapshot root invariant")
                .kind(),
            DomNodeKind::Document(_)
        )
    }
}

/// One exact ordinary root and the separate host of a shadow tree scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomTreeScope {
    root: NodeId,
    host: Option<NodeId>,
}
impl DomTreeScope {
    pub fn root(self) -> NodeId {
        self.root
    }
    pub fn host(self) -> Option<NodeId> {
        self.host
    }
}

/// Checked ShadowRoot capture. Host/node-document are independent source
/// relations: internal fragment replacement can adopt the root without its host.
#[derive(Clone, Copy, Debug)]
pub struct DomShadowRoot<'a> {
    node: DomNode<'a>,
}
impl<'a> DomShadowRoot<'a> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn host(self) -> NodeId {
        self.node.shadow_host().expect("canonical non-null host")
    }
    pub fn state(self) -> &'a crate::DomShadowRootState {
        let DomNodeKind::ShadowRoot(state) = self.node.kind() else {
            unreachable!("checked root kind")
        };
        state
    }
    pub fn node(self) -> DomNode<'a> {
        self.node
    }
}

/// Bounded by the captured finite tree; this iterator owns no traversal stack.
#[derive(Clone, Debug)]
pub struct DomTreeOrder<'a> {
    snapshot: &'a DomSnapshot,
    root: NodeId,
    next: Option<NodeId>,
    shadow: bool,
}
impl Iterator for DomTreeOrder<'_> {
    type Item = NodeId;
    fn next(&mut self) -> Option<NodeId> {
        let current = self.next?;
        let node = self.snapshot.node(current).expect("captured traversal");
        self.next = if self.shadow {
            node.hosted_shadow_root()
        } else {
            None
        }
        .or_else(|| node.children().first().copied());
        if self.next.is_none() {
            let mut node = node;
            while node.id() != self.root {
                if let Some(host) = self.shadow.then(|| node.shadow_host()).flatten() {
                    node = self.snapshot.node(host).expect("captured host");
                    if let Some(&child) = node.children().first() {
                        self.next = Some(child);
                        break;
                    }
                } else {
                    if let Some(sibling) = node.next_sibling() {
                        self.next = Some(sibling);
                        break;
                    }
                    let Some(parent) = node.parent() else {
                        break;
                    };
                    node = self.snapshot.node(parent).expect("captured parent");
                }
            }
        }
        Some(current)
    }
}
impl std::iter::FusedIterator for DomTreeOrder<'_> {}

impl DomSnapshot {
    pub fn shadow_root(&self, id: NodeId) -> Result<DomShadowRoot<'_>, DomError> {
        self.node(id)?
            .as_shadow_root()
            .ok_or(DomError::WrongNodeKind)
    }
}

impl DomSnapshot {
    pub fn view_transition(
        &self,
        id: crate::DomViewTransitionId,
    ) -> Result<crate::DomViewTransition<'_>, DomError> {
        let value = self.state.view_transitions.transition(self.owner, id)?;
        Ok(crate::DomViewTransition {
            snapshot: self,
            value,
        })
    }
    pub fn view_transition_record(
        &self,
        id: crate::DomViewTransitionRecordId,
    ) -> Result<crate::DomViewTransitionRecord<'_>, DomError> {
        let transition = self.view_transition(id.transition())?;
        let value = transition
            .value
            .records
            .iter()
            .find(|r| r.id == id)
            .ok_or(DomError::StaleViewTransitionRecord)?;
        Ok(crate::DomViewTransitionRecord { transition, value })
    }
    /// Complete relation reads require this snapshot's actual document element.
    pub fn view_transition_pseudo(
        &self,
        id: crate::DomViewTransitionPseudoId,
    ) -> Result<crate::DomViewTransitionPseudo<'_>, DomError> {
        let (value, record) = self.state.view_transitions.pseudo(self.owner, id)?;
        let origin = self
            .document(value.document)?
            .document_element()
            .ok_or(DomError::ViewTransitionOriginUnavailable)?;
        Ok(crate::DomViewTransitionPseudo {
            transition: crate::DomViewTransition {
                snapshot: self,
                value,
            },
            record,
            id,
            origin,
        })
    }
    /// Identity/flag reads remain available without an originating element.
    pub fn view_transition_pseudo_captured(
        &self,
        id: crate::DomViewTransitionPseudoId,
    ) -> Result<bool, DomError> {
        let (transition, record) = self.state.view_transitions.pseudo(self.owner, id)?;
        Ok(record.map_or(transition.root_captured, |r| {
            r.captured[crate::view_transition::flag_index(id.kind())]
        }))
    }
}
impl<'a> DomDocument<'a> {
    pub fn document_element(self) -> Option<NodeId> {
        self.snapshot
            .node(self.id.node())
            .expect("captured Document")
            .element_children()
            .next()
    }
    /// None means feature-uninitialized, not a source null stylesheet default.
    pub fn view_transitions(self) -> Option<&'a crate::DomDocumentViewTransitions> {
        self.snapshot
            .state
            .view_transitions
            .documents
            .iter()
            .find(|d| d.document == self.id)
    }
}
