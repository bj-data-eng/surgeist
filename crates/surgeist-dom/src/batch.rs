use std::sync::Arc;

use crate::DomDocumentStamp;
use crate::view_transition::*;
use crate::{DocumentId, DocumentString, DomSnapshot, Namespace, NodeId};
use surgeist_css::CssViewTransitionIdent;
use surgeist_cssom::{CssomSheetId, CssomSnapshot};

#[derive(Debug)]
pub(crate) struct BatchIdentity;

/// A batch-local creation ticket, resolved to a NodeId only after publication.
///
/// A failed batch cannot leak an identity that a later batch might publish.
#[derive(Clone, Debug)]
pub struct CreatedNode {
    pub(crate) owner: Arc<BatchIdentity>,
    pub(crate) index: usize,
}

/// A published node or a creation in the same batch.
#[derive(Clone, Debug)]
pub enum NodeOperand {
    Existing(NodeId),
    Created(CreatedNode),
}

impl From<NodeId> for NodeOperand {
    fn from(id: NodeId) -> Self {
        Self::Existing(id)
    }
}
impl From<DocumentId> for NodeOperand {
    fn from(id: DocumentId) -> Self {
        Self::Existing(id.node())
    }
}
impl From<&CreatedNode> for NodeOperand {
    fn from(id: &CreatedNode) -> Self {
        Self::Created(id.clone())
    }
}

#[derive(Debug)]
pub(crate) enum Creation {
    Document,
    Fragment,
    Element(DocumentString),
    ElementNs {
        namespace: Namespace,
        name: DocumentString,
    },
    Text(DocumentString),
    Comment(DocumentString),
    ProcessingInstruction {
        target: DocumentString,
        data: DocumentString,
    },
    DocumentType {
        name: DocumentString,
        public_id: DocumentString,
        system_id: DocumentString,
    },
}

#[derive(Debug)]
pub(crate) enum Command {
    CreateHtmlDocument {
        associated_document: NodeOperand,
        title: Option<DocumentString>,
    },
    AttachShadow {
        host: NodeOperand,
        init: crate::DomShadowRootInit,
    },
    ViewTransition {
        basis: DomDocumentStamp,
        edit: Edit,
    },
    Create {
        kind: Creation,
        document: Option<NodeOperand>,
    },
    Import {
        snapshot: DomSnapshot,
        source: NodeId,
        document: NodeOperand,
        subtree: bool,
    },
    Insert {
        node: NodeOperand,
        parent: NodeOperand,
        before: Option<NodeOperand>,
    },
    ReplaceChild {
        node: NodeOperand,
        parent: NodeOperand,
        child: NodeOperand,
    },
    Detach(NodeOperand),
    Adopt {
        node: NodeOperand,
        document: NodeOperand,
    },
    SetAttribute {
        node: NodeOperand,
        namespace: Option<Namespace>,
        name: DocumentString,
        value: DocumentString,
    },
    SetNonce {
        node: NodeOperand,
        value: DocumentString,
    },
    SetMarqueeRunning {
        node: NodeOperand,
        running: bool,
    },
    RemoveAttribute {
        node: NodeOperand,
        namespace: Option<Namespace>,
        name: DocumentString,
    },
    Destroy(NodeOperand),
    ReplaceData {
        node: NodeOperand,
        offset: usize,
        count: usize,
        data: DocumentString,
    },
}

/// Ordered edit requests. Its caller-owned operands are admitted at apply time.
///
/// Commands refer to creations through private tickets, never guessed arena IDs.
#[derive(Debug)]
pub struct DomBatch {
    pub(crate) owner: Arc<BatchIdentity>,
    pub(crate) commands: Vec<Command>,
    creations: usize,
    transitions: usize,
    transition_records: usize,
}

impl Default for DomBatch {
    fn default() -> Self {
        Self::new()
    }
}

