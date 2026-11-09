//! Drives HTML's sole record owner through this session's original controller.
use super::*;
use crate::parser::control::ConstructionAllocation;
use surgeist_html::bounded::construction::{
    HtmlConstructionAbort, HtmlConstructionAction, HtmlConstructionActionDescription,
    HtmlConstructionAllocationRole, HtmlConstructionAttributeEdit, HtmlConstructionCommand,
    HtmlConstructionCommit, HtmlConstructionGrowth, HtmlConstructionLease, HtmlConstructionNode,
    HtmlConstructionOperation, HtmlConstructionReadyOperation, HtmlConstructionRecovery,
    HtmlConstructionResult, HtmlConstructionRetirementDescription, HtmlConstructionStorage,
};
use surgeist_html::bounded::values::{HtmlAttributeNameIndex, HtmlNameIndexSlot};
use surgeist_html::construction::{HtmlBuilderNode, HtmlConstructionError, HtmlConstructionUsage};
mod attributes;
#[cfg(test)]
mod tests;
pub(super) use attributes::{
    NAME_INDEX_MIN_ENTRIES, NAME_INDEX_SPARE_ENTRIES, prepare_attribute_slot,
};

pub(super) struct ConstructionOwner {
    control: ControlIdentity,
    pub(super) owner: Option<HtmlConstructionStorage>,
    attribute_edit: Option<HtmlConstructionAttributeEdit>,
    attribute_plan: Option<ConstructionAllocation>,
    /// The table of attribute names that edits of a long attribute list borrow, and the ticket
    /// of its slots. The table is lent to an edit while one runs, so the ticket stays here.
    name_index: Option<HtmlAttributeNameIndex>,
    name_index_slots: Option<VectorCapacity<HtmlNameIndexSlot>>,
    identity: Option<ConstructionAllocation>,
    records: Option<ConstructionAllocation>,
    children: Vec<Option<ConstructionAllocation>>,
    children_allocation: VectorCapacity<Option<ConstructionAllocation>>,
    wrapper: Option<ConstructionAllocation>,
    /// Hides the attribute edit the next `begin_attributes` starts, as if its
    /// custody slot had been emptied behind the owner's back.
    #[cfg(test)]
    hide_next_attribute_edit: bool,
    #[cfg(test)]
    hidden_attribute_edit: Option<HtmlConstructionAttributeEdit>,
}

#[derive(Clone, Copy)]
enum Bank {
    Identity,
    Records,
    Children(usize),
    Wrapper,
}

/// What the construction owner's error violates, one fault per cause it can
/// report today, plus the cause this parser does not yet name.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OwnerFault {
    ForeignBuilder,
    ParentCannotHaveChildren,
    TemplateContentsRequired,
    ShadowHostMustBeElement,
    ShadowAlreadyAttached,
    UnknownNode,
    WrongNodeKind,
    RootMove,
    AssociationMove,
    WrongSibling,
    Cycle,
    AssociationAlreadyOwned,
    InvalidTemplateAssociation,
    DuplicateAttribute,
    AttributeNameMismatch,
    AttributeOrderMismatch,
    InvalidShadowAssociation,
    InvalidContentsReference,
    ParentChildConflict,
    InvalidOutputRoot,
    OperationNotReady,
    ForeignAllocation,
    CapacityMismatch,
    IndexUnavailable,
    SharedAllocationAliased,
    ReleaseUnderflow,
    LimitExceeded,
    RepresentationExhausted,
    AllocationFailed,
    /// A cause that the owner added after this parser classified its causes.
    Unlisted,
}

/// A predicate of the owner's error cause.
type CausePredicate = fn(&HtmlConstructionError) -> bool;

