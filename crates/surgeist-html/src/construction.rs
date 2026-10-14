//! Bounded native authoring and immutable, result-local construction output.

pub(crate) mod parser_session;

use crate::{DocumentString, Namespace, OrderedAttributes, QualifiedName};
use std::{
    error::Error,
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

/// The HTML document's selected compatibility mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDocumentMode {
    NoQuirks,
    LimitedQuirks,
    Quirks,
}

/// Explicit finite admission limits for a construction arena.
///
/// Root nodes count toward nodes. Ordinary children, template contents, and
/// shadow associations count toward edges and depth. Every stored name, URI,
/// prefix, attribute value, and textual slot counts toward UTF-16 units, even
/// when its immutable string allocation is shared. Root depth is zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlConstructionLimits {
    nodes: usize,
    edges: usize,
    attributes: usize,
    utf16_units: usize,
    depth: usize,
}
impl HtmlConstructionLimits {
    /// Selects every admission limit explicitly.
    pub fn new(
        nodes: usize,
        edges: usize,
        attributes: usize,
        utf16_units: usize,
        depth: usize,
    ) -> Self {
        Self {
            nodes,
            edges,
            attributes,
            utf16_units,
            depth,
        }
    }
    /// Maximum admitted nodes, including fragment roots.
    pub fn nodes(self) -> usize {
        self.nodes
    }
    /// Maximum ordinary and association edges.
    pub fn edges(self) -> usize {
        self.edges
    }
    /// Maximum stored attributes.
    pub fn attributes(self) -> usize {
        self.attributes
    }
    /// Maximum stored UTF-16 units.
    pub fn utf16_units(self) -> usize {
        self.utf16_units
    }
    /// Maximum depth, including association edges.
    pub fn depth(self) -> usize {
        self.depth
    }
}

/// Admitted work in a native construction artifact.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlConstructionUsage {
    nodes: usize,
    edges: usize,
    attributes: usize,
    utf16_units: usize,
    depth: usize,
}
impl HtmlConstructionUsage {
    /// Number of admitted nodes.
    pub fn nodes(self) -> usize {
        self.nodes
    }
    /// Number of ordinary and association edges.
    pub fn edges(self) -> usize {
        self.edges
    }
    /// Number of stored attributes.
    pub fn attributes(self) -> usize {
        self.attributes
    }
    /// Number of stored UTF-16 units.
    pub fn utf16_units(self) -> usize {
        self.utf16_units
    }
    /// Deepest admitted node, including association edges.
    pub fn depth(self) -> usize {
        self.depth
    }
}

/// The resource that an append would exceed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlConstructionResource {
    Nodes,
    Edges,
    Attributes,
    Utf16Units,
    Depth,
}

/// A rejected append; existing nodes and admitted usage remain unchanged.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlConstructionError {
    /// The supplied handle belongs to another builder.
    ForeignBuilder,
    /// A text, comment, PI, or doctype cannot receive children.
    ParentCannotHaveChildren,
    /// Native template children must be appended to its separate contents.
    TemplateContentsRequired,
    /// A shadow association requires an element host.
    ShadowHostMustBeElement,
    /// An element already has a shadow fragment.
    ShadowAlreadyAttached,
    /// A finite resource limit would be exceeded.
    LimitExceeded {
        resource: HtmlConstructionResource,
        limit: usize,
        attempted: usize,
    },
    /// A storage count cannot be represented by this platform.
    RepresentationExhausted,
    /// Reserving arena or edge storage failed.
    AllocationFailed,
}
impl fmt::Display for HtmlConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTML construction rejected: {self:?}")
    }
}
impl Error for HtmlConstructionError {}

#[derive(Debug)]
struct Identity;

/// An opaque node identity qualified by its immutable construction result.
///
/// Cloning a document preserves identity. A separate build always has different
/// identities, even when its text and local positions are identical.
#[derive(Clone, Debug)]
pub struct HtmlNodeId {
    identity: Arc<Identity>,
    index: usize,
}
impl PartialEq for HtmlNodeId {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.identity, &other.identity) && self.index == other.index
    }
}
impl Eq for HtmlNodeId {}
impl Hash for HtmlNodeId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.identity).hash(state);
        self.index.hash(state);
    }
}

