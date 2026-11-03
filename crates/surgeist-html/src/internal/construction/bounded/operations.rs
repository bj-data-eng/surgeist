use super::*;
#[cfg(test)]
mod charges;
mod depth;
mod orders;
use crate::{Attribute, bounded::HtmlValidatedAttributes, internal::values::HTML_NAMESPACE_URI};
use depth::{DepthPlan, owning_parent};

/// Closed generic changes; parser policy and source evidence are not operands.
#[derive(Debug)]
#[non_exhaustive]
pub enum HtmlConstructionCommand {
    /// Creates a detached record of `kind` with no parent. Document and Fragment kinds are refused
    /// with the wrong-node-kind error; use `CreateDocument` and `CreateFragment`.
    Create {
        /// The record to create. Its names, values and attributes count toward the UTF-16 unit and
        /// attribute limits.
        kind: HtmlNodeKind,
    },
    /// Creates exact element values and explicit producer-supplied history in
    /// one record commit. Flags require an actual HTML link expanded name.
    CreateElement {
        /// The exact element name. Creating an HTML `template` element also creates its owned
        /// contents fragment in the same commit.
        name: QualifiedName,
        /// The ordered raw attributes stored on the element.
        attributes: OrderedAttributes,
        /// Link construction flags, supplied by the producer. Only an element named `link` in the
        /// HTML namespace may carry them; otherwise the command is refused with the
        /// wrong-node-kind error.
        link_flags: Option<crate::HtmlLinkFlags>,
    },
    /// Creates a detached Document record.
    CreateDocument {
        /// The compatibility mode the Document starts in.
        mode: HtmlDocumentMode,
    },
    /// Creates a detached document-fragment record.
    CreateFragment,
    /// Reserves capacity for records without creating any, so a later creation need not allocate.
    /// The record count and usage are unchanged.
    ReserveRecords {
        /// The number of records the capacity must hold beyond those already created.
        additional: usize,
    },
    /// Checks, without changing anything, that moving the children of `from` into `to` is
    /// structurally possible. Both must be able to contain children, and `to` must not lie inside
    /// `from`'s subtree.
    CheckChildTransfer {
        /// The record whose children would move.
        from: HtmlBuilderNode,
        /// The record that would receive the children.
        to: HtmlBuilderNode,
    },
    /// Creates a detached fragment that will serve as declarative shadow contents. It has no host
    /// or owner until attached.
    CreateShadowContents {
        /// The mode, slot assignment and flags the shadow contents will carry once attached.
        properties: HtmlShadowRootProperties,
    },
    /// Inserts `node` into `parent`'s ordinary children, first detaching it from any previous
    /// parent. The insertion is refused with the cycle error when `parent` is `node` or lies
    /// inside `node`'s subtree. Only inserting `node` before itself is a no-op that changes
    /// nothing. Every other insertion runs the full ordering and ancestry phases, including
    /// appending a child that is already last or inserting a child before its own next sibling.
    Insert {
        /// The record to insert. Documents and fragments, including template contents and
        /// shadow contents, cannot be moved this way. A `template`
        /// element or a shadow host can, because moving it leaves its association in place.
        node: HtmlBuilderNode,
        /// The new parent. It must be able to contain ordinary children, so a `template` element
        /// and a leaf node are refused.
        parent: HtmlBuilderNode,
        /// The reference sibling before which `node` is placed, which must be a current child of
        /// `parent`. `None` appends after the last child.
        before: Option<HtmlBuilderNode>,
    },
    /// Removes `node` from its ordinary parent. The node must be movable: a Document, a fragment
    /// or an owned association fragment is refused even when it has no parent. A movable node
    /// that has no parent is left unchanged.
    Detach {
        /// The record to detach.
        node: HtmlBuilderNode,
    },
    /// Replaces the payload of an existing record. The node kind is preserved and topology cannot
    /// change; a payload of another kind is refused with the wrong-node-kind error.
    ReplaceContent {
        /// The record whose payload is replaced.
        node: HtmlBuilderNode,
        /// The replacement payload, which must match the record's current kind.
        content: HtmlConstructionContent,
    },
    /// Replaces the complete attribute list of an element.
    ReplaceAttributes {
        /// The element whose attributes are replaced. Another node kind is refused with the
        /// wrong-node-kind error.
        element: HtmlBuilderNode,
        /// The replacement list, already validated for expanded-name uniqueness.
        attributes: HtmlValidatedAttributes,
    },
    /// Attaches `contents` as the declarative shadow root of `host`.
    AttachShadow {
        /// The shadow host, which must be an element without a shadow root.
        host: HtmlBuilderNode,
        /// The unowned shadow contents fragment to attach.
        contents: HtmlBuilderNode,
    },
    /// Attaches `contents` as the shadow root of `host` and turns the `template` element's own
    /// contents into a reference to it, releasing the contents fragment that the template owned.
    AttachTemplateShadow {
        /// The `template` element whose contents become a shadow reference.
        template: HtmlBuilderNode,
        /// The shadow host, which must be an element without a shadow root.
        host: HtmlBuilderNode,
        /// The unowned shadow contents fragment to attach; it must differ from the contents the
        /// template owned.
        contents: HtmlBuilderNode,
    },
    /// Makes the `template` element's contents a reference to the already-created shadow contents
    /// `contents`, releasing the contents fragment that the template owned.
    SetTemplateShadowContents {
        /// The `template` element whose owned contents are released.
        template: HtmlBuilderNode,
        /// The unowned shadow contents fragment to reference; it must differ from the contents the
        /// template owned.
        contents: HtmlBuilderNode,
    },
    /// Makes `contents` the contents fragment that `template` owns, replacing its current
    /// contents. The old fragment, if it was owned, loses its owner. Reassigning the contents
    /// already owned is a no-op.
    ReassignTemplateContents {
        /// The element whose contents are reassigned. It must already carry template contents.
        template: HtmlBuilderNode,
        /// The unowned ordinary fragment that becomes the template's owned contents.
        contents: HtmlBuilderNode,
    },
    /// Sets the compatibility mode of a Document record.
    SetDocumentMode {
        /// The Document whose mode is set.
        document: HtmlBuilderNode,
        /// The new mode.
        mode: HtmlDocumentMode,
    },
    /// Records link construction flags on an element.
    SetLinkFlags {
        /// The element that receives the flags. It must be a `link` element in the HTML namespace.
        element: HtmlBuilderNode,
        /// The flags to store.
        flags: crate::HtmlLinkFlags,
    },
    /// Publishes the tree rooted at `root` as an immutable result. The publication validates every
    /// record and consumes the storage until `HtmlConstructionLease::try_recover` returns it.
    Publish {
        /// The Document or fragment record to publish as the result's root.
        root: HtmlBuilderNode,
    },
}
/// Content replacement preserves node kind and cannot replace topology.
#[derive(Debug)]
pub enum HtmlConstructionContent {
    /// Replaces an element's qualified name. Link history is cleared unless the new name keeps it.
    /// A rename that changes whether the element has template contents is refused with the
    /// invalid template association error.
    ElementName(QualifiedName),
    /// Replaces a text node's UTF-16 content.
    Text(DocumentString),
    /// Replaces a comment node's UTF-16 content.
    Comment(DocumentString),
    /// Replaces a processing instruction's target and data.
    ProcessingInstruction(HtmlProcessingInstruction),
    /// Replaces a doctype's name and identifiers.
    Doctype(HtmlDoctype),
}
impl HtmlConstructionCommand {
    fn operand(&self, index: usize) -> Option<&HtmlBuilderNode> {
        match (self, index) {
            (
                Self::Insert { node, .. }
                | Self::Detach { node }
                | Self::ReplaceContent { node, .. },
                0,
            ) => Some(node),
            (Self::Insert { parent, .. }, 1) => Some(parent),
            (Self::Insert { before, .. }, 2) => before.as_ref(),
            (Self::ReplaceAttributes { element, .. } | Self::SetLinkFlags { element, .. }, 0) => {
                Some(element)
            }
            (Self::AttachShadow { host, .. }, 0) => Some(host),
            (Self::AttachShadow { contents, .. }, 1) => Some(contents),
            (
                Self::AttachTemplateShadow { template, .. }
                | Self::SetTemplateShadowContents { template, .. }
                | Self::ReassignTemplateContents { template, .. },
                0,
            ) => Some(template),
            (Self::AttachTemplateShadow { host, .. }, 1) => Some(host),
            (Self::AttachTemplateShadow { contents, .. }, 2) => Some(contents),
            (
                Self::SetTemplateShadowContents { contents, .. }
                | Self::ReassignTemplateContents { contents, .. },
                1,
            ) => Some(contents),
            (Self::CheckChildTransfer { from, .. }, 0) => Some(from),
            (Self::CheckChildTransfer { to, .. }, 1) => Some(to),
            (Self::SetDocumentMode { document, .. }, 0) => Some(document),
            (Self::Publish { root }, 0) => Some(root),
            _ => None,
        }
    }
    fn creates(&self) -> bool {
        matches!(
            self,
            Self::Create { .. }
                | Self::CreateElement { .. }
                | Self::CreateDocument { .. }
                | Self::CreateFragment
                | Self::CreateShadowContents { .. }
        )
    }
}

/// Fixed next work or the actual bank to reserve/share. Counts exclude allocator internals.
#[derive(Debug)]
pub enum HtmlConstructionActionDescription<'a> {
    /// Creates the shared identity allocation of a new construction owner.
    InitializeIdentity {
        /// The identity allocation's payload size in bytes.
        payload_bytes: usize,
        /// The bytes the action initializes in the new owner's arena.
        initialized_bytes: usize,
        /// The allocation-provider requests the action makes.
        provider_calls: usize,
    },
    /// Prepares the allocation snapshot that the next `Reserve` or `Share` action describes. It
    /// copies a snapshot value and allocates nothing.
    PrepareAllocationReport {
        /// The retained records the action reads or updates.
        record_visits: usize,
        /// The provider requests the action makes.
        provider_calls: usize,
        /// The bytes copied into the snapshot.
        copied_bytes: usize,
    },
    /// Reads retained records, without allocation, to qualify operands, names, order, ancestry,
    /// publication membership or topology.
    Inspect {
        /// The retained records the action reads or updates.
        record_visits: usize,
        /// The bytes of expanded-name text the action compares; zero for record-only inspection.
        inspected_bytes: usize,
    },
    /// Compares one UTF-16 unit of an element's expanded name with its expected spelling.
    Compare {
        /// The bytes the action compares.
        inspected_bytes: usize,
    },
    /// Reserves capacity in the bank that `allocation` describes: the records vector for
    /// `ReserveRecords` and creation commands, or a parent's child-order deque for `Insert`.
    Reserve {
        /// The provider requests that producing the allocation report costs.
        report_provider_calls: usize,
        /// The bank to reserve in, as reported just before the action.
        allocation: &'a HtmlConstructionAllocationSnapshot,
        /// The least length the bank must be able to hold afterwards.
        required_len: usize,
        /// The least capacity that suffices. `perform` always reserves exactly this capacity;
        /// only `perform_with_capacity` can reserve more, and only when `growth` is `Geometric`.
        minimum_capacity: usize,
        /// Whether the minimum is the capacity to reserve (`Exact`) or the consumer may reserve a
        /// larger, amortizing capacity with `perform_with_capacity` (`Geometric`).
        growth: HtmlConstructionGrowth,
    },
    /// Creates the shared arena allocation that publication returns.
    Share {
        /// The provider requests that producing the allocation report costs.
        report_provider_calls: usize,
        /// The allocation that the action creates, reported before it runs.
        allocation: &'a HtmlConstructionAllocationSnapshot,
    },
    /// Creates the shared node identities that the change stores: the identity of each record a
    /// creation command makes (two for a `template` and its contents), or the host, contents or
    /// template identity of an association.
    ShareReferences {
        /// The provider requests, one per identity created.
        provider_calls: usize,
        /// The bytes of identity references copied into the change.
        copied_bytes: usize,
    },
    /// Initializes the child-order bank of a record the command is creating.
    InitializeChildren {
        /// The provider requests the action makes.
        provider_calls: usize,
        /// The bytes copied into the bank's bookkeeping.
        copied_bytes: usize,
        /// The record fields the action writes.
        record_writes: usize,
        /// The ordinal of the record whose child-order bank is initialized.
        position: usize,
        /// The size in bytes of one child-order entry.
        element_size: usize,
        /// The alignment in bytes of one child-order entry.
        element_alignment: usize,
    },
    /// Computed Insert usage, before any child-order growth.
    Preflight {
        /// The usage the command leaves in place, computed before any child-order growth. The
        /// consumer admits the node, edge, attribute, UTF-16 and depth limits against it.
        usage: HtmlConstructionUsage,
    },
    /// Describes the prepaid writes and moves of the command. Performing this action applies no
    /// write: it makes the operation ready, after checking that the planned order edits still
    /// hold. The writes and moves are applied by `HtmlConstructionReadyOperation::commit`, which
    /// follows `HtmlConstructionOperation::try_ready`.
    Commit {
        /// Commands without an earlier usage boundary observe depth here.
        depth_observation: Option<usize>,
        /// Records the commit's depth walk reads.
        record_visits: usize,
        /// The record fields the commit writes.
        record_writes: usize,
        /// The bytes the commit copies.
        copied_bytes: usize,
        /// The bytes the commit initializes.
        initialized_bytes: usize,
        /// The usage the commit leaves in place.
        usage: HtmlConstructionUsage,
    },
}
/// How a reserve action's capacity is selected.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlConstructionGrowth {
    /// `perform` reserves exactly the described minimum capacity.
    Exact,
    /// The described minimum is only the least capacity that suffices. The
    /// consumer admits and requests its own larger, amortizing capacity with
    /// `HtmlConstructionAction::perform_with_capacity`; `perform` still reserves
    /// the minimum.
    Geometric,
}