/// The owner's cause predicates, each with the fault it names.
const OWNER_CAUSES: [(CausePredicate, OwnerFault); 29] = [
    (
        HtmlConstructionError::is_foreign_builder,
        OwnerFault::ForeignBuilder,
    ),
    (
        HtmlConstructionError::is_parent_cannot_have_children,
        OwnerFault::ParentCannotHaveChildren,
    ),
    (
        HtmlConstructionError::is_template_contents_required,
        OwnerFault::TemplateContentsRequired,
    ),
    (
        HtmlConstructionError::is_shadow_host_must_be_element,
        OwnerFault::ShadowHostMustBeElement,
    ),
    (
        HtmlConstructionError::is_shadow_already_attached,
        OwnerFault::ShadowAlreadyAttached,
    ),
    (
        HtmlConstructionError::is_unknown_node,
        OwnerFault::UnknownNode,
    ),
    (
        HtmlConstructionError::is_wrong_node_kind,
        OwnerFault::WrongNodeKind,
    ),
    (HtmlConstructionError::is_root_move, OwnerFault::RootMove),
    (
        HtmlConstructionError::is_association_move,
        OwnerFault::AssociationMove,
    ),
    (
        HtmlConstructionError::is_wrong_sibling,
        OwnerFault::WrongSibling,
    ),
    (HtmlConstructionError::is_cycle, OwnerFault::Cycle),
    (
        HtmlConstructionError::is_association_already_owned,
        OwnerFault::AssociationAlreadyOwned,
    ),
    (
        HtmlConstructionError::is_invalid_template_association,
        OwnerFault::InvalidTemplateAssociation,
    ),
    (
        HtmlConstructionError::is_duplicate_attribute,
        OwnerFault::DuplicateAttribute,
    ),
    (
        HtmlConstructionError::is_attribute_name_mismatch,
        OwnerFault::AttributeNameMismatch,
    ),
    (
        HtmlConstructionError::is_attribute_order_mismatch,
        OwnerFault::AttributeOrderMismatch,
    ),
    (
        HtmlConstructionError::is_invalid_shadow_association,
        OwnerFault::InvalidShadowAssociation,
    ),
    (
        HtmlConstructionError::is_invalid_contents_reference,
        OwnerFault::InvalidContentsReference,
    ),
    (
        HtmlConstructionError::is_parent_child_conflict,
        OwnerFault::ParentChildConflict,
    ),
    (
        HtmlConstructionError::is_invalid_output_root,
        OwnerFault::InvalidOutputRoot,
    ),
    (
        HtmlConstructionError::is_operation_not_ready,
        OwnerFault::OperationNotReady,
    ),
    (
        HtmlConstructionError::is_foreign_allocation,
        OwnerFault::ForeignAllocation,
    ),
    (
        HtmlConstructionError::is_capacity_mismatch,
        OwnerFault::CapacityMismatch,
    ),
    (
        HtmlConstructionError::is_index_unavailable,
        OwnerFault::IndexUnavailable,
    ),
    (
        HtmlConstructionError::is_shared_allocation_aliased,
        OwnerFault::SharedAllocationAliased,
    ),
    (
        HtmlConstructionError::is_release_underflow,
        OwnerFault::ReleaseUnderflow,
    ),
    (
        HtmlConstructionError::is_limit_exceeded,
        OwnerFault::LimitExceeded,
    ),
    (
        HtmlConstructionError::is_representation_exhausted,
        OwnerFault::RepresentationExhausted,
    ),
    (
        HtmlConstructionError::is_allocation_failed,
        OwnerFault::AllocationFailed,
    ),
];

/// Names the fault of an owner error through the owner's predicates.
fn classify(error: &HtmlConstructionError) -> OwnerFault {
    OWNER_CAUSES
        .iter()
        .find(|(recognizes, _)| recognizes(error))
        .map_or(OwnerFault::Unlisted, |&(_, fault)| fault)
}

/// How a fault is reported to the session.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Disposition {
    /// A document-shaped error that `session_class` classifies; nothing is
    /// latched here.
    Session(SessionError),
    /// The class a stage error of the same kind would carry, latched through it.
    Class(SessionClass),
    /// A count that cannot be represented.
    Representation,
    /// An allocation whose layout is unknown here.
    AllocationFailed,
}

/// Maps every fault to its report. A new fault needs a row here to compile.
fn disposition(fault: OwnerFault) -> Disposition {
    use crate::parser::control::HtmlParseInvariant;
    use Disposition::{AllocationFailed, Class, Representation, Session};
    match fault {
        OwnerFault::ForeignBuilder | OwnerFault::UnknownNode => Session(SessionError::ForeignNode),
        OwnerFault::RootMove => Session(SessionError::RootMove),
        OwnerFault::AssociationMove => Session(SessionError::AssociationMove),
        OwnerFault::ParentCannotHaveChildren => Session(SessionError::NotContainer),
        OwnerFault::TemplateContentsRequired => Session(SessionError::TemplateContentsRequired),
        OwnerFault::WrongSibling | OwnerFault::ParentChildConflict => {
            Session(SessionError::WrongSibling)
        }
        OwnerFault::Cycle => Session(SessionError::Cycle),
        OwnerFault::SharedAllocationAliased => Class(SessionClass::Fault(
            HtmlParseInvariant::SharedAllocationAliased,
        )),
        OwnerFault::RepresentationExhausted => Representation,
        OwnerFault::CapacityMismatch
        | OwnerFault::ForeignAllocation
        | OwnerFault::LimitExceeded => {
            Class(SessionClass::Fault(HtmlParseInvariant::CapacityMismatch))
        }
        OwnerFault::ReleaseUnderflow => {
            Class(SessionClass::Fault(HtmlParseInvariant::ReleaseUnderflow))
        }
        // `layout_bytes: 0` means the layout is unknown here, not that none was
        // requested. Every bounded producer of this error reports it with its
        // real layout before it gets this far: the record and child reserves
        // in `ConstructionOwner::perform_action_with_commit`, the name-decision
        // and attribute-slot reserves in `attributes.rs`, and the candidate
        // backing reserve in `text_append.rs`. This is only a failure a
        // caller without a layout still has to report.
        OwnerFault::AllocationFailed => AllocationFailed,
        OwnerFault::ShadowHostMustBeElement
        | OwnerFault::ShadowAlreadyAttached
        | OwnerFault::WrongNodeKind
        | OwnerFault::AssociationAlreadyOwned
        | OwnerFault::InvalidTemplateAssociation
        | OwnerFault::DuplicateAttribute
        | OwnerFault::AttributeNameMismatch
        | OwnerFault::AttributeOrderMismatch
        | OwnerFault::InvalidShadowAssociation
        | OwnerFault::InvalidContentsReference
        | OwnerFault::InvalidOutputRoot
        | OwnerFault::OperationNotReady
        | OwnerFault::IndexUnavailable
        | OwnerFault::Unlisted => Class(SessionClass::Fault(
            HtmlParseInvariant::ConstructionOwnerRefusal,
        )),
    }
}

