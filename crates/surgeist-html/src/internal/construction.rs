//! Bounded native authoring and immutable, result-local construction output.

mod attribute_table;
pub(crate) mod bounded;
pub(crate) mod names;
mod text;
mod views;

pub use views::{
    BuildAttribute, BuildAttributeIter, BuildAttributes, BuildStr, BuildText, ConstructionDoctype,
    ConstructionElementView, ConstructionNodeKind, ConstructionProcessingInstruction,
    HtmlAttribute, HtmlAttributes, HtmlElementView, HtmlNodeKind,
};

use crate::internal::accounting::{
    HtmlAccountingFault, HtmlCapScope, HtmlChargedError, HtmlHostScope, HtmlLedgerOutcome,
    HtmlResource, HtmlWork, aggregate_tariffs, classify, wide,
};
#[cfg(test)]
use crate::internal::values::Namespace;
use crate::{
    internal::values::DocumentString, internal::values::OrderedAttributes,
    internal::values::QualifiedName, internal::values::TextBlock,
};
use attribute_table::{AttributeTable, RowSlice, RunIx};
use names::{NameEntry, NameTable};
use std::{collections::VecDeque, error::Error, fmt, num::NonZeroU32, sync::Arc};
use surgeist_ledger::{
    Change, Charge, Error as LedgerError, Operation, Scope, SharedOperation, Usage,
};
use text::{TextArena, TextRun};

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
/// [`Self::cause`] names what the step violated. The cause enum is exhaustive, so a caller that
/// maps every cause fails to compile when a new one is added. The payload accessor
/// [`Self::accounting_fault`] and every message state the cause in prose.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlConstructionError {
    cause: HtmlConstructionCause,
}

/// What a rejected construction step violated.
///
/// Every cause is named, and the enum is exhaustive: a consumer that classifies the causes
/// matches every variant, so a cause added later is a compile error at that consumer.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlConstructionCause {
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
    /// The node does not exist in this builder.
    UnknownNode,
    /// The node kind cannot take part in this command.
    WrongNodeKind,
    /// Documents and fragments cannot be moved or detached.
    RootMove,
    /// Template contents and shadow fragments cannot be moved as ordinary children.
    AssociationMove,
    /// The reference node is not a child of the target parent.
    WrongSibling,
    /// The move would make a node its own ancestor.
    Cycle,
    /// The fragment is already associated with another owner.
    AssociationAlreadyOwned,
    /// The template contents association does not satisfy the template rules.
    InvalidTemplateAssociation,
    /// Two attributes share a namespace and local name.
    DuplicateAttribute {
        /// The index of the first of the two attributes.
        first: usize,
        /// The index of the second of the two attributes.
        duplicate: usize,
    },
    /// The shadow association does not satisfy the shadow host rules.
    InvalidShadowAssociation,
    /// The replacement template contents are the existing contents.
    InvalidContentsReference,
    /// Parent and child records disagree about their relationship.
    ParentChildConflict,
    /// The published output root is not a detached document or fragment.
    InvalidOutputRoot,
    /// An operand the operation requires is absent: the attribute or name it was to take, or an
    /// attribute index past the list.
    OperandMissing,
    /// The capacity of a bank is below what the operation stores: the consumer chose less than
    /// the minimum, or the bank has no room where the operation writes.
    CapacityMismatch,
    /// A name index lent to an attribute edit could not serve a step.
    IndexUnavailable,
    /// A string of the text block that publication takes back is held by another owner.
    SharedAllocationAliased,
    /// The builder's own arithmetic released more than it admitted. A release that the ledger
    /// refuses is [`Self::Accounting`].
    ReleaseUnderflow,
    /// A text or comment record names a range that its storage's text arena does not hold.
    InvalidTextReference,
    /// An element or an attribute row names a run, a row or a value that its storage's attribute
    /// table or text arena does not hold, or a published element names rows its result does not
    /// hold.
    InvalidAttributeReference,
    /// A record was read through a view of a storage that has not published it, where the
    /// operation needs the published result.
    NotPublished,
    /// A finite resource limit would be exceeded.
    LimitExceeded {
        /// The resource that would exceed its limit.
        resource: HtmlConstructionResource,
        /// The configured limit.
        limit: usize,
        /// The total the step would have reached.
        attempted: usize,
    },
    /// A storage count, layout size, record index or storage serial has no representable value.
    RepresentationExhausted,
    /// Reserving arena or edge storage failed. A ledger that cannot allocate is
    /// [`Self::Accounting`].
    AllocationFailed,
    /// The ledger scope that enforces the limits failed in a way that is not a refused limit: a
    /// release it refused, an allocation it could not make or a state its own sequencing rules
    /// out.
    Accounting(HtmlAccountingFault),
}
impl HtmlConstructionCause {
    /// Wraps this cause as the public error where inference cannot convert it.
    pub(crate) const fn error(self) -> HtmlConstructionError {
        HtmlConstructionError { cause: self }
    }
}

impl From<HtmlConstructionCause> for HtmlConstructionError {
    fn from(cause: HtmlConstructionCause) -> Self {
        Self { cause }
    }
}

impl HtmlConstructionError {
    /// What the step violated, with the payload of a cause that has one.
    pub fn cause(&self) -> HtmlConstructionCause {
        self.cause
    }
    /// The accounting fault, when the cause is [`HtmlConstructionCause::Accounting`]: a failure of
    /// the ledger scope that enforces the limits that is not a refused limit. It equals that
    /// cause's payload and is the method every HTML error that can carry an accounting fault
    /// exposes.
    pub fn accounting_fault(&self) -> Option<HtmlAccountingFault> {
        match self.cause {
            HtmlConstructionCause::Accounting(fault) => Some(fault),
            _ => None,
        }
    }
}

