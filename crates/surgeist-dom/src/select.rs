//! Native select-family values and explicit intrinsic task checkpoint.
use crate::{DomNode, NodeId};

/// Native Select scalars; form identity remains on the canonical node.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomSelectState {
    pub(crate) control: crate::control::NativeControlState,
    pub(crate) flags: surgeist_html::HtmlSelectFlags,
}
impl DomSelectState {
    pub(crate) fn fresh() -> Self {
        Self {
            control: crate::control::NativeControlState::fresh(),
            flags: Default::default(),
        }
    }
    pub fn flags(&self) -> surgeist_html::HtmlSelectFlags {
        self.flags
    }
    pub fn custom_validity_message(&self) -> &crate::DocumentString {
        &self.control.message
    }
    pub fn parser_inserted(&self) -> bool {
        self.control.parser_inserted
    }
}
/// An Option's selectedness/dirty flags. Cached identity is exposed by its node view.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DomOptionState {
    pub(crate) flags: surgeist_html::HtmlOptionFlags,
}
impl DomOptionState {
    pub fn flags(self) -> surgeist_html::HtmlOptionFlags {
        self.flags
    }
}

#[derive(Clone, Copy, Debug)]
pub struct DomSelect<'a> {
    pub(crate) node: DomNode<'a>,
}
impl<'a> DomSelect<'a> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn state(self) -> &'a DomSelectState {
        self.node
            .as_element()
            .expect("Select")
            .select_state()
            .expect("Select")
    }
    pub fn form_owner(self) -> Result<Option<NodeId>, crate::DomError> {
        self.node.snapshot.require_complete_source()?;
        Ok(self.node.node.control_owner())
    }
}
#[derive(Clone, Copy, Debug)]
pub struct DomOption<'a> {
    pub(crate) node: DomNode<'a>,
}
impl DomOption<'_> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn state(self) -> DomOptionState {
        self.node
            .as_element()
            .expect("Option")
            .option_state()
            .expect("Option")
    }
    pub fn cached_select(self) -> Result<Option<NodeId>, crate::DomError> {
        self.node.snapshot.require_complete_source()?;
        Ok(self.node.node.option_select())
    }
}
#[derive(Clone, Copy, Debug)]
pub struct DomSelectedcontent<'a> {
    pub(crate) node: DomNode<'a>,
}
impl DomSelectedcontent<'_> {
    pub fn id(self) -> NodeId {
        self.node.id()
    }
    pub fn disabled(self) -> bool {
        self.node
            .as_element()
            .expect("Selectedcontent")
            .selectedcontent_disabled()
            .expect("Selectedcontent")
    }
}

#[derive(Clone, Debug)]
pub(crate) enum Edit {
    Value(crate::DocumentString),
    Index(i64),
    Reset,
    OptionSelected(bool),
}

/// Exact operation identity, independent of node slots and batch tickets.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomNativeTaskId {
    pub(crate) owner: crate::DomOwnerId,
    pub(crate) serial: u64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSelectTaskCancellationReason {
    TargetDestroyed,
}
pub(crate) struct Authorization {
    pub(crate) id: DomNativeTaskId,
    pub(crate) target: NodeId,
    state: std::sync::atomic::AtomicU8,
    pub(crate) source: crate::DomSnapshot,
}
impl Authorization {
    pub(crate) fn destroy_if_pending(&self) {
        let _ = self.state.compare_exchange(
            0,
            2,
            std::sync::atomic::Ordering::AcqRel,
            std::sync::atomic::Ordering::Acquire,
        );
    }
    pub(crate) fn new(id: DomNativeTaskId, target: NodeId, source: crate::DomSnapshot) -> Self {
        Self {
            id,
            target,
            source,
            state: std::sync::atomic::AtomicU8::new(0),
        }
    }
    pub(crate) fn status(&self) -> u8 {
        self.state.load(std::sync::atomic::Ordering::Acquire)
    }
    pub(crate) fn consume(&self) {
        self.state.store(1, std::sync::atomic::Ordering::Release);
    }
}
/// Immutable captured intention and shared once-only execution authorization.
/// Dropping every clone expires permission, leaving canonical FIFO work pending.
#[derive(Clone)]
pub struct DomSelectTask {
    pub(crate) authorization: std::sync::Arc<Authorization>,
    pub(crate) source: crate::DomSnapshot,
}
impl std::fmt::Debug for DomSelectTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DomSelectTask")
            .field("operation", &self.operation())
            .field("select", &self.select())
            .finish()
    }
}
impl DomSelectTask {
    pub fn operation(&self) -> DomNativeTaskId {
        self.authorization.id
    }
    pub fn select(&self) -> NodeId {
        self.authorization.target
    }
    pub fn source(&self) -> &crate::DomSnapshot {
        &self.source
    }
}
/// One native checkpoint turn. No callbacks or browser event loop are installed.
#[derive(Debug)]
pub enum DomNativeCheckpoint {
    Idle,
    Select(DomSelectTask),
    Slots(crate::DomSlotNotifications),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DomSelectTaskOutcome {
    Updated,
    SkippedTargetDestroyed,
}
#[derive(Clone, Debug)]
pub struct DomSelectTaskCompletion {
    pub(crate) outcome: DomSelectTaskOutcome,
    pub(crate) changes: Option<crate::DomChanges>,
}
impl DomSelectTaskCompletion {
    pub fn outcome(&self) -> DomSelectTaskOutcome {
        self.outcome
    }
    pub fn changes(&self) -> Option<&crate::DomChanges> {
        self.changes.as_ref()
    }
}
/// Canonical admission of an issued Select intention. A retired target reports
/// its historical skip directly; a live target executes through its Source root.
pub enum DomSelectCompletionStart<'a> {
    Skipped(DomSelectTaskCompletion),
    Source(crate::DomSourceLease<'a>),
}
/// Actual per-turn processing and Slot output admission. Select task capacity
/// remains the owner's independently configured pending-operation bound.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomNativeCheckpointLimits {
    pub(crate) scratch: usize,
    pub(crate) work: u64,
    pub(crate) slots: crate::DomSlotQueryLimits,
}
impl DomNativeCheckpointLimits {
    pub fn new(
        scratch_bytes: usize,
        work: u64,
        slots: crate::DomSlotQueryLimits,
    ) -> Result<Self, crate::DomError> {
        if scratch_bytes > isize::MAX as usize {
            return Err(crate::DomError::InvalidLimit(
                crate::DomResource::SelectScratch,
            ));
        }
        Ok(Self {
            scratch: scratch_bytes,
            work,
            slots,
        })
    }
}