#[derive(Debug)]
enum Phase {
    Operand(usize),
    ClassifyNamespace,
    NamespaceUnit(usize),
    ClassifyLocal,
    LocalUnit(usize),
    Command,
    NewAttribute(usize),
    OldAttribute(usize),
    /// Walk over `parent`'s child order from both ends, for the moving node's
    /// place in its old parent (`old`) or for the reference sibling in the new
    /// parent. The entries from `front` up to `back` are unexamined; the walk
    /// examines the last of them when `from_back` and the first otherwise,
    /// alternating, so a place costs about twice its distance from the nearer
    /// end.
    Order {
        parent: usize,
        front: usize,
        back: usize,
        old: bool,
        from_back: bool,
    },
    Cycle {
        cursor: Option<usize>,
        target: usize,
        steps: usize,
    },
    /// Publication first resets every record's membership count, then counts
    /// each record's entries in its parent's child list once, then validates
    /// every record in index order and reads its count in constant time.
    PublishReset(usize),
    PublishCount {
        parent: usize,
        offset: usize,
    },
    PublishNode(usize),
    PublishChild {
        parent: usize,
        offset: usize,
    },
    /// Planning walk up the host's owners after the change, which has climbed
    /// `depth` levels from the host.
    Host {
        cursor: usize,
        depth: usize,
        steps: usize,
    },
    /// Planning walk over one moved subtree, which entered `node` at `level`
    /// below the subtree's root.
    Shift {
        slot: usize,
        node: usize,
        level: usize,
        steps: usize,
    },
    /// Scan for the deepest record outside the moved subtrees.
    Settle {
        level: usize,
    },
    Topology,
    PrepareRecordsReport,
    PrepareWrapperReport,
    /// Reserves the records vector; the snapshot is the bank as reported just before.
    ReserveRecords(HtmlConstructionAllocationSnapshot),
    PrepareChildrenReport,
    /// Reserves the new parent's child order; the snapshot is the bank as reported just before.
    ReserveChildren(HtmlConstructionAllocationSnapshot),
    Preflight,
    Layout,
    References,
    Children(usize),
    /// Creates the shared arena; the snapshot is the wrapper as reported just before.
    Share(HtmlConstructionAllocationSnapshot),
    Commit,
    Observing,
    Ready,
    Failed,
}

/// The shared node identities that the reference-sharing action minted. Each variant belongs to
/// the commands that share identities; every other command stages nothing.
#[derive(Debug, Default)]
enum Staged {
    #[default]
    Nothing,
    /// A creation: the template element's identity, stored as the owner of its contents, when the
    /// created element is a template.
    Created { template_owner: Option<HtmlNodeId> },
    /// A shadow attachment: the host, the contents and the contents' owner.
    Shadow {
        host: HtmlNodeId,
        contents: HtmlNodeId,
        owner: HtmlNodeId,
    },
    /// A template contents reassignment: the template, which owns the new contents.
    Template(HtmlNodeId),
}
/// What a ready operation's commit stores beyond the command's own operands, paired with the
/// command by `try_ready`. A creation's template identity is an option because only a template
/// has contents to own; the association is applied by itself, so the commit needs no pairing of
/// the command with it.
#[derive(Debug)]
struct Resolved {
    template_owner: Option<HtmlNodeId>,
    association: Association,
}
/// The identities and releases of the commands that associate fragments.
#[derive(Debug)]
enum Association {
    None,
    AttachShadow {
        host: HtmlNodeId,
        contents: HtmlNodeId,
        owner: HtmlNodeId,
    },
    AttachTemplateShadow {
        host: HtmlNodeId,
        contents: HtmlNodeId,
        owner: HtmlNodeId,
        old_contents: usize,
    },
    SetTemplateShadowContents {
        old_contents: usize,
    },
    ReassignTemplateContents {
        template: HtmlNodeId,
        old_contents: Option<usize>,
    },
}
impl Resolved {
    fn only(association: Association) -> Self {
        Self {
            template_owner: None,
            association,
        }
    }
}
/// Pairs the command with what planning staged and released. Every command is listed, so a new
/// one must say what it stages. A mismatch hands the staged identities back.
fn resolve(
    command: &HtmlConstructionCommand,
    staged: Staged,
    old_contents: Option<usize>,
) -> Result<Resolved, Staged> {
    use HtmlConstructionCommand as Command;
    match command {
        Command::Create { .. } | Command::CreateElement { .. } => match staged {
            Staged::Created { template_owner } => Ok(Resolved {
                template_owner,
                association: Association::None,
            }),
            staged => Err(staged),
        },
        // The other creations mint their record's identity too, but share nothing with it.
        Command::CreateDocument { .. }
        | Command::CreateFragment
        | Command::CreateShadowContents { .. } => match staged {
            Staged::Created {
                template_owner: None,
            } => Ok(Resolved::only(Association::None)),
            staged => Err(staged),
        },
        Command::AttachShadow { .. } => match (staged, old_contents) {
            (
                Staged::Shadow {
                    host,
                    contents,
                    owner,
                },
                None,
            ) => Ok(Resolved::only(Association::AttachShadow {
                host,
                contents,
                owner,
            })),
            (staged, _) => Err(staged),
        },
        Command::AttachTemplateShadow { .. } => match (staged, old_contents) {
            (
                Staged::Shadow {
                    host,
                    contents,
                    owner,
                },
                Some(old_contents),
            ) => Ok(Resolved::only(Association::AttachTemplateShadow {
                host,
                contents,
                owner,
                old_contents,
            })),
            (staged, _) => Err(staged),
        },
        Command::SetTemplateShadowContents { .. } => match (staged, old_contents) {
            (Staged::Nothing, Some(old_contents)) => {
                Ok(Resolved::only(Association::SetTemplateShadowContents {
                    old_contents,
                }))
            }
            (staged, _) => Err(staged),
        },
        Command::ReassignTemplateContents { .. } => match staged {
            Staged::Template(template) => {
                Ok(Resolved::only(Association::ReassignTemplateContents {
                    template,
                    old_contents,
                }))
            }
            // A reassignment to the contents already owned plans nothing and commits nothing.
            Staged::Nothing => Ok(Resolved::only(Association::None)),
            staged => Err(staged),
        },
        Command::ReserveRecords { .. }
        | Command::CheckChildTransfer { .. }
        | Command::Insert { .. }
        | Command::Detach { .. }
        | Command::ReplaceContent { .. }
        | Command::ReplaceAttributes { .. }
        | Command::SetDocumentMode { .. }
        | Command::SetLinkFlags { .. }
        | Command::Publish { .. } => match staged {
            Staged::Nothing => Ok(Resolved::only(Association::None)),
            staged => Err(staged),
        },
    }
}

#[derive(Debug, Default)]
struct Plan {
    positions: [usize; 3],
    template: bool,
    output_document: bool,
    /// How the commit updates the cached depths.
    depth: DepthPlan,
    link: bool,
    local_expected: &'static [u8],
    usage: HtmlConstructionUsage,
    new_units: usize,
    new_attributes: usize,
    old_units: usize,
    old_attributes: usize,
    old_parent: Option<usize>,
    old_position: usize,
    /// The place in the new parent's order, counted without the moving node.
    insertion_position: usize,
    inserts: bool,
    has_before: bool,
    old_contents: Option<usize>,
    no_op: bool,
    record_count: usize,
    copied_bytes: usize,
    initialized_bytes: usize,
    record_visits: usize,
    record_writes: usize,
    reserve_required: usize,
    references: usize,
    /// The identities that the sharing action minted for the commit to store.
    staged: Staged,
    created: Option<HtmlBuilderNode>,
    new_children: [VecDeque<usize>; 2],
    new_children_generations: [u64; 2],
    /// Insert and detach edit the parents' actual child orders in place, in
    /// the atomic commit, after the new parent's capacity admits the push; no
    /// scratch order exists.
    edits_order: bool,
    children_capacity: Option<usize>,
}
#[derive(Debug)]
enum StorageOwner {
    Mutable(HtmlConstructionStorage),
    /// Publication shared the arena. The lease is the weak recovery capability issued with it.
    Shared {
        arena: Arc<Arena>,
        lease: HtmlConstructionLease,
    },
}
impl StorageOwner {
    fn arena(&self) -> &Arena {
        match self {
            Self::Mutable(v) => &v.arena,
            Self::Shared { arena, .. } => arena,
        }
    }
    /// The mutable storage. A published arena has no later mutation, so asking for it then is a
    /// protocol misuse that reports the not-ready error.
    fn mutable(&mut self) -> Result<&mut HtmlConstructionStorage, HtmlConstructionError> {
        match self {
            Self::Mutable(v) => Ok(v),
            Self::Shared { .. } => Err(HtmlConstructionErrorKind::OperationNotReady.into()),
        }
    }
}

/// A pending operation retains its exact storage and command through refusal.
#[derive(Debug)]
pub struct HtmlConstructionOperation {
    owner: StorageOwner,
    command: HtmlConstructionCommand,
    phase: Phase,
    failure: Option<HtmlConstructionError>,
    plan: Plan,
}
/// The storage and command of an abandoned operation, plus retirement evidence for any allocation
/// the operation had already created.
#[derive(Debug)]
pub struct HtmlConstructionAbort {
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
    retired: Option<HtmlConstructionRetired>,
}
impl HtmlConstructionAbort {
    /// Returns the storage, the command and the retirement evidence. The evidence is `Some` only
    /// when the abandoned operation had already published an arena, which this call reclaims.
    pub fn into_parts(
        self,
    ) -> (
        HtmlConstructionStorage,
        HtmlConstructionCommand,
        Option<HtmlConstructionRetired>,
    ) {
        (self.storage, self.command, self.retired)
    }
}
/// An operation whose every action was accepted. `commit` applies it; `abort` abandons it.
#[derive(Debug)]
pub struct HtmlConstructionReadyOperation {
    operation: HtmlConstructionOperation,
    resolved: Resolved,
}
/// The outcome of committing an operation.
#[derive(Debug)]
pub enum HtmlConstructionCommit {
    /// The command changed the storage, which comes back as the sole owner.
    Changed {
        /// The storage after the command, ready for the next command.
        storage: HtmlConstructionStorage,
        /// The handle of the record that the command created. `None` for commands that create
        /// nothing.
        created: Option<HtmlBuilderNode>,
    },
    /// `Publish` completed: the tree is now an immutable result.
    Published {
        /// The published immutable result.
        result: HtmlConstructionResult,
        /// The recovery capability for the published arena. Pass it with the result to
        /// `HtmlConstructionLease::try_recover` to take the storage back.
        lease: HtmlConstructionLease,
    },
}
/// One pending action of an operation. It borrows the operation exclusively; `perform` consumes
/// it.
#[derive(Debug)]
pub struct HtmlConstructionAction<'a> {
    operation: &'a mut HtmlConstructionOperation,
}
/// The observation after an action. It borrows the operation exclusively; `accept` resumes the
/// operation at the phase the action selected.
#[derive(Debug)]
pub struct HtmlConstructionObservation<'a> {
    operation: &'a mut HtmlConstructionOperation,
    /// The phase this accepted observation resumes.
    next: Phase,
    allocation: Option<HtmlConstructionAllocationSnapshot>,
}
/// An action that failed. The operation latched the first failure, so every later poll returns the
/// same cause.
#[derive(Debug)]
pub struct HtmlConstructionActionFailure {
    reason: HtmlConstructionError,
    allocation: Option<HtmlConstructionAllocationSnapshot>,
}
impl HtmlConstructionActionFailure {
    /// The first failure that the operation latched.
    pub fn reason(&self) -> &HtmlConstructionError {
        &self.reason
    }
    /// The snapshot of the allocation that the failed action concerned; `None` when it concerned
    /// none.
    pub fn allocation(&self) -> Option<&HtmlConstructionAllocationSnapshot> {
        self.allocation.as_ref()
    }
    /// Returns the cause and the allocation snapshot, if any.
    pub fn into_parts(
        self,
    ) -> (
        HtmlConstructionError,
        Option<HtmlConstructionAllocationSnapshot>,
    ) {
        (self.reason, self.allocation)
    }
}
impl fmt::Display for HtmlConstructionActionFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "construction action failed: {}", self.reason)
    }
}
impl std::error::Error for HtmlConstructionActionFailure {}
impl<'a> HtmlConstructionObservation<'a> {
    /// The allocation that the performed action created, for the actions that allocate; `None` for
    /// pure inspection.
    pub fn allocation(&self) -> Option<&HtmlConstructionAllocationSnapshot> {
        self.allocation.as_ref()
    }
    /// Accepts the observation, which resumes the operation at the next phase. Skipping it leaves
    /// the operation not-ready.
    pub fn accept(self) {
        self.operation.phase = self.next;
    }
}

