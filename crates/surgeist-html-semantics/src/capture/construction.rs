//! Owning construction-to-capture import. Only immutable public HTML views are read.
use super::*;
use crate::{
    HtmlDocument, HtmlFragment, HtmlFragmentOwner, HtmlNodeId, HtmlNodeKind,
    HtmlSemanticCancellation, HtmlTemplateContentsKind, bounded::HtmlConstructionResult,
};

/// Explicit document assignment at one coherent host revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HtmlConstructionCaptureDocument {
    /// The actual document root of a document import.
    ResultDocument,
    /// An existing document in the supplied capture builder.
    Existing(HtmlCaptureBuilderSubject),
}
/// External document observations, independent of construction topology.
#[derive(Clone, Debug)]
pub struct HtmlConstructionCaptureDocumentFacts {
    scripting: HtmlScriptingStatus,
    registry: HtmlCaptureRegistryInput,
}
impl HtmlConstructionCaptureDocumentFacts {
    pub fn scripting(&self) -> HtmlScriptingStatus {
        self.scripting
    }
    pub fn registry(&self) -> &HtmlCaptureRegistryInput {
        &self.registry
    }
    pub fn new(scripting: HtmlScriptingStatus, registry: HtmlCaptureRegistryInput) -> Self {
        Self {
            scripting,
            registry,
        }
    }
}
/// Saved creation slots, never inferred from content attributes.
#[derive(Clone, Debug)]
pub struct HtmlConstructionCaptureElementFacts {
    is_value: HtmlCapturedIsValue,
    registry: HtmlCaptureRegistryInput,
}
impl HtmlConstructionCaptureElementFacts {
    pub fn is_value(&self) -> &HtmlCapturedIsValue {
        &self.is_value
    }
    pub fn registry(&self) -> &HtmlCaptureRegistryInput {
        &self.registry
    }
    pub fn new(is_value: HtmlCapturedIsValue, registry: HtmlCaptureRegistryInput) -> Self {
        Self { is_value, registry }
    }
}
/// Root-local external history; the five structural properties come from HTML.
#[derive(Clone, Debug)]
pub struct HtmlConstructionCaptureShadowFacts {
    registry: HtmlCaptureRegistryInput,
    declarative: bool,
    available_to_internals: bool,
    keep_registry_null: bool,
}
impl HtmlConstructionCaptureShadowFacts {
    pub fn registry(&self) -> &HtmlCaptureRegistryInput {
        &self.registry
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
    pub fn new(
        registry: HtmlCaptureRegistryInput,
        declarative: bool,
        available_to_internals: bool,
        keep_registry_null: bool,
    ) -> Self {
        Self {
            registry,
            declarative,
            available_to_internals,
            keep_registry_null,
        }
    }
}
/// Observations qualified by the actual immutable source node ID.
#[derive(Clone, Debug)]
pub struct HtmlConstructionCaptureNodeFacts {
    node: HtmlNodeId,
    document: HtmlConstructionCaptureDocument,
    document_facts: Option<HtmlConstructionCaptureDocumentFacts>,
    element_facts: Option<HtmlConstructionCaptureElementFacts>,
    shadow_facts: Option<HtmlConstructionCaptureShadowFacts>,
}
impl HtmlConstructionCaptureNodeFacts {
    pub fn document_facts(&self) -> Option<&HtmlConstructionCaptureDocumentFacts> {
        self.document_facts.as_ref()
    }
    pub fn element_facts(&self) -> Option<&HtmlConstructionCaptureElementFacts> {
        self.element_facts.as_ref()
    }
    pub fn shadow_facts(&self) -> Option<&HtmlConstructionCaptureShadowFacts> {
        self.shadow_facts.as_ref()
    }
    pub fn new(
        node: HtmlNodeId,
        document: HtmlConstructionCaptureDocument,
        document_facts: Option<HtmlConstructionCaptureDocumentFacts>,
        element_facts: Option<HtmlConstructionCaptureElementFacts>,
        shadow_facts: Option<HtmlConstructionCaptureShadowFacts>,
    ) -> Self {
        Self {
            node,
            document,
            document_facts,
            element_facts,
            shadow_facts,
        }
    }
    pub fn node(&self) -> &HtmlNodeId {
        &self.node
    }
    pub fn document(&self) -> &HtmlConstructionCaptureDocument {
        &self.document
    }
}
/// A finite set of producer facts for exactly one host revision.
#[derive(Debug)]
pub struct HtmlConstructionCaptureFacts {
    revision: HtmlHostFactsRevision,
    nodes: Vec<HtmlConstructionCaptureNodeFacts>,
}
impl HtmlConstructionCaptureFacts {
    pub fn new(
        revision: HtmlHostFactsRevision,
        nodes: Vec<HtmlConstructionCaptureNodeFacts>,
    ) -> Self {
        Self { revision, nodes }
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.revision
    }
    pub fn nodes(&self) -> &[HtmlConstructionCaptureNodeFacts] {
        &self.nodes
    }
}
/// Finite adapter allowances, separate from capture retention limits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HtmlConstructionCaptureLimits {
    mapping_entries: usize,
    scratch_entries: usize,
    work: usize,
}
impl HtmlConstructionCaptureLimits {
    pub fn new(mapping_entries: usize, scratch_entries: usize, work: usize) -> Self {
        Self {
            mapping_entries,
            scratch_entries,
            work,
        }
    }
    pub fn mapping_entries(self) -> usize {
        self.mapping_entries
    }
    pub fn scratch_entries(self) -> usize {
        self.scratch_entries
    }
    pub fn work(self) -> usize {
        self.work
    }
}
/// Admitted adapter work and peak simultaneously live logical scratch entries.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HtmlConstructionCaptureUsage {
    mapping_entries: usize,
    scratch_entries: usize,
    work: usize,
}
impl HtmlConstructionCaptureUsage {
    pub fn mapping_entries(self) -> usize {
        self.mapping_entries
    }
    pub fn scratch_entries(self) -> usize {
        self.scratch_entries
    }
    pub fn work(self) -> usize {
        self.work
    }
}
/// The first refusal. Cleanup never replaces it or publishes partial mappings.
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum HtmlConstructionCaptureError {
    ForeignResult,
    HostRevisionMismatch,
    ForeignBuilder,
    DuplicateNodeFacts,
    WrongFactRole,
    MissingNodeDocument,
    MissingShadowFacts,
    OutsideImportedClosure,
    LimitExceeded,
    RepresentationExhausted,
    AllocationFailed,
    Cancelled,
    Capture(HtmlCaptureError),
}
impl fmt::Display for HtmlConstructionCaptureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "construction capture: {self:?}")
    }
}
impl Error for HtmlConstructionCaptureError {}
impl From<HtmlCaptureError> for HtmlConstructionCaptureError {
    fn from(error: HtmlCaptureError) -> Self {
        Self::Capture(error)
    }
}
fn failure(
    error: preflight::Failure<HtmlConstructionCaptureError>,
) -> HtmlConstructionCaptureError {
    match error {
        preflight::Failure::Admission(error) => error,
        preflight::Failure::Capture(error) => error.into(),
        preflight::Failure::AllocationFailed => HtmlConstructionCaptureError::AllocationFailed,
    }
}
#[derive(Debug)]
struct Control {
    limits: HtmlConstructionCaptureLimits,
    usage: HtmlConstructionCaptureUsage,
    cancellation: HtmlSemanticCancellation,
    scratch_live: usize,
}
impl Control {
    fn poll(&self) -> Result<(), HtmlConstructionCaptureError> {
        if self.cancellation.is_cancelled() {
            Err(HtmlConstructionCaptureError::Cancelled)
        } else {
            Ok(())
        }
    }
    fn work(&mut self, count: usize) -> Result<(), HtmlConstructionCaptureError> {
        self.poll()?;
        let next = self
            .usage
            .work
            .checked_add(count)
            .ok_or(HtmlConstructionCaptureError::RepresentationExhausted)?;
        if next > self.limits.work {
            return Err(HtmlConstructionCaptureError::LimitExceeded);
        }
        self.usage.work = next;
        Ok(())
    }
    fn scratch(&mut self, count: usize) -> Result<(), HtmlConstructionCaptureError> {
        self.work(1)?;
        let next = self
            .scratch_live
            .checked_add(count)
            .ok_or(HtmlConstructionCaptureError::RepresentationExhausted)?;
        if next > self.limits.scratch_entries {
            return Err(HtmlConstructionCaptureError::LimitExceeded);
        }
        self.scratch_live = next;
        self.usage.scratch_entries = self.usage.scratch_entries.max(next);
        Ok(())
    }
    fn action(&mut self, action: preflight::Action) -> Result<(), HtmlConstructionCaptureError> {
        match action {
            preflight::Action::Work(work) => self.work(work),
            preflight::Action::Reserve { entries, .. } => self.scratch(entries),
        }
    }
    fn reserve<T>(
        &mut self,
        vec: &mut Vec<T>,
        count: usize,
    ) -> Result<(), HtmlConstructionCaptureError> {
        self.scratch(count)?;
        self.work(vec.len())?;
        vec.try_reserve(count)
            .map_err(|_| HtmlConstructionCaptureError::AllocationFailed)?;
        self.poll()
    }
}
#[derive(Debug)]
struct Entry {
    node: HtmlNodeId,
    subject: Option<HtmlCaptureBuilderSubject>,
    structural: bool,
    expanded: bool,
}
/// Result-qualified mapping; its handles remain builder/capture qualified.
#[derive(Debug)]
pub struct HtmlConstructionCaptureMap {
    root: HtmlNodeId,
    revision: HtmlHostFactsRevision,
    capture: HtmlCaptureId,
    entries: Vec<Entry>,
}
impl HtmlConstructionCaptureMap {
    pub fn root(&self) -> &HtmlNodeId {
        &self.root
    }
    pub fn host_revision(&self) -> &HtmlHostFactsRevision {
        &self.revision
    }
    pub fn builder_subject(
        &self,
        node: &HtmlNodeId,
    ) -> Result<&HtmlCaptureBuilderSubject, HtmlConstructionCaptureError> {
        self.entries
            .iter()
            .find(|entry| &entry.node == node)
            .and_then(|entry| entry.subject.as_ref())
            .ok_or(HtmlConstructionCaptureError::OutsideImportedClosure)
    }
    pub fn subject(
        &self,
        node: &HtmlNodeId,
        capture: &HtmlCapture,
    ) -> Result<HtmlCaptureSubjectId, HtmlConstructionCaptureError> {
        if capture.0.identity != self.capture {
            return Err(HtmlConstructionCaptureError::ForeignBuilder);
        }
        capture
            .subject_id(self.builder_subject(node)?)
            .map_err(Into::into)
    }
}
/// Refused traversal returns the original logical builder and the same inputs.
#[derive(Debug)]
pub struct HtmlConstructionCaptureFailure {
    reason: HtmlConstructionCaptureError,
    builder: HtmlCaptureBuilder,
    source: HtmlConstructionResult,
    facts: HtmlConstructionCaptureFacts,
    usage: HtmlConstructionCaptureUsage,
}
impl HtmlConstructionCaptureFailure {
    pub fn reason(&self) -> &HtmlConstructionCaptureError {
        &self.reason
    }
    pub fn builder(&self) -> &HtmlCaptureBuilder {
        &self.builder
    }
    pub fn source(&self) -> &HtmlConstructionResult {
        &self.source
    }
    pub fn facts(&self) -> &HtmlConstructionCaptureFacts {
        &self.facts
    }
    pub fn usage(&self) -> HtmlConstructionCaptureUsage {
        self.usage
    }
    pub fn into_parts(
        self,
    ) -> (
        HtmlConstructionCaptureError,
        HtmlCaptureBuilder,
        HtmlConstructionResult,
        HtmlConstructionCaptureFacts,
        HtmlConstructionCaptureUsage,
    ) {
        (
            self.reason,
            self.builder,
            self.source,
            self.facts,
            self.usage,
        )
    }
}
/// Completed traversal, still permitting explicit producer fact establishment.
#[derive(Debug)]
pub struct HtmlConstructionCaptureImport {
    builder: HtmlCaptureBuilder,
    source: HtmlConstructionResult,
    mapping: HtmlConstructionCaptureMap,
    control: Control,
}
impl HtmlConstructionCaptureImport {
    pub fn builder(&self) -> &HtmlCaptureBuilder {
        &self.builder
    }
    pub fn builder_mut(&mut self) -> &mut HtmlCaptureBuilder {
        &mut self.builder
    }
    pub fn source(&self) -> &HtmlConstructionResult {
        &self.source
    }
    pub fn mapping(&self) -> &HtmlConstructionCaptureMap {
        &self.mapping
    }
    pub fn usage(&self) -> HtmlConstructionCaptureUsage {
        self.control.usage
    }
    pub fn into_parts(
        self,
    ) -> (
        HtmlCaptureBuilder,
        HtmlConstructionResult,
        HtmlConstructionCaptureMap,
        HtmlConstructionCaptureUsage,
    ) {
        (self.builder, self.source, self.mapping, self.control.usage)
    }
    #[expect(clippy::result_large_err)] // Refusal retains the complete owner without allocating after failure.
    pub fn finish(
        mut self,
    ) -> Result<HtmlConstructionCapturedResult, HtmlConstructionCaptureFinishFailure> {
        let mut scratch = preflight::DepthScratch::new();
        let mark = self.control.scratch_live;
        let result = self
            .builder
            .preflight_finish(&mut scratch, &mut |action| self.control.action(action))
            .map_err(failure)
            .and_then(|()| self.control.poll());
        self.control.scratch_live = mark;
        if let Err(reason) = result {
            return Err(HtmlConstructionCaptureFinishFailure {
                reason,
                import: self,
            });
        }
        // The existing checked handles already contain exactly the immutable
        // identity/index pair used by subject_id. No map copy/allocation is needed.
        Ok(HtmlConstructionCapturedResult {
            capture: self.builder.publish(),
            source: self.source,
            mapping: self.mapping,
        })
    }
}
/// A refused finish retains all caller additions to the current import.
#[derive(Debug)]
pub struct HtmlConstructionCaptureFinishFailure {
    reason: HtmlConstructionCaptureError,
    import: HtmlConstructionCaptureImport,
}
impl HtmlConstructionCaptureFinishFailure {
    pub fn reason(&self) -> &HtmlConstructionCaptureError {
        &self.reason
    }
    pub fn import(&self) -> &HtmlConstructionCaptureImport {
        &self.import
    }
    pub fn into_parts(self) -> (HtmlConstructionCaptureError, HtmlConstructionCaptureImport) {
        (self.reason, self.import)
    }
}
/// Immutable capture together with its original construction result and mapping.
#[derive(Debug)]
pub struct HtmlConstructionCapturedResult {
    capture: HtmlCapture,
    source: HtmlConstructionResult,
    mapping: HtmlConstructionCaptureMap,
}
impl HtmlConstructionCapturedResult {
    pub fn capture(&self) -> &HtmlCapture {
        &self.capture
    }
    pub fn source(&self) -> &HtmlConstructionResult {
        &self.source
    }
    pub fn mapping(&self) -> &HtmlConstructionCaptureMap {
        &self.mapping
    }
    pub fn into_parts(
        self,
    ) -> (
        HtmlCapture,
        HtmlConstructionResult,
        HtmlConstructionCaptureMap,
    ) {
        (self.capture, self.source, self.mapping)
    }
}
/// HTML-owned import; external observations are never inferred from topology.
pub struct HtmlConstructionCaptureAdapter;
impl HtmlConstructionCaptureAdapter {
    #[expect(clippy::result_large_err)] // Refusal returns the original builder, source and facts without boxing.
    pub fn document(
        builder: HtmlCaptureBuilder,
        source: HtmlDocument,
        facts: HtmlConstructionCaptureFacts,
        limits: HtmlConstructionCaptureLimits,
        cancellation: HtmlSemanticCancellation,
    ) -> Result<HtmlConstructionCaptureImport, HtmlConstructionCaptureFailure> {
        Self::run(
            builder,
            HtmlConstructionResult::Document(source),
            facts,
            limits,
            cancellation,
        )
    }
    #[expect(clippy::result_large_err)] // Refusal returns the original builder, source and facts without boxing.
    pub fn fragment(
        builder: HtmlCaptureBuilder,
        source: HtmlFragment,
        facts: HtmlConstructionCaptureFacts,
        limits: HtmlConstructionCaptureLimits,
        cancellation: HtmlSemanticCancellation,
    ) -> Result<HtmlConstructionCaptureImport, HtmlConstructionCaptureFailure> {
        Self::run(
            builder,
            HtmlConstructionResult::Fragment(source),
            facts,
            limits,
            cancellation,
        )
    }
    #[expect(clippy::result_large_err)] // Failure transfers the already owned inputs without allocating.
    fn run(
        mut builder: HtmlCaptureBuilder,
        source: HtmlConstructionResult,
        facts: HtmlConstructionCaptureFacts,
        limits: HtmlConstructionCaptureLimits,
        cancellation: HtmlSemanticCancellation,
    ) -> Result<HtmlConstructionCaptureImport, HtmlConstructionCaptureFailure> {
        let root = match &source {
            HtmlConstructionResult::Document(value) => value.root().id(),
            HtmlConstructionResult::Fragment(value) => value.root().id(),
        };
        let checkpoint = (builder.data.records.len(), builder.data.usage);
        let mut control = Control {
            limits,
            usage: HtmlConstructionCaptureUsage::default(),
            cancellation,
            scratch_live: 0,
        };
        let mut mapping = HtmlConstructionCaptureMap {
            root,
            revision: facts.revision.clone(),
            capture: builder.data.identity.clone(),
            entries: Vec::new(),
        };
        let result = import(&mut builder, &source, &facts, &mut mapping, &mut control);
        if let Err(reason) = result {
            builder.data.records.truncate(checkpoint.0);
            builder.data.usage = checkpoint.1;
            return Err(HtmlConstructionCaptureFailure {
                reason,
                builder,
                source,
                facts,
                usage: control.usage,
            });
        }
        Ok(HtmlConstructionCaptureImport {
            builder,
            source,
            mapping,
            control,
        })
    }
}

