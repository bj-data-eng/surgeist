//! Bounded immutable context facts, independent of canonical DOM identity.
//!
//! Fragment queries follow HTML 877d6146487f §13.4 and §13.2.4–6. Captured
//! availability describes producer evidence; it never advertises host execution.

use std::{collections::HashSet, error::Error, fmt, hash::Hash, sync::Arc};
use surgeist_html::accounting::HtmlAccountingFault;
use surgeist_html::accounting::HtmlHostScope;
use surgeist_html::bounded::lexical::validate_custom_element_name;
use surgeist_html::{
    construction::{HtmlDoctype, HtmlDocumentMode, HtmlShadowRootMode, HtmlShadowRootProperties},
    elements::native::HtmlCustomElementState,
    parsing::{HtmlFragmentInsertionMode, HtmlFragmentTokenizerState, HtmlParserScriptingMode},
    security::nonce::{
        HtmlBrowsingContextPresence, HtmlCapturedNonce, HtmlHeaderDeliveredCsp,
        HtmlNonceDocumentEnvironment,
    },
    values::{DocumentString, Namespace, OrderedAttributes, QualifiedName},
};

pub(crate) mod account;
mod construction;
mod element_input;
pub(crate) mod preflight;
mod registry;
pub(crate) mod storage;
pub use construction::*;
pub(crate) use element_input::FormFailure;
pub use element_input::HtmlCaptureElementError;
pub use registry::*;
pub use storage::{HtmlCaptureRetainedStorageUsage, HtmlCaptureStorageError};

identity!(
    /// Opaque identity of one immutable capture, independent of equal content.
    HtmlCaptureId,
    hash
);
identity!(
    /// Producer-issued identity of one coherent immutable host-fact revision.
    HtmlHostFactsRevision,
    fresh,
    hash
);

/// A subject qualified by its immutable capture; never a canonical DOM ID.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlCaptureSubjectId {
    capture: HtmlCaptureId,
    index: usize,
}
impl HtmlCaptureSubjectId {
    /// Owning capture identity.
    pub fn capture(&self) -> &HtmlCaptureId {
        &self.capture
    }
    /// The dense position of this subject within its own capture: the first
    /// subject added to a capture has position 0, and positions follow the order
    /// subjects were added, without gaps. A position is meaningful only together
    /// with [`capture`](Self::capture): subjects of different captures can share
    /// one. It identifies no canonical DOM node, and it is not a stable identity
    /// across captures. A consumer indexing by it compares `capture()` first, then
    /// checks the entry it finds, because a position outside this capture's
    /// subjects, or one that belongs to another capture, names no subject here.
    pub fn ordinal(&self) -> usize {
        self.index
    }
}
/// Local handle issued by one one-shot capture builder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCaptureBuilderSubject {
    capture: HtmlCaptureId,
    index: usize,
}
/// Local registry handle issued by one capture builder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCaptureBuilderRegistry {
    capture: HtmlCaptureId,
    index: usize,
}
/// Registry metadata identity qualified by a capture.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlCaptureRegistryId {
    capture: HtmlCaptureId,
    index: usize,
}
/// A checked record handle issued by one capture builder.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCaptureBuilderDefinition {
    capture: HtmlCaptureId,
    registry_index: usize,
    record_index: usize,
}
/// A definition record qualified by its owning captured registry.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct HtmlCaptureDefinitionId {
    registry: HtmlCaptureRegistryId,
    record_index: usize,
}
impl HtmlCaptureDefinitionId {
    pub fn registry(&self) -> &HtmlCaptureRegistryId {
        &self.registry
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCapturedCustomElementState {
    Known(HtmlCustomElementState),
    Unavailable,
}
/// Actual association supplied independently of registry-set completeness.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCaptureDefinitionInput {
    Null,
    Definition(HtmlCaptureBuilderDefinition),
    Unavailable,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCapturedCustomElementDefinition {
    Null,
    Definition(HtmlCaptureDefinitionId),
    Unavailable,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct CustomElementContext {
    state: HtmlCapturedCustomElementState,
    definition: HtmlCapturedCustomElementDefinition,
}
macro_rules! subject_kind_id {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        pub struct $name(HtmlCaptureSubjectId);
        impl $name {
            /// Underlying checked subject identity.
            pub fn subject(&self) -> &HtmlCaptureSubjectId {
                &self.0
            }
        }
    };
}
subject_kind_id!(HtmlCaptureDocumentId, "A checked document subject.");
subject_kind_id!(HtmlCaptureElementId, "A checked element subject.");
subject_kind_id!(HtmlCaptureFragmentId, "A checked DocumentFragment subject.");

/// Established context-document scripting status at the captured host revision.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlScriptingStatus {
    Enabled,
    Disabled,
    Unavailable,
}
/// Captured image policy at the host revision. Its declared applicability
/// distinguishes a target-only observation from a parser creation environment;
/// it is not intrinsic Document state or inferred from scripting or a loader.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlImageSupport {
    Unavailable,
    Disabled,
    Enabled,
}
/// Captured internal creation slot, independent of the current `is` attribute.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCapturedIsValue {
    Absent,
    Value(DocumentString),
    Unavailable,
}
/// Registry input with explicit absence or unavailable producer evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCaptureRegistryInput {
    Null,
    Registry(HtmlCaptureBuilderRegistry),
    Unavailable,
}
/// Frozen registry fact. Complete empty metadata and null identity differ.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCapturedRegistry {
    Null,
    Registry(HtmlCaptureRegistryId),
    Unavailable,
}
/// Registry scope influences DOM adoption; unavailable is not global scope.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlRegistryScope {
    Global,
    Scoped,
    Unavailable,
}
/// Source-qualified document type, independent of namespace and quirks mode.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlDocumentClass {
    Html,
    NonHtml,
    Unavailable,
}
/// Whether the document was created by the template contents owner algorithm.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlTemplateDocumentRole {
    Ordinary,
    InertTemplate,
    Unavailable,
}
/// Builder-qualified associated inert template document cache.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCaptureTemplateCacheInput {
    Absent,
    Document(HtmlCaptureBuilderSubject),
    Unavailable,
}
/// Captured cache identity; absence and unavailable evidence differ.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCapturedTemplateCache {
    Absent,
    Document(HtmlCaptureDocumentId),
    Unavailable,
}

/// Ordinary parent, separate from template and shadow host associations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCapturedParent {
    Root,
    Subject(HtmlCaptureSubjectId),
    Unavailable,
}
/// Complete ordered children, including established empty, or unavailable.
#[derive(Clone, Debug)]
pub enum HtmlCapturedChildren<'a> {
    Complete(HtmlCapturedChildIds<'a>),
    Unavailable,
}
/// Borrowed ordered child IDs without copying a captured subtree.
#[derive(Clone, Debug)]
pub struct HtmlCapturedChildIds<'a> {
    identity: &'a HtmlCaptureId,
    indices: std::slice::Iter<'a, usize>,
}
impl Iterator for HtmlCapturedChildIds<'_> {
    type Item = HtmlCaptureSubjectId;
    fn next(&mut self) -> Option<Self::Item> {
        self.indices.next().map(|&index| HtmlCaptureSubjectId {
            capture: self.identity.clone(),
            index,
        })
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.indices.size_hint()
    }
}
impl DoubleEndedIterator for HtmlCapturedChildIds<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.indices.next_back().map(|&index| HtmlCaptureSubjectId {
            capture: self.identity.clone(),
            index,
        })
    }
}
impl ExactSizeIterator for HtmlCapturedChildIds<'_> {}
/// DocumentFragment host, independent of its ordinary parent.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCapturedHost {
    Absent,
    Element(HtmlCaptureElementId),
    Unavailable,
}
/// Underlying shadow-root presence, including closed roots.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCapturedShadowRoot {
    Absent,
    Present(HtmlCaptureFragmentId),
    Unavailable,
}
/// HTML template contents are associated fragments, never ordinary children.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCapturedTemplateContents {
    Present(HtmlCaptureFragmentId),
    Unavailable,
}

/// Pure definition facts used by HTML lookup and shadow attachment.
/// Constructors, callback objects and runtime upgrade operations are not stored.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCustomElementDefinition {
    name: DocumentString,
    local_name: DocumentString,
    form_associated: bool,
    disable_shadow: bool,
}
impl HtmlCustomElementDefinition {
    /// Supplies captured definition metadata; capture admission validates names.
    pub fn new(
        name: DocumentString,
        local_name: DocumentString,
        form_associated: bool,
        disable_shadow: bool,
    ) -> Self {
        Self {
            name,
            local_name,
            form_associated,
            disable_shadow,
        }
    }
    pub fn name(&self) -> &DocumentString {
        &self.name
    }
    pub fn local_name(&self) -> &DocumentString {
        &self.local_name
    }
    pub fn form_associated(&self) -> bool {
        self.form_associated
    }
    pub fn disable_shadow(&self) -> bool {
        self.disable_shadow
    }
}