/// A checked local handle usable only by the builder that issued it.
#[derive(Clone, Debug)]
pub struct HtmlBuilderNode {
    identity: Arc<Identity>,
    index: usize,
}

/// Lookup rejected an identity from a different immutable result.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ForeignHtmlResult;
impl fmt::Display for ForeignHtmlResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("node identity belongs to a different HTML result")
    }
}
impl Error for ForeignHtmlResult {}

/// Exact native doctype data, without selecting or changing document mode.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDoctype {
    name: DocumentString,
    public_id: Option<DocumentString>,
    system_id: Option<DocumentString>,
}
impl HtmlDoctype {
    /// Preserves name and optional identifiers, including absent versus empty.
    pub fn new(
        name: impl Into<DocumentString>,
        public_id: Option<DocumentString>,
        system_id: Option<DocumentString>,
    ) -> Self {
        Self {
            name: name.into(),
            public_id,
            system_id,
        }
    }
    /// Exact doctype name.
    pub fn name(&self) -> &DocumentString {
        &self.name
    }
    /// Exact optional public identifier.
    pub fn public_id(&self) -> Option<&DocumentString> {
        self.public_id.as_ref()
    }
    /// Exact optional system identifier.
    pub fn system_id(&self) -> Option<&DocumentString> {
        self.system_id.as_ref()
    }
}

/// Exact immutable processing-instruction data in construction output.
///
/// Stored strings retain every original code unit. Parser recovery can produce
/// data that fails native authoring syntax, so this record is not checked input
/// for a native builder. [`HtmlAuthoredProcessingInstruction`] owns that check.
///
/// ```compile_fail,E0308
/// use surgeist_html::{HtmlAuthoredProcessingInstruction, HtmlConstructionLimits,
///     HtmlFragment, HtmlProcessingInstruction};
/// let stored: HtmlProcessingInstruction =
///     HtmlAuthoredProcessingInstruction::new("x", "data").unwrap().into();
/// let mut builder = HtmlFragment::builder(HtmlConstructionLimits::new(2, 1, 0, 5, 1)).unwrap();
/// let root = builder.root();
/// builder.append_processing_instruction(&root, stored);
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlProcessingInstruction {
    target: DocumentString,
    data: DocumentString,
}
impl HtmlProcessingInstruction {
    /// Exact target.
    pub fn target(&self) -> &DocumentString {
        &self.target
    }
    /// Exact data.
    pub fn data(&self) -> &DocumentString {
        &self.data
    }
}

/// Checked native PI input according to the pinned HTML authoring syntax.
///
/// This wraps the shared exact record without duplicating its fields. Recovery
/// does not relax native authoring: conversion from a stored record is fallible.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlAuthoredProcessingInstruction {
    instruction: HtmlProcessingInstruction,
}
impl HtmlAuthoredProcessingInstruction {
    /// Checks ASCII target syntax, excluded XML forms, and HTML PI data syntax.
    pub fn new(
        target: impl Into<DocumentString>,
        data: impl Into<DocumentString>,
    ) -> Result<Self, HtmlProcessingInstructionError> {
        Self::try_from(HtmlProcessingInstruction {
            target: target.into(),
            data: data.into(),
        })
    }
    /// Exact checked target.
    pub fn target(&self) -> &DocumentString {
        self.instruction.target()
    }
    /// Exact checked data.
    pub fn data(&self) -> &DocumentString {
        self.instruction.data()
    }
}
impl From<HtmlAuthoredProcessingInstruction> for HtmlProcessingInstruction {
    fn from(value: HtmlAuthoredProcessingInstruction) -> Self {
        value.instruction
    }
}
impl TryFrom<HtmlProcessingInstruction> for HtmlAuthoredProcessingInstruction {
    type Error = HtmlProcessingInstructionError;

