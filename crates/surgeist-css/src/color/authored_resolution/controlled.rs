//! Admitted typed feeding, path backing and the canonical reached resolver.

use super::{
    lowering::{Action as FeedAction, Lowering},
    path::{PathState, PreparedSite},
    position::Position,
    site::SiteRecord,
    *,
};
use crate::color::{
    bounded, component::SyntaxFailure, grammar::Site, operation as core, protocol::Data, *,
};
use crate::{CssColor, CssValueOrigin};
use std::mem::size_of;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssAuthoredColorObservation {
    ColorHeader,
    ColorPayload,
    Child(CssColorPathStep),
    ProfileHeader,
    HexByte { offset: usize },
    NameByte { offset: usize },
    ScalarHeader,
    ScalarPayload,
    LiteralHeader,
    NumericHeader,
    NumericChild { ordinal: usize },
    NumericSyntax { index: usize },
    NumericLiteral,
    NumericUnit { bytes: usize },
    NumericNameHeader,
    NumericByte { offset: usize },
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssAuthoredColorActionDescription<'css> {
    ReadAuthored {
        path: CssColorPathHandle<'css>,
        observation: CssAuthoredColorObservation,
    },
    ReadSite {
        index: usize,
    },
    ReadSlot {
        bank: CssColorBank,
        index: usize,
    },
    AppendSlot {
        bank: CssColorBank,
        index: usize,
    },
    UpdateSlot {
        bank: CssColorBank,
        index: usize,
    },
    ReadFact(CssColorFactRead),
    Compute(CssColorKernel),
    ReservePath {
        required_len: usize,
    },
    AppendPath(CssColorPathStep),
    TruncatePath(usize),
    PrepareSite,
    RestoreSite,
    Ready,
    Stopped,
}
/// Actual payload extents, in addition to the admitted continuation and guard.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssAuthoredColorActionLayout {
    pub authored_read_bytes: usize,
    pub slot_read_bytes: usize,
    pub slot_write_bytes: usize,
    pub fact_records: [usize; 3],
    pub path_read_bytes: usize,
    pub path_write_bytes: usize,
    pub result_bytes: usize,
    pub kernel: CssColorKernel,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssAuthoredColorInitializationLayout {
    pub owner_bytes: usize,
    pub register_bytes: usize,
    pub root_reference_bytes: usize,
    pub fact_identity_bytes: usize,
    pub path_header_bytes: usize,
}
/// Actual retained backing and fixed cleanup work; inspection allocates nothing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CssAuthoredColorCleanupLayout {
    pub register_bytes: usize,
    pub path_header_bytes: usize,
    pub path_len: usize,
    pub path_capacity_bytes: usize,
    pub owned_origin_bytes: usize,
}
/// Standard invalidity owns its actual path allocation exactly once.
#[derive(Debug, Eq, PartialEq)]
pub struct CssAuthoredColorStandardFailure {
    pub kind: CssColorStandardFailureKind,
    pub path: CssColorComponentPath,
    pub origin: Option<CssValueOrigin>,
}
#[derive(Debug, PartialEq)]
pub enum CssAuthoredColorOutcome {
    Used(surgeist_color::Color),
    StandardFailure(CssAuthoredColorStandardFailure),
}
/// A completed controlled result retains the actual CSS anchor and phase.
pub struct CssAuthoredColorCheckpoint<'css, 'authored, 'facts> {
    owner: &'css CssColorOperationOwner,
    phase: usize,
    root: &'authored CssColor,
    facts: &'facts CssColorFacts<'facts>,
    outcome: CssAuthoredColorOutcome,
}
impl<'authored, 'facts> CssAuthoredColorCheckpoint<'_, 'authored, 'facts> {
    pub const fn authored(&self) -> &'authored CssColor {
        self.root
    }
    pub const fn facts(&self) -> &'facts CssColorFacts<'facts> {
        self.facts
    }
    pub const fn outcome(&self) -> &CssAuthoredColorOutcome {
        &self.outcome
    }
    pub fn is_current(&self) -> bool {
        self.owner.current_phase() == self.phase
    }
    pub(super) fn into_parts(
        self,
    ) -> (
        &'authored CssColor,
        &'facts CssColorFacts<'facts>,
        CssAuthoredColorOutcome,
    ) {
        (self.root, self.facts, self.outcome)
    }
}
/// Cleanup observations retain neither the exclusive path borrow nor CSS anchor.
pub struct CssAuthoredColorRecovery<'authored, 'facts> {
    root: &'authored CssColor,
    facts: &'facts CssColorFacts<'facts>,
    observation: Option<CssColorPathCapacityObservation>,
}
impl<'authored, 'facts> CssAuthoredColorRecovery<'authored, 'facts> {
    pub const fn authored(&self) -> &'authored CssColor {
        self.root
    }
    pub const fn facts(&self) -> &'facts CssColorFacts<'facts> {
        self.facts
    }
    pub const fn capacity_observation(&self) -> Option<CssColorPathCapacityObservation> {
        self.observation
    }
}
#[expect(
    clippy::large_enum_variant,
    reason = "Typed feeding and reached resolution are mutually exclusive admitted inline continuations; no hidden heap frame"
)]
enum Work<'css, 'facts> {
    Feed(Lowering<'css>),
    Resolve(core::ColorOperation<'css, 'facts>),
    Ready,
}
#[derive(Clone, Copy)]
enum Gate {
    Work,
    Site(usize),
    Reserve,
    Append(CssColorPathStep),
    Truncate(usize),
    Prepare,
    Restore,
}
/// One controller lends the actual three CSS banks and a separate path buffer.
/// AST borrows in bank records are shortened to the CSS anchor's lifetime; the
/// original root/fact references remain independently available after cleanup.
pub struct CssAuthoredColorOperation<'path, 'css, 'authored: 'css, 'facts> {
    root: &'authored CssColor,
    facts: &'facts CssColorFacts<'facts>,
    expected: CssColorFactIdentity,
    path: PathState<'path, 'css>,
    work: Work<'css, 'facts>,
    gate: Gate,
    banks: [usize; 3],
    position: Option<Position>,
    target: Option<Position>,
    origin: Option<&'css CssValueOrigin>,
    origin_site: Option<usize>,
    reserved: bool,
    restore: bool,
    stopped: bool,
    abandoned: bool,
    outcome: Option<CssAuthoredColorOutcome>,
}
impl<'path, 'css, 'authored: 'css, 'facts>
    CssAuthoredColorOperation<'path, 'css, 'authored, 'facts>
{
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn resolve_authored(
        owner: &'css mut CssColorOperationOwner,
        root: &'authored CssColor,
        facts: &'facts CssColorFacts<'facts>,
        expected: CssColorFactIdentity,
        path: &'path mut CssColorPathBuffer,
    ) -> Result<Self, CssColorOperationError> {
        if facts.identity() != expected {
            return Err(CssColorOperationError::Facts(CssColorFactError::Foreign));
        }
        let path = PathState::initialize(owner, path)?;
        let work = Work::Feed(Lowering::start(path.owner(), path.phase(), root));
        let mut result = Self {
            root,
            facts,
            expected,
            path,
            work,
            gate: Gate::Work,
            position: None,
            target: None,
            banks: [0; 3],
            origin: None,
            origin_site: None,
            reserved: false,
            restore: false,
            stopped: false,
            abandoned: false,
            outcome: None,
        };
        result.plan()?;
        Ok(result)
    }
    pub const fn register_bytes() -> usize {
        size_of::<Self>()
    }
    pub const fn action_bytes() -> usize {
        size_of::<CssAuthoredColorAction<'_, 'path, 'css, 'authored, 'facts>>()
    }
    pub const fn initialization_layout() -> CssAuthoredColorInitializationLayout {
        CssAuthoredColorInitializationLayout {
            owner_bytes: size_of::<CssColorOperationOwner>(),
            register_bytes: Self::register_bytes(),
            root_reference_bytes: size_of::<&CssColor>(),
            fact_identity_bytes: size_of::<CssColorFactIdentity>(),
            path_header_bytes: size_of::<CssColorPathBuffer>(),
        }
    }
    /// Admit this fixed observation before retirement. A numerical failure has
    /// already transferred its path to the recipient and reports zero here.
    pub fn cleanup_layout(&self) -> CssAuthoredColorCleanupLayout {
        let (path_len, path_capacity_bytes, owned_origin_bytes) = match &self.outcome {
            Some(CssAuthoredColorOutcome::StandardFailure(failure)) => (
                failure.path.steps().len(),
                failure.path.capacity_bytes(),
                if failure.origin.is_some() {
                    size_of::<CssValueOrigin>()
                } else {
                    0
                },
            ),
            _ => self.path.retained_layout(),
        };
        CssAuthoredColorCleanupLayout {
            register_bytes: Self::register_bytes(),
            path_header_bytes: size_of::<CssColorPathBuffer>(),
            path_len,
            path_capacity_bytes,
            owned_origin_bytes,
        }
    }
    pub fn bank_length(&self, bank: CssColorBank) -> usize {
        match &self.work {
            Work::Feed(cursor) => match bank {
                CssColorBank::Algorithm => cursor.algorithm_end(),
                CssColorBank::Arithmetic => cursor.arithmetic_end(),
                CssColorBank::Evaluation => 0,
            },
            Work::Resolve(cursor) => cursor.bank_length(bank),
            Work::Ready => self.banks[ordinal(bank)],
        }
    }
    fn worker_action(&self) -> CssAuthoredColorActionDescription<'css> {
        use CssAuthoredColorActionDescription as A;
        match &self.work {
            Work::Feed(cursor) => match cursor.action() {
                FeedAction::Read { bank, index } => A::ReadSlot { bank, index },
                FeedAction::Append { bank, index } => A::AppendSlot { bank, index },
                FeedAction::Replace { bank, index } => A::UpdateSlot { bank, index },
                FeedAction::ReadColor => self.observation(CssAuthoredColorObservation::ColorHeader),
                FeedAction::ReadPayload { .. } => {
                    self.observation(CssAuthoredColorObservation::ColorPayload)
                }
                FeedAction::ReadChild { edge } => {
                    self.observation(CssAuthoredColorObservation::Child(edge))
                }
                FeedAction::ReadProfileHeader => {
                    self.observation(CssAuthoredColorObservation::ProfileHeader)
                }
                FeedAction::ReadHexByte { offset } => {
                    self.observation(CssAuthoredColorObservation::HexByte { offset })
                }
                FeedAction::ReadByte { offset } => {
                    self.observation(CssAuthoredColorObservation::NameByte { offset })
                }
                FeedAction::Scalar(action) => self.observation(scalar_observation(action)),
                FeedAction::Compute | FeedAction::Done | FeedAction::Failed(_) => {
                    A::Compute(cursor.kernel())
                }
            },
            Work::Resolve(cursor) => match bounded::description(cursor) {
                CssColorActionDescription::ReadSlot { bank, index } => A::ReadSlot { bank, index },
                CssColorActionDescription::AppendSlot { bank, index } => {
                    A::AppendSlot { bank, index }
                }
                CssColorActionDescription::UpdateSlot { bank, index } => {
                    A::UpdateSlot { bank, index }
                }
                CssColorActionDescription::ReadFact(fact) => A::ReadFact(fact),
                CssColorActionDescription::Compute(kernel) => A::Compute(kernel),
                CssColorActionDescription::UsedReady
                | CssColorActionDescription::StandardFailed => {
                    A::Compute(CssColorKernel::Transition)
                }
                _ => unreachable!(
                    "authored canonical resolution cannot lex or serialize source input"
                ),
            },
            Work::Ready => A::Ready,
        }
    }
    fn observation(
        &self,
        observation: CssAuthoredColorObservation,
    ) -> CssAuthoredColorActionDescription<'css> {
        CssAuthoredColorActionDescription::ReadAuthored {
            path: self.path.handle(),
            observation,
        }
    }
    pub fn action(&self) -> CssAuthoredColorActionDescription<'css> {
        use CssAuthoredColorActionDescription as A;
        if self.stopped || self.abandoned {
            return A::Stopped;
        }
        match self.gate {
            Gate::Work => self.worker_action(),
            Gate::Site(index) => A::ReadSite { index },
            Gate::Reserve => A::ReservePath {
                required_len: self.path.handle().length() + 1,
            },
            Gate::Append(step) => A::AppendPath(step),
            Gate::Truncate(len) => A::TruncatePath(len),
            Gate::Prepare => A::PrepareSite,
            Gate::Restore => A::RestoreSite,
        }
    }
    fn semantic(&self) -> bool {
        matches!(
            self.worker_action(),
            CssAuthoredColorActionDescription::Compute(_)
                | CssAuthoredColorActionDescription::ReadFact(_)
        )
    }
    fn plan(&mut self) -> Result<(), CssColorProtocolError> {
        if self.stopped || self.abandoned {
            return Ok(());
        }
        if self.restore {
            self.gate = Gate::Restore;
            return Ok(());
        }
        if matches!(self.work, Work::Ready) {
            self.gate = Gate::Work;
            return Ok(());
        }
        let pos = match &self.work {
            Work::Feed(cursor) => match cursor.action() {
                FeedAction::Failed(
                    SyntaxFailure::Component { site, .. } | SyntaxFailure::Numeric { site, .. },
                ) => {
                    if self.position.is_none_or(|p| p.site != site) {
                        self.gate = Gate::Site(site);
                        return Ok(());
                    }
                    self.position
                }
                _ => Some(cursor.position()),
            },
            Work::Resolve(cursor) => match cursor.reached_site() {
                Ok(Site::Authored { record }) => {
                    if self.position.is_some_and(|p| p.site == record) {
                        self.position
                    } else {
                        self.gate = Gate::Site(record);
                        return Ok(());
                    }
                }
                Ok(Site::Source(_)) => return Err(CssColorProtocolError::WrongSlot),
                Err(_) => None,
            },
            Work::Ready => None,
        };
        if let Some(pos) = pos {
            if self.position != Some(pos) {
                self.target = Some(pos);
                return self.align(pos);
            }
            if self.semantic() && self.path.prepared_site().is_none() {
                if self.origin_site != Some(pos.site) {
                    self.gate = Gate::Site(pos.site);
                } else {
                    self.gate = Gate::Prepare;
                }
                return Ok(());
            }
        }
        self.gate = Gate::Work;
        Ok(())
    }
    fn align(&mut self, pos: Position) -> Result<(), CssColorProtocolError> {
        let len = self.path.handle().length();
        if self.position.is_some_and(|old| old.site == pos.site) {
            if len > pos.depth {
                self.gate = Gate::Truncate(pos.depth);
                return Ok(());
            }
            if len != pos.depth {
                return Err(CssColorProtocolError::WrongSlot);
            }
        } else if let Some(edge) = pos.edge {
            let parent = pos
                .depth
                .checked_sub(1)
                .ok_or(CssColorProtocolError::WrongSlot)?;
            if len > parent {
                self.gate = Gate::Truncate(parent);
                return Ok(());
            }
            if len < parent {
                return Err(CssColorProtocolError::WrongSlot);
            }
            self.gate = if self.reserved {
                Gate::Append(edge)
            } else {
                Gate::Reserve
            };
            return Ok(());
        } else {
            if len > pos.depth {
                self.gate = Gate::Truncate(pos.depth);
                return Ok(());
            }
            if len != pos.depth {
                return Err(CssColorProtocolError::WrongSlot);
            }
        }
        self.position = Some(pos);
        self.target = None;
        self.plan()
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    fn after(&mut self) -> Result<(), CssColorOperationError> {
        if !matches!(self.work, Work::Ready) {
            self.banks = [
                self.bank_length(CssColorBank::Algorithm),
                self.bank_length(CssColorBank::Arithmetic),
                self.bank_length(CssColorBank::Evaluation),
            ];
        }
        if self.path.prepared_site().is_some() {
            self.restore = true;
        }
        self.plan().map_err(Into::into)
    }
    fn fail(&mut self, error: core::Error) -> CssColorOperationError {
        self.stopped = true;
        match error {
            core::Error::Numerical {
                error,
                site: Site::Authored { record },
            } => {
                assert_eq!(
                    self.path.prepared_site(),
                    Some(record),
                    "numerical action owns its actual prepared reached site"
                );
                let PreparedSite { path, origin, .. } = self.path.take_site();
                CssColorOperationError::Numerical {
                    error,
                    site: CssColorResolutionSite::Authored { path, origin },
                }
            }
            error => error.into(),
        }
    }
    pub fn next_action(&mut self) -> CssAuthoredColorAction<'_, 'path, 'css, 'authored, 'facts> {
        let description = self.action();
        CssAuthoredColorAction {
            operation: self,
            description,
            issued: false,
            completed: false,
            attempted: false,
        }
    }
    pub fn abandon(&mut self) {
        self.abandoned = true;
    }
    pub fn recover(
        slot: &mut Option<Self>,
    ) -> Result<CssAuthoredColorRecovery<'authored, 'facts>, CssColorProtocolError> {
        let op = slot.as_ref().ok_or(CssColorProtocolError::EmptySlot)?;
        if !op.abandoned {
            return Err(CssColorProtocolError::Terminal);
        }
        let mut op = slot
            .take()
            .expect("preflight proved an abandoned operation");
        if let Some(CssAuthoredColorOutcome::StandardFailure(failure)) = op.outcome.take() {
            op.path.reclaim_completed(
                failure.path,
                failure.origin,
                op.position
                    .expect("a completed failure has its actual position")
                    .site,
            );
        }
        Ok(CssAuthoredColorRecovery {
            root: op.root,
            facts: op.facts,
            observation: op.path.recover(),
        })
    }
    pub fn take(
        slot: &mut Option<Self>,
    ) -> Result<CssAuthoredColorCheckpoint<'css, 'authored, 'facts>, CssColorProtocolError> {
        let op = slot.as_ref().ok_or(CssColorProtocolError::EmptySlot)?;
        if op.stopped || op.abandoned || !matches!(op.work, Work::Ready) {
            return Err(CssColorProtocolError::Terminal);
        }
        let op = slot.take().expect("preflight proved a completed operation");
        let owner = op.path.owner();
        let phase = op.path.phase();
        op.path.recover();
        Ok(CssAuthoredColorCheckpoint {
            owner,
            phase,
            root: op.root,
            facts: op.facts,
            outcome: op.outcome.expect("ready owns its outcome"),
        })
    }
}
/// A pending action has one completion. Dropping it abandons without useful work.
pub struct CssAuthoredColorAction<'step, 'path, 'css, 'authored: 'css, 'facts> {
    operation: &'step mut CssAuthoredColorOperation<'path, 'css, 'authored, 'facts>,
    description: CssAuthoredColorActionDescription<'css>,
    issued: bool,
    completed: bool,
    attempted: bool,
}
impl<'path, 'css, 'authored: 'css, 'facts>
    CssAuthoredColorAction<'_, 'path, 'css, 'authored, 'facts>
{
    pub const fn description(&self) -> CssAuthoredColorActionDescription<'css> {
        self.description
    }
    /// Fixed snapshot only; the separate inspection layout admits reading steps.
    pub fn path_handle(&self) -> CssColorPathHandle<'css> {
        self.operation.path.handle()
    }
    /// A view cannot survive consumption of its guard.
    ///
    /// ```compile_fail
    /// use surgeist_css::CssAuthoredColorOperation;
    /// fn retain_view(op: &mut CssAuthoredColorOperation<'_, '_, '_, '_>) {
    ///     let action = op.next_action();
    ///     let handle = action.path_handle();
    ///     let view = action.path_view(handle).unwrap();
    ///     action.read_authored().unwrap();
    ///     assert!(view.is_empty());
    /// }
    /// ```
    pub fn path_inspection_layout(
        &self,
        handle: CssColorPathHandle<'css>,
    ) -> Result<CssColorPathInspectionLayout, CssColorProtocolError> {
        self.operation.path.inspection_layout(handle)
    }
    pub fn path_view(
        &self,
        handle: CssColorPathHandle<'css>,
    ) -> Result<&[CssColorPathStep], CssColorProtocolError> {
        self.operation.path.view(handle)
    }
    pub fn reservation_layout(
        &self,
    ) -> Result<CssColorPathReservationLayout, CssColorProtocolError> {
        if !matches!(
            self.description,
            CssAuthoredColorActionDescription::ReservePath { .. }
        ) {
            return Err(CssColorProtocolError::WrongAction);
        }
        self.operation.path.reservation_layout()
    }
    pub fn layout(&self) -> Result<CssAuthoredColorActionLayout, CssColorProtocolError> {
        use CssAuthoredColorActionDescription as A;
        let mut result = CssAuthoredColorActionLayout {
            authored_read_bytes: 0,
            slot_read_bytes: 0,
            slot_write_bytes: 0,
            fact_records: [0; 3],
            path_read_bytes: 0,
            path_write_bytes: 0,
            result_bytes: size_of::<Result<(), CssColorOperationError>>(),
            kernel: CssColorKernel::Transition,
        };
        match self.description {
            A::ReadAuthored { .. } => {
                if let Work::Feed(cursor) = &self.operation.work {
                    result.authored_read_bytes = cursor.input_bytes()?;
                }
            }
            A::ReadSite { .. } | A::ReadSlot { .. } => {
                result.slot_read_bytes = CssColorSlot::record_bytes()
            }
            A::AppendSlot { .. } | A::UpdateSlot { .. } => {
                result.slot_read_bytes = CssColorSlot::record_bytes();
                result.slot_write_bytes = CssColorSlot::record_bytes();
            }
            A::ReadFact(fact) => result.fact_records = fact.record_layouts(),
            A::Compute(kernel) => result.kernel = kernel,
            A::ReservePath { .. } => {
                let layout = self.reservation_layout()?;
                result.path_read_bytes = layout.header_bytes;
                result.path_write_bytes = layout.header_bytes + layout.observation_bytes;
                result.result_bytes = size_of::<CssColorPathCapacityObservation>();
            }
            A::AppendPath(_) => {
                result.path_read_bytes = size_of::<CssColorPathBuffer>();
                result.path_write_bytes =
                    size_of::<CssColorPathBuffer>() + size_of::<CssColorPathStep>();
            }
            A::TruncatePath(_) | A::RestoreSite => {
                result.path_read_bytes =
                    size_of::<CssColorPathBuffer>() + size_of::<PreparedSite>();
                result.path_write_bytes =
                    size_of::<CssColorPathBuffer>() + size_of::<Option<PreparedSite>>();
            }
            A::PrepareSite => {
                result.path_read_bytes = size_of::<CssColorPathBuffer>();
                result.authored_read_bytes = self
                    .operation
                    .origin
                    .map_or(0, |_| size_of::<CssValueOrigin>());
                result.path_write_bytes =
                    size_of::<PreparedSite>() + size_of::<CssColorPathBuffer>();
            }
            _ => {}
        }
        Ok(result)
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    fn complete(
        &mut self,
        result: Result<(), CssColorOperationError>,
    ) -> Result<(), CssColorOperationError> {
        self.completed = true;
        if result.is_err() {
            self.operation.stopped = true;
            return result;
        }
        let result = self.operation.after();
        if result.is_err() {
            self.operation.stopped = true;
        }
        result
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn read_authored(mut self) -> Result<(), CssColorOperationError> {
        if !matches!(
            self.description,
            CssAuthoredColorActionDescription::ReadAuthored { .. }
        ) {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        let result = match &mut self.operation.work {
            Work::Feed(cursor) => cursor.read_authored().map_err(Into::into),
            _ => Err(CssColorProtocolError::WrongAction.into()),
        };
        self.complete(result)
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn read_slot(
        mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'css>,
    ) -> Result<(), CssColorOperationError> {
        if self.description == (CssAuthoredColorActionDescription::ReadSite { index })
            && bank == CssColorBank::Algorithm
        {
            let result = (|| {
                slot.qualify(
                    self.operation.path.owner(),
                    self.operation.path.phase(),
                    index,
                    bank,
                )?;
                let Data::AuthoredSite(site) = slot.data else {
                    return Err(CssColorProtocolError::WrongSlot);
                };
                let SiteRecord {
                    edge,
                    depth,
                    origin,
                    ..
                } = site;
                let pos = Position {
                    site: index,
                    edge,
                    depth,
                };
                self.operation.origin = origin;
                self.operation.origin_site = Some(index);
                self.operation.target = Some(pos);
                self.operation.align(pos)
            })();
            self.completed = true;
            if result.is_err() {
                self.operation.stopped = true;
            }
            return result.map_err(Into::into);
        }
        if self.description != (CssAuthoredColorActionDescription::ReadSlot { bank, index }) {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        let result = match &mut self.operation.work {
            Work::Feed(cursor) => cursor.read(bank, index, slot).map_err(Into::into),
            Work::Resolve(cursor) => cursor.read_slot(bank, index, slot),
            Work::Ready => Err(CssColorProtocolError::WrongAction.into()),
        };
        let result = result.map_err(|e| self.operation.fail(e));
        self.complete(result)
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn record(&mut self) -> Result<CssColorSlot<'css>, CssColorOperationError> {
        if self.issued {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        let (bank, index) = match self.description {
            CssAuthoredColorActionDescription::AppendSlot { bank, index }
            | CssAuthoredColorActionDescription::UpdateSlot { bank, index } => (bank, index),
            _ => return Err(CssColorProtocolError::WrongAction.into()),
        };
        let result = match &mut self.operation.work {
            Work::Feed(cursor) => cursor.record(bank, index).map_err(Into::into),
            Work::Resolve(cursor) => cursor.pending_record(bank, index),
            Work::Ready => Err(CssColorProtocolError::WrongAction.into()),
        };
        let result = result.map_err(|e| self.operation.fail(e));
        self.issued = result.is_ok();
        result
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn written(
        mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'css>,
    ) -> Result<(), CssColorOperationError> {
        if !self.issued
            || !matches!(self.description,CssAuthoredColorActionDescription::AppendSlot {bank:b,index:i}|CssAuthoredColorActionDescription::UpdateSlot {bank:b,index:i} if bank==b&&index==i)
        {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        let result = match &mut self.operation.work {
            Work::Feed(cursor) => cursor.written(bank, index, slot).map_err(Into::into),
            Work::Resolve(cursor) => cursor.written(bank, index, slot),
            Work::Ready => Err(CssColorProtocolError::WrongAction.into()),
        };
        let result = result.map_err(|e| self.operation.fail(e));
        self.complete(result)
    }
    pub fn reserve_path(
        &mut self,
    ) -> Result<CssColorPathCapacityObservation, CssColorProtocolError> {
        if self.attempted
            || !matches!(
                self.description,
                CssAuthoredColorActionDescription::ReservePath { .. }
            )
        {
            return Err(CssColorProtocolError::WrongAction);
        }
        let result = self.operation.path.reserve();
        self.attempted = result.is_ok();
        result
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn capacity_observed(mut self) -> Result<(), CssColorOperationError> {
        if !self.attempted {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        let status = self.operation.path.capacity_observed()?;
        self.completed = true;
        if status == CssColorPathReservationStatus::AllocationFailed {
            self.operation.stopped = true;
            return Ok(());
        }
        self.operation.reserved = true;
        self.operation.align(
            self.operation
                .target
                .ok_or(CssColorProtocolError::WrongAction)?,
        )?;
        Ok(())
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn append_path(mut self) -> Result<(), CssColorOperationError> {
        let CssAuthoredColorActionDescription::AppendPath(step) = self.description else {
            return Err(CssColorProtocolError::WrongAction.into());
        };
        self.operation.path.append(step)?;
        self.operation.reserved = false;
        self.operation.position = self.operation.target.take();
        self.completed = true;
        self.operation.plan()?;
        Ok(())
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn truncate_path(mut self) -> Result<(), CssColorOperationError> {
        let CssAuthoredColorActionDescription::TruncatePath(length) = self.description else {
            return Err(CssColorProtocolError::WrongAction.into());
        };
        self.operation.path.truncate(length)?;
        self.completed = true;
        self.operation.align(
            self.operation
                .target
                .ok_or(CssColorProtocolError::WrongAction)?,
        )?;
        Ok(())
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn prepare_site(mut self) -> Result<(), CssColorOperationError> {
        if self.description != CssAuthoredColorActionDescription::PrepareSite {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        let site = self
            .operation
            .position
            .ok_or(CssColorProtocolError::WrongAction)?
            .site;
        self.operation.path.prepare(site, self.operation.origin)?;
        self.operation.gate = Gate::Work;
        self.completed = true;
        Ok(())
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn restore_site(mut self) -> Result<(), CssColorOperationError> {
        if self.description != CssAuthoredColorActionDescription::RestoreSite {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        self.operation.path.restore()?;
        self.operation.restore = false;
        self.completed = true;
        self.operation.plan()?;
        Ok(())
    }
    #[expect(
        clippy::result_large_err,
        reason = "The common CSS error propagates an admitted owned path and actual origin inline without allocation on failure"
    )]
    pub fn compute(mut self) -> Result<(), CssColorOperationError> {
        use CssAuthoredColorActionDescription as A;
        if !matches!(self.description, A::Compute(_) | A::ReadFact(_)) {
            return Err(CssColorProtocolError::WrongAction.into());
        }
        let result = match &mut self.operation.work {
            Work::Feed(cursor) => match cursor.action() {
                FeedAction::Done => {
                    let grammar = cursor.finish()?;
                    let banks = [cursor.algorithm_end(), cursor.arithmetic_end(), 0];
                    self.operation.work = Work::Resolve(core::ColorOperation::start_authored(
                        &grammar,
                        self.operation.facts,
                        self.operation.expected,
                        banks,
                    )?);
                    Ok(())
                }
                FeedAction::Failed(failure) => {
                    let (kind, site) = match failure {
                        SyntaxFailure::Component { site, .. } => {
                            (CssColorStandardFailureKind::ComponentGrammar, site)
                        }
                        SyntaxFailure::Numeric { site, .. } => {
                            (CssColorStandardFailureKind::NumericGrammar, site)
                        }
                    };
                    self.standard(kind, Site::Authored { record: site });
                    Ok(())
                }
                _ => cursor.compute().map_err(Into::into),
            },
            Work::Resolve(cursor) => cursor.compute(),
            Work::Ready => Err(CssColorProtocolError::WrongAction.into()),
        };
        let result = result.map_err(|error| self.operation.fail(error));
        if result.is_ok()
            && let Work::Resolve(cursor) = &self.operation.work
            && matches!(
                cursor.action(),
                core::Action::UsedReady | core::Action::StandardFailed
            )
        {
            let Work::Resolve(cursor) = std::mem::replace(&mut self.operation.work, Work::Ready)
            else {
                unreachable!()
            };
            match cursor.take_authored()? {
                Ok(color) => self.operation.outcome = Some(CssAuthoredColorOutcome::Used(color)),
                Err(failure) => self.standard(reason(failure.reason), failure.site),
            }
        }
        self.complete(result)
    }
    fn standard(&mut self, kind: CssColorStandardFailureKind, site: Site) {
        let Site::Authored { record } = site else {
            unreachable!("authored standard failure has typed authored provenance")
        };
        assert_eq!(self.operation.path.prepared_site(), Some(record));
        let PreparedSite { path, origin, .. } = self.operation.path.take_site();
        self.operation.outcome = Some(CssAuthoredColorOutcome::StandardFailure(
            CssAuthoredColorStandardFailure { kind, path, origin },
        ));
        self.operation.work = Work::Ready;
    }
}
impl Drop for CssAuthoredColorAction<'_, '_, '_, '_, '_> {
    fn drop(&mut self) {
        if !self.completed
            && !matches!(
                self.description,
                CssAuthoredColorActionDescription::Ready
                    | CssAuthoredColorActionDescription::Stopped
            )
        {
            self.operation.abandon();
        }
    }
}
fn reason(reason: core::StandardReason) -> CssColorStandardFailureKind {
    match reason {
        core::StandardReason::ColorGrammar => CssColorStandardFailureKind::ColorGrammar,
        core::StandardReason::ComponentGrammar => CssColorStandardFailureKind::ComponentGrammar,
        core::StandardReason::NumericGrammar => CssColorStandardFailureKind::NumericGrammar,
        core::StandardReason::InvalidProfile => CssColorStandardFailureKind::InvalidProfile,
        core::StandardReason::InvalidComponents => CssColorStandardFailureKind::InvalidComponents,
    }
}

fn ordinal(bank: CssColorBank) -> usize {
    match bank {
        CssColorBank::Algorithm => 0,
        CssColorBank::Arithmetic => 1,
        CssColorBank::Evaluation => 2,
    }
}

fn scalar_observation(action: super::scalar::Action) -> CssAuthoredColorObservation {
    use super::{numeric::Action as N, scalar::Action as S};
    use CssAuthoredColorObservation as O;
    match action {
        S::ReadHeader => O::ScalarHeader,
        S::ReadPayload { .. } => O::ScalarPayload,
        S::ReadLiteralHeader => O::LiteralHeader,
        S::ReadNameHeader => O::ProfileHeader,
        S::ReadByte { offset } => O::NameByte { offset },
        S::Numeric(action) => match action {
            N::ReadHeader => O::NumericHeader,
            N::ReadChild { ordinal } => O::NumericChild { ordinal },
            N::ReadSyntax { index } => O::NumericSyntax { index },
            N::ReadLiteral => O::NumericLiteral,
            N::ReadNameHeader => O::NumericNameHeader,
            N::ReadUnit { bytes } => O::NumericUnit { bytes },
            N::ReadByte { offset } => O::NumericByte { offset },
            _ => unreachable!(
                "non-observation numeric actions use their canonical bank/kernel action"
            ),
        },
        _ => unreachable!("non-observation scalar actions use their canonical bank/kernel action"),
    }
}