impl HtmlConstructionOperation {
    pub(super) fn new(storage: HtmlConstructionStorage, command: HtmlConstructionCommand) -> Self {
        let phase = if command.operand(0).is_some() {
            Phase::Operand(0)
        } else {
            Phase::ClassifyNamespace
        };
        Self {
            owner: StorageOwner::Mutable(storage),
            command,
            phase,
            failure: None,
            plan: Plan::default(),
        }
    }
    fn arena(&self) -> &Arena {
        self.owner.arena()
    }
    /// The pending arena, for scratch writes that are not logical state.
    fn arena_mut(&mut self) -> Result<&mut Arena, HtmlConstructionError> {
        Ok(&mut self.owner.mutable()?.arena)
    }
    /// The least child-order capacity that holds the pushed child.
    fn push_required(&self) -> usize {
        self.arena()
            .records
            .get(self.plan.positions[1])
            .map_or(usize::MAX, |parent| parent.children.len().saturating_add(1))
    }
    fn command(&self) -> &HtmlConstructionCommand {
        &self.command
    }
    fn name(&self) -> Option<&QualifiedName> {
        match self.command() {
            HtmlConstructionCommand::CreateElement { name, .. } => Some(name),
            HtmlConstructionCommand::Create {
                kind: HtmlNodeKind::Element(v),
            } => Some(v.name()),
            HtmlConstructionCommand::ReplaceContent {
                content: HtmlConstructionContent::ElementName(v),
                ..
            } => Some(v),
            HtmlConstructionCommand::SetLinkFlags { .. } => {
                match &self.arena().records[self.plan.positions[0]].kind {
                    HtmlNodeKind::Element(v) => Some(v.name()),
                    _ => None,
                }
            }
            _ => None,
        }
    }
    fn fail<T>(&mut self, error: HtmlConstructionError) -> Result<T, HtmlConstructionError> {
        let latched = self.failure.get_or_insert(error).clone();
        self.phase = Phase::Failed;
        Err(latched)
    }
    /// The next action to describe and perform. `Ok(None)` means the operation is ready. It
    /// returns the latched failure, or the not-ready error while an observation awaits `accept`.
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionAction<'_>>, HtmlConstructionError> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        if matches!(self.phase, Phase::Observing) {
            return Err(HtmlConstructionErrorKind::OperationNotReady.into());
        }
        if matches!(self.phase, Phase::Ready) {
            return Ok(None);
        }
        Ok(Some(HtmlConstructionAction { operation: self }))
    }
    /// Consumes a ready operation. Returns the latched failure, or the not-ready error before the
    /// last action was accepted; either way the operation is handed back unchanged, so the caller
    /// can abort it and recover the storage and command.
    #[expect(
        clippy::result_large_err,
        reason = "a refusal hands the complete owner back without boxing it"
    )]
    pub fn try_ready(
        mut self,
    ) -> Result<HtmlConstructionReadyOperation, HtmlConstructionRefusal<Self>> {
        if let Some(error) = &self.failure {
            let error = error.clone();
            return Err(HtmlConstructionRefusal::new(self, error));
        }
        if !matches!(self.phase, Phase::Ready) {
            return Err(HtmlConstructionRefusal::new(
                self,
                HtmlConstructionErrorKind::OperationNotReady,
            ));
        }
        // Only publication shares the arena, and its last action does. A ready operation whose
        // command and owner disagree, or whose staged identities do not belong to its command,
        // was driven outside the protocol and reports the same not-ready error.
        let publishes = matches!(self.command, HtmlConstructionCommand::Publish { .. });
        let shared = matches!(self.owner, StorageOwner::Shared { .. });
        if publishes != shared {
            return Err(HtmlConstructionRefusal::new(
                self,
                HtmlConstructionErrorKind::OperationNotReady,
            ));
        }
        let staged = std::mem::take(&mut self.plan.staged);
        match resolve(&self.command, staged, self.plan.old_contents) {
            Ok(resolved) => Ok(HtmlConstructionReadyOperation {
                operation: self,
                resolved,
            }),
            Err(staged) => {
                self.plan.staged = staged;
                Err(HtmlConstructionRefusal::new(
                    self,
                    HtmlConstructionErrorKind::OperationNotReady,
                ))
            }
        }
    }
    /// Abandons the operation and returns its storage and command. If a publication had already
    /// created the shared arena, it is reclaimed and the retirement evidence is returned.
    pub fn abort(self) -> HtmlConstructionAbort {
        let (storage, retired) = match self.owner {
            StorageOwner::Mutable(storage) => (storage, None),
            StorageOwner::Shared { arena, lease } => {
                let snapshot = lease.snapshot;
                drop(lease.arena);
                // The publication was never handed out, and the lease holds only a weak
                // reference, so the arena is uniquely owned.
                let arena =
                    Arc::try_unwrap(arena).expect("unescaped publication is uniquely owned");
                (
                    HtmlConstructionStorage { arena },
                    Some(HtmlConstructionRetired { snapshot }),
                )
            }
        };
        HtmlConstructionAbort {
            storage,
            command: self.command,
            retired,
        }
    }
}

impl<'a> HtmlConstructionAction<'a> {
    /// Borrows the exact retained storage only at the computed usage boundary.
    /// Every header or identity read still requires the caller's admission.
    pub fn storage_view(&self) -> Option<&HtmlConstructionStorage> {
        if !matches!(self.operation.phase, Phase::Preflight) {
            return None;
        }
        match &self.operation.owner {
            StorageOwner::Mutable(storage) => Some(storage),
            StorageOwner::Shared { .. } => None,
        }
    }
    /// Describes the work of the pending action. The consumer admits that work and its storage
    /// before calling `perform`, and polls cancellation then.
    pub fn description(&self) -> HtmlConstructionActionDescription<'_> {
        let operation = &self.operation;
        match &operation.phase {
            Phase::NamespaceUnit(_) | Phase::LocalUnit(_) => {
                HtmlConstructionActionDescription::Compare { inspected_bytes: 3 }
            }
            Phase::PrepareRecordsReport => {
                HtmlConstructionActionDescription::PrepareAllocationReport {
                    record_visits: 3,
                    provider_calls: 1,
                    copied_bytes: size_of::<HtmlConstructionAllocationSnapshot>(),
                }
            }
            Phase::PrepareWrapperReport => {
                HtmlConstructionActionDescription::PrepareAllocationReport {
                    record_visits: 2,
                    provider_calls: 1,
                    copied_bytes: size_of::<HtmlConstructionAllocationSnapshot>(),
                }
            }
            Phase::PrepareChildrenReport => {
                HtmlConstructionActionDescription::PrepareAllocationReport {
                    record_visits: 2,
                    provider_calls: 1,
                    copied_bytes: size_of::<HtmlConstructionAllocationSnapshot>(),
                }
            }
            Phase::ReserveChildren(snapshot) => {
                let required = operation.push_required();
                HtmlConstructionActionDescription::Reserve {
                    report_provider_calls: 1,
                    allocation: snapshot,
                    required_len: required,
                    minimum_capacity: required,
                    growth: HtmlConstructionGrowth::Geometric,
                }
            }
            Phase::ReserveRecords(snapshot) => HtmlConstructionActionDescription::Reserve {
                report_provider_calls: 1,
                allocation: snapshot,
                required_len: operation.plan.reserve_required,
                minimum_capacity: operation.plan.reserve_required,
                growth: HtmlConstructionGrowth::Exact,
            },
            Phase::Share(snapshot) => HtmlConstructionActionDescription::Share {
                report_provider_calls: 3,
                allocation: snapshot,
            },
            Phase::References => HtmlConstructionActionDescription::ShareReferences {
                provider_calls: operation.plan.references,
                copied_bytes: operation.plan.references * size_of::<HtmlNodeId>(),
            },
            Phase::Children(offset) => HtmlConstructionActionDescription::InitializeChildren {
                provider_calls: 1,
                copied_bytes: 2 * size_of::<u64>(),
                record_writes: 2,
                position: operation.plan.positions[0] + *offset,
                element_size: size_of::<usize>(),
                element_alignment: align_of::<usize>(),
            },
            Phase::Preflight => HtmlConstructionActionDescription::Preflight {
                usage: operation.plan.usage,
            },
            Phase::Commit => HtmlConstructionActionDescription::Commit {
                depth_observation: (!matches!(
                    operation.command(),
                    HtmlConstructionCommand::Insert { .. }
                ))
                .then_some(operation.plan.usage.depth),
                record_visits: operation.plan.record_visits,
                record_writes: operation.plan.record_writes,
                copied_bytes: operation.plan.copied_bytes,
                initialized_bytes: operation.plan.initialized_bytes,
                usage: operation.plan.usage,
            },
            Phase::NewAttribute(_) => HtmlConstructionActionDescription::Inspect {
                record_visits: 6,
                inspected_bytes: 0,
            },
            Phase::OldAttribute(_) => HtmlConstructionActionDescription::Inspect {
                record_visits: 7,
                inspected_bytes: 0,
            },
            Phase::Operand(_) => HtmlConstructionActionDescription::Inspect {
                record_visits: 2,
                inspected_bytes: 0,
            },
            Phase::PublishNode(_) => HtmlConstructionActionDescription::Inspect {
                record_visits: 24,
                inspected_bytes: 0,
            },
            // One visit to read the record or entry, one to update a count.
            Phase::PublishReset(_) | Phase::PublishCount { .. } => {
                HtmlConstructionActionDescription::Inspect {
                    record_visits: 2,
                    inspected_bytes: 0,
                }
            }
            Phase::Command => HtmlConstructionActionDescription::Inspect {
                record_visits: 32,
                inspected_bytes: 0,
            },
            Phase::Layout | Phase::Topology => HtmlConstructionActionDescription::Inspect {
                record_visits: 12,
                inspected_bytes: 0,
            },
            Phase::Cycle { .. } => HtmlConstructionActionDescription::Inspect {
                record_visits: 12,
                inspected_bytes: 0,
            },
            Phase::PublishChild { .. }
            | Phase::Host { .. }
            | Phase::Shift { .. }
            | Phase::Settle { .. } => HtmlConstructionActionDescription::Inspect {
                record_visits: 2,
                inspected_bytes: 0,
            },
            _ => HtmlConstructionActionDescription::Inspect {
                record_visits: 1,
                inspected_bytes: 0,
            },
        }
    }
    /// Performs a reserve whose described growth is `Geometric` with the
    /// consumer's admitted capacity, which cannot be below the described
    /// minimum. Any other action, or a smaller capacity, latches
    /// `CapacityMismatch` before any provider call.
    pub fn perform_with_capacity(
        self,
        capacity: usize,
    ) -> Result<HtmlConstructionObservation<'a>, HtmlConstructionActionFailure> {
        let operation = self.operation;
        if !matches!(operation.phase, Phase::ReserveChildren(_))
            || capacity < operation.push_required()
        {
            let reason = operation
                .fail::<()>(HtmlConstructionErrorKind::CapacityMismatch.error())
                .unwrap_err();
            return Err(HtmlConstructionActionFailure {
                reason,
                allocation: None,
            });
        }
        operation.plan.children_capacity = Some(capacity);
        Self { operation }.perform()
    }
    /// Performs the action. On success, the observation reports any allocation and must be
    /// accepted before the next action. On failure, the first cause is latched.
    pub fn perform(self) -> Result<HtmlConstructionObservation<'a>, HtmlConstructionActionFailure> {
        let operation = self.operation;
        match operation.advance() {
            Ok((next, allocation)) => {
                operation.phase = Phase::Observing;
                Ok(HtmlConstructionObservation {
                    operation,
                    next,
                    allocation,
                })
            }
            Err((error, allocation)) => {
                let reason = operation.fail::<()>(error).unwrap_err();
                Err(HtmlConstructionActionFailure { reason, allocation })
            }
        }
    }
}

fn add(left: usize, right: usize) -> Result<usize, HtmlConstructionError> {
    left.checked_add(right)
        .ok_or(HtmlConstructionErrorKind::RepresentationExhausted.error())
}
fn subtract(left: usize, right: usize) -> Result<usize, HtmlConstructionError> {
    left.checked_sub(right)
        .ok_or(HtmlConstructionErrorKind::ReleaseUnderflow.error())
}
fn checked_layout(slots: usize, element_size: usize) -> Result<usize, HtmlConstructionError> {
    slots
        .checked_mul(element_size)
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or(HtmlConstructionErrorKind::RepresentationExhausted.error())
}
fn name_units(name: &QualifiedName) -> Result<usize, HtmlConstructionError> {
    add(
        add(
            name.local_name().len(),
            name.prefix().map_or(0, DocumentString::len),
        )?,
        name.namespace().uri().map_or(0, DocumentString::len),
    )
}
pub(super) fn attribute_units(attribute: &Attribute) -> Result<usize, HtmlConstructionError> {
    add(name_units(attribute.name())?, attribute.value().len())
}
fn processing_instruction_units(
    value: &HtmlProcessingInstruction,
) -> Result<usize, HtmlConstructionError> {
    add(value.target().len(), value.data().len())
}
fn doctype_units(value: &HtmlDoctype) -> Result<usize, HtmlConstructionError> {
    add(
        add(
            value.name().len(),
            value.public_id().map_or(0, DocumentString::len),
        )?,
        value.system_id().map_or(0, DocumentString::len),
    )
}
fn leaf_units(kind: &HtmlNodeKind) -> Result<usize, HtmlConstructionError> {
    match kind {
        HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment => Ok(0),
        HtmlNodeKind::Element(value) => name_units(value.name()),
        HtmlNodeKind::Text(value) | HtmlNodeKind::Comment(value) => Ok(value.len()),
        HtmlNodeKind::ProcessingInstruction(value) => processing_instruction_units(value),
        HtmlNodeKind::Doctype(value) => doctype_units(value),
    }
}
/// Attributes and UTF-16 units a `Create` of `kind` commits: the same helpers
/// the bounded operation's plan sums, evaluated in one step so native admission
/// can check the usage before committing it.
pub(in crate::internal::construction) fn created_usage(
    kind: &HtmlNodeKind,
) -> Result<(usize, usize), HtmlConstructionError> {
    let mut units = leaf_units(kind)?;
    let HtmlNodeKind::Element(element) = kind else {
        return Ok((0, units));
    };
    for attribute in element.attributes().iter() {
        units = add(units, attribute_units(attribute)?)?;
    }
    Ok((element.attributes().len(), units))
}
fn structural_parent(arena: &Arena, index: usize) -> Option<usize> {
    arena.records.get(index).and_then(owning_parent)
}
fn ordinary_owned_contents(arena: &Arena, template: usize) -> Result<usize, HtmlConstructionError> {
    let record = &arena.records[template];
    if !matches!(record.kind, HtmlNodeKind::Element(_)) {
        return Err(HtmlConstructionErrorKind::InvalidTemplateAssociation.into());
    }
    let Some(TemplateContents::Owned(contents)) = record.template_contents else {
        return Err(HtmlConstructionErrorKind::InvalidTemplateAssociation.into());
    };
    let target = &arena.records[contents];
    if !matches!(target.kind, HtmlNodeKind::Fragment)
        || target.shadow_properties.is_some()
        || target.parent.is_some()
        || !matches!(&target.owner, Some(HtmlFragmentOwner::Template(owner)) if owner.index == template && Arc::ptr_eq(&owner.identity, &arena.identity))
    {
        return Err(HtmlConstructionErrorKind::InvalidTemplateAssociation.into());
    }
    Ok(contents)
}
fn unowned_fragment(
    arena: &Arena,
    index: usize,
    shadow: bool,
) -> Result<(), HtmlConstructionError> {
    let target = &arena.records[index];
    if !matches!(target.kind, HtmlNodeKind::Fragment)
        || target.parent.is_some()
        || target.shadow_properties.is_some() != shadow
    {
        return Err(if shadow {
            HtmlConstructionErrorKind::InvalidContentsReference.error()
        } else {
            HtmlConstructionErrorKind::InvalidTemplateAssociation.error()
        });
    }
    if target.owner.is_some() {
        return Err(HtmlConstructionErrorKind::AssociationAlreadyOwned.into());
    }
    Ok(())
}