    fn try_from(instruction: HtmlProcessingInstruction) -> Result<Self, Self::Error> {
        let units = instruction.target.code_units();
        let alpha = |unit: u16| matches!(unit, 65..=90 | 97..=122);
        if !units
            .first()
            .is_some_and(|unit| alpha(*unit) || *unit == 95)
            || !units
                .iter()
                .all(|unit| alpha(*unit) || matches!(*unit, 48..=57 | 45 | 95))
        {
            return Err(HtmlProcessingInstructionError::InvalidTarget);
        }
        let reserved = |text: &str| {
            units
                .iter()
                .copied()
                .map(|unit| (unit as u8).to_ascii_lowercase())
                .eq(text.bytes())
        };
        if reserved("xml") || reserved("xml-stylesheet") {
            return Err(HtmlProcessingInstructionError::ReservedTarget);
        }
        if instruction.data.code_units().contains(&62) {
            return Err(HtmlProcessingInstructionError::DataContainsGreaterThan);
        }
        if instruction.data.code_units().last() == Some(&63) {
            return Err(HtmlProcessingInstructionError::DataEndsWithQuestionMark);
        }
        Ok(Self { instruction })
    }
}
/// A strict processing-instruction authoring violation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlProcessingInstructionError {
    InvalidTarget,
    ReservedTarget,
    DataContainsGreaterThan,
    DataEndsWithQuestionMark,
}
impl fmt::Display for HtmlProcessingInstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid native HTML processing instruction: {self:?}")
    }
}
impl Error for HtmlProcessingInstructionError {}

/// Exact element name and ordered raw attributes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlElement {
    link_flags: Option<crate::HtmlLinkFlags>,
    name: QualifiedName,
    attributes: OrderedAttributes,
}
impl HtmlElement {
    /// Actual construction flags. Imported records without history return None.
    pub fn link_flags(&self) -> Option<crate::HtmlLinkFlags> {
        self.link_flags
    }
    /// Exact shared qualified name.
    pub fn name(&self) -> &QualifiedName {
        &self.name
    }
    /// Ordered shared raw attributes.
    pub fn attributes(&self) -> &OrderedAttributes {
        &self.attributes
    }
}

/// Immutable content records; hierarchy is exposed by [`HtmlNode`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlNodeKind {
    Document(HtmlDocumentMode),
    Fragment,
    Element(HtmlElement),
    Text(DocumentString),
    Comment(DocumentString),
    ProcessingInstruction(HtmlProcessingInstruction),
    Doctype(HtmlDoctype),
}

/// Declarative shadow-root visibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlShadowRootMode {
    Open,
    Closed,
}
/// Declarative shadow-root slot assignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSlotAssignment {
    Named,
    Manual,
}
/// Explicit properties of a native declarative shadow association.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlShadowRootProperties {
    mode: HtmlShadowRootMode,
    slot_assignment: HtmlSlotAssignment,
    delegates_focus: bool,
    clonable: bool,
    serializable: bool,
}
impl HtmlShadowRootProperties {
    /// Selects properties without guessing host or parser context.
    pub fn new(
        mode: HtmlShadowRootMode,
        slot_assignment: HtmlSlotAssignment,
        delegates_focus: bool,
        clonable: bool,
        serializable: bool,
    ) -> Self {
        Self {
            mode,
            slot_assignment,
            delegates_focus,
            clonable,
            serializable,
        }
    }
    /// Shadow visibility.
    pub fn mode(self) -> HtmlShadowRootMode {
        self.mode
    }
    /// Slot-assignment mode.
    pub fn slot_assignment(self) -> HtmlSlotAssignment {
        self.slot_assignment
    }
    /// Whether focus is delegated.
    pub fn delegates_focus(self) -> bool {
        self.delegates_focus
    }
    /// Whether the root is clonable.
    pub fn clonable(self) -> bool {
        self.clonable
    }
    /// Whether the root is serializable.
    pub fn serializable(self) -> bool {
        self.serializable
    }
}
/// A separate shadow fragment associated with its result-local host.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlDeclarativeShadowRoot {
    host: HtmlNodeId,
    contents: HtmlNodeId,
    properties: HtmlShadowRootProperties,
}
impl HtmlDeclarativeShadowRoot {
    /// Host element identity.
    pub fn host(&self) -> &HtmlNodeId {
        &self.host
    }
    /// Associated fragment root identity.
    pub fn contents(&self) -> &HtmlNodeId {
        &self.contents
    }
    /// Explicit shadow properties.
    pub fn properties(&self) -> HtmlShadowRootProperties {
        self.properties
    }
}
/// The non-child relationship that owns an associated fragment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlFragmentOwner {
    Template(HtmlNodeId),
    ShadowHost(HtmlNodeId),
}