fn find(
    map: &HtmlConstructionCaptureMap,
    node: &HtmlNodeId,
    control: &mut Control,
) -> Result<Option<usize>, HtmlConstructionCaptureError> {
    for (index, entry) in map.entries.iter().enumerate() {
        control.work(1)?;
        if &entry.node == node {
            return Ok(Some(index));
        }
    }
    Ok(None)
}
fn fact<'a>(
    facts: &'a HtmlConstructionCaptureFacts,
    node: &HtmlNodeId,
    control: &mut Control,
) -> Result<&'a HtmlConstructionCaptureNodeFacts, HtmlConstructionCaptureError> {
    for entry in &facts.nodes {
        control.work(1)?;
        if &entry.node == node {
            return Ok(entry);
        }
    }
    Err(HtmlConstructionCaptureError::MissingNodeDocument)
}
fn reached(
    map: &mut HtmlConstructionCaptureMap,
    node: HtmlNodeId,
    structural: bool,
    control: &mut Control,
) -> Result<usize, HtmlConstructionCaptureError> {
    if let Some(index) = find(map, &node, control)? {
        map.entries[index].structural |= structural;
        return Ok(index);
    }
    control.work(1)?;
    let next = map
        .entries
        .len()
        .checked_add(1)
        .ok_or(HtmlConstructionCaptureError::RepresentationExhausted)?;
    if next > control.limits.mapping_entries {
        return Err(HtmlConstructionCaptureError::LimitExceeded);
    }
    control.usage.mapping_entries = next;
    control.work(map.entries.len())?;
    map.entries
        .try_reserve(1)
        .map_err(|_| HtmlConstructionCaptureError::AllocationFailed)?;
    control.poll()?;
    map.entries.push(Entry {
        node,
        subject: None,
        structural,
        expanded: false,
    });
    control.poll()?;
    Ok(next - 1)
}
fn registry(
    builder: &HtmlCaptureBuilder,
    input: &HtmlCaptureRegistryInput,
) -> Result<(), HtmlConstructionCaptureError> {
    // Handles cannot be constructed outside this owner; identity qualifies the index.
    if let HtmlCaptureRegistryInput::Registry(handle) = input
        && handle.capture != builder.data.identity
    {
        return Err(HtmlConstructionCaptureError::ForeignBuilder);
    }
    Ok(())
}
fn validate(
    builder: &HtmlCaptureBuilder,
    source: &HtmlConstructionResult,
    facts: &HtmlConstructionCaptureFacts,
    control: &mut Control,
) -> Result<(), HtmlConstructionCaptureError> {
    control.work(1)?;
    if facts.revision != builder.data.host_revision {
        return Err(HtmlConstructionCaptureError::HostRevisionMismatch);
    }
    for (index, entry) in facts.nodes.iter().enumerate() {
        control.work(1)?;
        let node = source
            .node(&entry.node)
            .map_err(|_| HtmlConstructionCaptureError::ForeignResult)?;
        for prior in &facts.nodes[..index] {
            control.work(1)?;
            if prior.node == entry.node {
                return Err(HtmlConstructionCaptureError::DuplicateNodeFacts);
            }
        }
        if (entry.document_facts.is_some() && !matches!(node.kind(), HtmlNodeKind::Document(_)))
            || (entry.element_facts.is_some() && !matches!(node.kind(), HtmlNodeKind::Element(_)))
            || (entry.shadow_facts.is_some() && !node.is_shadow_contents())
        {
            return Err(HtmlConstructionCaptureError::WrongFactRole);
        }
        if matches!(node.kind(), HtmlNodeKind::Document(_))
            && !matches!(
                entry.document,
                HtmlConstructionCaptureDocument::ResultDocument
            )
        {
            return Err(HtmlConstructionCaptureError::WrongFactRole);
        }
        match &entry.document {
            HtmlConstructionCaptureDocument::Existing(handle) => {
                if handle.capture != builder.data.identity {
                    return Err(HtmlConstructionCaptureError::ForeignBuilder);
                }
                builder.require(builder.index(handle)?, HtmlCaptureSubjectKind::Document)?;
            }
            HtmlConstructionCaptureDocument::ResultDocument if source.document().is_none() => {
                return Err(HtmlConstructionCaptureError::WrongFactRole);
            }
            _ => {}
        }
        if let Some(value) = &entry.document_facts {
            registry(builder, &value.registry)?;
        }
        if let Some(value) = &entry.element_facts {
            registry(builder, &value.registry)?;
        }
        if let Some(value) = &entry.shadow_facts {
            registry(builder, &value.registry)?;
        }
    }
    control.poll()
}
fn import(
    builder: &mut HtmlCaptureBuilder,
    source: &HtmlConstructionResult,
    facts: &HtmlConstructionCaptureFacts,
    map: &mut HtmlConstructionCaptureMap,
    control: &mut Control,
) -> Result<(), HtmlConstructionCaptureError> {
    validate(builder, source, facts, control)?;
    // Explicit tasks preserve source order without recursion or replaying a path.
    let mut pending = Vec::new();
    control.reserve(&mut pending, 1)?;
    pending.push((map.root.clone(), true));
    while let Some((id, structural)) = pending.pop() {
        control.scratch_live -= 1;
        control.work(2)?;
        let index = reached(map, id.clone(), structural, control)?;
        let node = source
            .node(&id)
            .map_err(|_| HtmlConstructionCaptureError::ForeignResult)?;
        fact(facts, &id, control)?;
        if node.is_shadow_contents() && fact(facts, &id, control)?.shadow_facts.is_none() {
            return Err(HtmlConstructionCaptureError::MissingShadowFacts);
        }
        if let Some(owner) = node.fragment_owner() {
            let (HtmlFragmentOwner::Template(owner) | HtmlFragmentOwner::ShadowHost(owner)) = owner;
            // Only the endpoint is supplemental; do not traverse unrelated owner edges.
            let owner_index = reached(map, owner.clone(), false, control)?;
            fact(facts, &map.entries[owner_index].node, control)?;
        }
        if !structural || map.entries[index].expanded {
            continue;
        }
        map.entries[index].expanded = true;
        if let Some(shadow) = node.declarative_shadow_root() {
            control.reserve(&mut pending, 1)?;
            pending.push((shadow.contents().clone(), true));
        }
        if let Some(contents) = node.template_contents() {
            control.reserve(&mut pending, 1)?;
            pending.push((contents, true));
        }
        for child in node.children().rev() {
            control.work(1)?;
            control.reserve(&mut pending, 1)?;
            pending.push((child, true));
        }
        control.poll()?;
    }
    // All independent records precede hosted shadow records. Each read/clone is
    // admitted; names/strings/attributes keep their exact shared UTF-16 backing.
    for shadow_pass in [false, true] {
        for index in 0..map.entries.len() {
            control.work(2)?;
            let node = source
                .node(&map.entries[index].node)
                .map_err(|_| HtmlConstructionCaptureError::ForeignResult)?;
            if node.is_shadow_contents() != shadow_pass {
                continue;
            }
            let observations = fact(facts, &map.entries[index].node, control)?;
            let document = match &observations.document {
                HtmlConstructionCaptureDocument::Existing(document) => Some(document.clone()),
                HtmlConstructionCaptureDocument::ResultDocument
                    if matches!(node.kind(), HtmlNodeKind::Document(_)) =>
                {
                    None
                }
                HtmlConstructionCaptureDocument::ResultDocument => Some(
                    map.entries[0]
                        .subject
                        .clone()
                        .ok_or(HtmlConstructionCaptureError::MissingNodeDocument)?,
                ),
            };
            // Existing retained arrays may move during a capture reservation.
            control.work(add(
                builder.data.records.len(),
                builder.data.usage.relationships,
            )?)?;
            let subject = match node.kind() {
                HtmlNodeKind::Document(mode) => {
                    if document.is_some() {
                        return Err(HtmlConstructionCaptureError::WrongFactRole);
                    }
                    let observed = observations
                        .document_facts
                        .as_ref()
                        .ok_or(HtmlConstructionCaptureError::WrongFactRole)?;
                    control.work(4)?;
                    builder.add_document(*mode, observed.scripting, observed.registry.clone())?
                }
                HtmlNodeKind::Element(element) => {
                    let observed = observations
                        .element_facts
                        .as_ref()
                        .ok_or(HtmlConstructionCaptureError::WrongFactRole)?;
                    let mut units = name_units(element.name())?;
                    for attr in element.attributes().iter() {
                        control.work(1)?;
                        units = add(units, add(name_units(attr.name())?, attr.value().len())?)?;
                    }
                    if let HtmlCapturedIsValue::Value(value) = &observed.is_value {
                        units = add(units, value.len())?;
                    }
                    control.work(add(
                        units,
                        add(
                            element
                                .attributes()
                                .len()
                                .checked_mul(8)
                                .ok_or(HtmlConstructionCaptureError::RepresentationExhausted)?,
                            8,
                        )?,
                    )?)?;
                    builder.add_element(
                        document
                            .as_ref()
                            .ok_or(HtmlConstructionCaptureError::MissingNodeDocument)?,
                        element.name().clone(),
                        element.attributes().clone(),
                        observed.is_value.clone(),
                        observed.registry.clone(),
                    )?
                }
                HtmlNodeKind::Fragment if shadow_pass => {
                    let observed = observations
                        .shadow_facts
                        .as_ref()
                        .ok_or(HtmlConstructionCaptureError::MissingShadowFacts)?;
                    let properties = HtmlCapturedShadowProperties::new(
                        node.shadow_properties()
                            .ok_or(HtmlConstructionCaptureError::WrongFactRole)?,
                        observed.declarative,
                        observed.available_to_internals,
                        observed.keep_registry_null,
                    );
                    let document = document
                        .as_ref()
                        .ok_or(HtmlConstructionCaptureError::MissingNodeDocument)?;
                    control.work(10)?;
                    if let Some(HtmlFragmentOwner::ShadowHost(host)) = node.fragment_owner() {
                        let host = find(map, host, control)?
                            .and_then(|i| map.entries[i].subject.as_ref())
                            .ok_or(HtmlConstructionCaptureError::OutsideImportedClosure)?;
                        builder.add_shadow_root(
                            document,
                            host,
                            properties,
                            observed.registry.clone(),
                        )?
                    } else {
                        builder.add_shadow_fragment(
                            document,
                            properties,
                            observed.registry.clone(),
                        )?
                    }
                }
                HtmlNodeKind::Fragment => {
                    control.work(4)?;
                    let subject = builder.add_fragment(
                        document
                            .as_ref()
                            .ok_or(HtmlConstructionCaptureError::MissingNodeDocument)?,
                    )?;
                    control.poll()?;
                    if node.fragment_owner().is_none() {
                        control.work(2)?;
                        builder.establish_unhosted(&subject)?;
                        control.poll()?;
                    }
                    subject
                }
                HtmlNodeKind::Text(value) => {
                    control.work(add(value.len(), 4)?)?;
                    builder.add_text(
                        document
                            .as_ref()
                            .ok_or(HtmlConstructionCaptureError::MissingNodeDocument)?,
                        value.clone(),
                    )?
                }
                HtmlNodeKind::Comment(value) => {
                    control.work(add(value.len(), 4)?)?;
                    builder.add_comment(
                        document
                            .as_ref()
                            .ok_or(HtmlConstructionCaptureError::MissingNodeDocument)?,
                        value.clone(),
                    )?
                }
                HtmlNodeKind::Doctype(value) => {
                    control.work(add(
                        add(
                            value.name().len(),
                            value.public_id().map_or(0, DocumentString::len),
                        )?,
                        add(value.system_id().map_or(0, DocumentString::len), 4)?,
                    )?)?;
                    builder.add_doctype(
                        document
                            .as_ref()
                            .ok_or(HtmlConstructionCaptureError::MissingNodeDocument)?,
                        value.clone(),
                    )?
                }
                HtmlNodeKind::ProcessingInstruction(value) => {
                    control.work(add(add(value.target().len(), value.data().len())?, 4)?)?;
                    builder.add_processing_instruction(
                        document
                            .as_ref()
                            .ok_or(HtmlConstructionCaptureError::MissingNodeDocument)?,
                        value.target().clone(),
                        value.data().clone(),
                    )?
                }
            };
            control.poll()?;
            control.work(1)?;
            map.entries[index].subject = Some(subject);
            control.poll()?;
        }
    }
    for index in 0..map.entries.len() {
        control.work(2)?;
        let entry = &map.entries[index];
        let node = source
            .node(&entry.node)
            .map_err(|_| HtmlConstructionCaptureError::ForeignResult)?;
        let subject = entry
            .subject
            .as_ref()
            .ok_or(HtmlConstructionCaptureError::OutsideImportedClosure)?;
        if entry.structural
            && matches!(
                node.kind(),
                HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment | HtmlNodeKind::Element(_)
            )
        {
            let mark = control.scratch_live;
            let mut children = Vec::new();
            control.reserve(&mut children, node.children().len())?;
            for id in node.children() {
                let i = find(map, &id, control)?
                    .ok_or(HtmlConstructionCaptureError::OutsideImportedClosure)?;
                children.push(
                    map.entries[i]
                        .subject
                        .clone()
                        .ok_or(HtmlConstructionCaptureError::OutsideImportedClosure)?,
                );
            }
            builder
                .establish_children_controlled(subject, &children, &mut |action| {
                    control.action(action)
                })
                .map_err(failure)?;
            control.poll()?;
            drop(children);
            control.scratch_live = mark;
        }
        // Associations from supplemental fragment views are still established.
        if let Some(HtmlFragmentOwner::Template(host)) = node.fragment_owner() {
            let host = find(map, host, control)?
                .and_then(|i| map.entries[i].subject.as_ref())
                .ok_or(HtmlConstructionCaptureError::OutsideImportedClosure)?;
            let mark = control.scratch_live;
            builder
                .establish_template_contents_controlled(host, subject, &mut |action| {
                    control.action(action)
                })
                .map_err(failure)?;
            control.scratch_live = mark;
            control.poll()?;
        }
        if entry.structural
            && node.template_contents_kind() == Some(HtmlTemplateContentsKind::ShadowReference)
        {
            let id = node
                .template_contents()
                .ok_or(HtmlConstructionCaptureError::OutsideImportedClosure)?;
            let target = find(map, &id, control)?
                .and_then(|i| map.entries[i].subject.as_ref())
                .ok_or(HtmlConstructionCaptureError::OutsideImportedClosure)?;
            let HtmlNodeKind::Element(element) = node.kind() else {
                return Err(HtmlConstructionCaptureError::WrongFactRole);
            };
            control.work(add(name_units(element.name())?, 8)?)?;
            builder.establish_template_shadow_reference(subject, target)?;
            control.poll()?;
        }
    }
    control.poll()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn every_work_cutoff_rolls_back_new_records_and_retains_input_allocations() {
        let mut source =
            HtmlFragment::builder(crate::HtmlConstructionLimits::new(20, 40, 0, 100, 20)).unwrap();
        let root = source.root();
        let host = source
            .append_element(
                &root,
                QualifiedName::new(Namespace::html(), None, "div"),
                OrderedAttributes::default(),
            )
            .unwrap();
        let shadow = source
            .attach_declarative_shadow(
                &host,
                HtmlShadowRootProperties::new(
                    crate::HtmlShadowRootMode::Closed,
                    crate::HtmlSlotAssignment::Manual,
                    false,
                    true,
                    false,
                ),
            )
            .unwrap();
        source.append_text(&shadow, "inside").unwrap();
        let source = source.finish();
        let source_id = source.root().id();
        let mut ids = vec![source_id.clone()];
        let mut cursor = 0;
        while cursor < ids.len() {
            let node = source.node(&ids[cursor]).unwrap();
            ids.extend(node.children());
            if let Some(shadow) = node.declarative_shadow_root() {
                ids.push(shadow.contents().clone());
            }
            cursor += 1;
        }
        let make = |work| {
            let revision = HtmlHostFactsRevision::fresh();
            let mut builder = HtmlCaptureBuilder::new(
                HtmlCaptureLimits::new(20, 100, 0, 100, 20, 0, 0),
                revision.clone(),
            );
            let document = builder
                .add_document(
                    crate::HtmlDocumentMode::Quirks,
                    HtmlScriptingStatus::Enabled,
                    HtmlCaptureRegistryInput::Unavailable,
                )
                .unwrap();
            let nodes = ids
                .iter()
                .map(|id| {
                    let node = source.node(id).unwrap();
                    HtmlConstructionCaptureNodeFacts::new(
                        id.clone(),
                        HtmlConstructionCaptureDocument::Existing(document.clone()),
                        None,
                        matches!(node.kind(), HtmlNodeKind::Element(_)).then(|| {
                            HtmlConstructionCaptureElementFacts::new(
                                HtmlCapturedIsValue::Unavailable,
                                HtmlCaptureRegistryInput::Unavailable,
                            )
                        }),
                        node.is_shadow_contents().then(|| {
                            HtmlConstructionCaptureShadowFacts::new(
                                HtmlCaptureRegistryInput::Unavailable,
                                true,
                                false,
                                true,
                            )
                        }),
                    )
                })
                .collect();
            let facts = HtmlConstructionCaptureFacts::new(revision, nodes);
            (
                builder,
                document,
                facts,
                HtmlConstructionCaptureLimits::new(20, 1000, work),
            )
        };
        let (builder, _, facts, limits) = make(100_000);
        let full = HtmlConstructionCaptureAdapter::fragment(
            builder,
            source.clone(),
            facts,
            limits,
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        let total = full.usage().work();
        for cutoff in 0..total {
            let (builder, document, facts, limits) = make(cutoff);
            let usage = builder.usage();
            let identity = builder.data.identity.clone();
            let capacity = builder.data.records.capacity();
            let fact_allocation = facts.nodes.as_ptr();
            let failure = HtmlConstructionCaptureAdapter::fragment(
                builder,
                source.clone(),
                facts,
                limits,
                HtmlSemanticCancellation::new(),
            )
            .unwrap_err();
            assert_eq!(
                failure.reason,
                HtmlConstructionCaptureError::LimitExceeded,
                "cutoff {cutoff}"
            );
            assert_eq!(failure.builder.data.identity, identity);
            assert_eq!(failure.builder.usage(), usage);
            assert_eq!(failure.builder.data.records.len(), 1);
            assert!(failure.builder.data.records.capacity() >= capacity);
            assert_eq!(failure.facts.nodes.as_ptr(), fact_allocation);
            assert_eq!(failure.source.fragment().unwrap().root().id(), source_id);
            let capture = failure.builder.finish().unwrap();
            let id = capture.subject_id(&document).unwrap();
            let HtmlCapturedNodeKind::Document(facts) = capture.node(&id).unwrap().kind() else {
                panic!("document")
            };
            assert_eq!(facts.scripting(), HtmlScriptingStatus::Enabled);
        }
    }
}
