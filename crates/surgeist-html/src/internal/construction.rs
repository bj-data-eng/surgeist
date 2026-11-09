//! Bounded native authoring and immutable, result-local construction output.

pub(crate) mod bounded;

use crate::internal::accounting::{
    HtmlAccountingFault, HtmlCapScope, HtmlLedgerOutcome, HtmlResource, HtmlWork, classify, wide,
};
#[cfg(test)]
use crate::internal::values::Namespace;
use crate::{
    internal::values::DocumentString, internal::values::OrderedAttributes,
    internal::values::QualifiedName,
};
use std::{
    collections::VecDeque,
    error::Error,
    fmt,
    hash::{Hash, Hasher},
    sync::{Arc, Weak},
};
use surgeist_ledger::{Change, Charge, Error as LedgerError, Operation, Scope};

/// The HTML document's selected compatibility mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDocumentMode {
    /// The document is in no-quirks (standards) mode. Mode names follow the DOM Standard's
    /// document modes.
    NoQuirks,
    /// The document is in limited-quirks (almost standards) mode, distinct from both no-quirks and
    /// quirks mode. The HTML parser's initial insertion mode selects it for certain DOCTYPE public
    /// identifiers (HTML Standard 13.2.6.4.1).
    LimitedQuirks,
    /// The document is in quirks mode, which the HTML parser's initial insertion mode selects for
    /// a missing or legacy DOCTYPE (HTML Standard 13.2.6.4.1).
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
    /// The node limit; [`HtmlConstructionLimits::nodes`] was reached.
    Nodes,
    /// The edge limit (ordinary children, template contents and shadow associations) was reached.
    Edges,
    /// The stored-attribute limit was reached.
    Attributes,
    /// The limit on stored UTF-16 code units (names, URIs, prefixes, attribute values and textual
    /// slots) was reached.
    Utf16Units,
    /// The depth limit was reached; association edges count toward depth.
    Depth,
}

/// A rejected append or construction step; existing nodes and admitted usage
/// remain unchanged.
///
/// The cause is private so that new refusal causes can be added without
/// breaking callers. Query it through the `is_*` predicates and the payload
/// accessors; every message states the cause in prose.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlConstructionError {
    kind: HtmlConstructionErrorKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum HtmlConstructionErrorKind {
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
    UnknownNode,
    WrongNodeKind,
    RootMove,
    AssociationMove,
    WrongSibling,
    Cycle,
    AssociationAlreadyOwned,
    InvalidTemplateAssociation,
    DuplicateAttribute {
        first: usize,
        duplicate: usize,
    },
    AttributeNameMismatch,
    AttributeOrderMismatch,
    InvalidShadowAssociation,
    InvalidContentsReference,
    ParentChildConflict,
    InvalidOutputRoot,
    OperationNotReady,
    ForeignAllocation,
    CapacityMismatch,
    /// A name index lent to an attribute edit could not serve a step.
    IndexUnavailable,
    SharedAllocationAliased,
    ReleaseUnderflow,
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
    /// The ledger scope that enforces the limits failed in a way that is not a refused limit.
    Accounting(HtmlAccountingFault),
}
impl HtmlConstructionErrorKind {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlConstructionError {
        HtmlConstructionError { kind: self }
    }
}

impl From<HtmlConstructionErrorKind> for HtmlConstructionError {
    fn from(kind: HtmlConstructionErrorKind) -> Self {
        Self { kind }
    }
}

impl HtmlConstructionError {
    /// Whether the cause is: node handle belongs to a different builder.
    pub fn is_foreign_builder(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::ForeignBuilder)
    }
    /// Whether the cause is: text, comment, processing-instruction and doctype nodes cannot have children.
    pub fn is_parent_cannot_have_children(&self) -> bool {
        matches!(
            self.kind,
            HtmlConstructionErrorKind::ParentCannotHaveChildren
        )
    }
    /// Whether the cause is: template children must be appended to its separate contents.
    pub fn is_template_contents_required(&self) -> bool {
        matches!(
            self.kind,
            HtmlConstructionErrorKind::TemplateContentsRequired
        )
    }
    /// Whether the cause is: a shadow root requires an element host.
    pub fn is_shadow_host_must_be_element(&self) -> bool {
        matches!(
            self.kind,
            HtmlConstructionErrorKind::ShadowHostMustBeElement
        )
    }
    /// Whether the cause is: element already has a shadow root.
    pub fn is_shadow_already_attached(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::ShadowAlreadyAttached)
    }
    /// Whether the cause is: node does not exist in this builder.
    pub fn is_unknown_node(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::UnknownNode)
    }
    /// Whether the cause is: node kind cannot take part in this command.
    pub fn is_wrong_node_kind(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::WrongNodeKind)
    }
    /// Whether the cause is: documents and fragments cannot be moved or detached.
    pub fn is_root_move(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::RootMove)
    }
    /// Whether the cause is: template contents and shadow fragments cannot be moved as ordinary children.
    pub fn is_association_move(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::AssociationMove)
    }
    /// Whether the cause is: reference node is not a child of the target parent.
    pub fn is_wrong_sibling(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::WrongSibling)
    }
    /// Whether the cause is: move would make a node its own ancestor.
    pub fn is_cycle(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::Cycle)
    }
    /// Whether the cause is: fragment is already associated with another owner.
    pub fn is_association_already_owned(&self) -> bool {
        matches!(
            self.kind,
            HtmlConstructionErrorKind::AssociationAlreadyOwned
        )
    }
    /// Whether the cause is: template contents association does not satisfy the template rules.
    pub fn is_invalid_template_association(&self) -> bool {
        matches!(
            self.kind,
            HtmlConstructionErrorKind::InvalidTemplateAssociation
        )
    }
    /// Whether the cause is: attribute candidate name does not match the planned name.
    pub fn is_attribute_name_mismatch(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::AttributeNameMismatch)
    }
    /// Whether the cause is: attribute candidates were not presented in order.
    pub fn is_attribute_order_mismatch(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::AttributeOrderMismatch)
    }
    /// Whether the cause is: shadow association does not satisfy the shadow host rules.
    pub fn is_invalid_shadow_association(&self) -> bool {
        matches!(
            self.kind,
            HtmlConstructionErrorKind::InvalidShadowAssociation
        )
    }
    /// Whether the cause is: replacement template contents are the existing contents.
    pub fn is_invalid_contents_reference(&self) -> bool {
        matches!(
            self.kind,
            HtmlConstructionErrorKind::InvalidContentsReference
        )
    }
    /// Whether the cause is: parent and child records disagree about their relationship.
    pub fn is_parent_child_conflict(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::ParentChildConflict)
    }
    /// Whether the cause is: published output root is not a detached document or fragment.
    pub fn is_invalid_output_root(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::InvalidOutputRoot)
    }
    /// Whether the cause is: construction operation still has pending actions.
    pub fn is_operation_not_ready(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::OperationNotReady)
    }
    /// Whether the cause is: retired storage does not belong to this result.
    pub fn is_foreign_allocation(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::ForeignAllocation)
    }
    /// Whether the cause is: observed capacity does not match the pending reservation.
    pub fn is_capacity_mismatch(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::CapacityMismatch)
    }
    /// Whether the cause is: the name index lent to an attribute edit could not serve a step.
    pub fn is_index_unavailable(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::IndexUnavailable)
    }
    /// Whether the cause is: result storage is shared and cannot be recovered.
    pub fn is_shared_allocation_aliased(&self) -> bool {
        matches!(
            self.kind,
            HtmlConstructionErrorKind::SharedAllocationAliased
        )
    }
    /// Whether the cause is: the builder's own arithmetic released more than it admitted. A
    /// release that the ledger refuses is reported by [`Self::accounting_fault`].
    pub fn is_release_underflow(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::ReleaseUnderflow)
    }
    /// Whether the cause is: storage count cannot be represented on this platform.
    pub fn is_representation_exhausted(&self) -> bool {
        matches!(
            self.kind,
            HtmlConstructionErrorKind::RepresentationExhausted
        )
    }
    /// Whether the cause is: reserving arena or edge storage failed. A ledger that cannot allocate
    /// is reported by [`Self::accounting_fault`].
    pub fn is_allocation_failed(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::AllocationFailed)
    }
    /// The accounting fault, when the cause is a failure of the ledger scope that enforces the
    /// limits that is not a refused limit: a release it refused, an allocation it could not make or
    /// a state its own sequencing rules out.
    pub fn accounting_fault(&self) -> Option<HtmlAccountingFault> {
        match self.kind {
            HtmlConstructionErrorKind::Accounting(fault) => Some(fault),
            _ => None,
        }
    }
    /// Whether two attributes share a namespace and local name.
    pub fn is_duplicate_attribute(&self) -> bool {
        matches!(
            self.kind,
            HtmlConstructionErrorKind::DuplicateAttribute { .. }
        )
    }
    /// The indices of the first and the duplicate attribute, when the cause is a duplicate.
    pub fn duplicate_attribute_indices(&self) -> Option<(usize, usize)> {
        match self.kind {
            HtmlConstructionErrorKind::DuplicateAttribute { first, duplicate } => {
                Some((first, duplicate))
            }
            _ => None,
        }
    }
    /// Whether a finite resource limit would be exceeded.
    pub fn is_limit_exceeded(&self) -> bool {
        matches!(self.kind, HtmlConstructionErrorKind::LimitExceeded { .. })
    }
    /// The resource that would exceed its limit, when the cause is a limit.
    pub fn exceeded_resource(&self) -> Option<HtmlConstructionResource> {
        match self.kind {
            HtmlConstructionErrorKind::LimitExceeded { resource, .. } => Some(resource),
            _ => None,
        }
    }
    /// The configured limit, when the cause is a limit.
    pub fn limit(&self) -> Option<usize> {
        match self.kind {
            HtmlConstructionErrorKind::LimitExceeded { limit, .. } => Some(limit),
            _ => None,
        }
    }
    /// The attempted total, when the cause is a limit.
    pub fn attempted(&self) -> Option<usize> {
        match self.kind {
            HtmlConstructionErrorKind::LimitExceeded { attempted, .. } => Some(attempted),
            _ => None,
        }
    }
}

