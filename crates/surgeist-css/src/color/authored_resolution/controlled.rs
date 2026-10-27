//! Admitted typed feeding, path backing and the canonical reached resolver.
//!
//! This controlled authored protocol is crate-private (#1286, #1353 5.7): its
//! only driver is the ordinary [`resolve_authored_color`](super::ordinary::resolve_authored_color)
//! front. It classifies the shared resolver's pending work with the same
//! typed classification as the public source parse.

use super::lowering::Action as FeedAction;
use super::lowering::Lowering;
use super::path::{PathState, PreparedSite};
use super::position::Position;
use super::site::SiteRecord;
use crate::color::CssColor;
#[cfg(test)]
use crate::color::authored_resolution::path::CssColorPathInspectionLayout;
use crate::color::authored_resolution::path::{
    CssColorPathBuffer, CssColorPathCapacityObservation, CssColorPathHandle,
    CssColorPathReservationLayout, CssColorPathReservationStatus,
};
use crate::color::component::SyntaxFailure;
use crate::color::controlled::bounded::Pending;
use crate::color::controlled::protocol::{ColorProtocolError, Data};
use crate::color::controlled::{
    CssColorBank, CssColorFactRead, CssColorKernel, CssColorOperationOwner, CssColorSlot, bounded,
};
use crate::color::grammar::Site;
use crate::color::operation as core;
use crate::color::{CssColorComponentPath, CssColorPathStep};
use crate::color::{
    CssColorError, CssColorFactDefect, CssColorFactIdentity, CssColorFacts,
    CssColorStandardFailureKind,
};
use crate::component::CssValueOrigin;
use std::mem::size_of;