/// The session error for a refusal from HTML's construction owner, with every
/// cause of its error classified by what it violates.
///
/// There are two tables, and the second is built on the first.
/// [`session_class`] classifies every session error; `OWNER_CAUSES` and
/// [`disposition`] classify the construction owner's causes, which are not
/// session errors until this function converts them. A document-shaped
/// refusal converts to the [`SessionError`] that `session_class` already
/// classifies, and nothing is latched here. Every other cause converts to the
/// [`SessionClass`] that a stage error would carry and is latched through
/// [`SessionClass::latch`], so the stop and the failure's reason derive from
/// the same class (`StopCause`). Two environment stops, a count that cannot be
/// represented and an allocation that failed, are latched directly, as their
/// controller owners latch them. A test checks that no document-shaped
/// disposition disagrees with `session_class`.
///
/// - Handle, placement and cycle refusals are errors about the document's own
///   shape; their callers classify them and nothing is latched here.
/// - A payload that was aliased and a count that cannot be represented keep
///   their own stops.
/// - A ticket, layout or limit that disagrees with what the controller
///   charged, and a release beyond what was held, are accounting faults. The
///   parse controller admits every resource before the owner is asked, so a
///   limit the owner reports is the owner's accounting disagreeing with it.
/// - An allocation that failed is the environment refusing, a reached stop
///   that never names an invariant. The sites that know the layout classify
///   it themselves first, so none is known here.
/// - Every other refusal is the owner declining a request that the parser's
///   sequencing guarantees valid.
///
/// The owner keeps its error's cause private and open to later causes, so the
/// compiler cannot make this crate classify a new one. [`classify`] names each
/// current cause through the owner's predicates and [`disposition`] maps every
/// named fault exhaustively. A cause neither names is a fault of the parser,
/// never an outcome of the document: every cause that parsed input can reach
/// (a limit, an allocation, a count) is named, so `Unlisted` reports the
/// construction-owner invariant and cannot end a parse as a document outcome.
/// The tests cover what they can reach: every fault's disposition, the table
/// naming each fault once, and the errors a builder provokes classifying
/// through exactly one predicate. That the table lists all 29 causes of the
/// owner's error rests on the owner's own cause tests, which pin one predicate
/// per cause, and on this table's fixed length; the parser cannot build every
/// cause through the public API.
pub(super) fn construction_error(
    control: &mut ParseControl,
    error: HtmlConstructionError,
) -> SessionError {
    report(control, disposition(classify(&error)))
}

/// Reports a disposition to the session, latching a stop where it names one.
fn report(control: &mut ParseControl, disposition: Disposition) -> SessionError {
    match disposition {
        Disposition::Session(error) => error,
        Disposition::Class(class) => SessionError::Stop(class.latch(control)),
        Disposition::Representation => {
            SessionError::Stop(control.stop_representation(HtmlParseResource::ManagedCapacityBytes))
        }
        Disposition::AllocationFailed => {
            SessionError::Stop(control.construction_allocation_failed(0))
        }
    }
}