/// Finite capture retention limits. Relationships count every retained non-self
/// reference: node-document, registry, ordinary parent, internal backlink,
/// ordered child and reciprocal template/shadow association. A document's own
/// node-document identity is covered by its subject count, not a relationship.
/// Document roots have depth zero.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlCaptureLimits {
    subjects: usize,
    relationships: usize,
    attributes: usize,
    utf16_units: usize,
    depth: usize,
    registries: usize,
    definitions: usize,
    element_value_bytes: usize,
}
impl HtmlCaptureLimits {
    #[expect(
        clippy::too_many_arguments,
        reason = "Each argument is a required independent finite capture policy; no implicit or default storage allowance is supplied."
    )]
    pub fn new(
        subjects: usize,
        relationships: usize,
        attributes: usize,
        utf16_units: usize,
        depth: usize,
        registries: usize,
        definitions: usize,
        element_value_bytes: usize,
    ) -> Self {
        Self {
            subjects,
            relationships,
            attributes,
            utf16_units,
            depth,
            registries,
            definitions,
            element_value_bytes,
        }
    }
    pub fn subjects(self) -> usize {
        self.subjects
    }
    pub fn relationships(self) -> usize {
        self.relationships
    }
    pub fn attributes(self) -> usize {
        self.attributes
    }
    pub fn utf16_units(self) -> usize {
        self.utf16_units
    }
    pub fn depth(self) -> usize {
        self.depth
    }
    pub fn registries(self) -> usize {
        self.registries
    }
    pub fn definitions(self) -> usize {
        self.definitions
    }
    /// Cumulative element name, attribute and saved-is storage, including pending
    /// formation. This is not a ceiling for the complete capture heap.
    pub fn element_value_bytes(self) -> usize {
        self.element_value_bytes
    }
}
/// Admitted capture retention, read from the capture's ledger scope.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlCaptureUsage {
    subjects: usize,
    relationships: usize,
    attributes: usize,
    utf16_units: usize,
    depth: usize,
    registries: usize,
    definitions: usize,
    element_value_bytes: usize,
}
impl HtmlCaptureUsage {
    pub fn subjects(self) -> usize {
        self.subjects
    }
    pub fn relationships(self) -> usize {
        self.relationships
    }
    pub fn attributes(self) -> usize {
        self.attributes
    }
    pub fn utf16_units(self) -> usize {
        self.utf16_units
    }
    pub fn depth(self) -> usize {
        self.depth
    }
    pub fn registries(self) -> usize {
        self.registries
    }
    pub fn definitions(self) -> usize {
        self.definitions
    }
    /// Retained element-value storage, conservatively counted per occurrence.
    pub fn element_value_bytes(self) -> usize {
        self.element_value_bytes
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCaptureResource {
    Subjects,
    Relationships,
    Attributes,
    Utf16Units,
    Depth,
    Registries,
    Definitions,
    ElementValueBytes,
}
/// Required producer fact that has not been established.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCaptureFact {
    BrowsingContextConnection,
    HeaderDeliveredCsp,
    Parent,
    FragmentHost,
    ScriptingStatus,
    ImageSupport,
    Registry,
    Definitions,
    IsValue,
    ShadowRoot,
    DocumentClass,
    TemplateDocumentRole,
    TemplateDocumentCache,
    RegistryScope,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCaptureSubjectKind {
    Document,
    Element,
    Fragment,
    Text,
    Comment,
    ProcessingInstruction,
    Doctype,
}
/// Refusal of a capture operation whose work the caller meters: establishing an
/// edge, or walking to the nearest form. An interruption is the caller's own
/// first refusal, returned without translation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCaptureEdgeError<E> {
    /// The capture refused the operation itself, for example an edge as a cycle
    /// or beyond a capture limit, or a walk through unavailable ancestry.
    Capture(HtmlCaptureError),
    /// The caller's step refused to continue; an edge was not established and a
    /// walk ended without an answer.
    Interrupted(E),
}
impl<E> HtmlCaptureEdgeError<E> {
    /// The accounting fault of the ledger that carried the operation, when this error is one or
    /// wraps an error that is one. The match lists every variant, so a new variant that wraps an
    /// error is decided here.
    pub fn accounting_fault(&self) -> Option<surgeist_html::accounting::HtmlAccountingFault> {
        match self {
            Self::Capture(error) => error.accounting_fault(),
            Self::Interrupted(_) => None,
        }
    }
}
impl<E> From<HtmlCaptureError> for HtmlCaptureEdgeError<E> {
    fn from(error: HtmlCaptureError) -> Self {
        Self::Capture(error)
    }
}
impl<E: fmt::Display> fmt::Display for HtmlCaptureEdgeError<E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Capture(error) => error.fmt(f),
            Self::Interrupted(error) => error.fmt(f),
        }
    }
}
impl<E: Error + 'static> Error for HtmlCaptureEdgeError<E> {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Capture(error) => Some(error),
            Self::Interrupted(error) => Some(error),
        }
    }
}
/// Rejected assembly, lookup or source-derived context admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCaptureError {
    NotHtmlLink,
    NotHtmlHyperlink,
    HyperlinkUrlAlreadyEstablished,
    HyperlinkHrefMismatch,
    LinkHistoryAlreadyEstablished,
    ForeignBuilder,
    ForeignCapture,
    WrongKind {
        expected: HtmlCaptureSubjectKind,
        actual: HtmlCaptureSubjectKind,
    },
    MissingFact {
        subject: Option<HtmlCaptureSubjectId>,
        fact: HtmlCaptureFact,
    },
    UnavailableDefinitions(HtmlCaptureRegistryId),
    ParentAlreadyEstablished,
    ChildrenAlreadyEstablished,
    ParentChildConflict,
    DuplicateChild,
    DocumentMismatch,
    AssociationAlreadyEstablished,
    FragmentAlreadyHosted,
    UnhostedFragment,
    NotHtmlTemplate,
    CannotHaveChildren(HtmlCaptureSubjectKind),
    CannotBeOrdinaryChild(HtmlCaptureSubjectKind),
    Cycle,
    InvalidDefinitionName,
    UnknownDefinition,
    CompleteRegistry,
    DocumentFactsAlreadyEstablished,
    RegistryScopeAlreadyEstablished,
    InvalidTemplateDocumentCache,
    /// The target kind is invalid for the selected owning or non-owning contents relation.
    InvalidTemplateContentsReference,
    /// A shadow root with an unknown external host cannot establish host absence.
    ShadowRootCannotBeUnhosted,
    CustomElementContextAlreadyEstablished,
    CustomElementDefinitionRequired,
    CustomElementDefinitionNameMismatch,
    CustomElementDefinitionNamespaceMismatch,
    CustomElementSavedIsMismatch,
    DuplicateDefinitionName,
    NonceAlreadyEstablished,
    NonceInapplicable,
    LimitExceeded {
        resource: HtmlCaptureResource,
        limit: usize,
        required: usize,
    },
    RepresentationExhausted,
    /// The ledger that carries the accounting failed in a way that is not a refused limit: a release
    /// that matches nothing held, a ledger that cannot allocate, a scope that is not the
    /// operation's, or a change the operation does not charge. It carries the accounting fault.
    Accounting(HtmlAccountingFault),
    /// A bounded HTML operation charged a change that the capture's work steps do not admit: a
    /// resource change or a gauge record. The capture charges only work, so an operation that does
    /// this breaks its contract with the capture.
    ChargeProtocol,
    InvalidFragmentScriptingMode(HtmlParserScriptingMode),
    UnsupportedExecution(HtmlParserScriptingMode),
}
impl HtmlCaptureError {
    /// The accounting fault of the ledger that carried the operation, when this error is one or
    /// wraps an error that is one. The match lists every variant, so a new variant that wraps an
    /// error is decided here.
    pub fn accounting_fault(&self) -> Option<surgeist_html::accounting::HtmlAccountingFault> {
        match self {
            Self::Accounting(fault) => Some(*fault),
            Self::NotHtmlLink
            | Self::NotHtmlHyperlink
            | Self::HyperlinkUrlAlreadyEstablished
            | Self::HyperlinkHrefMismatch
            | Self::LinkHistoryAlreadyEstablished
            | Self::ForeignBuilder
            | Self::ForeignCapture
            | Self::WrongKind { .. }
            | Self::MissingFact { .. }
            | Self::UnavailableDefinitions(_)
            | Self::ParentAlreadyEstablished
            | Self::ChildrenAlreadyEstablished
            | Self::ParentChildConflict
            | Self::DuplicateChild
            | Self::DocumentMismatch
            | Self::AssociationAlreadyEstablished
            | Self::FragmentAlreadyHosted
            | Self::UnhostedFragment
            | Self::NotHtmlTemplate
            | Self::CannotHaveChildren(_)
            | Self::CannotBeOrdinaryChild(_)
            | Self::Cycle
            | Self::InvalidDefinitionName
            | Self::UnknownDefinition
            | Self::CompleteRegistry
            | Self::DocumentFactsAlreadyEstablished
            | Self::RegistryScopeAlreadyEstablished
            | Self::InvalidTemplateDocumentCache
            | Self::InvalidTemplateContentsReference
            | Self::ShadowRootCannotBeUnhosted
            | Self::CustomElementContextAlreadyEstablished
            | Self::CustomElementDefinitionRequired
            | Self::CustomElementDefinitionNameMismatch
            | Self::CustomElementDefinitionNamespaceMismatch
            | Self::CustomElementSavedIsMismatch
            | Self::DuplicateDefinitionName
            | Self::NonceAlreadyEstablished
            | Self::NonceInapplicable
            | Self::LimitExceeded { .. }
            | Self::RepresentationExhausted
            | Self::ChargeProtocol
            | Self::InvalidFragmentScriptingMode(_)
            | Self::UnsupportedExecution(_) => None,
        }
    }
}
impl fmt::Display for HtmlCaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "HTML capture rejected: {self:?}")
    }
}
impl Error for HtmlCaptureError {}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Registry {
    Null,
    Index(usize),
    Unavailable,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Parent {
    Root,
    Index(usize),
    Unavailable,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Association {
    Absent,
    Index(usize),
    Unavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TemplateAssociation {
    Absent,
    Owned(usize),
    ShadowReference(usize),
    Unavailable,
}

/// Captured document facts; mode is actual data, scripting is explicit evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCapturedDocument {
    mode: HtmlDocumentMode,
    scripting: HtmlScriptingStatus,
    image_support: HtmlImageSupport,
    parser_creation_image_support: HtmlImageSupport,
    registry: Registry,
    class: HtmlDocumentClass,
    template_role: HtmlTemplateDocumentRole,
    template_cache: Association,
    template_facts_established: bool,
    nonce_environment: HtmlNonceDocumentEnvironment,
    nonce_environment_established: bool,
}
impl HtmlCapturedDocument {
    pub fn nonce_environment(&self) -> &HtmlNonceDocumentEnvironment {
        &self.nonce_environment
    }
    pub fn mode(&self) -> HtmlDocumentMode {
        self.mode
    }
    /// Target-only policy observation; it does not authorize parser-created
    /// temporary or inert documents.
    pub fn image_support(&self) -> HtmlImageSupport {
        self.image_support
    }
    /// Explicit policy attestation for this entry Document's parser creation
    /// environment. Its capture identity/host revision qualify that scope;
    /// target-only image_support does not establish it.
    pub fn parser_creation_image_support(&self) -> HtmlImageSupport {
        self.parser_creation_image_support
    }
    pub fn scripting(&self) -> HtmlScriptingStatus {
        self.scripting
    }
    pub fn document_class(&self) -> HtmlDocumentClass {
        self.class
    }
    pub fn template_role(&self) -> HtmlTemplateDocumentRole {
        self.template_role
    }
}
/// Exact captured element values and internal `is` slot.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCapturedElement {
    name: QualifiedName,
    attributes: OrderedAttributes,
    is_value: HtmlCapturedIsValue,
    registry: Registry,
    template: TemplateAssociation,
    shadow: Association,
    custom_context: Option<CustomElementContext>,
    nonce: HtmlCapturedNonce,
    link_history: Option<surgeist_html::elements::link::HtmlLinkHistory>,
    hyperlink_url: Option<crate::HtmlCapturedHyperlinkUrl>,
}
impl HtmlCapturedElement {
    pub fn hyperlink_url(&self) -> Option<&crate::HtmlCapturedHyperlinkUrl> {
        self.hyperlink_url.as_ref()
    }
    pub fn cryptographic_nonce(&self) -> &HtmlCapturedNonce {
        &self.nonce
    }
    pub fn link_history(&self) -> Option<surgeist_html::elements::link::HtmlLinkHistory> {
        self.link_history
    }
    pub fn custom_element_state(&self) -> HtmlCapturedCustomElementState {
        self.custom_context
            .as_ref()
            .map_or(HtmlCapturedCustomElementState::Unavailable, |c| c.state)
    }
    pub fn custom_element_definition(&self) -> &HtmlCapturedCustomElementDefinition {
        self.custom_context
            .as_ref()
            .map_or(&HtmlCapturedCustomElementDefinition::Unavailable, |c| {
                &c.definition
            })
    }
    pub fn name(&self) -> &QualifiedName {
        &self.name
    }
    pub fn attributes(&self) -> &OrderedAttributes {
        &self.attributes
    }
    pub fn is_value(&self) -> &HtmlCapturedIsValue {
        &self.is_value
    }
}
/// Source metadata of a captured existing shadow root; no runtime is implied.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCapturedShadowProperties {
    properties: HtmlShadowRootProperties,
    declarative: bool,
    available_to_internals: bool,
    keep_registry_null: bool,
}
impl HtmlCapturedShadowProperties {
    pub fn new(
        properties: HtmlShadowRootProperties,
        declarative: bool,
        available_to_internals: bool,
        keep_registry_null: bool,
    ) -> Self {
        Self {
            properties,
            declarative,
            available_to_internals,
            keep_registry_null,
        }
    }
    pub fn properties(&self) -> HtmlShadowRootProperties {
        self.properties
    }
    pub fn declarative(&self) -> bool {
        self.declarative
    }
    pub fn available_to_internals(&self) -> bool {
        self.available_to_internals
    }
    pub fn keep_registry_null(&self) -> bool {
        self.keep_registry_null
    }
}
/// Fragment class determines source-defined registry lookup.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCapturedFragmentKind {
    Plain,
    ShadowRoot(HtmlCapturedShadowProperties),
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCapturedFragment {
    kind: HtmlCapturedFragmentKind,
    host: Association,
    registry: Registry,
}
/// Exact producer PI payload, without native HTML authoring restrictions.
/// Generic capture and later parser recovery must retain their actual values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCapturedProcessingInstruction {
    target: DocumentString,
    data: DocumentString,
}
impl HtmlCapturedProcessingInstruction {
    pub(crate) fn controlled(target: DocumentString, data: DocumentString) -> Self {
        Self { target, data }
    }
    pub fn target(&self) -> &DocumentString {
        &self.target
    }
    pub fn data(&self) -> &DocumentString {
        &self.data
    }
}
impl HtmlCapturedFragment {
    pub fn kind(&self) -> &HtmlCapturedFragmentKind {
        &self.kind
    }
}
/// Exact immutable payloads; kind-specific builders supply only applicable facts.
#[expect(
    clippy::large_enum_variant,
    reason = "The capture retains each node payload inline and accounts that record layout; an indirection for elements would add an unadmitted allocation per element"
)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCapturedNodeKind {
    Document(HtmlCapturedDocument),
    Element(HtmlCapturedElement),
    Fragment(HtmlCapturedFragment),
    Text(DocumentString),
    Comment(DocumentString),
    ProcessingInstruction(HtmlCapturedProcessingInstruction),
    Doctype(HtmlDoctype),
}
impl HtmlCapturedNodeKind {
    /// The Document payload, or the wrong-kind refusal that `require` gives.
    fn document(&self) -> Result<&HtmlCapturedDocument, HtmlCaptureError> {
        match self {
            Self::Document(document) => Ok(document),
            other => Err(other.wrong_kind(HtmlCaptureSubjectKind::Document)),
        }
    }
    fn document_mut(&mut self) -> Result<&mut HtmlCapturedDocument, HtmlCaptureError> {
        match self {
            Self::Document(document) => Ok(document),
            other => Err(other.wrong_kind(HtmlCaptureSubjectKind::Document)),
        }
    }
    /// The Element payload, or the wrong-kind refusal that `require` gives.
    fn element(&self) -> Result<&HtmlCapturedElement, HtmlCaptureError> {
        match self {
            Self::Element(element) => Ok(element),
            other => Err(other.wrong_kind(HtmlCaptureSubjectKind::Element)),
        }
    }
    fn element_mut(&mut self) -> Result<&mut HtmlCapturedElement, HtmlCaptureError> {
        match self {
            Self::Element(element) => Ok(element),
            other => Err(other.wrong_kind(HtmlCaptureSubjectKind::Element)),
        }
    }
    /// The Fragment payload, or the wrong-kind refusal that `require` gives.
    fn fragment(&self) -> Result<&HtmlCapturedFragment, HtmlCaptureError> {
        match self {
            Self::Fragment(fragment) => Ok(fragment),
            other => Err(other.wrong_kind(HtmlCaptureSubjectKind::Fragment)),
        }
    }
    fn fragment_mut(&mut self) -> Result<&mut HtmlCapturedFragment, HtmlCaptureError> {
        match self {
            Self::Fragment(fragment) => Ok(fragment),
            other => Err(other.wrong_kind(HtmlCaptureSubjectKind::Fragment)),
        }
    }
    fn wrong_kind(&self, expected: HtmlCaptureSubjectKind) -> HtmlCaptureError {
        HtmlCaptureError::WrongKind {
            expected,
            actual: self.subject_kind(),
        }
    }
}
impl HtmlCapturedNodeKind {
    fn subject_kind(&self) -> HtmlCaptureSubjectKind {
        match self {
            Self::Document(_) => HtmlCaptureSubjectKind::Document,
            Self::Element(_) => HtmlCaptureSubjectKind::Element,
            Self::Fragment(_) => HtmlCaptureSubjectKind::Fragment,
            Self::Text(_) => HtmlCaptureSubjectKind::Text,
            Self::Comment(_) => HtmlCaptureSubjectKind::Comment,
            Self::ProcessingInstruction(_) => HtmlCaptureSubjectKind::ProcessingInstruction,
            Self::Doctype(_) => HtmlCaptureSubjectKind::Doctype,
        }
    }
}
#[derive(Debug)]
struct Record {
    kind: HtmlCapturedNodeKind,
    document: usize,
    parent: Parent,
    children: Option<Vec<usize>>,
    dependents: Vec<usize>,
    depth: DepthLink,
}
/// A record's depth, kept as a weighted union-find over the owning edges
/// (ordinary parent, shadow host and template contents) attached so far.
///
/// Each attached component is a tree whose root has no owning edge. Attaching a
/// root below a record only links the root, so no record of the attached
/// subtree is revisited, and the order in which a producer attaches edges
/// cannot change the cost. The depth of a record is the sum of the offsets on
/// its path to the root of its component, found with path halving. Depths that
/// are not asked for are never materialized.
#[derive(Debug)]
struct DepthLink {
    /// The next record towards the root; a root links to itself.
    link: usize,
    /// Edges from this record to `link`; zero for a root.
    offset: usize,
    /// Greatest depth below this record. Meaningful only while this record is a
    /// root; a record that gained an owner never reads it again.
    height: usize,
}
impl DepthLink {
    const fn root(index: usize) -> Self {
        Self {
            link: index,
            offset: 0,
            height: 0,
        }
    }
}
/// The outcome of admitting new edges below one parent, applied after every
/// fallible step of the call has succeeded.
struct DepthPlan {
    /// Root of the component holding the parent.
    root: usize,
    /// Edges from `root` to each attached child.
    first: usize,
    /// Greatest depth in the component once the children are attached.
    height: usize,
}
impl Record {
    fn incoming(&self) -> Option<usize> {
        match &self.kind {
            HtmlCapturedNodeKind::Fragment(f) => match f.host {
                Association::Index(i) => Some(i),
                _ => None,
            },
            _ => match self.parent {
                Parent::Index(i) => Some(i),
                _ => None,
            },
        }
    }
    fn is_container(&self) -> bool {
        matches!(
            self.kind,
            HtmlCapturedNodeKind::Document(_)
                | HtmlCapturedNodeKind::Element(_)
                | HtmlCapturedNodeKind::Fragment(_)
        )
    }
}
#[derive(Debug)]
struct RegistryRecord {
    definitions: Vec<HtmlCustomElementDefinition>,
    complete: bool,
    scope: HtmlRegistryScope,
    scope_established: bool,
}
#[derive(Debug)]
struct CaptureData {
    identity: HtmlCaptureId,
    host_revision: HtmlHostFactsRevision,
    records: Vec<Record>,
    registries: Vec<RegistryRecord>,
    /// The blocks that borrowed strings of the records index, each with the capacity it keeps
    /// alive, so that the capture accounts for each block once.
    blocks: storage::BlockTable,
    /// The number the next listed block takes, which only grows.
    block_sequence: usize,
    usage: HtmlCaptureUsage,
}
/// One immutable bounded producer capture, reusable across context queries.
#[derive(Clone, Debug)]
pub struct HtmlCapture(Arc<CaptureData>);
/// One-shot capture assembler. Unspecified evidence remains unavailable.
#[derive(Debug)]
pub struct HtmlCaptureBuilder {
    data: CaptureData,
    limits: HtmlCaptureLimits,
    controlled: bool,
    account: account::Account,
}