/// A failure of the private authored protocol: a reached semantic failure, or
/// a protocol inconsistency in the crate's own driver.
#[derive(Debug, Eq, PartialEq)]
pub(crate) enum AuthoredError {
    Color(CssColorError),
    Protocol(ColorProtocolError),
}
impl From<ColorProtocolError> for AuthoredError {
    fn from(error: ColorProtocolError) -> Self {
        Self::Protocol(error)
    }
}
impl From<CssColorError> for AuthoredError {
    fn from(error: CssColorError) -> Self {
        Self::Color(error)
    }
}
impl From<core::Error> for AuthoredError {
    fn from(error: core::Error) -> Self {
        match bounded::classify(error) {
            Ok(error) => Self::Color(error),
            Err(defect) => Self::Protocol(defect),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CssAuthoredColorObservation {
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
pub(crate) enum CssAuthoredColorActionDescription<'css> {
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
pub(crate) struct CssAuthoredColorActionLayout {
    pub authored_read_bytes: usize,
    pub slot_read_bytes: usize,
    pub slot_write_bytes: usize,
    pub fact_records: [usize; 3],
    pub path_read_bytes: usize,
    pub path_write_bytes: usize,
    pub result_bytes: usize,
    pub kernel: CssColorKernel,
}
/// Actual retained backing and fixed cleanup work; inspection allocates nothing.
#[cfg(test)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CssAuthoredColorCleanupLayout {
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
/// A completed controlled result; it borrows neither the anchor nor the path.
pub(crate) struct CssAuthoredColorCheckpoint<'authored, 'facts> {
    root: &'authored CssColor,
    facts: &'facts CssColorFacts<'facts>,
    outcome: CssAuthoredColorOutcome,
}
impl<'authored, 'facts> CssAuthoredColorCheckpoint<'authored, 'facts> {
    #[cfg(test)]
    pub(crate) const fn authored(&self) -> &'authored CssColor {
        self.root
    }
    #[cfg(test)]
    pub(crate) const fn outcome(&self) -> &CssAuthoredColorOutcome {
        &self.outcome
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
pub(crate) struct CssAuthoredColorOperation<'path, 'css, 'authored: 'css, 'facts> {
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
    pub(crate) fn resolve_authored(
        owner: &'css mut CssColorOperationOwner,
        root: &'authored CssColor,
        facts: &'facts CssColorFacts<'facts>,
        expected: CssColorFactIdentity,
        path: &'path mut CssColorPathBuffer,
    ) -> Result<Self, AuthoredError> {
        if facts.identity() != expected {
            return Err(
                CssColorError::from(crate::color::bounded::ColorErrorKind::Facts(
                    CssColorFactDefect::Foreign,
                ))
                .into(),
            );
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
    /// Admit this fixed observation before retirement.
    #[cfg(test)]
    pub(crate) fn cleanup_layout(&self) -> CssAuthoredColorCleanupLayout {
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
            register_bytes: size_of::<Self>(),
            path_header_bytes: size_of::<CssColorPathBuffer>(),
            path_len,
            path_capacity_bytes,
            owned_origin_bytes,
        }
    }
    pub(crate) fn bank_length(&self, bank: CssColorBank) -> usize {
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
            Work::Resolve(cursor) => match bounded::pending(cursor) {
                Pending::ReadSlot { bank, index } => A::ReadSlot { bank, index },
                Pending::AppendSlot { bank, index } => A::AppendSlot { bank, index },
                Pending::UpdateSlot { bank, index } => A::UpdateSlot { bank, index },
                Pending::ReadFact(fact) => A::ReadFact(fact),
                Pending::Compute(kernel) => A::Compute(kernel),
                Pending::Complete => A::Compute(CssColorKernel::Transition),
                Pending::ReadInput { .. } | Pending::Stopped => {
                    unreachable!("authored canonical resolution reads no source input")
                }
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
    pub(crate) fn action(&self) -> CssAuthoredColorActionDescription<'css> {
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
    fn plan(&mut self) -> Result<(), ColorProtocolError> {
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
                Ok(Site::Source(_)) => return Err(ColorProtocolError::WrongSlot),
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
    fn align(&mut self, pos: Position) -> Result<(), ColorProtocolError> {
        let len = self.path.handle().length();
        if self.position.is_some_and(|old| old.site == pos.site) {
            if len > pos.depth {
                self.gate = Gate::Truncate(pos.depth);
                return Ok(());
            }
            if len != pos.depth {
                return Err(ColorProtocolError::WrongSlot);
            }
        } else if let Some(edge) = pos.edge {
            let parent = pos
                .depth
                .checked_sub(1)
                .ok_or(ColorProtocolError::WrongSlot)?;
            if len > parent {
                self.gate = Gate::Truncate(parent);
                return Ok(());
            }
            if len < parent {
                return Err(ColorProtocolError::WrongSlot);
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
                return Err(ColorProtocolError::WrongSlot);
            }
        }
        self.position = Some(pos);
        self.target = None;
        self.plan()
    }
    fn after(&mut self) -> Result<(), AuthoredError> {
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
    fn fail(&mut self, error: core::Error) -> AuthoredError {
        self.stopped = true;
        error.into()
    }
    pub(crate) fn next_action(
        &mut self,
    ) -> CssAuthoredColorAction<'_, 'path, 'css, 'authored, 'facts> {
        let description = self.action();
        CssAuthoredColorAction {
            operation: self,
            description,
            issued: false,
            completed: false,
            attempted: false,
        }
    }
    pub(crate) fn abandon(&mut self) {
        self.abandoned = true;
    }
    /// Retires an abandoned operation and returns its buffer's outstanding
    /// reservation observation, if any; it borrows neither anchor nor path.
    pub(crate) fn recover(
        slot: &mut Option<Self>,
    ) -> Result<Option<CssColorPathCapacityObservation>, ColorProtocolError> {
        let op = slot.as_ref().ok_or(ColorProtocolError::EmptySlot)?;
        if !op.abandoned {
            return Err(ColorProtocolError::Terminal);
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
        Ok(op.path.recover())
    }
    pub(crate) fn take(
        slot: &mut Option<Self>,
    ) -> Result<CssAuthoredColorCheckpoint<'authored, 'facts>, ColorProtocolError> {
        let op = slot.as_ref().ok_or(ColorProtocolError::EmptySlot)?;
        if op.stopped || op.abandoned || !matches!(op.work, Work::Ready) {
            return Err(ColorProtocolError::Terminal);
        }
        let op = slot.take().expect("preflight proved a completed operation");
        op.path.recover();
        Ok(CssAuthoredColorCheckpoint {
            root: op.root,
            facts: op.facts,
            outcome: op.outcome.expect("ready owns its outcome"),
        })
    }
}
/// A pending action has one completion. Dropping it abandons without useful work.
pub(crate) struct CssAuthoredColorAction<'step, 'path, 'css, 'authored: 'css, 'facts> {
    operation: &'step mut CssAuthoredColorOperation<'path, 'css, 'authored, 'facts>,
    description: CssAuthoredColorActionDescription<'css>,
    issued: bool,
    completed: bool,
    attempted: bool,
}
impl<'path, 'css, 'authored: 'css, 'facts>
    CssAuthoredColorAction<'_, 'path, 'css, 'authored, 'facts>
{
    pub(crate) const fn description(&self) -> CssAuthoredColorActionDescription<'css> {
        self.description
    }
    /// Fixed snapshot only; the separate inspection layout admits reading steps.
    #[cfg(test)]
    pub(crate) fn path_handle(&self) -> CssColorPathHandle<'css> {
        self.operation.path.handle()
    }
    #[cfg(test)]
    pub(crate) fn path_inspection_layout(
        &self,
        handle: CssColorPathHandle<'css>,
    ) -> Result<CssColorPathInspectionLayout, ColorProtocolError> {
        self.operation.path.inspection_layout(handle)
    }
    #[cfg(test)]
    pub(crate) fn path_view(
        &self,
        handle: CssColorPathHandle<'css>,
    ) -> Result<&[CssColorPathStep], ColorProtocolError> {
        self.operation.path.view(handle)
    }
    pub(crate) fn reservation_layout(
        &self,
    ) -> Result<CssColorPathReservationLayout, ColorProtocolError> {
        if !matches!(
            self.description,
            CssAuthoredColorActionDescription::ReservePath { .. }
        ) {
            return Err(ColorProtocolError::WrongAction);
        }
        self.operation.path.reservation_layout()
    }
    pub(crate) fn layout(&self) -> Result<CssAuthoredColorActionLayout, ColorProtocolError> {
        use CssAuthoredColorActionDescription as A;
        let mut result = CssAuthoredColorActionLayout {
            authored_read_bytes: 0,
            slot_read_bytes: 0,
            slot_write_bytes: 0,
            fact_records: [0; 3],
            path_read_bytes: 0,
            path_write_bytes: 0,
            result_bytes: size_of::<Result<(), AuthoredError>>(),
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
    fn complete(&mut self, result: Result<(), AuthoredError>) -> Result<(), AuthoredError> {
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
    pub(crate) fn read_authored(mut self) -> Result<(), AuthoredError> {
        if !matches!(
            self.description,
            CssAuthoredColorActionDescription::ReadAuthored { .. }
        ) {
            return Err(ColorProtocolError::WrongAction.into());
        }
        let result = match &mut self.operation.work {
            Work::Feed(cursor) => cursor.read_authored().map_err(Into::into),
            _ => Err(ColorProtocolError::WrongAction.into()),
        };
        self.complete(result)
    }
    pub(crate) fn read_slot(
        mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'css>,
    ) -> Result<(), AuthoredError> {
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
                    return Err(ColorProtocolError::WrongSlot);
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
            return Err(ColorProtocolError::WrongAction.into());
        }
        let result = match &mut self.operation.work {
            Work::Feed(cursor) => cursor.read(bank, index, slot).map_err(Into::into),
            Work::Resolve(cursor) => cursor.read_slot(bank, index, slot),
            Work::Ready => Err(ColorProtocolError::WrongAction.into()),
        };
        let result = result.map_err(|e| self.operation.fail(e));
        self.complete(result)
    }
    pub(crate) fn record(&mut self) -> Result<CssColorSlot<'css>, AuthoredError> {
        if self.issued {
            return Err(ColorProtocolError::WrongAction.into());
        }
        let (bank, index) = match self.description {
            CssAuthoredColorActionDescription::AppendSlot { bank, index }
            | CssAuthoredColorActionDescription::UpdateSlot { bank, index } => (bank, index),
            _ => return Err(ColorProtocolError::WrongAction.into()),
        };
        let result = match &mut self.operation.work {
            Work::Feed(cursor) => cursor.record(bank, index).map_err(Into::into),
            Work::Resolve(cursor) => cursor.pending_record(bank, index),
            Work::Ready => Err(ColorProtocolError::WrongAction.into()),
        };
        let result = result.map_err(|e| self.operation.fail(e));
        self.issued = result.is_ok();
        result
    }
    pub(crate) fn written(
        mut self,
        bank: CssColorBank,
        index: usize,
        slot: &CssColorSlot<'css>,
    ) -> Result<(), AuthoredError> {
        if !self.issued
            || !matches!(self.description,CssAuthoredColorActionDescription::AppendSlot {bank:b,index:i}|CssAuthoredColorActionDescription::UpdateSlot {bank:b,index:i} if bank==b&&index==i)
        {
            return Err(ColorProtocolError::WrongAction.into());
        }
        let result = match &mut self.operation.work {
            Work::Feed(cursor) => cursor.written(bank, index, slot).map_err(Into::into),
            Work::Resolve(cursor) => cursor.written(bank, index, slot),
            Work::Ready => Err(ColorProtocolError::WrongAction.into()),
        };
        let result = result.map_err(|e| self.operation.fail(e));
        self.complete(result)
    }
    pub(crate) fn reserve_path(
        &mut self,
    ) -> Result<CssColorPathCapacityObservation, ColorProtocolError> {
        if self.attempted
            || !matches!(
                self.description,
                CssAuthoredColorActionDescription::ReservePath { .. }
            )
        {
            return Err(ColorProtocolError::WrongAction);
        }
        let result = self.operation.path.reserve();
        self.attempted = result.is_ok();
        result
    }
    pub(crate) fn capacity_observed(mut self) -> Result<(), AuthoredError> {
        if !self.attempted {
            return Err(ColorProtocolError::WrongAction.into());
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
                .ok_or(ColorProtocolError::WrongAction)?,
        )?;
        Ok(())
    }
    pub(crate) fn append_path(mut self) -> Result<(), AuthoredError> {
        let CssAuthoredColorActionDescription::AppendPath(step) = self.description else {
            return Err(ColorProtocolError::WrongAction.into());
        };
        self.operation.path.append(step)?;
        self.operation.reserved = false;
        self.operation.position = self.operation.target.take();
        self.completed = true;
        self.operation.plan()?;
        Ok(())
    }
    pub(crate) fn truncate_path(mut self) -> Result<(), AuthoredError> {
        let CssAuthoredColorActionDescription::TruncatePath(length) = self.description else {
            return Err(ColorProtocolError::WrongAction.into());
        };
        self.operation.path.truncate(length)?;
        self.completed = true;
        self.operation.align(
            self.operation
                .target
                .ok_or(ColorProtocolError::WrongAction)?,
        )?;
        Ok(())
    }
    pub(crate) fn prepare_site(mut self) -> Result<(), AuthoredError> {
        if self.description != CssAuthoredColorActionDescription::PrepareSite {
            return Err(ColorProtocolError::WrongAction.into());
        }
        let site = self
            .operation
            .position
            .ok_or(ColorProtocolError::WrongAction)?
            .site;
        self.operation.path.prepare(site, self.operation.origin)?;
        self.operation.gate = Gate::Work;
        self.completed = true;
        Ok(())
    }
    pub(crate) fn restore_site(mut self) -> Result<(), AuthoredError> {
        if self.description != CssAuthoredColorActionDescription::RestoreSite {
            return Err(ColorProtocolError::WrongAction.into());
        }
        self.operation.path.restore()?;
        self.operation.restore = false;
        self.completed = true;
        self.operation.plan()?;
        Ok(())
    }
    pub(crate) fn compute(mut self) -> Result<(), AuthoredError> {
        use CssAuthoredColorActionDescription as A;
        if !matches!(self.description, A::Compute(_) | A::ReadFact(_)) {
            return Err(ColorProtocolError::WrongAction.into());
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
            Work::Ready => Err(ColorProtocolError::WrongAction.into()),
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