pub(super) struct PendingConstruction {
    operation: Option<HtmlConstructionOperation>,
    before_edges: usize,
}
impl PendingConstruction {
    pub(super) fn action(
        &mut self,
        control: &mut ParseControl,
    ) -> Result<Option<HtmlConstructionAction<'_>>, SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 3)?;
        let Some(operation) = self.operation.as_mut() else {
            return Err(latch_registry_row(control));
        };
        operation
            .next_action()
            .map_err(|error| construction_error(control, error))
    }
    pub(super) fn before_edges(&self) -> usize {
        self.before_edges
    }
}
fn ticket_work(control: &mut ParseControl) -> Result<(), SessionError> {
    control.charge(HtmlParseWorkCategory::ProviderCall, 2)?;
    control.charge(
        HtmlParseWorkCategory::CopyByte,
        mem::size_of::<ConstructionAllocation>() as u64,
    )?;
    control.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
    Ok(())
}
impl ConstructionOwner {
    pub(super) fn new(control: &mut ParseControl) -> Result<Self, SessionError> {
        control.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
        let mut initialization = HtmlConstructionStorage::begin();
        let Some(action) = initialization
            .next_action()
            .map_err(|error| construction_error(control, error))?
        else {
            return Err(latch_owner_refusal(control));
        };
        let description = action.description();
        let HtmlConstructionActionDescription::InitializeIdentity { payload_bytes, .. } =
            description
        else {
            return Err(latch_owner_refusal(control));
        };
        control.prepare_construction_share(payload_bytes)?;
        // The first call was charged above; a plan that counts none differs
        // from the work already charged for it.
        description
            .charge_identity(1, control)
            .map_err(|error| charge_fault(control, error))?;
        ticket_work(control)?;
        let observation = action.perform();
        let mut identity = None;
        if let Err(stop) = control.observe_construction_allocation(
            observation.allocation(),
            &mut identity,
            payload_bytes,
        ) {
            drop(observation);
            if let Some(receipt) = initialization.abort() {
                let Some(ticket) = identity.take() else {
                    return Err(latch_registry_row(control));
                };
                control.retire_construction_allocation(ticket, receipt)?;
            }
            return Err(stop.into());
        }
        observation.accept();
        let owner = initialization
            .try_ready()
            .map_err(|refusal| construction_error(control, refusal.into_parts().1))?
            .commit();
        let (children, children_allocation) = match control.new_vector() {
            Ok(value) => value,
            Err(stop) => {
                let mut retirement = owner.retire();
                // Every bank is retired even after an impossible one: the
                // walk only advances by performing its actions.
                let mut impossible = false;
                while let Some(action) = retirement
                    .next_action()
                    .map_err(|error| construction_error(control, error))?
                {
                    let is_identity = matches!(action.description(),HtmlConstructionRetirementDescription::Allocation(value) if matches!(value.role(),HtmlConstructionAllocationRole::Identity));
                    if let Some(receipt) = action.perform() {
                        if is_identity {
                            match identity.take() {
                                Some(ticket) => {
                                    control.retire_construction_allocation(ticket, receipt)?;
                                }
                                None => impossible = true,
                            }
                        } else if receipt.allocation().payload_bytes() != 0 {
                            // An empty initialization record bank holds nothing.
                            impossible = true;
                        }
                    }
                }
                if impossible {
                    return Err(latch_registry_row(control));
                }
                return Err(stop.into());
            }
        };
        Ok(Self {
            control: control.identity(),
            owner: Some(owner),
            attribute_edit: None,
            attribute_plan: None,
            name_index: None,
            name_index_slots: None,
            identity,
            records: None,
            children,
            children_allocation,
            wrapper: None,
            #[cfg(test)]
            hide_next_attribute_edit: false,
            #[cfg(test)]
            hidden_attribute_edit: None,
        })
    }
    pub(super) fn len(&self, control: &mut ParseControl) -> Result<usize, SessionError> {
        Ok(self.storage(control)?.len())
    }
    /// The storage while it is at rest or lent to an attribute edit. It is
    /// absent only while a pending operation or a publication holds it, and a
    /// caller that reads it then has met an impossible custody state.
    fn resident(&self) -> Option<&HtmlConstructionStorage> {
        self.owner.as_ref().or_else(|| {
            self.attribute_edit
                .as_ref()
                .map(HtmlConstructionAttributeEdit::storage_view)
        })
    }
    pub(super) fn storage(
        &self,
        control: &mut ParseControl,
    ) -> Result<&HtmlConstructionStorage, SessionError> {
        self.resident().ok_or_else(|| latch_registry_row(control))
    }
    pub(super) fn reserve_child_tickets(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
    ) -> Result<(), SessionError> {
        control.reserve_vector_geometric(
            &mut self.children,
            &mut self.children_allocation,
            additional,
        )?;
        Ok(())
    }
    /// Reserves HTML-owned record storage for `additional` new records with
    /// geometric spare capacity, so a document of N records costs O(N) bytes of
    /// reservation work instead of O(N^2).
    pub(super) fn reserve_records(
        &mut self,
        control: &mut ParseControl,
        additional: usize,
    ) -> Result<(), SessionError> {
        let len = self.len(control)?;
        let additional =
            control.geometric_construction_additional(self.records.as_ref(), len, additional)?;
        self.apply(
            control,
            HtmlConstructionCommand::ReserveRecords { additional },
            &[],
        )?;
        Ok(())
    }
    #[cfg(test)]
    pub(super) fn actual_record_layout(&self) -> (usize, usize) {
        let snapshot = self
            .records
            .as_ref()
            .expect("created actual record bank")
            .snapshot();
        (snapshot.capacity(), snapshot.element_size())
    }
    pub(super) fn usage(
        &self,
        control: &mut ParseControl,
    ) -> Result<HtmlConstructionUsage, SessionError> {
        Ok(self.storage(control)?.usage())
    }
    /// The record at `position`; storage that is not at rest holds no record
    /// a caller may name.
    pub(super) fn node_at(&self, position: usize) -> Option<HtmlConstructionNode<'_>> {
        self.resident()?.node_at(position)
    }
    fn allocation_slot(
        &mut self,
        control: &mut ParseControl,
        bank: Bank,
    ) -> Result<&mut Option<ConstructionAllocation>, SessionError> {
        match bank {
            Bank::Identity => Some(&mut self.identity),
            Bank::Records => Some(&mut self.records),
            Bank::Children(position) => self.children.get_mut(position),
            Bank::Wrapper => Some(&mut self.wrapper),
        }
        .ok_or_else(|| latch_registry_row(control))
    }
    pub(super) fn apply(
        &mut self,
        control: &mut ParseControl,
        command: HtmlConstructionCommand,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<Option<HtmlBuilderNode>, SessionError> {
        let committed = self.run(control, command, requests)?;
        self.adopt_mutation(control, committed)
    }
    pub(super) fn adopt_mutation(
        &mut self,
        control: &mut ParseControl,
        committed: HtmlConstructionCommit,
    ) -> Result<Option<HtmlBuilderNode>, SessionError> {
        match committed {
            HtmlConstructionCommit::Changed { storage, created } => {
                self.owner = Some(storage);
                Ok(created)
            }
            HtmlConstructionCommit::Published { result, lease } => {
                // A mutation never publishes. The published storage is taken
                // back through its lease, so the owner keeps every record.
                let invariant = latch_registry_row(control);
                if let Ok(recovery) = HtmlConstructionLease::try_recover(result, lease) {
                    // A refused recovery cannot displace the invariant above,
                    // which is already the reported first stop.
                    let _ = self.recovered(control, recovery);
                }
                Err(invariant)
            }
        }
    }
    pub(super) fn start(
        &mut self,
        control: &mut ParseControl,
        command: HtmlConstructionCommand,
    ) -> Result<PendingConstruction, SessionError> {
        control.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        // Moving bounded inline control metadata is not a payload copy in the
        // parser work model. Actual retained bank/slot writes and providers are
        // admitted by the operation's actions.
        let Some(storage) = self.owner.take() else {
            return Err(latch_registry_row(control));
        };
        let before_edges = storage.usage().edges();
        Ok(PendingConstruction {
            operation: Some(storage.prepare(command)),
            before_edges,
        })
    }
    pub(super) fn perform_action(
        &mut self,
        control: &mut ParseControl,
        action: HtmlConstructionAction<'_>,
        requests: &[(HtmlParseResource, u64)],
        before_edges: usize,
    ) -> Result<bool, SessionError> {
        self.perform_action_with_commit(control, action, requests, before_edges, |_| Ok(()))
    }
    pub(super) fn perform_action_with_commit(
        &mut self,
        control: &mut ParseControl,
        action: HtmlConstructionAction<'_>,
        requests: &[(HtmlParseResource, u64)],
        before_edges: usize,
        before_commit: impl FnOnce(&mut ParseControl) -> Result<(), SessionError>,
    ) -> Result<bool, SessionError> {
        let mut allocation = None;
        let mut expected = 0;
        let mut reserved_capacity = None;
        let mut final_action = false;
        let description = action.description();
        match &description {
            HtmlConstructionActionDescription::Inspect { .. }
            | HtmlConstructionActionDescription::Compare { .. } => description
                .charge_inspection(control)
                .map_err(|error| charge_fault(control, error))?,
            HtmlConstructionActionDescription::PrepareAllocationReport { .. } => description
                .charge_report_preparation(control)
                .map_err(|error| charge_fault(control, error))?,
            HtmlConstructionActionDescription::Reserve {
                allocation: snapshot,
                required_len,
                minimum_capacity,
                growth,
                ..
            } => {
                let (snapshot, required_len, minimum_capacity, growth) =
                    (*snapshot, *required_len, *minimum_capacity, *growth);
                control.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
                let bank = snapshot
                    .child_position()
                    .map_or(Bank::Records, Bank::Children);
                // A geometric bank grows by the parser's own amortized plan,
                // which never reserves spare capacity the limits cannot hold.
                let capacity = match growth {
                    HtmlConstructionGrowth::Exact => minimum_capacity,
                    HtmlConstructionGrowth::Geometric => {
                        let slot = self.allocation_slot(control, bank)?;
                        let additional = control.geometric_construction_additional(
                            slot.as_ref(),
                            snapshot.initialized_len(),
                            required_len.saturating_sub(snapshot.initialized_len()),
                        )?;
                        snapshot
                            .initialized_len()
                            .checked_add(additional)
                            .map(|planned| planned.max(minimum_capacity))
                            .ok_or_else(|| {
                                control.stop_representation(HtmlParseResource::ManagedCapacityBytes)
                            })?
                    }
                };
                reserved_capacity =
                    matches!(growth, HtmlConstructionGrowth::Geometric).then_some(capacity);
                expected = capacity
                    .checked_mul(snapshot.element_size())
                    .ok_or_else(|| {
                        control.stop_representation(HtmlParseResource::ManagedCapacityBytes)
                    })?;
                let slot = self.allocation_slot(control, bank)?;
                control.prepare_construction_reserve(snapshot, capacity, slot.as_ref())?;
                ticket_work(control)?;
                description
                    .charge_report_calls(control)
                    .map_err(|error| charge_fault(control, error))?;
                allocation = Some(bank);
            }
            HtmlConstructionActionDescription::Share {
                allocation: snapshot,
                ..
            } => {
                expected = snapshot.element_size();
                control.prepare_construction_share(expected)?;
                ticket_work(control)?;
                description
                    .charge_report_calls(control)
                    .map_err(|error| charge_fault(control, error))?;
                allocation = Some(Bank::Wrapper);
            }
            HtmlConstructionActionDescription::ShareReferences { .. } => description
                .charge_references(control)
                .map_err(|error| charge_fault(control, error))?,
            HtmlConstructionActionDescription::InitializeChildren { position, .. } => {
                let position = *position;
                control.reserve_vector_geometric(
                    &mut self.children,
                    &mut self.children_allocation,
                    1,
                )?;
                description
                    .charge_children_initialization(control)
                    .map_err(|error| charge_fault(control, error))?;
                control.charge(
                    HtmlParseWorkCategory::CopyByte,
                    const {
                        (mem::size_of::<std::collections::VecDeque<usize>>()
                            + mem::size_of::<Option<ConstructionAllocation>>())
                            as u64
                    },
                )?;
                control.charge(HtmlParseWorkCategory::RecordWrite, 2)?;
                ticket_work(control)?;
                // The real child-bank registry append lands at the next row.
                if position != self.children.len() {
                    return Err(latch_registry_row(control));
                }
                // The zero-capacity bank physically exists in the pending owner;
                // retain its ticket even if a later post-call checkpoint refuses.
                description
                    .charge_children_calls(control)
                    .map_err(|error| charge_fault(control, error))?;
                self.children.push(None);
                allocation = Some(Bank::Children(position));
            }
            HtmlConstructionActionDescription::Preflight { .. } => {
                description
                    .charge_depth(control)
                    .map_err(|error| charge_fault(control, error))?;
                control.check_acquire_many(requests)?;
            }
            HtmlConstructionActionDescription::Commit { usage, .. } => {
                let usage = *usage;
                description
                    .charge_commit(control)
                    .map_err(|error| charge_fault(control, error))?;
                before_commit(control)?;
                control.commit_construction_logical(
                    &self.control,
                    requests,
                    before_edges.saturating_sub(usage.edges()) as u64,
                )?;
                final_action = true;
            }
            HtmlConstructionActionDescription::InitializeIdentity { .. } => {
                // Identity belongs to the separate initialization owner.
                return Err(latch_owner_refusal(control));
            }
        }
        let performed = match reserved_capacity {
            Some(capacity) => action.perform_with_capacity(capacity),
            None => action.perform(),
        };
        match performed {
            Ok(observation) => {
                if let Some(bank) = allocation {
                    let Some(snapshot) = observation.allocation() else {
                        return Err(latch_owner_refusal(control));
                    };
                    let slot = self.allocation_slot(control, bank)?;
                    control.observe_construction_allocation(snapshot, slot, expected)?;
                }
                observation.accept();
                Ok(final_action)
            }
            Err(failure) => {
                if let Some(bank) = allocation
                    && let Some(snapshot) = failure.allocation()
                {
                    let slot = self.allocation_slot(control, bank)?;
                    control.record_construction_allocation(snapshot, slot, expected)?;
                }
                let error = failure.into_parts().0;
                Err(if error.is_allocation_failed() {
                    SessionError::Stop(control.construction_allocation_failed(expected))
                } else {
                    construction_error(control, error)
                })
            }
        }
    }
    pub(super) fn abort_pending(
        &mut self,
        control: &mut ParseControl,
        mut pending: PendingConstruction,
    ) -> Result<(), SessionError> {
        // An operation that is gone took its storage with it; there is none
        // to put back, and the invariant is all that can be reported.
        let Some(operation) = pending.operation.take() else {
            return Err(latch_registry_row(control));
        };
        self.abort_operation(control, operation)
    }
    /// Abandons an operation and takes back its storage. An abort the operation refuses cannot
    /// return the storage; the invariant is all that can be reported.
    fn abort_operation(
        &mut self,
        control: &mut ParseControl,
        operation: HtmlConstructionOperation,
    ) -> Result<(), SessionError> {
        match operation.abort() {
            Ok(aborted) => self.restore_aborted(control, aborted),
            Err(_) => Err(latch_registry_row(control)),
        }
    }
    /// As `abort_operation`, for an operation whose every action was accepted.
    fn abort_ready(
        &mut self,
        control: &mut ParseControl,
        ready: HtmlConstructionReadyOperation,
    ) -> Result<(), SessionError> {
        match ready.abort() {
            Ok(aborted) => self.restore_aborted(control, aborted),
            Err(_) => Err(latch_registry_row(control)),
        }
    }
    fn restore_aborted(
        &mut self,
        control: &mut ParseControl,
        aborted: HtmlConstructionAbort,
    ) -> Result<(), SessionError> {
        let (storage, _, retired) = aborted.into_parts();
        let mut impossible = false;
        while self.children.len() > storage.len() {
            match self.children.pop() {
                Some(Some(ticket)) if ticket.snapshot().payload_bytes() != 0 => {
                    // Uncommitted child banks cannot have reserved; the ticket
                    // stays registered rather than being dropped uncredited.
                    self.children.push(Some(ticket));
                    impossible = true;
                    break;
                }
                Some(_) => {}
                None => {
                    impossible = true;
                    break;
                }
            }
        }
        self.owner = Some(storage);
        if let Some(receipt) = retired {
            match self.wrapper.take() {
                Some(ticket) => control.retire_construction_allocation(ticket, receipt)?,
                None => impossible = true,
            }
        }
        if impossible {
            return Err(latch_registry_row(control));
        }
        Ok(())
    }
    pub(super) fn run(
        &mut self,
        control: &mut ParseControl,
        command: HtmlConstructionCommand,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<HtmlConstructionCommit, SessionError> {
        let pending = self.start(control, command)?;
        self.continue_pending(control, pending, requests)
    }
    pub(super) fn finish_pending(
        &mut self,
        control: &mut ParseControl,
        mut pending: PendingConstruction,
    ) -> Result<Option<HtmlBuilderNode>, SessionError> {
        let ready = self.ready_pending(control, &mut pending)?;
        let committed = self.commit_ready(control, ready)?;
        self.adopt_mutation(control, committed)
    }
    /// Commits a ready operation. A commit the operation refuses hands it back, and it is
    /// aborted, which returns its storage.
    fn commit_ready(
        &mut self,
        control: &mut ParseControl,
        ready: HtmlConstructionReadyOperation,
    ) -> Result<HtmlConstructionCommit, SessionError> {
        match ready.commit() {
            Ok(committed) => Ok(committed),
            Err(refusal) => {
                let (ready, reason) = refusal.into_parts();
                let error = construction_error(control, reason);
                self.abort_ready(control, ready)?;
                Err(error)
            }
        }
    }
    /// The ready operation once every observation was accepted. An operation
    /// that is not ready is aborted, which returns its storage.
    fn ready_pending(
        &mut self,
        control: &mut ParseControl,
        pending: &mut PendingConstruction,
    ) -> Result<HtmlConstructionReadyOperation, SessionError> {
        let Some(operation) = pending.operation.take() else {
            return Err(latch_registry_row(control));
        };
        match operation.try_ready() {
            Ok(ready) => Ok(ready),
            Err(refusal) => {
                let (operation, reason) = refusal.into_parts();
                let error = construction_error(control, reason);
                self.abort_operation(control, operation)?;
                Err(error)
            }
        }
    }
    pub(super) fn continue_pending(
        &mut self,
        control: &mut ParseControl,
        pending: PendingConstruction,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<HtmlConstructionCommit, SessionError> {
        let ready = self.prepare_pending(control, pending, requests)?;
        self.commit_ready(control, ready)
    }
    fn prepare_pending(
        &mut self,
        control: &mut ParseControl,
        mut pending: PendingConstruction,
        requests: &[(HtmlParseResource, u64)],
    ) -> Result<HtmlConstructionReadyOperation, SessionError> {
        let result = (|| {
            loop {
                let before_edges = pending.before_edges();
                let Some(action) = pending.action(control)? else {
                    break;
                };
                if self.perform_action(control, action, requests, before_edges)? {
                    break;
                }
            }
            Ok::<_, SessionError>(())
        })();
        if let Err(error) = result {
            self.abort_pending(control, pending)?;
            return Err(error);
        }
        self.ready_pending(control, &mut pending)
    }
}

impl ConstructionOwner {
    pub(super) fn publish(
        &mut self,
        control: &mut ParseControl,
        root: HtmlBuilderNode,
    ) -> Result<(HtmlConstructionResult, HtmlConstructionLease), SessionError> {
        match self.run(control, HtmlConstructionCommand::Publish { root }, &[])? {
            HtmlConstructionCommit::Published { result, lease } => Ok((result, lease)),
            HtmlConstructionCommit::Changed { storage, .. } => {
                // Publication never leaves the storage changed in place; what
                // came back is the owner's storage and goes back to it.
                self.owner = Some(storage);
                Err(latch_registry_row(control))
            }
        }
    }
    pub(super) fn require_recovery(
        &self,
        control: &mut ParseControl,
        lease: &HtmlConstructionLease,
    ) -> Result<(), SessionError> {
        let Some(wrapper) = self.wrapper.as_ref() else {
            return Err(latch_registry_row(control));
        };
        control.require_construction_retirement(wrapper, lease.allocation())?;
        Ok(())
    }
    pub(super) fn recovered(
        &mut self,
        control: &mut ParseControl,
        recovery: HtmlConstructionRecovery,
    ) -> Result<(), SessionError> {
        let (storage, receipt) = recovery.into_parts();
        self.owner = Some(storage);
        // The storage is back before anything can refuse, so the owner can
        // still be retired whatever this reports.
        let Some(ticket) = self.wrapper.take() else {
            return Err(latch_registry_row(control));
        };
        control.retire_construction_allocation(ticket, receipt)?;
        #[cfg(test)]
        if control.take_construction_recovery_fault() {
            return Err(latch_registry_row(control));
        }
        Ok(())
    }
    /// Retires every bank this owner holds. A refusal is remembered and the
    /// walk continues, so each remaining bank is still credited and its ticket
    /// released; the first refusal is returned at the end.
    pub(super) fn retire(
        mut self,
        control: &mut ParseControl,
    ) -> Result<HtmlConstructionUsage, SessionError> {
        let mut refused = None;
        keep_first(&mut refused, self.abort_attributes(control));
        keep_first(&mut refused, self.retire_name_index(control));
        let Some(storage) = self.owner.take() else {
            // The storage is lent to something that was not ended, so no bank
            // can be retired here; the tickets below still go back to the
            // controller with the children registry.
            keep_first(&mut refused, Err(latch_registry_row(control)));
            keep_first(
                &mut refused,
                control
                    .release_vector(self.children, self.children_allocation)
                    .map_err(|error| SessionError::Stop(error.into_parts().0)),
            );
            return Err(refused.unwrap_or_else(|| latch_registry_row(control)));
        };
        let usage = storage.usage();
        let mut retirement = storage.retire();
        loop {
            let action = match retirement.next_action() {
                Ok(Some(action)) => action,
                Ok(None) => break,
                Err(error) => {
                    keep_first(&mut refused, Err(construction_error(control, error)));
                    break;
                }
            };
            let credit = match action.description() {
                HtmlConstructionRetirementDescription::Payload { .. } => Credit::None,
                HtmlConstructionRetirementDescription::Allocation(snapshot) => {
                    let bank = if let Some(position) = snapshot.child_position() {
                        Some(Bank::Children(position))
                    } else {
                        match snapshot.role() {
                            HtmlConstructionAllocationRole::Records => Some(Bank::Records),
                            HtmlConstructionAllocationRole::Identity => Some(Bank::Identity),
                            // A recovered owner holds only record, children
                            // and identity banks.
                            _ => None,
                        }
                    };
                    match bank {
                        None => {
                            keep_first(&mut refused, Err(latch_registry_row(control)));
                            Credit::Refused
                        }
                        Some(bank) => match self.allocation_slot(control, bank) {
                            Err(error) => {
                                keep_first(&mut refused, Err(error));
                                Credit::Refused
                            }
                            Ok(slot) => match slot.as_ref() {
                                Some(ticket) => {
                                    match control.require_construction_retirement(ticket, snapshot)
                                    {
                                        Ok(()) => Credit::Bank(bank),
                                        Err(stop) => {
                                            keep_first(&mut refused, Err(stop));
                                            Credit::Refused
                                        }
                                    }
                                }
                                // An unallocated record bank holds nothing.
                                None if snapshot.payload_bytes() == 0 => Credit::Empty,
                                None => {
                                    keep_first(&mut refused, Err(latch_registry_row(control)));
                                    Credit::Refused
                                }
                            },
                        },
                    }
                }
            };
            // The walk advances only by performing its actions.
            let Some(receipt) = action.perform() else {
                continue;
            };
            match credit {
                Credit::Bank(bank) => match self.allocation_slot(control, bank) {
                    Ok(slot) => match slot.take() {
                        Some(ticket) => {
                            let credited = control.retire_construction_allocation(ticket, receipt);
                            #[cfg(test)]
                            inject_walk_refusal(control, WalkSite::ConstructionBank, &mut refused);
                            keep_first(&mut refused, credited);
                        }
                        None => keep_first(&mut refused, Err(latch_registry_row(control))),
                    },
                    Err(error) => keep_first(&mut refused, Err(error)),
                },
                // An empty record bank has no physical refund.
                Credit::Empty if receipt.allocation().payload_bytes() != 0 => {
                    keep_first(&mut refused, Err(latch_registry_row(control)));
                }
                Credit::Empty | Credit::None | Credit::Refused => {}
            }
        }
        keep_first(
            &mut refused,
            control
                .release_vector(self.children, self.children_allocation)
                .map_err(|error| SessionError::Stop(error.into_parts().0)),
        );
        #[cfg(test)]
        if control.take_construction_retire_fault() {
            keep_first(&mut refused, Err(latch_registry_row(control)));
        }
        refused.map_or(Ok(usage), Err)
    }
}

/// What a retired action's receipt is credited against.
enum Credit {
    /// A payload action: nothing to credit.
    None,
    /// An unallocated bank: its receipt must hold nothing.
    Empty,
    /// The ticket of this bank.
    Bank(Bank),
    /// The action was refused and already recorded; it is only performed.
    Refused,
}

/// Borrows the same storage before or after publication; it owns no records or identity.
#[derive(Clone, Copy)]
pub(super) enum ConstructionView<'a> {
    Active(&'a HtmlConstructionStorage),
    Frozen(&'a HtmlConstructionResult),
}
impl<'a> ConstructionView<'a> {
    pub(super) fn node_at(self, position: usize) -> Option<HtmlConstructionNode<'a>> {
        match self {
            Self::Active(value) => value.node_at(position),
            Self::Frozen(value) => value.node_at(position),
        }
    }
    pub(super) fn len(self) -> usize {
        match self {
            Self::Active(value) => value.len(),
            Self::Frozen(value) => value.len(),
        }
    }
}

#[cfg(test)]
impl ConstructionOwner {
    /// The storage at rest, for tests that hold the owner in a known state.
    pub(super) fn test_storage(&self) -> &HtmlConstructionStorage {
        self.resident().expect("storage at rest")
    }
    /// Makes the next attribute edit begin with its custody slot emptied; the
    /// hidden edit is returned by [`Self::restore_hidden_attribute_edit`].
    pub(super) fn hide_next_attribute_edit(&mut self) {
        self.hide_next_attribute_edit = true;
    }
    pub(super) fn restore_hidden_attribute_edit(&mut self) {
        self.attribute_edit = self.hidden_attribute_edit.take();
    }
}