/// Whether template contents are owned or reference a separate shadow subtree.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTemplateContentsKind {
    /// The template owns its separate ordinary contents fragment.
    Owned,
    /// The template refers to shadow contents without becoming their owner.
    ShadowReference,
}

#[derive(Debug)]
struct Record {
    kind: HtmlNodeKind,
    parent: Option<usize>,
    children: Vec<usize>,
    depth: usize,
    template_contents: Option<usize>,
    shadow: Option<HtmlDeclarativeShadowRoot>,
    owner: Option<HtmlFragmentOwner>,
}
impl Record {
    fn new(kind: HtmlNodeKind, parent: Option<usize>, depth: usize) -> Self {
        Self {
            kind,
            parent,
            children: Vec::new(),
            depth,
            template_contents: None,
            shadow: None,
            owner: None,
        }
    }
}
#[derive(Debug)]
struct Arena {
    identity: Arc<Identity>,
    records: Vec<Record>,
    usage: HtmlConstructionUsage,
}
impl Arena {
    fn id(&self, index: usize) -> HtmlNodeId {
        HtmlNodeId {
            identity: self.identity.clone(),
            index,
        }
    }
    fn lookup(&self, id: &HtmlNodeId) -> Result<HtmlNode<'_>, ForeignHtmlResult> {
        if !Arc::ptr_eq(&self.identity, &id.identity) {
            return Err(ForeignHtmlResult);
        }
        Ok(HtmlNode {
            arena: self,
            index: id.index,
        })
    }
}

/// A borrowed immutable node view, with ordinary and association edges distinct.
#[derive(Clone, Copy, Debug)]
pub struct HtmlNode<'a> {
    arena: &'a Arena,
    index: usize,
}
impl<'a> HtmlNode<'a> {
    /// Result-qualified node identity.
    pub fn id(self) -> HtmlNodeId {
        self.arena.id(self.index)
    }
    /// Immutable content record.
    pub fn kind(self) -> &'a HtmlNodeKind {
        &self.arena.records[self.index].kind
    }
    /// Ordinary parent; association fragment roots have no ordinary parent.
    pub fn parent(self) -> Option<HtmlNodeId> {
        self.arena.records[self.index]
            .parent
            .map(|index| self.arena.id(index))
    }
    /// Ordinary children in exact append order; associations are excluded.
    pub fn children(self) -> impl ExactSizeIterator<Item = HtmlNodeId> + DoubleEndedIterator + 'a {
        self.arena.records[self.index]
            .children
            .iter()
            .map(|index| self.arena.id(*index))
    }
    /// Separate contents of an HTML template element.
    pub fn template_contents(self) -> Option<HtmlNodeId> {
        self.arena.records[self.index]
            .template_contents
            .map(|index| self.arena.id(index))
    }
    /// Native declarative shadow association, if supplied.
    pub fn declarative_shadow_root(self) -> Option<&'a HtmlDeclarativeShadowRoot> {
        self.arena.records[self.index].shadow.as_ref()
    }
    /// Template or shadow association owning this fragment root.
    pub fn fragment_owner(self) -> Option<&'a HtmlFragmentOwner> {
        self.arena.records[self.index].owner.as_ref()
    }
}

