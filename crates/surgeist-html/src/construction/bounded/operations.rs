use super::*;
mod orders;
use crate::{Attribute, bounded::HtmlValidatedAttributes};
pub use orders::*;

/// Closed generic changes; parser policy and source evidence are not operands.
#[derive(Debug)]
#[non_exhaustive]
pub enum HtmlConstructionCommand {
    Create {
        kind: HtmlNodeKind,
    },
    /// Creates exact element values and explicit producer-supplied history in
    /// one record commit. Flags require an actual HTML link expanded name.
    CreateElement {
        name: QualifiedName,
        attributes: OrderedAttributes,
        link_flags: Option<crate::HtmlLinkFlags>,
    },
    CreateDocument {
        mode: HtmlDocumentMode,
    },
    CreateFragment,
    ReserveRecords {
        additional: usize,
    },
    CheckChildTransfer {
        from: HtmlBuilderNode,
        to: HtmlBuilderNode,
    },
    CreateShadowContents {
        properties: HtmlShadowRootProperties,
    },
    Insert {
        node: HtmlBuilderNode,
        parent: HtmlBuilderNode,
        before: Option<HtmlBuilderNode>,
    },
    Detach {
        node: HtmlBuilderNode,
    },
    ReplaceContent {
        node: HtmlBuilderNode,
        content: HtmlConstructionContent,
    },
    ReplaceAttributes {
        element: HtmlBuilderNode,
        attributes: HtmlValidatedAttributes,
    },
    AttachShadow {
        host: HtmlBuilderNode,
        contents: HtmlBuilderNode,
    },
    AttachTemplateShadow {
        template: HtmlBuilderNode,
        host: HtmlBuilderNode,
        contents: HtmlBuilderNode,
    },
    DetachShadow {
        host: HtmlBuilderNode,
    },
    SetTemplateShadowContents {
        template: HtmlBuilderNode,
        contents: HtmlBuilderNode,
    },
    ReassignTemplateContents {
        template: HtmlBuilderNode,
        contents: HtmlBuilderNode,
    },
    SetDocumentMode {
        document: HtmlBuilderNode,
        mode: HtmlDocumentMode,
    },
    SetLinkFlags {
        element: HtmlBuilderNode,
        flags: crate::HtmlLinkFlags,
    },
    Publish {
        root: HtmlBuilderNode,
    },
}
/// Content replacement preserves node kind and cannot replace topology.
#[derive(Debug)]
pub enum HtmlConstructionContent {
    ElementName(QualifiedName),
    Text(DocumentString),
    Comment(DocumentString),
    ProcessingInstruction(HtmlProcessingInstruction),
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
            (Self::AttachShadow { host, .. } | Self::DetachShadow { host }, 0) => Some(host),
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
    InitializeIdentity {
        payload_bytes: usize,
        initialized_bytes: usize,
        provider_calls: usize,
    },
    PrepareAllocationReport {
        record_visits: usize,
        provider_calls: usize,
        copied_bytes: usize,
    },
    Inspect {
        record_visits: usize,
        inspected_bytes: usize,
    },
    Compare {
        inspected_bytes: usize,
    },
    Reserve {
        report_provider_calls: usize,
        allocation: &'a HtmlConstructionAllocationSnapshot,
        required_len: usize,
        minimum_capacity: usize,
        growth: HtmlConstructionGrowth,
    },
    Share {
        report_provider_calls: usize,
        allocation: &'a HtmlConstructionAllocationSnapshot,
    },
    ShareReferences {
        provider_calls: usize,
        copied_bytes: usize,
    },
    CopyChildOrderSource {
        record_visits: usize,
        copied_bytes: usize,
    },
    InitializeChildOrder {
        provider_calls: usize,
        slot: usize,
        parent_position: usize,
        initialized_bytes: usize,
    },
    PrepareChildOrderTransfers {
        count: usize,
        record_writes: usize,
        provider_calls: usize,
        copied_bytes: usize,
    },
    PushChildOrder {
        slot: usize,
        copied_bytes: usize,
        record_writes: usize,
        auxiliary_entries: usize,
    },
    InitializeChildren {
        provider_calls: usize,
        copied_bytes: usize,
        record_writes: usize,
        position: usize,
        element_size: usize,
        element_alignment: usize,
    },
    /// Computed Insert usage, before any pending child-order initialization.
    Preflight {
        usage: HtmlConstructionUsage,
    },
    Commit {
        /// Commands without an earlier usage boundary observe depth here.
        depth_observation: Option<usize>,
        record_writes: usize,
        copied_bytes: usize,
        initialized_bytes: usize,
        usage: HtmlConstructionUsage,
        auxiliary_entries_released: usize,
    },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HtmlConstructionGrowth {
    Exact,
    Geometric,
}

#[derive(Clone, Copy, Debug)]
enum Phase {
    Operand(usize),
    ClassifyNamespace,
    NamespaceUnit(usize),
    ClassifyLocal,
    LocalUnit(usize),
    Command,
    NewAttribute(usize),
    OldAttribute(usize),
    Order {
        parent: usize,
        offset: usize,
        old: bool,
    },
    Cycle {
        cursor: Option<usize>,
        target: usize,
        steps: usize,
    },
    PublishNode(usize),
    PublishChild {
        parent: usize,
        offset: usize,
    },
    PublishParent {
        node: usize,
        parent: usize,
        offset: usize,
        found: usize,
    },
    Depth {
        node: usize,
        cursor: usize,
        depth: usize,
        steps: usize,
    },
    Topology,
    PrepareRecordsReport,
    PrepareWrapperReport,
    ReserveRecords,
    InitializeOrder(usize),
    ReserveOrder(usize),
    CopyOrder {
        slot: usize,
        offset: usize,
    },
    PushInserted {
        slot: usize,
        offset: usize,
        following: Option<usize>,
    },
    PushExisting {
        slot: usize,
        offset: usize,
        child: usize,
    },
    TransferOrders,
    Preflight,
    Layout,
    References,
    Children(usize),
    Share,
    Commit,
    Observing,
    Ready,
    Failed,
}

#[derive(Debug, Default)]
struct Plan {
    positions: [usize; 3],
    template: bool,
    output_document: bool,
    fast_depth: Option<usize>,
    link: bool,
    local_expected: &'static [u8],
    usage: HtmlConstructionUsage,
    new_units: usize,
    new_attributes: usize,
    old_units: usize,
    old_attributes: usize,
    old_parent: Option<usize>,
    old_position: usize,
    insertion_position: usize,
    before_found: bool,
    old_found: bool,
    old_contents: Option<usize>,
    max_depth: usize,
    no_op: bool,
    record_count: usize,
    copied_bytes: usize,
    initialized_bytes: usize,
    record_writes: usize,
    reserve_required: usize,
    references: usize,
    ids: [Option<HtmlNodeId>; 3],
    created: Option<HtmlBuilderNode>,
    new_children: [Vec<usize>; 2],
    new_children_generations: [u64; 2],
    orders: [Vec<usize>; 2],
    order_count: usize,
    order_parents: [usize; 2],
    order_required: [usize; 2],
    order_source_len: [usize; 2],
    order_inserted: [bool; 2],
    order_snapshots: [Option<HtmlConstructionAllocationSnapshot>; 2],
    order_retired_snapshots: [Option<HtmlConstructionAllocationSnapshot>; 2],
    transfers: [Option<HtmlConstructionChildOrderTransfer>; 2],
    scratch_entries: usize,
}
enum StorageOwner {
    Mutable(HtmlConstructionStorage),
    Shared(Arc<Arena>),
}
impl StorageOwner {
    fn arena(&self) -> &Arena {
        match self {
            Self::Mutable(v) => &v.arena,
            Self::Shared(v) => v,
        }
    }
    fn mutable(&mut self) -> &mut HtmlConstructionStorage {
        match self {
            Self::Mutable(v) => v,
            Self::Shared(_) => unreachable!("shared publication has no later mutation"),
        }
    }
}

/// A pending operation retains its exact storage and command through refusal.
pub struct HtmlConstructionOperation {
    owner: Option<StorageOwner>,
    command: Option<HtmlConstructionCommand>,
    phase: Phase,
    next: Option<Phase>,
    failure: Option<HtmlConstructionError>,
    plan: Plan,
    snapshot: Option<HtmlConstructionAllocationSnapshot>,
    lease: Option<HtmlConstructionLease>,
}
/// No rejected command loses its input or storage owner.
pub struct HtmlConstructionRejection {
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
    reason: HtmlConstructionError,
}
impl HtmlConstructionRejection {
    pub fn into_parts(
        self,
    ) -> (
        HtmlConstructionStorage,
        HtmlConstructionCommand,
        HtmlConstructionError,
    ) {
        (self.storage, self.command, self.reason)
    }
}
pub struct HtmlConstructionAbort {
    storage: HtmlConstructionStorage,
    command: HtmlConstructionCommand,
    retired: Option<HtmlConstructionRetired>,
    orders: HtmlConstructionOrderRetirement,
}
impl HtmlConstructionAbort {
    pub fn into_parts(
        self,
    ) -> (
        HtmlConstructionStorage,
        HtmlConstructionCommand,
        Option<HtmlConstructionRetired>,
        HtmlConstructionOrderRetirement,
    ) {
        (self.storage, self.command, self.retired, self.orders)
    }
}
pub struct HtmlConstructionReadyOperation {
    operation: HtmlConstructionOperation,
}
#[expect(clippy::large_enum_variant)] // Owns actual pending banks; boxing would add an unadmitted allocation.
pub enum HtmlConstructionCommit {
    Changed {
        storage: HtmlConstructionStorage,
        created: Option<HtmlBuilderNode>,
        transfers: [Option<HtmlConstructionChildOrderTransfer>; 2],
        retired_orders: HtmlConstructionOrderRetirement,
    },
    Published {
        result: HtmlConstructionResult,
        lease: HtmlConstructionLease,
    },
}
pub struct HtmlConstructionAction<'a> {
    operation: &'a mut HtmlConstructionOperation,
}
pub struct HtmlConstructionObservation<'a> {
    operation: &'a mut HtmlConstructionOperation,
    allocation: Option<HtmlConstructionAllocationSnapshot>,
}
#[derive(Debug)]
pub struct HtmlConstructionActionFailure {
    reason: HtmlConstructionError,
    allocation: Option<HtmlConstructionAllocationSnapshot>,
}
impl HtmlConstructionActionFailure {
    pub fn reason(&self) -> &HtmlConstructionError {
        &self.reason
    }
    pub fn allocation(&self) -> Option<&HtmlConstructionAllocationSnapshot> {
        self.allocation.as_ref()
    }
    pub fn into_parts(
        self,
    ) -> (
        HtmlConstructionError,
        Option<HtmlConstructionAllocationSnapshot>,
    ) {
        (self.reason, self.allocation)
    }
}
impl<'a> HtmlConstructionObservation<'a> {
    pub fn allocation(&self) -> Option<&HtmlConstructionAllocationSnapshot> {
        self.allocation.as_ref()
    }
    pub fn accept(self) {
        self.operation.phase = self
            .operation
            .next
            .take()
            .expect("one outstanding observation");
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
            owner: Some(StorageOwner::Mutable(storage)),
            command: Some(command),
            phase,
            next: None,
            failure: None,
            plan: Plan::default(),
            snapshot: None,
            lease: None,
        }
    }
    fn arena(&self) -> &Arena {
        self.owner.as_ref().expect("pending storage").arena()
    }
    fn command(&self) -> &HtmlConstructionCommand {
        self.command.as_ref().expect("pending command")
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
        if self.failure.is_none() {
            self.failure = Some(error);
        }
        self.phase = Phase::Failed;
        Err(self.failure.clone().expect("latched failure"))
    }
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionAction<'_>>, HtmlConstructionError> {
        if let Some(error) = &self.failure {
            return Err(error.clone());
        }
        if matches!(self.phase, Phase::Observing) {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        if matches!(self.phase, Phase::Ready) {
            return Ok(None);
        }
        Ok(Some(HtmlConstructionAction { operation: self }))
    }
    pub fn try_ready(
        owner: &mut Option<Self>,
    ) -> Result<HtmlConstructionReadyOperation, HtmlConstructionError> {
        let operation = owner
            .as_ref()
            .ok_or(HtmlConstructionError::OperationConsumed)?;
        if let Some(error) = &operation.failure {
            return Err(error.clone());
        }
        if !matches!(operation.phase, Phase::Ready) {
            return Err(HtmlConstructionError::OperationNotReady);
        }
        Ok(HtmlConstructionReadyOperation {
            operation: owner.take().expect("occupied operation"),
        })
    }
    pub fn abort(mut self) -> HtmlConstructionAbort {
        let (storage, retired) = match self.owner.take().expect("pending storage") {
            StorageOwner::Mutable(storage) => (storage, None),
            StorageOwner::Shared(arena) => {
                let lease = self
                    .lease
                    .take()
                    .expect("actual unescaped publication lease");
                let snapshot = lease.snapshot;
                drop(lease.arena);
                let arena =
                    Arc::try_unwrap(arena).expect("unescaped publication is uniquely owned");
                (
                    HtmlConstructionStorage { arena },
                    Some(HtmlConstructionRetired { snapshot }),
                )
            }
        };
        let orders = self.abort_orders();
        HtmlConstructionAbort {
            storage,
            command: self.command.take().expect("pending command"),
            retired,
            orders,
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
        match self.operation.owner.as_ref()? {
            StorageOwner::Mutable(storage) => Some(storage),
            StorageOwner::Shared(_) => None,
        }
    }
    pub fn description(&self) -> HtmlConstructionActionDescription<'_> {
        let operation = &self.operation;
        match operation.phase {
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
            Phase::InitializeOrder(slot) => {
                HtmlConstructionActionDescription::InitializeChildOrder {
                    provider_calls: 2,
                    slot,
                    parent_position: operation.plan.order_parents[slot],
                    initialized_bytes: size_of::<Vec<usize>>() + size_of::<u64>(),
                }
            }
            Phase::ReserveOrder(slot) => HtmlConstructionActionDescription::Reserve {
                report_provider_calls: 1,
                allocation: operation.plan.order_snapshots[slot]
                    .as_ref()
                    .expect("actual pending order"),
                required_len: operation.plan.order_required[slot],
                minimum_capacity: operation.plan.order_required[slot],
                growth: HtmlConstructionGrowth::Exact,
            },
            Phase::CopyOrder { slot, offset } => {
                HtmlConstructionActionDescription::CopyChildOrderSource {
                    record_visits: 1,
                    copied_bytes: if offset < operation.plan.order_source_len[slot] {
                        size_of::<usize>()
                    } else {
                        0
                    },
                }
            }
            Phase::PushInserted { slot, .. } => HtmlConstructionActionDescription::PushChildOrder {
                slot,
                copied_bytes: size_of::<usize>(),
                record_writes: 1,
                auxiliary_entries: 1,
            },
            Phase::PushExisting { slot, .. } => HtmlConstructionActionDescription::PushChildOrder {
                slot,
                copied_bytes: 0,
                record_writes: 1,
                auxiliary_entries: 1,
            },
            Phase::TransferOrders => {
                HtmlConstructionActionDescription::PrepareChildOrderTransfers {
                    count: operation.plan.order_count,
                    record_writes: operation.plan.order_count,
                    provider_calls: 3 * operation.plan.order_count,
                    copied_bytes: operation.plan.order_count
                        * (size_of::<HtmlConstructionChildOrderTransfer>()
                            + size_of::<HtmlConstructionAllocationSnapshot>()
                            + size_of::<u64>()),
                }
            }
            Phase::ReserveRecords => HtmlConstructionActionDescription::Reserve {
                report_provider_calls: 1,
                allocation: operation.snapshot.as_ref().expect("issued actual snapshot"),
                required_len: operation.plan.reserve_required,
                minimum_capacity: operation.plan.reserve_required,
                growth: HtmlConstructionGrowth::Exact,
            },
            Phase::Share => HtmlConstructionActionDescription::Share {
                report_provider_calls: 3,
                allocation: operation.snapshot.as_ref().expect("issued actual snapshot"),
            },
            Phase::References => HtmlConstructionActionDescription::ShareReferences {
                provider_calls: operation.plan.references,
                copied_bytes: operation.plan.references * size_of::<HtmlNodeId>(),
            },
            Phase::Children(offset) => HtmlConstructionActionDescription::InitializeChildren {
                provider_calls: 1,
                copied_bytes: 2 * size_of::<u64>(),
                record_writes: 2,
                position: operation.plan.positions[0] + offset,
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
                record_writes: operation.plan.record_writes,
                copied_bytes: operation.plan.copied_bytes,
                initialized_bytes: operation.plan.initialized_bytes,
                usage: operation.plan.usage,
                auxiliary_entries_released: operation.plan.scratch_entries,
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
            Phase::Depth { .. } | Phase::PublishChild { .. } => {
                HtmlConstructionActionDescription::Inspect {
                    record_visits: 2,
                    inspected_bytes: 0,
                }
            }
            _ => HtmlConstructionActionDescription::Inspect {
                record_visits: 1,
                inspected_bytes: 0,
            },
        }
    }
    pub fn perform(self) -> Result<HtmlConstructionObservation<'a>, HtmlConstructionActionFailure> {
        let operation = self.operation;
        match operation.advance() {
            Ok((next, allocation)) => {
                operation.next = Some(next);
                operation.phase = Phase::Observing;
                Ok(HtmlConstructionObservation {
                    operation,
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
        .ok_or(HtmlConstructionError::RepresentationExhausted)
}
fn subtract(left: usize, right: usize) -> Result<usize, HtmlConstructionError> {
    left.checked_sub(right)
        .ok_or(HtmlConstructionError::ReleaseUnderflow)
}
fn checked_layout(slots: usize, element_size: usize) -> Result<usize, HtmlConstructionError> {
    slots
        .checked_mul(element_size)
        .filter(|bytes| *bytes <= isize::MAX as usize)
        .ok_or(HtmlConstructionError::RepresentationExhausted)
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
fn attribute_units(attribute: &Attribute) -> Result<usize, HtmlConstructionError> {
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
pub(in crate::construction) fn created_usage(
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
    let record = &arena.records[index];
    record.parent.or(match &record.owner {
        Some(HtmlFragmentOwner::Template(owner) | HtmlFragmentOwner::ShadowHost(owner)) => {
            Some(owner.index)
        }
        None => None,
    })
}
fn ordinary_owned_contents(arena: &Arena, template: usize) -> Result<usize, HtmlConstructionError> {
    let record = &arena.records[template];
    if !matches!(record.kind, HtmlNodeKind::Element(_)) {
        return Err(HtmlConstructionError::InvalidTemplateAssociation);
    }
    let Some(TemplateContents::Owned(contents)) = record.template_contents else {
        return Err(HtmlConstructionError::InvalidTemplateAssociation);
    };
    let target = &arena.records[contents];
    if !matches!(target.kind, HtmlNodeKind::Fragment)
        || target.shadow_properties.is_some()
        || target.parent.is_some()
        || !matches!(&target.owner, Some(HtmlFragmentOwner::Template(owner)) if owner.index == template && Arc::ptr_eq(&owner.identity, &arena.identity))
    {
        return Err(HtmlConstructionError::InvalidTemplateAssociation);
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
            HtmlConstructionError::InvalidContentsReference
        } else {
            HtmlConstructionError::InvalidTemplateAssociation
        });
    }
    if target.owner.is_some() {
        return Err(HtmlConstructionError::AssociationAlreadyOwned);
    }
    Ok(())
}

impl HtmlConstructionOperation {
    fn command_phase(&mut self) -> Result<Phase, HtmlConstructionError> {
        use HtmlConstructionCommand as Command;
        let mut plan = std::mem::take(&mut self.plan);
        let result = (|| {
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
                                return Err(HtmlConstructionError::WrongNodeKind);
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
                                return Err(HtmlConstructionError::WrongNodeKind);
                            }
                            plan.new_attributes = attributes.len();
                            name_units(name)?
                        }
                        _ => unreachable!("creation command"),
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
                                return Err(HtmlConstructionError::InvalidTemplateAssociation);
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
                        _ => return Err(HtmlConstructionError::WrongNodeKind),
                    }
                    plan.copied_bytes = add(plan.copied_bytes, size_of::<HtmlNodeKind>())?;
                }
                Command::ReplaceAttributes { attributes, .. } => {
                    let HtmlNodeKind::Element(old) = &arena.records[a].kind else {
                        return Err(HtmlConstructionError::WrongNodeKind);
                    };
                    plan.new_attributes = attributes.as_slice().len();
                    plan.old_attributes = old.attributes().len();
                    plan.copied_bytes = add(plan.copied_bytes, size_of::<OrderedAttributes>())?;
                }
                Command::AttachShadow { .. } => {
                    if !matches!(arena.records[a].kind, HtmlNodeKind::Element(_)) {
                        return Err(HtmlConstructionError::ShadowHostMustBeElement);
                    }
                    if arena.records[a].shadow.is_some() {
                        return Err(HtmlConstructionError::ShadowAlreadyAttached);
                    }
                    unowned_fragment(arena, b, true)?;
                    plan.usage.edges = add(plan.usage.edges, 1)?;
                }
                Command::AttachTemplateShadow { .. } => {
                    plan.old_contents = Some(ordinary_owned_contents(arena, a)?);
                    if !matches!(arena.records[b].kind, HtmlNodeKind::Element(_)) {
                        return Err(HtmlConstructionError::ShadowHostMustBeElement);
                    }
                    if arena.records[b].shadow.is_some() {
                        return Err(HtmlConstructionError::ShadowAlreadyAttached);
                    }
                    unowned_fragment(arena, c, true)?;
                    if Some(c) == plan.old_contents {
                        return Err(HtmlConstructionError::InvalidContentsReference);
                    }
                }
                Command::SetTemplateShadowContents { .. } => {
                    plan.old_contents = Some(ordinary_owned_contents(arena, a)?);
                    unowned_fragment(arena, b, true)?;
                    if Some(b) == plan.old_contents {
                        return Err(HtmlConstructionError::InvalidContentsReference);
                    }
                    plan.usage.edges = subtract(plan.usage.edges, 1)?;
                }
                Command::ReassignTemplateContents { .. } => {
                    if !matches!(arena.records[a].kind, HtmlNodeKind::Element(_)) {
                        return Err(HtmlConstructionError::InvalidTemplateAssociation);
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
                                return Err(HtmlConstructionError::InvalidTemplateAssociation);
                            }
                            unowned_fragment(arena, b, false)?;
                            plan.usage.edges = add(plan.usage.edges, 1)?;
                        }
                        None => return Err(HtmlConstructionError::InvalidTemplateAssociation),
                    }
                }
                Command::DetachShadow { .. } => {
                    if !matches!(arena.records[a].kind, HtmlNodeKind::Element(_)) {
                        return Err(HtmlConstructionError::ShadowHostMustBeElement);
                    }
                    plan.old_contents = arena.records[a]
                        .shadow
                        .as_ref()
                        .map(|value| value.contents.index);
                    plan.no_op = plan.old_contents.is_none();
                    plan.usage.edges =
                        subtract(plan.usage.edges, usize::from(plan.old_contents.is_some()))?;
                }
                Command::SetDocumentMode { .. } => {
                    if !matches!(arena.records[a].kind, HtmlNodeKind::Document(_)) {
                        return Err(HtmlConstructionError::WrongNodeKind);
                    }
                }
                Command::SetLinkFlags { .. } => {
                    if !plan.link {
                        return Err(HtmlConstructionError::WrongNodeKind);
                    }
                }
                Command::Publish { .. } => {
                    if !matches!(
                        arena.records[a].kind,
                        HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment
                    ) || arena.records[a].parent.is_some()
                    {
                        return Err(HtmlConstructionError::InvalidOutputRoot);
                    }
                    plan.output_document =
                        matches!(arena.records[a].kind, HtmlNodeKind::Document(_));
                    return Ok(Phase::PublishNode(0));
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
        if let Some(parent) = plan.old_parent {
            return Ok(Phase::Order {
                parent,
                offset: 0,
                old: true,
            });
        }
        match self.command() {
            Command::Insert { .. } => Ok(Phase::Order {
                parent: plan.positions[1],
                offset: 0,
                old: false,
            }),
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
                | HtmlConstructionCommand::DetachShadow { .. }
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
    fn topology(&mut self) -> Result<Phase, HtmlConstructionError> {
        use HtmlConstructionCommand as Command;
        let arena = self.arena();
        let p = self.plan.positions;
        if arena.depths_valid {
            if matches!(self.command(), Command::Publish { .. }) {
                return Ok(Phase::Layout);
            }
            let indices = match self.command() {
                Command::Insert { .. } if self.plan.old_parent.is_none() => Some((p[0], p[1])),
                Command::AttachShadow { .. } => Some((p[1], p[0])),
                _ => None,
            };
            if let Some((node, parent)) = indices {
                let record = &arena.records[node];
                let contents = record.template_contents.and_then(|value| match value {
                    TemplateContents::Owned(index) => Some(index),
                    TemplateContents::ShadowReference(_) => None,
                });
                let shallow = record.children.is_empty()
                    && record.shadow.is_none()
                    && contents.is_none_or(|index| {
                        let contents = &arena.records[index];
                        contents.children.is_empty() && contents.shadow.is_none()
                    });
                if shallow {
                    let depth = add(arena.records[parent].depth, 1)?;
                    self.plan.usage.depth = self
                        .plan
                        .usage
                        .depth
                        .max(add(depth, usize::from(contents.is_some()))?);
                    self.plan.fast_depth = Some(depth);
                    return self.before_writes();
                }
            }
        }
        self.plan.max_depth = 0;
        Ok(Phase::Depth {
            node: 0,
            cursor: 0,
            depth: 0,
            steps: 0,
        })
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
        let result = (|| {
            match self.command() {
                Command::Insert { .. } | Command::Detach { .. } => {
                    let target = if matches!(self.command(), Command::Insert { .. }) {
                        plan.positions[1]
                    } else {
                        plan.old_parent.expect("nonempty detach")
                    };
                    plan.order_count = 1;
                    plan.order_parents[0] = target;
                    if let Some(old) = plan.old_parent.filter(|old| *old != target) {
                        plan.order_count = 2;
                        plan.order_parents[1] = old;
                    }
                    for slot in 0..plan.order_count {
                        let len = self.arena().records[plan.order_parents[slot]]
                            .children
                            .len();
                        plan.order_source_len[slot] = len;
                        plan.order_required[slot] = add(len, 1)?;
                    }
                    plan.record_writes = if matches!(self.command(), Command::Insert { .. }) {
                        8
                    } else {
                        4
                    };
                    plan.copied_bytes = add(
                        plan.copied_bytes,
                        plan.order_count * 2 * size_of::<Vec<usize>>(),
                    )?;
                }
                Command::AttachShadow { .. } | Command::AttachTemplateShadow { .. } => {
                    plan.references = 3;
                    plan.record_writes = add(plan.record_writes, 5)?;
                    plan.copied_bytes = add(
                        plan.copied_bytes,
                        size_of::<ShadowAssociation>()
                            + 2 * size_of::<Option<HtmlFragmentOwner>>()
                            + size_of::<Option<TemplateContents>>(),
                    )?;
                }
                Command::SetTemplateShadowContents { .. }
                | Command::ReassignTemplateContents { .. }
                | Command::DetachShadow { .. } => {
                    plan.references = usize::from(matches!(
                        self.command(),
                        Command::ReassignTemplateContents { .. }
                    ));
                    plan.record_writes = add(plan.record_writes, 3)?;
                    plan.copied_bytes = add(
                        plan.copied_bytes,
                        2 * size_of::<Option<HtmlFragmentOwner>>()
                            + size_of::<Option<TemplateContents>>()
                            + size_of::<Option<ShadowAssociation>>(),
                    )?;
                }
                Command::SetDocumentMode { .. } => {
                    plan.copied_bytes = add(plan.copied_bytes, size_of::<HtmlDocumentMode>())?;
                    plan.record_writes += 1;
                }
                Command::SetLinkFlags { .. } => {
                    plan.copied_bytes = add(plan.copied_bytes, size_of::<crate::HtmlLinkFlags>())?;
                    plan.record_writes += 1;
                }
                _ => {}
            }
            if self.command().creates() {
                plan.initialized_bytes = plan
                    .record_count
                    .checked_mul(size_of::<Record>())
                    .ok_or(HtmlConstructionError::RepresentationExhausted)?;
                plan.copied_bytes = add(plan.copied_bytes, plan.initialized_bytes)?;
                plan.record_writes = add(plan.record_writes, plan.record_count)?;
                plan.references = 1 + usize::from(plan.template);
                plan.reserve_required = add(self.arena().records.len(), plan.record_count)?;
                Ok(if plan.reserve_required > self.arena().records.capacity() {
                    Phase::PrepareRecordsReport
                } else {
                    Phase::References
                })
            } else if matches!(
                self.command(),
                Command::Insert { .. } | Command::Detach { .. }
            ) && !plan.no_op
            {
                Ok(Phase::InitializeOrder(0))
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
    fn virtual_parent(&self, index: usize) -> Option<usize> {
        use HtmlConstructionCommand as Command;
        let p = self.plan.positions;
        if Some(index) == self.plan.old_contents {
            return None;
        }
        match self.command() {
            Command::Insert { .. } if index == p[0] => Some(p[1]),
            Command::Detach { .. } if index == p[0] => None,
            Command::AttachShadow { .. } if index == p[1] => Some(p[0]),
            Command::AttachTemplateShadow { .. } if index == p[2] => Some(p[1]),
            Command::ReassignTemplateContents { .. } if index == p[1] => Some(p[0]),
            _ => structural_parent(self.arena(), index),
        }
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
                let snapshot = self.owner.as_ref().expect("actual storage").arena();
                self.snapshot = Some(
                    HtmlConstructionAllocationSnapshot::vector(
                        &snapshot.identity,
                        AllocationRole::Records,
                        snapshot.records_generation,
                        &snapshot.records,
                    )
                    .map_err(|error| (error, None))?,
                );
                Ok((Phase::ReserveRecords, None))
            }
            Phase::PrepareWrapperReport => {
                let arena = self.arena();
                let generation = arena
                    .wrapper_generation
                    .checked_add(1)
                    .ok_or((HtmlConstructionError::RepresentationExhausted, None))?;
                let mut snapshot = HtmlConstructionAllocationSnapshot::shared::<Arena>(
                    &arena.identity,
                    AllocationRole::ArenaWrapper,
                    generation,
                );
                snapshot.initialized_len = 0;
                snapshot.capacity = 0;
                snapshot.payload_bytes = 0;
                self.snapshot = Some(snapshot);
                Ok((Phase::Share, None))
            }
            Phase::InitializeOrder(slot) => self.initialize_order(slot),
            Phase::ReserveOrder(slot) => self.reserve_order(slot),
            Phase::ReserveRecords => {
                let required = self.plan.reserve_required;
                let reserve_only = matches!(
                    self.command(),
                    HtmlConstructionCommand::ReserveRecords { .. }
                );
                let storage = self.owner.as_mut().expect("storage").mutable();
                let count = required
                    .checked_sub(storage.arena.records.len())
                    .ok_or((HtmlConstructionError::RepresentationExhausted, None))?;
                let changed = required > storage.arena.records.capacity();
                let next_generation = storage
                    .arena
                    .records_generation
                    .checked_add(u64::from(changed))
                    .ok_or((HtmlConstructionError::RepresentationExhausted, None))?;
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
                    Err(_) => Err((HtmlConstructionError::AllocationFailed, Some(snapshot))),
                }
            }
            Phase::Share => {
                let StorageOwner::Mutable(mut storage) = self.owner.take().expect("storage") else {
                    unreachable!("one publication action")
                };
                storage.arena.wrapper_generation = self
                    .snapshot
                    .as_ref()
                    .expect("issued wrapper snapshot")
                    .id
                    .generation;
                storage.arena.usage = self.plan.usage;
                let arena = Arc::new(storage.arena);
                let snapshot = HtmlConstructionAllocationSnapshot::shared::<Arena>(
                    &arena.identity,
                    AllocationRole::ArenaWrapper,
                    arena.wrapper_generation,
                );
                self.lease = Some(HtmlConstructionLease {
                    arena: Arc::downgrade(&arena),
                    snapshot: snapshot.clone(),
                });
                self.owner = Some(StorageOwner::Shared(arena));
                Ok((Phase::Commit, Some(snapshot)))
            }
            Phase::Children(offset) => {
                let arena = &mut self
                    .owner
                    .as_mut()
                    .expect("pending storage")
                    .mutable()
                    .arena;
                let generation = arena
                    .temporary_generation
                    .checked_add(1)
                    .ok_or((HtmlConstructionError::RepresentationExhausted, None))?;
                let snapshot = HtmlConstructionAllocationSnapshot::vector(
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
                    .expect("described actual operand");
                let arena = self.arena();
                if handle.identity.as_ptr() != Arc::as_ptr(&arena.identity) {
                    return Err(HtmlConstructionError::ForeignBuilder);
                }
                if handle.index >= arena.records.len() {
                    return Err(HtmlConstructionError::UnknownNode);
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
                        .is_some_and(|uri| uri.len() == b"http://www.w3.org/1999/xhtml".len())
                    {
                        Phase::NamespaceUnit(0)
                    } else {
                        Phase::Command
                    },
                )
            }
            Phase::NamespaceUnit(offset) => {
                const HTML: &[u8] = b"http://www.w3.org/1999/xhtml";
                let uri = self
                    .name()
                    .expect("name comparison")
                    .namespace()
                    .uri()
                    .expect("namespace length checked");
                Ok(if uri.code_units()[offset] != u16::from(HTML[offset]) {
                    Phase::Command
                } else if offset + 1 == HTML.len() {
                    Phase::ClassifyLocal
                } else {
                    Phase::NamespaceUnit(offset + 1)
                })
            }
            Phase::ClassifyLocal => {
                let name = self.name().expect("HTML namespace was compared");
                self.plan.local_expected = match name.local_name().len() {
                    8 => b"template",
                    4 => b"link",
                    _ => return Ok(Phase::Command),
                };
                Ok(Phase::LocalUnit(0))
            }
            Phase::LocalUnit(offset) => {
                let local = self.name().expect("local name comparison").local_name();
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
            Phase::CopyOrder { slot, offset } => self.read_order(slot, offset),
            Phase::PushInserted {
                slot,
                offset,
                following,
            } => Ok(self.push_inserted(slot, offset, following)),
            Phase::PushExisting {
                slot,
                offset,
                child,
            } => {
                self.push_order(slot, child);
                Ok(Phase::CopyOrder {
                    slot,
                    offset: offset + 1,
                })
            }
            Phase::TransferOrders => self.prepare_order_transfers(),
            Phase::References => {
                let arena = self.arena();
                let p = self.plan.positions;
                if self.command().creates() {
                    let created = HtmlBuilderNode {
                        identity: Arc::downgrade(&arena.identity),
                        index: p[0],
                    };
                    let owner = self.plan.template.then(|| arena.id(p[0]));
                    self.plan.created = Some(created);
                    self.plan.ids[0] = owner;
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
                        self.plan.ids = [Some(host_id), Some(contents_id), Some(owner)];
                    }
                    Command::ReassignTemplateContents { .. } => {
                        self.plan.ids[0] = Some(arena.id(p[0]))
                    }
                    _ => unreachable!("fixed reference-sharing plan"),
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
                    _ => unreachable!("only real new attributes are counted"),
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
                    unreachable!("old attribute kind checked");
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
            Phase::Order {
                parent,
                offset,
                old,
            } => {
                let children = &self.arena().records[parent].children;
                if offset < children.len() {
                    let value = children[offset];
                    if old && value == self.plan.positions[0] {
                        if self.plan.old_found {
                            return Err(HtmlConstructionError::ParentChildConflict);
                        }
                        self.plan.old_found = true;
                        self.plan.old_position = offset;
                    }
                    if !old
                        && self.command().operand(2).is_some()
                        && value == self.plan.positions[2]
                    {
                        if self.plan.before_found {
                            return Err(HtmlConstructionError::ParentChildConflict);
                        }
                        self.plan.before_found = true;
                        self.plan.insertion_position = offset;
                    }
                    return Ok(Phase::Order {
                        parent,
                        offset: offset + 1,
                        old,
                    });
                }
                if old {
                    if !self.plan.old_found {
                        return Err(HtmlConstructionError::ParentChildConflict);
                    }
                    if matches!(self.command(), Command::Insert { .. }) {
                        return Ok(Phase::Order {
                            parent: self.plan.positions[1],
                            offset: 0,
                            old: false,
                        });
                    }
                    let mut plan = std::mem::take(&mut self.plan);
                    let result = self.after_topology(&mut plan);
                    self.plan = plan;
                    result
                } else {
                    if self.command().operand(2).is_some() && !self.plan.before_found {
                        return Err(HtmlConstructionError::WrongSibling);
                    }
                    if self.plan.old_parent == Some(parent)
                        && self.plan.old_position < self.plan.insertion_position
                    {
                        self.plan.insertion_position -= 1;
                    }
                    Ok(Phase::Cycle {
                        cursor: Some(parent),
                        target: self.plan.positions[0],
                        steps: 0,
                    })
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
                    return Err(HtmlConstructionError::Cycle);
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
                            return Err(HtmlConstructionError::Cycle);
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
                let record = &self.arena().records[parent];
                if let Some(&child) = record.children.get(offset) {
                    let target = self
                        .arena()
                        .records
                        .get(child)
                        .ok_or(HtmlConstructionError::UnknownNode)?;
                    if target.parent != Some(parent)
                        || target.owner.is_some()
                        || matches!(
                            target.kind,
                            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment
                        )
                    {
                        return Err(HtmlConstructionError::ParentChildConflict);
                    }
                    Ok(Phase::PublishChild {
                        parent,
                        offset: offset + 1,
                    })
                } else if let Some(parent_index) = record.parent {
                    Ok(Phase::PublishParent {
                        node: parent,
                        parent: parent_index,
                        offset: 0,
                        found: 0,
                    })
                } else {
                    Ok(Phase::PublishNode(parent + 1))
                }
            }
            Phase::PublishParent {
                node,
                parent,
                offset,
                found,
            } => {
                let children = &self.arena().records[parent].children;
                if let Some(&child) = children.get(offset) {
                    Ok(Phase::PublishParent {
                        node,
                        parent,
                        offset: offset + 1,
                        found: found + usize::from(child == node),
                    })
                } else if found != 1 {
                    Err(HtmlConstructionError::ParentChildConflict)
                } else {
                    Ok(Phase::PublishNode(node + 1))
                }
            }
            Phase::Depth {
                node,
                cursor,
                depth,
                steps,
            } => {
                if steps >= self.arena().records.len() {
                    return Err(HtmlConstructionError::Cycle);
                }
                if let Some(parent) = self.virtual_parent(cursor) {
                    if parent >= self.arena().records.len() {
                        return Err(HtmlConstructionError::UnknownNode);
                    }
                    Ok(Phase::Depth {
                        node,
                        cursor: parent,
                        depth: add(depth, 1)?,
                        steps: steps + 1,
                    })
                } else {
                    self.plan.max_depth = self.plan.max_depth.max(depth);
                    if node + 1 < self.arena().records.len() {
                        Ok(Phase::Depth {
                            node: node + 1,
                            cursor: node + 1,
                            depth: 0,
                            steps: 0,
                        })
                    } else {
                        self.plan.usage.depth = self.plan.max_depth;
                        self.before_writes()
                    }
                }
            }
            Phase::Preflight => Ok(Phase::Layout),
            Phase::Commit => Ok(Phase::Ready),
            _ => Err(HtmlConstructionError::OperationNotReady),
        }
    }
}

fn validate_record(arena: &Arena, index: usize) -> Result<(), HtmlConstructionError> {
    let record = &arena.records[index];
    if let Some(parent) = record.parent {
        let parent = arena
            .records
            .get(parent)
            .ok_or(HtmlConstructionError::UnknownNode)?;
        if !matches!(
            parent.kind,
            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment | HtmlNodeKind::Element(_)
        ) || parent.template_contents.is_some()
        {
            return Err(HtmlConstructionError::ParentChildConflict);
        }
    }
    if !record.children.is_empty()
        && (!matches!(
            record.kind,
            HtmlNodeKind::Document(_) | HtmlNodeKind::Fragment | HtmlNodeKind::Element(_)
        ) || record.template_contents.is_some())
    {
        return Err(HtmlConstructionError::ParentChildConflict);
    }
    if record.shadow_properties.is_some() && !matches!(record.kind, HtmlNodeKind::Fragment) {
        return Err(HtmlConstructionError::InvalidShadowAssociation);
    }
    if let Some(contents) = record.template_contents {
        if !matches!(record.kind, HtmlNodeKind::Element(_)) {
            return Err(HtmlConstructionError::InvalidTemplateAssociation);
        }
        let target = arena
            .records
            .get(contents.position())
            .ok_or(HtmlConstructionError::UnknownNode)?;
        if !matches!(target.kind, HtmlNodeKind::Fragment) || target.parent.is_some() {
            return Err(HtmlConstructionError::InvalidTemplateAssociation);
        }
        match contents {
            TemplateContents::Owned(_) => {
                if target.shadow_properties.is_some()
                    || !matches!(&target.owner,Some(HtmlFragmentOwner::Template(owner)) if owner.index==index && Arc::ptr_eq(&owner.identity,&arena.identity))
                {
                    return Err(HtmlConstructionError::InvalidTemplateAssociation);
                }
            }
            TemplateContents::ShadowReference(_) => {
                if target.shadow_properties.is_none() {
                    return Err(HtmlConstructionError::InvalidContentsReference);
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
            return Err(HtmlConstructionError::InvalidShadowAssociation);
        }
        let target = arena
            .records
            .get(shadow.contents.index)
            .ok_or(HtmlConstructionError::UnknownNode)?;
        if target.shadow_properties.is_none()
            || target.parent.is_some()
            || !matches!(&target.owner,Some(HtmlFragmentOwner::ShadowHost(owner)) if owner==&shadow.host)
        {
            return Err(HtmlConstructionError::InvalidShadowAssociation);
        }
    }
    if let Some(owner) = &record.owner {
        if !matches!(record.kind, HtmlNodeKind::Fragment) || record.parent.is_some() {
            return Err(HtmlConstructionError::AssociationAlreadyOwned);
        }
        let id = match owner {
            HtmlFragmentOwner::Template(id) | HtmlFragmentOwner::ShadowHost(id) => id,
        };
        if !Arc::ptr_eq(&id.identity, &arena.identity) {
            return Err(HtmlConstructionError::ForeignBuilder);
        }
        let target = arena
            .records
            .get(id.index)
            .ok_or(HtmlConstructionError::UnknownNode)?;
        match owner {
            HtmlFragmentOwner::Template(_) => {
                if target.template_contents != Some(TemplateContents::Owned(index))
                    || record.shadow_properties.is_some()
                {
                    return Err(HtmlConstructionError::InvalidTemplateAssociation);
                }
            }
            HtmlFragmentOwner::ShadowHost(_) => {
                if !matches!(&target.shadow,Some(shadow) if shadow.contents.index==index)
                    || record.shadow_properties.is_none()
                {
                    return Err(HtmlConstructionError::InvalidShadowAssociation);
                }
            }
        }
    }
    Ok(())
}

impl HtmlConstructionReadyOperation {
    pub fn abort(self) -> HtmlConstructionAbort {
        self.operation.abort()
    }
    /// Applies only the fixed writes and owning moves described before readiness.
    pub fn commit(mut self) -> HtmlConstructionCommit {
        use HtmlConstructionCommand as Command;
        let operation = &mut self.operation;
        if matches!(operation.command(), Command::Publish { .. }) {
            let StorageOwner::Shared(arena) = operation.owner.take().expect("ready publication")
            else {
                unreachable!("publication provider completed");
            };
            let root = operation.plan.positions[0];
            let result = if operation.plan.output_document {
                HtmlConstructionResult::Document(HtmlDocument { arena, root })
            } else {
                HtmlConstructionResult::Fragment(HtmlFragment { arena, root })
            };
            return HtmlConstructionCommit::Published {
                result,
                lease: operation.lease.take().expect("observed actual lease"),
            };
        }
        let StorageOwner::Mutable(mut storage) = operation.owner.take().expect("ready storage")
        else {
            unreachable!("only publication shares an arena");
        };
        let command = match operation.command.take().expect("ready command") {
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
        let attach_shadow = matches!(command, Command::AttachShadow { .. });
        let changes_topology = matches!(
            command,
            Command::Insert { .. }
                | Command::Detach { .. }
                | Command::AttachShadow { .. }
                | Command::AttachTemplateShadow { .. }
                | Command::DetachShadow { .. }
                | Command::SetTemplateShadowContents { .. }
                | Command::ReassignTemplateContents { .. }
        );
        let plan = &mut operation.plan;
        let p = plan.positions;
        let mut retired_orders = HtmlConstructionOrderRetirement::default();
        if !plan.no_op {
            let arena = &mut storage.arena;
            match command {
                Command::CreateElement { .. } => unreachable!("element operands formed at Ready"),
                Command::ReserveRecords { .. } | Command::CheckChildTransfer { .. } => {}
                Command::Create { kind } => {
                    let mut record = Record::new(kind, None, 0);
                    record.children = std::mem::take(&mut plan.new_children[0]);
                    record.children_generation = plan.new_children_generations[0];
                    if plan.template {
                        record.template_contents = Some(TemplateContents::Owned(p[0] + 1));
                    }
                    arena.records.push(record);
                    if plan.template {
                        let mut contents = Record::new(HtmlNodeKind::Fragment, None, 1);
                        contents.children = std::mem::take(&mut plan.new_children[1]);
                        contents.children_generation = plan.new_children_generations[1];
                        contents.owner = Some(HtmlFragmentOwner::Template(
                            plan.ids[0].take().expect("preadmitted template owner"),
                        ));
                        arena.records.push(contents);
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
                }
                Command::CreateFragment | Command::CreateShadowContents { .. } => {
                    let mut record = Record::new(HtmlNodeKind::Fragment, None, 0);
                    record.children = std::mem::take(&mut plan.new_children[0]);
                    record.children_generation = plan.new_children_generations[0];
                    if let Command::CreateShadowContents { properties } = command {
                        record.shadow_properties = Some(properties);
                    }
                    arena.records.push(record);
                }
                Command::Insert { .. } => {
                    retired_orders = plan.commit_orders(arena);
                    arena.records[p[0]].parent = Some(p[1]);
                }
                Command::Detach { .. } => {
                    retired_orders = plan.commit_orders(arena);
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
                Command::AttachShadow { .. } | Command::AttachTemplateShadow { .. } => {
                    let (host, contents) = if matches!(command, Command::AttachShadow { .. }) {
                        (p[0], p[1])
                    } else {
                        (p[1], p[2])
                    };
                    if let Some(old) = plan.old_contents {
                        arena.records[old].owner = None;
                        arena.records[p[0]].template_contents =
                            Some(TemplateContents::ShadowReference(contents));
                    }
                    arena.records[contents].owner = Some(HtmlFragmentOwner::ShadowHost(
                        plan.ids[2].take().expect("preadmitted shadow owner"),
                    ));
                    arena.records[host].shadow = Some(ShadowAssociation {
                        host: plan.ids[0].take().expect("preadmitted host ID"),
                        contents: plan.ids[1].take().expect("preadmitted contents ID"),
                    });
                }
                Command::DetachShadow { .. } => {
                    arena.records[plan.old_contents.expect("ready shadow detach")].owner = None;
                    arena.records[p[0]].shadow = None;
                }
                Command::SetTemplateShadowContents { .. } => {
                    arena.records[plan.old_contents.expect("ordinary owning contents")].owner =
                        None;
                    arena.records[p[0]].template_contents =
                        Some(TemplateContents::ShadowReference(p[1]));
                }
                Command::ReassignTemplateContents { .. } => {
                    if let Some(old) = plan.old_contents {
                        arena.records[old].owner = None;
                    }
                    arena.records[p[0]].template_contents = Some(TemplateContents::Owned(p[1]));
                    arena.records[p[1]].owner = Some(HtmlFragmentOwner::Template(
                        plan.ids[0].take().expect("preadmitted template ID"),
                    ));
                }
                Command::SetDocumentMode { mode, .. } => {
                    arena.records[p[0]].kind = HtmlNodeKind::Document(mode)
                }
                Command::SetLinkFlags { flags, .. } => {
                    let HtmlNodeKind::Element(element) = &mut arena.records[p[0]].kind else {
                        unreachable!("checked link element");
                    };
                    element.link_flags = Some(flags);
                }
                Command::Publish { .. } => unreachable!("publication handled separately"),
            }
            if let Some(depth) = plan.fast_depth {
                let index = if attach_shadow { p[1] } else { p[0] };
                arena.records[index].depth = depth;
                if let Some(TemplateContents::Owned(contents)) =
                    arena.records[index].template_contents
                {
                    arena.records[contents].depth = depth + 1;
                }
            } else if changes_topology {
                arena.depths_valid = false;
            }
            arena.usage = plan.usage;
        }
        HtmlConstructionCommit::Changed {
            storage,
            created: plan.created.take(),
            transfers: std::mem::take(&mut plan.transfers),
            retired_orders,
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum RetirementPhase {
    Children(usize),
    Payload(usize),
    Records,
    Identity,
    Complete,
}
/// Deterministic cleanup of the original banks; it never resumes construction.
pub struct HtmlConstructionRetirement {
    storage: Option<HtmlConstructionStorage>,
    phase: RetirementPhase,
}
pub struct HtmlConstructionRetirementAction<'a> {
    owner: &'a mut HtmlConstructionRetirement,
    snapshot: Option<HtmlConstructionAllocationSnapshot>,
}
#[derive(Debug)]
pub enum HtmlConstructionRetirementDescription<'a> {
    Allocation(&'a HtmlConstructionAllocationSnapshot),
    Payload {
        position: usize,
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
            storage: Some(storage),
            phase,
        }
    }
    pub fn next_action(
        &mut self,
    ) -> Result<Option<HtmlConstructionRetirementAction<'_>>, HtmlConstructionError> {
        let snapshot = match self.phase {
            RetirementPhase::Complete => return Ok(None),
            RetirementPhase::Children(index) => Some(
                self.storage
                    .as_ref()
                    .expect("cleanup owner")
                    .children_snapshot(index)?,
            ),
            RetirementPhase::Payload(_) => None,
            RetirementPhase::Records => Some(
                self.storage
                    .as_ref()
                    .expect("cleanup owner")
                    .records_snapshot()?,
            ),
            RetirementPhase::Identity => {
                Some(HtmlConstructionAllocationSnapshot::shared::<Identity>(
                    &self.storage.as_ref().expect("cleanup owner").arena.identity,
                    AllocationRole::Identity,
                    0,
                ))
            }
        };
        Ok(Some(HtmlConstructionRetirementAction {
            owner: self,
            snapshot,
        }))
    }
}
impl HtmlConstructionRetirementAction<'_> {
    pub fn description(&self) -> HtmlConstructionRetirementDescription<'_> {
        match self.owner.phase {
            RetirementPhase::Payload(position) => HtmlConstructionRetirementDescription::Payload {
                position,
                initialized_bytes: size_of::<Record>(),
            },
            _ => HtmlConstructionRetirementDescription::Allocation(
                self.snapshot.as_ref().expect("actual cleanup allocation"),
            ),
        }
    }
    pub fn perform(self) -> Option<HtmlConstructionRetired> {
        match self.owner.phase {
            RetirementPhase::Children(index) => {
                drop(std::mem::take(
                    &mut self
                        .owner
                        .storage
                        .as_mut()
                        .expect("cleanup owner")
                        .arena
                        .records[index]
                        .children,
                ));
                self.owner.phase = RetirementPhase::Payload(index);
            }
            RetirementPhase::Payload(index) => {
                let record = &mut self
                    .owner
                    .storage
                    .as_mut()
                    .expect("cleanup owner")
                    .arena
                    .records[index];
                record.kind = HtmlNodeKind::Fragment;
                record.document_focused_area_designation = None;
                record.owner = None;
                record.shadow = None;
                record.template_contents = None;
                record.shadow_properties = None;
                self.owner.phase = if index == 0 {
                    RetirementPhase::Records
                } else {
                    RetirementPhase::Children(index - 1)
                };
            }
            RetirementPhase::Records => {
                drop(std::mem::take(
                    &mut self
                        .owner
                        .storage
                        .as_mut()
                        .expect("cleanup owner")
                        .arena
                        .records,
                ));
                self.owner.phase = RetirementPhase::Identity;
            }
            RetirementPhase::Identity => {
                drop(self.owner.storage.take().expect("cleanup owner"));
                self.owner.phase = RetirementPhase::Complete;
            }
            RetirementPhase::Complete => unreachable!("completed cleanup cannot issue an action"),
        }
        self.snapshot
            .map(|snapshot| HtmlConstructionRetired { snapshot })
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
        HtmlConstructionOperation::try_ready(&mut Some(operation))
            .unwrap()
            .commit()
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
        let storage = HtmlConstructionInitialization::try_ready(&mut initialization)
            .unwrap()
            .commit();
        let identity = storage.arena.identity.clone();
        let HtmlConstructionCommit::Changed { storage, .. } = drive(
            storage
                .prepare(HtmlConstructionCommand::ReserveRecords { additional: 4 })
                .unwrap_or_else(|_| panic!("retained reserve command")),
            &identity,
        ) else {
            panic!("reserve retains actual storage")
        };
        let HtmlConstructionCommit::Changed {
            storage,
            created: Some(root),
            ..
        } = drive(
            storage
                .prepare(HtmlConstructionCommand::CreateFragment)
                .unwrap_or_else(|_| panic!("retained create command")),
            &identity,
        )
        else {
            panic!("created actual root")
        };
        let HtmlConstructionCommit::Published { result, lease } = drive(
            storage
                .prepare_finish(root)
                .unwrap_or_else(|_| panic!("retained publish command")),
            &identity,
        ) else {
            panic!("published actual storage")
        };
        let recovery =
            HtmlConstructionLease::try_recover(&mut Some(result), &mut Some(lease)).unwrap();
        assert_eq!(recovery.storage.len(), 1);
    }
}