impl DomBatch {
    pub fn new() -> Self {
        Self {
            owner: Arc::new(BatchIdentity),
            commands: Vec::new(),
            creations: 0,
            transitions: 0,
            transition_records: 0,
        }
    }
    /// Initializes structural VT slots from a real CSSOM sheet capture. The
    /// caller owns actual UA-origin placement/document installation evidence.
    pub fn initialize_view_transitions(
        &mut self,
        basis: DomDocumentStamp,
        snapshot: &CssomSnapshot,
        sheet: &CssomSheetId,
    ) {
        self.commands.push(Command::ViewTransition {
            basis,
            edit: Edit::Initialize {
                snapshot: snapshot.clone(),
                sheet: sheet.clone(),
            },
        });
    }
    /// Every VT operation requires this exact pre-batch published document basis.
    pub fn begin_view_transition(&mut self, basis: DomDocumentStamp) -> CreatedViewTransition {
        let ticket = CreatedViewTransition {
            owner: self.owner.clone(),
            index: self.transitions,
        };
        self.transitions += 1;
        self.commands.push(Command::ViewTransition {
            basis,
            edit: Edit::Begin {
                ticket: ticket.index,
            },
        });
        ticket
    }
    pub fn append_view_transition_record(
        &mut self,
        basis: DomDocumentStamp,
        transition: impl Into<ViewTransitionOperand>,
        name: CssViewTransitionIdent,
    ) -> CreatedViewTransitionRecord {
        let ticket = CreatedViewTransitionRecord {
            owner: self.owner.clone(),
            index: self.transition_records,
        };
        self.transition_records += 1;
        self.commands.push(Command::ViewTransition {
            basis,
            edit: Edit::Append {
                transition: transition.into(),
                name,
                ticket: ticket.index,
            },
        });
        ticket
    }
    pub fn set_view_transition_branches(
        &mut self,
        basis: DomDocumentStamp,
        record: impl Into<ViewTransitionRecordOperand>,
        branches: DomViewTransitionBranches,
    ) {
        self.commands.push(Command::ViewTransition {
            basis,
            edit: Edit::Branches {
                record: record.into(),
                branches,
            },
        });
    }
    pub fn set_view_transition_rules(
        &mut self,
        basis: DomDocumentStamp,
        record: impl Into<ViewTransitionRecordOperand>,
        snapshot: &CssomSnapshot,
        rules: DomViewTransitionRuleAssociations,
    ) {
        self.commands.push(Command::ViewTransition {
            basis,
            edit: Edit::Rules {
                record: record.into(),
                snapshot: snapshot.clone(),
                rules,
            },
        });
    }
    /// Clears only canonical association records; makes no CSSOM removal claim.
    pub fn clear_view_transition_rules(
        &mut self,
        basis: DomDocumentStamp,
        record: impl Into<ViewTransitionRecordOperand>,
    ) {
        self.commands.push(Command::ViewTransition {
            basis,
            edit: Edit::ClearRules(record.into()),
        });
    }
    pub fn set_view_transition_captured(
        &mut self,
        basis: DomDocumentStamp,
        subject: DomViewTransitionSubject,
        captured: bool,
    ) {
        self.commands.push(Command::ViewTransition {
            basis,
            edit: Edit::Captured { subject, captured },
        });
    }
    pub fn set_view_transition_rendering_suppression(
        &mut self,
        basis: DomDocumentStamp,
        suppressed: bool,
    ) {
        self.commands.push(Command::ViewTransition {
            basis,
            edit: Edit::Suppression(suppressed),
        });
    }
    pub fn setup_view_transition_pseudos(
        &mut self,
        basis: DomDocumentStamp,
        transition: impl Into<ViewTransitionOperand>,
    ) {
        self.commands.push(Command::ViewTransition {
            basis,
            edit: Edit::Setup(transition.into()),
        });
    }
    /// Clears active/show/new-element flags, retaining suppression and records.
    pub fn clear_view_transition_state(
        &mut self,
        basis: DomDocumentStamp,
        transition: impl Into<ViewTransitionOperand>,
    ) {
        self.commands.push(Command::ViewTransition {
            basis,
            edit: Edit::Clear(transition.into()),
        });
    }
    /// Releases an inactive transition and its records/subjects, never CSSOM rules.
    pub fn release_view_transition(
        &mut self,
        basis: DomDocumentStamp,
        transition: impl Into<ViewTransitionOperand>,
    ) {
        self.commands.push(Command::ViewTransition {
            basis,
            edit: Edit::Release(transition.into()),
        });
    }
    fn ticket(&mut self) -> CreatedNode {
        let ticket = CreatedNode {
            owner: self.owner.clone(),
            index: self.creations,
        };
        self.creations += 1;
        ticket
    }
    fn create(&mut self, kind: Creation, document: Option<NodeOperand>) -> CreatedNode {
        let ticket = self.ticket();
        self.commands.push(Command::Create { kind, document });
        ticket
    }
    /// Clones a captured non-Document node into the target Document, with fresh
    /// identities and a detached root. `subtree` includes ordinary descendants;
    /// Element attributes are copied in either mode. Source reads use this
    /// exact immutable snapshot, including when its live source was destroyed.
    ///
    /// Supported native families are initialized through their real hooks. A
    /// clonable hosted shadow tree is copied deeply even when subtree is false;
    /// non-clonable roots are omitted. Direct ShadowRoot import is NotSupported.
    /// Registry objects and HTML construction installation remain unadmitted.
    /// The returned ticket is usable by later commands and resolves only after
    /// a successful publication; descendant identities are then in the snapshot.
    pub fn import_node(
        &mut self,
        snapshot: &DomSnapshot,
        source: NodeId,
        target_document: impl Into<NodeOperand>,
        subtree: bool,
    ) -> CreatedNode {
        let ticket = self.ticket();
        self.commands.push(Command::Import {
            snapshot: snapshot.clone(),
            source,
            document: target_document.into(),
            subtree,
        });
        ticket
    }
    /// Creates an empty native XML Document with the selected DOM defaults.
    pub fn create_document(&mut self) -> CreatedNode {
        self.create(Creation::Document, None)
    }
    /// Creates a native HTML Document through DOM's complete createHTMLDocument
    /// algorithm. The actual same-store associated Document supplies origin;
    /// other metadata uses the new Document defaults. Omitted title differs
    /// from a supplied empty title, which creates both title and Text nodes.
    /// Skeleton identities are private allocations, not caller creation tickets.
    pub fn create_html_document(
        &mut self,
        associated_document: impl Into<NodeOperand>,
        title: Option<DocumentString>,
    ) -> CreatedNode {
        let ticket = self.ticket();
        self.commands.push(Command::CreateHtmlDocument {
            associated_document: associated_document.into(),
            title,
        });
        ticket
    }
    /// Attaches one native root to an admitted HTML host. The canonical owner
    /// retains closed roots; open-only exposure is a snapshot accessor policy.
    /// Returns a batch-local ticket, never a guessed or prematurely published ID.
    pub fn attach_shadow(
        &mut self,
        host: impl Into<NodeOperand>,
        init: impl Into<crate::DomShadowRootInit>,
    ) -> CreatedNode {
        let ticket = self.ticket();
        self.commands.push(Command::AttachShadow {
            host: host.into(),
            init: init.into(),
        });
        ticket
    }
    pub fn create_fragment(&mut self, document: impl Into<NodeOperand>) -> CreatedNode {
        self.create(Creation::Fragment, Some(document.into()))
    }
    /// Uses DOM's whole-local-name createElement rule. HTML Documents select
    /// HTML namespace and ASCII lowercase; native XML preserves exact spelling
    /// and no namespace. QName splitting belongs only to create_element_ns.
    /// Actual family initialization is required before any Element publishes.
    pub fn create_element(
        &mut self,
        document: impl Into<NodeOperand>,
        local_name: impl Into<DocumentString>,
    ) -> CreatedNode {
        self.create(Creation::Element(local_name.into()), Some(document.into()))
    }
    /// Namespace-aware native creation with DOM element-name validation.
    /// Known families without their complete native adapter are rejected at apply.
    pub fn create_element_ns(
        &mut self,
        document: impl Into<NodeOperand>,
        namespace: Namespace,
        qualified_name: impl Into<DocumentString>,
    ) -> CreatedNode {
        self.create(
            Creation::ElementNs {
                namespace,
                name: qualified_name.into(),
            },
            Some(document.into()),
        )
    }
    /// Changes the applicable cryptographic nonce slot without changing content.
    /// An Element outside HTML/SVG/MathML rejects this operation.
    pub fn set_nonce(&mut self, node: impl Into<NodeOperand>, value: impl Into<DocumentString>) {
        self.commands.push(Command::SetNonce {
            node: node.into(),
            value: value.into(),
        });
    }
    /// Turns a native HTML Marquee on without changing its loop index.
    pub fn start_marquee(&mut self, node: impl Into<NodeOperand>) {
        self.commands.push(Command::SetMarqueeRunning {
            node: node.into(),
            running: true,
        });
    }
    /// Turns a native HTML Marquee off without changing its loop index.
    pub fn stop_marquee(&mut self, node: impl Into<NodeOperand>) {
        self.commands.push(Command::SetMarqueeRunning {
            node: node.into(),
            running: false,
        });
    }
    pub fn create_text(
        &mut self,
        document: impl Into<NodeOperand>,
        data: impl Into<DocumentString>,
    ) -> CreatedNode {
        self.create(Creation::Text(data.into()), Some(document.into()))
    }
    pub fn create_comment(
        &mut self,
        document: impl Into<NodeOperand>,
        data: impl Into<DocumentString>,
    ) -> CreatedNode {
        self.create(Creation::Comment(data.into()), Some(document.into()))
    }
    /// Creates a PI using DOM's XML Name target rule and initial ?> rejection.
    /// Validation and pseudo-attribute derivation happen atomically at apply.
    pub fn create_processing_instruction(
        &mut self,
        document: impl Into<NodeOperand>,
        target: impl Into<DocumentString>,
        data: impl Into<DocumentString>,
    ) -> CreatedNode {
        self.create(
            Creation::ProcessingInstruction {
                target: target.into(),
                data: data.into(),
            },
            Some(document.into()),
        )
    }
    /// Creates a doctype under DOM's relaxed name rule. Empty names are valid;
    /// public/system identifiers are exact strings without extra XML checks.
    pub fn create_document_type(
        &mut self,
        document: impl Into<NodeOperand>,
        name: impl Into<DocumentString>,
        public_id: impl Into<DocumentString>,
        system_id: impl Into<DocumentString>,
    ) -> CreatedNode {
        self.create(
            Creation::DocumentType {
                name: name.into(),
                public_id: public_id.into(),
                system_id: system_id.into(),
            },
            Some(document.into()),
        )
    }
    /// Inserts before the reference child, or appends for None. A fragment moves
    /// its children and becomes empty, retaining its own node document. Each
    /// inserted node is adopted into the parent's document without changing ID.
    pub fn insert_before(
        &mut self,
        node: impl Into<NodeOperand>,
        parent: impl Into<NodeOperand>,
        before: Option<NodeOperand>,
    ) {
        self.commands.push(Command::Insert {
            node: node.into(),
            parent: parent.into(),
            before,
        });
    }
    /// Detaches while preserving live identity and node-document membership.
    pub fn detach(&mut self, node: impl Into<NodeOperand>) {
        self.commands.push(Command::Detach(node.into()));
    }
    /// Replaces a child using the selected source's child exclusion list.
    /// The removed child remains addressable. Fragment replacement first
    /// adopts the supplied fragment itself, unlike ordinary fragment insertion.
    pub fn replace_child(
        &mut self,
        node: impl Into<NodeOperand>,
        parent: impl Into<NodeOperand>,
        child: impl Into<NodeOperand>,
    ) {
        self.commands.push(Command::ReplaceChild {
            node: node.into(),
            parent: parent.into(),
            child: child.into(),
        });
    }
    /// Removes a same-store non-Document node and changes its entire subtree's
    /// node document, preserving canonical identity. Same-document adoption
    /// still removes an attached node; a detached same-document node is a no-op.
    pub fn adopt(&mut self, node: impl Into<NodeOperand>, document: impl Into<NodeOperand>) {
        self.commands.push(Command::Adopt {
            node: node.into(),
            document: document.into(),
        });
    }
    pub fn set_attribute(
        &mut self,
        node: impl Into<NodeOperand>,
        name: impl Into<DocumentString>,
        value: impl Into<DocumentString>,
    ) {
        self.commands.push(Command::SetAttribute {
            node: node.into(),
            namespace: None,
            name: name.into(),
            value: value.into(),
        });
    }
    /// Sets by expanded name. Existing attributes preserve their prefix/order.
    pub fn set_attribute_ns(
        &mut self,
        node: impl Into<NodeOperand>,
        namespace: Namespace,
        qualified_name: impl Into<DocumentString>,
        value: impl Into<DocumentString>,
    ) {
        self.commands.push(Command::SetAttribute {
            node: node.into(),
            namespace: Some(namespace),
            name: qualified_name.into(),
            value: value.into(),
        });
    }
    pub fn remove_attribute(
        &mut self,
        node: impl Into<NodeOperand>,
        qualified_name: impl Into<DocumentString>,
    ) {
        self.commands.push(Command::RemoveAttribute {
            node: node.into(),
            namespace: None,
            name: qualified_name.into(),
        });
    }
    /// Removes by exact namespace/local identity. Removal performs no name
    /// validation; a spelling absent from the attribute list is a no-op.
    pub fn remove_attribute_ns(
        &mut self,
        node: impl Into<NodeOperand>,
        namespace: Namespace,
        local_name: impl Into<DocumentString>,
    ) {
        self.commands.push(Command::RemoveAttribute {
            node: node.into(),
            namespace: Some(namespace),
            name: local_name.into(),
        });
    }
    /// Destroys a detached non-Document, non-ShadowRoot subtree, including owned
    /// shadow trees, retiring published identities.
    pub fn destroy(&mut self, node: impl Into<NodeOperand>) {
        self.commands.push(Command::Destroy(node.into()));
    }
    /// CharacterData replacement using UTF-16 code-unit offsets and counts.
    /// An excessive count truncates to the remaining length; offset past the
    /// end is IndexSize. The operation accepts Text, Comment and PI. PI data
    /// edits recompute pseudo-attributes without repeating creation validation.
    pub fn replace_data(
        &mut self,
        node: impl Into<NodeOperand>,
        offset: usize,
        count: usize,
        data: impl Into<DocumentString>,
    ) {
        self.commands.push(Command::ReplaceData {
            node: node.into(),
            offset,
            count,
            data: data.into(),
        });
    }
    pub fn len(&self) -> usize {
        self.commands.len()
    }
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}