/// A concrete immutable HTML document produced by native construction.
/// It makes no claim about parsing, source provenance, or conformance checking.
#[derive(Clone, Debug)]
pub struct HtmlDocument {
    arena: Arc<Arena>,
}
impl HtmlDocument {
    /// Begins bounded native document construction with an explicit mode.
    pub fn builder(
        limits: HtmlConstructionLimits,
        mode: HtmlDocumentMode,
    ) -> Result<HtmlDocumentBuilder, HtmlConstructionError> {
        Ok(HtmlDocumentBuilder {
            builder: Builder::new(limits, HtmlNodeKind::Document(mode))?,
        })
    }
    /// Document root.
    pub fn root(&self) -> HtmlNode<'_> {
        HtmlNode {
            arena: &self.arena,
            index: 0,
        }
    }
    /// Explicit document mode, unaffected by authored doctype text.
    pub fn mode(&self) -> HtmlDocumentMode {
        match self.arena.records[0].kind {
            HtmlNodeKind::Document(mode) => mode,
            _ => unreachable!("document constructor owns root kind"),
        }
    }
    /// Looks up a node, explicitly rejecting foreign-result identities.
    pub fn node(&self, id: &HtmlNodeId) -> Result<HtmlNode<'_>, ForeignHtmlResult> {
        self.arena.lookup(id)
    }
    /// Admitted construction usage.
    pub fn usage(&self) -> HtmlConstructionUsage {
        self.arena.usage
    }
    /// Obtains a separate immutable fragment view from the same flat arena.
    pub fn fragment(&self, id: &HtmlNodeId) -> Result<Option<HtmlFragment>, ForeignHtmlResult> {
        fragment_view(&self.arena, id)
    }
}
/// A concrete immutable HTML fragment, including associated template/shadow views.
#[derive(Clone, Debug)]
pub struct HtmlFragment {
    arena: Arc<Arena>,
    root: usize,
}
impl HtmlFragment {
    /// Begins source-free native fragment construction. No parser context is inferred.
    pub fn builder(
        limits: HtmlConstructionLimits,
    ) -> Result<HtmlFragmentBuilder, HtmlConstructionError> {
        Ok(HtmlFragmentBuilder {
            builder: Builder::new(limits, HtmlNodeKind::Fragment)?,
        })
    }
    /// Fragment root, whose children are the fragment's ordinary output.
    pub fn root(&self) -> HtmlNode<'_> {
        HtmlNode {
            arena: &self.arena,
            index: self.root,
        }
    }
    /// Looks up an identity from this arena, including its associated fragments.
    pub fn node(&self, id: &HtmlNodeId) -> Result<HtmlNode<'_>, ForeignHtmlResult> {
        self.arena.lookup(id)
    }
    /// Usage for the whole shared construction arena, not just this fragment view.
    pub fn usage(&self) -> HtmlConstructionUsage {
        self.arena.usage
    }
    /// Obtains another fragment view belonging to the same construction result.
    pub fn fragment(&self, id: &HtmlNodeId) -> Result<Option<HtmlFragment>, ForeignHtmlResult> {
        fragment_view(&self.arena, id)
    }
}
fn fragment_view(
    arena: &Arc<Arena>,
    id: &HtmlNodeId,
) -> Result<Option<HtmlFragment>, ForeignHtmlResult> {
    Ok(
        matches!(arena.lookup(id)?.kind(), HtmlNodeKind::Fragment).then(|| HtmlFragment {
            arena: arena.clone(),
            root: id.index,
        }),
    )
}