impl HtmlConstructionOperation {
    fn command_phase(&mut self) -> Result<Phase, HtmlConstructionError> {
        use HtmlConstructionCommand as Command;
        let mut plan = std::mem::take(&mut self.plan);
        let result = (|| -> Result<_, HtmlConstructionError> {
            let arena = self.arena();
            plan.usage = arena.usage;
            plan.record_writes = 1;
            plan.copied_bytes = size_of::<HtmlConstructionUsage>();
            let a = plan.positions[0];
            let b = plan.positions[1];
            let c = plan.positions[2];
            match self.command() {
                Command::Create { .. } | Command::CreateElement { .. } => {
                    plan.new_units = match self.command() {
                        Command::Create { kind } => {
                            if matches!(kind, HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment) {
                                return Err(HtmlConstructionErrorKind::WrongNodeKind.into());
                            }
                            if let HtmlNodeKind::Element(element) = kind {
                                plan.new_attributes = element.attributes().len();
                            }
                            leaf_units(kind)?
                        }
                        Command::CreateElement {
                            name,
                            attributes,
                            link_flags,
                        } => {
                            if link_flags.is_some() && !plan.link {
                                return Err(HtmlConstructionErrorKind::WrongNodeKind.into());
                            }
                            plan.new_attributes = attributes.len();
                            name_units(name)?
                        }
                        // The outer arm admits only the two creation commands.
                        _ => return Err(HtmlConstructionErrorKind::OperationNotReady.into()),
                    };
                    plan.record_count = 1 + usize::from(plan.template);
                    plan.positions[0] = arena.records.len();
                    plan.usage.edges = add(plan.usage.edges, usize::from(plan.template))?;
                    plan.usage.depth = plan.usage.depth.max(usize::from(plan.template));
                }
                Command::CheckChildTransfer { .. } => {
                    for index in [a, b] {
                        HtmlConstructionNode { arena, index }.check_container()?;
                    }
                    plan.record_writes = 0;
                    plan.copied_bytes = 0;
                    return Ok(Phase::Cycle {
                        cursor: Some(b),
                        target: a,
                        steps: 0,
                    });
                }
                Command::ReserveRecords { additional } => {
                    plan.reserve_required = add(arena.records.len(), *additional)?;
                    plan.record_writes = 0;
                    plan.copied_bytes = 0;
                    return Ok(if plan.reserve_required > arena.records.capacity() {
                        Phase::PrepareRecordsReport
                    } else {
                        Phase::Commit
                    });
                }
                Command::CreateDocument { .. }
                | Command::CreateFragment
                | Command::CreateShadowContents { .. } => {
                    plan.record_count = 1;
                    plan.positions[0] = arena.records.len();
                }
                Command::Insert { before, .. } => {
                    HtmlConstructionNode { arena, index: b }.check_container()?;
                    HtmlConstructionNode { arena, index: a }.check_detachment()?;
                    if before.is_some() {
                        HtmlConstructionNode { arena, index: c }
                            .check_reference_child_of(HtmlConstructionNode { arena, index: b })?;
                    }
                    plan.old_parent = arena.records[a].parent;
                    plan.inserts = true;
                    plan.has_before = before.is_some();
                    if before.is_some() && c == a {
                        plan.no_op = true;
                    }
                    plan.usage.edges =
                        add(plan.usage.edges, usize::from(plan.old_parent.is_none()))?;
                    plan.insertion_position = arena.records[b].children.len();
                }
                Command::Detach { .. } => {
                    HtmlConstructionNode { arena, index: a }.check_detachment()?;
                    plan.old_parent = arena.records[a].parent;
                    plan.no_op = plan.old_parent.is_none();
                    plan.usage.edges =
                        subtract(plan.usage.edges, usize::from(plan.old_parent.is_some()))?;
                }
                Command::ReplaceContent { content, .. } => {
                    let old = &arena.records[a].kind;
                    match (old, content) {
                        (
                            HtmlNodeKind::Element(element),
                            HtmlConstructionContent::ElementName(name),
                        ) => {
                            if arena.records[a].template_contents.is_some() != plan.template {
                                return Err(
                                    HtmlConstructionErrorKind::InvalidTemplateAssociation.into()
                                );
                            }
                            plan.old_units = name_units(element.name())?;
                            plan.new_units = name_units(name)?;
                        }
                        (HtmlNodeKind::Text(old), HtmlConstructionContent::Text(new))
                        | (HtmlNodeKind::Comment(old), HtmlConstructionContent::Comment(new)) => {
                            plan.old_units = old.len();
                            plan.new_units = new.len();
                        }
                        (
                            HtmlNodeKind::ProcessingInstruction(old),
                            HtmlConstructionContent::ProcessingInstruction(new),
                        ) => {
                            plan.old_units = processing_instruction_units(old)?;
                            plan.new_units = processing_instruction_units(new)?;
                        }
                        (HtmlNodeKind::Doctype(old), HtmlConstructionContent::Doctype(new)) => {
                            plan.old_units = doctype_units(old)?;
                            plan.new_units = doctype_units(new)?;
                        }
                        _ => return Err(HtmlConstructionErrorKind::WrongNodeKind.into()),
                    }
                    // A name replacement assigns the name, and clears the link
                    // history unless the new name keeps it; every other payload
                    // assigns the record's kind.
                    let (writes, bytes) = match content {
                        HtmlConstructionContent::ElementName(_) => {
                            let history = usize::from(!plan.link);
                            (
                                1 + history,
                                size_of::<QualifiedName>()
                                    + history * size_of::<Option<crate::HtmlLinkFlags>>(),
                            )
                        }
                        _ => (1, size_of::<HtmlNodeKind>()),
                    };
                    plan.record_writes = add(plan.record_writes, writes)?;
                    plan.copied_bytes = add(plan.copied_bytes, bytes)?;
                }
                Command::ReplaceAttributes { attributes, .. } => {
                    let HtmlNodeKind::Element(old) = &arena.records[a].kind else {
                        return Err(HtmlConstructionErrorKind::WrongNodeKind.into());
                    };
                    plan.new_attributes = attributes.as_slice().len();
                    plan.old_attributes = old.attributes().len();
                    // The element's attribute collection is the one assignment.
                    plan.record_writes = add(plan.record_writes, 1)?;
                    plan.copied_bytes = add(plan.copied_bytes, size_of::<OrderedAttributes>())?;
                }
                Command::AttachShadow { .. } => {
                    if !matches!(arena.records[a].kind, HtmlNodeKind::Element(_)) {
                        return Err(HtmlConstructionErrorKind::ShadowHostMustBeElement.into());
                    }
                    if arena.records[a].shadow.is_some() {
                        return Err(HtmlConstructionErrorKind::ShadowAlreadyAttached.into());
                    }
                    unowned_fragment(arena, b, true)?;
                    plan.usage.edges = add(plan.usage.edges, 1)?;
                }
                Command::AttachTemplateShadow { .. } => {
                    plan.old_contents = Some(ordinary_owned_contents(arena, a)?);
                    if !matches!(arena.records[b].kind, HtmlNodeKind::Element(_)) {
                        return Err(HtmlConstructionErrorKind::ShadowHostMustBeElement.into());
                    }
                    if arena.records[b].shadow.is_some() {
                        return Err(HtmlConstructionErrorKind::ShadowAlreadyAttached.into());
                    }
                    unowned_fragment(arena, c, true)?;
                    if Some(c) == plan.old_contents {
                        return Err(HtmlConstructionErrorKind::InvalidContentsReference.into());
                    }
                }
                Command::SetTemplateShadowContents { .. } => {
                    plan.old_contents = Some(ordinary_owned_contents(arena, a)?);
                    unowned_fragment(arena, b, true)?;
                    if Some(b) == plan.old_contents {
                        return Err(HtmlConstructionErrorKind::InvalidContentsReference.into());
                    }
                    plan.usage.edges = subtract(plan.usage.edges, 1)?;
                }
                Command::ReassignTemplateContents { .. } => {
                    if !matches!(arena.records[a].kind, HtmlNodeKind::Element(_)) {
                        return Err(HtmlConstructionErrorKind::InvalidTemplateAssociation.into());
                    }
                    match arena.records[a].template_contents {
                        Some(TemplateContents::Owned(old)) => {
                            ordinary_owned_contents(arena, a)?;
                            if old == b {
                                plan.no_op = true;
                            } else {
                                plan.old_contents = Some(old);
                                unowned_fragment(arena, b, false)?;
                            }
                        }
                        Some(TemplateContents::ShadowReference(old)) => {
                            if old == b {
                                return Err(
                                    HtmlConstructionErrorKind::InvalidTemplateAssociation.into()
                                );
                            }
                            unowned_fragment(arena, b, false)?;
                            plan.usage.edges = add(plan.usage.edges, 1)?;
                        }
                        None => {
                            return Err(
                                HtmlConstructionErrorKind::InvalidTemplateAssociation.into()
                            );
                        }
                    }
                }
                Command::SetDocumentMode { .. } => {
                    if !matches!(arena.records[a].kind, HtmlNodeKind::Document(_)) {
                        return Err(HtmlConstructionErrorKind::WrongNodeKind.into());
                    }
                }
                Command::SetLinkFlags { .. } => {
                    if !plan.link {
                        return Err(HtmlConstructionErrorKind::WrongNodeKind.into());
                    }
                }
                Command::Publish { .. } => {
                    if !matches!(
                        arena.records[a].kind,
                        HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment
                    ) || arena.records[a].parent.is_some()
                    {
                        return Err(HtmlConstructionErrorKind::InvalidOutputRoot.into());
                    }
                    plan.output_document =
                        matches!(arena.records[a].kind, HtmlNodeKind::Document(_));
                    return Ok(Phase::PublishReset(0));
                }
            }
            if plan.new_attributes != 0 {
                Ok(Phase::NewAttribute(0))
            } else if plan.old_attributes != 0 {
                Ok(Phase::OldAttribute(0))
            } else {
                self.after_counts(&mut plan)
            }
        })();
        self.plan = plan;
        result
    }