impl fmt::Display for HtmlConstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use HtmlConstructionCause as Cause;
        match &self.cause {
            Cause::ForeignBuilder => f.write_str("node handle belongs to a different builder"),
            Cause::ParentCannotHaveChildren => f.write_str(
                "text, comment, processing-instruction and doctype nodes cannot have children",
            ),
            Cause::TemplateContentsRequired => {
                f.write_str("template children must be appended to its separate contents")
            }
            Cause::ShadowHostMustBeElement => f.write_str("a shadow root requires an element host"),
            Cause::ShadowAlreadyAttached => f.write_str("element already has a shadow root"),
            Cause::UnknownNode => f.write_str("node does not exist in this builder"),
            Cause::WrongNodeKind => f.write_str("node kind cannot take part in this command"),
            Cause::RootMove => f.write_str("documents and fragments cannot be moved or detached"),
            Cause::AssociationMove => f.write_str(
                "template contents and shadow fragments cannot be moved as ordinary children",
            ),
            Cause::WrongSibling => {
                f.write_str("reference node is not a child of the target parent")
            }
            Cause::Cycle => f.write_str("move would make a node its own ancestor"),
            Cause::AssociationAlreadyOwned => {
                f.write_str("fragment is already associated with another owner")
            }
            Cause::InvalidTemplateAssociation => {
                f.write_str("template contents association does not satisfy the template rules")
            }
            Cause::InvalidShadowAssociation => {
                f.write_str("shadow association does not satisfy the shadow host rules")
            }
            Cause::InvalidContentsReference => {
                f.write_str("replacement template contents are the existing contents")
            }
            Cause::ParentChildConflict => {
                f.write_str("parent and child records disagree about their relationship")
            }
            Cause::InvalidOutputRoot => {
                f.write_str("published output root is not a detached document or fragment")
            }
            Cause::Accounting(fault) => write!(
                f,
                "the ledger scope that enforces the limits failed: {fault}"
            ),
            Cause::OperandMissing => f.write_str("an operand the operation requires is absent"),
            Cause::CapacityMismatch => {
                f.write_str("bank capacity is below what the operation stores")
            }
            Cause::IndexUnavailable => {
                f.write_str("the attribute name index lent to the edit could not serve a step")
            }
            Cause::SharedAllocationAliased => {
                f.write_str("a string of the text block taken back is held by another owner")
            }
            Cause::ReleaseUnderflow => f.write_str("released usage exceeds the admitted usage"),
            Cause::RepresentationExhausted => {
                f.write_str("a storage count, layout size, record index or storage serial has no representable value")
            }
            Cause::AllocationFailed => f.write_str("reserving arena or edge storage failed"),
            Cause::InvalidTextReference => {
                f.write_str("a text record names a range its text arena does not hold")
            }
            Cause::InvalidAttributeReference => f.write_str(
                "an element or attribute row names a run, row or value its storage does not hold",
            ),
            Cause::NotPublished => {
                f.write_str("the record is read through a storage that has not published it")
            }
            Cause::DuplicateAttribute { first, duplicate } => write!(
                f,
                "attribute at index {duplicate} duplicates the namespace and local name of the attribute at index {first}"
            ),
            Cause::LimitExceeded {
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

/// A record position within one construction storage, at most `u32::MAX - 1`.
///
/// Only [`NodeIx::new`] makes one, so a position that exists fits the 32-bit index of a node id.
/// The stored value is the position plus one, which gives `Option<NodeIx>` the size of a `NodeIx`.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct NodeIx(NonZeroU32);
impl NodeIx {
    /// The first record position.
    pub(crate) const FIRST: Self = Self(NonZeroU32::MIN);
    /// The position as an index, or `None` when it does not fit the 32-bit index of a node id.
    pub(crate) fn new(position: usize) -> Option<Self> {
        let stored = u32::try_from(position).ok()?.checked_add(1)?;
        NonZeroU32::new(stored).map(Self)
    }
    /// The position as an index into the storage's records.
    pub(crate) fn position(self) -> usize {
        // A `u32` fits `usize` on every supported target; the stored value is never zero.
        self.0.get().saturating_sub(1) as usize
    }
    /// The index of `position`, which a test names within a few records.
    #[cfg(test)]
    pub(crate) fn at(position: usize) -> Self {
        Self::new(position).expect("a test position fits a node index")
    }
    /// The position `count` records after this one, or `None` when it does not fit.
    pub(crate) fn offset(self, count: usize) -> Option<Self> {
        Self::new(self.position().checked_add(count)?)
    }
}

impl Default for NodeIx {
    fn default() -> Self {
        Self::FIRST
    }
}

/// An opaque node identity qualified by its immutable construction result.
///
/// Cloning a document preserves identity. A separate build always has different
/// identities, even when its text and local positions are identical. An id
/// holds the serial of the storage that issued it and a record index, and has
/// no order.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct HtmlNodeId {
    tree: u64,
    index: NodeIx,
}
impl fmt::Debug for HtmlNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HtmlNodeId")
            .field("tree", &self.tree)
            .field("index", &self.index.position())
            .finish()
    }
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<HtmlNodeId>() == 16);

/// A checked local handle usable only by the builder that issued it.
#[derive(Clone, Copy, Eq, Hash, PartialEq)]
pub struct HtmlBuilderNode {
    tree: u64,
    index: NodeIx,
}
impl fmt::Debug for HtmlBuilderNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HtmlBuilderNode")
            .field("tree", &self.tree)
            .field("index", &self.index.position())
            .finish()
    }
}
#[cfg(target_pointer_width = "64")]
const _: () = assert!(size_of::<HtmlBuilderNode>() == 16);

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

/// How a record holds what it keeps in the arenas of its storage: `Self` is the cell that holds
/// the units of a text or comment, and [`Self::Attrs`] the cell that holds the attributes of an
/// element.
///
/// A [`TextRun`] and an optional run of the attribute table while the storage builds; a
/// [`DocumentString`] of the published block and an optional slice of the published table once
/// it is published. A record of one form cannot hold the other.
trait Cell {
    /// The attributes of an element in this form.
    type Attrs: Clone + fmt::Debug + Eq;
}
impl Cell for TextRun {
    type Attrs = Option<RunIx>;
}
impl Cell for DocumentString {
    type Attrs = Option<RowSlice>;
}

/// The stored name, attributes and link history of an element record.
#[derive(Clone, Debug, Eq, PartialEq)]
struct ElementContent<T: Cell> {
    link_flags: Option<crate::internal::elements::rel::HtmlLinkFlags>,
    name: Arc<NameEntry>,
    attrs: T::Attrs,
}
impl<T: Cell> ElementContent<T> {
    fn name(&self) -> &QualifiedName {
        self.name.name()
    }
    fn key(&self) -> crate::internal::values::element_key::ElementKey {
        self.name.key()
    }
}

/// The stored content of a node record. Readers see it only through [`HtmlNodeKind`] and
/// [`ConstructionNodeKind`].
///
/// `T` is how the content holds the units of a text or comment: a [`TextRun`] of the storage's
/// text arena while the storage builds, and a [`DocumentString`] of the published block once it
/// is published. A record of one form cannot hold the other.
#[derive(Clone, Debug, Eq, PartialEq)]
enum NodeContent<T: Cell> {
    /// The Document node, carrying the compatibility mode selected when it was built.
    Document(HtmlDocumentMode),
    /// A document-fragment root.
    Fragment,
    /// An element with its exact qualified name and ordered raw attributes.
    Element(ElementContent<T>),
    /// A text node; the payload preserves the exact UTF-16 code units.
    Text(T),
    /// A comment node; the payload preserves the exact UTF-16 code units.
    Comment(T),
    /// A processing instruction recorded with its exact target and data.
    ProcessingInstruction(HtmlProcessingInstruction),
    /// A document type declaration.
    Doctype(HtmlDoctype),
}