#[derive(Debug)]
struct Builder {
    identity: Arc<Identity>,
    records: Vec<Record>,
    limits: HtmlConstructionLimits,
    usage: HtmlConstructionUsage,
}
impl Builder {
    fn new(
        limits: HtmlConstructionLimits,
        kind: HtmlNodeKind,
    ) -> Result<Self, HtmlConstructionError> {
        let mut builder = Self {
            identity: Arc::new(Identity),
            records: Vec::new(),
            limits,
            usage: HtmlConstructionUsage::default(),
        };
        let usage = builder.admit(1, 0, 0, 0, 0)?;
        builder
            .records
            .try_reserve(1)
            .map_err(|_| HtmlConstructionError::AllocationFailed)?;
        builder.records.push(Record::new(kind, None, 0));
        builder.usage = usage;
        Ok(builder)
    }
    fn handle(&self, index: usize) -> HtmlBuilderNode {
        HtmlBuilderNode {
            identity: self.identity.clone(),
            index,
        }
    }
    fn index(&self, node: &HtmlBuilderNode) -> Result<usize, HtmlConstructionError> {
        if !Arc::ptr_eq(&self.identity, &node.identity) {
            return Err(HtmlConstructionError::ForeignBuilder);
        }
        Ok(node.index)
    }
    fn admit(
        &self,
        nodes: usize,
        edges: usize,
        attributes: usize,
        units: usize,
        depth: usize,
    ) -> Result<HtmlConstructionUsage, HtmlConstructionError> {
        let add = |left: usize, right| {
            left.checked_add(right)
                .ok_or(HtmlConstructionError::RepresentationExhausted)
        };
        let next = HtmlConstructionUsage {
            nodes: add(self.usage.nodes, nodes)?,
            edges: add(self.usage.edges, edges)?,
            attributes: add(self.usage.attributes, attributes)?,
            utf16_units: add(self.usage.utf16_units, units)?,
            depth: self.usage.depth.max(depth),
        };
        for (resource, limit, attempted) in [
            (
                HtmlConstructionResource::Nodes,
                self.limits.nodes,
                next.nodes,
            ),
            (
                HtmlConstructionResource::Edges,
                self.limits.edges,
                next.edges,
            ),
            (
                HtmlConstructionResource::Attributes,
                self.limits.attributes,
                next.attributes,
            ),
            (
                HtmlConstructionResource::Utf16Units,
                self.limits.utf16_units,
                next.utf16_units,
            ),
            (
                HtmlConstructionResource::Depth,
                self.limits.depth,
                next.depth,
            ),
        ] {
            if attempted > limit {
                return Err(HtmlConstructionError::LimitExceeded {
                    resource,
                    limit,
                    attempted,
                });
            }
        }
        Ok(next)
    }
    fn append(
        &mut self,
        parent: &HtmlBuilderNode,
        kind: HtmlNodeKind,
    ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
        let parent = self.index(parent)?;
        match &self.records[parent].kind {
            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment | HtmlNodeKind::Element(_) => {}
            _ => return Err(HtmlConstructionError::ParentCannotHaveChildren),
        }
        if self.records[parent].template_contents.is_some() {
            return Err(HtmlConstructionError::TemplateContentsRequired);
        }
        let template = matches!(&kind, HtmlNodeKind::Element(element) if element.name.namespace() == &Namespace::html() && element.name.local_name().code_units().iter().copied().eq("template".encode_utf16()));
        let depth = self.records[parent]
            .depth
            .checked_add(1)
            .ok_or(HtmlConstructionError::RepresentationExhausted)?;
        let final_depth = depth
            .checked_add(usize::from(template))
            .ok_or(HtmlConstructionError::RepresentationExhausted)?;
        let (attributes, units) = content_usage(&kind)?;
        let next = self.admit(
            1 + usize::from(template),
            1 + usize::from(template),
            attributes,
            units,
            final_depth,
        )?;
        self.records
            .try_reserve(1 + usize::from(template))
            .map_err(|_| HtmlConstructionError::AllocationFailed)?;
        self.records[parent]
            .children
            .try_reserve(1)
            .map_err(|_| HtmlConstructionError::AllocationFailed)?;
        let index = self.records.len();
        let mut record = Record::new(kind, Some(parent), depth);
        if template {
            record.template_contents = Some(index + 1);
        }
        self.records.push(record);
        self.records[parent].children.push(index);
        if template {
            let mut contents = Record::new(HtmlNodeKind::Fragment, None, final_depth);
            contents.owner = Some(HtmlFragmentOwner::Template(HtmlNodeId {
                identity: self.identity.clone(),
                index,
            }));
            self.records.push(contents);
        }
        self.usage = next;
        Ok(self.handle(index))
    }
    fn template_contents(
        &self,
        template: &HtmlBuilderNode,
    ) -> Result<Option<HtmlBuilderNode>, HtmlConstructionError> {
        Ok(self.records[self.index(template)?]
            .template_contents
            .map(|index| self.handle(index)))
    }
    fn shadow(
        &mut self,
        host: &HtmlBuilderNode,
        properties: HtmlShadowRootProperties,
    ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
        let host = self.index(host)?;
        if !matches!(self.records[host].kind, HtmlNodeKind::Element(_)) {
            return Err(HtmlConstructionError::ShadowHostMustBeElement);
        }
        if self.records[host].shadow.is_some() {
            return Err(HtmlConstructionError::ShadowAlreadyAttached);
        }
        let depth = self.records[host]
            .depth
            .checked_add(1)
            .ok_or(HtmlConstructionError::RepresentationExhausted)?;
        let next = self.admit(1, 1, 0, 0, depth)?;
        self.records
            .try_reserve(1)
            .map_err(|_| HtmlConstructionError::AllocationFailed)?;
        let index = self.records.len();
        let host_id = HtmlNodeId {
            identity: self.identity.clone(),
            index: host,
        };
        let contents_id = HtmlNodeId {
            identity: self.identity.clone(),
            index,
        };
        let mut contents = Record::new(HtmlNodeKind::Fragment, None, depth);
        contents.owner = Some(HtmlFragmentOwner::ShadowHost(host_id.clone()));
        self.records.push(contents);
        self.records[host].shadow = Some(HtmlDeclarativeShadowRoot {
            host: host_id,
            contents: contents_id,
            properties,
        });
        self.usage = next;
        Ok(self.handle(index))
    }
    fn finish(self) -> Arc<Arena> {
        Arc::new(Arena {
            identity: self.identity,
            records: self.records,
            usage: self.usage,
        })
    }
}