    fn after_counts(&self, plan: &mut Plan) -> Result<Phase, HtmlConstructionError> {
        use HtmlConstructionCommand as Command;
        plan.usage.nodes = add(plan.usage.nodes, plan.record_count)?;
        plan.usage.attributes = add(
            subtract(plan.usage.attributes, plan.old_attributes)?,
            plan.new_attributes,
        )?;
        plan.usage.utf16_units = add(
            subtract(plan.usage.utf16_units, plan.old_units)?,
            plan.new_units,
        )?;
        if plan.no_op {
            plan.usage = self.arena().usage;
            plan.record_writes = 0;
            plan.copied_bytes = 0;
            return Ok(Phase::Commit);
        }
        plan.edits_order = matches!(
            self.command(),
            Command::Insert { .. } | Command::Detach { .. }
        );
        if let Some(parent) = plan.old_parent {
            let remaining = self
                .arena()
                .records
                .get(parent)
                .ok_or(HtmlConstructionErrorKind::UnknownNode)?
                .children
                .len();
            return Ok(Phase::order(parent, remaining, true));
        }
        match self.command() {
            // A detached node with no reference sibling goes last; finding its
            // place requires no walk over the parent's existing children.
            Command::Insert { before: None, .. } => Ok(Phase::Cycle {
                cursor: Some(plan.positions[1]),
                target: plan.positions[0],
                steps: 0,
            }),
            Command::Insert { .. } => {
                let remaining = self
                    .arena()
                    .records
                    .get(plan.positions[1])
                    .ok_or(HtmlConstructionErrorKind::UnknownNode)?
                    .children
                    .len();
                Ok(Phase::order(plan.positions[1], remaining, false))
            }
            Command::AttachShadow { .. } => Ok(Phase::Cycle {
                cursor: Some(plan.positions[0]),
                target: plan.positions[1],
                steps: 0,
            }),
            Command::AttachTemplateShadow { .. } => Ok(Phase::Cycle {
                cursor: Some(plan.positions[1]),
                target: plan.positions[2],
                steps: 0,
            }),
            Command::ReassignTemplateContents { .. } => Ok(Phase::Cycle {
                cursor: Some(plan.positions[0]),
                target: plan.positions[1],
                steps: 0,
            }),
            _ => self.after_topology(plan),
        }
    }
    fn changes_topology(&self) -> bool {
        matches!(
            self.command(),
            HtmlConstructionCommand::Insert { .. }
                | HtmlConstructionCommand::Detach { .. }
                | HtmlConstructionCommand::AttachShadow { .. }
                | HtmlConstructionCommand::AttachTemplateShadow { .. }
                | HtmlConstructionCommand::SetTemplateShadowContents { .. }
                | HtmlConstructionCommand::ReassignTemplateContents { .. }
                | HtmlConstructionCommand::Publish { .. }
        )
    }
    fn after_topology(&self, _plan: &mut Plan) -> Result<Phase, HtmlConstructionError> {
        if self.changes_topology() && !self.arena().records.is_empty() {
            Ok(Phase::Topology)
        } else {
            Ok(Phase::Layout)
        }
    }
    fn before_writes(&self) -> Result<Phase, HtmlConstructionError> {
        Ok(
            if matches!(self.command(), HtmlConstructionCommand::Insert { .. }) {
                Phase::Preflight
            } else {
                Phase::Layout
            },
        )
    }
    fn layout(&mut self) -> Result<Phase, HtmlConstructionError> {
        use HtmlConstructionCommand as Command;
        let mut plan = std::mem::take(&mut self.plan);
        let result = (|| -> Result<_, HtmlConstructionError> {
            match self.command() {
                Command::Insert { .. } | Command::Detach { .. } => {
                    // The parent link, the removed and inserted entries and
                    // every entry shifted, each one write of one entry.
                    let writes = plan.order_writes(self.arena())?;
                    plan.record_writes = add(plan.record_writes, writes)?;
                    plan.copied_bytes = add(
                        plan.copied_bytes,
                        writes
                            .checked_mul(size_of::<usize>())
                            .ok_or(HtmlConstructionErrorKind::RepresentationExhausted)?,
                    )?;
                }
                // Each arm below charges the fields its commit arm assigns, and
                // no others; the usage write is already in the base.
                Command::AttachShadow { .. } => {
                    // The contents' owner and the host's shadow association.
                    plan.references = 3;
                    plan.record_writes = add(plan.record_writes, 2)?;
                    plan.copied_bytes = add(
                        plan.copied_bytes,
                        size_of::<Option<HtmlFragmentOwner>>()
                            + size_of::<Option<ShadowAssociation>>(),
                    )?;
                }
                Command::AttachTemplateShadow { .. } => {
                    // The old contents' owner, the template's contents, the new
                    // contents' owner and the host's shadow association.
                    plan.references = 3;
                    plan.record_writes = add(plan.record_writes, 4)?;
                    plan.copied_bytes = add(
                        plan.copied_bytes,
                        2 * size_of::<Option<HtmlFragmentOwner>>()
                            + size_of::<Option<TemplateContents>>()
                            + size_of::<Option<ShadowAssociation>>(),
                    )?;
                }
                Command::SetTemplateShadowContents { .. } => {
                    // The old contents' owner and the template's contents.
                    plan.record_writes = add(plan.record_writes, 2)?;
                    plan.copied_bytes = add(
                        plan.copied_bytes,
                        size_of::<Option<HtmlFragmentOwner>>()
                            + size_of::<Option<TemplateContents>>(),
                    )?;
                }
                Command::ReassignTemplateContents { .. } => {
                    // The template's contents and the new contents' owner, and
                    // the old contents' owner when the template owned them: a
                    // shadow reference owns nothing to release.
                    let releases = usize::from(plan.old_contents.is_some());
                    plan.references = 1;
                    plan.record_writes = add(plan.record_writes, 2 + releases)?;
                    plan.copied_bytes = add(
                        plan.copied_bytes,
                        (1 + releases) * size_of::<Option<HtmlFragmentOwner>>()
                            + size_of::<Option<TemplateContents>>(),
                    )?;
                }
                Command::SetDocumentMode { .. } => {
                    plan.copied_bytes = add(plan.copied_bytes, size_of::<HtmlNodeKind>())?;
                    plan.record_writes += 1;
                }
                Command::SetLinkFlags { .. } => {
                    plan.copied_bytes =
                        add(plan.copied_bytes, size_of::<Option<crate::HtmlLinkFlags>>())?;
                    plan.record_writes += 1;
                }
                _ => {}
            }
            // The depth cache's visits and writes, for the records it places.
            let (visits, writes) = plan.depth.commit_work()?;
            plan.record_visits = add(plan.record_visits, visits)?;
            plan.record_writes = add(plan.record_writes, writes)?;
            plan.copied_bytes = add(
                plan.copied_bytes,
                writes
                    .checked_mul(size_of::<usize>())
                    .ok_or(HtmlConstructionErrorKind::RepresentationExhausted)?,
            )?;
            if self.command().creates() {
                plan.initialized_bytes = plan
                    .record_count
                    .checked_mul(size_of::<Record>())
                    .ok_or(HtmlConstructionErrorKind::RepresentationExhausted)?;
                // Each record, and the histogram count of its depth, one word.
                let counts = plan
                    .record_count
                    .checked_mul(size_of::<usize>())
                    .ok_or(HtmlConstructionErrorKind::RepresentationExhausted)?;
                plan.copied_bytes = add(add(plan.copied_bytes, plan.initialized_bytes)?, counts)?;
                plan.record_writes = add(plan.record_writes, 2 * plan.record_count)?;
                plan.references = 1 + usize::from(plan.template);
                plan.reserve_required = add(self.arena().records.len(), plan.record_count)?;
                Ok(if plan.reserve_required > self.arena().records.capacity() {
                    Phase::PrepareRecordsReport
                } else {
                    Phase::References
                })
            } else if plan.edits_order && !plan.no_op {
                // Only a push into an order without spare room allocates.
                let grows = plan.inserts && {
                    let target = plan.positions[1];
                    plan.old_parent != Some(target) && {
                        let children = &self
                            .arena()
                            .records
                            .get(target)
                            .ok_or(HtmlConstructionErrorKind::UnknownNode)?
                            .children;
                        children.len() >= children.capacity()
                    }
                };
                Ok(if grows {
                    Phase::PrepareChildrenReport
                } else {
                    Phase::Commit
                })
            } else if matches!(self.command(), Command::Publish { .. }) {
                Ok(Phase::PrepareWrapperReport)
            } else {
                Ok(if plan.references == 0 {
                    Phase::Commit
                } else {
                    Phase::References
                })
            }
        })();
        self.plan = plan;
        result
    }
}

impl HtmlConstructionOperation {
    fn advance(
        &mut self,
    ) -> Result<
        (Phase, Option<HtmlConstructionAllocationSnapshot>),
        (
            HtmlConstructionError,
            Option<HtmlConstructionAllocationSnapshot>,
        ),
    > {
        match self.phase {
            Phase::PrepareRecordsReport => {
                let arena = self.arena();
                let snapshot = HtmlConstructionAllocationSnapshot::vector(
                    &arena.identity,
                    AllocationRole::Records,
                    arena.records_generation,
                    &arena.records,
                )
                .map_err(|error| (error, None))?;
                Ok((Phase::ReserveRecords(snapshot), None))
            }
            Phase::PrepareWrapperReport => {
                let arena = self.arena();
                let generation = arena.wrapper_generation.checked_add(1).ok_or((
                    HtmlConstructionErrorKind::RepresentationExhausted.error(),
                    None,
                ))?;
                let mut snapshot = HtmlConstructionAllocationSnapshot::shared::<Arena>(
                    &arena.identity,
                    AllocationRole::ArenaWrapper,
                    generation,
                );
                snapshot.initialized_len = 0;
                snapshot.capacity = 0;
                snapshot.payload_bytes = 0;
                Ok((Phase::Share(snapshot), None))
            }
            Phase::PrepareChildrenReport => {
                let arena = self.arena();
                let parent = self.plan.positions[1];
                let record = arena
                    .records
                    .get(parent)
                    .ok_or((HtmlConstructionErrorKind::UnknownNode.error(), None))?;
                let snapshot = HtmlConstructionAllocationSnapshot::deque(
                    &arena.identity,
                    AllocationRole::Children(parent),
                    record.children_generation,
                    &record.children,
                )
                .map_err(|error| (error, None))?;
                Ok((Phase::ReserveChildren(snapshot), None))
            }
            Phase::ReserveChildren(_) => self.reserve_children(),
            Phase::ReserveRecords(_) => {
                let required = self.plan.reserve_required;
                let reserve_only = matches!(
                    self.command(),
                    HtmlConstructionCommand::ReserveRecords { .. }
                );
                let storage = self.owner.mutable().map_err(|error| (error, None))?;
                let count = required.checked_sub(storage.arena.records.len()).ok_or((
                    HtmlConstructionErrorKind::RepresentationExhausted.error(),
                    None,
                ))?;
                let changed = required > storage.arena.records.capacity();
                let next_generation = storage
                    .arena
                    .records_generation
                    .checked_add(u64::from(changed))
                    .ok_or((
                        HtmlConstructionErrorKind::RepresentationExhausted.error(),
                        None,
                    ))?;
                checked_layout(required, size_of::<Record>()).map_err(|error| (error, None))?;
                let result = storage.arena.records.try_reserve_exact(count);
                if result.is_ok() {
                    storage.arena.records_generation = next_generation;
                }
                let snapshot = storage.records_snapshot().map_err(|error| (error, None))?;
                match result {
                    Ok(()) => Ok((
                        if reserve_only {
                            Phase::Commit
                        } else {
                            Phase::References
                        },
                        Some(snapshot),
                    )),
                    Err(_) => Err((
                        HtmlConstructionErrorKind::AllocationFailed.error(),
                        Some(snapshot),
                    )),
                }
            }
            Phase::Share(ref snapshot) => {
                let wrapper_generation = snapshot.id.generation;
                let storage = self.owner.mutable().map_err(|error| (error, None))?;
                storage.arena.wrapper_generation = wrapper_generation;
                storage.arena.usage = self.plan.usage;
                // The arena moves into its shared wrapper. The storage keeps an empty arena of the
                // same identity only until the owner is replaced a few statements below, which
                // drops it. Nothing can observe the vacated arena in between: this action holds
                // the only reference to the operation, the code between the swap and the
                // replacement calls no user code and returns no early error, and a failed
                // allocation aborts the process instead of unwinding.
                let vacated = Arena::vacated(&storage.arena.identity);
                let arena = Arc::new(std::mem::replace(&mut storage.arena, vacated));
                let snapshot = HtmlConstructionAllocationSnapshot::shared::<Arena>(
                    &arena.identity,
                    AllocationRole::ArenaWrapper,
                    arena.wrapper_generation,
                );
                let lease = HtmlConstructionLease {
                    arena: Arc::downgrade(&arena),
                    snapshot: snapshot.clone(),
                };
                self.owner = StorageOwner::Shared { arena, lease };
                Ok((Phase::Commit, Some(snapshot)))
            }
            Phase::Children(offset) => {
                let arena = &mut self.owner.mutable().map_err(|error| (error, None))?.arena;
                let generation = arena.temporary_generation.checked_add(1).ok_or((
                    HtmlConstructionErrorKind::RepresentationExhausted.error(),
                    None,
                ))?;
                let snapshot = HtmlConstructionAllocationSnapshot::deque(
                    &arena.identity,
                    AllocationRole::Children(self.plan.positions[0] + offset),
                    generation,
                    &self.plan.new_children[offset],
                )
                .map_err(|error| (error, None))?;
                // A rejected creation must never reissue this bank incarnation.
                arena.temporary_generation = generation;
                self.plan.new_children_generations[offset] = generation;
                Ok((
                    if offset + 1 < self.plan.record_count {
                        Phase::Children(offset + 1)
                    } else {
                        Phase::Commit
                    },
                    Some(snapshot),
                ))
            }
            _ => self
                .inspect()
                .map(|next| (next, None))
                .map_err(|error| (error, None)),
        }
    }
    fn inspect(&mut self) -> Result<Phase, HtmlConstructionError> {
        use HtmlConstructionCommand as Command;
        match self.phase {
            Phase::Operand(index) => {
                let handle = self
                    .command()
                    .operand(index)
                    .ok_or(HtmlConstructionErrorKind::OperationNotReady)?;
                let arena = self.arena();
                if handle.identity.as_ptr() != Arc::as_ptr(&arena.identity) {
                    return Err(HtmlConstructionErrorKind::ForeignBuilder.into());
                }
                if handle.index >= arena.records.len() {
                    return Err(HtmlConstructionErrorKind::UnknownNode.into());
                }
                self.plan.positions[index] = handle.index;
                Ok(if self.command().operand(index + 1).is_some() {
                    Phase::Operand(index + 1)
                } else {
                    Phase::ClassifyNamespace
                })
            }
            Phase::ClassifyNamespace => {
                let Some(name) = self.name() else {
                    return Ok(Phase::Command);
                };
                Ok(
                    if name
                        .namespace()
                        .uri()
                        .is_some_and(|uri| uri.len() == HTML_NAMESPACE_URI.len())
                    {
                        Phase::NamespaceUnit(0)
                    } else {
                        Phase::Command
                    },
                )
            }
            Phase::NamespaceUnit(offset) => {
                const HTML: &[u8] = HTML_NAMESPACE_URI.as_bytes();
                // The classification step found a name whose namespace has the expected length.
                let uri = self
                    .name()
                    .and_then(|name| name.namespace().uri())
                    .ok_or(HtmlConstructionErrorKind::OperationNotReady)?;
                Ok(if uri.code_units()[offset] != u16::from(HTML[offset]) {
                    Phase::Command
                } else if offset + 1 == HTML.len() {
                    Phase::ClassifyLocal
                } else {
                    Phase::NamespaceUnit(offset + 1)
                })
            }
            Phase::ClassifyLocal => {
                let name = self
                    .name()
                    .ok_or(HtmlConstructionErrorKind::OperationNotReady)?;
                self.plan.local_expected = match name.local_name().len() {
                    8 => b"template",
                    4 => b"link",
                    _ => return Ok(Phase::Command),
                };
                Ok(Phase::LocalUnit(0))
            }
            Phase::LocalUnit(offset) => {
                let local = self
                    .name()
                    .ok_or(HtmlConstructionErrorKind::OperationNotReady)?
                    .local_name();
                let expected = self.plan.local_expected;
                if local.code_units()[offset] != u16::from(expected[offset]) {
                    return Ok(Phase::Command);
                }
                if offset + 1 == expected.len() {
                    self.plan.template = expected.len() == 8;
                    self.plan.link = expected.len() == 4;
                    Ok(Phase::Command)
                } else {
                    Ok(Phase::LocalUnit(offset + 1))
                }
            }
            Phase::Command => self.command_phase(),
            Phase::Layout => self.layout(),
            Phase::Topology => self.topology(),
            Phase::Host {
                cursor,
                depth,
                steps,
            } => self.host_step(cursor, depth, steps),
            Phase::Shift {
                slot,
                node,
                level,
                steps,
            } => self.shift_step(slot, node, level, steps),
            Phase::Settle { level } => self.settle_step(level),
            Phase::Order {
                parent,
                front,
                back,
                old,
                from_back,
            } => self.locate_order(parent, front, back, old, from_back),
            Phase::References => {
                let arena = self.arena();
                let p = self.plan.positions;
                if self.command().creates() {
                    let created = HtmlBuilderNode {
                        identity: Arc::downgrade(&arena.identity),
                        index: p[0],
                    };
                    let template_owner = self.plan.template.then(|| arena.id(p[0]));
                    self.plan.created = Some(created);
                    self.plan.staged = Staged::Created { template_owner };
                    return Ok(Phase::Children(0));
                }
                match self.command() {
                    Command::AttachShadow { .. } | Command::AttachTemplateShadow { .. } => {
                        let (host, contents) =
                            if matches!(self.command(), Command::AttachShadow { .. }) {
                                (p[0], p[1])
                            } else {
                                (p[1], p[2])
                            };
                        let host_id = arena.id(host);
                        let contents_id = arena.id(contents);
                        let owner = host_id.clone();
                        self.plan.staged = Staged::Shadow {
                            host: host_id,
                            contents: contents_id,
                            owner,
                        };
                    }
                    Command::ReassignTemplateContents { .. } => {
                        self.plan.staged = Staged::Template(arena.id(p[0]));
                    }
                    // Only the commands above plan reference sharing.
                    _ => return Err(HtmlConstructionErrorKind::OperationNotReady.into()),
                }
                Ok(Phase::Commit)
            }
            Phase::NewAttribute(index) => {
                let attribute = match self.command() {
                    Command::Create {
                        kind: HtmlNodeKind::Element(element),
                    } => &element.attributes().as_slice()[index],
                    Command::CreateElement { attributes, .. } => &attributes.as_slice()[index],
                    Command::ReplaceAttributes { attributes, .. } => &attributes.as_slice()[index],
                    // Only a command with new attributes enters this phase.
                    _ => return Err(HtmlConstructionErrorKind::OperationNotReady.into()),
                };
                self.plan.new_units = add(self.plan.new_units, attribute_units(attribute)?)?;
                if index + 1 < self.plan.new_attributes {
                    Ok(Phase::NewAttribute(index + 1))
                } else if self.plan.old_attributes != 0 {
                    Ok(Phase::OldAttribute(0))
                } else {
                    let mut plan = std::mem::take(&mut self.plan);
                    let result = self.after_counts(&mut plan);
                    self.plan = plan;
                    result
                }
            }
            Phase::OldAttribute(index) => {
                let HtmlNodeKind::Element(element) =
                    &self.arena().records[self.plan.positions[0]].kind
                else {
                    // The command phase checked that the target is an element.
                    return Err(HtmlConstructionErrorKind::OperationNotReady.into());
                };
                self.plan.old_units = add(
                    self.plan.old_units,
                    attribute_units(&element.attributes().as_slice()[index])?,
                )?;
                if index + 1 < self.plan.old_attributes {
                    Ok(Phase::OldAttribute(index + 1))
                } else {
                    let mut plan = std::mem::take(&mut self.plan);
                    let result = self.after_counts(&mut plan);
                    self.plan = plan;
                    result
                }
            }
            Phase::Cycle {
                cursor,
                target,
                steps,
            } => {
                let Some(cursor) = cursor else {
                    let mut plan = std::mem::take(&mut self.plan);
                    let result = self.after_topology(&mut plan);
                    self.plan = plan;
                    return result;
                };
                if cursor == target || steps >= self.arena().records.len() {
                    return Err(HtmlConstructionErrorKind::Cycle.into());
                }
                if steps == 0 && !matches!(self.command(), Command::CheckChildTransfer { .. }) {
                    let record = &self.arena().records[target];
                    let contents = record.template_contents.and_then(|value| match value {
                        TemplateContents::Owned(index) => Some(index),
                        TemplateContents::ShadowReference(_) => None,
                    });
                    let shallow = record.children.is_empty()
                        && record.shadow.is_none()
                        && contents.is_none_or(|index| {
                            self.arena().records[index].children.is_empty()
                                && self.arena().records[index].shadow.is_none()
                        });
                    if shallow {
                        if contents == Some(cursor) {
                            return Err(HtmlConstructionErrorKind::Cycle.into());
                        }
                        let mut plan = std::mem::take(&mut self.plan);
                        let result = self.after_topology(&mut plan);
                        self.plan = plan;
                        return result;
                    }
                }
                Ok(Phase::Cycle {
                    cursor: structural_parent(self.arena(), cursor),
                    target,
                    steps: steps + 1,
                })
            }
            Phase::PublishReset(index) => {
                let arena = self.arena_mut()?;
                if let Some(record) = arena.records.get_mut(index) {
                    record.publish_found = 0;
                    Ok(Phase::PublishReset(index + 1))
                } else {
                    Ok(Phase::PublishCount {
                        parent: 0,
                        offset: 0,
                    })
                }
            }
            Phase::PublishCount { parent, offset } => {
                let arena = self.arena_mut()?;
                let Some(record) = arena.records.get(parent) else {
                    return Ok(Phase::PublishNode(0));
                };
                let Some(&child) = record.children.get(offset) else {
                    return Ok(Phase::PublishCount {
                        parent: parent + 1,
                        offset: 0,
                    });
                };
                // Only an entry naming this list's owner as its parent counts;
                // PublishChild refuses every other entry when it reaches it.
                if let Some(target) = arena.records.get_mut(child)
                    && target.parent == Some(parent)
                {
                    target.publish_found = target.publish_found.saturating_add(1).min(2);
                }
                Ok(Phase::PublishCount {
                    parent,
                    offset: offset + 1,
                })
            }
            Phase::PublishNode(index) => {
                if index == self.arena().records.len() {
                    let mut plan = std::mem::take(&mut self.plan);
                    let result = self.after_topology(&mut plan);
                    self.plan = plan;
                    return result;
                }
                validate_record(self.arena(), index)?;
                Ok(Phase::PublishChild {
                    parent: index,
                    offset: 0,
                })
            }
            Phase::PublishChild { parent, offset } => {
                let record = self
                    .arena()
                    .records
                    .get(parent)
                    .ok_or(HtmlConstructionErrorKind::UnknownNode)?;
                if let Some(&child) = record.children.get(offset) {
                    let target = self
                        .arena()
                        .records
                        .get(child)
                        .ok_or(HtmlConstructionErrorKind::UnknownNode)?;
                    if target.parent != Some(parent)
                        || target.owner.is_some()
                        || matches!(
                            target.kind,
                            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment
                        )
                    {
                        return Err(HtmlConstructionErrorKind::ParentChildConflict.into());
                    }
                    Ok(Phase::PublishChild {
                        parent,
                        offset: offset + 1,
                    })
                } else {
                    // Every entry of the parent's list was counted before this
                    // pass: the record must appear in it exactly once.
                    if record.parent.is_some() && record.publish_found != 1 {
                        return Err(HtmlConstructionErrorKind::ParentChildConflict.into());
                    }
                    Ok(Phase::PublishNode(parent + 1))
                }
            }
            Phase::Preflight => Ok(Phase::Layout),
            Phase::Commit => {
                if self.plan.edits_order && !self.plan.no_op {
                    // The commit's edits must find every entry where planning
                    // did and must not reallocate the parent's child order.
                    self.plan.check_order_edits(self.arena())?;
                }
                Ok(Phase::Ready)
            }
            _ => Err(HtmlConstructionErrorKind::OperationNotReady.into()),
        }
    }
}