impl fmt::Display for HtmlConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use HtmlConstructionErrorKind as Kind;
        match &self.kind {
            Kind::ForeignBuilder => f.write_str("node handle belongs to a different builder"),
            Kind::ParentCannotHaveChildren => f.write_str(
                "text, comment, processing-instruction and doctype nodes cannot have children",
            ),
            Kind::TemplateContentsRequired => {
                f.write_str("template children must be appended to its separate contents")
            }
            Kind::ShadowHostMustBeElement => f.write_str("a shadow root requires an element host"),
            Kind::ShadowAlreadyAttached => f.write_str("element already has a shadow root"),
            Kind::UnknownNode => f.write_str("node does not exist in this builder"),
            Kind::WrongNodeKind => f.write_str("node kind cannot take part in this command"),
            Kind::RootMove => f.write_str("documents and fragments cannot be moved or detached"),
            Kind::AssociationMove => f.write_str(
                "template contents and shadow fragments cannot be moved as ordinary children",
            ),
            Kind::WrongSibling => f.write_str("reference node is not a child of the target parent"),
            Kind::Cycle => f.write_str("move would make a node its own ancestor"),
            Kind::AssociationAlreadyOwned => {
                f.write_str("fragment is already associated with another owner")
            }
            Kind::InvalidTemplateAssociation => {
                f.write_str("template contents association does not satisfy the template rules")
            }
            Kind::AttributeNameMismatch => {
                f.write_str("attribute candidate name does not match the planned name")
            }
            Kind::AttributeOrderMismatch => {
                f.write_str("attribute candidates were not presented in order")
            }
            Kind::InvalidShadowAssociation => {
                f.write_str("shadow association does not satisfy the shadow host rules")
            }
            Kind::InvalidContentsReference => {
                f.write_str("replacement template contents are the existing contents")
            }
            Kind::ParentChildConflict => {
                f.write_str("parent and child records disagree about their relationship")
            }
            Kind::InvalidOutputRoot => {
                f.write_str("published output root is not a detached document or fragment")
            }
            Kind::Accounting(fault) => write!(
                f,
                "the ledger scope that enforces the limits failed: {fault}"
            ),
            Kind::OperationNotReady => {
                f.write_str("construction operation still has pending actions")
            }
            Kind::ForeignAllocation => {
                f.write_str("retired storage does not belong to this result")
            }
            Kind::CapacityMismatch => {
                f.write_str("observed capacity does not match the pending reservation")
            }
            Kind::IndexUnavailable => {
                f.write_str("the attribute name index lent to the edit could not serve a step")
            }
            Kind::SharedAllocationAliased => {
                f.write_str("result storage is shared and cannot be recovered")
            }
            Kind::ReleaseUnderflow => f.write_str("released usage exceeds the admitted usage"),
            Kind::RepresentationExhausted => {
                f.write_str("storage count cannot be represented on this platform")
            }
            Kind::AllocationFailed => f.write_str("reserving arena or edge storage failed"),
            Kind::DuplicateAttribute { first, duplicate } => write!(
                f,
                "attribute at index {duplicate} duplicates the namespace and local name of the attribute at index {first}"
            ),
            Kind::LimitExceeded {
                resource,
                limit,
                attempted,
            } => match resource {
                HtmlConstructionResource::Depth => write!(
                    f,
                    "depth limit of {limit} cannot admit a depth of {attempted}"
                ),
                _ => write!(
                    f,
                    "{resource} limit of {limit} cannot admit a total of {attempted}"
                ),
            },
        }
    }
}
impl Error for HtmlConstructionError {}

impl fmt::Display for HtmlConstructionResource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Nodes => "node",
            Self::Edges => "edge",
            Self::Attributes => "attribute",
            Self::Utf16Units => "UTF-16 unit",
            Self::Depth => "depth",
        })
    }
}

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
    identity: Weak<Identity>,
    index: usize,
}
impl PartialEq for HtmlBuilderNode {
    fn eq(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.identity, &other.identity) && self.index == other.index
    }
}
impl Eq for HtmlBuilderNode {}
impl Hash for HtmlBuilderNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.identity.as_ptr().hash(state);
        self.index.hash(state);
    }
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
/// use surgeist_html::construction::{HtmlAuthoredProcessingInstruction, HtmlConstructionLimits,
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
    /// Stores exact recovered target and data without native-author syntax checks.
    pub fn new(target: impl Into<DocumentString>, data: impl Into<DocumentString>) -> Self {
        Self {
            target: target.into(),
            data: data.into(),
        }
    }
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
                .map(crate::internal::microsyntax::tokens::ascii_lowercase)
                .eq(text.bytes().map(u16::from))
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
#[non_exhaustive]
pub enum HtmlProcessingInstructionError {
    /// The target is empty or contains a character outside the HTML processing-instruction target
    /// syntax (ASCII alphanumerics, `-` and `_`, starting with an ASCII letter or `_`).
    InvalidTarget,
    /// The target is, ignoring ASCII case, `xml` or `xml-stylesheet`, which HTML
    /// processing-instruction syntax excludes.
    ReservedTarget,
    /// The data contains U+003E (`>`), which processing-instruction data must not contain.
    DataContainsGreaterThan,
    /// The data ends with U+003F (`?`), which processing-instruction data must not end with.
    DataEndsWithQuestionMark,
}
impl fmt::Display for HtmlProcessingInstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidTarget => "processing-instruction target is not a valid XML name",
            Self::ReservedTarget => {
                "processing-instruction target is reserved (xml or xml-stylesheet)"
            }
            Self::DataContainsGreaterThan => "processing-instruction data contains '>'",
            Self::DataEndsWithQuestionMark => "processing-instruction data ends with '?'",
        })
    }
}
impl Error for HtmlProcessingInstructionError {}

