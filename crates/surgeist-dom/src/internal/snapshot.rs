use std::sync::Arc;

use crate::internal::store::{Node, State};
use crate::{
    internal::document::DomDocumentMetadata, internal::element::DomElement,
    internal::node_values::DomDocumentType, internal::node_values::DomProcessingInstruction,
};
use crate::{
    internal::identity::DocumentId, internal::identity::DomOwnerId,
    internal::identity::DomRevision, internal::identity::DomStamp, internal::identity::NodeId,
    internal::store::DomError,
};
use surgeist_html::values::DocumentString;

mod document_title;
mod editability;
mod equality;
mod namespace;
mod option_text;
mod position;
mod tabindex;
mod translation;
pub use namespace::DomNamespaceUri;
pub use position::DomDocumentPosition;

enum Descendant<Value> {
    Descend,
    Prune,
    Found(Value),
}

/// A borrowed view of a node's immutable payload in one captured snapshot.
/// Copying this view preserves its snapshot lifetime and creates no node.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DomNodeKind<'a> {
    Document(&'a DomDocumentMetadata),
    Element(&'a DomElement),
    Attr(&'a crate::internal::name::DomAttribute),
    DocumentFragment,
    ShadowRoot(&'a crate::internal::shadow::DomShadowRootState),
    Text(&'a DocumentString),
    CdataSection(&'a DocumentString),
    Comment(&'a DocumentString),
    ProcessingInstruction(&'a DomProcessingInstruction),
    DocumentType(&'a DomDocumentType),
}

impl<'a> DomNodeKind<'a> {
    pub(crate) fn text_data(self) -> Option<&'a DocumentString> {
        match self {
            Self::Text(data) | Self::CdataSection(data) => Some(data),
            _ => None,
        }
    }

    /// The NodeFilter bit selected by the DOM node type minus one.
    pub(crate) const fn traversal_mask(self) -> u32 {
        1 << match self {
            Self::Element(_) => 0,
            Self::Attr(_) => 1,
            Self::Text(_) => 2,
            Self::CdataSection(_) => 3,
            Self::ProcessingInstruction(_) => 6,
            Self::Comment(_) => 7,
            Self::Document(_) => 8,
            Self::DocumentType(_) => 9,
            Self::DocumentFragment | Self::ShadowRoot(_) => 10,
        }
    }
}

/// Owned immutable state, including detached nodes and all documents.
#[derive(Clone, Debug)]
pub struct DomSnapshot {
    pub(crate) source_tail: Option<crate::internal::source::DomSourceOperationId>,
    pub(crate) owner: DomOwnerId,
    pub(crate) revision: DomRevision,
    pub(crate) state: Arc<State>,
    pub(crate) script_registration: Option<crate::internal::hosted::ScriptRegistrationKey>,
}

impl DomSnapshot {
    pub fn range(
        &self,
        id: crate::internal::range::DomRangeId,
    ) -> Result<crate::internal::range::DomRange<'_>, DomError> {
        if id.owner != self.owner {
            return Err(DomError::ForeignOwner);
        }
        let record = self
            .state
            .ranges
            .get(id.slot)
            .filter(|slot| slot.generation == id.generation)
            .and_then(|slot| slot.record.as_ref())
            .ok_or(DomError::StaleRange)?;
        Ok(crate::internal::range::DomRange { id, record })
    }
    pub fn incomplete_hosted_operation(
        &self,
    ) -> Option<crate::internal::hosted::DomHostedOperationId> {
        self.state.incomplete_hosted
    }
    pub fn incomplete_source_operation(
        &self,
    ) -> Option<crate::internal::source::DomSourceOperationId> {
        self.source_tail.or(self.state.incomplete_source)
    }
    pub(crate) fn require_complete_source(&self) -> Result<(), DomError> {
        if self.incomplete_source_operation().is_some() || self.state.incomplete_hosted.is_some() {
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
        self.node_with(id, || Ok(()))
    }
    /// Read an exact node and admit the canonical lookup and its visited pages.
    pub fn node_with<E: From<DomError>>(
        &self,
        id: NodeId,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<DomNode<'_>, E> {
        let node = self.state.node_with(self.owner, id, &mut step)?;
        Ok(DomNode {
            snapshot: self,
            id,
            node,
        })
    }

    /// Retarget a node against an invocation node, or against a non-Node target
    /// when `against` is absent. Every topology lookup uses the supplied meter.
    pub fn retarget_node_with<E: From<DomError>>(
        &self,
        source: NodeId,
        against: Option<NodeId>,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<NodeId, E> {
        let mut candidate = self.node_with(source, &mut step)?;
        loop {
            let root = candidate.root_with(false, &mut step)?;
            let Some(shadow) = root.as_shadow_root() else {
                return Ok(candidate.id());
            };
            if let Some(against) = against
                && root.is_shadow_including_inclusive_ancestor_with(against, &mut step)?
            {
                return Ok(candidate.id());
            }
            candidate = self.node_with(shadow.host(), &mut step)?;
        }
    }
    pub fn document(&self, id: DocumentId) -> Result<DomDocument<'_>, DomError> {
        if !matches!(self.node(id.node())?.kind(), DomNodeKind::Document(_)) {
            return Err(DomError::WrongNodeKind);
        }
        Ok(DomDocument { snapshot: self, id })
    }
    pub fn attr(&self, id: NodeId) -> Result<crate::internal::attr::DomAttr<'_>, DomError> {
        self.node(id)?.as_attr().ok_or(DomError::WrongNodeKind)
    }

    /// Find the first Element child of the exact captured Document.
    ///
    /// Admit the bank entry and each actual lookup page for the Document and
    /// each inspected child. Return the caller's first admission failure.
    pub fn document_element_with<E: From<DomError>>(
        &self,
        id: DocumentId,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<NodeId>, E> {
        let document = self.state.node_with(self.owner, id.node(), &mut step)?;
        if !matches!(document.kind(), DomNodeKind::Document(_)) {
            return Err(DomError::WrongNodeKind.into());
        }
        let document = DomNode {
            snapshot: self,
            id: id.node(),
            node: document,
        };
        Ok(document.first_element_child_with(step)?.map(DomNode::id))
    }

    /// Every managed node, including detached nodes. Arena order is unspecified.
    pub fn nodes(&self) -> impl Iterator<Item = NodeId> + '_ {
        self.state.slots.iter().filter_map(|(slot, value)| {
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
    /// Scripting fact available from the captured native profile. A captured
    /// engine registration needs its own qualified policy observation.
    pub fn scripting(self) -> surgeist_html_semantics::HtmlScriptingStatus {
        use surgeist_html_semantics::HtmlScriptingStatus;
        if self
            .snapshot
            .script_registration
            .is_some_and(|registration| registration.document == self.id)
        {
            HtmlScriptingStatus::Unavailable
        } else {
            HtmlScriptingStatus::Disabled
        }
    }

    /// The retained designation at this exact Document revision. Native fresh
    /// construction establishes known null, as does transferred original
    /// controlled HTML constructor proof. Imported history without a producer
    /// proof stays unavailable. Neither value supplies a current-area resolver,
    /// viewport, system-focus observation or execution authority. Reads preserve
    /// the captured value across later inactivity, metadata and tree changes.
    pub fn focused_area_designation(self) -> crate::internal::document::DomFocusedAreaDesignation {
        self.snapshot
            .state
            .node(self.snapshot.owner, self.id.0)
            .expect("captured Document")
            .focused_area_designation()
    }
    pub fn active_range(self) -> crate::internal::range::DomActiveRangeState {
        self.snapshot
            .state
            .node(self.snapshot.owner, self.id.0)
            .expect("captured Document")
            .active_range()
    }
    pub fn id(self) -> DocumentId {
        self.id
    }
    /// The actual navigable association captured in this canonical snapshot.
    /// Current execution additionally qualifies it through the owning Dom.
    pub fn navigable(self) -> Option<crate::internal::hosted::DomHostedNavigableId> {
        match self
            .metadata()
            .document_environment()
            .hosted
            .as_ref()?
            .phase
        {
            crate::internal::hosted::DocumentPhase::InitialEntry { navigable, .. } => {
                Some(navigable)
            }
            _ => None,
        }
    }
    /// Activity derived from this immutable snapshot's actual published
    /// association. Later issuer retirement never changes this historical read.
    pub fn is_fully_active(self) -> Result<bool, crate::internal::hosted::DomHostedError> {
        self.snapshot.require_complete_source()?;
        match &self.metadata().document_environment().hosted {
            Some(association) => {
                match association.phase {
                    crate::internal::hosted::DocumentPhase::InitialEntry { navigable, .. } => {
                        let graph =
                            self.snapshot.state.hosted_history.as_deref().ok_or(
                                crate::internal::hosted::DomHostedError::UnsupportedProfile,
                            )?;
                        crate::internal::store::hosted::captured_fully_active(
                            graph,
                            self.id,
                            navigable,
                            || Ok(()),
                        )
                    }
                    crate::internal::hosted::DocumentPhase::Retired { .. } => Ok(false),
                    _ => Err(crate::internal::hosted::DomHostedError::IncompleteOperation),
                }
            }
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
        if node.node.document() != self.id {
            return Err(DomError::WrongDocument);
        }
        Ok(node)
    }
}

/// Exact store, document and revision qualification.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomDocumentStamp {
    pub(crate) store: DomStamp,
    pub(crate) document: DocumentId,
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

/// The exact qualified tag spelling sampled from a node's current Document.
/// This owns shared name handles and allocates no assembled UTF-16 buffer.
#[derive(Clone, Debug)]
pub struct DomTagName {
    prefix: Option<DocumentString>,
    local: DocumentString,
    uppercase: bool,
    length: usize,
}

impl DomTagName {
    pub const fn code_unit_len(&self) -> usize {
        self.length
    }

    /// Borrow the original units when no prefix or case transformation is due.
    pub fn contiguous_units(&self) -> Option<&[u16]> {
        (self.prefix.is_none() && !self.uppercase).then(|| self.local.code_units())
    }

    /// Iterate the exact presentation without allocating. Consumers own
    /// admission for the unit traversal and any destination allocation.
    pub fn code_units(&self) -> impl Iterator<Item = u16> + '_ {
        self.prefix
            .iter()
            .flat_map(|prefix| prefix.code_units().iter().copied().chain([0x3a]))
            .chain(self.local.code_units().iter().copied())
            .map(|unit| {
                if self.uppercase && (0x61..=0x7a).contains(&unit) {
                    unit - 0x20
                } else {
                    unit
                }
            })
    }
}

impl<'a> DomNode<'a> {
    pub fn as_form(self) -> Option<crate::internal::form::DomForm<'a>> {
        self.as_element()
            .and_then(DomElement::form_state)
            .map(|_| crate::internal::form::DomForm { node: self })
    }
    pub fn as_fieldset(self) -> Option<crate::internal::fieldset::DomFieldset<'a>> {
        self.as_element()
            .and_then(DomElement::fieldset_state)
            .map(|_| crate::internal::fieldset::DomFieldset { node: self })
    }
    /// Read the stored Form owner of this node's immediate ordinary Fieldset parent.
    ///
    /// Other parents and missing associations yield `None`. The result belongs
    /// to this published snapshot, including during an unfinished Source edit.
    /// A live caller qualifies its access to that segment. `step` admits the
    /// parent relation, Fieldset classification, stored association and reached lookups.
    ///
    /// # Errors
    ///
    /// Returns the first admission or lookup failure, or
    /// [`DomError::WrongNodeKind`] if a stored owner is not a native Form.
    pub fn fieldset_parent_form_owner_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<crate::internal::form::DomForm<'a>>, E> {
        step()?;
        let Some(parent) = self.ordinary_parent_element(&mut step)? else {
            return Ok(None);
        };
        step()?;
        let Some(fieldset) = parent.as_fieldset() else {
            return Ok(None);
        };
        fieldset.node.stored_form_owner_with(step)
    }

    pub(crate) fn stored_form_owner_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<crate::internal::form::DomForm<'a>>, E> {
        step()?;
        self.node
            .control_owner()
            .map(|owner| {
                self.snapshot
                    .node_with(owner, &mut step)?
                    .as_form()
                    .ok_or_else(|| DomError::WrongNodeKind.into())
            })
            .transpose()
    }

    pub fn as_button(self) -> Option<crate::internal::button::DomButton<'a>> {
        self.as_element()
            .and_then(DomElement::button_state)
            .map(|_| crate::internal::button::DomButton { node: self })
    }
    pub fn as_input(self) -> Option<crate::internal::input::DomInput<'a>> {
        self.as_element()?.input_state()?;
        Some(crate::internal::input::DomInput { node: self })
    }
    pub fn as_details(self) -> Option<crate::internal::details::DomDetails<'a>> {
        self.as_element()?
            .is_details()
            .then_some(crate::internal::details::DomDetails { node: self })
    }
    pub fn as_select(self) -> Option<crate::internal::select::DomSelect<'a>> {
        self.as_element()?.select_state()?;
        Some(crate::internal::select::DomSelect { node: self })
    }
    pub fn as_option(self) -> Option<crate::internal::select::DomOption<'a>> {
        self.as_element()?.option_state()?;
        Some(crate::internal::select::DomOption { node: self })
    }
    pub fn as_selectedcontent(self) -> Option<crate::internal::select::DomSelectedcontent<'a>> {
        self.as_element()?.selectedcontent_disabled()?;
        Some(crate::internal::select::DomSelectedcontent { node: self })
    }
    pub fn as_output(self) -> Option<crate::internal::output::DomOutput<'a>> {
        self.as_element()?.output_state()?;
        Some(crate::internal::output::DomOutput { node: self })
    }
    pub fn as_textarea(self) -> Option<crate::internal::textarea::DomTextarea<'a>> {
        self.as_element()?.textarea_state()?;
        Some(crate::internal::textarea::DomTextarea { node: self })
    }

    /// Privileged canonical ownership, including closed roots. This does not
    /// implement the source's open-only Element.shadowRoot presentation.
    pub fn hosted_shadow_root(self) -> Option<NodeId> {
        self.node.shadow_root()
    }
    /// Qualify the host's existing root for matching declarative reuse.
    /// A host without a root returns `None`. A mode mismatch or a
    /// nondeclarative root returns `NotSupported`.
    pub fn reusable_shadow_root_with<E: From<DomError>>(
        self,
        mode: surgeist_html::construction::HtmlShadowRootMode,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<NodeId>, E> {
        step()?;
        if self.as_element().is_none() {
            return Err(DomError::WrongNodeKind.into());
        }
        let Some(root) = self.hosted_shadow_root() else {
            return Ok(None);
        };
        let node = self.snapshot.node_with(root, step)?;
        let DomNodeKind::ShadowRoot(state) = node.kind() else {
            return Err(DomError::WrongNodeKind.into());
        };
        state.require_declarative_reuse(mode)?;
        Ok(Some(root))
    }
    /// Source Element.shadowRoot presentation: a closed root is not exposed.
    pub fn shadow_root(self) -> Option<NodeId> {
        self.shadow_root_with(|| Ok::<(), DomError>(()))
            .unwrap_or_default()
    }
    /// Read the open-only shadow root with work admission.
    ///
    /// Admit the host relation and the canonical lookup of any root whose
    /// mode determines visibility. Return the first admission failure.
    pub fn shadow_root_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<NodeId>, E> {
        step()?;
        let Some(root) = self.hosted_shadow_root() else {
            return Ok(None);
        };
        let node = self.snapshot.node_with(root, step)?;
        let DomNodeKind::ShadowRoot(state) = node.kind() else {
            return Err(DomError::WrongNodeKind.into());
        };
        Ok((state.mode() == surgeist_html::construction::HtmlShadowRootMode::Open).then_some(root))
    }
    /// The exact ShadowRoot's host. Template contents instead use fragment_host().
    pub fn shadow_host(self) -> Option<NodeId> {
        matches!(self.kind(), DomNodeKind::ShadowRoot(_))
            .then_some(self.node.host())
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
        self.parent().or(self.node.host())
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
        self.is_shadow_including_inclusive_ancestor_with(other, || Ok(()))
    }
    fn is_shadow_including_inclusive_ancestor_with<E: From<DomError>>(
        self,
        other: NodeId,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        let mut next = Some(other);
        while let Some(id) = next {
            let node = self.snapshot.node_with(id, &mut step)?;
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
    /// Test ordinary inclusive ancestry in this snapshot. The other node and
    /// each visited ancestor are resolved through the supplied meter.
    pub fn contains_with<E: From<DomError>>(
        self,
        other: NodeId,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        let mut candidate = self.snapshot.node_with(other, &mut step)?;
        loop {
            if candidate.id == self.id {
                return Ok(true);
            }
            let Some(parent) = candidate.parent() else {
                return Ok(false);
            };
            candidate = self.snapshot.node_with(parent, &mut step)?;
        }
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
    /// Visit this Text and its contiguous Text siblings in tree order, including CDATA sections.
    /// Each inspected sibling and canonical lookup is admitted through `step`.
    pub fn visit_contiguous_text_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
        mut visit: impl FnMut(&'a DocumentString) -> Result<(), E>,
    ) -> Result<(), E> {
        let Some(value) = self.kind().text_data() else {
            return Err(DomError::WrongNodeKind.into());
        };
        let Some(parent) = self.parent() else {
            return visit(value);
        };
        let parent = self.snapshot.node_with(parent, &mut step)?;
        let children = parent.children();
        let mut siblings = children.iter().enumerate();
        let index = loop {
            let (index, child) = siblings.next().ok_or(DomError::NotFound)?;
            step()?;
            if *child == self.id {
                break index;
            }
        };
        let mut start = index;
        for &sibling in children[..index].iter().rev() {
            step()?;
            let sibling = self.snapshot.node_with(sibling, &mut step)?;
            if sibling.kind().text_data().is_none() {
                break;
            }
            start -= 1;
        }
        for &sibling in &children[start..index] {
            step()?;
            let sibling = self.snapshot.node_with(sibling, &mut step)?;
            let Some(text) = sibling.kind().text_data() else {
                return Err(DomError::WrongNodeKind.into());
            };
            visit(text)?;
        }
        visit(value)?;
        for (_, &sibling) in siblings {
            step()?;
            let sibling = self.snapshot.node_with(sibling, &mut step)?;
            let Some(text) = sibling.kind().text_data() else {
                break;
            };
            visit(text)?;
        }
        Ok(())
    }

    /// Visit Text and CDATASection descendants on ordinary child edges in tree order.
    /// Shadow trees and Template contents have separate ownership edges.
    ///
    /// Admit each child edge and canonical lookup page through `step`. The
    /// caller owns any work it does with the borrowed Text. `reserve` receives
    /// the full byte extent of each new scratch allocation, with the previous
    /// allocation still retained. Its returned backing lives with the scratch.
    /// The first failed step, reservation, lookup, or visit ends traversal.
    pub fn visit_descendant_text_with<E: From<DomError>, Backing>(
        self,
        step: impl FnMut() -> Result<(), E>,
        reserve: impl FnMut(usize) -> Result<Backing, E>,
        mut visit: impl FnMut(&'a DocumentString) -> Result<(), E>,
    ) -> Result<(), E> {
        self.find_descendant_with(step, reserve, |node, _| {
            if let Some(text) = node.kind().text_data() {
                visit(text)?;
            }
            Ok(Descendant::<()>::Descend)
        })?;
        Ok(())
    }

    fn find_descendant_with<E: From<DomError>, Backing, Step, Value>(
        self,
        mut step: Step,
        mut reserve: impl FnMut(usize) -> Result<Backing, E>,
        mut inspect: impl FnMut(Self, &mut Step) -> Result<Descendant<Value>, E>,
    ) -> Result<Option<Value>, E>
    where
        Step: FnMut() -> Result<(), E>,
    {
        let mut remaining = self.children();
        // Keep the allocation before its backing so unwind releases it first.
        let mut pending = (Vec::<&[NodeId]>::new(), None);
        loop {
            let Some((&child, siblings)) = remaining.split_first() else {
                let Some(siblings) = pending.0.pop() else {
                    return Ok(None);
                };
                remaining = siblings;
                continue;
            };
            step()?;
            let child = self.snapshot.node_with(child, &mut step)?;
            match inspect(child, &mut step)? {
                Descendant::Descend => remaining = child.children(),
                Descendant::Prune => remaining = &[],
                Descendant::Found(value) => return Ok(Some(value)),
            }
            if remaining.is_empty() {
                remaining = siblings;
            } else if !siblings.is_empty() {
                if pending.0.len() == pending.0.capacity() {
                    let capacity = pending
                        .0
                        .capacity()
                        .checked_mul(2)
                        .ok_or(DomError::RepresentationExhausted)?
                        .max(1);
                    let bytes = capacity
                        .checked_mul(size_of::<&[NodeId]>())
                        .ok_or(DomError::RepresentationExhausted)?;
                    let backing = reserve(bytes)?;
                    let mut replacement = Vec::new();
                    replacement
                        .try_reserve_exact(capacity)
                        .map_err(|_| DomError::AllocationFailed)?;
                    if replacement.capacity() > capacity {
                        return Err(DomError::AllocationFailed.into());
                    }
                    for &frame in &pending.0 {
                        step()?;
                        replacement.push(frame);
                    }
                    pending = (replacement, Some(backing));
                }
                pending.0.push(siblings);
            }
        }
    }

    /// Visit direct Text children in ordinary tree order, admitting every
    /// child lookup before inspecting its kind and returning the first refusal.
    pub fn visit_child_text_with<E: From<DomError>>(
        self,
        step: impl FnMut() -> Result<(), E>,
        mut visit: impl FnMut(&'a DocumentString) -> Result<(), E>,
    ) -> Result<(), E> {
        self.visit_child_text_admitted(step, |value, _| visit(value))
    }

    fn visit_child_text_admitted<E: From<DomError>, Step>(
        self,
        mut step: Step,
        mut visit: impl FnMut(&'a DocumentString, &mut Step) -> Result<(), E>,
    ) -> Result<(), E>
    where
        Step: FnMut() -> Result<(), E>,
    {
        for &id in self.children() {
            let child = self.snapshot.node_with(id, &mut step)?;
            if let Some(value) = child.kind().text_data() {
                visit(value, &mut step)?;
            }
        }
        Ok(())
    }
    /// Visit matching custom-element definition candidates in shadow-including
    /// preorder, including closed roots. Each retained traversal allocation
    /// uses the caller's scratch admission and each visited node is metered.
    pub fn visit_custom_element_upgrade_candidates_with<E: From<DomError>, Backing>(
        self,
        (registry, definition): (
            crate::internal::registry::DomRegistryId,
            &surgeist_html_semantics::HtmlCustomElementDefinition,
        ),
        mut step: impl FnMut() -> Result<(), E>,
        mut reserve: impl FnMut(usize) -> Result<Backing, E>,
        mut visit: impl FnMut(NodeId) -> Result<(), E>,
    ) -> Result<(), E> {
        if !matches!(self.kind(), DomNodeKind::Document(_)) {
            return Err(DomError::WrongNodeKind.into());
        }
        if registry.owner() != self.snapshot.owner() {
            return Err(DomError::ForeignOwner.into());
        }
        let customized = !crate::internal::names::same_code_units(
            definition.name().code_units(),
            definition.local_name().code_units(),
            &mut step,
        )?;
        let mut siblings = self.children();
        let mut shadow = None;
        let mut pending = (Vec::<&[NodeId]>::new(), None);
        loop {
            let id = if let Some(root) = shadow.take() {
                root
            } else if let Some((&id, remaining)) = siblings.split_first() {
                siblings = remaining;
                id
            } else if let Some(remaining) = pending.0.pop() {
                step()?;
                siblings = remaining;
                continue;
            } else {
                return Ok(());
            };
            step()?;
            let node = self.snapshot.node_with(id, &mut step)?;
            if let Some(element) = node.as_element()
                && element.custom_element_registry()
                    == crate::internal::document::DomRegistryBinding::Registry(registry)
                && crate::internal::names::in_html_namespace_with(element.name(), &mut step)?
                && crate::internal::names::same_code_units(
                    element.name().local_name().code_units(),
                    definition.local_name().code_units(),
                    &mut step,
                )?
                && (!customized
                    || match element.saved_is() {
                        Some(saved) => crate::internal::names::same_code_units(
                            saved.code_units(),
                            definition.name().code_units(),
                            &mut step,
                        )?,
                        None => false,
                    })
            {
                visit(id)?;
            }
            let children = node.children();
            shadow = node.hosted_shadow_root();
            if children.is_empty() && shadow.is_none() {
                continue;
            }
            if !siblings.is_empty() {
                step()?;
                if pending.0.len() == pending.0.capacity() {
                    let capacity = pending
                        .0
                        .capacity()
                        .checked_mul(2)
                        .ok_or(DomError::RepresentationExhausted)?
                        .max(1);
                    let bytes = capacity
                        .checked_mul(size_of::<&[NodeId]>())
                        .ok_or(DomError::RepresentationExhausted)?;
                    let backing = reserve(bytes)?;
                    let mut replacement = Vec::new();
                    replacement
                        .try_reserve_exact(capacity)
                        .map_err(|_| DomError::AllocationFailed)?;
                    if replacement.capacity() > capacity {
                        return Err(DomError::AllocationFailed.into());
                    }
                    for &frame in &pending.0 {
                        step()?;
                        replacement.push(frame);
                    }
                    pending = (replacement, Some(backing));
                }
                pending.0.push(siblings);
            }
            siblings = children;
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
    /// The authentic historical Global of this already qualified node.
    pub fn relevant_global(
        self,
    ) -> Result<
        Option<crate::internal::hosted::DomHostedGlobalId>,
        crate::internal::hosted::DomHostedError,
    > {
        let global = self.node.relevant_global();
        if global.is_some_and(|id| id.source.owner != self.snapshot.owner) {
            return Err(crate::internal::hosted::DomHostedError::ForeignSource);
        }
        Ok(global)
    }
    pub fn kind(self) -> DomNodeKind<'a> {
        self.node.kind()
    }
    /// DOM nodeName readback; PI targets and doctype names preserve exact units.
    pub fn node_name(self) -> DocumentString {
        match self.kind() {
            DomNodeKind::Document(_) => "#document".into(),
            DomNodeKind::Element(_) => self.tag_name().expect("Element tag name"),
            DomNodeKind::Attr(value) => value.name().qualified_spelling(),
            DomNodeKind::DocumentFragment | DomNodeKind::ShadowRoot(_) => {
                "#document-fragment".into()
            }
            DomNodeKind::Text(_) => "#text".into(),
            DomNodeKind::CdataSection(_) => "#cdata-section".into(),
            DomNodeKind::Comment(_) => "#comment".into(),
            DomNodeKind::ProcessingInstruction(value) => value.target().clone(),
            DomNodeKind::DocumentType(value) => value.name().clone(),
        }
    }
    /// Checked document view, including for documents discovered by enumeration.
    pub fn as_document(self) -> Option<DomDocument<'a>> {
        matches!(self.kind(), DomNodeKind::Document(_)).then_some(DomDocument {
            snapshot: self.snapshot,
            id: DocumentId(self.id),
        })
    }
    pub fn as_element(self) -> Option<&'a DomElement> {
        self.node.element()
    }
    pub fn as_attr(self) -> Option<crate::internal::attr::DomAttr<'a>> {
        let (data, owner) = self.node.attribute_node()?;
        Some(crate::internal::attr::DomAttr {
            id: crate::internal::identity::DomAttrId(self.id),
            document: self.node.document(),
            owner,
            data,
        })
    }
    /// Select the exact tag spelling without copying its code units. Any
    /// lookup of the current node-document pays the visited native pages.
    pub fn tag_name_spelling_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<DomTagName>, E> {
        let Some(element) = self.as_element() else {
            return Ok(None);
        };
        let name = element.name();
        let prefix = name.prefix();
        let local = name.local_name();
        let mut length = local.len();
        if let Some(prefix) = prefix {
            length = length
                .checked_add(prefix.len())
                .and_then(|length| length.checked_add(1))
                .ok_or(DomError::RepresentationExhausted)?;
        }
        let uppercase = if crate::internal::names::in_html_namespace(name) {
            let document = self
                .snapshot
                .node_with(self.node.document().node(), &mut step)?;
            let DomNodeKind::Document(metadata) = document.kind() else {
                return Err(DomError::WrongNodeKind.into());
            };
            metadata.class() == crate::internal::document::DomDocumentClass::Html
        } else {
            false
        };
        Ok(Some(DomTagName {
            prefix: prefix.cloned(),
            local: local.clone(),
            uppercase,
            length,
        }))
    }

    /// Source tagName follows the actual current node-document. Raw namespace,
    /// prefix and local name remain unchanged by this presentation or adoption.
    pub fn tag_name(self) -> Option<DocumentString> {
        let spelling = self
            .tag_name_spelling_with(|| Ok::<_, DomError>(()))
            .ok()??;
        Some(if spelling.contiguous_units().is_some() {
            spelling.local
        } else {
            DocumentString::from_utf16(spelling.code_units().collect::<Vec<_>>())
        })
    }
    pub(crate) fn html_element_in_html_document_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        let Some(element) = self.as_element() else {
            return Ok(false);
        };
        if !crate::internal::names::in_html_namespace_with(element.name(), &mut step)? {
            return Ok(false);
        }
        let document = self.snapshot.node_with(self.node.document().node(), step)?;
        let DomNodeKind::Document(metadata) = document.kind() else {
            return Err(DomError::WrongNodeKind.into());
        };
        Ok(metadata.class() == crate::internal::document::DomDocumentClass::Html)
    }
    pub fn attributes(self) -> Option<crate::internal::attr::DomNamedNodeMap<'a>> {
        self.as_element()?;
        Some(crate::internal::attr::DomNamedNodeMap { element: self })
    }
    pub fn get_attribute_node(
        self,
        qualified_name: &DocumentString,
    ) -> Option<crate::internal::attr::DomAttr<'a>> {
        self.attributes()?.get_named_item(qualified_name)
    }
    pub fn get_attribute_node_ns(
        self,
        namespace: &surgeist_html::values::Namespace,
        local_name: &DocumentString,
    ) -> Option<crate::internal::attr::DomAttr<'a>> {
        self.attributes()?.get_named_item_ns(namespace, local_name)
    }
    /// DOM getAttribute lookup, including HTML input casing from this exact
    /// captured node-document.
    pub fn attribute(self, qualified_name: &DocumentString) -> Option<&'a DocumentString> {
        self.get_attribute_node(qualified_name)
            .map(crate::internal::attr::DomAttr::value)
    }
    /// Exact namespace/local-name lookup; HTML input casing does not apply.
    pub fn attribute_ns(
        self,
        namespace: &surgeist_html::values::Namespace,
        local_name: &DocumentString,
    ) -> Option<&'a DocumentString> {
        self.get_attribute_node_ns(namespace, local_name)
            .map(crate::internal::attr::DomAttr::value)
    }
    pub fn parent_element(self) -> Option<NodeId> {
        self.parent_element_with(|| Ok::<_, DomError>(()))
            .expect("captured parent")
    }
    /// Observe the ordinary parent only when it is an Element. Charge the
    /// reached parent lookup and return the original refusal.
    pub fn parent_element_with<E: From<DomError>>(
        self,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<NodeId>, E> {
        Ok(self.ordinary_parent_element(step)?.map(Self::id))
    }
    fn ordinary_parent_element<E: From<DomError>>(
        self,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        let Some(parent) = self.parent() else {
            return Ok(None);
        };
        let parent = self.snapshot.node_with(parent, step)?;
        Ok(parent.as_element().map(|_| parent))
    }
    pub fn element_children(self) -> impl DoubleEndedIterator<Item = NodeId> + 'a {
        self.node.children().iter().copied().filter(move |id| {
            self.snapshot
                .node(*id)
                .expect("captured child")
                .as_element()
                .is_some()
        })
    }
    pub fn previous_sibling(self) -> Option<NodeId> {
        self.sibling_with(DomSiblingDirection::Previous, || Ok::<_, DomError>(()))
            .expect("captured sibling")
    }
    pub fn next_sibling(self) -> Option<NodeId> {
        self.sibling_with(DomSiblingDirection::Next, || Ok::<_, DomError>(()))
            .expect("captured sibling")
    }
    /// Observe the adjacent ordinary sibling, admitting the parent lookup
    /// and each inspected child identity in tree order.
    pub fn sibling_with<E: From<DomError>>(
        self,
        direction: DomSiblingDirection,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<NodeId>, E> {
        let Some(parent) = self.parent() else {
            return Ok(None);
        };
        let parent = self.snapshot.node_with(parent, &mut step)?;
        let mut previous = None;
        let mut children = parent.children().iter().copied();
        for child in children.by_ref() {
            step()?;
            if child == self.id {
                return match direction {
                    DomSiblingDirection::Previous => Ok(previous),
                    DomSiblingDirection::Next => {
                        let next = children.next();
                        if next.is_some() {
                            step()?;
                        }
                        Ok(next)
                    }
                };
            }
            previous = Some(child);
        }
        Err(DomError::NotFound.into())
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
        (!matches!(self.kind(), DomNodeKind::Document(_))).then_some(self.node.document())
    }
    pub fn parent(self) -> Option<NodeId> {
        self.node.parent()
    }
    pub fn children(self) -> &'a [NodeId] {
        self.node.children()
    }
    fn first_element_child_with<E: From<DomError>>(
        self,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<Self>, E> {
        for &child in self.children() {
            let node = self.snapshot.node_with(child, &mut step)?;
            if node.as_element().is_some() {
                return Ok(Some(node));
            }
        }
        Ok(None)
    }
    pub fn data(self) -> Option<&'a DocumentString> {
        match self.kind() {
            DomNodeKind::Text(data)
            | DomNodeKind::CdataSection(data)
            | DomNodeKind::Comment(data) => Some(data),
            DomNodeKind::ProcessingInstruction(value) => Some(value.data()),
            _ => None,
        }
    }
    /// Ordinary tree root; a detached node's owner document is not its root.
    pub fn root(self) -> NodeId {
        let mut id = self.id;
        let mut node = self.node;
        while let Some(parent) = node.parent() {
            id = parent;
            node = self
                .snapshot
                .state
                .node(self.snapshot.owner, parent)
                .expect("snapshot parent invariant");
        }
        id
    }
    /// Return the root of the ordinary or shadow-including tree. Admit each
    /// exact ancestor lookup in order and stop at the first refusal.
    pub fn root_with<E: From<DomError>>(
        self,
        composed: bool,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Self, E> {
        let mut node = self;
        while let Some(parent) = if composed {
            node.shadow_including_parent()
        } else {
            node.parent()
        } {
            node = self.snapshot.node_with(parent, &mut step)?;
        }
        Ok(node)
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

/// Which ordinary sibling to observe from a captured node.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSiblingDirection {
    Previous,
    Next,
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
    pub fn state(self) -> &'a crate::internal::shadow::DomShadowRootState {
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
        id: crate::internal::view_transition::DomViewTransitionId,
    ) -> Result<crate::internal::view_transition::DomViewTransition<'_>, DomError> {
        let value = self.state.view_transitions.transition(self.owner, id)?;
        Ok(crate::internal::view_transition::DomViewTransition {
            snapshot: self,
            value,
        })
    }
    pub fn view_transition_record(
        &self,
        id: crate::internal::view_transition::DomViewTransitionRecordId,
    ) -> Result<crate::internal::view_transition::DomViewTransitionRecord<'_>, DomError> {
        let transition = self.view_transition(id.transition())?;
        let value = transition
            .value
            .records
            .get(usize::try_from(id.generation).map_err(|_| DomError::RepresentationExhausted)?)
            .filter(|r| r.id == id)
            .ok_or(DomError::StaleViewTransitionRecord)?;
        Ok(crate::internal::view_transition::DomViewTransitionRecord { transition, value })
    }
    /// Complete relation reads require this snapshot's actual document element.
    pub fn view_transition_pseudo(
        &self,
        id: crate::internal::view_transition::DomViewTransitionPseudoId,
    ) -> Result<crate::internal::view_transition::DomViewTransitionPseudo<'_>, DomError> {
        let (value, record) = self.state.view_transitions.pseudo(self.owner, id)?;
        let origin = self
            .document(value.document)?
            .document_element()
            .ok_or(DomError::ViewTransitionOriginUnavailable)?;
        Ok(crate::internal::view_transition::DomViewTransitionPseudo {
            transition: crate::internal::view_transition::DomViewTransition {
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
        id: crate::internal::view_transition::DomViewTransitionPseudoId,
    ) -> Result<bool, DomError> {
        let (transition, record) = self.state.view_transitions.pseudo(self.owner, id)?;
        Ok(record.map_or(transition.root_captured, |r| {
            r.captured[crate::internal::view_transition::flag_index(id.kind())]
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
    pub fn view_transitions(
        self,
    ) -> Option<&'a crate::internal::view_transition::DomDocumentViewTransitions> {
        self.snapshot.state.view_transitions.document(self.id)
    }
}