fn validate_record(arena: &Arena, index: usize) -> Result<(), HtmlConstructionError> {
    let record = &arena.records[index];
    if let Some(parent) = record.parent {
        let parent = arena
            .records
            .get(parent)
            .ok_or(HtmlConstructionErrorKind::UnknownNode)?;
        if !matches!(
            parent.kind,
            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment | HtmlNodeKind::Element(_)
        ) || parent.template_contents.is_some()
        {
            return Err(HtmlConstructionErrorKind::ParentChildConflict.into());
        }
    }
    if !record.children.is_empty()
        && (!matches!(
            record.kind,
            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment | HtmlNodeKind::Element(_)
        ) || record.template_contents.is_some())
    {
        return Err(HtmlConstructionErrorKind::ParentChildConflict.into());
    }
    if record.shadow_properties.is_some() && !matches!(record.kind, HtmlNodeKind::Fragment) {
        return Err(HtmlConstructionErrorKind::InvalidShadowAssociation.into());
    }
    if let Some(contents) = record.template_contents {
        if !matches!(record.kind, HtmlNodeKind::Element(_)) {
            return Err(HtmlConstructionErrorKind::InvalidTemplateAssociation.into());
        }
        let target = arena
            .records
            .get(contents.position())
            .ok_or(HtmlConstructionErrorKind::UnknownNode)?;
        if !matches!(target.kind, HtmlNodeKind::Fragment) || target.parent.is_some() {
            return Err(HtmlConstructionErrorKind::InvalidTemplateAssociation.into());
        }
        match contents {
            TemplateContents::Owned(_) => {
                if target.shadow_properties.is_some()
                    || !matches!(&target.owner,Some(HtmlFragmentOwner::Template(owner)) if owner.index==index && Arc::ptr_eq(&owner.identity,&arena.identity))
                {
                    return Err(HtmlConstructionErrorKind::InvalidTemplateAssociation.into());
                }
            }
            TemplateContents::ShadowReference(_) => {
                if target.shadow_properties.is_none() {
                    return Err(HtmlConstructionErrorKind::InvalidContentsReference.into());
                }
            }
        }
    }
    if let Some(shadow) = &record.shadow {
        if !matches!(record.kind, HtmlNodeKind::Element(_))
            || shadow.host.index != index
            || !Arc::ptr_eq(&shadow.host.identity, &arena.identity)
            || !Arc::ptr_eq(&shadow.contents.identity, &arena.identity)
        {
            return Err(HtmlConstructionErrorKind::InvalidShadowAssociation.into());
        }
        let target = arena
            .records
            .get(shadow.contents.index)
            .ok_or(HtmlConstructionErrorKind::UnknownNode)?;
        if target.shadow_properties.is_none()
            || target.parent.is_some()
            || !matches!(&target.owner,Some(HtmlFragmentOwner::ShadowHost(owner)) if owner==&shadow.host)
        {
            return Err(HtmlConstructionErrorKind::InvalidShadowAssociation.into());
        }
    }
    if let Some(owner) = &record.owner {
        if !matches!(record.kind, HtmlNodeKind::Fragment) || record.parent.is_some() {
            return Err(HtmlConstructionErrorKind::AssociationAlreadyOwned.into());
        }
        let id = match owner {
            HtmlFragmentOwner::Template(id) | HtmlFragmentOwner::ShadowHost(id) => id,
        };
        if !Arc::ptr_eq(&id.identity, &arena.identity) {
            return Err(HtmlConstructionErrorKind::ForeignBuilder.into());
        }
        let target = arena
            .records
            .get(id.index)
            .ok_or(HtmlConstructionErrorKind::UnknownNode)?;
        match owner {
            HtmlFragmentOwner::Template(_) => {
                if target.template_contents != Some(TemplateContents::Owned(index))
                    || record.shadow_properties.is_some()
                {
                    return Err(HtmlConstructionErrorKind::InvalidTemplateAssociation.into());
                }
            }
            HtmlFragmentOwner::ShadowHost(_) => {
                if !matches!(&target.shadow,Some(shadow) if shadow.contents.index==index)
                    || record.shadow_properties.is_none()
                {
                    return Err(HtmlConstructionErrorKind::InvalidShadowAssociation.into());
                }
            }
        }
    }
    Ok(())
}