/// The content of a record while its storage builds.
type BuildContent = NodeContent<TextRun>;
/// The content of a record of a published result.
type PublishedContent = NodeContent<DocumentString>;

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
    host: HtmlNodeId,
    contents: HtmlNodeId,
    properties: &'a HtmlShadowRootProperties,
}
impl<'a> HtmlDeclarativeShadowRoot<'a> {
    /// Host element identity.
    pub fn host(&self) -> HtmlNodeId {
        self.host
    }
    /// Associated fragment root identity.
    pub fn contents(&self) -> HtmlNodeId {
        self.contents
    }
    /// Actual properties retained once with the shadow contents.
    pub fn properties(&self) -> HtmlShadowRootProperties {
        *self.properties
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ShadowAssociation {
    host: HtmlNodeId,
    contents: HtmlNodeId,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TemplateContents {
    Owned(NodeIx),
    ShadowReference(NodeIx),
}
impl TemplateContents {
    fn position(self) -> NodeIx {
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
struct Record<T: Cell> {
    kind: NodeContent<T>,
    document_focused_area_designation: Option<HtmlFocusedAreaDesignation>,
    parent: Option<NodeIx>,
    /// The ordinary children in order. A deque, so that taking or placing an entry near either
    /// end shifts no more entries than lie between it and the nearer end.
    children: VecDeque<NodeIx>,
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
impl Record<TextRun> {
    /// This record as a record of a published result, with `kind` for its content, which was made
    /// from this record's content. Every other field moves unchanged.
    fn published(self, kind: PublishedContent) -> Record<DocumentString> {
        let Self {
            kind: _,
            document_focused_area_designation,
            parent,
            children,
            depth,
            population,
            walk_cursor,
            level_count,
            template_contents,
            shadow,
            shadow_properties,
            owner,
            publish_found,
        } = self;
        Record {
            kind,
            document_focused_area_designation,
            parent,
            children,
            depth,
            population,
            walk_cursor,
            level_count,
            template_contents,
            shadow,
            shadow_properties,
            owner,
            publish_found,
        }
    }
}
impl<T: Cell> Record<T> {
    fn new(kind: NodeContent<T>, parent: Option<NodeIx>, depth: usize) -> Self {
        let document_focused_area_designation = matches!(kind, NodeContent::Document(_))
            .then_some(HtmlFocusedAreaDesignation::Unavailable);
        Self {
            kind,
            document_focused_area_designation,
            parent,
            children: VecDeque::new(),
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
struct Arena<T: Cell> {
    records: Vec<Record<T>>,
    /// The names of the elements and attributes of the records.
    names: NameTable,
    usage: HtmlConstructionUsage,
    // The tree serial of this storage's node ids and builder handles. It names this storage among
    // all storages, so a lookup tells an id or handle of another storage from its own, and so that a
    // table of attribute names can tell the attributes it describes from those of another storage.
    serial: u64,
    // Advances with every committed change to the attributes of any element of this storage,
    // whichever edit made it, so that a table of attribute names can tell the attributes it
    // describes from those that have changed since.
    attribute_revision: u64,
}
/// A publication replaces the vector of records of a storage with a vector of published records
/// and the consumer records the capacity of the one it replaces as the bank the other grows from,
/// which holds only while the two records have one size.
const _: () = assert!(size_of::<Record<TextRun>>() == size_of::<Record<DocumentString>>());
/// The contents vector of a publication holds one published content for each record, and the
/// build view of a record charges the size of its content to the inspections that read it, so the
/// content of a record is one size in both forms.
const _: () =
    assert!(size_of::<NodeContent<TextRun>>() == size_of::<NodeContent<DocumentString>>());

/// The arena of a storage that builds, whose text is in its storage's text arena.
type BuildArena = Arena<TextRun>;
/// The arena of a published result.
type PublishedArena = Arena<DocumentString>;

impl<T: Cell> Arena<T> {
    /// How many records have exactly this depth.
    fn population(&self, depth: usize) -> usize {
        self.records
            .get(depth)
            .map_or(0, |bucket| bucket.population)
    }
    fn id(&self, index: NodeIx) -> HtmlNodeId {
        HtmlNodeId {
            tree: self.serial,
            index,
        }
    }
    /// The record at `position`, or `None` past the last record.
    fn has_record(&self, position: usize) -> Option<NodeIx> {
        NodeIx::new(position).filter(|index| index.position() < self.records.len())
    }
    /// The record index a builder handle names: `ForeignBuilder` for another storage's handle and
    /// `UnknownNode` for an index past the records.
    fn index(&self, handle: &HtmlBuilderNode) -> Result<NodeIx, HtmlConstructionError> {
        if handle.tree != self.serial {
            return Err(HtmlConstructionCause::ForeignBuilder.into());
        }
        if handle.index.position() >= self.records.len() {
            return Err(HtmlConstructionCause::UnknownNode.into());
        }
        Ok(handle.index)
    }
    /// The declarative shadow root of the host element at `index`: the association whose contents
    /// carry shadow properties.
    fn declarative_shadow_root(&self, index: NodeIx) -> Option<HtmlDeclarativeShadowRoot<'_>> {
        let relation = self.records[index.position()].shadow.as_ref()?;
        // A declarative shadow root is an association whose contents carry shadow properties.
        let properties = self
            .records
            .get(relation.contents.index.position())?
            .shadow_properties
            .as_ref()?;
        Some(HtmlDeclarativeShadowRoot {
            host: relation.host,
            contents: relation.contents,
            properties,
        })
    }
    /// The record index a result id names, or `ForeignHtmlResult` for the id of another storage
    /// or one past the records.
    fn lookup_index(&self, id: &HtmlNodeId) -> Result<NodeIx, ForeignHtmlResult> {
        if self.serial != id.tree || id.index.position() >= self.records.len() {
            return Err(ForeignHtmlResult);
        }
        Ok(id.index)
    }
}
impl PublishedArena {
    fn lookup(&self, id: &HtmlNodeId) -> Result<HtmlNode<'_>, ForeignHtmlResult> {
        Ok(HtmlNode {
            arena: self,
            index: self.lookup_index(id)?,
        })
    }
}

/// A borrowed immutable node view, with ordinary and association edges distinct.
#[derive(Clone, Copy, Debug)]
pub struct HtmlNode<'a> {
    arena: &'a PublishedArena,
    index: NodeIx,
}
impl<'a> HtmlNode<'a> {
    /// Result-qualified node identity.
    pub fn id(self) -> HtmlNodeId {
        self.arena.id(self.index)
    }
    /// The node's content, by value.
    pub fn kind(self) -> HtmlNodeKind<'a> {
        self.content().published()
    }
    fn content(self) -> &'a PublishedContent {
        &self.arena.records[self.index.position()].kind
    }
    /// Original design-mode state of this owned Document, or `None` for a
    /// non-Document. Controlled construction initializes Documents with design
    /// mode disabled and admits no operation that changes it. Captured external
    /// Documents are not owned construction nodes and receive no such grant.
    pub fn document_design_mode_enabled(self) -> Option<bool> {
        matches!(self.content(), NodeContent::Document(_)).then_some(false)
    }
    /// Stored original designation of this owned Document, or `None` for a
    /// non-Document. Only the controlled CreateDocument transition issues known
    /// null. Its closed operation domain preserves that state through recovery
    /// and publication; captured external Documents receive no such grant.
    pub fn document_focused_area_designation(self) -> Option<HtmlFocusedAreaDesignation> {
        self.arena.records[self.index.position()].document_focused_area_designation
    }
    /// Ordinary parent; association fragment roots have no ordinary parent.
    pub fn parent(self) -> Option<HtmlNodeId> {
        self.arena.records[self.index.position()]
            .parent
            .map(|index| self.arena.id(index))
    }
    /// Ordinary children in exact append order; associations are excluded.
    pub fn children(self) -> impl ExactSizeIterator<Item = HtmlNodeId> + DoubleEndedIterator + 'a {
        self.arena.records[self.index.position()]
            .children
            .iter()
            .map(|index| self.arena.id(*index))
    }
    /// Separate contents of an HTML template element.
    pub fn template_contents(self) -> Option<HtmlNodeId> {
        self.arena.records[self.index.position()]
            .template_contents
            .map(|contents| self.arena.id(contents.position()))
    }
    /// Native declarative shadow association, if supplied. A declarative shadow root is the
    /// association whose contents carry shadow properties, and publication refuses any `shadow`
    /// relation whose contents lack them (`validate_record`, `InvalidShadowAssociation`), so a
    /// published tree has no association that this answers with `None`.
    pub fn declarative_shadow_root(self) -> Option<HtmlDeclarativeShadowRoot<'a>> {
        self.arena.declarative_shadow_root(self.index)
    }
    /// Distinguishes an owning template association from a shadow reference.
    pub fn template_contents_kind(self) -> Option<HtmlTemplateContentsKind> {
        self.arena.records[self.index.position()]
            .template_contents
            .map(TemplateContents::kind)
    }
    /// Whether this fragment was explicitly constructed as shadow contents.
    pub fn is_shadow_contents(self) -> bool {
        self.shadow_properties().is_some()
    }
    /// Actual shadow properties, including when no local host exists.
    pub fn shadow_properties(self) -> Option<HtmlShadowRootProperties> {
        self.arena.records[self.index.position()].shadow_properties
    }
    /// Template or shadow association owning this fragment root.
    pub fn fragment_owner(self) -> Option<HtmlFragmentOwner> {
        self.arena.records[self.index.position()].owner
    }
}

/// A concrete immutable HTML document produced by native construction.
/// It makes no claim about parsing, source provenance, or conformance checking.
#[derive(Clone, Debug)]
pub struct HtmlDocument {
    arena: Arc<PublishedArena>,
    root: NodeIx,
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
    /// Begins bounded native document construction that charges below the scope `host` supplies,
    /// in the host's ledger operation.
    ///
    /// The builder opens `html-construction` below the scope, which carries the caps of `limits`
    /// and converts into the scope by [`aggregate_tariffs`]; a cap of the scope or of an ancestor
    /// refuses an append with the limit that scope carries. The host operation keeps the
    /// admitted amounts after the builder and its result drop, and keeps the scope.
    pub fn builder_with_accounting(
        limits: HtmlConstructionLimits,
        mode: HtmlDocumentMode,
        host: &HtmlHostScope,
    ) -> Result<HtmlDocumentBuilder, HtmlConstructionError> {
        Ok(HtmlDocumentBuilder {
            builder: Builder::charging(limits, NativeRoot::Document(mode), Some(host))?,
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
    arena: Arc<PublishedArena>,
    root: NodeIx,
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
    /// Begins source-free native fragment construction that charges below the scope `host`
    /// supplies; see [`HtmlDocument::builder_with_accounting`].
    pub fn builder_with_accounting(
        limits: HtmlConstructionLimits,
        host: &HtmlHostScope,
    ) -> Result<HtmlFragmentBuilder, HtmlConstructionError> {
        Ok(HtmlFragmentBuilder {
            builder: Builder::charging(limits, NativeRoot::Fragment, Some(host))?,
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
    arena: &Arc<PublishedArena>,
    id: &HtmlNodeId,
) -> Result<Option<HtmlFragment>, ForeignHtmlResult> {
    Ok(
        matches!(arena.lookup(id)?.content(), NodeContent::Fragment).then(|| HtmlFragment {
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

/// What a native append creates: an element or a leaf.
#[derive(Clone, Debug)]
enum Creation {
    Element {
        name: QualifiedName,
        attributes: OrderedAttributes,
        link_flags: Option<crate::internal::elements::rel::HtmlLinkFlags>,
    },
    Text(DocumentString),
    Comment(DocumentString),
    ProcessingInstruction(HtmlProcessingInstruction),
    Doctype(HtmlDoctype),
}
impl Creation {
    /// Whether the created record is an HTML `template` element, which brings its contents.
    fn is_template(&self) -> bool {
        matches!(self, Self::Element { name, .. }
            if crate::vocabulary::KnownHtmlElement::from_name(name)
                == Some(crate::vocabulary::KnownHtmlElement::Template))
    }
    /// Attributes and UTF-16 units the creation commits, evaluated in one step so native
    /// admission can check the usage before committing it.
    fn usage(&self) -> Result<(usize, usize), HtmlConstructionError> {
        let sum = |left: usize, right: usize| {
            left.checked_add(right)
                .ok_or(HtmlConstructionCause::RepresentationExhausted.error())
        };
        match self {
            Self::Text(value) | Self::Comment(value) => Ok((0, value.len())),
            Self::ProcessingInstruction(value) => {
                Ok((0, sum(value.target().len(), value.data().len())?))
            }
            Self::Doctype(value) => Ok((
                0,
                sum(
                    sum(
                        value.name().len(),
                        value.public_id().map_or(0, DocumentString::len),
                    )?,
                    value.system_id().map_or(0, DocumentString::len),
                )?,
            )),
            Self::Element {
                name, attributes, ..
            } => {
                let mut units = bounded::records::name_units(name)?;
                for attribute in attributes.iter() {
                    units = sum(
                        units,
                        bounded::records::attribute_units(
                            attribute.name(),
                            attribute.value().len(),
                        )?,
                    )?;
                }
                Ok((attributes.len(), units))
            }
        }
    }
    /// Creates the record in `storage`.
    fn create(
        self,
        storage: &mut bounded::HtmlConstructionStorage,
        admission: &mut NativeAdmission,
    ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
        match self {
            Self::Element {
                name,
                attributes,
                link_flags,
            } => storage.create_element(&name, attributes, link_flags, admission),
            Self::Text(value) => storage.create_text(value.code_units(), admission),
            Self::Comment(value) => storage.create_comment(value.code_units(), admission),
            Self::ProcessingInstruction(value) => {
                storage.create_shared_processing_instruction(value, admission)
            }
            Self::Doctype(value) => storage.create_shared_doctype(value, admission),
        }
        .map_err(native)
    }
}

/// The admission of a native builder. The builder admits the limits of an append in full before it
/// calls the storage, so every operation of the storage is admitted here without a further check,
/// and the builder charges no work. A bank that is filled one entry at a time (the record bank and
/// a child order) grows to its amortized capacity, never above what the limits admit, when the
/// admission has spare room, as does the text arena that every text and comment is copied into;
/// every other bank, and every bank of an admission without it, grows to the least capacity that
/// suffices.
struct NativeAdmission {
    /// The most records, the most child entries and the most text units the limits admit, or
    /// `None` for exact growth.
    spare_room: Option<(usize, usize, usize)>,
}
impl NativeAdmission {
    fn exact() -> Self {
        Self { spare_room: None }
    }
    fn with_spare_room(limits: HtmlConstructionLimits) -> Self {
        Self {
            spare_room: Some((limits.nodes(), limits.edges(), limits.utf16_units())),
        }
    }
}
impl Charge<HtmlWork, HtmlResource> for NativeAdmission {
    type Stop = std::convert::Infallible;
    fn charge(&mut self, _batch: &[Change<HtmlWork, HtmlResource>]) -> Result<(), Self::Stop> {
        Ok(())
    }
    fn record_largest(&mut self, _resource: HtmlResource, _value: u64) -> Result<(), Self::Stop> {
        Ok(())
    }
}
impl bounded::records::HtmlRecordAdmission for NativeAdmission {
    fn admit_change(
        &mut self,
        _storage: &bounded::HtmlConstructionStorage,
        _change: bounded::records::HtmlConstructionChange,
    ) -> Result<(), Self::Stop> {
        Ok(())
    }
    /// The capacity a bank takes when it may grow with spare room: the amortized capacity of the
    /// bank, never above the most entries the limits admit, and never below what the operation
    /// needs. The native builder admits nothing beyond its node and edge limits and has no
    /// capacity-bytes limit, so a capacity above the entries those limits admit is spare room that
    /// no append can use; capping it there is the admission of the allocation.
    fn admit_growth(
        &mut self,
        growth: bounded::records::HtmlConstructionBankGrowth,
    ) -> Result<usize, Self::Stop> {
        let ceiling = match (self.spare_room, growth.bank()) {
            (Some((nodes, ..)), bounded::records::HtmlConstructionBank::Records) => Some(nodes),
            (Some((_, edges, _)), bounded::records::HtmlConstructionBank::Children { .. }) => {
                Some(edges)
            }
            (Some((.., units)), bounded::records::HtmlConstructionBank::TextArena) => Some(units),
            _ => None,
        };
        let minimum = growth.minimum_capacity();
        Ok(match ceiling {
            Some(ceiling) => crate::internal::amortized::amortized_capacity(
                growth.before().initialized_len(),
                growth.before().capacity(),
            )
            .min(ceiling)
            .max(minimum),
            None => minimum,
        })
    }
    fn record_growth(
        &mut self,
        _record: bounded::records::HtmlConstructionGrowthRecord,
    ) -> Result<(), Self::Stop> {
        Ok(())
    }
    fn record_change(
        &mut self,
        _storage: &bounded::HtmlConstructionStorage,
        _change: bounded::records::HtmlConstructionChange,
    ) -> Result<(), Self::Stop> {
        Ok(())
    }
}
/// The construction error of a refusal of the storage under a [`NativeAdmission`], which never
/// stops.
fn native(
    error: bounded::records::HtmlConstructionRecordError<std::convert::Infallible>,
) -> HtmlConstructionError {
    match error {
        HtmlChargedError::Stopped(never) => match never {},
        HtmlChargedError::Faulted(error) => error,
    }
}

/// The operation a native builder charges: its own, or a host's.
enum ConstructionLedger {
    Owned(Operation<HtmlWork, HtmlResource>),
    Shared(SharedOperation<HtmlWork, HtmlResource>),
}
impl ConstructionLedger {
    fn apply_batch(
        &mut self,
        scope: Scope,
        batch: &[Change<HtmlWork, HtmlResource>],
    ) -> Result<(), LedgerError<HtmlWork, HtmlResource>> {
        match self {
            Self::Owned(operation) => operation.apply_batch(scope, batch),
            Self::Shared(operation) => operation.apply_batch(scope, batch),
        }
    }
    fn record_largest(
        &mut self,
        scope: Scope,
        resource: HtmlResource,
        value: u64,
    ) -> Result<(), LedgerError<HtmlWork, HtmlResource>> {
        match self {
            Self::Owned(operation) => operation.charge_at(scope).record_largest(resource, value),
            Self::Shared(operation) => operation.charge_at(scope).record_largest(resource, value),
        }
    }
    fn usage(
        &self,
        scope: Scope,
    ) -> Result<Usage<HtmlWork, HtmlResource>, LedgerError<HtmlWork, HtmlResource>> {
        match self {
            Self::Owned(operation) => operation.usage(scope).copied(),
            Self::Shared(operation) => operation.usage(scope),
        }
    }
}

/// The construction scope of a native builder: the caps of its limits, enforced by a ledger scope,
/// and the typed usage read back from that scope after each charge. The scope is the root of an
/// operation of the builder's own, or a child of the scope a host supplies; a host operation keeps
/// the admitted amounts after the builder and its result drop.
struct Accounting {
    operation: ConstructionLedger,
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
    #[cfg(test)]
    fn new(limits: HtmlConstructionLimits) -> Result<Self, HtmlConstructionError> {
        Self::charging(limits, None)
    }
    /// The accounting of a builder: a scope below the one `host` supplies when it is `Some`, else
    /// the root of an operation of its own.
    fn charging(
        limits: HtmlConstructionLimits,
        host: Option<&HtmlHostScope>,
    ) -> Result<Self, HtmlConstructionError> {
        let caps = crate::internal::accounting::construction_limits(limits);
        let (operation, scope) = match host {
            Some(host) => {
                let scope = host
                    .ledger()
                    .child(
                        host.scope(),
                        HtmlCapScope::Construction.name(),
                        caps,
                        aggregate_tariffs(),
                    )
                    .map_err(ledger_error)?;
                (ConstructionLedger::Shared(host.ledger().clone()), scope)
            }
            None => {
                let operation = Operation::new(HtmlCapScope::Construction.name(), caps, 1)
                    .map_err(ledger_error)?;
                let scope = operation.root();
                (ConstructionLedger::Owned(operation), scope)
            }
        };
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
            return Err(
                HtmlConstructionCause::Accounting(HtmlAccountingFault::UnexpectedChange).into(),
            );
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
                .record_largest(self.scope, HtmlResource::TreeDepth, wide(previous))
                .map_err(ledger_error)?;
        }
        self.read()
    }
    /// Reads the typed usage from the scope.
    fn read(&mut self) -> Result<(), HtmlConstructionError> {
        let usage = self.operation.usage(self.scope).map_err(ledger_error)?;
        let read = |value: u64| {
            usize::try_from(value).map_err(|_| HtmlConstructionCause::RepresentationExhausted)
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
        HtmlLedgerOutcome::Fault(fault) => HtmlConstructionCause::Accounting(fault),
        HtmlLedgerOutcome::Exhausted(_) => HtmlConstructionCause::RepresentationExhausted,
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
                (Some(resource), Ok(figures)) => HtmlConstructionCause::LimitExceeded {
                    resource,
                    limit: figures.limit,
                    attempted: figures.required,
                },
                (None, _) => {
                    HtmlConstructionCause::Accounting(HtmlAccountingFault::UnexpectedChange)
                }
                (Some(_), Err(_)) => HtmlConstructionCause::RepresentationExhausted,
            }
        }
    };
    kind.into()
}

#[derive(Debug)]
struct Builder {
    storage: bounded::HtmlConstructionStorage,
    /// The root issued when the builder was created, which the storage keeps for its life.
    root: HtmlBuilderNode,
    accounting: Accounting,
}
impl Builder {
    /// The usage the builder's accounting admitted.
    fn usage(&self) -> HtmlConstructionUsage {
        self.accounting.usage
    }
    fn new(
        limits: HtmlConstructionLimits,
        root: NativeRoot,
    ) -> Result<Self, HtmlConstructionError> {
        Self::charging(limits, root, None)
    }
    /// A builder that charges below the scope `host` supplies when it is `Some`, else an
    /// operation of its own.
    fn charging(
        limits: HtmlConstructionLimits,
        root: NativeRoot,
        host: Option<&HtmlHostScope>,
    ) -> Result<Self, HtmlConstructionError> {
        let mut admission = NativeAdmission::exact();
        let mut storage = bounded::HtmlConstructionStorage::new(&mut admission).map_err(native)?;
        let mut accounting = Accounting::charging(limits, host)?;
        accounting.admit(1, 0, 0, 0, 0)?;
        let root = match root {
            NativeRoot::Document(mode) => storage.create_document(mode, &mut admission),
            NativeRoot::Fragment => storage.create_fragment(&mut admission),
        }
        .map_err(native)?;
        Ok(Self {
            storage,
            root,
            accounting,
        })
    }
    /// Undoes an uncommitted append: the records it created and what it admitted. A failure of
    /// the undo itself is a defect and takes the place of `error`, which it follows.
    fn undo_append(
        &mut self,
        charged: Charged,
        len: usize,
        usage: HtmlConstructionUsage,
        error: HtmlConstructionError,
    ) -> HtmlConstructionError {
        if let Err(defect) = self.storage.rollback_native_append(len, usage) {
            // The accounting still holds the append, which the defect leaves unrecoverable.
            return defect;
        }
        match self.accounting.release(charged) {
            Ok(()) => error,
            Err(release) => release,
        }
    }
    fn append(
        &mut self,
        parent: &HtmlBuilderNode,
        creation: Creation,
    ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
        let parent_position = self.storage.position(parent)?;
        let parent_view = self.storage.node(parent)?;
        if !matches!(
            parent_view.kind(),
            ConstructionNodeKind::Document(_)
                | ConstructionNodeKind::Fragment
                | ConstructionNodeKind::Element(_)
        ) {
            return Err(HtmlConstructionCause::ParentCannotHaveChildren.into());
        }
        if parent_view.template_contents().is_some() {
            return Err(HtmlConstructionCause::TemplateContentsRequired.into());
        }
        let template = creation.is_template();
        let depth = self
            .storage
            .native_depth(parent_position)?
            .checked_add(1 + usize::from(template))
            .ok_or(HtmlConstructionCause::RepresentationExhausted)?;
        let (attributes, units) = creation.usage()?;
        let charged = self.accounting.admit(
            1 + usize::from(template),
            1 + usize::from(template),
            attributes,
            units,
            depth,
        )?;
        let previous_usage = self.storage.usage();
        let previous_len = self.storage.len();
        let mut admission = NativeAdmission::with_spare_room(self.accounting.limits);
        let created = match creation.create(&mut self.storage, &mut admission) {
            Ok(created) => created,
            Err(error) => {
                self.accounting.release(charged)?;
                return Err(error);
            }
        };
        if let Err(error) = self
            .storage
            .insert(&created, parent, None, &mut admission)
            .map_err(native)
        {
            return Err(self.undo_append(charged, previous_len, previous_usage, error));
        }
        Ok(created)
    }
    fn template_contents(
        &self,
        template: &HtmlBuilderNode,
    ) -> Result<Option<HtmlBuilderNode>, HtmlConstructionError> {
        Ok(self
            .storage
            .node(template)?
            .template_contents()
            .map(bounded::HtmlConstructionNode::handle))
    }
    fn shadow(
        &mut self,
        host: &HtmlBuilderNode,
        properties: HtmlShadowRootProperties,
    ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
        let host_position = self.storage.position(host)?;
        let view = self.storage.node(host)?;
        if !matches!(view.kind(), ConstructionNodeKind::Element(_)) {
            return Err(HtmlConstructionCause::ShadowHostMustBeElement.into());
        }
        if view.shadow_root().is_some() {
            return Err(HtmlConstructionCause::ShadowAlreadyAttached.into());
        }
        let depth = self
            .storage
            .native_depth(host_position)?
            .checked_add(1)
            .ok_or(HtmlConstructionCause::RepresentationExhausted)?;
        let charged = self.accounting.admit(1, 1, 0, 0, depth)?;
        let previous_usage = self.storage.usage();
        let previous_len = self.storage.len();
        let mut admission = NativeAdmission::with_spare_room(self.accounting.limits);
        let contents = match self
            .storage
            .create_shadow_contents(properties, &mut admission)
            .map_err(native)
        {
            Ok(contents) => contents,
            Err(error) => {
                self.accounting.release(charged)?;
                return Err(error);
            }
        };
        if let Err(error) = self
            .storage
            .attach_shadow(host, &contents, &mut admission)
            .map_err(native)
        {
            return Err(self.undo_append(charged, previous_len, previous_usage, error));
        }
        Ok(contents)
    }
    /// Publishes the whole arena. The builder is consumed: a refusal here means the tree failed
    /// the publication's own validation, which the append steps are meant to make impossible, so
    /// no half-published builder is worth keeping.
    fn finish(self) -> Result<bounded::HtmlConstructionResult, HtmlConstructionError> {
        let mut admission = NativeAdmission::exact();
        self.storage
            .publish(&self.root, &mut admission)
            .map_err(|refusal| native(refusal.into_parts().1))
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
                Creation::Element {
                    link_flags: (crate::vocabulary::KnownHtmlElement::from_name(&name)
                        == Some(crate::vocabulary::KnownHtmlElement::Link))
                    .then(crate::internal::elements::rel::HtmlLinkFlags::fresh),
                    name,
                    attributes,
                },
            )
        }
        /// Appends exact text, retaining nulls and isolated surrogates.
        pub fn append_text(
            &mut self,
            parent: &HtmlBuilderNode,
            value: impl Into<DocumentString>,
        ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
            self.builder.append(parent, Creation::Text(value.into()))
        }
        /// Appends exact comment data without imposing serialized-comment syntax.
        pub fn append_comment(
            &mut self,
            parent: &HtmlBuilderNode,
            value: impl Into<DocumentString>,
        ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
            self.builder.append(parent, Creation::Comment(value.into()))
        }
        /// Appends a checked native processing instruction.
        pub fn append_processing_instruction(
            &mut self,
            parent: &HtmlBuilderNode,
            value: HtmlAuthoredProcessingInstruction,
        ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
            self.builder
                .append(parent, Creation::ProcessingInstruction(value.into()))
        }
        /// Appends exact doctype data without deriving document mode.
        pub fn append_doctype(
            &mut self,
            parent: &HtmlBuilderNode,
            value: HtmlDoctype,
        ) -> Result<HtmlBuilderNode, HtmlConstructionError> {
            self.builder.append(parent, Creation::Doctype(value))
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
                Err(HtmlConstructionCause::InvalidOutputRoot.into())
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
                Err(HtmlConstructionCause::InvalidOutputRoot.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// How many times the record bank and the root's child order change capacity over `appends`
    /// native appends, each of which makes room for one more record, and one more child of the
    /// root.
    fn capacity_changes(appends: usize, nested: bool) -> (usize, usize) {
        let mut builder = Builder::new(
            HtmlConstructionLimits::new(appends + 1, appends, 0, 8 * appends, appends + 1),
            NativeRoot::Fragment,
        )
        .unwrap();
        let root = builder.root;
        let mut parent = root;
        let mut previous = (
            builder.storage.records_capacity(),
            builder.storage.children_capacity(0),
        );
        let (mut records, mut children) = (0, 0);
        for _ in 0..appends {
            let kind = if nested {
                Creation::Element {
                    name: QualifiedName::new(Namespace::from_uri("urn:x"), None, "e"),
                    attributes: OrderedAttributes::new(Vec::new()).unwrap(),
                    link_flags: None,
                }
            } else {
                Creation::Comment("c".into())
            };
            let appended = builder.append(&parent, kind).unwrap();
            if nested {
                parent = appended;
            }
            let storage = &builder.storage;
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
        let root = builder.root;
        for _ in 0..edges {
            builder
                .append(&root, Creation::Comment("c".into()))
                .unwrap();
            let storage = &builder.storage;
            assert!(storage.records_capacity() <= nodes);
            assert!(storage.children_capacity(0).unwrap() <= edges);
        }
        assert_eq!(builder.storage.records_capacity(), nodes);
        assert_eq!(builder.storage.children_capacity(0), Some(edges));
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
        let root = builder.root;
        let _refusing = crate::internal::amortized::fault::arm(1);
        for count in 1..=appends {
            builder
                .append(&root, Creation::Comment("c".into()))
                .unwrap();
            let storage = &builder.storage;
            assert_eq!(storage.len(), count + 1);
            assert_eq!(storage.records_capacity(), count + 1);
            assert_eq!(storage.children_capacity(0), Some(count));
        }
        assert!(crate::internal::amortized::fault::requests() >= 2 * appends - 1);
    }

    #[test]
    fn a_node_index_names_every_position_below_the_32_bit_maximum_and_no_other() {
        assert_eq!(NodeIx::FIRST.position(), 0);
        assert_eq!(NodeIx::new(0), Some(NodeIx::FIRST));
        assert_eq!(NodeIx::default(), NodeIx::FIRST);
        // The positions u32::MAX - 1 and u32::MAX - 2 are the last two that fit a stored
        // position plus one in 32 bits.
        let last = u32::MAX as usize - 1;
        assert_eq!(NodeIx::new(last).map(NodeIx::position), Some(last));
        assert_eq!(NodeIx::new(last - 1).map(NodeIx::position), Some(last - 1));
        assert_eq!(NodeIx::new(last + 1), None);
        assert_eq!(NodeIx::new(usize::MAX), None);
        // An offset keeps to the same bound and refuses an overflowing sum.
        assert_eq!(NodeIx::at(3).offset(4).map(NodeIx::position), Some(7));
        assert_eq!(
            NodeIx::at(last - 1).offset(1).map(NodeIx::position),
            Some(last)
        );
        assert_eq!(NodeIx::at(last - 1).offset(2), None);
        assert_eq!(NodeIx::at(1).offset(usize::MAX), None);
        // A position that exists can be absent in the same four bytes.
        assert_eq!(size_of::<Option<NodeIx>>(), size_of::<NodeIx>());
        assert_eq!(size_of::<NodeIx>(), 4);
    }

    #[test]
    fn unqualified_document_record_readback_is_not_a_node_kind_constant() {
        let arena = Arena {
            records: vec![Record::new(
                NodeContent::Document(HtmlDocumentMode::NoQuirks),
                None,
                0,
            )],
            names: NameTable::new(),
            usage: HtmlConstructionUsage::default(),
            serial: 0,
            attribute_revision: 0,
        };
        let node = HtmlNode {
            arena: &arena,
            index: NodeIx::FIRST,
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
            NodeContent::<DocumentString>::ProcessingInstruction(recovered.clone()),
            None,
            0,
        );
        let NodeContent::ProcessingInstruction(stored) = record.kind else {
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
        let element = |attributes: Vec<crate::internal::values::Attribute>| Creation::Element {
            name: QualifiedName::new(Namespace::from_uri("urn:x"), Some("p".into()), "tag"),
            attributes: OrderedAttributes::new(attributes).unwrap(),
            link_flags: None,
        };
        let cases: Vec<(&str, Creation, usize, usize)> = vec![
            (
                "text with an isolated surrogate pair and a null",
                Creation::Text(DocumentString::from_utf16(vec![0xD83D, 0xDE00, 0])),
                0,
                3,
            ),
            (
                "comment",
                Creation::Comment(DocumentString::from("xyz")),
                0,
                3,
            ),
            (
                "processing instruction",
                Creation::ProcessingInstruction(HtmlProcessingInstruction {
                    target: DocumentString::from("xy"),
                    data: DocumentString::from("abc"),
                }),
                0,
                5,
            ),
            (
                "doctype with both identifiers",
                Creation::Doctype(HtmlDoctype::new("h", Some("ab".into()), Some("xyz".into()))),
                0,
                6,
            ),
            (
                "doctype with only a present empty public identifier",
                Creation::Doctype(HtmlDoctype::new(
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
                let root = builder.root;
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
                Err(HtmlConstructionCause::LimitExceeded {
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
                    Err(HtmlConstructionCause::LimitExceeded {
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
        match error.cause {
            HtmlConstructionCause::LimitExceeded {
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
        assert!(!matches!(
            error.cause(),
            HtmlConstructionCause::ReleaseUnderflow
        ));
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
            assert!(!matches!(
                error.cause(),
                HtmlConstructionCause::AllocationFailed
                    | HtmlConstructionCause::RepresentationExhausted
            ));
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
        let error = HtmlConstructionCause::LimitExceeded {
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

    /// Every cause states its documented meaning, and the error built from it
    /// reports exactly that cause.
    #[test]
    fn every_cause_has_its_message_and_is_reported_as_itself() {
        let cases: Vec<(HtmlConstructionCause, &str)> = vec![
            (
                HtmlConstructionCause::ForeignBuilder,
                "node handle belongs to a different builder",
            ),
            (
                HtmlConstructionCause::ParentCannotHaveChildren,
                "text, comment, processing-instruction and doctype nodes cannot have children",
            ),
            (
                HtmlConstructionCause::TemplateContentsRequired,
                "template children must be appended to its separate contents",
            ),
            (
                HtmlConstructionCause::ShadowHostMustBeElement,
                "a shadow root requires an element host",
            ),
            (
                HtmlConstructionCause::ShadowAlreadyAttached,
                "element already has a shadow root",
            ),
            (
                HtmlConstructionCause::UnknownNode,
                "node does not exist in this builder",
            ),
            (
                HtmlConstructionCause::WrongNodeKind,
                "node kind cannot take part in this command",
            ),
            (
                HtmlConstructionCause::RootMove,
                "documents and fragments cannot be moved or detached",
            ),
            (
                HtmlConstructionCause::AssociationMove,
                "template contents and shadow fragments cannot be moved as ordinary children",
            ),
            (
                HtmlConstructionCause::WrongSibling,
                "reference node is not a child of the target parent",
            ),
            (
                HtmlConstructionCause::Cycle,
                "move would make a node its own ancestor",
            ),
            (
                HtmlConstructionCause::AssociationAlreadyOwned,
                "fragment is already associated with another owner",
            ),
            (
                HtmlConstructionCause::InvalidTemplateAssociation,
                "template contents association does not satisfy the template rules",
            ),
            (
                HtmlConstructionCause::InvalidShadowAssociation,
                "shadow association does not satisfy the shadow host rules",
            ),
            (
                HtmlConstructionCause::InvalidContentsReference,
                "replacement template contents are the existing contents",
            ),
            (
                HtmlConstructionCause::ParentChildConflict,
                "parent and child records disagree about their relationship",
            ),
            (
                HtmlConstructionCause::InvalidOutputRoot,
                "published output root is not a detached document or fragment",
            ),
            (
                HtmlConstructionCause::OperandMissing,
                "an operand the operation requires is absent",
            ),
            (
                HtmlConstructionCause::CapacityMismatch,
                "bank capacity is below what the operation stores",
            ),
            (
                HtmlConstructionCause::IndexUnavailable,
                "the attribute name index lent to the edit could not serve a step",
            ),
            (
                HtmlConstructionCause::SharedAllocationAliased,
                "a string of the text block taken back is held by another owner",
            ),
            (
                HtmlConstructionCause::InvalidAttributeReference,
                "an element or attribute row names a run, row or value its storage does not hold",
            ),
            (
                HtmlConstructionCause::ReleaseUnderflow,
                "released usage exceeds the admitted usage",
            ),
            (
                HtmlConstructionCause::RepresentationExhausted,
                "a storage count, layout size, record index or storage serial has no representable value",
            ),
            (
                HtmlConstructionCause::AllocationFailed,
                "reserving arena or edge storage failed",
            ),
            (
                HtmlConstructionCause::DuplicateAttribute {
                    first: 0,
                    duplicate: 1,
                },
                "attribute at index 1 duplicates the namespace and local name of the attribute at index 0",
            ),
            (
                HtmlConstructionCause::LimitExceeded {
                    resource: HtmlConstructionResource::Nodes,
                    limit: 1,
                    attempted: 2,
                },
                "node limit of 1 cannot admit a total of 2",
            ),
            (
                HtmlConstructionCause::Accounting(HtmlAccountingFault::ForeignScope),
                "the ledger scope that enforces the limits failed: the supplied scope belongs to another operation",
            ),
        ];
        for (cause, message) in &cases {
            let error = cause.error();
            assert_eq!(error.to_string(), *message);
            assert_eq!(error.cause(), *cause, "{message}");
            let matching = cases.iter().filter(|(other, _)| other == cause).count();
            assert_eq!(matching, 1, "{message}");
        }
    }
}