fn content_usage(kind: &HtmlNodeKind) -> Result<(usize, usize), HtmlConstructionError> {
    let add = |left: usize, right| {
        left.checked_add(right)
            .ok_or(HtmlConstructionError::RepresentationExhausted)
    };
    let name_units = |name: &QualifiedName| -> Result<usize, HtmlConstructionError> {
        add(
            add(
                name.local_name().len(),
                name.prefix().map_or(0, DocumentString::len),
            )?,
            name.namespace().uri().map_or(0, DocumentString::len),
        )
    };
    Ok(match kind {
        HtmlNodeKind::Element(element) => {
            let mut units = name_units(&element.name)?;
            for attribute in element.attributes.iter() {
                units = add(
                    add(units, name_units(attribute.name())?)?,
                    attribute.value().len(),
                )?;
            }
            (element.attributes.len(), units)
        }
        HtmlNodeKind::Text(value) | HtmlNodeKind::Comment(value) => (0, value.len()),
        HtmlNodeKind::ProcessingInstruction(pi) => (0, add(pi.target.len(), pi.data.len())?),
        HtmlNodeKind::Doctype(doctype) => (
            0,
            add(
                add(
                    doctype.name.len(),
                    doctype.public_id.as_ref().map_or(0, DocumentString::len),
                )?,
                doctype.system_id.as_ref().map_or(0, DocumentString::len),
            )?,
        ),
        HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment => (0, 0),
    })
}

/// A bounded append-only native document authoring session.
#[derive(Debug)]
pub struct HtmlDocumentBuilder {
    builder: Builder,
}
/// A bounded append-only native fragment authoring session.
#[derive(Debug)]
pub struct HtmlFragmentBuilder {
    builder: Builder,
}