impl HtmlConstructionReadyOperation {
    /// Abandons the ready operation and returns its storage and command. A ready publication
    /// reclaims its arena and reports the retirement evidence.
    pub fn abort(self) -> HtmlConstructionAbort {
        self.operation.abort()
    }
    /// Applies only the fixed writes and owning moves described before readiness.
    pub fn commit(self) -> HtmlConstructionCommit {
        use HtmlConstructionCommand as Command;
        let HtmlConstructionOperation {
            owner,
            command,
            mut plan,
            ..
        } = self.operation;
        let resolved = self.resolved;
        // `try_ready` paired the owner with the command: only a publication has shared its arena.
        let mut storage = match owner {
            StorageOwner::Shared { arena, lease } => {
                let root = plan.positions[0];
                let result = if plan.output_document {
                    HtmlConstructionResult::Document(HtmlDocument { arena, root })
                } else {
                    HtmlConstructionResult::Fragment(HtmlFragment { arena, root })
                };
                return HtmlConstructionCommit::Published { result, lease };
            }
            StorageOwner::Mutable(storage) => storage,
        };
        let command = match command {
            Command::CreateElement {
                name,
                attributes,
                link_flags,
            } => Command::Create {
                kind: HtmlNodeKind::Element(HtmlElement {
                    name,
                    attributes,
                    link_flags,
                }),
            },
            command => command,
        };
        let plan = &mut plan;
        let p = plan.positions;
        if plan.edits_order && !plan.no_op && plan.check_order_edits(&storage.arena).is_err() {
            // The Commit action refuses this state first; were it ever reached,
            // no write may land, so the parent link and the lists cannot diverge.
            return HtmlConstructionCommit::Changed {
                storage,
                created: plan.created.take(),
            };
        }
        if !plan.no_op {
            let arena = &mut storage.arena;
            // A reservation and a transfer check assign nothing, not even the
            // unchanged usage, and describe no write.
            let assigns_usage = !matches!(
                command,
                Command::ReserveRecords { .. } | Command::CheckChildTransfer { .. }
            );
            // Depths move while the edges are the ones planning walked.
            plan.commit_depths(arena);
            let Resolved {
                template_owner,
                association,
            } = resolved;
            match command {
                // The element form was converted to `Create` above.
                Command::CreateElement { .. } => {}
                Command::ReserveRecords { .. } | Command::CheckChildTransfer { .. } => {}
                Command::Create { kind } => {
                    // A template's identity, staged for its contents, is what makes the created
                    // element a template here.
                    let mut record = Record::new(kind, None, 0);
                    record.children = std::mem::take(&mut plan.new_children[0]);
                    record.children_generation = plan.new_children_generations[0];
                    if template_owner.is_some() {
                        record.template_contents = Some(TemplateContents::Owned(p[0] + 1));
                    }
                    arena.records.push(record);
                    if let Some(owner) = template_owner {
                        let mut contents = Record::new(HtmlNodeKind::Fragment, None, 1);
                        contents.children = std::mem::take(&mut plan.new_children[1]);
                        contents.children_generation = plan.new_children_generations[1];
                        contents.owner = Some(HtmlFragmentOwner::Template(owner));
                        arena.records.push(contents);
                        arena.populate(0);
                        arena.populate(1);
                    } else {
                        arena.populate(0);
                    }
                }
                Command::CreateDocument { mode } => {
                    let mut record = Record::new(HtmlNodeKind::Document(mode), None, 0);
                    // This actual admitted constructor issues original state;
                    // generic raw kinds and later mode edits cannot issue it.
                    record.document_focused_area_designation =
                        Some(HtmlFocusedAreaDesignation::KnownNull);
                    record.children = std::mem::take(&mut plan.new_children[0]);
                    record.children_generation = plan.new_children_generations[0];
                    arena.records.push(record);
                    arena.populate(0);
                }
                Command::CreateFragment => {
                    let mut record = Record::new(HtmlNodeKind::Fragment, None, 0);
                    record.children = std::mem::take(&mut plan.new_children[0]);
                    record.children_generation = plan.new_children_generations[0];
                    arena.records.push(record);
                    arena.populate(0);
                }
                Command::CreateShadowContents { properties } => {
                    let mut record = Record::new(HtmlNodeKind::Fragment, None, 0);
                    record.children = std::mem::take(&mut plan.new_children[0]);
                    record.children_generation = plan.new_children_generations[0];
                    record.shadow_properties = Some(properties);
                    arena.records.push(record);
                    arena.populate(0);
                }
                Command::Insert { .. } => {
                    // The guard above the match proved these edits apply.
                    plan.apply_order_edits(arena);
                    arena.records[p[0]].parent = Some(p[1]);
                }
                Command::Detach { .. } => {
                    plan.apply_order_edits(arena);
                    arena.records[p[0]].parent = None;
                }
                Command::ReplaceContent { content, .. } => match content {
                    HtmlConstructionContent::ElementName(name) => {
                        let HtmlNodeKind::Element(element) = &mut arena.records[p[0]].kind else {
                            unreachable!("checked element content kind");
                        };
                        element.name = name;
                        if !plan.link {
                            element.link_flags = None;
                        }
                    }
                    HtmlConstructionContent::Text(value) => {
                        arena.records[p[0]].kind = HtmlNodeKind::Text(value)
                    }
                    HtmlConstructionContent::Comment(value) => {
                        arena.records[p[0]].kind = HtmlNodeKind::Comment(value)
                    }
                    HtmlConstructionContent::ProcessingInstruction(value) => {
                        arena.records[p[0]].kind = HtmlNodeKind::ProcessingInstruction(value)
                    }
                    HtmlConstructionContent::Doctype(value) => {
                        arena.records[p[0]].kind = HtmlNodeKind::Doctype(value)
                    }
                },
                Command::ReplaceAttributes { attributes, .. } => {
                    let HtmlNodeKind::Element(element) = &mut arena.records[p[0]].kind else {
                        unreachable!("checked attribute target kind");
                    };
                    element.attributes = attributes.into_attributes();
                }
                // The associations are applied below, from what planning resolved.
                Command::AttachShadow { .. }
                | Command::AttachTemplateShadow { .. }
                | Command::SetTemplateShadowContents { .. }
                | Command::ReassignTemplateContents { .. } => {}
                Command::SetDocumentMode { mode, .. } => {
                    arena.records[p[0]].kind = HtmlNodeKind::Document(mode)
                }
                Command::SetLinkFlags { flags, .. } => {
                    let HtmlNodeKind::Element(element) = &mut arena.records[p[0]].kind else {
                        unreachable!("checked link element");
                    };
                    element.link_flags = Some(flags);
                }
                // A publication returned above, where its owner is shared.
                Command::Publish { .. } => {}
            }
            match association {
                Association::None => {}
                Association::AttachShadow {
                    host,
                    contents,
                    owner,
                } => {
                    arena.records[p[1]].owner = Some(HtmlFragmentOwner::ShadowHost(owner));
                    arena.records[p[0]].shadow = Some(ShadowAssociation { host, contents });
                }
                Association::AttachTemplateShadow {
                    host,
                    contents,
                    owner,
                    old_contents,
                } => {
                    arena.records[old_contents].owner = None;
                    arena.records[p[0]].template_contents =
                        Some(TemplateContents::ShadowReference(p[2]));
                    arena.records[p[2]].owner = Some(HtmlFragmentOwner::ShadowHost(owner));
                    arena.records[p[1]].shadow = Some(ShadowAssociation { host, contents });
                }
                Association::SetTemplateShadowContents { old_contents } => {
                    arena.records[old_contents].owner = None;
                    arena.records[p[0]].template_contents =
                        Some(TemplateContents::ShadowReference(p[1]));
                }
                Association::ReassignTemplateContents {
                    template,
                    old_contents,
                } => {
                    if let Some(old) = old_contents {
                        arena.records[old].owner = None;
                    }
                    arena.records[p[0]].template_contents = Some(TemplateContents::Owned(p[1]));
                    arena.records[p[1]].owner = Some(HtmlFragmentOwner::Template(template));
                }
            }
            if assigns_usage {
                arena.usage = plan.usage;
            }
        }
        HtmlConstructionCommit::Changed {
            storage,
            created: plan.created.take(),
        }
    }
}

/// Which bank cleanup releases next, from the last record down.
#[derive(Clone, Copy, Debug)]
enum RetirementPhase {
    Children(usize),
    Payload(usize),
    Records,
    Identity,
}
/// The storage that cleanup still holds. Releasing the identity drops it, which completes cleanup.
#[derive(Debug)]
enum RetirementState {
    Active {
        storage: HtmlConstructionStorage,
        phase: RetirementPhase,
    },
    Complete,
}
/// Deterministic cleanup of the original banks; it never resumes construction.
#[derive(Debug)]
pub struct HtmlConstructionRetirement {
    state: RetirementState,
}
/// One pending cleanup action. It borrows the retirement exclusively; `perform` consumes it.
#[derive(Debug)]
pub struct HtmlConstructionRetirementAction<'a> {
    owner: &'a mut HtmlConstructionRetirement,
    release: Release,
}
/// What the pending action releases: an allocation with its snapshot, or one record's payload.
#[derive(Debug)]
enum Release {
    Allocation(HtmlConstructionAllocationSnapshot),
    Payload { position: usize },
}
/// What a cleanup action releases.
#[derive(Debug)]
pub enum HtmlConstructionRetirementDescription<'a> {
    /// A whole allocation: a child-order bank, the records vector or the identity.
    Allocation(&'a HtmlConstructionAllocationSnapshot),
    /// The payload of one record, which cleanup resets in place before its slot is released.
    Payload {
        /// The ordinal of the record whose payload is released.
        position: usize,
        /// The bytes of the record slot that the action resets.
        initialized_bytes: usize,
    },
}
impl HtmlConstructionRetirement {
    pub(super) fn new(storage: HtmlConstructionStorage) -> Self {
        let phase = if storage.arena.records.is_empty() {
            RetirementPhase::Records
        } else {
            RetirementPhase::Children(storage.arena.records.len() - 1)
        };
        Self {
            state: RetirementState::Active { storage, phase },
        }
    }
    /// The next cleanup action, or `Ok(None)` once cleanup completed. It returns an error only if
    /// an allocation snapshot cannot be represented.
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionRetirementAction<'_>>, HtmlConstructionError> {
        let RetirementState::Active { storage, phase } = &self.state else {
            return Ok(None);
        };
        let release = match *phase {
            RetirementPhase::Children(index) => {
                Release::Allocation(storage.children_snapshot(index)?)
            }
            RetirementPhase::Payload(position) => Release::Payload { position },
            RetirementPhase::Records => Release::Allocation(storage.records_snapshot()?),
            RetirementPhase::Identity => {
                Release::Allocation(HtmlConstructionAllocationSnapshot::shared::<Identity>(
                    &storage.arena.identity,
                    AllocationRole::Identity,
                    0,
                ))
            }
        };
        Ok(Some(HtmlConstructionRetirementAction {
            owner: self,
            release,
        }))
    }
}
impl HtmlConstructionRetirementAction<'_> {
    /// Describes the allocation or payload that the pending action releases.
    pub fn description(&self) -> HtmlConstructionRetirementDescription<'_> {
        match &self.release {
            Release::Payload { position } => HtmlConstructionRetirementDescription::Payload {
                position: *position,
                initialized_bytes: size_of::<Record>(),
            },
            Release::Allocation(snapshot) => {
                HtmlConstructionRetirementDescription::Allocation(snapshot)
            }
        }
    }
    /// Performs the cleanup. It returns the retirement evidence for an allocation release and
    /// `None` for a payload reset.
    pub fn perform(self) -> Option<HtmlConstructionRetired> {
        let RetirementState::Active { storage, phase } = &mut self.owner.state else {
            // An action is issued only while cleanup is active, and nothing else can complete it.
            return None;
        };
        match *phase {
            RetirementPhase::Children(index) => {
                drop(std::mem::take(&mut storage.arena.records[index].children));
                *phase = RetirementPhase::Payload(index);
            }
            RetirementPhase::Payload(index) => {
                let record = &mut storage.arena.records[index];
                record.kind = HtmlNodeKind::Fragment;
                record.document_focused_area_designation = None;
                record.owner = None;
                record.shadow = None;
                record.template_contents = None;
                record.shadow_properties = None;
                *phase = if index == 0 {
                    RetirementPhase::Records
                } else {
                    RetirementPhase::Children(index - 1)
                };
            }
            RetirementPhase::Records => {
                drop(std::mem::take(&mut storage.arena.records));
                *phase = RetirementPhase::Identity;
            }
            RetirementPhase::Identity => {
                // Replacing the state drops the storage, which releases the identity.
                self.owner.state = RetirementState::Complete;
            }
        }
        match self.release {
            Release::Allocation(snapshot) => Some(HtmlConstructionRetired { snapshot }),
            Release::Payload { .. } => None,
        }
    }
}

#[cfg(test)]
mod reference_admission_tests {
    use super::*;

    fn drive(
        mut operation: HtmlConstructionOperation,
        identity: &Arc<Identity>,
    ) -> HtmlConstructionCommit {
        loop {
            let before = (Arc::strong_count(identity), Arc::weak_count(identity));
            // Repeated unperformed descriptions must neither mint a witness
            // nor acquire another identity reference before caller admission.
            for _ in 0..2 {
                let Some(action) = operation.next_action().unwrap() else {
                    break;
                };
                let _description = action.description();
                assert_eq!(
                    (Arc::strong_count(identity), Arc::weak_count(identity)),
                    before
                );
            }
            let Some(action) = operation.next_action().unwrap() else {
                break;
            };
            action.perform().unwrap().accept();
        }
        slots::operation(&mut Some(operation)).unwrap().commit()
    }

    #[test]
    fn dropped_action_descriptions_do_not_create_unadmitted_identity_references() {
        let mut initialization = Some(HtmlConstructionStorage::begin());
        initialization
            .as_mut()
            .unwrap()
            .next_action()
            .unwrap()
            .unwrap()
            .perform()
            .accept();
        let storage = slots::initialization(&mut initialization).unwrap().commit();
        let identity = storage.arena.identity.clone();
        let HtmlConstructionCommit::Changed { storage, .. } = drive(
            storage.prepare(HtmlConstructionCommand::ReserveRecords { additional: 4 }),
            &identity,
        ) else {
            panic!("reserve retains actual storage")
        };
        let HtmlConstructionCommit::Changed {
            storage,
            created: Some(root),
            ..
        } = drive(
            storage.prepare(HtmlConstructionCommand::CreateFragment),
            &identity,
        )
        else {
            panic!("created actual root")
        };
        let HtmlConstructionCommit::Published { result, lease } =
            drive(storage.prepare_finish(root), &identity)
        else {
            panic!("published actual storage")
        };
        let recovery = slots::recover(&mut Some(result), &mut Some(lease)).unwrap();
        assert_eq!(recovery.storage.len(), 1);
    }
}

#[cfg(test)]
mod publish_membership_tests {
    use super::*;

    fn drive(
        storage: HtmlConstructionStorage,
        command: HtmlConstructionCommand,
    ) -> (HtmlConstructionStorage, Option<HtmlBuilderNode>) {
        let mut operation = Some(storage.prepare(command));
        while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        match slots::operation(&mut operation).unwrap().commit() {
            HtmlConstructionCommit::Changed {
                storage, created, ..
            } => (storage, created),
            HtmlConstructionCommit::Published { .. } => panic!("mutation published"),
        }
    }

