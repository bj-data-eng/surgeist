//! Owning construction-to-capture import. Only immutable public HTML views are read.
use super::*;
use crate::HtmlSemanticCancellation;
use preflight::ActionPolicy;
use std::collections::HashMap;
use surgeist_html::construction::{
    HtmlDocument, HtmlFragment, HtmlFragmentOwner, HtmlNodeId, HtmlNodeKind,
    HtmlTemplateContentsKind,
};
use surgeist_html::{bounded::construction::HtmlConstructionResult, values::DocumentString};

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
///
/// Importing through the adapter is linear in the nodes and facts: node and fact
/// lookups are indexed and charged one work credit each, array and index growth
/// is charged when it moves entries, and establishing a container's children or
/// attaching template contents links each attached root, charging the records
/// visited finding the root of its parent's component. The adapter attaches
/// containers before their descendants, so that search stays a few steps and the
/// work per node is constant; an arbitrary producer order costs amortized
/// logarithmic work per edge, not quadratic. Each
/// supplied fact and each mapped node holds one scratch entry for the whole
/// import, in addition to the worklist and one container's temporary lists, so
/// scratch is a constant number of entries per node. The node lookup of a
/// successful import is not released but retained by the returned
/// [`HtmlConstructionCaptureMap`]; its entries then stop counting as scratch.
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
    pub(super) mapping_entries: usize,
    pub(super) scratch_entries: usize,
    pub(super) work: usize,
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
    /// The ledger that carries the import's accounting reported a state its own sequencing rules
    /// out: a release that matches nothing held, a refusal that names no import resource, or a
    /// ledger defect. Valid use never reaches it.
    LedgerState,
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
    account: account::ImportAccount,
    cancellation: HtmlSemanticCancellation,
}
impl Control {
    fn poll(&self) -> Result<(), HtmlConstructionCaptureError> {
        if self.cancellation.is_cancelled() {
            Err(HtmlConstructionCaptureError::Cancelled)
        } else {
            Ok(())
        }
    }
    fn usage(&self) -> HtmlConstructionCaptureUsage {
        self.account.usage()
    }
    fn work(&mut self, count: usize) -> Result<(), HtmlConstructionCaptureError> {
        self.poll()?;
        self.account.work(count)
    }
    fn scratch(&mut self, count: usize) -> Result<(), HtmlConstructionCaptureError> {
        self.work(1)?;
        self.account.scratch(count)
    }
    /// The live scratch entries, to give back to later with `release_to`.
    fn mark(&self) -> Result<usize, HtmlConstructionCaptureError> {
        self.account.mark()
    }
    fn release_to(&mut self, mark: usize) -> Result<(), HtmlConstructionCaptureError> {
        self.account.release_to(mark)
    }
    fn reserve<T>(
        &mut self,
        vec: &mut Vec<T>,
        count: usize,
    ) -> Result<(), HtmlConstructionCaptureError> {
        self.scratch(count)?;
        // Only a reservation that must move the existing entries pays for moving
        // them; the amortized growth makes those moves sum to a constant per entry.
        if vec
            .len()
            .checked_add(count)
            .is_none_or(|needed| needed > vec.capacity())
        {
            self.work(vec.len())?;
        }
        vec.try_reserve(count)
            .map_err(|_| HtmlConstructionCaptureError::AllocationFailed)?;
        self.poll()
    }
}
impl preflight::ActionPolicy for Control {
    type Error = HtmlConstructionCaptureError;
    fn on_work(&mut self, units: usize) -> Result<(), Self::Error> {
        self.work(units)
    }
    fn on_reserve(&mut self, entries: usize) -> Result<(), Self::Error> {
        self.scratch(entries)
    }
    fn on_storage_before(&mut self, _requested: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn on_storage_after(
        &mut self,
        _old: usize,
        _actual: usize,
        _retained: bool,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
    fn on_storage_transfer(&mut self, _bytes: usize) -> Result<(), Self::Error> {
        Ok(())
    }
    fn on_element_values(
        &mut self,
        _bytes: usize,
        _units: usize,
        _observed: bool,
    ) -> Result<(), Self::Error> {
        Ok(())
    }
}
/// Position lookup keyed by source node identity.
///
/// The standard randomly keyed hash map is used because `HtmlNodeId` offers
/// equality and hashing but neither an order nor a public dense index. It is
/// only ever probed, never iterated, so no outcome depends on hash seeds or
/// bucket order. During an import each probe is charged one work credit
/// (expected constant); each new entry is admitted as one scratch entry before
/// its allocation, and the rare growth that rehashes the existing entries is
/// charged for them. The node index of a successful import is then moved into
/// the mapping, which keeps it for uncharged `probe`s.
#[derive(Debug)]
struct Positions<K> {
    slots: HashMap<K, usize>,
}
impl<K: Eq + std::hash::Hash> Positions<K> {
    fn new() -> Self {
        Self {
            slots: HashMap::new(),
        }
    }
    fn get(
        &self,
        node: &HtmlNodeId,
        control: &mut Control,
    ) -> Result<Option<usize>, HtmlConstructionCaptureError>
    where
        K: std::borrow::Borrow<HtmlNodeId>,
    {
        control.work(1)?;
        Ok(self.slots.get(node).copied())
    }
    /// Uncharged lookup for the retained mapping, outside any import.
    fn probe(&self, node: &HtmlNodeId) -> Option<usize>
    where
        K: std::borrow::Borrow<HtmlNodeId>,
    {
        self.slots.get(node).copied()
    }
    /// Admits and allocates room for one more entry without recording it.
    fn reserve_one(&mut self, control: &mut Control) -> Result<(), HtmlConstructionCaptureError> {
        control.scratch(1)?;
        let len = self.slots.len();
        if len == self.slots.capacity() {
            control.work(len)?;
            self.slots
                .try_reserve(len.max(4))
                .map_err(|_| HtmlConstructionCaptureError::AllocationFailed)?;
        }
        control.poll()
    }
    /// Records an entry for which `reserve_one` succeeded; never allocates.
    fn insert_reserved(&mut self, node: K, position: usize) {
        self.slots.insert(node, position);
    }
}
/// Test-only instrument: the entries or probes examined by mapping lookups on
/// this thread. It exists so a test can observe the shape of lookup cost without
/// timing; it is compiled out of every other build.
#[cfg(test)]
mod lookup_steps {
    use std::cell::Cell;
    thread_local! {
        static STEPS: Cell<usize> = const { Cell::new(0) };
    }
    pub(super) fn count() {
        STEPS.with(|steps| steps.set(steps.get() + 1));
    }
    pub(super) fn take() -> usize {
        STEPS.with(|steps| steps.replace(0))
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
///
/// The mapping retains, for every mapped node, one entry (the source node
/// identity and its checked handle) and one slot of a hash index from that
/// identity, a second clone of it, to the entry's position. The index is the
/// import's own node index, moved here when the import succeeds, so
/// `builder_subject` and `subject` take expected constant time and no lookup
/// scans the mapping. Their logical sizes are bounded by the import's
/// mapping-entry limit; the entry list and the index may also hold spare
/// capacity from their growth. The index is only probed,
/// never iterated, so no outcome depends on hash seeds or bucket order. The
/// mapping has no equality, and its `Debug` output lists the entries only.
pub struct HtmlConstructionCaptureMap {
    root: HtmlNodeId,
    revision: HtmlHostFactsRevision,
    capture: HtmlCaptureId,
    entries: Vec<Entry>,
    index: Positions<HtmlNodeId>,
}
impl fmt::Debug for HtmlConstructionCaptureMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("HtmlConstructionCaptureMap")
            .field("root", &self.root)
            .field("revision", &self.revision)
            .field("capture", &self.capture)
            .field("entries", &self.entries)
            .finish_non_exhaustive()
    }
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
        self.entry(node)
            .and_then(|entry| entry.subject.as_ref())
            .ok_or(HtmlConstructionCaptureError::OutsideImportedClosure)
    }
    fn entry(&self, node: &HtmlNodeId) -> Option<&Entry> {
        #[cfg(test)]
        lookup_steps::count();
        self.index
            .probe(node)
            .and_then(|position| self.entries.get(position))
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
        self.control.usage()
    }
    pub fn into_parts(
        self,
    ) -> (
        HtmlCaptureBuilder,
        HtmlConstructionResult,
        HtmlConstructionCaptureMap,
        HtmlConstructionCaptureUsage,
    ) {
        (
            self.builder,
            self.source,
            self.mapping,
            self.control.usage(),
        )
    }
    #[expect(clippy::result_large_err)] // Refusal retains the complete owner without allocating after failure.
    pub fn finish(
        mut self,
    ) -> Result<HtmlConstructionCapturedResult, HtmlConstructionCaptureFinishFailure> {
        let mut scratch = preflight::DepthScratch::new();
        let mark = self.control.mark();
        let result = self
            .builder
            .preflight_finish(&mut scratch, &mut |action| self.control.action(action))
            .map_err(failure)
            .and_then(|()| self.control.poll());
        let released = mark.and_then(|mark| self.control.release_to(mark));
        let synced = self.control.account.sync();
        let result = result.and(released).and(synced);
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
        let account = builder
            .account
            .ready(builder.limits)
            .map_err(HtmlConstructionCaptureError::from)
            .and_then(|account| account.import_account(limits));
        let account = match account {
            Ok(account) => account,
            Err(reason) => {
                return Err(HtmlConstructionCaptureFailure {
                    reason,
                    builder,
                    source,
                    facts,
                    usage: HtmlConstructionCaptureUsage::default(),
                });
            }
        };
        let mut control = Control {
            account,
            cancellation,
        };
        let mut mapping = HtmlConstructionCaptureMap {
            root,
            revision: facts.revision.clone(),
            capture: builder.data.identity.clone(),
            entries: Vec::new(),
            index: Positions::new(),
        };
        let result = import(&mut builder, &source, &facts, &mut mapping, &mut control);
        let synced = control.account.sync();
        let result = result.and(synced);
        if let Err(reason) = result {
            builder.data.records.truncate(checkpoint.0);
            // A refused give-back can only be a ledger defect, which stays latched in the
            // operation and fails the builder's next charge; the import's own refusal is the one
            // reported.
            let _ = builder.roll_back_usage(checkpoint.1);
            return Err(HtmlConstructionCaptureFailure {
                reason,
                builder,
                source,
                facts,
                usage: control.usage(),
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

fn fact<'a>(
    facts: &'a HtmlConstructionCaptureFacts,
    index: &Positions<&'a HtmlNodeId>,
    node: &HtmlNodeId,
    control: &mut Control,
) -> Result<&'a HtmlConstructionCaptureNodeFacts, HtmlConstructionCaptureError> {
    index
        .get(node, control)?
        .map(|position| &facts.nodes[position])
        .ok_or(HtmlConstructionCaptureError::MissingNodeDocument)
}
fn reached(
    map: &mut HtmlConstructionCaptureMap,
    index: &mut Positions<HtmlNodeId>,
    node: HtmlNodeId,
    structural: bool,
    control: &mut Control,
) -> Result<usize, HtmlConstructionCaptureError> {
    if let Some(position) = index.get(&node, control)? {
        map.entries[position].structural |= structural;
        return Ok(position);
    }
    control.work(1)?;
    let next = map
        .entries
        .len()
        .checked_add(1)
        .ok_or(HtmlConstructionCaptureError::RepresentationExhausted)?;
    control.account.mapping_entry()?;
    // Only growth moves the existing entries; amortized growth keeps the moves
    // proportional to the final count.
    if map.entries.len() == map.entries.capacity() {
        control.work(map.entries.len())?;
    }
    map.entries
        .try_reserve(1)
        .map_err(|_| HtmlConstructionCaptureError::AllocationFailed)?;
    index.reserve_one(control)?;
    index.insert_reserved(node.clone(), next - 1);
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
fn validate<'a>(
    builder: &HtmlCaptureBuilder,
    source: &HtmlConstructionResult,
    facts: &'a HtmlConstructionCaptureFacts,
    index: &mut Positions<&'a HtmlNodeId>,
    control: &mut Control,
) -> Result<(), HtmlConstructionCaptureError> {
    control.work(1)?;
    if facts.revision != builder.data.host_revision {
        return Err(HtmlConstructionCaptureError::HostRevisionMismatch);
    }
    for (position, entry) in facts.nodes.iter().enumerate() {
        control.work(1)?;
        let node = source
            .node(&entry.node)
            .map_err(|_| HtmlConstructionCaptureError::ForeignResult)?;
        if index.get(&entry.node, control)?.is_some() {
            return Err(HtmlConstructionCaptureError::DuplicateNodeFacts);
        }
        index.reserve_one(control)?;
        index.insert_reserved(&entry.node, position);
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
    let mark = control.mark()?;
    let mut fact_index = Positions::new();
    let mut node_index = Positions::new();
    let result = traverse(
        builder,
        source,
        facts,
        map,
        &mut fact_index,
        &mut node_index,
        control,
    );
    // The fact lookup is scratch of this import only. Release it explicitly,
    // whatever the outcome. The node lookup is released on refusal, so no
    // partial mapping is published; after success it becomes the mapping's
    // retained index. Its entries then stop counting as live scratch. The first
    // refusal is kept.
    drop(fact_index);
    if result.is_ok() {
        map.index = node_index;
    } else {
        drop(node_index);
    }
    let released = control.release_to(mark);
    result.and(released)
}
fn traverse<'a>(
    builder: &mut HtmlCaptureBuilder,
    source: &HtmlConstructionResult,
    facts: &'a HtmlConstructionCaptureFacts,
    map: &mut HtmlConstructionCaptureMap,
    fact_index: &mut Positions<&'a HtmlNodeId>,
    node_index: &mut Positions<HtmlNodeId>,
    control: &mut Control,
) -> Result<(), HtmlConstructionCaptureError> {
    validate(builder, source, facts, fact_index, control)?;
    // Explicit tasks preserve source order without recursion or replaying a path.
    let mut pending = Vec::new();
    control.reserve(&mut pending, 1)?;
    pending.push((map.root.clone(), true));
    while let Some((id, structural)) = pending.pop() {
        control.account.release_scratch(1)?;
        control.work(2)?;
        let index = reached(map, node_index, id.clone(), structural, control)?;
        let node = source
            .node(&id)
            .map_err(|_| HtmlConstructionCaptureError::ForeignResult)?;
        fact(facts, fact_index, &id, control)?;
        if node.is_shadow_contents()
            && fact(facts, fact_index, &id, control)?
                .shadow_facts
                .is_none()
        {
            return Err(HtmlConstructionCaptureError::MissingShadowFacts);
        }
        if let Some(owner) = node.fragment_owner() {
            let (HtmlFragmentOwner::Template(owner) | HtmlFragmentOwner::ShadowHost(owner)) = owner;
            // Only the endpoint is supplemental; do not traverse unrelated owner edges.
            let owner_index = reached(map, node_index, owner.clone(), false, control)?;
            fact(facts, fact_index, &map.entries[owner_index].node, control)?;
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
    // A fact keyed to a node the traversal never reached would be ignored, and a
    // fact keyed to the wrong node never reported. The imported nodes are known
    // now, so each fact must name one of them.
    for entry in &facts.nodes {
        control.work(1)?;
        if node_index.get(&entry.node, control)?.is_none() {
            return Err(HtmlConstructionCaptureError::OutsideImportedClosure);
        }
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
            let observations = fact(facts, fact_index, &map.entries[index].node, control)?;
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
            // The retained record array moves only when its reservation grows it;
            // amortized growth keeps those moves proportional to the final count.
            control.work(1)?;
            if builder.data.records.len() == builder.data.records.capacity() {
                control.work(builder.data.records.len())?;
            }
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
                        let host = node_index
                            .get(host, control)?
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
            let mark = control.mark()?;
            let mut children = Vec::new();
            control.reserve(&mut children, node.children().len())?;
            for id in node.children() {
                let i = node_index
                    .get(&id, control)?
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
            control.release_to(mark)?;
        }
        // Associations from supplemental fragment views are still established.
        if let Some(HtmlFragmentOwner::Template(host)) = node.fragment_owner() {
            let host = node_index
                .get(host, control)?
                .and_then(|i| map.entries[i].subject.as_ref())
                .ok_or(HtmlConstructionCaptureError::OutsideImportedClosure)?;
            let mark = control.mark()?;
            builder
                .establish_template_contents_controlled(host, subject, &mut |action| {
                    control.action(action)
                })
                .map_err(failure)?;
            control.release_to(mark)?;
            control.poll()?;
        }
        if entry.structural
            && node.template_contents_kind() == Some(HtmlTemplateContentsKind::ShadowReference)
        {
            let id = node
                .template_contents()
                .ok_or(HtmlConstructionCaptureError::OutsideImportedClosure)?;
            let target = node_index
                .get(&id, control)?
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
    use surgeist_html::{
        construction::{HtmlFragment, HtmlNodeId, HtmlNodeKind, HtmlShadowRootProperties},
        values::{Namespace, OrderedAttributes, QualifiedName},
    };
    /// A fragment of `n` text siblings imported with generous limits.
    fn imported_siblings(n: usize) -> (HtmlConstructionCapturedResult, Vec<HtmlNodeId>) {
        let mut source = HtmlFragment::builder(
            surgeist_html::construction::HtmlConstructionLimits::new(n + 1, n, 0, 4 * n, 4),
        )
        .unwrap();
        let root = source.root();
        for _ in 0..n {
            source.append_text(&root, "x").unwrap();
        }
        let source = source.finish().unwrap();
        let root_id = source.root().id();
        let mut ids = vec![root_id.clone()];
        ids.extend(source.node(&root_id).unwrap().children());
        let revision = HtmlHostFactsRevision::fresh();
        let mut builder = HtmlCaptureBuilder::new(
            HtmlCaptureLimits::new(n + 4, 4 * n + 8, 0, 4 * n + 8, 8, 0, 0, 16 * 1024 * 1024),
            revision.clone(),
        );
        let document = builder
            .add_document(
                surgeist_html::construction::HtmlDocumentMode::Quirks,
                HtmlScriptingStatus::Enabled,
                HtmlCaptureRegistryInput::Unavailable,
            )
            .unwrap();
        let nodes = ids
            .iter()
            .map(|id| {
                HtmlConstructionCaptureNodeFacts::new(
                    id.clone(),
                    HtmlConstructionCaptureDocument::Existing(document.clone()),
                    None,
                    None,
                    None,
                )
            })
            .collect();
        let import = HtmlConstructionCaptureAdapter::fragment(
            builder,
            source,
            HtmlConstructionCaptureFacts::new(revision, nodes),
            HtmlConstructionCaptureLimits::new(n + 1, 8 * n + 64, 100 * n + 1000),
            HtmlSemanticCancellation::new(),
        )
        .unwrap();
        (import.finish().unwrap(), ids)
    }
    #[test]
    fn every_mapped_node_finds_its_own_distinct_subject() {
        let (result, ids) = imported_siblings(50);
        let mut seen = Vec::new();
        for id in &ids {
            let subject = result.mapping().subject(id, result.capture()).unwrap();
            assert!(!seen.contains(&subject));
            seen.push(subject);
        }
        assert_eq!(seen.len(), 51);
    }
    #[test]
    fn unmapped_nodes_and_foreign_captures_are_refused_in_order() {
        let (result, ids) = imported_siblings(8);
        let (other, other_ids) = imported_siblings(8);
        let outside = &other_ids[3];
        lookup_steps::take();
        assert_eq!(
            result.mapping().builder_subject(outside),
            Err(HtmlConstructionCaptureError::OutsideImportedClosure)
        );
        assert!(lookup_steps::take() <= 1, "a refused lookup is one probe");
        assert_eq!(
            result.mapping().subject(outside, result.capture()),
            Err(HtmlConstructionCaptureError::OutsideImportedClosure)
        );
        // The capture is checked first, so a foreign capture is refused even for
        // a node the mapping does not contain.
        assert_eq!(
            result.mapping().subject(outside, other.capture()),
            Err(HtmlConstructionCaptureError::ForeignBuilder)
        );
        assert_eq!(
            result.mapping().subject(&ids[3], other.capture()),
            Err(HtmlConstructionCaptureError::ForeignBuilder)
        );
    }
    #[test]
    fn looking_up_every_mapped_subject_costs_constant_steps_per_node() {
        // Independent oracle: each lookup of one node must cost a bounded number
        // of steps whatever the number of mapped nodes, so n lookups cost O(n);
        // a scan examines up to n entries per lookup, about n^2 / 2 in total.
        // Against the index the counter in `entry` counts calls, not the work
        // inside them, so after the fix this shows only that each lookup is one
        // `entry` call; the quadratic scan it replaced was counted per entry.
        let n = 200;
        let (result, ids) = imported_siblings(n);
        lookup_steps::take();
        for id in &ids {
            result.mapping().builder_subject(id).unwrap();
        }
        let steps = lookup_steps::take();
        assert!(
            steps <= 2 * ids.len(),
            "{} lookups examined {steps} steps",
            ids.len()
        );
    }
    #[test]
    fn every_work_cutoff_rolls_back_new_records_and_retains_input_allocations() {
        let mut source = HtmlFragment::builder(
            surgeist_html::construction::HtmlConstructionLimits::new(20, 40, 0, 100, 20),
        )
        .unwrap();
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
                    surgeist_html::construction::HtmlShadowRootMode::Closed,
                    surgeist_html::construction::HtmlSlotAssignment::Manual,
                    false,
                    true,
                    false,
                ),
            )
            .unwrap();
        source.append_text(&shadow, "inside").unwrap();
        let source = source.finish().unwrap();
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
                HtmlCaptureLimits::new(20, 100, 0, 100, 20, 0, 0, 16 * 1024 * 1024),
                revision.clone(),
            );
            let document = builder
                .add_document(
                    surgeist_html::construction::HtmlDocumentMode::Quirks,
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