/// Exact element name and ordered raw attributes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlElement {
    link_flags: Option<crate::internal::elements::rel::HtmlLinkFlags>,
    name: QualifiedName,
    attributes: OrderedAttributes,
}
impl HtmlElement {
    /// Stores exact element values without fabricating construction history.
    pub fn new(name: QualifiedName, attributes: OrderedAttributes) -> Self {
        Self {
            name,
            attributes,
            link_flags: None,
        }
    }
    /// Actual construction flags. Imported records without history return None.
    pub fn link_flags(&self) -> Option<crate::internal::elements::rel::HtmlLinkFlags> {
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
    /// The Document node, carrying the compatibility mode selected when it was built.
    Document(HtmlDocumentMode),
    /// A document-fragment root.
    Fragment,
    /// An element with its exact qualified name and ordered raw attributes.
    Element(HtmlElement),
    /// A text node; the payload preserves the exact UTF-16 code units.
    Text(DocumentString),
    /// A comment node; the payload preserves the exact UTF-16 code units.
    Comment(DocumentString),
    /// A processing instruction recorded with its exact target and data.
    ProcessingInstruction(HtmlProcessingInstruction),
    /// A document type declaration.
    Doctype(HtmlDoctype),
}

/// Declarative shadow-root visibility.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlShadowRootMode {
    /// The shadow root is open, so script on the host can reach it.
    Open,
    /// The shadow root is closed, so script on the host cannot reach it.
    Closed,
}
/// Declarative shadow-root slot assignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlSlotAssignment {
    /// Slots are assigned by name: a slottable goes to the slot whose `name` matches its `slot`
    /// attribute.
    Named,
    /// Slot assignment is manual: slottables are assigned only by explicit requests on the slot.
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
/// A borrowed local shadow association with properties owned by its fragment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlDeclarativeShadowRoot<'a> {
    host: &'a HtmlNodeId,
    contents: &'a HtmlNodeId,
    properties: &'a HtmlShadowRootProperties,
}
impl<'a> HtmlDeclarativeShadowRoot<'a> {
    /// Host element identity.
    pub fn host(&self) -> &'a HtmlNodeId {
        self.host
    }
    /// Associated fragment root identity.
    pub fn contents(&self) -> &'a HtmlNodeId {
        self.contents
    }
    /// Actual properties retained once with the shadow contents.
    pub fn properties(&self) -> HtmlShadowRootProperties {
        *self.properties
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct ShadowAssociation {
    host: HtmlNodeId,
    contents: HtmlNodeId,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TemplateContents {
    Owned(usize),
    ShadowReference(usize),
}
impl TemplateContents {
    fn position(self) -> usize {
        match self {
            Self::Owned(i) | Self::ShadowReference(i) => i,
        }
    }
    fn kind(self) -> HtmlTemplateContentsKind {
        match self {
            Self::Owned(_) => HtmlTemplateContentsKind::Owned,
            Self::ShadowReference(_) => HtmlTemplateContentsKind::ShadowReference,
        }
    }
}
/// The non-child relationship that owns an associated fragment.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlFragmentOwner {
    /// The owner is the `template` element, which owns its separate ordinary contents fragment.
    /// The payload is the template element's node identity.
    Template(HtmlNodeId),
    /// The owner is a shadow host element, which owns the shadow root contents fragment associated
    /// with it. The payload is the host element's node identity.
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

/// Original focused-area designation retained by an owned construction Document.
/// This supplies no viewport, positive focusing operation or execution permission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFocusedAreaDesignation {
    /// The actual controlled Document constructor established null, preserved
    /// through this owner's admitted operations.
    KnownNull,
    /// No qualified original designation history is retained.
    Unavailable,
}

#[derive(Debug)]
struct Record {
    kind: HtmlNodeKind,
    document_focused_area_designation: Option<HtmlFocusedAreaDesignation>,
    parent: Option<usize>,
    /// The ordinary children in order. A deque, so that taking or placing an entry near either
    /// end shifts no more entries than lie between it and the nearer end.
    children: VecDeque<usize>,
    children_generation: u64,
    /// Depth of this record within its own tree, counting association edges;
    /// the root of every tree, attached or detached, is at zero. Every
    /// admitted change keeps it exact.
    depth: usize,
    /// How many records of the arena have a depth equal to this record's
    /// *index*. The depth histogram needs no allocation of its own: a record
    /// at depth `d` implies a chain of `d + 1` records, so index `d` exists
    /// whenever the bucket for `d` is nonempty. Only depth placement writes it.
    population: usize,
    /// Scratch: next edge a subtree walk takes from this record. A walk sets it
    /// on entry, so a stale value is never trusted.
    walk_cursor: usize,
    /// Scratch: how many records of the walked subtree sit at a level equal to
    /// this record's *index*. A walk initializes a level on first reaching it.
    level_count: usize,
    template_contents: Option<TemplateContents>,
    shadow: Option<ShadowAssociation>,
    shadow_properties: Option<HtmlShadowRootProperties>,
    owner: Option<HtmlFragmentOwner>,
    /// Scratch count of this record's entries in its parent's child list,
    /// saturating at two. Only a publication check writes it: reset, then
    /// counted, then read, so a stale value is never trusted.
    publish_found: u8,
}
impl Record {
    fn new(kind: HtmlNodeKind, parent: Option<usize>, depth: usize) -> Self {
        let document_focused_area_designation = matches!(kind, HtmlNodeKind::Document(_))
            .then_some(HtmlFocusedAreaDesignation::Unavailable);
        Self {
            kind,
            document_focused_area_designation,
            parent,
            children: VecDeque::new(),
            children_generation: 0,
            depth,
            population: 0,
            walk_cursor: 0,
            level_count: 0,
            template_contents: None,
            shadow: None,
            shadow_properties: None,
            owner: None,
            publish_found: 0,
        }
    }
}
#[derive(Debug)]
struct Arena {
    identity: Arc<Identity>,
    records: Vec<Record>,
    usage: HtmlConstructionUsage,
    records_generation: u64,
    wrapper_generation: u64,
    // Shared never-reused namespace for child-bank incarnations and temporary
    // banks, including every successful reallocation or child-bank replacement.
    temporary_generation: u64,
    // Names this storage among all storages, so that a table of attribute names can tell the
    // attributes it describes from those of another storage.
    serial: u64,
    // Advances with every committed change to the attributes of any element of this storage,
    // whichever edit made it, so that a table of attribute names can tell the attributes it
    // describes from those that have changed since.
    attribute_revision: u64,
}
impl Arena {
    /// An arena with no records and the given identity. A move out of an arena that sits behind a
    /// `&mut` leaves this behind; it allocates nothing and is dropped by the same step.
    fn vacated(identity: &Arc<Identity>) -> Self {
        Self {
            identity: Arc::clone(identity),
            records: Vec::new(),
            usage: HtmlConstructionUsage::default(),
            records_generation: 0,
            wrapper_generation: 0,
            temporary_generation: 0,
            serial: 0,
            attribute_revision: 0,
        }
    }
    /// How many records have exactly this depth.
    fn population(&self, depth: usize) -> usize {
        self.records
            .get(depth)
            .map_or(0, |bucket| bucket.population)
    }
    /// Counts one more record at `depth`. The bucket lives in the record at
    /// that index, which exists whenever a record sits at `depth`.
    fn populate(&mut self, depth: usize) {
        if let Some(bucket) = self.records.get_mut(depth) {
            bucket.population = bucket.population.saturating_add(1);
        }
    }
    fn unpopulate(&mut self, depth: usize) {
        if let Some(bucket) = self.records.get_mut(depth) {
            bucket.population = bucket.population.saturating_sub(1);
        }
    }
    /// Gives the record a new depth and moves it between histogram buckets.
    fn place(&mut self, index: usize, depth: usize) {
        let Some(record) = self.records.get_mut(index) else {
            return;
        };
        let old = std::mem::replace(&mut record.depth, depth);
        self.unpopulate(old);
        self.populate(depth);
    }
    fn id(&self, index: usize) -> HtmlNodeId {
        HtmlNodeId {
            identity: self.identity.clone(),
            index,
        }
    }
    fn lookup(&self, id: &HtmlNodeId) -> Result<HtmlNode<'_>, ForeignHtmlResult> {
        if !Arc::ptr_eq(&self.identity, &id.identity) || id.index >= self.records.len() {
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
    /// Original design-mode state of this owned Document, or `None` for a
    /// non-Document. Controlled construction initializes Documents with design
    /// mode disabled and admits no operation that changes it. Captured external
    /// Documents are not owned construction nodes and receive no such grant.
    pub fn document_design_mode_enabled(self) -> Option<bool> {
        matches!(self.kind(), HtmlNodeKind::Document(_)).then_some(false)
    }
    /// Stored original designation of this owned Document, or `None` for a
    /// non-Document. Only the controlled CreateDocument transition issues known
    /// null. Its closed operation domain preserves that state through recovery
    /// and publication; captured external Documents receive no such grant.
    pub fn document_focused_area_designation(self) -> Option<HtmlFocusedAreaDesignation> {
        self.arena.records[self.index].document_focused_area_designation
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
            .map(|contents| self.arena.id(contents.position()))
    }
    /// Native declarative shadow association, if supplied. A declarative shadow root is the
    /// association whose contents carry shadow properties, and publication refuses any `shadow`
    /// relation whose contents lack them (`validate_record`, `InvalidShadowAssociation`), so a
    /// published tree has no association that this answers with `None`.
    pub fn declarative_shadow_root(self) -> Option<HtmlDeclarativeShadowRoot<'a>> {
        let relation = self.arena.records[self.index].shadow.as_ref()?;
        // A declarative shadow root is an association whose contents carry shadow properties.
        let properties = self
            .arena
            .records
            .get(relation.contents.index)?
            .shadow_properties
            .as_ref()?;
        Some(HtmlDeclarativeShadowRoot {
            host: &relation.host,
            contents: &relation.contents,
            properties,
        })
    }
    /// Distinguishes an owning template association from a shadow reference.
    pub fn template_contents_kind(self) -> Option<HtmlTemplateContentsKind> {
        self.arena.records[self.index]
            .template_contents
            .map(TemplateContents::kind)
    }
    /// Whether this fragment was explicitly constructed as shadow contents.
    pub fn is_shadow_contents(self) -> bool {
        self.shadow_properties().is_some()
    }
    /// Actual shadow properties, including when no local host exists.
    pub fn shadow_properties(self) -> Option<HtmlShadowRootProperties> {
        self.arena.records[self.index].shadow_properties
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
    root: usize,
    /// The mode the published root Document had, which an immutable arena cannot change.
    mode: HtmlDocumentMode,
}
impl HtmlDocument {
    /// Begins bounded native document construction with an explicit mode.
    pub fn builder(
        limits: HtmlConstructionLimits,
        mode: HtmlDocumentMode,
    ) -> Result<HtmlDocumentBuilder, HtmlConstructionError> {
        Ok(HtmlDocumentBuilder {
            builder: Builder::new(limits, NativeRoot::Document(mode))?,
        })
    }
    /// Document root.
    pub fn root(&self) -> HtmlNode<'_> {
        HtmlNode {
            arena: &self.arena,
            index: self.root,
        }
    }
    /// Explicit document mode, unaffected by authored doctype text.
    pub fn mode(&self) -> HtmlDocumentMode {
        self.mode
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
            builder: Builder::new(limits, NativeRoot::Fragment)?,
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

/// The root a native builder creates: the closed set of kinds that can be published.
#[derive(Clone, Copy, Debug)]
enum NativeRoot {
    Document(HtmlDocumentMode),
    Fragment,
}

/// The capacity a reservation takes when it may grow with spare room: the amortized capacity of the
/// bank, never above `ceiling`, the most entries the limits admit, and never below what the action
/// needs. The target depends on the bank and the limits only, and the reservation takes exactly
/// it. `None` for an action that reserves no such bank.
///
/// Unlike [`crate::internal::amortized::amortized_capacity`], whose result never depends on a
/// limit, this target is capped at a limit. The native builder admits nothing beyond its node and
/// edge limits and has no capacity-bytes limit, so a capacity above the entries those limits admit
/// is spare room that no append can use; capping it there is the admission of the allocation.
fn amortized_target(
    action: &bounded::operations::HtmlConstructionAction<'_>,
    ceiling: usize,
) -> Option<usize> {
    let bounded::operations::HtmlConstructionActionDescription::Reserve {
        allocation,
        minimum_capacity,
        growth: bounded::operations::HtmlConstructionGrowth::Geometric,
        ..
    } = action.description()
    else {
        return None;
    };
    Some(
        crate::internal::amortized::amortized_capacity(
            allocation.initialized_len(),
            allocation.capacity(),
        )
        .min(ceiling)
        .max(minimum_capacity),
    )
}

/// Prepares `command` on `storage`, drives it to its ready state and commits it. The storage comes
/// back on every outcome, so a refused step never loses what the builder owns. It is absent only
/// when the command published, which no native mutation does, or when an abort was refused, which
/// a mutation's unshared storage cannot be; the first failure is reported either way.
///
/// With `spare_room`, a reservation that may take spare room (a child order) grows to the
/// amortized capacity the limit `spare_room` allows; without it, to the least capacity that
/// suffices.
fn drive_native(
    storage: bounded::HtmlConstructionStorage,
    command: bounded::operations::HtmlConstructionCommand,
    spare_room: Option<usize>,
) -> (
    Option<bounded::HtmlConstructionStorage>,
    Result<Option<HtmlBuilderNode>, HtmlConstructionError>,
) {
    let mut operation = storage.prepare(command);
    let driven = (|| {
        while let Some(action) = operation.next_action()? {
            let capacity = spare_room.and_then(|ceiling| amortized_target(&action, ceiling));
            let performed = match capacity {
                Some(capacity) => {
                    #[cfg(test)]
                    if crate::internal::amortized::fault::refuses_next_reservation() {
                        return Err(HtmlConstructionErrorKind::AllocationFailed.into());
                    }
                    action.perform_with_capacity(capacity)
                }
                None => action.perform(),
            };
            performed
                .map_err(|failure| failure.into_parts().0)?
                .accept();
        }
        Ok::<_, HtmlConstructionError>(())
    })();
    if let Err(error) = driven {
        return (
            operation.abort().ok().map(|abort| abort.into_parts().0),
            Err(error),
        );
    }
    match ready_or_recover(operation) {
        Err((storage, error)) => (storage, Err(error)),
        Ok(ready) => match ready.commit() {
            Ok(bounded::operations::HtmlConstructionCommit::Changed { storage, created }) => {
                (Some(storage), Ok(created))
            }
            // A publication moves the arena into its shared result. Native commands never publish;
            // were one to, the sole owner takes the storage back through its lease.
            Ok(bounded::operations::HtmlConstructionCommit::Published { result, lease }) => (
                bounded::HtmlConstructionLease::try_recover(result, lease)
                    .ok()
                    .map(|recovery| recovery.into_parts().0),
                Err(HtmlConstructionErrorKind::OperationNotReady.into()),
            ),
            Err(refusal) => {
                let (ready, error) = refusal.into_parts();
                (
                    ready.abort().ok().map(|abort| abort.into_parts().0),
                    Err(error),
                )
            }
        },
    }
}

/// Readies a driven operation. A refusal aborts the operation the refusal hands back and returns
/// its storage with the cause, so no storage is dropped on the error path.
#[expect(
    clippy::result_large_err,
    reason = "a refusal hands the complete storage back without boxing it"
)]
fn ready_or_recover(
    operation: bounded::operations::HtmlConstructionOperation,
) -> Result<
    bounded::operations::HtmlConstructionReadyOperation,
    (
        Option<bounded::HtmlConstructionStorage>,
        HtmlConstructionError,
    ),
> {
    operation.try_ready().map_err(|refusal| {
        let (operation, error) = refusal.into_parts();
        (
            operation.abort().ok().map(|abort| abort.into_parts().0),
            error,
        )
    })
}

/// The storage a builder holds, or the fact that an operation could not hand it back.
#[derive(Debug)]
enum BuilderStorage {
    Held(bounded::HtmlConstructionStorage),
    /// An operation took the storage and its abort was refused, so no storage remains. No native
    /// command can cause this, since only a publication shares its arena. Every handle operation
    /// and `finish` report the shared-allocation cause, and `usage` reports what the builder's
    /// accounting admitted.
    Lost,
}

/// The construction scope of a native builder: the caps of its limits, enforced by a ledger scope,
/// and the typed usage read back from that scope after each charge.
struct Accounting {
    operation: Operation<HtmlWork, HtmlResource>,
    scope: Scope,
    usage: HtmlConstructionUsage,
    limits: HtmlConstructionLimits,
}
impl fmt::Debug for Accounting {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Accounting")
            .field("usage", &self.usage)
            .finish_non_exhaustive()
    }
}
/// What one admitted append charged, so a rejected append can give it back.
#[derive(Clone, Copy, Debug)]
struct Charged {
    nodes: usize,
    edges: usize,
    attributes: usize,
    units: usize,
    /// The depth the append raised, with the depth it left behind.
    depth: Option<usize>,
}
impl Accounting {
    fn new(limits: HtmlConstructionLimits) -> Result<Self, HtmlConstructionError> {
        let operation = Operation::new(
            HtmlCapScope::Construction.name(),
            crate::internal::accounting::construction_limits(limits),
            1,
        )
        .map_err(ledger_error)?;
        let scope = operation.root();
        Ok(Self {
            operation,
            scope,
            usage: HtmlConstructionUsage::default(),
            limits,
        })
    }
    /// Charges the amounts of one append as one all-or-nothing batch, in the order of the limits'
    /// precedence: nodes, edges, attributes, UTF-16 units, then depth. A depth no deeper than the
    /// deepest admitted node charges nothing.
    fn admit(
        &mut self,
        nodes: usize,
        edges: usize,
        attributes: usize,
        units: usize,
        depth: usize,
    ) -> Result<Charged, HtmlConstructionError> {
        let held = self.usage.depth;
        let raises = depth > held;
        let batch = [
            Change::Acquire(HtmlResource::ManagedNodes, wide(nodes)),
            Change::Acquire(HtmlResource::ManagedEdges, wide(edges)),
            Change::Acquire(HtmlResource::ManagedAttributes, wide(attributes)),
            Change::Acquire(HtmlResource::ManagedUtf16Units, wide(units)),
            Change::Release(HtmlResource::TreeDepth, wide(held)),
            Change::Acquire(HtmlResource::TreeDepth, wide(depth)),
        ];
        let Some(batch) = batch.get(..if raises { 6 } else { 4 }) else {
            return Err(HtmlConstructionErrorKind::Accounting(
                HtmlAccountingFault::UnexpectedChange,
            )
            .into());
        };
        self.operation
            .apply_batch(self.scope, batch)
            .map_err(ledger_error)?;
        self.read()?;
        Ok(Charged {
            nodes,
            edges,
            attributes,
            units,
            depth: raises.then_some(held),
        })
    }
    /// Gives back what an append charged and restores the depth it raised.
    fn release(&mut self, charged: Charged) -> Result<(), HtmlConstructionError> {
        self.operation
            .apply_batch(
                self.scope,
                &[
                    Change::Release(HtmlResource::ManagedNodes, wide(charged.nodes)),
                    Change::Release(HtmlResource::ManagedEdges, wide(charged.edges)),
                    Change::Release(HtmlResource::ManagedAttributes, wide(charged.attributes)),
                    Change::Release(HtmlResource::ManagedUtf16Units, wide(charged.units)),
                ],
            )
            .map_err(ledger_error)?;
        if let Some(previous) = charged.depth {
            self.operation
                .charge_at(self.scope)
                .record_largest(HtmlResource::TreeDepth, wide(previous))
                .map_err(ledger_error)?;
        }
        self.read()
    }
    /// Reads the typed usage from the scope.
    fn read(&mut self) -> Result<(), HtmlConstructionError> {
        let usage = self.operation.usage(self.scope).map_err(ledger_error)?;
        let read = |value: u64| {
            usize::try_from(value).map_err(|_| HtmlConstructionErrorKind::RepresentationExhausted)
        };
        self.usage = HtmlConstructionUsage {
            nodes: read(usage.live(HtmlResource::ManagedNodes))?,
            edges: read(usage.live(HtmlResource::ManagedEdges))?,
            attributes: read(usage.live(HtmlResource::ManagedAttributes))?,
            utf16_units: read(usage.live(HtmlResource::ManagedUtf16Units))?,
            depth: read(usage.live(HtmlResource::TreeDepth))?,
        };
        Ok(())
    }
}
/// The construction error of a ledger error: a refused limit keeps its resource, limit and the
/// total the charge would have reached.
fn ledger_error(error: LedgerError<HtmlWork, HtmlResource>) -> HtmlConstructionError {
    let kind = match classify(error) {
        HtmlLedgerOutcome::Fault(fault) => HtmlConstructionErrorKind::Accounting(fault),
        HtmlLedgerOutcome::Exhausted(_) => HtmlConstructionErrorKind::RepresentationExhausted,
        HtmlLedgerOutcome::Cap(refusal) => {
            let resource = match refusal.change() {
                Change::Acquire(HtmlResource::ManagedNodes, _) => {
                    Some(HtmlConstructionResource::Nodes)
                }
                Change::Acquire(HtmlResource::ManagedEdges, _) => {
                    Some(HtmlConstructionResource::Edges)
                }
                Change::Acquire(HtmlResource::ManagedAttributes, _) => {
                    Some(HtmlConstructionResource::Attributes)
                }
                Change::Acquire(HtmlResource::ManagedUtf16Units, _) => {
                    Some(HtmlConstructionResource::Utf16Units)
                }
                Change::Acquire(HtmlResource::TreeDepth, _) => {
                    Some(HtmlConstructionResource::Depth)
                }
                _ => None,
            };
            match (resource, refusal.figures()) {
                (Some(resource), Ok(figures)) => HtmlConstructionErrorKind::LimitExceeded {
                    resource,
                    limit: figures.limit,
                    attempted: figures.required,
                },
                (None, _) => {
                    HtmlConstructionErrorKind::Accounting(HtmlAccountingFault::UnexpectedChange)
                }
                (Some(_), Err(_)) => HtmlConstructionErrorKind::RepresentationExhausted,
            }
        }
    };
    kind.into()
}