fn reserve<T>(v: &mut Vec<T>, additional: usize) -> Result<(), HtmlCaptureError> {
    v.try_reserve(additional)
        .map_err(|_| HtmlCaptureError::RepresentationExhausted)
}
fn add(a: usize, b: usize) -> Result<usize, HtmlCaptureError> {
    a.checked_add(b)
        .ok_or(HtmlCaptureError::RepresentationExhausted)
}
fn name_units(name: &QualifiedName) -> Result<usize, HtmlCaptureError> {
    let n = add(
        name.local_name().len(),
        name.prefix().map_or(0, DocumentString::len),
    )?;
    add(n, name.namespace().uri().map_or(0, DocumentString::len))
}
fn exact(value: &DocumentString, literal: &str) -> bool {
    value
        .code_units()
        .iter()
        .copied()
        .eq(literal.encode_utf16())
}
fn html(name: &QualifiedName, local: &str) -> bool {
    namespace(name.namespace(), "http://www.w3.org/1999/xhtml") && exact(name.local_name(), local)
}
fn namespace(ns: &Namespace, uri: &str) -> bool {
    ns.uri().is_some_and(|v| exact(v, uri))
}
impl HtmlCaptureBuilder {
    /// Establishes actual A/Area cache history. The observed Document is the
    /// historical parsing Document, which may differ after adoption. No URL,
    /// Blob-store, origin or current-Document facts are inferred from this join.
    /// Checks, admission and interruption complete before any state is written.
    pub fn establish_hyperlink_url_with<E>(
        &mut self,
        subject: &HtmlCaptureBuilderSubject,
        observation: Option<(&HtmlCaptureBuilderSubject, crate::HtmlHyperlinkUrlSnapshot)>,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<(), crate::HtmlHyperlinkCaptureError<E>> {
        use crate::{HtmlCapturedHyperlinkUrl, HtmlHyperlinkCaptureError as Error};
        step().map_err(Error::Interrupted)?;
        let index = self.index(subject)?;
        let HtmlCapturedNodeKind::Element(element) = &self.data.records[index].kind else {
            return Err(HtmlCaptureError::NotHtmlHyperlink.into());
        };
        if !html(element.name(), "a") && !html(element.name(), "area") {
            return Err(HtmlCaptureError::NotHtmlHyperlink.into());
        }
        if element.hyperlink_url.is_some() {
            return Err(HtmlCaptureError::HyperlinkUrlAlreadyEstablished.into());
        }
        let mut href = None;
        for attribute in element.attributes.iter() {
            step().map_err(Error::Interrupted)?;
            if attribute.name().namespace().uri().is_none()
                && exact(attribute.name().local_name(), "href")
            {
                href = Some(attribute.value());
                break;
            }
        }
        let (state, units, relations) = match observation {
            None => {
                if href.is_some() {
                    return Err(HtmlCaptureError::HyperlinkHrefMismatch.into());
                }
                (HtmlCapturedHyperlinkUrl::Null, 0, 0)
            }
            Some((document, snapshot)) => {
                let document = self.index(document)?;
                self.require(document, HtmlCaptureSubjectKind::Document)?;
                let Some(href) = href else {
                    return Err(HtmlCaptureError::HyperlinkHrefMismatch.into());
                };
                if href.len() != snapshot.raw().len() {
                    return Err(HtmlCaptureError::HyperlinkHrefMismatch.into());
                }
                for (a, b) in href.code_units().iter().zip(snapshot.raw().code_units()) {
                    step().map_err(Error::Interrupted)?;
                    if a != b {
                        return Err(HtmlCaptureError::HyperlinkHrefMismatch.into());
                    }
                }
                let units = snapshot.retained_utf16_extent();
                (
                    HtmlCapturedHyperlinkUrl::Observed {
                        document: HtmlCaptureDocumentId(subject_id(&self.data, document)),
                        snapshot,
                    },
                    units,
                    1,
                )
            }
        };
        let charges = self.admit(0, relations, 0, units, 0, 0)?;
        step().map_err(Error::Interrupted)?;
        self.commit(charges)?;
        let element = self.data.records[index].kind.element_mut()?;
        element.hyperlink_url = Some(state);
        Ok(())
    }
    /// Attests actual canonical flags and parser creation relative to this
    /// immutable capture's node document and host revision. No provenance is
    /// inferred from attributes or namespace.
    pub fn establish_link_history(
        &mut self,
        subject: &HtmlCaptureBuilderSubject,
        history: surgeist_html::elements::link::HtmlLinkHistory,
    ) -> Result<(), HtmlCaptureError> {
        let index = self.index(subject)?;
        let HtmlCapturedNodeKind::Element(element) = &self.data.records[index].kind else {
            return Err(HtmlCaptureError::NotHtmlLink);
        };
        if !html(element.name(), "link") {
            return Err(HtmlCaptureError::NotHtmlLink);
        }
        if element.link_history.is_some() {
            return Err(HtmlCaptureError::LinkHistoryAlreadyEstablished);
        }
        let element = self.data.records[index].kind.element_mut()?;
        element.link_history = Some(history);
        Ok(())
    }
    pub fn new(limits: HtmlCaptureLimits, host_revision: HtmlHostFactsRevision) -> Self {
        Self {
            data: CaptureData {
                identity: HtmlCaptureId(Arc::new(())),
                host_revision,
                records: Vec::new(),
                registries: Vec::new(),
                blocks: storage::BlockTable::new(),
                block_sequence: 0,
                usage: HtmlCaptureUsage::default(),
            },
            limits,
            controlled: false,
            account: account::Account::Pending,
        }
    }
    /// A builder that charges below the scope `accounting` names, in the host's operation. The
    /// schema module documentation of `surgeist_html::accounting` lists the scopes it creates. Without accounting a builder charges
    /// an operation of its own, which it creates at its first charge.
    pub fn with_accounting(
        limits: HtmlCaptureLimits,
        host_revision: HtmlHostFactsRevision,
        accounting: &HtmlHostScope,
    ) -> Result<Self, HtmlAccountingFault> {
        let mut builder = Self::new(limits, host_revision);
        builder.account = account::Account::hosted(limits, accounting)
            .map_err(surgeist_html::accounting::fault_of)?;
        Ok(builder)
    }
    pub(crate) fn enable_controlled_storage(&mut self) {
        self.controlled = true;
    }
    pub(crate) fn controlled_data_header_bytes() -> usize {
        std::mem::size_of::<CaptureData>()
    }
    #[cfg(test)]
    pub(crate) fn inspect_controlled_records(&self) -> (usize, usize, usize) {
        (
            self.data.records.len(),
            self.data.records.capacity(),
            std::mem::size_of::<Record>(),
        )
    }
    pub(crate) fn validate_controlled_document(
        &self,
        subject: &HtmlCaptureBuilderSubject,
    ) -> Result<(), HtmlCaptureError> {
        self.require(self.index(subject)?, HtmlCaptureSubjectKind::Document)
    }
    pub fn usage(&self) -> HtmlCaptureUsage {
        self.data.usage
    }
    fn index(&self, handle: &HtmlCaptureBuilderSubject) -> Result<usize, HtmlCaptureError> {
        if handle.capture != self.data.identity {
            return Err(HtmlCaptureError::ForeignBuilder);
        }
        Ok(handle.index)
    }
    fn require(
        &self,
        index: usize,
        expected: HtmlCaptureSubjectKind,
    ) -> Result<(), HtmlCaptureError> {
        let actual = self.data.records[index].kind.subject_kind();
        if actual != expected {
            return Err(HtmlCaptureError::WrongKind { expected, actual });
        }
        Ok(())
    }
    fn registry(&self, value: HtmlCaptureRegistryInput) -> Result<Registry, HtmlCaptureError> {
        Ok(match value {
            HtmlCaptureRegistryInput::Null => Registry::Null,
            HtmlCaptureRegistryInput::Unavailable => Registry::Unavailable,
            HtmlCaptureRegistryInput::Registry(id) => {
                if id.capture != self.data.identity {
                    return Err(HtmlCaptureError::ForeignBuilder);
                }
                Registry::Index(id.index)
            }
        })
    }
    /// Checks the counts an operation adds against the caps and returns them as charges to
    /// commit. Nothing is recorded: a refusal and any later failure leave the usage as it was.
    fn admit(
        &mut self,
        subjects: usize,
        relationships: usize,
        attributes: usize,
        units: usize,
        registries: usize,
        definitions: usize,
    ) -> Result<account::Charges, HtmlCaptureError> {
        let charges = account::Charges {
            subjects,
            relationships,
            attributes,
            units,
            registries,
            definitions,
            ..account::Charges::default()
        };
        self.account.ready(self.limits)?.check(&charges)?;
        Ok(charges)
    }
    /// Checks that a subject at `required` depth fits the cap.
    fn depth(&mut self, required: usize) -> Result<(), HtmlCaptureError> {
        let held = self.data.usage.depth;
        self.account.ready(self.limits)?.check_depth(held, required)
    }
    /// Gives back what was recorded since the usage `to`, and reads the typed usage back.
    pub(crate) fn roll_back_usage(&mut self, to: HtmlCaptureUsage) -> Result<(), HtmlCaptureError> {
        let from = self.data.usage;
        self.data.usage = self.account.ready(self.limits)?.rollback(from, to)?;
        Ok(())
    }
    /// Records the charges an operation checked, and reads the typed usage back.
    fn commit(&mut self, charges: account::Charges) -> Result<(), HtmlCaptureError> {
        let held = self.data.usage.depth;
        self.data.usage = self.account.ready(self.limits)?.commit(&charges, held)?;
        Ok(())
    }
    fn push(
        &mut self,
        kind: HtmlCapturedNodeKind,
        document: usize,
        parent: Parent,
        charges: account::Charges,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        self.push_controlled(kind, document, parent, charges, &mut preflight::ordinary)
            .map_err(preflight::ordinary_error)
    }
    fn push_controlled<E>(
        &mut self,
        kind: HtmlCapturedNodeKind,
        document: usize,
        parent: Parent,
        charges: account::Charges,
        admit: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<HtmlCaptureBuilderSubject, preflight::Failure<E>> {
        preflight::reserve_storage(&mut self.data.records, 1, true, self.controlled, admit)?;
        let listing = storage::list_blocks(
            &mut self.data.blocks,
            &mut self.data.block_sequence,
            |visit| storage::kind_strings(&kind, visit),
        )
        .map_err(|_| preflight::Failure::AllocationFailed)?;
        if let Err(error) = self.commit(charges) {
            listing.undo(&mut self.data.blocks);
            return Err(error.into());
        }
        let index = self.data.records.len();
        let container = matches!(
            kind,
            HtmlCapturedNodeKind::Document(_)
                | HtmlCapturedNodeKind::Element(_)
                | HtmlCapturedNodeKind::Fragment(_)
        );
        self.data.records.push(Record {
            kind,
            document,
            parent,
            children: if container { None } else { Some(Vec::new()) },
            dependents: Vec::new(),
            depth: DepthLink::root(index),
        });
        Ok(HtmlCaptureBuilderSubject {
            capture: self.data.identity.clone(),
            index,
        })
    }
    /// Adds explicit document facts; neither mode nor scripting is guessed.
    pub fn add_document(
        &mut self,
        mode: HtmlDocumentMode,
        scripting: HtmlScriptingStatus,
        registry: HtmlCaptureRegistryInput,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        self.add_document_controlled(mode, scripting, registry, &mut preflight::ordinary)
            .map_err(preflight::ordinary_error)
    }
    pub(crate) fn add_document_controlled<E>(
        &mut self,
        mode: HtmlDocumentMode,
        scripting: HtmlScriptingStatus,
        registry: HtmlCaptureRegistryInput,
        admit: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<HtmlCaptureBuilderSubject, preflight::Failure<E>> {
        preflight::work(admit, 8)?;
        let registry = self.registry(registry)?;
        let charges = self.admit(
            1,
            usize::from(matches!(registry, Registry::Index(_))),
            0,
            0,
            0,
            0,
        )?;
        self.push_controlled(
            HtmlCapturedNodeKind::Document(HtmlCapturedDocument {
                mode,
                scripting,
                image_support: HtmlImageSupport::Unavailable,
                parser_creation_image_support: HtmlImageSupport::Unavailable,
                registry,
                class: HtmlDocumentClass::Unavailable,
                template_role: HtmlTemplateDocumentRole::Unavailable,
                template_cache: Association::Unavailable,
                template_facts_established: false,
                nonce_environment: HtmlNonceDocumentEnvironment::new(
                    HtmlBrowsingContextPresence::Unavailable,
                    HtmlHeaderDeliveredCsp::Unavailable,
                ),
                nonce_environment_established: false,
            }),
            self.data.records.len(),
            Parent::Root,
            charges,
            admit,
        )
    }
    /// Establishes a target-only policy observation associated with this
    /// builder-qualified Document. It does not establish creation-environment
    /// applicability or infer policy from mode or scripting.
    pub fn establish_document_image_support(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        support: HtmlImageSupport,
    ) -> Result<(), HtmlCaptureError> {
        let index = self.index(document)?;
        self.require(index, HtmlCaptureSubjectKind::Document)?;
        let facts = self.data.records[index].kind.document_mut()?;
        if facts.image_support != HtmlImageSupport::Unavailable {
            return Err(HtmlCaptureError::DocumentFactsAlreadyEstablished);
        }
        facts.image_support = support;
        Ok(())
    }
    /// Attests image policy for the parser creation environment associated
    /// with this builder-qualified entry Document, including documents created
    /// within that operation. This is an adapter premise, not copied Document
    /// URL/settings/resource state. Target-only support is an independent fact.
    pub fn establish_parser_creation_image_support(
        &mut self,
        entry_document: &HtmlCaptureBuilderSubject,
        support: HtmlImageSupport,
    ) -> Result<(), HtmlCaptureError> {
        let index = self.index(entry_document)?;
        self.require(index, HtmlCaptureSubjectKind::Document)?;
        let facts = self.data.records[index].kind.document_mut()?;
        if facts.parser_creation_image_support != HtmlImageSupport::Unavailable {
            return Err(HtmlCaptureError::DocumentFactsAlreadyEstablished);
        }
        facts.parser_creation_image_support = support;
        Ok(())
    }
    /// Establishes conditional template-owner facts without deriving them from
    /// mode, scripting or element namespace. A cache must name an actual inert
    /// document already qualified by this builder.
    pub fn establish_document_template_facts(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        class: HtmlDocumentClass,
        role: HtmlTemplateDocumentRole,
        cache: HtmlCaptureTemplateCacheInput,
    ) -> Result<(), HtmlCaptureError> {
        let index = self.index(document)?;
        self.require(index, HtmlCaptureSubjectKind::Document)?;
        let current = self.data.records[index].kind.document()?;
        if current.template_facts_established {
            return Err(HtmlCaptureError::DocumentFactsAlreadyEstablished);
        }
        let cache = match cache {
            HtmlCaptureTemplateCacheInput::Absent => Association::Absent,
            HtmlCaptureTemplateCacheInput::Unavailable => Association::Unavailable,
            HtmlCaptureTemplateCacheInput::Document(owner) => {
                let owner = self.index(&owner)?;
                self.require(owner, HtmlCaptureSubjectKind::Document)?;
                let facts = self.data.records[owner].kind.document()?;
                if role != HtmlTemplateDocumentRole::Ordinary
                    || facts.template_role != HtmlTemplateDocumentRole::InertTemplate
                    || owner == index
                    || (class != HtmlDocumentClass::Unavailable
                        && facts.class != HtmlDocumentClass::Unavailable
                        && class != facts.class)
                {
                    return Err(HtmlCaptureError::InvalidTemplateDocumentCache);
                }
                Association::Index(owner)
            }
        };
        let charges = self.admit(
            0,
            usize::from(matches!(cache, Association::Index(_))),
            0,
            0,
            0,
            0,
        )?;
        self.commit(charges)?;
        let current = self.data.records[index].kind.document_mut()?;
        current.class = class;
        current.template_role = role;
        current.template_cache = cache;
        current.template_facts_established = true;
        Ok(())
    }
    /// Establishes the actual slot once, without inferring it from content.
    pub fn establish_element_nonce(
        &mut self,
        subject: &HtmlCaptureBuilderSubject,
        value: DocumentString,
    ) -> Result<(), HtmlCaptureError> {
        let index = self.index(subject)?;
        self.require(index, HtmlCaptureSubjectKind::Element)?;
        let element = self.data.records[index].kind.element()?;
        if !surgeist_html::bounded::security::is_nonce_element(element.name()) {
            return Err(HtmlCaptureError::NonceInapplicable);
        }
        if !matches!(element.nonce, HtmlCapturedNonce::Unavailable) {
            return Err(HtmlCaptureError::NonceAlreadyEstablished);
        }
        let charges = self.admit(0, 0, 0, value.len(), 0, 0)?;
        let listing = storage::list_blocks(
            &mut self.data.blocks,
            &mut self.data.block_sequence,
            |visit| visit(&value),
        )
        .map_err(|_| HtmlCaptureError::RepresentationExhausted)?;
        if let Err(error) = self.commit(charges) {
            listing.undo(&mut self.data.blocks);
            return Err(error);
        }
        let element = self.data.records[index].kind.element_mut()?;
        element.nonce = HtmlCapturedNonce::Value(value);
        Ok(())
    }
    /// Supplies independently qualified Document environment facts once.
    pub fn establish_document_nonce_environment(
        &mut self,
        subject: &HtmlCaptureBuilderSubject,
        environment: HtmlNonceDocumentEnvironment,
    ) -> Result<(), HtmlCaptureError> {
        let index = self.index(subject)?;
        self.require(index, HtmlCaptureSubjectKind::Document)?;
        let document = self.data.records[index].kind.document_mut()?;
        if document.nonce_environment_established {
            return Err(HtmlCaptureError::NonceAlreadyEstablished);
        }
        document.nonce_environment = environment;
        document.nonce_environment_established = true;
        Ok(())
    }
    /// Adds exact element values; unspecified topology and associations stay unavailable.
    pub fn add_element(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        name: QualifiedName,
        attributes: OrderedAttributes,
        is_value: HtmlCapturedIsValue,
        registry: HtmlCaptureRegistryInput,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        self.add_element_controlled(
            document,
            name,
            attributes,
            is_value,
            registry,
            &mut preflight::ordinary,
        )
        .map_err(preflight::ordinary_error)
    }
    pub(crate) fn add_element_controlled<E>(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        name: QualifiedName,
        attributes: OrderedAttributes,
        is_value: HtmlCapturedIsValue,
        registry: HtmlCaptureRegistryInput,
        admit: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<HtmlCaptureBuilderSubject, preflight::Failure<E>> {
        preflight::work(admit, 8)?;
        let document = self.index(document)?;
        self.require(document, HtmlCaptureSubjectKind::Document)?;
        let registry = self.registry(registry)?;
        self.admit(
            1,
            1 + usize::from(matches!(registry, Registry::Index(_))),
            attributes.len(),
            0,
            0,
            0,
        )?;
        preflight::work(admit, 5)?;
        let mut units = name_units(&name)?;
        for a in attributes.iter() {
            preflight::work(admit, 7)?;
            units = add(units, add(name_units(a.name())?, a.value().len())?)?;
        }
        if let HtmlCapturedIsValue::Value(v) = &is_value {
            units = add(units, v.len())?;
        }
        let mut charges = self.admit(
            1,
            1 + usize::from(matches!(registry, Registry::Index(_))),
            attributes.len(),
            units,
            0,
            0,
        )?;
        let saved_is = match &is_value {
            HtmlCapturedIsValue::Value(value) => Some(value),
            HtmlCapturedIsValue::Absent | HtmlCapturedIsValue::Unavailable => None,
        };
        let retained = storage::option_value_element(&name, &attributes, saved_is, &mut || {
            preflight::work(admit, 1)
        })
        .map_err(|error| match error {
            HtmlCaptureStorageError::RepresentationExhausted => {
                preflight::Failure::Capture(HtmlCaptureError::RepresentationExhausted)
            }
            HtmlCaptureStorageError::Interrupted(error) => error,
        })?;
        charges.element_value_bytes = retained.retained_bytes();
        self.admit_element_value_bytes(charges.element_value_bytes)?;
        preflight::work(admit, add(name_units(&name)?, 16)?)?;
        let template = if html(&name, "template") {
            TemplateAssociation::Unavailable
        } else {
            TemplateAssociation::Absent
        };
        self.push_controlled(
            HtmlCapturedNodeKind::Element(HtmlCapturedElement {
                name,
                attributes,
                is_value,
                registry,
                template,
                shadow: Association::Unavailable,
                custom_context: None,
                nonce: HtmlCapturedNonce::Unavailable,
                link_history: None,
                hyperlink_url: None,
            }),
            document,
            Parent::Unavailable,
            charges,
            admit,
        )
    }
    /// Adds a plain fragment with explicit document and an unavailable host.
    pub fn add_fragment(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        let document = self.index(document)?;
        self.require(document, HtmlCaptureSubjectKind::Document)?;
        let charges = self.admit(1, 1, 0, 0, 0, 0)?;
        self.push(
            HtmlCapturedNodeKind::Fragment(HtmlCapturedFragment {
                kind: HtmlCapturedFragmentKind::Plain,
                host: Association::Unavailable,
                registry: Registry::Null,
            }),
            document,
            Parent::Root,
            charges,
        )
    }
    fn add_leaf(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        kind: HtmlCapturedNodeKind,
        units: usize,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        self.add_leaf_controlled(document, kind, units, &mut preflight::ordinary)
            .map_err(preflight::ordinary_error)
    }
    pub(crate) fn add_leaf_controlled<E>(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        kind: HtmlCapturedNodeKind,
        units: usize,
        admit: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<HtmlCaptureBuilderSubject, preflight::Failure<E>> {
        preflight::work(admit, 7)?;
        let document = self.index(document)?;
        self.require(document, HtmlCaptureSubjectKind::Document)?;
        let charges = self.admit(1, 1, 0, units, 0, 0)?;
        self.push_controlled(kind, document, Parent::Unavailable, charges, admit)
    }
    pub fn add_text(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        text: DocumentString,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        let units = text.len();
        self.add_leaf(document, HtmlCapturedNodeKind::Text(text), units)
    }
    pub fn add_comment(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        text: DocumentString,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        let units = text.len();
        self.add_leaf(document, HtmlCapturedNodeKind::Comment(text), units)
    }
    pub fn add_processing_instruction(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        target: DocumentString,
        data: DocumentString,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        let units = add(target.len(), data.len())?;
        self.add_leaf(
            document,
            HtmlCapturedNodeKind::ProcessingInstruction(HtmlCapturedProcessingInstruction {
                target,
                data,
            }),
            units,
        )
    }
    pub fn add_doctype(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        doctype: HtmlDoctype,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        let units = add(
            add(
                doctype.name().len(),
                doctype.public_id().map_or(0, DocumentString::len),
            )?,
            doctype.system_id().map_or(0, DocumentString::len),
        )?;
        self.add_leaf(document, HtmlCapturedNodeKind::Doctype(doctype), units)
    }
    /// Captures a complete finite definition set, or explicitly unavailable metadata.
    pub fn add_registry(
        &mut self,
        definitions: Option<Vec<HtmlCustomElementDefinition>>,
    ) -> Result<HtmlCaptureBuilderRegistry, HtmlCaptureError> {
        self.admit(0, 0, 0, 0, 1, definitions.as_ref().map_or(0, Vec::len))?;
        let mut units = 0;
        let mut names = HashSet::new();
        if let Some(definitions) = &definitions {
            names
                .try_reserve(definitions.len())
                .map_err(|_| HtmlCaptureError::RepresentationExhausted)?;
            for d in definitions {
                if validate_custom_element_name(&d.name).is_err() || d.local_name.is_empty() {
                    return Err(HtmlCaptureError::InvalidDefinitionName);
                }
                if !names.insert(&d.name) {
                    return Err(HtmlCaptureError::DuplicateDefinitionName);
                }
                units = add(units, add(d.name.len(), d.local_name.len())?)?;
            }
        }
        let charges = self.admit(0, 0, 0, units, 1, definitions.as_ref().map_or(0, Vec::len))?;
        reserve(&mut self.data.registries, 1)?;
        let definition_strings = |visit: &mut dyn FnMut(&DocumentString)| {
            for definition in definitions.iter().flatten() {
                visit(&definition.name);
                visit(&definition.local_name);
            }
        };
        let listing = storage::list_blocks(
            &mut self.data.blocks,
            &mut self.data.block_sequence,
            definition_strings,
        )
        .map_err(|_| HtmlCaptureError::RepresentationExhausted)?;
        if let Err(error) = self.commit(charges) {
            listing.undo(&mut self.data.blocks);
            return Err(error);
        }
        let index = self.data.registries.len();
        self.data.registries.push(RegistryRecord {
            complete: definitions.is_some(),
            definitions: definitions.unwrap_or_default(),
            scope: HtmlRegistryScope::Unavailable,
            scope_established: false,
        });
        Ok(HtmlCaptureBuilderRegistry {
            capture: self.data.identity.clone(),
            index,
        })
    }
    /// Supplies actual registry scope without changing definition availability.
    pub fn establish_registry_scope(
        &mut self,
        registry: &HtmlCaptureBuilderRegistry,
        scope: HtmlRegistryScope,
    ) -> Result<(), HtmlCaptureError> {
        if registry.capture != self.data.identity {
            return Err(HtmlCaptureError::ForeignBuilder);
        }
        let record = &mut self.data.registries[registry.index];
        if record.scope_established {
            return Err(HtmlCaptureError::RegistryScopeAlreadyEstablished);
        }
        record.scope = scope;
        record.scope_established = true;
        Ok(())
    }
    /// Obtains a handle to an actual retained definition, even in a partial set.
    pub fn definition_handle(
        &self,
        registry: &HtmlCaptureBuilderRegistry,
        index: usize,
    ) -> Result<HtmlCaptureBuilderDefinition, HtmlCaptureError> {
        if registry.capture != self.data.identity {
            return Err(HtmlCaptureError::ForeignBuilder);
        }
        if index >= self.data.registries[registry.index].definitions.len() {
            return Err(HtmlCaptureError::UnknownDefinition);
        }
        Ok(HtmlCaptureBuilderDefinition {
            capture: self.data.identity.clone(),
            registry_index: registry.index,
            record_index: index,
        })
    }
    /// Retains one independently known record in an incomplete registry. This
    /// does not complete the set or execute `customElements.define`.
    pub fn add_definition(
        &mut self,
        registry: &HtmlCaptureBuilderRegistry,
        definition: HtmlCustomElementDefinition,
    ) -> Result<HtmlCaptureBuilderDefinition, HtmlCaptureError> {
        if registry.capture != self.data.identity {
            return Err(HtmlCaptureError::ForeignBuilder);
        }
        let records = &self.data.registries[registry.index];
        if records.complete {
            return Err(HtmlCaptureError::CompleteRegistry);
        }
        if validate_custom_element_name(&definition.name).is_err()
            || definition.local_name.is_empty()
        {
            return Err(HtmlCaptureError::InvalidDefinitionName);
        }
        if records
            .definitions
            .iter()
            .any(|d| d.name == definition.name)
        {
            return Err(HtmlCaptureError::DuplicateDefinitionName);
        }
        let charges = self.admit(
            0,
            0,
            0,
            add(definition.name.len(), definition.local_name.len())?,
            0,
            1,
        )?;
        reserve(&mut self.data.registries[registry.index].definitions, 1)?;
        let definition_strings = |visit: &mut dyn FnMut(&DocumentString)| {
            visit(&definition.name);
            visit(&definition.local_name);
        };
        let listing = storage::list_blocks(
            &mut self.data.blocks,
            &mut self.data.block_sequence,
            definition_strings,
        )
        .map_err(|_| HtmlCaptureError::RepresentationExhausted)?;
        if let Err(error) = self.commit(charges) {
            listing.undo(&mut self.data.blocks);
            return Err(error);
        }
        let records = &mut self.data.registries[registry.index].definitions;
        let record_index = records.len();
        records.push(definition);
        Ok(HtmlCaptureBuilderDefinition {
            capture: self.data.identity.clone(),
            registry_index: registry.index,
            record_index,
        })
    }
    /// Supplies actual state and definition slots once. Saved runtime `is` is
    /// never inferred from current content. Custom associations check the HTML
    /// namespace/local join and customized saved-name join; autonomous records
    /// may retain a saved value after source-defined creation and upgrade.
    /// Association keys may belong to a different current lookup registry after
    /// adoption. Other states retain transient/null/non-null facts independently.
    pub fn establish_custom_element_context(
        &mut self,
        subject: &HtmlCaptureBuilderSubject,
        state: HtmlCapturedCustomElementState,
        definition: HtmlCaptureDefinitionInput,
    ) -> Result<(), HtmlCaptureError> {
        let index = self.index(subject)?;
        self.require(index, HtmlCaptureSubjectKind::Element)?;
        let element = self.data.records[index].kind.element()?;
        if element.custom_context.is_some() {
            return Err(HtmlCaptureError::CustomElementContextAlreadyEstablished);
        }
        let definition = match definition {
            HtmlCaptureDefinitionInput::Null => HtmlCapturedCustomElementDefinition::Null,
            HtmlCaptureDefinitionInput::Unavailable => {
                HtmlCapturedCustomElementDefinition::Unavailable
            }
            HtmlCaptureDefinitionInput::Definition(handle) => {
                if handle.capture != self.data.identity {
                    return Err(HtmlCaptureError::ForeignBuilder);
                }
                let d = self
                    .data
                    .registries
                    .get(handle.registry_index)
                    .and_then(|r| r.definitions.get(handle.record_index))
                    .ok_or(HtmlCaptureError::UnknownDefinition)?;
                if state == HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom) {
                    if !namespace(element.name.namespace(), "http://www.w3.org/1999/xhtml") {
                        return Err(HtmlCaptureError::CustomElementDefinitionNamespaceMismatch);
                    }
                    if element.name.local_name() != &d.local_name {
                        return Err(HtmlCaptureError::CustomElementDefinitionNameMismatch);
                    }
                    if d.name != d.local_name {
                        match &element.is_value {
                            HtmlCapturedIsValue::Unavailable => {}
                            HtmlCapturedIsValue::Value(value) if value == &d.name => {}
                            _ => return Err(HtmlCaptureError::CustomElementSavedIsMismatch),
                        }
                    }
                }
                HtmlCapturedCustomElementDefinition::Definition(HtmlCaptureDefinitionId {
                    registry: HtmlCaptureRegistryId {
                        capture: self.data.identity.clone(),
                        index: handle.registry_index,
                    },
                    record_index: handle.record_index,
                })
            }
        };
        if state == HtmlCapturedCustomElementState::Known(HtmlCustomElementState::Custom)
            && definition == HtmlCapturedCustomElementDefinition::Null
        {
            return Err(HtmlCaptureError::CustomElementDefinitionRequired);
        }
        let charges = self.admit(
            0,
            usize::from(matches!(
                definition,
                HtmlCapturedCustomElementDefinition::Definition(_)
            )),
            0,
            0,
            0,
            0,
        )?;
        self.commit(charges)?;
        let element = self.data.records[index].kind.element_mut()?;
        element.custom_context = Some(CustomElementContext { state, definition });
        Ok(())
    }
    pub fn establish_root(
        &mut self,
        child: &HtmlCaptureBuilderSubject,
    ) -> Result<(), HtmlCaptureError> {
        let i = self.index(child)?;
        if self.data.records[i].parent != Parent::Unavailable {
            return Err(HtmlCaptureError::ParentAlreadyEstablished);
        }
        self.data.records[i].parent = Parent::Root;
        Ok(())
    }
    fn ordinary_edge(&self, child: usize, parent: usize) -> Result<(), HtmlCaptureError> {
        let c = &self.data.records[child];
        let p = &self.data.records[parent];
        if !p.is_container() {
            return Err(HtmlCaptureError::CannotHaveChildren(p.kind.subject_kind()));
        }
        if matches!(
            c.kind,
            HtmlCapturedNodeKind::Document(_) | HtmlCapturedNodeKind::Fragment(_)
        ) {
            return Err(HtmlCaptureError::CannotBeOrdinaryChild(
                c.kind.subject_kind(),
            ));
        }
        if c.document != p.document {
            return Err(HtmlCaptureError::DocumentMismatch);
        }
        if let Some(children) = &p.children
            && !children.contains(&child)
        {
            return Err(HtmlCaptureError::ParentChildConflict);
        }
        Ok(())
    }
    /// Admits one owning edge from `child`, a root, to `parent`.
    fn edge_depths_controlled<E>(
        &mut self,
        child: usize,
        parent: usize,
        admit: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<Option<DepthPlan>, preflight::Failure<E>> {
        preflight::work(admit, 8)?;
        if child == parent {
            return Err(preflight::Failure::Capture(HtmlCaptureError::Cycle));
        }
        self.plan_depths(&[(child, parent)], parent, admit)
    }
    /// Finds the root of `start`'s component and `start`'s depth below it,
    /// halving the path walked. Each step is charged, so the work is the
    /// records actually visited; amortized it is logarithmic in the records.
    fn find_root<E>(
        &mut self,
        start: usize,
        admit: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<(usize, usize), preflight::Failure<E>> {
        let mut at = start;
        let mut distance = 0;
        loop {
            preflight::work(admit, 6)?;
            let link = self.data.records[at].depth.link;
            if link == at {
                return Ok((at, distance));
            }
            let beyond = self.data.records[link].depth.link;
            if beyond == link {
                return Ok((link, add(distance, self.data.records[at].depth.offset)?));
            }
            let offset = add(
                self.data.records[at].depth.offset,
                self.data.records[link].depth.offset,
            )?;
            self.data.records[at].depth.link = beyond;
            self.data.records[at].depth.offset = offset;
            distance = add(distance, offset)?;
            at = beyond;
        }
    }
    /// Admits edges from each of `edges`' children, all roots, to `parent`.
    /// The new edges close a cycle exactly when `parent` is below one of the
    /// children, that is, when the root of `parent`'s component is a child. They
    /// exceed the depth limit exactly when the deepest child's subtree, placed
    /// below `parent`, does. Both are decided from the roots and heights alone;
    /// only a refusal walks the attached subtrees, to report the record reached
    /// first.
    fn plan_depths<E>(
        &mut self,
        edges: &[(usize, usize)],
        parent: usize,
        admit: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<Option<DepthPlan>, preflight::Failure<E>> {
        if edges.is_empty() {
            return Ok(None);
        }
        let (root, below) = self.find_root(parent, admit)?;
        let first = add(below, 1)?;
        preflight::work(admit, edges.len())?;
        let mut cycle = false;
        let mut tallest = 0;
        for &(child, _) in edges {
            cycle |= child == root;
            tallest = tallest.max(self.data.records[child].depth.height);
        }
        let height = add(first, tallest)?;
        if cycle || self.depth(height).is_err() {
            return Err(self.first_refusal(edges, parent, first, cycle, height, admit));
        }
        Ok(Some(DepthPlan {
            root,
            first,
            height: height.max(self.data.records[root].depth.height),
        }))
    }
    /// The failure the breadth-first revision of the attached subtrees reaches
    /// first, for edges already known to be refused. At each record, reaching the
    /// parent is a cycle and is reported before that record's depth is checked
    /// against the limit; the new edge itself is checked before any traversal.
    /// Work is proportional to the records visited.
    fn first_refusal<E>(
        &mut self,
        edges: &[(usize, usize)],
        parent: usize,
        first: usize,
        cycle: bool,
        height: usize,
        admit: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> preflight::Failure<E> {
        match self.refusal_walk(edges, parent, first, admit) {
            Err(failure) => failure,
            // The walk and the roots and heights decide the same refusal.
            Ok(()) if cycle => preflight::Failure::Capture(HtmlCaptureError::Cycle),
            Ok(()) => preflight::Failure::Capture(match self.depth(height) {
                Err(error) => error,
                // The walk found no refusal where the check that sent it here found one.
                Ok(()) => HtmlCaptureError::Accounting(HtmlAccountingFault::UnexpectedChange),
            }),
        }
    }
    fn refusal_walk<E>(
        &mut self,
        edges: &[(usize, usize)],
        parent: usize,
        first: usize,
        admit: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<(), preflight::Failure<E>> {
        preflight::work(admit, 8)?;
        self.depth(first)?;
        let mut updates = Vec::new();
        preflight::reserve(&mut updates, edges.len(), admit)?;
        for &(child, _) in edges {
            preflight::work(admit, 3)?;
            updates.push((child, first));
        }
        self.propagate_depths(&mut updates, Some(parent), admit)
    }
    /// Depths of every record below the seeded ones, in breadth-first order. At
    /// each record, reaching `forbidden` is a cycle and is reported before that
    /// record's depth is checked against the limit. Work is proportional to the
    /// records visited, never to the whole forest. Used only to name a refusal.
    fn propagate_depths<E>(
        &mut self,
        updates: &mut Vec<(usize, usize)>,
        forbidden: Option<usize>,
        admit: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<(), preflight::Failure<E>> {
        let mut cursor = 0;
        while cursor < updates.len() {
            preflight::work(admit, 4)?;
            let (i, depth) = updates[cursor];
            if forbidden == Some(i) {
                return Err(preflight::Failure::Capture(HtmlCaptureError::Cycle));
            }
            self.depth(depth)?;
            preflight::reserve(updates, self.data.records[i].dependents.len(), admit)?;
            preflight::work(
                admit,
                self.data.records[i]
                    .dependents
                    .len()
                    .checked_mul(2)
                    .ok_or(HtmlCaptureError::RepresentationExhausted)?,
            )?;
            for &dependent in &self.data.records[i].dependents {
                updates.push((dependent, add(depth, 1)?));
            }
            cursor += 1;
        }
        Ok(())
    }
    /// Links each child's component below the root the plan names.
    fn apply_depths(&mut self, plan: Option<DepthPlan>, edges: &[(usize, usize)]) {
        let Some(plan) = plan else { return };
        for &(child, _) in edges {
            let depth = &mut self.data.records[child].depth;
            depth.link = plan.root;
            depth.offset = plan.first;
        }
        self.data.records[plan.root].depth.height = plan.height;
    }
    /// Establishes an ordinary parent without claiming complete ordered children.
    pub fn establish_parent(
        &mut self,
        child: &HtmlCaptureBuilderSubject,
        parent: &HtmlCaptureBuilderSubject,
    ) -> Result<(), HtmlCaptureError> {
        self.establish_parent_controlled(child, parent, &mut preflight::ordinary)
            .map_err(preflight::ordinary_error)
    }
    /// As [`Self::establish_parent`], taking one `step` for each unit of work
    /// the builder performs, including finding the root of `parent`'s
    /// component. The first refusal of `step` is returned unchanged and
    /// establishes no edge; it can only have shortened depth links, which
    /// leave every fact the capture reports as it was.
    pub fn establish_parent_with<E>(
        &mut self,
        child: &HtmlCaptureBuilderSubject,
        parent: &HtmlCaptureBuilderSubject,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<(), HtmlCaptureEdgeError<E>> {
        self.establish_parent_controlled(child, parent, &mut preflight::steps(&mut step))
            .map_err(preflight::edge_error)
    }
    fn establish_parent_controlled<E>(
        &mut self,
        child: &HtmlCaptureBuilderSubject,
        parent: &HtmlCaptureBuilderSubject,
        admit: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<(), preflight::Failure<E>> {
        let child = self.index(child)?;
        let parent = self.index(parent)?;
        if self.data.records[child].parent != Parent::Unavailable {
            return Err(preflight::Failure::Capture(
                HtmlCaptureError::ParentAlreadyEstablished,
            ));
        }
        self.ordinary_edge(child, parent)?;
        let mut charges = self.admit(0, 2, 0, 0, 0, 0)?;
        let plan = self.edge_depths_controlled(child, parent, admit)?;
        reserve(&mut self.data.records[parent].dependents, 1)?;
        charges.depth = plan.as_ref().map(|plan| plan.height);
        self.commit(charges)?;
        self.data.records[child].parent = Parent::Index(parent);
        self.data.records[parent].dependents.push(child);
        self.apply_depths(plan, &[(child, parent)]);
        Ok(())
    }
    /// Establishes all ordered children atomically, also resolving unknown parents.
    pub fn establish_children(
        &mut self,
        parent: &HtmlCaptureBuilderSubject,
        children: &[HtmlCaptureBuilderSubject],
    ) -> Result<(), HtmlCaptureError> {
        self.establish_children_controlled(parent, children, &mut preflight::ordinary)
            .map_err(preflight::ordinary_error)
    }
    /// As [`Self::establish_children`], taking one `step` for each unit of work
    /// the builder performs. The first refusal of `step` is returned
    /// unchanged and establishes nothing.
    pub fn establish_children_with<E>(
        &mut self,
        parent: &HtmlCaptureBuilderSubject,
        children: &[HtmlCaptureBuilderSubject],
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<(), HtmlCaptureEdgeError<E>> {
        self.establish_children_controlled(parent, children, &mut preflight::steps(&mut step))
            .map_err(preflight::edge_error)
    }
    pub(crate) fn establish_children_controlled<E>(
        &mut self,
        parent: &HtmlCaptureBuilderSubject,
        children: &[HtmlCaptureBuilderSubject],
        admit: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<(), preflight::Failure<E>> {
        preflight::work(admit, 8)?;
        let parent = self.index(parent)?;
        if !self.data.records[parent].is_container() {
            return Err(preflight::Failure::Capture(
                HtmlCaptureError::CannotHaveChildren(self.data.records[parent].kind.subject_kind()),
            ));
        }
        if self.data.records[parent].children.is_some() {
            return Err(preflight::Failure::Capture(
                HtmlCaptureError::ChildrenAlreadyEstablished,
            ));
        }
        self.admit(0, children.len(), 0, 0, 0, 0)?;
        let mut indices = Vec::new();
        preflight::reserve_storage(&mut indices, children.len(), false, self.controlled, admit)?;
        let mut edges = Vec::new();
        preflight::reserve_storage(&mut edges, children.len(), false, self.controlled, admit)?;
        let mut seen = (!self.controlled).then(HashSet::new);
        admit(preflight::Action::Reserve {
            entries: children.len(),
            record_bytes: std::mem::size_of::<usize>(),
        })
        .map_err(preflight::Failure::Admission)?;
        if let Some(seen) = &mut seen {
            seen.try_reserve(children.len())
                .map_err(|_| preflight::Failure::AllocationFailed)?;
        }
        preflight::work(admit, 0)?;
        // Duplicate detection probes the hash set per child; only the controlled
        // path without one compares each child with every earlier child.
        preflight::work(
            admit,
            if seen.is_some() {
                children.len()
            } else {
                children
                    .len()
                    .checked_mul(add(children.len(), 6)?)
                    .ok_or(HtmlCaptureError::RepresentationExhausted)?
            },
        )?;
        for child in children {
            preflight::work(admit, 6)?;
            let child = self.index(child)?;
            let duplicate = match &mut seen {
                Some(seen) => !seen.insert(child),
                None => indices.contains(&child),
            };
            if duplicate {
                return Err(preflight::Failure::Capture(
                    HtmlCaptureError::DuplicateChild,
                ));
            }
            self.ordinary_edge(child, parent)?;
            match self.data.records[child].parent {
                Parent::Unavailable => edges.push((child, parent)),
                Parent::Index(p) if p == parent => {}
                _ => {
                    return Err(preflight::Failure::Capture(
                        HtmlCaptureError::ParentChildConflict,
                    ));
                }
            }
            indices.push(child);
        }
        preflight::work(
            admit,
            self.data.records[parent]
                .dependents
                .len()
                .checked_mul(if seen.is_some() {
                    2
                } else {
                    add(children.len(), 1)?
                })
                .ok_or(HtmlCaptureError::RepresentationExhausted)?,
        )?;
        if self.data.records[parent].dependents.iter().any(|i| {
            matches!(self.data.records[*i].parent, Parent::Index(p) if p == parent)
                && !match &seen {
                    Some(seen) => seen.contains(i),
                    None => indices.contains(i),
                }
        }) {
            return Err(preflight::Failure::Capture(
                HtmlCaptureError::ParentChildConflict,
            ));
        }
        let relationships = add(
            indices.len(),
            edges
                .len()
                .checked_mul(2)
                .ok_or(HtmlCaptureError::RepresentationExhausted)?,
        )?;
        let mut charges = self.admit(0, relationships, 0, 0, 0, 0)?;
        // Only the roots attached here are linked: every other record keeps the
        // depth its own edge established. A cycle needs the parent below one of
        // the attached children, which the roots reveal.
        if !edges.is_empty() {
            preflight::work(admit, 8)?;
        }
        let plan = self.plan_depths(&edges, parent, admit)?;
        preflight::work(
            admit,
            edges
                .len()
                .checked_mul(2)
                .ok_or(HtmlCaptureError::RepresentationExhausted)?,
        )?;
        preflight::reserve_storage(
            &mut self.data.records[parent].dependents,
            edges.len(),
            true,
            self.controlled,
            admit,
        )?;
        admit(preflight::Action::StorageTransfer {
            bytes: indices.capacity() * std::mem::size_of::<usize>(),
        })
        .map_err(preflight::Failure::Admission)?;
        charges.depth = plan.as_ref().map(|plan| plan.height);
        self.commit(charges)?;
        for &(child, _) in &edges {
            self.data.records[child].parent = Parent::Index(parent);
            self.data.records[parent].dependents.push(child);
        }
        self.data.records[parent].children = Some(indices);
        self.apply_depths(plan, &edges);
        Ok(())
    }
    pub fn establish_unhosted(
        &mut self,
        fragment: &HtmlCaptureBuilderSubject,
    ) -> Result<(), HtmlCaptureError> {
        let i = self.index(fragment)?;
        self.require(i, HtmlCaptureSubjectKind::Fragment)?;
        let f = self.data.records[i].kind.fragment_mut()?;
        if f.host != Association::Unavailable {
            return Err(HtmlCaptureError::FragmentAlreadyHosted);
        }
        if matches!(f.kind, HtmlCapturedFragmentKind::ShadowRoot(_)) {
            return Err(HtmlCaptureError::ShadowRootCannotBeUnhosted);
        }
        f.host = Association::Absent;
        Ok(())
    }
    pub fn establish_no_shadow_root(
        &mut self,
        element: &HtmlCaptureBuilderSubject,
    ) -> Result<(), HtmlCaptureError> {
        let i = self.index(element)?;
        self.require(i, HtmlCaptureSubjectKind::Element)?;
        let e = self.data.records[i].kind.element_mut()?;
        if e.shadow != Association::Unavailable {
            return Err(HtmlCaptureError::AssociationAlreadyEstablished);
        }
        e.shadow = Association::Absent;
        Ok(())
    }
    /// Establishes separate template contents. Its document may differ from its host's.
    pub fn establish_template_contents(
        &mut self,
        template: &HtmlCaptureBuilderSubject,
        fragment: &HtmlCaptureBuilderSubject,
    ) -> Result<(), HtmlCaptureError> {
        self.establish_template_contents_controlled(template, fragment, &mut preflight::ordinary)
            .map_err(preflight::ordinary_error)
    }
    /// As [`Self::establish_template_contents`], taking one `step` for each
    /// unit of work the builder performs. The first refusal of `step` is
    /// returned unchanged and establishes nothing.
    pub fn establish_template_contents_with<E>(
        &mut self,
        template: &HtmlCaptureBuilderSubject,
        fragment: &HtmlCaptureBuilderSubject,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<(), HtmlCaptureEdgeError<E>> {
        self.establish_template_contents_controlled(
            template,
            fragment,
            &mut preflight::steps(&mut step),
        )
        .map_err(preflight::edge_error)
    }
    fn establish_template_contents_controlled<E>(
        &mut self,
        template: &HtmlCaptureBuilderSubject,
        fragment: &HtmlCaptureBuilderSubject,
        admit: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<(), preflight::Failure<E>> {
        preflight::work(admit, 8)?;
        let t = self.index(template)?;
        let f = self.index(fragment)?;
        self.require(t, HtmlCaptureSubjectKind::Element)?;
        self.require(f, HtmlCaptureSubjectKind::Fragment)?;
        let e = self.data.records[t].kind.element()?;
        preflight::work(admit, name_units(&e.name)?)?;
        if !html(&e.name, "template") {
            return Err(preflight::Failure::Capture(
                HtmlCaptureError::NotHtmlTemplate,
            ));
        }
        if e.template != TemplateAssociation::Unavailable {
            return Err(preflight::Failure::Capture(
                HtmlCaptureError::AssociationAlreadyEstablished,
            ));
        }
        let fragment = self.data.records[f].kind.fragment()?;
        if fragment.host != Association::Unavailable {
            return Err(preflight::Failure::Capture(
                HtmlCaptureError::FragmentAlreadyHosted,
            ));
        }
        if matches!(fragment.kind, HtmlCapturedFragmentKind::ShadowRoot(_)) {
            return Err(preflight::Failure::Capture(
                HtmlCaptureError::InvalidTemplateContentsReference,
            ));
        }
        let mut charges = self.admit(0, 3, 0, 0, 0, 0)?;
        let plan = self.edge_depths_controlled(f, t, admit)?;
        reserve(&mut self.data.records[t].dependents, 1)?;
        charges.depth = plan.as_ref().map(|plan| plan.height);
        self.commit(charges)?;
        let e = self.data.records[t].kind.element_mut()?;
        e.template = TemplateAssociation::Owned(f);
        let frag = self.data.records[f].kind.fragment_mut()?;
        frag.host = Association::Index(t);
        self.data.records[t].dependents.push(f);
        self.apply_depths(plan, &[(f, t)]);
        Ok(())
    }
    /// Records a non-owning template reference to existing shadow contents.
    /// The target's host, ordinary parent, depth and owning dependents are unchanged.
    /// The reference is independent of the template's ordinary position: HTML
    /// never inserts a template whose contents became the declarative host's
    /// root, and the DOM lets a retained template be inserted under any element,
    /// so its parent may be the host, another element or nothing.
    pub fn establish_template_shadow_reference(
        &mut self,
        template: &HtmlCaptureBuilderSubject,
        contents: &HtmlCaptureBuilderSubject,
    ) -> Result<(), HtmlCaptureError> {
        let t = self.index(template)?;
        let f = self.index(contents)?;
        self.require(t, HtmlCaptureSubjectKind::Element)?;
        self.require(f, HtmlCaptureSubjectKind::Fragment)?;
        let element = self.data.records[t].kind.element()?;
        if !html(&element.name, "template") {
            return Err(HtmlCaptureError::NotHtmlTemplate);
        }
        if element.template != TemplateAssociation::Unavailable {
            return Err(HtmlCaptureError::AssociationAlreadyEstablished);
        }
        let fragment = self.data.records[f].kind.fragment()?;
        if !matches!(fragment.kind, HtmlCapturedFragmentKind::ShadowRoot(_)) {
            return Err(HtmlCaptureError::InvalidTemplateContentsReference);
        }
        let charges = self.admit(0, 1, 0, 0, 0, 0)?;
        self.commit(charges)?;
        let element = self.data.records[t].kind.element_mut()?;
        element.template = TemplateAssociation::ShadowReference(f);
        Ok(())
    }
    /// Adds shadow contents with an unavailable external host, without attaching a host.
    pub fn add_shadow_fragment(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        properties: HtmlCapturedShadowProperties,
        registry: HtmlCaptureRegistryInput,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        let document = self.index(document)?;
        self.require(document, HtmlCaptureSubjectKind::Document)?;
        let registry = self.registry(registry)?;
        let charges = self.admit(
            1,
            1 + usize::from(matches!(registry, Registry::Index(_))),
            0,
            0,
            0,
            0,
        )?;
        self.push(
            HtmlCapturedNodeKind::Fragment(HtmlCapturedFragment {
                kind: HtmlCapturedFragmentKind::ShadowRoot(properties),
                host: Association::Unavailable,
                registry,
            }),
            document,
            Parent::Root,
            charges,
        )
    }
    /// Captures an existing shadow root and all its explicit source metadata.
    /// This records producer facts; it does not attach anything to a live host.
    /// The root's actual document is independent of its retained host: internal
    /// DOM replacement/adoption can change it without changing that association.
    pub fn add_shadow_root(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        host: &HtmlCaptureBuilderSubject,
        properties: HtmlCapturedShadowProperties,
        registry: HtmlCaptureRegistryInput,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        let document = self.index(document)?;
        self.require(document, HtmlCaptureSubjectKind::Document)?;
        let h = self.index(host)?;
        self.require(h, HtmlCaptureSubjectKind::Element)?;
        let e = self.data.records[h].kind.element()?;
        if e.shadow != Association::Unavailable {
            return Err(HtmlCaptureError::AssociationAlreadyEstablished);
        }
        let registry = self.registry(registry)?;
        let mut charges = self.admit(
            1,
            4 + usize::from(matches!(registry, Registry::Index(_))),
            0,
            0,
            0,
            0,
        )?;
        let (root, below) = self
            .find_root(h, &mut preflight::ordinary)
            .map_err(preflight::ordinary_error)?;
        let depth = add(below, 1)?;
        self.depth(depth)?;
        reserve(&mut self.data.records[h].dependents, 1)?;
        reserve(&mut self.data.records, 1)?;
        charges.depth = Some(depth);
        self.commit(charges)?;
        let i = self.data.records.len();
        self.data.records.push(Record {
            kind: HtmlCapturedNodeKind::Fragment(HtmlCapturedFragment {
                kind: HtmlCapturedFragmentKind::ShadowRoot(properties),
                host: Association::Index(h),
                registry,
            }),
            document,
            parent: Parent::Root,
            children: None,
            dependents: Vec::new(),
            depth: DepthLink {
                link: root,
                offset: depth,
                height: 0,
            },
        });
        let top = &mut self.data.records[root].depth.height;
        *top = (*top).max(depth);
        let e = self.data.records[h].kind.element_mut()?;
        e.shadow = Association::Index(i);
        self.data.records[h].dependents.push(i);
        Ok(HtmlCaptureBuilderSubject {
            capture: self.data.identity.clone(),
            index: i,
        })
    }
    /// Publishes validated immutable records; unavailable facts remain unavailable.
    pub fn finish(mut self) -> Result<HtmlCapture, HtmlCaptureError> {
        let mut scratch = preflight::DepthScratch::new();
        self.preflight_finish(&mut scratch, &mut preflight::ordinary)
            .map_err(preflight::ordinary_error)?;
        Ok(self.publish())
    }

    pub(crate) fn publish(self) -> HtmlCapture {
        HtmlCapture(Arc::new(self.data))
    }

    /// Publishes the assembled data from a builder the caller owns only by reference. The records
    /// move into the capture and the builder keeps the shared identities, so it is a spent shell
    /// that the caller replaces or drops at once and never assembles into again.
    pub(crate) fn publish_taking(&mut self) -> HtmlCapture {
        HtmlCapture(Arc::new(CaptureData {
            identity: self.data.identity.clone(),
            host_revision: self.data.host_revision.clone(),
            records: std::mem::take(&mut self.data.records),
            registries: std::mem::take(&mut self.data.registries),
            blocks: std::mem::take(&mut self.data.blocks),
            block_sequence: self.data.block_sequence,
            usage: self.data.usage,
        }))
    }

    /// Validates borrowed data before the caller moves it into the immutable wrapper.
    /// Admission failure retains this exact builder and the caller's actual scratch.
    pub(crate) fn preflight_finish<E>(
        &mut self,
        scratch: &mut preflight::DepthScratch,
        admit: &mut impl FnMut(preflight::Action) -> Result<(), E>,
    ) -> Result<(), preflight::Failure<E>> {
        preflight::references(self, admit)?;
        preflight::depths(self, scratch, admit)
    }
}

/// Borrowed captured record. Its IDs are issued by the owning immutable capture.
#[derive(Clone, Copy, Debug)]
pub struct HtmlCapturedNode<'a> {
    data: &'a CaptureData,
    index: usize,
}
impl<'a> HtmlCapturedNode<'a> {
    pub fn id(self) -> HtmlCaptureSubjectId {
        subject_id(self.data, self.index)
    }
    pub fn kind(self) -> &'a HtmlCapturedNodeKind {
        &self.data.records[self.index].kind
    }
    pub fn node_document(self) -> HtmlCaptureDocumentId {
        HtmlCaptureDocumentId(subject_id(
            self.data,
            self.data.records[self.index].document,
        ))
    }
    /// Conditional template-owner cache for an actual captured Document.
    pub fn template_document_cache(self) -> Option<HtmlCapturedTemplateCache> {
        let HtmlCapturedNodeKind::Document(document) = self.kind() else {
            return None;
        };
        Some(match document.template_cache {
            Association::Absent => HtmlCapturedTemplateCache::Absent,
            Association::Unavailable => HtmlCapturedTemplateCache::Unavailable,
            Association::Index(index) => HtmlCapturedTemplateCache::Document(
                HtmlCaptureDocumentId(subject_id(self.data, index)),
            ),
        })
    }
    pub fn parent(self) -> HtmlCapturedParent {
        match self.data.records[self.index].parent {
            Parent::Root => HtmlCapturedParent::Root,
            Parent::Index(i) => HtmlCapturedParent::Subject(subject_id(self.data, i)),
            Parent::Unavailable => HtmlCapturedParent::Unavailable,
        }
    }
    pub fn children(self) -> HtmlCapturedChildren<'a> {
        match &self.data.records[self.index].children {
            Some(indices) => HtmlCapturedChildren::Complete(HtmlCapturedChildIds {
                identity: &self.data.identity,
                indices: indices.iter(),
            }),
            None => HtmlCapturedChildren::Unavailable,
        }
    }
    fn local_registry(self) -> Registry {
        match self.kind() {
            HtmlCapturedNodeKind::Document(d) => d.registry,
            HtmlCapturedNodeKind::Element(e) => e.registry,
            HtmlCapturedNodeKind::Fragment(f)
                if matches!(f.kind, HtmlCapturedFragmentKind::ShadowRoot(_)) =>
            {
                f.registry
            }
            _ => Registry::Null,
        }
    }
    /// Whether the immutable lookup will clone one capture identity.
    /// Observe this header before admitting the conditional ID transfer.
    pub fn lookup_registry_clones_identity(self) -> bool {
        matches!(self.local_registry(), Registry::Index(_))
    }
    /// Source-defined registry lookup: plain fragments and other nodes return null.
    pub fn lookup_registry(self) -> HtmlCapturedRegistry {
        registry_fact(self.data, self.local_registry())
    }
    pub fn host(self) -> Option<HtmlCapturedHost> {
        let HtmlCapturedNodeKind::Fragment(f) = self.kind() else {
            return None;
        };
        Some(match f.host {
            Association::Absent => HtmlCapturedHost::Absent,
            Association::Index(i) => {
                HtmlCapturedHost::Element(HtmlCaptureElementId(subject_id(self.data, i)))
            }
            Association::Unavailable => HtmlCapturedHost::Unavailable,
        })
    }
    pub fn shadow_root(self) -> Option<HtmlCapturedShadowRoot> {
        let HtmlCapturedNodeKind::Element(e) = self.kind() else {
            return None;
        };
        Some(match e.shadow {
            Association::Absent => HtmlCapturedShadowRoot::Absent,
            Association::Index(i) => {
                HtmlCapturedShadowRoot::Present(HtmlCaptureFragmentId(subject_id(self.data, i)))
            }
            Association::Unavailable => HtmlCapturedShadowRoot::Unavailable,
        })
    }
    /// None means this kind is not an HTML template, rather than unavailable contents.
    pub fn template_contents(self) -> Option<HtmlCapturedTemplateContents> {
        let HtmlCapturedNodeKind::Element(e) = self.kind() else {
            return None;
        };
        match e.template {
            TemplateAssociation::Absent => None,
            TemplateAssociation::Owned(i) | TemplateAssociation::ShadowReference(i) => {
                Some(HtmlCapturedTemplateContents::Present(
                    HtmlCaptureFragmentId(subject_id(self.data, i)),
                ))
            }
            TemplateAssociation::Unavailable => Some(HtmlCapturedTemplateContents::Unavailable),
        }
    }
    /// The established contents' owning or non-owning relationship kind.
    /// Non-template elements and unavailable contents return None.
    pub fn template_contents_kind(
        self,
    ) -> Option<surgeist_html::construction::HtmlTemplateContentsKind> {
        let HtmlCapturedNodeKind::Element(element) = self.kind() else {
            return None;
        };
        match element.template {
            TemplateAssociation::Owned(_) => {
                Some(surgeist_html::construction::HtmlTemplateContentsKind::Owned)
            }
            TemplateAssociation::ShadowReference(_) => {
                Some(surgeist_html::construction::HtmlTemplateContentsKind::ShadowReference)
            }
            TemplateAssociation::Absent | TemplateAssociation::Unavailable => None,
        }
    }
}
fn subject_id(data: &CaptureData, index: usize) -> HtmlCaptureSubjectId {
    HtmlCaptureSubjectId {
        capture: data.identity.clone(),
        index,
    }
}
fn registry_fact(data: &CaptureData, registry: Registry) -> HtmlCapturedRegistry {
    match registry {
        Registry::Null => HtmlCapturedRegistry::Null,
        Registry::Unavailable => HtmlCapturedRegistry::Unavailable,
        Registry::Index(index) => HtmlCapturedRegistry::Registry(HtmlCaptureRegistryId {
            capture: data.identity.clone(),
            index,
        }),
    }
}
/// Complete definition metadata, including an empty set, or unavailable evidence.
#[derive(Clone, Copy, Debug)]
pub enum HtmlCapturedDefinitions<'a> {
    Complete(&'a [HtmlCustomElementDefinition]),
    Unavailable,
}
impl HtmlCapture {
    pub fn identity(&self) -> &HtmlCaptureId {
        &self.0.identity
    }
    pub fn host_facts_revision(&self) -> &HtmlHostFactsRevision {
        &self.0.host_revision
    }
    /// Actual registry scope, independent of complete definition metadata.
    pub fn registry_scope(
        &self,
        registry: &HtmlCaptureRegistryId,
    ) -> Result<HtmlRegistryScope, HtmlCaptureError> {
        if registry.capture != self.0.identity {
            return Err(HtmlCaptureError::ForeignCapture);
        }
        Ok(self.0.registries[registry.index].scope)
    }
    fn template_cache_association(
        &self,
        document: &HtmlCaptureDocumentId,
    ) -> Result<Association, HtmlCaptureError> {
        let node = self.node(document.subject())?;
        let HtmlCapturedNodeKind::Document(facts) = node.kind() else {
            return Err(HtmlCaptureError::WrongKind {
                expected: HtmlCaptureSubjectKind::Document,
                actual: node.kind().subject_kind(),
            });
        };
        Ok(facts.template_cache)
    }
    /// Whether this checked document's cache read will clone one capture identity.
    /// Absent and unavailable caches both require no ID transfer.
    pub fn template_document_cache_clones_identity(
        &self,
        document: &HtmlCaptureDocumentId,
    ) -> Result<bool, HtmlCaptureError> {
        Ok(matches!(
            self.template_cache_association(document)?,
            Association::Index(_)
        ))
    }
    /// Reads one checked document's cache, preserving unavailable and absent facts.
    pub fn template_document_cache(
        &self,
        document: &HtmlCaptureDocumentId,
    ) -> Result<HtmlCapturedTemplateCache, HtmlCaptureError> {
        Ok(match self.template_cache_association(document)? {
            Association::Absent => HtmlCapturedTemplateCache::Absent,
            Association::Unavailable => HtmlCapturedTemplateCache::Unavailable,
            Association::Index(index) => HtmlCapturedTemplateCache::Document(
                HtmlCaptureDocumentId(subject_id(&self.0, index)),
            ),
        })
    }
    pub fn usage(&self) -> HtmlCaptureUsage {
        self.0.usage
    }
    /// Converts this builder's retained handle to an immutable subject identity.
    pub fn subject_id(
        &self,
        handle: &HtmlCaptureBuilderSubject,
    ) -> Result<HtmlCaptureSubjectId, HtmlCaptureError> {
        if handle.capture != self.0.identity {
            return Err(HtmlCaptureError::ForeignBuilder);
        }
        Ok(subject_id(&self.0, handle.index))
    }
    pub fn registry_id(
        &self,
        handle: &HtmlCaptureBuilderRegistry,
    ) -> Result<HtmlCaptureRegistryId, HtmlCaptureError> {
        if handle.capture != self.0.identity {
            return Err(HtmlCaptureError::ForeignBuilder);
        }
        Ok(HtmlCaptureRegistryId {
            capture: self.0.identity.clone(),
            index: handle.index,
        })
    }
    pub fn node(
        &self,
        id: &HtmlCaptureSubjectId,
    ) -> Result<HtmlCapturedNode<'_>, HtmlCaptureError> {
        if id.capture != self.0.identity {
            return Err(HtmlCaptureError::ForeignCapture);
        }
        Ok(HtmlCapturedNode {
            data: &self.0,
            index: id.index,
        })
    }
    fn require(
        &self,
        id: &HtmlCaptureSubjectId,
        expected: HtmlCaptureSubjectKind,
    ) -> Result<(), HtmlCaptureError> {
        let actual = self.node(id)?.kind().subject_kind();
        if actual != expected {
            return Err(HtmlCaptureError::WrongKind { expected, actual });
        }
        Ok(())
    }
    pub fn document_id(
        &self,
        id: &HtmlCaptureSubjectId,
    ) -> Result<HtmlCaptureDocumentId, HtmlCaptureError> {
        self.require(id, HtmlCaptureSubjectKind::Document)?;
        Ok(HtmlCaptureDocumentId(id.clone()))
    }
    pub fn element_id(
        &self,
        id: &HtmlCaptureSubjectId,
    ) -> Result<HtmlCaptureElementId, HtmlCaptureError> {
        self.require(id, HtmlCaptureSubjectKind::Element)?;
        Ok(HtmlCaptureElementId(id.clone()))
    }
    pub fn fragment_id(
        &self,
        id: &HtmlCaptureSubjectId,
    ) -> Result<HtmlCaptureFragmentId, HtmlCaptureError> {
        self.require(id, HtmlCaptureSubjectKind::Fragment)?;
        Ok(HtmlCaptureFragmentId(id.clone()))
    }
    pub fn definition_id(
        &self,
        handle: &HtmlCaptureBuilderDefinition,
    ) -> Result<HtmlCaptureDefinitionId, HtmlCaptureError> {
        if handle.capture != self.0.identity {
            return Err(HtmlCaptureError::ForeignCapture);
        }
        let id = HtmlCaptureDefinitionId {
            registry: HtmlCaptureRegistryId {
                capture: self.0.identity.clone(),
                index: handle.registry_index,
            },
            record_index: handle.record_index,
        };
        self.definition(&id)?;
        Ok(id)
    }
    pub fn definition(
        &self,
        id: &HtmlCaptureDefinitionId,
    ) -> Result<&HtmlCustomElementDefinition, HtmlCaptureError> {
        if id.registry.capture != self.0.identity {
            return Err(HtmlCaptureError::ForeignCapture);
        }
        self.0
            .registries
            .get(id.registry.index)
            .and_then(|r| r.definitions.get(id.record_index))
            .ok_or(HtmlCaptureError::UnknownDefinition)
    }
    /// Begins allocation-free ordered local/is-name lookup in this registry.
    /// The caller selects HTML namespace applicability before this operation.
    /// Unavailable definitions retain their actual registry identity in the error.
    pub fn definition_lookup_cursor<'a>(
        &'a self,
        registry: &'a HtmlCapturedRegistry,
        local: &'a DocumentString,
        is_value: Option<&'a DocumentString>,
    ) -> Result<HtmlDefinitionLookupCursor<'a>, HtmlCaptureError> {
        let source = match registry {
            HtmlCapturedRegistry::Null => LookupSource::Null,
            HtmlCapturedRegistry::Unavailable => {
                return Err(HtmlCaptureError::MissingFact {
                    subject: None,
                    fact: HtmlCaptureFact::Registry,
                });
            }
            HtmlCapturedRegistry::Registry(id) => {
                let definitions = match self.definitions(id)? {
                    HtmlCapturedDefinitions::Complete(d) => d,
                    HtmlCapturedDefinitions::Unavailable => {
                        return Err(HtmlCaptureError::UnavailableDefinitions(id.clone()));
                    }
                };
                LookupSource::Registry { id, definitions }
            }
        };
        let ready = matches!(source, LookupSource::Null).then_some(None);
        Ok(HtmlDefinitionLookupCursor {
            source,
            local,
            is_value,
            phase: LookupPhase::Local,
            record: 0,
            field: 0,
            offset: None,
            ready,
        })
    }
    pub fn definitions(
        &self,
        registry: &HtmlCaptureRegistryId,
    ) -> Result<HtmlCapturedDefinitions<'_>, HtmlCaptureError> {
        if registry.capture != self.0.identity {
            return Err(HtmlCaptureError::ForeignCapture);
        }
        let record = &self.0.registries[registry.index];
        Ok(if record.complete {
            HtmlCapturedDefinitions::Complete(&record.definitions)
        } else {
            HtmlCapturedDefinitions::Unavailable
        })
    }
    /// Pure pinned HTML definition lookup. It neither upgrades nor executes elements.
    pub fn lookup_definition(
        &self,
        registry: &HtmlCapturedRegistry,
        namespace_value: &Namespace,
        local_name: &DocumentString,
        is_value: Option<&DocumentString>,
    ) -> Result<Option<&HtmlCustomElementDefinition>, HtmlCaptureError> {
        if let HtmlCapturedRegistry::Registry(id) = registry
            && id.capture != self.0.identity
        {
            return Err(HtmlCaptureError::ForeignCapture);
        }
        if !namespace(namespace_value, "http://www.w3.org/1999/xhtml")
            || matches!(registry, HtmlCapturedRegistry::Null)
        {
            return Ok(None);
        }
        let mut cursor = self.definition_lookup_cursor(registry, local_name, is_value)?;
        loop {
            if let HtmlDefinitionLookupStep::Ready(id) = cursor.step() {
                return id.as_ref().map(|id| self.definition(id)).transpose();
            }
        }
    }

    /// Resolves the actual inclusive ordinary ancestor form, without following hosts.
    /// The walk is bounded by the capture's depth, and nothing here admits or
    /// cancels it; use [`nearest_form_with`](Self::nearest_form_with) to meter it.
    pub fn nearest_form(
        &self,
        element: &HtmlCaptureElementId,
    ) -> Result<Option<HtmlCaptureElementId>, HtmlCaptureError> {
        match self.nearest_form_with::<std::convert::Infallible>(element, || Ok(())) {
            Ok(form) => Ok(form),
            Err(HtmlCaptureEdgeError::Capture(error)) => Err(error),
            Err(HtmlCaptureEdgeError::Interrupted(never)) => match never {},
        }
    }
    /// As [`nearest_form`](Self::nearest_form), taking one `step` before each
    /// record it examines: the element itself, then each ordinary ancestor in
    /// turn, up to and including the form it finds or the topmost ancestor. The
    /// first refusal of `step` is returned unchanged and ends the walk.
    pub fn nearest_form_with<E>(
        &self,
        element: &HtmlCaptureElementId,
        mut step: impl FnMut() -> Result<(), E>,
    ) -> Result<Option<HtmlCaptureElementId>, HtmlCaptureEdgeError<E>> {
        self.require(&element.0, HtmlCaptureSubjectKind::Element)?;
        let mut i = element.0.index;
        loop {
            step().map_err(HtmlCaptureEdgeError::Interrupted)?;
            let r = &self.0.records[i];
            if let HtmlCapturedNodeKind::Element(e) = &r.kind
                && html(&e.name, "form")
            {
                return Ok(Some(HtmlCaptureElementId(subject_id(&self.0, i))));
            }
            match r.parent {
                Parent::Root => return Ok(None),
                Parent::Index(parent) => i = parent,
                Parent::Unavailable => {
                    return Err(HtmlCaptureEdgeError::Capture(
                        HtmlCaptureError::MissingFact {
                            subject: Some(subject_id(&self.0, i)),
                            fact: HtmlCaptureFact::Parent,
                        },
                    ));
                }
            }
        }
    }

    /// Returns known absence/presence, or `None` for an unavailable shadow fact.
    pub fn shadow_presence(
        &self,
        element: &HtmlCaptureElementId,
    ) -> Result<Option<bool>, HtmlCaptureError> {
        let node = self.node(element.subject())?;
        let element = node.kind().element()?;
        Ok(match element.shadow {
            Association::Absent => Some(false),
            Association::Index(_) => Some(true),
            Association::Unavailable => None,
        })
    }

    /// Answers DOM "attach a shadow root" (steps 1 to 4) for `element` and a
    /// request for a root of `mode`, from captured facts only.
    ///
    /// The steps are consulted in specification order and only the facts the
    /// first reached step needs are required: an invalid host needs no registry
    /// or shadow facts, and a disabling definition is reported before the
    /// host's shadow root is read. The request's other inputs (delegates focus,
    /// serializable, slot assignment, clonable and registry) take part in no
    /// step that can fail, so they are not parameters.
    pub fn shadow_attachment_status(
        &self,
        element: &HtmlCaptureElementId,
        mode: HtmlShadowRootMode,
    ) -> Result<HtmlShadowAttachmentStatus, HtmlCaptureError> {
        let node = self.node(&element.0)?;
        let e = node.kind().element()?;
        let custom_name = validate_custom_element_name(e.name.local_name()).is_ok();
        let ordinary_host = [
            "article",
            "aside",
            "blockquote",
            "body",
            "div",
            "footer",
            "h1",
            "h2",
            "h3",
            "h4",
            "h5",
            "h6",
            "header",
            "main",
            "nav",
            "p",
            "section",
            "span",
        ]
        .iter()
        .any(|n| exact(e.name.local_name(), n));
        // Steps 1 and 2.
        if !namespace(e.name.namespace(), "http://www.w3.org/1999/xhtml")
            || !(custom_name || ordinary_host)
        {
            return Ok(HtmlShadowAttachmentStatus::InvalidHost);
        }
        // Step 3: a matching definition with disable shadow set throws. Its
        // condition is a valid custom element name or a non-null `is` value.
        if (custom_name || !matches!(e.is_value, HtmlCapturedIsValue::Absent))
            && self.definition_disables_shadow(element, node, custom_name)?
        {
            return Ok(HtmlShadowAttachmentStatus::DefinitionDisallows);
        }
        // Step 4: an existing shadow root is either rejected or reused.
        match e.shadow {
            Association::Absent => Ok(HtmlShadowAttachmentStatus::Allowed),
            Association::Unavailable => Err(HtmlCaptureError::MissingFact {
                subject: Some(element.0.clone()),
                fact: HtmlCaptureFact::ShadowRoot,
            }),
            Association::Index(root) => {
                let Some(HtmlCapturedNodeKind::Fragment(HtmlCapturedFragment {
                    kind: HtmlCapturedFragmentKind::ShadowRoot(current),
                    ..
                })) = self.0.records.get(root).map(|record| &record.kind)
                else {
                    return Err(HtmlCaptureError::MissingFact {
                        subject: Some(element.0.clone()),
                        fact: HtmlCaptureFact::ShadowRoot,
                    });
                };
                Ok(
                    if !current.declarative() || current.properties().mode() != mode {
                        HtmlShadowAttachmentStatus::ExistingRootDisallows
                    } else {
                        HtmlShadowAttachmentStatus::DeclarativeRootReused
                    },
                )
            }
        }
    }
    /// Step 3.1 and 3.2: looks up the host's definition in its own registry,
    /// first by local name and then by `is` value, and reads its disable shadow.
    fn definition_disables_shadow(
        &self,
        element: &HtmlCaptureElementId,
        node: HtmlCapturedNode<'_>,
        custom_name: bool,
    ) -> Result<bool, HtmlCaptureError> {
        let HtmlCapturedNodeKind::Element(e) = node.kind() else {
            return Ok(false);
        };
        let registry = node.lookup_registry();
        if matches!(registry, HtmlCapturedRegistry::Null) {
            return Ok(false);
        }
        if custom_name
            && let Some(d) =
                self.lookup_definition(&registry, e.name.namespace(), e.name.local_name(), None)?
        {
            return Ok(d.disable_shadow);
        }
        let is_value = match &e.is_value {
            HtmlCapturedIsValue::Absent => return Ok(false),
            HtmlCapturedIsValue::Value(value) => value,
            HtmlCapturedIsValue::Unavailable => {
                return Err(HtmlCaptureError::MissingFact {
                    subject: Some(element.0.clone()),
                    fact: HtmlCaptureFact::IsValue,
                });
            }
        };
        Ok(self
            .lookup_definition(
                &registry,
                e.name.namespace(),
                e.name.local_name(),
                Some(is_value),
            )?
            .is_some_and(|d| d.disable_shadow))
    }
}

/// DOM "attach a shadow root" outcome for one host and requested mode, in
/// specification step order. `Allowed` creates a new root (step 5 onward,
/// which cannot fail); the other variants are the reached early exits.
///
/// Both `Allowed` and `DeclarativeRootReused` mean the attach does not throw:
/// `Allowed` creates a root, while `DeclarativeRootReused` reuses the existing
/// one (steps 4.3 to 4.5). The remaining variants are the throwing outcomes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlShadowAttachmentStatus {
    /// Steps 1 to 4 pass and the host has no shadow root.
    Allowed,
    /// Step 1 or 2: the host is not an HTML-namespace element with a valid shadow host name.
    InvalidHost,
    /// Step 3.2: the host's definition has disable shadow set.
    DefinitionDisallows,
    /// Step 4.2: the existing root is not declarative or its mode differs from the request.
    ExistingRootDisallows,
    /// Step 4.3 to 4.5: the existing root is declarative and has the requested mode,
    /// so the algorithm empties it, clears its declarative flag and returns.
    DeclarativeRootReused,
}
/// A checked target of the fragment algorithm, not an imported output wrapper.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlFragmentTarget {
    Element(HtmlCaptureElementId),
    HostedFragment(HtmlCaptureFragmentId),
}
/// Checked source context retained from one immutable capture.
/// No parse request, recovered report, effect application or execution is performed.
#[derive(Clone, Debug)]
pub struct HtmlFragmentContext {
    capture: HtmlCapture,
    target: HtmlFragmentTarget,
    context: HtmlCaptureElementId,
    target_document: HtmlCaptureDocumentId,
    context_document: HtmlCaptureDocumentId,
    nearest_form: Option<HtmlCaptureElementId>,
    root_registry: HtmlCapturedRegistry,
    /// What the context element and its Document contribute, read once from the immutable capture
    /// when the context was selected.
    context_name: QualifiedName,
    context_html_integration_point: bool,
    document_mode: HtmlDocumentMode,
    scripting: HtmlScriptingStatus,
}
impl HtmlFragmentContext {
    pub fn element(
        capture: &HtmlCapture,
        target: &HtmlCaptureSubjectId,
    ) -> Result<Self, HtmlCaptureError> {
        Self::unmetered(Self::element_with(capture, target, || Ok(())))
    }
    /// As [`element`](Self::element), taking one `step` for each record the walk
    /// for the nearest form examines; see [`HtmlCapture::nearest_form_with`].
    pub fn element_with<E>(
        capture: &HtmlCapture,
        target: &HtmlCaptureSubjectId,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Self, HtmlCaptureEdgeError<E>> {
        let element = capture.element_id(target)?;
        Self::select(
            capture,
            HtmlFragmentTarget::Element(element.clone()),
            element,
            step,
        )
    }
    pub fn hosted_fragment(
        capture: &HtmlCapture,
        target: &HtmlCaptureSubjectId,
    ) -> Result<Self, HtmlCaptureError> {
        Self::unmetered(Self::hosted_fragment_with(capture, target, || Ok(())))
    }
    /// As [`hosted_fragment`](Self::hosted_fragment), metered as
    /// [`element_with`](Self::element_with) is.
    pub fn hosted_fragment_with<E>(
        capture: &HtmlCapture,
        target: &HtmlCaptureSubjectId,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Self, HtmlCaptureEdgeError<E>> {
        let fragment = capture.fragment_id(target)?;
        let node = capture.node(target)?;
        let context = match node.host() {
            Some(HtmlCapturedHost::Element(host)) => host,
            Some(HtmlCapturedHost::Absent) => {
                return Err(HtmlCaptureError::UnhostedFragment.into());
            }
            _ => {
                return Err(HtmlCaptureError::MissingFact {
                    subject: Some(target.clone()),
                    fact: HtmlCaptureFact::FragmentHost,
                }
                .into());
            }
        };
        Self::select(
            capture,
            HtmlFragmentTarget::HostedFragment(fragment),
            context,
            step,
        )
    }
    /// The result of a selection whose step never refuses.
    fn unmetered(
        selected: Result<Self, HtmlCaptureEdgeError<std::convert::Infallible>>,
    ) -> Result<Self, HtmlCaptureError> {
        match selected {
            Ok(context) => Ok(context),
            Err(HtmlCaptureEdgeError::Capture(error)) => Err(error),
            Err(HtmlCaptureEdgeError::Interrupted(never)) => match never {},
        }
    }
    fn select<E>(
        capture: &HtmlCapture,
        target: HtmlFragmentTarget,
        context: HtmlCaptureElementId,
        step: impl FnMut() -> Result<(), E>,
    ) -> Result<Self, HtmlCaptureEdgeError<E>> {
        let target_id = match &target {
            HtmlFragmentTarget::Element(e) => &e.0,
            HtmlFragmentTarget::HostedFragment(f) => &f.0,
        };
        let target_node = capture.node(target_id)?;
        let root_registry = target_node.lookup_registry();
        if matches!(root_registry, HtmlCapturedRegistry::Unavailable) {
            return Err(HtmlCaptureError::MissingFact {
                subject: Some(target_id.clone()),
                fact: HtmlCaptureFact::Registry,
            }
            .into());
        }
        let target_document = target_node.node_document();
        let context_document = capture.node(&context.0)?.node_document();
        let nearest_form = capture.nearest_form_with(&context, step)?;
        let element = capture.node(&context.0)?.kind().element()?;
        let context_name = element.name.clone();
        let context_html_integration_point =
            surgeist_html::parsing::is_html_integration_point(&element.name, &element.attributes);
        let document = capture.node(&context_document.0)?.kind().document()?;
        let (document_mode, scripting) = (document.mode, document.scripting);
        Ok(Self {
            capture: capture.clone(),
            target,
            context,
            target_document,
            context_document,
            nearest_form,
            root_registry,
            context_name,
            context_html_integration_point,
            document_mode,
            scripting,
        })
    }
    pub fn capture(&self) -> &HtmlCapture {
        &self.capture
    }
    pub fn target(&self) -> &HtmlFragmentTarget {
        &self.target
    }
    pub fn context_element(&self) -> &HtmlCaptureElementId {
        &self.context
    }
    pub fn target_document(&self) -> &HtmlCaptureDocumentId {
        &self.target_document
    }
    pub fn context_document(&self) -> &HtmlCaptureDocumentId {
        &self.context_document
    }
    pub fn nearest_form(&self) -> Option<&HtmlCaptureElementId> {
        self.nearest_form.as_ref()
    }
    pub fn root_registry(&self) -> &HtmlCapturedRegistry {
        &self.root_registry
    }
    pub fn document_mode(&self) -> HtmlDocumentMode {
        self.document_mode
    }
    pub fn scripting_status(&self) -> HtmlScriptingStatus {
        self.scripting
    }
    /// Resolves §13.4's context override and this independent parser's capability.
    /// Disabled context forces Disabled before execution admission; enabled
    /// Fragment is explicitly unsupported, never downgraded to Disabled.
    pub fn resolve_scripting_mode(
        &self,
        requested: HtmlParserScriptingMode,
    ) -> Result<HtmlParserScriptingMode, HtmlCaptureError> {
        if !matches!(
            requested,
            HtmlParserScriptingMode::Inert | HtmlParserScriptingMode::Fragment
        ) {
            return Err(HtmlCaptureError::InvalidFragmentScriptingMode(requested));
        }
        match self.scripting_status() {
            HtmlScriptingStatus::Disabled => Ok(HtmlParserScriptingMode::Disabled),
            HtmlScriptingStatus::Enabled if requested == HtmlParserScriptingMode::Inert => {
                Ok(HtmlParserScriptingMode::Inert)
            }
            HtmlScriptingStatus::Enabled => Err(HtmlCaptureError::UnsupportedExecution(requested)),
            HtmlScriptingStatus::Unavailable => Err(HtmlCaptureError::MissingFact {
                subject: Some(self.context_document.0.clone()),
                fact: HtmlCaptureFact::ScriptingStatus,
            }),
        }
    }
    pub fn is_template(&self) -> bool {
        html(&self.context_name, "template")
    }
    pub fn is_mathml_text_integration_point(&self) -> bool {
        surgeist_html::parsing::is_mathml_text_integration_point(&self.context_name)
    }
    pub fn is_html_integration_point(&self) -> bool {
        self.context_html_integration_point
    }
    /// Initial tokenizer choice derives from context identity and effective mode.
    pub fn initial_tokenizer_state(
        &self,
        requested: HtmlParserScriptingMode,
    ) -> Result<HtmlFragmentTokenizerState, HtmlCaptureError> {
        let mode = self.resolve_scripting_mode(requested)?;
        Ok(surgeist_html::parsing::fragment_tokenizer_state(
            &self.context_name,
            mode,
        ))
    }
    /// Resets initial insertion mode with the single internal root replaced by context.
    /// Captured ancestor templates/tables are not put on the parser's open stack.
    pub fn initial_insertion_mode(&self) -> HtmlFragmentInsertionMode {
        surgeist_html::parsing::fragment_insertion_mode(&self.context_name)
    }
}
/// Allocation-free ordered lookup. Every action is admitted before `step`;
/// comparison headers cost one, each pair two, and Ready transfer one.
pub struct HtmlDefinitionLookupCursor<'a> {
    source: LookupSource<'a>,
    local: &'a DocumentString,
    is_value: Option<&'a DocumentString>,
    phase: LookupPhase<'a>,
    record: usize,
    field: u8,
    offset: Option<usize>,
    ready: Option<Option<usize>>,
}
/// Where a lookup reads definitions from: a null registry has none to find, and a registry
/// keeps its identity together with its definitions.
#[derive(Clone, Copy)]
enum LookupSource<'a> {
    Null,
    Registry {
        id: &'a HtmlCaptureRegistryId,
        definitions: &'a [HtmlCustomElementDefinition],
    },
}
/// The name a pass compares with: the local name first, then the `is` value, which exists only
/// when one was supplied.
#[derive(Clone, Copy)]
enum LookupPhase<'a> {
    Local,
    Is(&'a DocumentString),
}
/// One admitted lookup step; only `Ready(Some(_))` shares a registry identity.
pub enum HtmlDefinitionLookupStep {
    Searching,
    Ready(Option<HtmlCaptureDefinitionId>),
}
impl HtmlDefinitionLookupCursor<'_> {
    /// Describes the actual next ready transfer without advancing the cursor.
    /// Only a found definition clones a retained registry handle.
    pub fn next_clones_registry(&self) -> bool {
        matches!(self.ready, Some(Some(_)))
    }
    /// One header/transfer unit or two exact UTF-16 comparison units.
    pub fn next_cost(&self) -> usize {
        if self.offset.is_some() { 2 } else { 1 }
    }
    fn next_record(&mut self) {
        self.record += 1;
        self.field = 0;
        self.offset = None;
    }
    /// Performs the next comparison or ready transfer after caller admission.
    pub fn step(&mut self) -> HtmlDefinitionLookupStep {
        let LookupSource::Registry { id, definitions } = self.source else {
            // A null registry has no definitions, so nothing is found in it.
            self.ready = Some(None);
            return HtmlDefinitionLookupStep::Ready(None);
        };
        if let Some(record) = self.ready {
            return HtmlDefinitionLookupStep::Ready(record.map(|record_index| {
                HtmlCaptureDefinitionId {
                    registry: id.clone(),
                    record_index,
                }
            }));
        }
        if self.record == definitions.len() {
            match (self.phase, self.is_value) {
                (LookupPhase::Local, Some(is_value)) => {
                    self.phase = LookupPhase::Is(is_value);
                    self.record = 0;
                }
                (LookupPhase::Local, None) | (LookupPhase::Is(_), _) => {
                    self.ready = Some(None);
                }
            }
            return HtmlDefinitionLookupStep::Searching;
        }
        let d = &definitions[self.record];
        let (left, right) = if self.field == 0 {
            (
                &d.name,
                match self.phase {
                    LookupPhase::Local => self.local,
                    LookupPhase::Is(is_value) => is_value,
                },
            )
        } else {
            (&d.local_name, self.local)
        };
        match self.offset {
            None => {
                if left.len() != right.len() {
                    self.next_record();
                } else if left.is_empty() {
                    self.field_finished();
                } else {
                    self.offset = Some(0);
                }
            }
            Some(offset) => {
                if left.code_units()[offset] != right.code_units()[offset] {
                    self.next_record();
                } else if offset + 1 == left.len() {
                    self.field_finished();
                } else {
                    self.offset = Some(offset + 1);
                }
            }
        }
        HtmlDefinitionLookupStep::Searching
    }
    fn field_finished(&mut self) {
        self.offset = None;
        if self.field == 0 {
            self.field = 1;
        } else {
            self.ready = Some(Some(self.record));
        }
    }
}

#[cfg(test)]
mod depth_tests;
#[cfg(test)]
mod definition_cursor_tests {
    use super::*;
    use surgeist_html::values::DocumentString;

    fn capture(
        definitions: Vec<HtmlCustomElementDefinition>,
    ) -> (HtmlCapture, HtmlCapturedRegistry) {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(0, 0, 0, 1000, 0, 1, 10, 16 * 1024 * 1024),
            HtmlHostFactsRevision::fresh(),
        );
        let r = builder.add_registry(Some(definitions)).unwrap();
        let capture = builder.finish().unwrap();
        let registry = HtmlCapturedRegistry::Registry(capture.registry_id(&r).unwrap());
        (capture, registry)
    }
    fn run(
        cursor: &mut HtmlDefinitionLookupCursor<'_>,
    ) -> (usize, Option<HtmlCaptureDefinitionId>) {
        let mut work = 0;
        loop {
            work += cursor.next_cost();
            if let HtmlDefinitionLookupStep::Ready(id) = cursor.step() {
                return (work, id);
            }
        }
    }
    #[test]
    fn autonomous_lookup_charges_two_exact_comparisons_then_ready_transfer() {
        let (capture, registry) = capture(vec![HtmlCustomElementDefinition::new(
            "x-host".into(),
            "x-host".into(),
            false,
            false,
        )]);
        let local = DocumentString::from("x-host");
        let current = DocumentString::from("x-other");
        let mut cursor = capture
            .definition_lookup_cursor(&registry, &local, Some(&current))
            .unwrap();
        // Two six-unit equality checks: 2*(one header + 2*6), then Ready.
        let (work, id) = run(&mut cursor);
        assert_eq!(work, 27);
        assert_eq!(capture.definition(&id.unwrap()).unwrap().name(), &local);
    }
    #[test]
    fn second_lookup_search_keeps_header_short_circuit_and_transition_costs() {
        let (capture, registry) = capture(vec![HtmlCustomElementDefinition::new(
            "x-good".into(),
            "button".into(),
            false,
            false,
        )]);
        let local = DocumentString::from("button");
        let current = DocumentString::from("x-good");
        let (work, id) = run(&mut capture
            .definition_lookup_cursor(&registry, &local, Some(&current))
            .unwrap());
        // First field mismatches on its first pair:3; phase transition:1;
        // two six-unit checks:26; Ready transfer:1.
        assert_eq!(work, 31);
        assert_eq!(capture.definition(&id.unwrap()).unwrap().name(), &current);
    }
    #[test]
    fn null_and_complete_empty_lookup_have_distinct_admitted_search_completion() {
        let (capture, registry) = capture(vec![]);
        let local = DocumentString::from("button");
        let current = DocumentString::from("x-good");
        assert_eq!(
            run(&mut capture
                .definition_lookup_cursor(&registry, &local, Some(&current))
                .unwrap()),
            (3, None)
        );
        assert_eq!(
            run(&mut capture
                .definition_lookup_cursor(&HtmlCapturedRegistry::Null, &local, None)
                .unwrap()),
            (1, None)
        );
    }
}
