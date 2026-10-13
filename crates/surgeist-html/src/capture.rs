//! Bounded immutable context facts, independent of canonical DOM identity.
//!
//! Fragment queries follow HTML 877d6146487f §13.4 and §13.2.4–6. Captured
//! availability describes producer evidence; it never advertises host execution.

use crate::custom_names::validate_custom_element_name;
use crate::{
    DocumentString, HtmlDoctype, HtmlDocumentMode, HtmlShadowRootProperties, Namespace,
    OrderedAttributes, QualifiedName,
};
use std::{
    collections::HashSet,
    error::Error,
    fmt,
    hash::{Hash, Hasher},
    sync::Arc,
};

macro_rules! identity {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug)]
        pub struct $name(Arc<()>);
        impl PartialEq for $name {
            fn eq(&self, other: &Self) -> bool {
                Arc::ptr_eq(&self.0, &other.0)
            }
        }
        impl Eq for $name {}
        impl Hash for $name {
            fn hash<H: Hasher>(&self, state: &mut H) {
                Arc::as_ptr(&self.0).hash(state);
            }
        }
    };
}
identity!(
    HtmlCaptureId,
    "Opaque identity of one immutable capture, independent of equal content."
);
identity!(
    HtmlHostFactsRevision,
    "Producer-issued identity of one coherent immutable host-fact revision."
);
impl HtmlHostFactsRevision {
    /// Issues a distinct host-fact revision. Clone to reuse unchanged facts.
    pub fn fresh() -> Self {
        Self(Arc::new(()))
    }
}

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
/// Actual DOM custom-element state, independent of names and current attributes.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCustomElementState {
    Undefined,
    Failed,
    Uncustomized,
    Precustomized,
    Custom,
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
#[derive(Clone, Copy)]
pub(crate) enum CapturedTemplateCacheKey {
    Absent,
    Document(usize),
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
}
impl HtmlCaptureLimits {
    pub fn new(
        subjects: usize,
        relationships: usize,
        attributes: usize,
        utf16_units: usize,
        depth: usize,
        registries: usize,
        definitions: usize,
    ) -> Self {
        Self {
            subjects,
            relationships,
            attributes,
            utf16_units,
            depth,
            registries,
            definitions,
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
}
/// Admitted capture retention, independent of future parser-work accounting.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlCaptureUsage {
    subjects: usize,
    relationships: usize,
    attributes: usize,
    utf16_units: usize,
    depth: usize,
    registries: usize,
    definitions: usize,
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
}
/// Required producer fact that has not been established.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlCaptureFact {
    Parent,
    FragmentHost,
    ScriptingStatus,
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
/// Rejected assembly, lookup or source-derived context admission.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlCaptureError {
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
    CustomElementContextAlreadyEstablished,
    CustomElementDefinitionRequired,
    CustomElementDefinitionNameMismatch,
    CustomElementDefinitionNamespaceMismatch,
    CustomElementSavedIsMismatch,
    DuplicateDefinitionName,
    LimitExceeded {
        resource: HtmlCaptureResource,
        limit: usize,
        required: usize,
    },
    RepresentationExhausted,
    InvalidFragmentScriptingMode(HtmlParserScriptingMode),
    UnsupportedExecution(HtmlParserScriptingMode),
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

/// Captured document facts; mode is actual data, scripting is explicit evidence.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HtmlCapturedDocument {
    mode: HtmlDocumentMode,
    scripting: HtmlScriptingStatus,
    registry: Registry,
    class: HtmlDocumentClass,
    template_role: HtmlTemplateDocumentRole,
    template_cache: Association,
    template_facts_established: bool,
}
impl HtmlCapturedDocument {
    pub fn mode(&self) -> HtmlDocumentMode {
        self.mode
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
    template: Association,
    shadow: Association,
    custom_context: Option<CustomElementContext>,
}
impl HtmlCapturedElement {
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
    depth: usize,
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
    pub fn new(limits: HtmlCaptureLimits, host_revision: HtmlHostFactsRevision) -> Self {
        Self {
            data: CaptureData {
                identity: HtmlCaptureId(Arc::new(())),
                host_revision,
                records: Vec::new(),
                registries: Vec::new(),
                usage: HtmlCaptureUsage::default(),
            },
            limits,
        }
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
    fn admit(
        &self,
        subjects: usize,
        relationships: usize,
        attributes: usize,
        units: usize,
        registries: usize,
        definitions: usize,
    ) -> Result<HtmlCaptureUsage, HtmlCaptureError> {
        let old = self.data.usage;
        let next = HtmlCaptureUsage {
            subjects: add(old.subjects, subjects)?,
            relationships: add(old.relationships, relationships)?,
            attributes: add(old.attributes, attributes)?,
            utf16_units: add(old.utf16_units, units)?,
            depth: old.depth,
            registries: add(old.registries, registries)?,
            definitions: add(old.definitions, definitions)?,
        };
        for (resource, required, limit) in [
            (
                HtmlCaptureResource::Subjects,
                next.subjects,
                self.limits.subjects,
            ),
            (
                HtmlCaptureResource::Relationships,
                next.relationships,
                self.limits.relationships,
            ),
            (
                HtmlCaptureResource::Attributes,
                next.attributes,
                self.limits.attributes,
            ),
            (
                HtmlCaptureResource::Utf16Units,
                next.utf16_units,
                self.limits.utf16_units,
            ),
            (
                HtmlCaptureResource::Registries,
                next.registries,
                self.limits.registries,
            ),
            (
                HtmlCaptureResource::Definitions,
                next.definitions,
                self.limits.definitions,
            ),
        ] {
            if required > limit {
                return Err(HtmlCaptureError::LimitExceeded {
                    resource,
                    limit,
                    required,
                });
            }
        }
        Ok(next)
    }
    fn depth(&self, required: usize) -> Result<(), HtmlCaptureError> {
        if required > self.limits.depth {
            return Err(HtmlCaptureError::LimitExceeded {
                resource: HtmlCaptureResource::Depth,
                limit: self.limits.depth,
                required,
            });
        }
        Ok(())
    }
    fn push(
        &mut self,
        kind: HtmlCapturedNodeKind,
        document: usize,
        parent: Parent,
        usage: HtmlCaptureUsage,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        reserve(&mut self.data.records, 1)?;
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
            depth: 0,
        });
        self.data.usage = usage;
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
        let registry = self.registry(registry)?;
        let usage = self.admit(
            1,
            usize::from(matches!(registry, Registry::Index(_))),
            0,
            0,
            0,
            0,
        )?;
        self.push(
            HtmlCapturedNodeKind::Document(HtmlCapturedDocument {
                mode,
                scripting,
                registry,
                class: HtmlDocumentClass::Unavailable,
                template_role: HtmlTemplateDocumentRole::Unavailable,
                template_cache: Association::Unavailable,
                template_facts_established: false,
            }),
            self.data.records.len(),
            Parent::Root,
            usage,
        )
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
        let HtmlCapturedNodeKind::Document(current) = &self.data.records[index].kind else {
            unreachable!("qualified document")
        };
        if current.template_facts_established {
            return Err(HtmlCaptureError::DocumentFactsAlreadyEstablished);
        }
        let cache = match cache {
            HtmlCaptureTemplateCacheInput::Absent => Association::Absent,
            HtmlCaptureTemplateCacheInput::Unavailable => Association::Unavailable,
            HtmlCaptureTemplateCacheInput::Document(owner) => {
                let owner = self.index(&owner)?;
                self.require(owner, HtmlCaptureSubjectKind::Document)?;
                let HtmlCapturedNodeKind::Document(facts) = &self.data.records[owner].kind else {
                    unreachable!("qualified cache document")
                };
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
        let usage = self.admit(
            0,
            usize::from(matches!(cache, Association::Index(_))),
            0,
            0,
            0,
            0,
        )?;
        let HtmlCapturedNodeKind::Document(current) = &mut self.data.records[index].kind else {
            unreachable!("qualified document")
        };
        current.class = class;
        current.template_role = role;
        current.template_cache = cache;
        current.template_facts_established = true;
        self.data.usage = usage;
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
        let mut units = name_units(&name)?;
        for a in attributes.iter() {
            units = add(units, add(name_units(a.name())?, a.value().len())?)?;
        }
        if let HtmlCapturedIsValue::Value(v) = &is_value {
            units = add(units, v.len())?;
        }
        let usage = self.admit(
            1,
            1 + usize::from(matches!(registry, Registry::Index(_))),
            attributes.len(),
            units,
            0,
            0,
        )?;
        let template = if html(&name, "template") {
            Association::Unavailable
        } else {
            Association::Absent
        };
        self.push(
            HtmlCapturedNodeKind::Element(HtmlCapturedElement {
                name,
                attributes,
                is_value,
                registry,
                template,
                shadow: Association::Unavailable,
                custom_context: None,
            }),
            document,
            Parent::Unavailable,
            usage,
        )
    }
    /// Adds a plain fragment with explicit document and an unavailable host.
    pub fn add_fragment(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        let document = self.index(document)?;
        self.require(document, HtmlCaptureSubjectKind::Document)?;
        let usage = self.admit(1, 1, 0, 0, 0, 0)?;
        self.push(
            HtmlCapturedNodeKind::Fragment(HtmlCapturedFragment {
                kind: HtmlCapturedFragmentKind::Plain,
                host: Association::Unavailable,
                registry: Registry::Null,
            }),
            document,
            Parent::Root,
            usage,
        )
    }
    fn add_leaf(
        &mut self,
        document: &HtmlCaptureBuilderSubject,
        kind: HtmlCapturedNodeKind,
        units: usize,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        let document = self.index(document)?;
        self.require(document, HtmlCaptureSubjectKind::Document)?;
        let usage = self.admit(1, 1, 0, units, 0, 0)?;
        self.push(kind, document, Parent::Unavailable, usage)
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
        let usage = self.admit(0, 0, 0, units, 1, definitions.as_ref().map_or(0, Vec::len))?;
        reserve(&mut self.data.registries, 1)?;
        let index = self.data.registries.len();
        self.data.registries.push(RegistryRecord {
            complete: definitions.is_some(),
            definitions: definitions.unwrap_or_default(),
            scope: HtmlRegistryScope::Unavailable,
            scope_established: false,
        });
        self.data.usage = usage;
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
        let usage = self.admit(
            0,
            0,
            0,
            add(definition.name.len(), definition.local_name.len())?,
            0,
            1,
        )?;
        let records = &mut self.data.registries[registry.index].definitions;
        reserve(records, 1)?;
        let record_index = records.len();
        records.push(definition);
        self.data.usage = usage;
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
        let HtmlCapturedNodeKind::Element(element) = &self.data.records[index].kind else {
            unreachable!()
        };
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
        let usage = self.admit(
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
        let HtmlCapturedNodeKind::Element(element) = &mut self.data.records[index].kind else {
            unreachable!()
        };
        element.custom_context = Some(CustomElementContext { state, definition });
        self.data.usage = usage;
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
    fn edge_depths(
        &self,
        child: usize,
        parent: usize,
    ) -> Result<Vec<(usize, usize)>, HtmlCaptureError> {
        if child == parent {
            return Err(HtmlCaptureError::Cycle);
        }
        if !self.data.records[child].dependents.is_empty() {
            let mut ancestor = Some(parent);
            while let Some(i) = ancestor {
                if i == child {
                    return Err(HtmlCaptureError::Cycle);
                }
                ancestor = self.data.records[i].incoming();
            }
        }
        let first = add(self.data.records[parent].depth, 1)?;
        self.depth(first)?;
        let mut updates = Vec::new();
        reserve(&mut updates, 1)?;
        updates.push((child, first));
        let mut cursor = 0;
        while cursor < updates.len() {
            let (i, depth) = updates[cursor];
            self.depth(depth)?;
            reserve(&mut updates, self.data.records[i].dependents.len())?;
            for &dependent in &self.data.records[i].dependents {
                updates.push((dependent, add(depth, 1)?));
            }
            cursor += 1;
        }
        Ok(updates)
    }
    fn apply_depths(&mut self, updates: Vec<(usize, usize)>) {
        for (i, depth) in updates {
            self.data.records[i].depth = depth;
            self.data.usage.depth = self.data.usage.depth.max(depth);
        }
    }
    /// Establishes an ordinary parent without claiming complete ordered children.
    pub fn establish_parent(
        &mut self,
        child: &HtmlCaptureBuilderSubject,
        parent: &HtmlCaptureBuilderSubject,
    ) -> Result<(), HtmlCaptureError> {
        let child = self.index(child)?;
        let parent = self.index(parent)?;
        if self.data.records[child].parent != Parent::Unavailable {
            return Err(HtmlCaptureError::ParentAlreadyEstablished);
        }
        self.ordinary_edge(child, parent)?;
        let usage = self.admit(0, 2, 0, 0, 0, 0)?;
        let updates = self.edge_depths(child, parent)?;
        reserve(&mut self.data.records[parent].dependents, 1)?;
        self.data.records[child].parent = Parent::Index(parent);
        self.data.records[parent].dependents.push(child);
        self.data.usage = usage;
        self.apply_depths(updates);
        Ok(())
    }
    // Computes forest depths iteratively, including proposed ordinary associations.
    fn all_depths(&self, edges: &[(usize, usize)]) -> Result<Vec<usize>, HtmlCaptureError> {
        let n = self.data.records.len();
        let mut incoming = Vec::new();
        reserve(&mut incoming, n)?;
        incoming.extend(self.data.records.iter().map(Record::incoming));
        for &(child, parent) in edges {
            incoming[child] = Some(parent);
        }
        let mut state = Vec::new();
        reserve(&mut state, n)?;
        state.resize(n, 0u8);
        let mut depths = Vec::new();
        reserve(&mut depths, n)?;
        depths.resize(n, 0);
        let mut path = Vec::new();
        reserve(&mut path, n)?;
        for start in 0..n {
            if state[start] != 0 {
                continue;
            }
            let mut cursor = Some(start);
            let mut depth = 0;
            while let Some(i) = cursor {
                match state[i] {
                    1 => return Err(HtmlCaptureError::Cycle),
                    2 => {
                        depth = add(depths[i], 1)?;
                        break;
                    }
                    _ => {
                        state[i] = 1;
                        path.push(i);
                        cursor = incoming[i];
                    }
                }
            }
            while let Some(i) = path.pop() {
                self.depth(depth)?;
                depths[i] = depth;
                state[i] = 2;
                if !path.is_empty() {
                    depth = add(depth, 1)?;
                }
            }
        }
        Ok(depths)
    }
    /// Establishes all ordered children atomically, also resolving unknown parents.
    pub fn establish_children(
        &mut self,
        parent: &HtmlCaptureBuilderSubject,
        children: &[HtmlCaptureBuilderSubject],
    ) -> Result<(), HtmlCaptureError> {
        let parent = self.index(parent)?;
        if !self.data.records[parent].is_container() {
            return Err(HtmlCaptureError::CannotHaveChildren(
                self.data.records[parent].kind.subject_kind(),
            ));
        }
        if self.data.records[parent].children.is_some() {
            return Err(HtmlCaptureError::ChildrenAlreadyEstablished);
        }
        self.admit(0, children.len(), 0, 0, 0, 0)?;
        let mut indices = Vec::new();
        reserve(&mut indices, children.len())?;
        let mut edges = Vec::new();
        reserve(&mut edges, children.len())?;
        let mut seen = HashSet::new();
        seen.try_reserve(children.len())
            .map_err(|_| HtmlCaptureError::RepresentationExhausted)?;
        for child in children {
            let child = self.index(child)?;
            if !seen.insert(child) {
                return Err(HtmlCaptureError::DuplicateChild);
            }
            self.ordinary_edge(child, parent)?;
            match self.data.records[child].parent {
                Parent::Unavailable => edges.push((child, parent)),
                Parent::Index(p) if p == parent => {}
                _ => return Err(HtmlCaptureError::ParentChildConflict),
            }
            indices.push(child);
        }
        if self.data.records[parent].dependents.iter().any(|i| {
            matches!(self.data.records[*i].parent, Parent::Index(p) if p == parent)
                && !seen.contains(i)
        }) {
            return Err(HtmlCaptureError::ParentChildConflict);
        }
        let relationships = add(
            indices.len(),
            edges
                .len()
                .checked_mul(2)
                .ok_or(HtmlCaptureError::RepresentationExhausted)?,
        )?;
        let usage = self.admit(0, relationships, 0, 0, 0, 0)?;
        let depths = self.all_depths(&edges)?;
        reserve(&mut self.data.records[parent].dependents, edges.len())?;
        for (child, _) in edges {
            self.data.records[child].parent = Parent::Index(parent);
            self.data.records[parent].dependents.push(child);
        }
        self.data.records[parent].children = Some(indices);
        self.data.usage = usage;
        for (i, depth) in depths.into_iter().enumerate() {
            self.data.records[i].depth = depth;
            self.data.usage.depth = self.data.usage.depth.max(depth);
        }
        Ok(())
    }
    pub fn establish_unhosted(
        &mut self,
        fragment: &HtmlCaptureBuilderSubject,
    ) -> Result<(), HtmlCaptureError> {
        let i = self.index(fragment)?;
        self.require(i, HtmlCaptureSubjectKind::Fragment)?;
        let HtmlCapturedNodeKind::Fragment(f) = &mut self.data.records[i].kind else {
            unreachable!()
        };
        if f.host != Association::Unavailable {
            return Err(HtmlCaptureError::FragmentAlreadyHosted);
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
        let HtmlCapturedNodeKind::Element(e) = &mut self.data.records[i].kind else {
            unreachable!()
        };
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
        let t = self.index(template)?;
        let f = self.index(fragment)?;
        self.require(t, HtmlCaptureSubjectKind::Element)?;
        self.require(f, HtmlCaptureSubjectKind::Fragment)?;
        let HtmlCapturedNodeKind::Element(e) = &self.data.records[t].kind else {
            unreachable!()
        };
        if !html(&e.name, "template") {
            return Err(HtmlCaptureError::NotHtmlTemplate);
        }
        if e.template != Association::Unavailable {
            return Err(HtmlCaptureError::AssociationAlreadyEstablished);
        }
        let HtmlCapturedNodeKind::Fragment(fragment) = &self.data.records[f].kind else {
            unreachable!()
        };
        if fragment.host != Association::Unavailable {
            return Err(HtmlCaptureError::FragmentAlreadyHosted);
        }
        let usage = self.admit(0, 3, 0, 0, 0, 0)?;
        let updates = self.edge_depths(f, t)?;
        reserve(&mut self.data.records[t].dependents, 1)?;
        let HtmlCapturedNodeKind::Element(e) = &mut self.data.records[t].kind else {
            unreachable!()
        };
        e.template = Association::Index(f);
        let HtmlCapturedNodeKind::Fragment(frag) = &mut self.data.records[f].kind else {
            unreachable!()
        };
        frag.host = Association::Index(t);
        self.data.records[t].dependents.push(f);
        self.data.usage = usage;
        self.apply_depths(updates);
        Ok(())
    }
    /// Captures an existing shadow root and all its explicit source metadata.
    /// This records producer facts; it does not attach anything to a live host.
    pub fn add_shadow_root(
        &mut self,
        host: &HtmlCaptureBuilderSubject,
        properties: HtmlCapturedShadowProperties,
        registry: HtmlCaptureRegistryInput,
    ) -> Result<HtmlCaptureBuilderSubject, HtmlCaptureError> {
        let h = self.index(host)?;
        self.require(h, HtmlCaptureSubjectKind::Element)?;
        let HtmlCapturedNodeKind::Element(e) = &self.data.records[h].kind else {
            unreachable!()
        };
        if e.shadow != Association::Unavailable {
            return Err(HtmlCaptureError::AssociationAlreadyEstablished);
        }
        let registry = self.registry(registry)?;
        let mut usage = self.admit(
            1,
            4 + usize::from(matches!(registry, Registry::Index(_))),
            0,
            0,
            0,
            0,
        )?;
        let depth = add(self.data.records[h].depth, 1)?;
        self.depth(depth)?;
        reserve(&mut self.data.records[h].dependents, 1)?;
        reserve(&mut self.data.records, 1)?;
        let document = self.data.records[h].document;
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
            depth,
        });
        let HtmlCapturedNodeKind::Element(e) = &mut self.data.records[h].kind else {
            unreachable!()
        };
        e.shadow = Association::Index(i);
        self.data.records[h].dependents.push(i);
        usage.depth = usage.depth.max(depth);
        self.data.usage = usage;
        Ok(HtmlCaptureBuilderSubject {
            capture: self.data.identity.clone(),
            index: i,
        })
    }
    /// Publishes validated immutable records; unavailable facts remain unavailable.
    pub fn finish(self) -> Result<HtmlCapture, HtmlCaptureError> {
        self.all_depths(&[])?;
        Ok(HtmlCapture(Arc::new(self.data)))
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
    /// Source-defined registry lookup: plain fragments and other nodes return null.
    pub(crate) fn lookup_registry_index(self) -> Option<Option<usize>> {
        let registry = match self.kind() {
            HtmlCapturedNodeKind::Document(document) => document.registry,
            HtmlCapturedNodeKind::Element(element) => element.registry,
            HtmlCapturedNodeKind::Fragment(fragment)
                if matches!(fragment.kind, HtmlCapturedFragmentKind::ShadowRoot(_)) =>
            {
                fragment.registry
            }
            _ => Registry::Null,
        };
        match registry {
            Registry::Null => Some(None),
            Registry::Index(index) => Some(Some(index)),
            Registry::Unavailable => None,
        }
    }
    pub fn lookup_registry(self) -> HtmlCapturedRegistry {
        let registry = match self.kind() {
            HtmlCapturedNodeKind::Document(d) => d.registry,
            HtmlCapturedNodeKind::Element(e) => e.registry,
            HtmlCapturedNodeKind::Fragment(f)
                if matches!(f.kind, HtmlCapturedFragmentKind::ShadowRoot(_)) =>
            {
                f.registry
            }
            _ => Registry::Null,
        };
        registry_fact(self.data, registry)
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
            Association::Absent => None,
            Association::Index(i) => Some(HtmlCapturedTemplateContents::Present(
                HtmlCaptureFragmentId(subject_id(self.data, i)),
            )),
            Association::Unavailable => Some(HtmlCapturedTemplateContents::Unavailable),
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
    // Compact internal keys are issued only after actual capture qualification.
    // The consuming session retains this exact capture for their entire lifetime;
    // these indices never stand in for native node or externally visible IDs.
    pub(crate) fn registry_index(
        &self,
        id: &HtmlCaptureRegistryId,
    ) -> Result<usize, HtmlCaptureError> {
        if id.capture != self.0.identity {
            return Err(HtmlCaptureError::ForeignCapture);
        }
        Ok(id.index)
    }
    pub(crate) fn registry_at(
        &self,
        index: usize,
    ) -> Result<HtmlCaptureRegistryId, HtmlCaptureError> {
        if index >= self.0.registries.len() {
            return Err(HtmlCaptureError::ForeignCapture);
        }
        Ok(HtmlCaptureRegistryId {
            capture: self.0.identity.clone(),
            index,
        })
    }
    pub(crate) fn document_index(
        &self,
        document: &HtmlCaptureDocumentId,
    ) -> Result<usize, HtmlCaptureError> {
        self.require(document.subject(), HtmlCaptureSubjectKind::Document)?;
        Ok(document.0.index)
    }
    pub(crate) fn document_at(
        &self,
        index: usize,
    ) -> Result<HtmlCapturedNode<'_>, HtmlCaptureError> {
        let actual = self
            .0
            .records
            .get(index)
            .ok_or(HtmlCaptureError::ForeignCapture)?;
        if actual.kind.subject_kind() != HtmlCaptureSubjectKind::Document {
            return Err(HtmlCaptureError::WrongKind {
                expected: HtmlCaptureSubjectKind::Document,
                actual: actual.kind.subject_kind(),
            });
        }
        Ok(HtmlCapturedNode {
            data: &self.0,
            index,
        })
    }
    pub(crate) fn template_cache_index(
        &self,
        index: usize,
    ) -> Result<CapturedTemplateCacheKey, HtmlCaptureError> {
        let node = self.document_at(index)?;
        let HtmlCapturedNodeKind::Document(document) = node.kind() else {
            unreachable!("qualified document")
        };
        Ok(match document.template_cache {
            Association::Absent => CapturedTemplateCacheKey::Absent,
            Association::Index(index) => CapturedTemplateCacheKey::Document(index),
            Association::Unavailable => CapturedTemplateCacheKey::Unavailable,
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
    pub(crate) fn definition_lookup_cursor<'a>(
        &'a self,
        registry: &'a HtmlCapturedRegistry,
        local: &'a DocumentString,
        is_value: Option<&'a DocumentString>,
    ) -> Result<DefinitionLookupCursor<'a>, HtmlCaptureError> {
        let (id, definitions) = match registry {
            HtmlCapturedRegistry::Null => (None, &[][..]),
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
                (Some(id), definitions)
            }
        };
        Ok(DefinitionLookupCursor {
            registry: id,
            definitions,
            local,
            is_value,
            phase: 0,
            record: 0,
            field: 0,
            offset: None,
            ready: if id.is_none() { Some(None) } else { None },
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
            if let DefinitionLookupStep::Ready(id) = cursor.step() {
                return id.as_ref().map(|id| self.definition(id)).transpose();
            }
        }
    }

    /// Resolves the actual inclusive ordinary ancestor form, without following hosts.
    pub fn nearest_form(
        &self,
        element: &HtmlCaptureElementId,
    ) -> Result<Option<HtmlCaptureElementId>, HtmlCaptureError> {
        self.require(&element.0, HtmlCaptureSubjectKind::Element)?;
        let mut i = element.0.index;
        loop {
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
                    return Err(HtmlCaptureError::MissingFact {
                        subject: Some(subject_id(&self.0, i)),
                        fact: HtmlCaptureFact::Parent,
                    });
                }
            }
        }
    }

    pub(crate) fn shadow_presence(
        &self,
        element: &HtmlCaptureElementId,
    ) -> Result<Option<bool>, HtmlCaptureError> {
        let node = self.node(element.subject())?;
        let HtmlCapturedNodeKind::Element(element) = node.kind() else {
            unreachable!()
        };
        Ok(match element.shadow {
            Association::Absent => Some(false),
            Association::Index(_) => Some(true),
            Association::Unavailable => None,
        })
    }

    /// Determines source shadow-attachment eligibility from captured facts only.
    /// Invalid host names need no unrelated registry or shadow subtree facts.
    pub fn shadow_attachment_status(
        &self,
        element: &HtmlCaptureElementId,
    ) -> Result<HtmlShadowAttachmentStatus, HtmlCaptureError> {
        let node = self.node(&element.0)?;
        let HtmlCapturedNodeKind::Element(e) = node.kind() else {
            unreachable!()
        };
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
        if !namespace(e.name.namespace(), "http://www.w3.org/1999/xhtml")
            || !(custom_name || ordinary_host)
        {
            return Ok(HtmlShadowAttachmentStatus::InvalidHost);
        }
        match e.shadow {
            Association::Index(_) => return Ok(HtmlShadowAttachmentStatus::ExistingRoot),
            Association::Unavailable => {
                return Err(HtmlCaptureError::MissingFact {
                    subject: Some(element.0.clone()),
                    fact: HtmlCaptureFact::ShadowRoot,
                });
            }
            Association::Absent => {}
        }
        let registry = node.lookup_registry();
        if matches!(registry, HtmlCapturedRegistry::Null)
            || (!custom_name && matches!(e.is_value, HtmlCapturedIsValue::Absent))
        {
            return Ok(HtmlShadowAttachmentStatus::Allowed);
        }
        if custom_name
            && let Some(d) =
                self.lookup_definition(&registry, e.name.namespace(), e.name.local_name(), None)?
        {
            return Ok(definition_shadow_status(d));
        }
        let is_value = match &e.is_value {
            HtmlCapturedIsValue::Absent => return Ok(HtmlShadowAttachmentStatus::Allowed),
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
            .map_or(
                HtmlShadowAttachmentStatus::Allowed,
                definition_shadow_status,
            ))
    }
}

/// Source-derived eligibility, independent of the request's declarative permission.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlShadowAttachmentStatus {
    Allowed,
    ExistingRoot,
    InvalidHost,
    DefinitionDisallows,
}
fn definition_shadow_status(d: &HtmlCustomElementDefinition) -> HtmlShadowAttachmentStatus {
    if d.disable_shadow {
        HtmlShadowAttachmentStatus::DefinitionDisallows
    } else {
        HtmlShadowAttachmentStatus::Allowed
    }
}
/// All four pinned parser modes. Fragment admission accepts requested Inert/Fragment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlParserScriptingMode {
    Normal,
    Disabled,
    Inert,
    Fragment,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFragmentTokenizerState {
    Data,
    Rcdata,
    Rawtext,
    ScriptData,
    Plaintext,
}
/// Initial §13.4 insertion-mode selection; no complete parser state is claimed.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlFragmentInsertionMode {
    InBody,
    InRow,
    InTableBody,
    InCaption,
    InColumnGroup,
    InTable,
    InTemplate,
    InFrameset,
    BeforeHead,
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
}
impl HtmlFragmentContext {
    pub fn element(
        capture: &HtmlCapture,
        target: &HtmlCaptureSubjectId,
    ) -> Result<Self, HtmlCaptureError> {
        let element = capture.element_id(target)?;
        Self::select(
            capture,
            HtmlFragmentTarget::Element(element.clone()),
            element,
        )
    }
    pub fn hosted_fragment(
        capture: &HtmlCapture,
        target: &HtmlCaptureSubjectId,
    ) -> Result<Self, HtmlCaptureError> {
        let fragment = capture.fragment_id(target)?;
        let node = capture.node(target)?;
        let context = match node.host() {
            Some(HtmlCapturedHost::Element(host)) => host,
            Some(HtmlCapturedHost::Absent) => return Err(HtmlCaptureError::UnhostedFragment),
            _ => {
                return Err(HtmlCaptureError::MissingFact {
                    subject: Some(target.clone()),
                    fact: HtmlCaptureFact::FragmentHost,
                });
            }
        };
        Self::select(
            capture,
            HtmlFragmentTarget::HostedFragment(fragment),
            context,
        )
    }
    fn select(
        capture: &HtmlCapture,
        target: HtmlFragmentTarget,
        context: HtmlCaptureElementId,
    ) -> Result<Self, HtmlCaptureError> {
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
            });
        }
        let target_document = target_node.node_document();
        let context_document = capture.node(&context.0)?.node_document();
        let nearest_form = capture.nearest_form(&context)?;
        Ok(Self {
            capture: capture.clone(),
            target,
            context,
            target_document,
            context_document,
            nearest_form,
            root_registry,
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
    fn element_values(&self) -> &HtmlCapturedElement {
        let HtmlCapturedNodeKind::Element(e) = &self.capture.0.records[self.context.0.index].kind
        else {
            unreachable!()
        };
        e
    }
    fn document_values(&self) -> &HtmlCapturedDocument {
        let HtmlCapturedNodeKind::Document(d) =
            &self.capture.0.records[self.context_document.0.index].kind
        else {
            unreachable!()
        };
        d
    }
    pub fn document_mode(&self) -> HtmlDocumentMode {
        self.document_values().mode
    }
    pub fn scripting_status(&self) -> HtmlScriptingStatus {
        self.document_values().scripting
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
        html(&self.element_values().name, "template")
    }
    pub fn is_mathml_text_integration_point(&self) -> bool {
        let name = &self.element_values().name;
        namespace(name.namespace(), "http://www.w3.org/1998/Math/MathML")
            && ["mi", "mo", "mn", "ms", "mtext"]
                .iter()
                .any(|n| exact(name.local_name(), n))
    }
    pub fn is_html_integration_point(&self) -> bool {
        let element = self.element_values();
        let name = &element.name;
        if namespace(name.namespace(), "http://www.w3.org/2000/svg") {
            return ["foreignObject", "desc", "title"]
                .iter()
                .any(|n| exact(name.local_name(), n));
        }
        namespace(name.namespace(), "http://www.w3.org/1998/Math/MathML")
            && exact(name.local_name(), "annotation-xml")
            && element.attributes.iter().any(|a| {
                a.name().namespace().uri().is_none()
                    && a.name().prefix().is_none()
                    && exact(a.name().local_name(), "encoding")
                    && (ascii_case_equal(a.value(), "text/html")
                        || ascii_case_equal(a.value(), "application/xhtml+xml"))
            })
    }
    /// Initial tokenizer choice derives from context identity and effective mode.
    pub fn initial_tokenizer_state(
        &self,
        requested: HtmlParserScriptingMode,
    ) -> Result<HtmlFragmentTokenizerState, HtmlCaptureError> {
        let mode = self.resolve_scripting_mode(requested)?;
        let name = &self.element_values().name;
        if !namespace(name.namespace(), "http://www.w3.org/1999/xhtml") {
            return Ok(HtmlFragmentTokenizerState::Data);
        }
        let local = name.local_name();
        Ok(if ["title", "textarea"].iter().any(|n| exact(local, n)) {
            HtmlFragmentTokenizerState::Rcdata
        } else if ["style", "xmp", "iframe", "noembed", "noframes"]
            .iter()
            .any(|n| exact(local, n))
            || (exact(local, "noscript") && mode != HtmlParserScriptingMode::Disabled)
        {
            HtmlFragmentTokenizerState::Rawtext
        } else if exact(local, "script") {
            HtmlFragmentTokenizerState::ScriptData
        } else if exact(local, "plaintext") {
            HtmlFragmentTokenizerState::Plaintext
        } else {
            HtmlFragmentTokenizerState::Data
        })
    }
    /// Resets initial insertion mode with the single internal root replaced by context.
    /// Captured ancestor templates/tables are not put on the parser's open stack.
    pub fn initial_insertion_mode(&self) -> HtmlFragmentInsertionMode {
        let name = &self.element_values().name;
        if !namespace(name.namespace(), "http://www.w3.org/1999/xhtml") {
            return HtmlFragmentInsertionMode::InBody;
        }
        let local = name.local_name();
        if exact(local, "tr") {
            HtmlFragmentInsertionMode::InRow
        } else if ["tbody", "thead", "tfoot"].iter().any(|n| exact(local, n)) {
            HtmlFragmentInsertionMode::InTableBody
        } else if exact(local, "caption") {
            HtmlFragmentInsertionMode::InCaption
        } else if exact(local, "colgroup") {
            HtmlFragmentInsertionMode::InColumnGroup
        } else if exact(local, "table") {
            HtmlFragmentInsertionMode::InTable
        } else if exact(local, "template") {
            HtmlFragmentInsertionMode::InTemplate
        } else if exact(local, "frameset") {
            HtmlFragmentInsertionMode::InFrameset
        } else if exact(local, "html") {
            HtmlFragmentInsertionMode::BeforeHead
        } else {
            HtmlFragmentInsertionMode::InBody
        }
    }
}
fn ascii_case_equal(value: &DocumentString, literal: &str) -> bool {
    value
        .code_units()
        .iter()
        .map(|&u| {
            if (0x41..=0x5a).contains(&u) {
                u + 0x20
            } else {
                u
            }
        })
        .eq(literal.encode_utf16())
}

/// Allocation-free ordered lookup. Every action is admitted before `step`;
/// comparison headers cost one, each pair two, and Ready transfer one.
pub(crate) struct DefinitionLookupCursor<'a> {
    registry: Option<&'a HtmlCaptureRegistryId>,
    definitions: &'a [HtmlCustomElementDefinition],
    local: &'a DocumentString,
    is_value: Option<&'a DocumentString>,
    phase: u8,
    record: usize,
    field: u8,
    offset: Option<usize>,
    ready: Option<Option<usize>>,
}
pub(crate) enum DefinitionLookupStep {
    Searching,
    Ready(Option<HtmlCaptureDefinitionId>),
}
impl DefinitionLookupCursor<'_> {
    /// Describes the actual next ready transfer without advancing the cursor.
    /// Only a found definition clones a retained registry handle.
    pub(crate) fn next_clones_registry(&self) -> bool {
        matches!(self.ready, Some(Some(_)))
    }
    pub(crate) fn next_cost(&self) -> usize {
        if self.offset.is_some() { 2 } else { 1 }
    }
    fn next_record(&mut self) {
        self.record += 1;
        self.field = 0;
        self.offset = None;
    }
    pub(crate) fn step(&mut self) -> DefinitionLookupStep {
        if let Some(record) = self.ready {
            return DefinitionLookupStep::Ready(record.map(|record_index| {
                HtmlCaptureDefinitionId {
                    registry: self.registry.expect("selected record has registry").clone(),
                    record_index,
                }
            }));
        }
        if self.record == self.definitions.len() {
            if self.phase == 0 && self.is_value.is_some() {
                self.phase = 1;
                self.record = 0;
            } else {
                self.ready = Some(None);
            }
            return DefinitionLookupStep::Searching;
        }
        let d = &self.definitions[self.record];
        let (left, right) = if self.field == 0 {
            (
                &d.name,
                if self.phase == 0 {
                    self.local
                } else {
                    self.is_value.expect("second phase has is")
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
        DefinitionLookupStep::Searching
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
mod definition_cursor_tests {
    use super::*;

    fn capture(
        definitions: Vec<HtmlCustomElementDefinition>,
    ) -> (HtmlCapture, HtmlCapturedRegistry) {
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(0, 0, 0, 1000, 0, 1, 10),
            HtmlHostFactsRevision::fresh(),
        );
        let r = builder.add_registry(Some(definitions)).unwrap();
        let capture = builder.finish().unwrap();
        let registry = HtmlCapturedRegistry::Registry(capture.registry_id(&r).unwrap());
        (capture, registry)
    }
    fn run(cursor: &mut DefinitionLookupCursor<'_>) -> (usize, Option<HtmlCaptureDefinitionId>) {
        let mut work = 0;
        loop {
            work += cursor.next_cost();
            if let DefinitionLookupStep::Ready(id) = cursor.step() {
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