    /// A fragment `root` with ordinary children `a` (position 1) and `b` (position 2).
    fn tree() -> (HtmlConstructionStorage, HtmlBuilderNode) {
        let mut initialization = Some(HtmlConstructionStorage::begin());
        initialization
            .as_mut()
            .unwrap()
            .next_action()
            .unwrap()
            .unwrap()
            .perform()
            .accept();
        let storage = slots::initialization(&mut initialization).unwrap().commit();
        let (mut storage, root) = drive(storage, HtmlConstructionCommand::CreateFragment);
        let root = root.unwrap();
        for local in ["a", "b"] {
            let (next, node) = drive(
                storage,
                HtmlConstructionCommand::Create {
                    kind: HtmlNodeKind::Element(HtmlElement::new(
                        QualifiedName::new(Namespace::html(), None, local),
                        OrderedAttributes::default(),
                    )),
                },
            );
            let (next, _) = drive(
                next,
                HtmlConstructionCommand::Insert {
                    node: node.unwrap(),
                    parent: root.clone(),
                    before: None,
                },
            );
            storage = next;
        }
        (storage, root)
    }

    /// Runs publication until its first refusal, or to completion.
    fn publish(
        storage: HtmlConstructionStorage,
        root: HtmlBuilderNode,
    ) -> (HtmlConstructionStorage, Result<(), HtmlConstructionError>) {
        let mut operation = Some(storage.prepare_finish(root));
        let result = loop {
            match operation.as_mut().unwrap().next_action() {
                Ok(Some(action)) => match action.perform() {
                    Ok(observation) => observation.accept(),
                    Err(failure) => break Err(failure.into_parts().0),
                },
                Ok(None) => break Ok(()),
                Err(error) => break Err(error),
            }
        };
        if result.is_ok() {
            let HtmlConstructionCommit::Published { result, lease } =
                slots::operation(&mut operation).unwrap().commit()
            else {
                panic!("publication produced mutation")
            };
            let recovery = slots::recover(&mut Some(result), &mut Some(lease)).unwrap();
            return (recovery.storage, Ok(()));
        }
        (operation.unwrap().abort().storage, result)
    }

    #[test]
    fn publication_accepts_each_child_listed_exactly_once_whatever_stale_counts_say() {
        let (mut storage, root) = tree();
        // A count left by an earlier refused publication is never trusted.
        for record in &mut storage.arena.records {
            record.publish_found = 2;
        }
        let (storage, result) = publish(storage, root.clone());
        assert_eq!(result, Ok(()));
        // Publishing again from the recovered storage succeeds the same way.
        assert_eq!(publish(storage, root).1, Ok(()));
    }

    #[test]
    fn publication_refuses_a_child_listed_twice() {
        let (mut storage, root) = tree();
        storage.arena.records[0].children = VecDeque::from([1, 1, 2]);
        assert_eq!(
            publish(storage, root).1,
            Err(HtmlConstructionErrorKind::ParentChildConflict.into())
        );
    }

    #[test]
    fn publication_refuses_a_child_missing_from_its_parents_list_despite_a_stale_count() {
        let (mut storage, root) = tree();
        storage.arena.records[0].children = VecDeque::from([1]);
        storage.arena.records[2].publish_found = 1;
        assert_eq!(
            publish(storage, root).1,
            Err(HtmlConstructionErrorKind::ParentChildConflict.into())
        );
    }

    #[test]
    fn publication_refuses_a_child_listed_only_by_another_parent() {
        // `b` names the root as its parent but only `a` lists it.
        let (mut storage, root) = tree();
        storage.arena.records[0].children = VecDeque::from([1]);
        storage.arena.records[1].children = VecDeque::from([2]);
        assert_eq!(
            publish(storage, root).1,
            Err(HtmlConstructionErrorKind::ParentChildConflict.into())
        );
    }
}

#[cfg(test)]
mod order_commit_guard_tests {
    use super::*;

    /// A fragment `root` with children `first` and `second`, and a detached
    /// element `node`.
    fn parts() -> (
        HtmlConstructionStorage,
        HtmlBuilderNode,
        [HtmlBuilderNode; 2],
        HtmlBuilderNode,
    ) {
        fn drive(
            storage: HtmlConstructionStorage,
            command: HtmlConstructionCommand,
        ) -> (HtmlConstructionStorage, Option<HtmlBuilderNode>) {
            let mut operation = Some(storage.prepare(command));
            while let Some(action) = operation.as_mut().unwrap().next_action().unwrap() {
                action.perform().unwrap().accept();
            }
            let HtmlConstructionCommit::Changed {
                storage, created, ..
            } = slots::operation(&mut operation).unwrap().commit()
            else {
                panic!("mutation published")
            };
            (storage, created)
        }
        fn element(local: &str) -> HtmlConstructionCommand {
            HtmlConstructionCommand::Create {
                kind: HtmlNodeKind::Element(HtmlElement::new(
                    QualifiedName::new(Namespace::html(), None, local),
                    OrderedAttributes::default(),
                )),
            }
        }
        let mut initialization = Some(HtmlConstructionStorage::begin());
        initialization
            .as_mut()
            .unwrap()
            .next_action()
            .unwrap()
            .unwrap()
            .perform()
            .accept();
        let storage = slots::initialization(&mut initialization).unwrap().commit();
        let (storage, root) = drive(storage, HtmlConstructionCommand::CreateFragment);
        let root = root.unwrap();
        let (storage, first) = drive(storage, element("first"));
        let (storage, second) = drive(storage, element("second"));
        let (first, second) = (first.unwrap(), second.unwrap());
        let mut storage = storage;
        for child in [&first, &second] {
            storage = drive(
                storage,
                HtmlConstructionCommand::Insert {
                    node: child.clone(),
                    parent: root.clone(),
                    before: None,
                },
            )
            .0;
        }
        let (storage, node) = drive(storage, element("node"));
        (storage, root, [first, second], node.unwrap())
    }

    /// The command `pending_commit` drives, from `root`, its children and `node`.
    type Build =
        fn(&HtmlBuilderNode, &[HtmlBuilderNode; 2], &HtmlBuilderNode) -> HtmlConstructionCommand;

    fn append(
        root: &HtmlBuilderNode,
        _: &[HtmlBuilderNode; 2],
        node: &HtmlBuilderNode,
    ) -> HtmlConstructionCommand {
        HtmlConstructionCommand::Insert {
            node: node.clone(),
            parent: root.clone(),
            before: None,
        }
    }
    fn insert_before_first(
        root: &HtmlBuilderNode,
        children: &[HtmlBuilderNode; 2],
        node: &HtmlBuilderNode,
    ) -> HtmlConstructionCommand {
        HtmlConstructionCommand::Insert {
            node: node.clone(),
            parent: root.clone(),
            before: Some(children[0].clone()),
        }
    }
    fn move_first_to_end(
        root: &HtmlBuilderNode,
        children: &[HtmlBuilderNode; 2],
        _: &HtmlBuilderNode,
    ) -> HtmlConstructionCommand {
        HtmlConstructionCommand::Insert {
            node: children[0].clone(),
            parent: root.clone(),
            before: None,
        }
    }
    fn detach_first(
        _: &HtmlBuilderNode,
        children: &[HtmlBuilderNode; 2],
        _: &HtmlBuilderNode,
    ) -> HtmlConstructionCommand {
        HtmlConstructionCommand::Detach {
            node: children[0].clone(),
        }
    }

    /// Drives `build`'s command up to, but not through, its Commit action,
    /// then lets `spoil` corrupt the pending storage or plan.
    fn pending_commit(
        build: Build,
        spoil: impl FnOnce(&mut HtmlConstructionOperation),
    ) -> (
        HtmlConstructionOperation,
        VecDeque<usize>,
        HtmlConstructionUsage,
    ) {
        let (storage, root, children, node) = parts();
        let usage = storage.usage();
        let mut operation = storage.prepare(build(&root, &children, &node));
        loop {
            let action = operation.next_action().unwrap().unwrap();
            if matches!(
                action.description(),
                HtmlConstructionActionDescription::Commit { .. }
            ) {
                break;
            }
            action.perform().unwrap().accept();
        }
        spoil(&mut operation);
        let children = operation.arena().records[0].children.clone();
        (operation, children, usage)
    }

    fn fill_child_order(operation: &mut HtmlConstructionOperation) {
        let children = &mut operation.arena_mut().unwrap().records[0].children;
        children.shrink_to_fit();
        while children.len() < children.capacity() {
            children.push_back(0);
        }
    }

    fn lose_parent(operation: &mut HtmlConstructionOperation) {
        operation.plan.positions[1] = usize::MAX;
    }

    fn stale_old_position(operation: &mut HtmlConstructionOperation) {
        operation.plan.old_position += 1;
    }

    fn stale_insertion_position(operation: &mut HtmlConstructionOperation) {
        operation.plan.insertion_position += 1;
    }

    #[test]
    fn commit_action_refuses_an_append_whose_parent_order_is_full_or_missing() {
        for spoil in [fill_child_order, lose_parent] {
            let (mut operation, children, usage) = pending_commit(append, spoil);
            let node = operation.plan.positions[0];
            let action = operation.next_action().unwrap().unwrap();
            let failure = action.perform().expect_err("guarded commit");
            assert_eq!(
                failure.reason(),
                &HtmlConstructionErrorKind::CapacityMismatch.error()
            );
            assert_eq!(
                operation.next_action().err(),
                Some(HtmlConstructionErrorKind::CapacityMismatch.error())
            );
            let storage = operation.abort().storage;
            assert_eq!(storage.arena.records[0].children, children);
            assert_eq!(storage.arena.records[node].parent, None);
            assert_eq!(storage.usage(), usage);
        }
    }

    #[test]
    fn ready_commit_applies_nothing_when_the_append_cannot_be_pushed() {
        for spoil in [fill_child_order, lose_parent] {
            let (mut operation, children, usage) = pending_commit(append, spoil);
            let node = operation.plan.positions[0];
            // Bypass the refusing action to reach the commit itself.
            operation.phase = Phase::Ready;
            let HtmlConstructionCommit::Changed { storage, .. } =
                slots::operation(&mut Some(operation)).unwrap().commit()
            else {
                panic!("mutation published")
            };
            assert_eq!(storage.arena.records[0].children, children);
            assert_eq!(storage.arena.records[node].parent, None);
            assert_eq!(storage.usage(), usage);
        }
    }

    /// Corrupts a pending operation's plan or storage.
    type Spoil = fn(&mut HtmlConstructionOperation);

    /// Edits whose planned places no longer hold what planning found.
    const STALE: [(Build, Spoil); 4] = [
        (insert_before_first, stale_insertion_position),
        (move_first_to_end, stale_old_position),
        (move_first_to_end, stale_insertion_position),
        (detach_first, stale_old_position),
    ];

    #[test]
    fn commit_action_refuses_edits_whose_planned_places_are_stale() {
        for (build, spoil) in STALE {
            let (mut operation, children, usage) = pending_commit(build, spoil);
            let action = operation.next_action().unwrap().unwrap();
            let failure = action.perform().expect_err("guarded commit");
            assert_eq!(
                failure.reason(),
                &HtmlConstructionErrorKind::ParentChildConflict.error()
            );
            let storage = operation.abort().storage;
            assert_eq!(storage.arena.records[0].children, children);
            assert_eq!(storage.usage(), usage);
        }
    }

    #[test]
    fn ready_commit_applies_nothing_when_planned_places_are_stale() {
        for (build, spoil) in STALE {
            let (mut operation, children, usage) = pending_commit(build, spoil);
            let nodes: Vec<usize> = operation
                .arena()
                .records
                .iter()
                .map(|record| record.parent.unwrap_or(usize::MAX))
                .collect();
            operation.phase = Phase::Ready;
            let HtmlConstructionCommit::Changed { storage, .. } =
                slots::operation(&mut Some(operation)).unwrap().commit()
            else {
                panic!("mutation published")
            };
            assert_eq!(storage.arena.records[0].children, children);
            let after: Vec<usize> = storage
                .arena
                .records
                .iter()
                .map(|record| record.parent.unwrap_or(usize::MAX))
                .collect();
            assert_eq!(after, nodes, "no parent link changes");
            assert_eq!(storage.usage(), usage);
        }
    }
}

#[cfg(test)]
mod pairing_tests {
    use super::*;

    fn ready_creation() -> HtmlConstructionOperation {
        let mut initialization = HtmlConstructionStorage::begin();
        while let Some(action) = initialization.next_action().unwrap() {
            action.perform().accept();
        }
        let storage = initialization.try_ready().unwrap().commit();
        let mut operation = storage.prepare(HtmlConstructionCommand::CreateFragment);
        while let Some(action) = operation.next_action().unwrap() {
            action.perform().unwrap().accept();
        }
        operation
    }

    #[test]
    fn staged_identities_that_do_not_belong_to_the_command_are_handed_back() {
        let creation = HtmlConstructionCommand::CreateFragment;
        let reserve = HtmlConstructionCommand::ReserveRecords { additional: 1 };
        assert!(matches!(
            resolve(&creation, Staged::Nothing, None),
            Err(Staged::Nothing)
        ));
        assert!(matches!(
            resolve(
                &reserve,
                Staged::Created {
                    template_owner: None
                },
                None
            ),
            Err(Staged::Created {
                template_owner: None
            })
        ));
        assert!(matches!(
            resolve(&reserve, Staged::Nothing, None),
            Ok(Resolved {
                template_owner: None,
                association: Association::None
            })
        ));
    }

    #[test]
    fn a_ready_operation_whose_staged_identities_do_not_fit_its_command_is_refused_unchanged() {
        let mut operation = ready_creation();
        assert!(matches!(operation.phase, Phase::Ready));
        operation.plan.staged = Staged::Nothing;
        let refusal = operation.try_ready().unwrap_err();
        assert!(refusal.reason().is_operation_not_ready());
        assert!(matches!(refusal.owner().plan.staged, Staged::Nothing));
        // The refusal handed the operation back intact: restoring what the creation staged
        // lets the same operation commit.
        let (mut operation, _) = refusal.into_parts();
        operation.plan.staged = Staged::Created {
            template_owner: None,
        };
        let HtmlConstructionCommit::Changed { storage, created } =
            operation.try_ready().unwrap().commit()
        else {
            panic!("creation");
        };
        assert!(created.is_some());
        assert_eq!(storage.len(), 1);
    }
}