macro_rules! authoring_methods {
    () => {
        /// Root handle issued by this builder.
        pub fn root(&self) -> HtmlBuilderNode {
            self.builder.handle(0)
        }
        /// Current admitted usage; rejected appends never change it.
        pub fn usage(&self) -> HtmlConstructionUsage {
            self.builder.usage
        }
        /// Appends exact shared element values without HTML normalization or a
        /// general content-conformance check. HTML templates get separate contents.
        pub fn append_element(
            &mut self,
            parent: &HtmlBuilderNode,
            name: QualifiedName,
            attributes: OrderedAttributes,
        ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
            self.builder.append(
                parent,
                HtmlNodeKind::Element(HtmlElement {
                    link_flags: (crate::KnownHtmlElement::from_name(&name)
                        == Some(crate::KnownHtmlElement::Link))
                    .then(crate::HtmlLinkFlags::fresh),
                    name,
                    attributes,
                }),
            )
        }
        /// Appends exact text, retaining nulls and isolated surrogates.
        pub fn append_text(
            &mut self,
            parent: &HtmlBuilderNode,
            value: impl Into<DocumentString>,
        ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
            self.builder
                .append(parent, HtmlNodeKind::Text(value.into()))
        }
        /// Appends exact comment data without imposing serialized-comment syntax.
        pub fn append_comment(
            &mut self,
            parent: &HtmlBuilderNode,
            value: impl Into<DocumentString>,
        ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
            self.builder
                .append(parent, HtmlNodeKind::Comment(value.into()))
        }
        /// Appends a checked native processing instruction.
        pub fn append_processing_instruction(
            &mut self,
            parent: &HtmlBuilderNode,
            value: HtmlAuthoredProcessingInstruction,
        ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
            self.builder
                .append(parent, HtmlNodeKind::ProcessingInstruction(value.into()))
        }
        /// Appends exact doctype data without deriving document mode.
        pub fn append_doctype(
            &mut self,
            parent: &HtmlBuilderNode,
            value: HtmlDoctype,
        ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
            self.builder.append(parent, HtmlNodeKind::Doctype(value))
        }
        /// Retrieves an HTML template's separate contents handle.
        pub fn template_contents(
            &self,
            template: &HtmlBuilderNode,
        ) -> Result<Option<HtmlBuilderNode>, HtmlConstructionError> {
            self.builder.template_contents(template)
        }
        /// Records a native declarative shadow association with a separate
        /// fragment. This checks arena ownership and single attachment; host
        /// language conformance and external context admission are separate.
        pub fn attach_declarative_shadow(
            &mut self,
            host: &HtmlBuilderNode,
            properties: HtmlShadowRootProperties,
        ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
            self.builder.shadow(host, properties)
        }
    };
}
impl HtmlDocumentBuilder {
    authoring_methods!();
    /// Consumes the session and freezes its entire flat arena.
    pub fn finish(self) -> HtmlDocument {
        HtmlDocument {
            arena: self.builder.finish(),
        }
    }
}
impl HtmlFragmentBuilder {
    authoring_methods!();
    /// Consumes the session and freezes its entire flat arena.
    pub fn finish(self) -> HtmlFragment {
        HtmlFragment {
            arena: self.builder.finish(),
            root: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_recovered_pi_data_is_stored_without_becoming_checked_author_input() {
        // The five pinned PI states produce x/a? from <?x a??>. This is a
        // construction-record witness, not a substitute parser or public factory.
        let recovered = HtmlProcessingInstruction {
            target: DocumentString::from("x"),
            data: DocumentString::from("a?"),
        };
        let record = Record::new(
            HtmlNodeKind::ProcessingInstruction(recovered.clone()),
            None,
            0,
        );
        let HtmlNodeKind::ProcessingInstruction(stored) = record.kind else {
            panic!("expected exact processing-instruction record");
        };
        assert_eq!(stored.target().code_units(), [120]);
        assert_eq!(stored.data().code_units(), [97, 63]);
        let builder = HtmlFragment::builder(HtmlConstructionLimits::new(2, 1, 0, 3, 1)).unwrap();
        let before = builder.usage();
        assert_eq!(
            HtmlAuthoredProcessingInstruction::try_from(stored),
            Err(HtmlProcessingInstructionError::DataEndsWithQuestionMark)
        );
        assert_eq!(builder.usage(), before);
        assert_eq!(builder.finish().root().children().len(), 0);
        assert_eq!(recovered.data().code_units(), [97, 63]);
    }

    #[test]
    fn conversion_from_exact_pi_records_uses_the_same_typed_authoring_checks() {
        for (target, data, error) in [
            (
                "9start",
                "data",
                HtmlProcessingInstructionError::InvalidTarget,
            ),
            (
                "Xml-StYlEsHeEt",
                "data",
                HtmlProcessingInstructionError::ReservedTarget,
            ),
            (
                "x",
                "a>b",
                HtmlProcessingInstructionError::DataContainsGreaterThan,
            ),
            (
                "x",
                "a?",
                HtmlProcessingInstructionError::DataEndsWithQuestionMark,
            ),
        ] {
            let stored = HtmlProcessingInstruction {
                target: DocumentString::from(target),
                data: DocumentString::from(data),
            };
            assert_eq!(
                HtmlAuthoredProcessingInstruction::try_from(stored.clone()),
                Err(error)
            );
            assert_eq!(
                stored.target().code_units(),
                target.encode_utf16().collect::<Vec<_>>()
            );
            assert_eq!(
                stored.data().code_units(),
                data.encode_utf16().collect::<Vec<_>>()
            );
        }
    }
}