#[derive(Debug)]
struct Builder {
    storage: BuilderStorage,
    /// The root issued when the builder was created, which the storage keeps for its life.
    root: HtmlBuilderNode,
    accounting: Accounting,
}
impl Builder {
    fn storage(&self) -> Result<&bounded::HtmlConstructionStorage, HtmlConstructionError> {
        match &self.storage {
            BuilderStorage::Held(storage) => Ok(storage),
            BuilderStorage::Lost => Err(HtmlConstructionErrorKind::SharedAllocationAliased.into()),
        }
    }
    /// The usage the builder's accounting admitted.
    fn usage(&self) -> HtmlConstructionUsage {
        self.accounting.usage
    }
    /// Hands the storage to an operation. The builder holds none until the operation hands it
    /// back.
    fn take_storage(&mut self) -> Result<bounded::HtmlConstructionStorage, HtmlConstructionError> {
        match std::mem::replace(&mut self.storage, BuilderStorage::Lost) {
            BuilderStorage::Held(storage) => Ok(storage),
            BuilderStorage::Lost => Err(HtmlConstructionErrorKind::SharedAllocationAliased.into()),
        }
    }
    fn new(
        limits: HtmlConstructionLimits,
        root: NativeRoot,
    ) -> Result<Self, HtmlConstructionError> {
        let mut initialization = bounded::HtmlConstructionStorage::begin();
        while let Some(action) = initialization.next_action()? {
            action.perform().accept();
        }
        let storage = initialization
            .try_ready()
            .map_err(|refusal| refusal.into_parts().1)?
            .commit();
        let mut accounting = Accounting::new(limits)?;
        accounting.admit(1, 0, 0, 0, 0)?;
        let command = match root {
            NativeRoot::Document(mode) => {
                bounded::operations::HtmlConstructionCommand::CreateDocument { mode }
            }
            NativeRoot::Fragment => bounded::operations::HtmlConstructionCommand::CreateFragment,
        };
        let (storage, outcome) = drive_native(storage, command, None);
        let root = outcome?.ok_or(HtmlConstructionErrorKind::OperationNotReady)?;
        Ok(Self {
            storage: BuilderStorage::Held(
                storage.ok_or(HtmlConstructionErrorKind::SharedAllocationAliased)?,
            ),
            root,
            accounting,
        })
    }
    fn index(&self, node: &HtmlBuilderNode) -> Result<usize, HtmlConstructionError> {
        self.storage()?.position(node)
    }
    /// Runs `command` on the storage, growing a child order it fills with the amortized capacity
    /// that the edge limit allows. See [`Builder::run`].
    fn apply(
        &mut self,
        command: bounded::operations::HtmlConstructionCommand,
    ) -> Result<Option<HtmlBuilderNode>, HtmlConstructionError> {
        let edges = self.accounting.limits.edges();
        self.run(command, Some(edges))
    }
    /// Runs `command` on the storage. The storage moves into the operation and comes back on every
    /// outcome. It stays lost only if the operation could not hand it back, which no native command
    /// can cause. `spare_room` is as for [`drive_native`].
    fn run(
        &mut self,
        command: bounded::operations::HtmlConstructionCommand,
        spare_room: Option<usize>,
    ) -> Result<Option<HtmlBuilderNode>, HtmlConstructionError> {
        let storage = self.take_storage()?;
        let (storage, outcome) = drive_native(storage, command, spare_room);
        if let Some(storage) = storage {
            self.storage = BuilderStorage::Held(storage);
        }
        outcome
    }
    /// Appends `node` as the last child of `parent`. A refusal of the spare room that the child
    /// order would take is retried with the least capacity that suffices, so the append fails only
    /// where growing by one entry fails.
    fn insert_last(
        &mut self,
        node: &HtmlBuilderNode,
        parent: &HtmlBuilderNode,
    ) -> Result<(), HtmlConstructionError> {
        let insert = || bounded::operations::HtmlConstructionCommand::Insert {
            node: node.clone(),
            parent: parent.clone(),
            before: None,
        };
        match self.apply(insert()) {
            Err(error) if error.is_allocation_failed() => self.run(insert(), None),
            outcome => outcome,
        }
        .map(|_| ())
    }
    /// Makes room in the record bank for `additional` more records, with the amortized spare room
    /// that the node limit allows (capped at the limit for the reason given at
    /// [`amortized_target`]). A refusal of the spare room is retried with the least capacity
    /// that suffices. A bank that already has the room is left as it is.
    fn reserve_records(&mut self, additional: usize) -> Result<(), HtmlConstructionError> {
        let storage = self.storage()?;
        let (len, capacity) = (storage.len(), storage.records_capacity());
        let required = len
            .checked_add(additional)
            .ok_or(HtmlConstructionErrorKind::RepresentationExhausted)?;
        if required <= capacity {
            return Ok(());
        }
        let target = crate::internal::amortized::amortized_capacity(len, capacity)
            .min(self.accounting.limits.nodes())
            .max(required);
        let reserve =
            |capacity: usize| bounded::operations::HtmlConstructionCommand::ReserveRecords {
                additional: capacity - len,
            };
        #[cfg(test)]
        let refused =
            target > required && crate::internal::amortized::fault::refuses_next_reservation();
        #[cfg(not(test))]
        let refused = false;
        let outcome = if refused {
            Err(HtmlConstructionErrorKind::AllocationFailed.into())
        } else {
            self.run(reserve(target), None)
        };
        match outcome {
            Err(error) if error.is_allocation_failed() && target > required => {
                self.run(reserve(required), None)
            }
            outcome => outcome,
        }
        .map(|_| ())
    }
    /// As `apply`, for a command that creates a record.
    fn apply_creating(
        &mut self,
        command: bounded::operations::HtmlConstructionCommand,
    ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
        self.apply(command)?
            .ok_or_else(|| HtmlConstructionErrorKind::OperationNotReady.into())
    }
    fn depth(&self, index: usize) -> Result<usize, HtmlConstructionError> {
        Ok(self.storage()?.native_depth(index))
    }
    fn append(
        &mut self,
        parent: &HtmlBuilderNode,
        kind: HtmlNodeKind,
    ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
        let parent_position = self.index(parent)?;
        let parent_view = self.storage()?.node(parent)?;
        if !matches!(
            parent_view.kind(),
            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment | HtmlNodeKind::Element(_)
        ) {
            return Err(HtmlConstructionErrorKind::ParentCannotHaveChildren.into());
        }
        if parent_view.template_contents().is_some() {
            return Err(HtmlConstructionErrorKind::TemplateContentsRequired.into());
        }
        let template = matches!(&kind, HtmlNodeKind::Element(element) if crate::vocabulary::KnownHtmlElement::from_name(element.name()) == Some(crate::vocabulary::KnownHtmlElement::Template));
        let depth = self
            .depth(parent_position)?
            .checked_add(1 + usize::from(template))
            .ok_or(HtmlConstructionErrorKind::RepresentationExhausted)?;
        let (attributes, units) = bounded::operations::created_usage(&kind)?;
        let charged = self.accounting.admit(
            1 + usize::from(template),
            1 + usize::from(template),
            attributes,
            units,
            depth,
        )?;
        let previous_usage = self.storage()?.usage();
        let previous_len = self.storage()?.len();
        if let Err(error) = self.reserve_records(1 + usize::from(template)) {
            self.accounting.release(charged)?;
            return Err(error);
        }
        let created = match self
            .apply_creating(bounded::operations::HtmlConstructionCommand::Create { kind })
        {
            Ok(created) => created,
            Err(error) => {
                self.accounting.release(charged)?;
                return Err(error);
            }
        };
        if let Err(error) = self.insert_last(&created, parent) {
            self.roll_back_append(previous_len, previous_usage);
            self.accounting.release(charged)?;
            return Err(error);
        }
        Ok(created)
    }
    /// Undoes an uncommitted append. Storage that was lost has nothing to undo.
    fn roll_back_append(&mut self, len: usize, usage: HtmlConstructionUsage) {
        if let BuilderStorage::Held(storage) = &mut self.storage {
            storage.rollback_native_append(len, usage);
        }
    }
    fn template_contents(
        &self,
        template: &HtmlBuilderNode,
    ) -> Result<Option<HtmlBuilderNode>, HtmlConstructionError> {
        Ok(self
            .storage()?
            .node(template)?
            .template_contents()
            .map(bounded::HtmlConstructionNode::handle))
    }
    fn shadow(
        &mut self,
        host: &HtmlBuilderNode,
        properties: HtmlShadowRootProperties,
    ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
        let host_position = self.index(host)?;
        let view = self.storage()?.node(host)?;
        if !matches!(view.kind(), HtmlNodeKind::Element(_)) {
            return Err(HtmlConstructionErrorKind::ShadowHostMustBeElement.into());
        }
        if view.shadow_root().is_some() {
            return Err(HtmlConstructionErrorKind::ShadowAlreadyAttached.into());
        }
        let depth = self
            .depth(host_position)?
            .checked_add(1)
            .ok_or(HtmlConstructionErrorKind::RepresentationExhausted)?;
        let charged = self.accounting.admit(1, 1, 0, 0, depth)?;
        let previous_usage = self.storage()?.usage();
        let previous_len = self.storage()?.len();
        if let Err(error) = self.reserve_records(1) {
            self.accounting.release(charged)?;
            return Err(error);
        }
        let contents = match self.apply_creating(
            bounded::operations::HtmlConstructionCommand::CreateShadowContents { properties },
        ) {
            Ok(contents) => contents,
            Err(error) => {
                self.accounting.release(charged)?;
                return Err(error);
            }
        };
        if let Err(error) = self.apply(bounded::operations::HtmlConstructionCommand::AttachShadow {
            host: host.clone(),
            contents: contents.clone(),
        }) {
            self.roll_back_append(previous_len, previous_usage);
            self.accounting.release(charged)?;
            return Err(error);
        }
        Ok(contents)
    }
    /// Publishes the whole arena. The builder is consumed: a refusal here means the tree failed
    /// the publication's own validation, which the append steps are meant to make impossible, so
    /// no half-published builder is worth keeping.
    fn finish(self) -> Result<bounded::HtmlConstructionResult, HtmlConstructionError> {
        let BuilderStorage::Held(storage) = self.storage else {
            return Err(HtmlConstructionErrorKind::SharedAllocationAliased.into());
        };
        let root = storage
            .node_at(0)
            .ok_or(HtmlConstructionErrorKind::UnknownNode)?
            .handle();
        let mut operation = storage.prepare_finish(root);
        while let Some(action) = operation.next_action()? {
            action
                .perform()
                .map_err(|failure| failure.into_parts().0)?
                .accept();
        }
        match operation
            .try_ready()
            .map_err(|refusal| refusal.into_parts().1)?
            .commit()
            .map_err(|refusal| refusal.into_parts().1)?
        {
            bounded::operations::HtmlConstructionCommit::Published { result, lease } => {
                drop(lease);
                Ok(result)
            }
            bounded::operations::HtmlConstructionCommit::Changed { .. } => {
                Err(HtmlConstructionErrorKind::OperationNotReady.into())
            }
        }
    }
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
            self.builder.root.clone()
        }
        /// Current admitted usage; rejected appends never change it.
        pub fn usage(&self) -> HtmlConstructionUsage {
            self.builder.usage()
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
                    link_flags: (crate::vocabulary::KnownHtmlElement::from_name(&name)
                        == Some(crate::vocabulary::KnownHtmlElement::Link))
                    .then(crate::internal::elements::rel::HtmlLinkFlags::fresh),
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
    ///
    /// Publication validates the whole tree again. A refusal returns the cause and ends the
    /// session; the append steps keep the tree valid, so none is known to occur.
    pub fn finish(self) -> Result<HtmlDocument, HtmlConstructionError> {
        match self.builder.finish()? {
            bounded::HtmlConstructionResult::Document(document) => Ok(document),
            bounded::HtmlConstructionResult::Fragment(_) => {
                Err(HtmlConstructionErrorKind::InvalidOutputRoot.into())
            }
        }
    }
}
impl HtmlFragmentBuilder {
    authoring_methods!();
    /// Consumes the session and freezes its entire flat arena.
    ///
    /// Publication validates the whole tree again. A refusal returns the cause and ends the
    /// session; the append steps keep the tree valid, so none is known to occur.
    pub fn finish(self) -> Result<HtmlFragment, HtmlConstructionError> {
        match self.builder.finish()? {
            bounded::HtmlConstructionResult::Fragment(fragment) => Ok(fragment),
            bounded::HtmlConstructionResult::Document(_) => {
                Err(HtmlConstructionErrorKind::InvalidOutputRoot.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// What an operation that cannot hand the storage back leaves: no native command can cause it,
    /// because only a publication shares its arena, so the test produces the state directly.
    fn lose_storage(builder: &mut Builder) {
        drop(builder.take_storage().unwrap());
    }

    /// How many times the record bank and the root's child order change capacity over `appends`
    /// native appends, each of which makes room for one more record, and one more child of the
    /// root.
    fn capacity_changes(appends: usize, nested: bool) -> (usize, usize) {
        let mut builder = Builder::new(
            HtmlConstructionLimits::new(appends + 1, appends, 0, 8 * appends, appends + 1),
            NativeRoot::Fragment,
        )
        .unwrap();
        let root = builder.root.clone();
        let mut parent = root.clone();
        let mut previous = (
            builder.storage().unwrap().records_capacity(),
            builder.storage().unwrap().children_capacity(0),
        );
        let (mut records, mut children) = (0, 0);
        for _ in 0..appends {
            let kind = if nested {
                HtmlNodeKind::Element(HtmlElement::new(
                    QualifiedName::new(Namespace::from_uri("urn:x"), None, "e"),
                    OrderedAttributes::new(Vec::new()).unwrap(),
                ))
            } else {
                HtmlNodeKind::Comment("c".into())
            };
            let appended = builder.append(&parent, kind).unwrap();
            if nested {
                parent = appended;
            }
            let storage = builder.storage().unwrap();
            let now = (storage.records_capacity(), storage.children_capacity(0));
            records += usize::from(now.0 != previous.0);
            children += usize::from(now.1 != previous.1);
            previous = now;
        }
        (records, children)
    }

    /// Exact growth changes the capacity on every append; amortized growth changes it a number of
    /// times that grows with the logarithm of the appends.
    #[test]
    fn native_appends_change_capacity_a_logarithmic_number_of_times() {
        for appends in [1_usize << 10, 1 << 12, 1 << 14] {
            let bound = appends.ilog2() as usize + 1;
            let (records, children) = capacity_changes(appends, false);
            assert!(
                records <= bound && children <= bound,
                "{appends} appended children changed the record bank {records} times and the \
                 root's child order {children} times, over the bound {bound}"
            );
            let (records, _) = capacity_changes(appends, true);
            assert!(
                records <= bound,
                "{appends} nested elements changed the record bank {records} times, over the \
                 bound {bound}"
            );
        }
    }

    /// The spare room of a reservation is never above what the limits admit: the record bank holds
    /// at most the node limit and a child order at most the edge limit.
    #[test]
    fn native_spare_room_stays_within_the_limits() {
        let (nodes, edges) = (6, 5);
        let mut builder = Builder::new(
            HtmlConstructionLimits::new(nodes, edges, 0, 8, 2),
            NativeRoot::Fragment,
        )
        .unwrap();
        let root = builder.root.clone();
        for _ in 0..edges {
            builder
                .append(&root, HtmlNodeKind::Comment("c".into()))
                .unwrap();
            let storage = builder.storage().unwrap();
            assert!(storage.records_capacity() <= nodes);
            assert!(storage.children_capacity(0).unwrap() <= edges);
        }
        assert_eq!(builder.storage().unwrap().records_capacity(), nodes);
        assert_eq!(builder.storage().unwrap().children_capacity(0), Some(edges));
    }

    /// An allocator refusal of the spare room never fails an append that growing by one slot
    /// would have allowed: every reservation falls back to the least capacity that suffices.
    #[test]
    fn a_refused_spare_room_falls_back_to_the_capacity_that_suffices() {
        let appends = 40;
        let mut builder = Builder::new(
            HtmlConstructionLimits::new(appends + 1, appends, 0, appends, 1),
            NativeRoot::Fragment,
        )
        .unwrap();
        let root = builder.root.clone();
        let _refusing = crate::internal::amortized::fault::arm(1);
        for count in 1..=appends {
            builder
                .append(&root, HtmlNodeKind::Comment("c".into()))
                .unwrap();
            let storage = builder.storage().unwrap();
            assert_eq!(storage.len(), count + 1);
            assert_eq!(storage.records_capacity(), count + 1);
            assert_eq!(storage.children_capacity(0), Some(count));
        }
        assert!(crate::internal::amortized::fault::requests() >= 2 * appends - 1);
    }

    #[test]
    fn a_builder_without_storage_reports_the_loss_and_the_usage_it_last_held() {
        let mut builder =
            HtmlFragment::builder(HtmlConstructionLimits::new(4, 4, 0, 8, 3)).unwrap();
        let root = builder.root();
        let comment = builder.append_comment(&root, "c").unwrap();
        let usage = builder.usage();
        assert_eq!(usage.utf16_units(), 1);
        lose_storage(&mut builder.builder);
        assert_eq!(
            builder.usage(),
            usage,
            "usage reports what the storage held, not an empty stand-in"
        );
        for outcome in [
            builder.append_comment(&root, "d").map(|_| ()),
            builder.append_comment(&comment, "e").map(|_| ()),
            builder.template_contents(&root).map(|_| ()),
        ] {
            let error = outcome.expect_err("a builder without storage refuses every operation");
            assert!(error.is_shared_allocation_aliased(), "{error}");
        }
        let error = builder
            .finish()
            .expect_err("a builder without storage cannot finish");
        assert!(error.is_shared_allocation_aliased(), "{error}");
    }

    /// An operation refused at its ready transition returns the storage it was prepared on, with
    /// the cause, instead of dropping it with the operation. The undriven operation is the refused
    /// state: it has pending actions, so it is not ready.
    #[test]
    fn a_refused_ready_transition_returns_the_storage_and_the_cause() {
        let mut builder = Builder::new(
            HtmlConstructionLimits::new(4, 4, 0, 8, 2),
            NativeRoot::Fragment,
        )
        .unwrap();
        let usage = builder.usage();
        let records = builder.storage().unwrap().len();
        let storage = builder.take_storage().unwrap();
        let operation =
            storage.prepare(bounded::operations::HtmlConstructionCommand::CreateFragment);
        let Err((Some(storage), error)) = ready_or_recover(operation) else {
            panic!("an undriven operation is not ready");
        };
        assert!(error.is_operation_not_ready());
        assert_eq!((storage.usage(), storage.len()), (usage, records));
        builder.storage = BuilderStorage::Held(storage);
        // The builder keeps working with the recovered storage.
        let root = builder.root.clone();
        builder
            .append(&root, HtmlNodeKind::Comment("c".into()))
            .unwrap();
    }

    #[test]
    fn unqualified_document_record_readback_is_not_a_node_kind_constant() {
        let arena = Arena {
            identity: Arc::new(Identity),
            records: vec![Record::new(
                HtmlNodeKind::Document(HtmlDocumentMode::NoQuirks),
                None,
                0,
            )],
            usage: HtmlConstructionUsage::default(),
            records_generation: 0,
            wrapper_generation: 0,
            temporary_generation: 0,
            serial: 0,
            attribute_revision: 0,
        };
        let node = HtmlNode {
            arena: &arena,
            index: 0,
        };
        assert_eq!(
            node.document_focused_area_designation(),
            Some(HtmlFocusedAreaDesignation::Unavailable)
        );
    }

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
        assert_eq!(builder.finish().unwrap().root().children().len(), 0);
        assert_eq!(recovered.data().code_units(), [97, 63]);
    }

    /// Native admission checks the usage the committed append stores.
    ///
    /// Expected values are counted by hand from the documented UTF-16 unit
    /// rule (names, URIs, prefixes, values, and textual slots). Admission is
    /// observed through its limit errors at the real append boundary: a limit
    /// equal to the committed value admits, and one below it reports that same
    /// committed value as the attempted usage.
    #[test]
    fn native_admission_checks_exactly_the_usage_the_append_commits() {
        let attribute = |namespace: &str, prefix: Option<&str>, local: &str, value: &str| {
            crate::internal::values::Attribute::new(
                QualifiedName::new(
                    Namespace::from_uri(namespace),
                    prefix.map(DocumentString::from),
                    local,
                ),
                value,
            )
        };
        let element = |attributes: Vec<crate::internal::values::Attribute>| {
            HtmlNodeKind::Element(HtmlElement::new(
                QualifiedName::new(Namespace::from_uri("urn:x"), Some("p".into()), "tag"),
                OrderedAttributes::new(attributes).unwrap(),
            ))
        };
        let cases: Vec<(&str, HtmlNodeKind, usize, usize)> = vec![
            (
                "text with an isolated surrogate pair and a null",
                HtmlNodeKind::Text(DocumentString::from_utf16(vec![0xD83D, 0xDE00, 0])),
                0,
                3,
            ),
            (
                "comment",
                HtmlNodeKind::Comment(DocumentString::from("xyz")),
                0,
                3,
            ),
            (
                "processing instruction",
                HtmlNodeKind::ProcessingInstruction(HtmlProcessingInstruction {
                    target: DocumentString::from("xy"),
                    data: DocumentString::from("abc"),
                }),
                0,
                5,
            ),
            (
                "doctype with both identifiers",
                HtmlNodeKind::Doctype(HtmlDoctype::new("h", Some("ab".into()), Some("xyz".into()))),
                0,
                6,
            ),
            (
                "doctype with only a present empty public identifier",
                HtmlNodeKind::Doctype(HtmlDoctype::new(
                    "html",
                    Some(DocumentString::default()),
                    None,
                )),
                0,
                4,
            ),
            // URI 5 + prefix 1 + local 3.
            ("element without attributes", element(Vec::new()), 0, 9),
            (
                "element with qualified and unqualified attributes",
                element(vec![
                    // Attribute: URI 5 + prefix 1 + local 1 + value 3.
                    attribute("urn:a", Some("q"), "k", "val"),
                    // Attribute: local 2 + empty value.
                    attribute("", None, "k2", ""),
                ]),
                2,
                9 + 10 + 2,
            ),
        ];
        for (label, kind, attributes, units) in cases {
            let append = |limits: HtmlConstructionLimits| {
                let mut builder = Builder::new(limits, NativeRoot::Fragment).unwrap();
                let root = builder.root.clone();
                let result = builder.append(&root, kind.clone());
                (result.map(|_| ()), builder.usage())
            };
            let roomy = HtmlConstructionLimits::new(8, 8, 8, 1_000, 8);
            let (result, committed) = append(roomy);
            result.unwrap();
            assert_eq!(
                (committed.attributes(), committed.utf16_units()),
                (attributes, units),
                "{label}: committed usage"
            );
            let exact = HtmlConstructionLimits::new(8, 8, attributes, units, 8);
            let (result, admitted) = append(exact);
            assert_eq!(result, Ok(()), "{label}: exact limits admit");
            assert_eq!(admitted, committed, "{label}: exact limits commit");
            let tight_units = HtmlConstructionLimits::new(8, 8, 8, units - 1, 8);
            let (result, unchanged) = append(tight_units);
            assert_eq!(
                result,
                Err(HtmlConstructionErrorKind::LimitExceeded {
                    resource: HtmlConstructionResource::Utf16Units,
                    limit: units - 1,
                    attempted: units,
                }
                .error()),
                "{label}: units checked"
            );
            assert_eq!(
                unchanged.utf16_units(),
                0,
                "{label}: rejection commits nothing"
            );
            if attributes > 0 {
                let tight = HtmlConstructionLimits::new(8, 8, attributes - 1, 1_000, 8);
                let (result, unchanged) = append(tight);
                assert_eq!(
                    result,
                    Err(HtmlConstructionErrorKind::LimitExceeded {
                        resource: HtmlConstructionResource::Attributes,
                        limit: attributes - 1,
                        attempted: attributes,
                    }
                    .error()),
                    "{label}: attributes checked"
                );
                assert_eq!(
                    unchanged.attributes(),
                    0,
                    "{label}: rejection commits nothing"
                );
            }
        }
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

#[cfg(test)]
mod accounting_tests {
    use super::*;

    fn accounting(limits: HtmlConstructionLimits) -> Accounting {
        Accounting::new(limits).unwrap()
    }

    fn refused(error: HtmlConstructionError) -> (HtmlConstructionResource, usize, usize) {
        match error.kind {
            HtmlConstructionErrorKind::LimitExceeded {
                resource,
                limit,
                attempted,
            } => (resource, limit, attempted),
            other => panic!("expected a limit refusal, got {other:?}"),
        }
    }

    /// Each limit is a cap of the construction scope. A charge that would pass a cap is refused with
    /// that resource, the cap and the total the charge would have reached, and the refusal changes
    /// nothing.
    #[test]
    fn each_limit_refuses_with_its_resource_cap_and_attempted_total() {
        use HtmlConstructionResource as R;
        let limits = HtmlConstructionLimits::new(3, 4, 5, 6, 7);
        for (amounts, expected) in [
            ((3, 0, 0, 0, 0), (R::Nodes, 3, 4)),
            ((0, 4, 0, 0, 0), (R::Edges, 4, 5)),
            ((0, 0, 5, 0, 0), (R::Attributes, 5, 6)),
            ((0, 0, 0, 6, 0), (R::Utf16Units, 6, 7)),
            ((0, 0, 0, 0, 7), (R::Depth, 7, 8)),
        ] {
            let mut accounting = accounting(limits);
            // One of each resource is in place, so the exact cap is one more than fits.
            accounting.admit(1, 1, 1, 1, 1).unwrap();
            let before = accounting.usage;
            let (nodes, edges, attributes, units, depth) = amounts;
            let (resource, limit, attempted) = refused(
                accounting
                    .admit(
                        nodes,
                        edges,
                        attributes,
                        units,
                        if depth == 0 { 1 } else { depth + 1 },
                    )
                    .unwrap_err(),
            );
            assert_eq!((resource, limit, attempted), expected, "{amounts:?}");
            assert_eq!(accounting.usage, before, "a refusal changes nothing");
        }
    }

    /// Exact limits admit, and the first limit in the order nodes, edges, attributes, UTF-16 units
    /// and depth is the one reported when several would fail.
    #[test]
    fn exact_limits_admit_and_the_first_limit_in_order_is_reported() {
        let mut accounting = accounting(HtmlConstructionLimits::new(2, 2, 2, 2, 2));
        accounting.admit(2, 2, 2, 2, 2).unwrap();
        assert_eq!(accounting.usage.nodes, 2);
        assert_eq!(accounting.usage.depth, 2);
        let mut accounting = self::accounting(HtmlConstructionLimits::new(1, 1, 1, 1, 1));
        let (resource, ..) = refused(accounting.admit(2, 2, 2, 2, 2).unwrap_err());
        assert_eq!(resource, HtmlConstructionResource::Nodes);
    }

    /// A release that matches nothing held is a release underflow of the ledger, and it changes
    /// nothing.
    #[test]
    fn a_release_beyond_the_admitted_amounts_is_a_ledger_release_underflow() {
        let mut accounting = accounting(HtmlConstructionLimits::new(9, 9, 9, 9, 9));
        accounting.admit(1, 0, 0, 0, 1).unwrap();
        let before = accounting.usage;
        let error = accounting
            .release(Charged {
                nodes: 2,
                edges: 0,
                attributes: 0,
                units: 0,
                depth: None,
            })
            .unwrap_err();
        assert_eq!(
            error.accounting_fault(),
            Some(HtmlAccountingFault::ReleaseUnderflow)
        );
        assert!(!error.is_release_underflow());
        assert_eq!(accounting.usage, before);
    }

    /// A ledger that cannot allocate, a foreign scope and a spent scope bound each reach the caller
    /// as the one accounting fault, not as the builder's own allocation failure.
    #[test]
    fn a_ledger_failure_that_is_not_a_refused_limit_is_an_accounting_fault() {
        for (error, fault) in [
            (LedgerError::Allocation, HtmlAccountingFault::Allocation),
            (LedgerError::ForeignScope, HtmlAccountingFault::ForeignScope),
            (
                LedgerError::ScopeLimit { limit: 2 },
                HtmlAccountingFault::ScopeLimit { limit: 2 },
            ),
        ] {
            let error = ledger_error(error);
            assert_eq!(error.accounting_fault(), Some(fault));
            assert!(!error.is_allocation_failed());
            assert!(!error.is_representation_exhausted());
        }
    }

    /// A refusal that names a dimension the construction scope does not report is a change the
    /// builder does not charge.
    #[test]
    fn a_cap_refusal_of_another_dimension_is_an_unexpected_change() {
        let limits = surgeist_html_limits_for_unrelated_dimension();
        let mut operation = Operation::new("scope", limits, 1).unwrap();
        let root = operation.root();
        let error = operation
            .apply(root, Change::Acquire(HtmlResource::FactRecords, 2))
            .unwrap_err();
        assert_eq!(
            ledger_error(error).accounting_fault(),
            Some(HtmlAccountingFault::UnexpectedChange)
        );
    }

    fn surgeist_html_limits_for_unrelated_dimension() -> crate::accounting::HtmlLimits {
        crate::accounting::HtmlLimits::from_fn(
            |_| u64::MAX,
            |resource| match resource {
                HtmlResource::FactRecords => 1,
                _ => u64::MAX,
            },
        )
    }

    /// A rejected append gives back its amounts and the depth it raised.
    #[test]
    fn releasing_a_charge_restores_the_amounts_and_the_depth() {
        let mut accounting = accounting(HtmlConstructionLimits::new(9, 9, 9, 9, 9));
        accounting.admit(1, 1, 0, 0, 2).unwrap();
        let before = accounting.usage;
        let charged = accounting.admit(2, 1, 3, 4, 5).unwrap();
        assert_eq!(accounting.usage.depth, 5);
        accounting.release(charged).unwrap();
        assert_eq!(accounting.usage, before);
        // A depth no deeper than the deepest admitted node raises nothing and restores nothing.
        let charged = accounting.admit(0, 0, 0, 0, 1).unwrap();
        accounting.release(charged).unwrap();
        assert_eq!(accounting.usage, before);
    }
}

#[cfg(test)]
mod cause_tests {
    use super::*;

    /// A depth is not a total: its limit refuses a depth, not a count.
    #[test]
    fn a_depth_limit_cannot_admit_a_depth() {
        let error = HtmlConstructionErrorKind::LimitExceeded {
            resource: HtmlConstructionResource::Depth,
            limit: 3,
            attempted: 4,
        }
        .error();
        assert_eq!(
            error.to_string(),
            "depth limit of 3 cannot admit a depth of 4"
        );
    }

    /// Every cause states its documented meaning, and exactly one of the
    /// cause predicates, its own, recognizes it.
    #[test]
    fn every_cause_has_its_message_and_exactly_one_matching_predicate() {
        type Predicate = fn(&HtmlConstructionError) -> bool;
        let cases: Vec<(HtmlConstructionError, &str, Predicate)> = vec![
            (
                HtmlConstructionErrorKind::ForeignBuilder.error(),
                "node handle belongs to a different builder",
                HtmlConstructionError::is_foreign_builder,
            ),
            (
                HtmlConstructionErrorKind::ParentCannotHaveChildren.error(),
                "text, comment, processing-instruction and doctype nodes cannot have children",
                HtmlConstructionError::is_parent_cannot_have_children,
            ),
            (
                HtmlConstructionErrorKind::TemplateContentsRequired.error(),
                "template children must be appended to its separate contents",
                HtmlConstructionError::is_template_contents_required,
            ),
            (
                HtmlConstructionErrorKind::ShadowHostMustBeElement.error(),
                "a shadow root requires an element host",
                HtmlConstructionError::is_shadow_host_must_be_element,
            ),
            (
                HtmlConstructionErrorKind::ShadowAlreadyAttached.error(),
                "element already has a shadow root",
                HtmlConstructionError::is_shadow_already_attached,
            ),
            (
                HtmlConstructionErrorKind::UnknownNode.error(),
                "node does not exist in this builder",
                HtmlConstructionError::is_unknown_node,
            ),
            (
                HtmlConstructionErrorKind::WrongNodeKind.error(),
                "node kind cannot take part in this command",
                HtmlConstructionError::is_wrong_node_kind,
            ),
            (
                HtmlConstructionErrorKind::RootMove.error(),
                "documents and fragments cannot be moved or detached",
                HtmlConstructionError::is_root_move,
            ),
            (
                HtmlConstructionErrorKind::AssociationMove.error(),
                "template contents and shadow fragments cannot be moved as ordinary children",
                HtmlConstructionError::is_association_move,
            ),
            (
                HtmlConstructionErrorKind::WrongSibling.error(),
                "reference node is not a child of the target parent",
                HtmlConstructionError::is_wrong_sibling,
            ),
            (
                HtmlConstructionErrorKind::Cycle.error(),
                "move would make a node its own ancestor",
                HtmlConstructionError::is_cycle,
            ),
            (
                HtmlConstructionErrorKind::AssociationAlreadyOwned.error(),
                "fragment is already associated with another owner",
                HtmlConstructionError::is_association_already_owned,
            ),
            (
                HtmlConstructionErrorKind::InvalidTemplateAssociation.error(),
                "template contents association does not satisfy the template rules",
                HtmlConstructionError::is_invalid_template_association,
            ),
            (
                HtmlConstructionErrorKind::AttributeNameMismatch.error(),
                "attribute candidate name does not match the planned name",
                HtmlConstructionError::is_attribute_name_mismatch,
            ),
            (
                HtmlConstructionErrorKind::AttributeOrderMismatch.error(),
                "attribute candidates were not presented in order",
                HtmlConstructionError::is_attribute_order_mismatch,
            ),
            (
                HtmlConstructionErrorKind::InvalidShadowAssociation.error(),
                "shadow association does not satisfy the shadow host rules",
                HtmlConstructionError::is_invalid_shadow_association,
            ),
            (
                HtmlConstructionErrorKind::InvalidContentsReference.error(),
                "replacement template contents are the existing contents",
                HtmlConstructionError::is_invalid_contents_reference,
            ),
            (
                HtmlConstructionErrorKind::ParentChildConflict.error(),
                "parent and child records disagree about their relationship",
                HtmlConstructionError::is_parent_child_conflict,
            ),
            (
                HtmlConstructionErrorKind::InvalidOutputRoot.error(),
                "published output root is not a detached document or fragment",
                HtmlConstructionError::is_invalid_output_root,
            ),
            (
                HtmlConstructionErrorKind::OperationNotReady.error(),
                "construction operation still has pending actions",
                HtmlConstructionError::is_operation_not_ready,
            ),
            (
                HtmlConstructionErrorKind::ForeignAllocation.error(),
                "retired storage does not belong to this result",
                HtmlConstructionError::is_foreign_allocation,
            ),
            (
                HtmlConstructionErrorKind::CapacityMismatch.error(),
                "observed capacity does not match the pending reservation",
                HtmlConstructionError::is_capacity_mismatch,
            ),
            (
                HtmlConstructionErrorKind::IndexUnavailable.error(),
                "the attribute name index lent to the edit could not serve a step",
                HtmlConstructionError::is_index_unavailable,
            ),
            (
                HtmlConstructionErrorKind::SharedAllocationAliased.error(),
                "result storage is shared and cannot be recovered",
                HtmlConstructionError::is_shared_allocation_aliased,
            ),
            (
                HtmlConstructionErrorKind::ReleaseUnderflow.error(),
                "released usage exceeds the admitted usage",
                HtmlConstructionError::is_release_underflow,
            ),
            (
                HtmlConstructionErrorKind::RepresentationExhausted.error(),
                "storage count cannot be represented on this platform",
                HtmlConstructionError::is_representation_exhausted,
            ),
            (
                HtmlConstructionErrorKind::AllocationFailed.error(),
                "reserving arena or edge storage failed",
                HtmlConstructionError::is_allocation_failed,
            ),
            (
                HtmlConstructionErrorKind::DuplicateAttribute {
                    first: 0,
                    duplicate: 1,
                }
                .error(),
                "attribute at index 1 duplicates the namespace and local name of the attribute at index 0",
                HtmlConstructionError::is_duplicate_attribute,
            ),
            (
                HtmlConstructionErrorKind::LimitExceeded {
                    resource: HtmlConstructionResource::Nodes,
                    limit: 1,
                    attempted: 2,
                }
                .error(),
                "node limit of 1 cannot admit a total of 2",
                HtmlConstructionError::is_limit_exceeded,
            ),
        ];
        for (error, message, predicate) in &cases {
            assert_eq!(error.to_string(), *message);
            assert!(predicate(error), "{message}");
            let matching = cases.iter().filter(|(_, _, other)| other(error)).count();
            assert_eq!(matching, 1, "{message}");
        }
    }
}
