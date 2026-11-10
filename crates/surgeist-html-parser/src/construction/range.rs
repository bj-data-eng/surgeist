//! Controlled Range arithmetic and generated replacement in the existing owner.
use super::*;
use crate::keyed::{Table, slots};
use provenance::{
    HtmlParseRangeBranch, HtmlParseRangeOperandRole, RangeComputation, RangeOperand,
    RangeOperandTable, RangeOriginal,
};
use surgeist_html::attributes::input_bounds::{HtmlInputNumericContent, HtmlInputNumericStep};
use surgeist_html::bounded::numeric::{
    HtmlBestFiniteNumberCursor, HtmlFiniteDecimalIngressCursor, HtmlFloatingPointGrammarCursor,
    HtmlFloatingPointParseMode, HtmlRangeBufferHeader, HtmlRangeLatticeCursor,
    HtmlRangeMidpointCursor, HtmlRangeNumericError, HtmlRangeNumericHeaders, HtmlRangeNumericReply,
    HtmlRangeNumericRequest, HtmlRangeScratchRole,
};
use surgeist_html::construction::ConstructionNodeKind;

const SCRATCH_COUNT: usize = 8;
/// One scratch value per [`HtmlRangeScratchRole`]; a lookup cannot be out of range.
type ScratchTable<T> = Table<HtmlRangeScratchRole, T, SCRATCH_COUNT>;
slots!(HtmlRangeScratchRole, SCRATCH_COUNT, {
    Coefficient => 0,
    Lower => 1,
    Upper => 2,
    Left => 3,
    Right => 4,
    Quotient => 5,
    Remainder => 6,
    Trial => 7,
});
// The HTML owner's own index and role list agree with the table's slots.
#[expect(
    clippy::indexing_slicing,
    reason = "evaluated at compile time with i < SCRATCH_COUNT, the array length: an index out of range fails the build"
)]
const _: () = {
    let mut i = 0;
    while i < SCRATCH_COUNT {
        assert!(HtmlRangeScratchRole::ALL[i].index() == i);
        assert!(HtmlRangeScratchRole::ALL[i] as usize == i);
        i += 1;
    }
};

// Closed fixture scheduling at actual committed ingress replies only.
#[cfg(test)]
#[derive(Clone, Copy)]
pub(super) enum RangeIngressCancellationPoint {
    CoefficientFold,
    DecimalPower,
}
#[cfg(test)]
pub(super) struct RangeIngressCancellation {
    pub(super) point: RangeIngressCancellationPoint,
    pub(super) scalars_after_fold: u8,
    pub(super) token: crate::HtmlParseCancellation,
}

/// The bytes that name which machine runs: a one-byte tag.
const MACHINE_TAG_BYTES: usize = mem::size_of::<u8>();
#[cfg(test)]
#[derive(Clone, Copy, Debug)]
enum MachineKind {
    Ingress,
    Best,
    Lattice,
    Midpoint,
}
/// The one numeric cursor a staged computation is running.
#[derive(Clone, Copy, Debug)]
pub(super) enum Machine {
    Ingress(HtmlFiniteDecimalIngressCursor),
    Best(HtmlBestFiniteNumberCursor),
    Lattice(HtmlRangeLatticeCursor),
    Midpoint(HtmlRangeMidpointCursor),
}
impl Machine {
    #[cfg(test)]
    fn kind(&self) -> MachineKind {
        match self {
            Self::Ingress(_) => MachineKind::Ingress,
            Self::Best(_) => MachineKind::Best,
            Self::Lattice(_) => MachineKind::Lattice,
            Self::Midpoint(_) => MachineKind::Midpoint,
        }
    }
    fn ingress(&self) -> Option<&HtmlFiniteDecimalIngressCursor> {
        match self {
            Self::Ingress(cursor) => Some(cursor),
            _ => None,
        }
    }
    fn best(&self) -> Option<&HtmlBestFiniteNumberCursor> {
        match self {
            Self::Best(cursor) => Some(cursor),
            _ => None,
        }
    }
    fn lattice(&self) -> Option<&HtmlRangeLatticeCursor> {
        match self {
            Self::Lattice(cursor) => Some(cursor),
            _ => None,
        }
    }
    fn midpoint(&self) -> Option<&HtmlRangeMidpointCursor> {
        match self {
            Self::Midpoint(cursor) => Some(cursor),
            _ => None,
        }
    }
    fn request(&self) -> Option<HtmlRangeNumericRequest> {
        match self {
            Self::Ingress(cursor) => cursor.request(),
            Self::Best(cursor) => cursor.request(),
            Self::Lattice(cursor) => cursor.request(),
            Self::Midpoint(cursor) => cursor.request(),
        }
    }
    fn headers(&self) -> Option<HtmlRangeNumericHeaders> {
        match self {
            Self::Ingress(cursor) => Some(cursor.headers()),
            Self::Best(cursor) => Some(cursor.headers()),
            _ => None,
        }
    }
    fn receiver_bytes(&self) -> usize {
        match self {
            Self::Ingress(_) => mem::size_of::<HtmlFiniteDecimalIngressCursor>(),
            Self::Best(_) => mem::size_of::<HtmlBestFiniteNumberCursor>(),
            Self::Lattice(_) => mem::size_of::<HtmlRangeLatticeCursor>(),
            Self::Midpoint(_) => mem::size_of::<HtmlRangeMidpointCursor>(),
        }
    }
    fn reply(&mut self, reply: HtmlRangeNumericReply) -> Result<(), HtmlRangeNumericError> {
        match self {
            Self::Ingress(cursor) => cursor.reply(reply),
            Self::Best(cursor) => cursor.reply(reply),
            Self::Lattice(cursor) => cursor.reply(reply),
            Self::Midpoint(cursor) => cursor.reply(reply),
        }
    }
}
/// The buffers every numeric computation of one staging shares.
pub(super) struct RangeScratch {
    buffers: ScratchTable<OwnedVec<u8>>,
    output: Option<Utf16Buffer>,
    source: usize,
}
/// A staged Range computation: idle until the first numeric machine starts,
/// then running the machine that last started.
#[expect(
    clippy::large_enum_variant,
    reason = "one staging exists per session and is held inline; boxing would add an unaccounted allocation"
)]
pub(super) enum RangeStaging {
    Idle(RangeScratch),
    Running {
        scratch: RangeScratch,
        machine: Machine,
    },
}
impl std::ops::Deref for RangeStaging {
    type Target = RangeScratch;
    fn deref(&self) -> &RangeScratch {
        match self {
            Self::Idle(scratch) | Self::Running { scratch, .. } => scratch,
        }
    }
}
impl std::ops::DerefMut for RangeStaging {
    fn deref_mut(&mut self) -> &mut RangeScratch {
        match self {
            Self::Idle(scratch) | Self::Running { scratch, .. } => scratch,
        }
    }
}
impl RangeScratch {
    #[cfg(test)]
    fn owned_cells(&self) -> usize {
        self.buffers.as_array().iter().map(OwnedVec::len).sum()
    }
    #[cfg(test)]
    fn right_cache_extent(&self) -> (usize, usize) {
        let buffer = self.buffers.get(HtmlRangeScratchRole::Right);
        (buffer.len(), buffer.capacity())
    }
    fn new(c: &mut ParseControl, source: usize) -> Result<Self, SessionError> {
        Ok(Self {
            buffers: ScratchTable::new([
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
            ]),
            output: Some(Utf16Buffer::new(c)?),
            source,
        })
    }
    fn release(self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        for buffer in self.buffers.into_array() {
            buffer.release(c)?;
        }
        if let Some(output) = self.output {
            output.release(c)?;
        }
        Ok(())
    }
    fn headers(&self, c: &mut ParseControl) -> Result<HtmlRangeNumericHeaders, SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 9)?;
        c.charge(
            HtmlParseWorkCategory::CopyByte,
            mem::size_of::<HtmlRangeNumericHeaders>() as u64,
        )?;
        // An actual non-ZST Vec obeys the header's allocation invariants; a
        // violation is reported as the capacity-mismatch stop.
        fn actual_header(
            c: &mut ParseControl,
            len: usize,
            capacity: usize,
        ) -> Result<HtmlRangeBufferHeader, SessionError> {
            HtmlRangeBufferHeader::new(len, capacity).map_err(|error| numeric_failure(c, error))
        }
        let mut buffers = [HtmlRangeBufferHeader::default(); SCRATCH_COUNT];
        for (header, buffer) in buffers.iter_mut().zip(self.buffers.as_array()) {
            *header = actual_header(c, buffer.len(), buffer.capacity())?;
        }
        let output = match &self.output {
            Some(output) => actual_header(c, output.len(), output.capacity())?,
            None => actual_header(c, 0, 0)?,
        };
        Ok(HtmlRangeNumericHeaders::new(buffers, output))
    }
    fn scratch_empty(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        for role in HtmlRangeScratchRole::ALL {
            while self.buffers.get_mut(role).try_pop(c)?.is_some() {}
        }
        Ok(())
    }
}
impl RangeStaging {
    fn new(c: &mut ParseControl, source: usize) -> Result<Self, SessionError> {
        RangeScratch::new(c, source).map(Self::Idle)
    }
    pub(super) fn release(self, c: &mut ParseControl) -> Result<(), HtmlParseStop> {
        match self {
            Self::Idle(scratch) | Self::Running { scratch, .. } => scratch.release(c),
        }
    }
    /// The machine of the last started computation, if any started.
    fn machine(&self) -> Option<&Machine> {
        match self {
            Self::Idle(_) => None,
            Self::Running { machine, .. } => Some(machine),
        }
    }
    fn machine_mut(&mut self) -> Option<&mut Machine> {
        match self {
            Self::Idle(_) => None,
            Self::Running { machine, .. } => Some(machine),
        }
    }
    /// Starts `machine` over the same scratch. Nothing fallible happens
    /// between taking the scratch and returning the new phase.
    fn start(self, machine: Machine) -> Self {
        let (Self::Idle(scratch) | Self::Running { scratch, .. }) = self;
        Self::Running { scratch, machine }
    }
    #[cfg(test)]
    pub(super) fn owned_cells(&self) -> usize {
        RangeScratch::owned_cells(self)
    }
    #[cfg(test)]
    pub(super) fn right_cache_extent(&self) -> (usize, usize) {
        RangeScratch::right_cache_extent(self)
    }
    #[cfg(test)]
    pub(super) fn ingress_fixture_state(
        &self,
    ) -> (
        usize,
        &[u8],
        usize,
        usize,
        Option<&HtmlFiniteDecimalIngressCursor>,
    ) {
        (
            self.source,
            self.buffers
                .get(HtmlRangeScratchRole::Coefficient)
                .as_slice(),
            self.buffers.get(HtmlRangeScratchRole::Right).len(),
            self.buffers.get(HtmlRangeScratchRole::Trial).len(),
            self.machine().and_then(Machine::ingress),
        )
    }
}
fn numeric_failure(c: &mut ParseControl, error: HtmlRangeNumericError) -> SessionError {
    match error {
        HtmlRangeNumericError::Representation => c
            .stop_representation(HtmlParseResource::AuxiliaryEntries)
            .into(),
        _ => integrity(c),
    }
}
/// The range machine, its staged buffers or its numeric protocol is in a state
/// the machine's own sequencing excludes. Faults of the registry rows around
/// it (a missing stage, a missing attribute row) keep the registry-row label.
fn integrity(c: &mut ParseControl) -> SessionError {
    SessionError::Stop(
        c.fail_invariant(crate::parser::control::HtmlParseInvariant::RangeMachineState),
    )
}
impl SessionStorage {
    fn range_stage(&mut self, c: &mut ParseControl) -> Result<&mut RangeStaging, SessionError> {
        c.require_identity(&self.control)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        match self.effects.range.as_mut() {
            Some(stage) => Ok(stage),
            None => Err(latch_registry_row(c)),
        }
    }
    /// Starts the next numeric machine over the staged buffers.
    /// The caller has qualified the staging in the same step.
    fn start_machine(
        &mut self,
        c: &mut ParseControl,
        machine: Machine,
    ) -> Result<(), SessionError> {
        let stage = self
            .effects
            .range
            .take()
            .ok_or_else(|| latch_registry_row(c))?;
        self.effects.range = Some(stage.start(machine));
        Ok(())
    }
    fn execute_range(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        loop {
            c.require_identity(&self.control)?;
            c.charge(HtmlParseWorkCategory::RecordVisit, 2)?;
            c.charge(
                HtmlParseWorkCategory::InspectByte,
                const {
                    (mem::size_of::<Option<HtmlRangeNumericRequest>>()
                        + MACHINE_TAG_BYTES
                        + mem::size_of::<usize>()) as u64
                },
            )?;
            let stage = self
                .effects
                .range
                .as_ref()
                .ok_or_else(|| latch_registry_row(c))?;
            let Some(request) = stage.machine().ok_or_else(|| integrity(c))?.request() else {
                let expected = stage.machine().and_then(Machine::headers);
                if let Some(expected) = expected {
                    let actual = stage.headers(c)?;
                    if expected != actual {
                        return Err(integrity(c));
                    }
                }
                return Ok(());
            };
            payload_write::<HtmlRangeNumericRequest>(c, 1)?;
            let reply =
                match request {
                    HtmlRangeNumericRequest::SourceUnit { index } => {
                        let source = stage.source;
                        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                        c.charge_unit_inspection()?;
                        let unit = admitted_input_string(&self.payloads, c, source)?
                            .code_units()
                            .get(index)
                            .copied()
                            .ok_or_else(|| integrity(c))?;
                        HtmlRangeNumericReply::Unit(unit)
                    }
                    HtmlRangeNumericRequest::ReadCells { cells } => {
                        let mut read = [None; 3];
                        for (out, cell) in read.iter_mut().zip(cells) {
                            if let Some(cell) = cell {
                                let buffer = stage.buffers.get(cell.role());
                                buffer.require_disposal_owner(c)?;
                                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                                c.charge(HtmlParseWorkCategory::InspectByte, 1)?;
                                *out = Some(
                                    *buffer
                                        .as_slice()
                                        .get(cell.index())
                                        .ok_or_else(|| integrity(c))?,
                                );
                            }
                        }
                        HtmlRangeNumericReply::Cells(read)
                    }
                    HtmlRangeNumericRequest::Reserve { role, required_len } => {
                        let stage = self.range_stage(c)?;
                        let buffer = stage.buffers.get_mut(role);
                        let additional = required_len
                            .checked_sub(buffer.len())
                            .ok_or_else(|| integrity(c))?;
                        buffer.reserve_geometric(c, additional)?;
                        HtmlRangeNumericReply::Reserved {
                            actual_capacity: buffer.capacity(),
                            actual_len: buffer.len(),
                        }
                    }
                    HtmlRangeNumericRequest::AppendCell { role, value } => {
                        let stage = self.range_stage(c)?;
                        let buffer = stage.buffers.get_mut(role);
                        if value > 9 || buffer.len() == buffer.capacity() {
                            return Err(integrity(c));
                        }
                        buffer.try_push(c, value)?;
                        // Test scheduling only: the original token is cancelled after
                        // this actual admitted cell commit; normal reply admission polls.
                        #[cfg(test)]
                        {
                            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                            c.charge(
                                HtmlParseWorkCategory::InspectByte,
                                mem::size_of::<
                                    Option<(
                                        Option<HtmlRangeScratchRole>,
                                        crate::HtmlParseCancellation,
                                    )>,
                                >() as u64,
                            )?;
                            if self.cancel_after_range_append.as_ref().is_some_and(
                                |(selected, _)| selected.is_none_or(|target| target == role),
                            ) {
                                payload_write::<
                                    Option<(
                                        Option<HtmlRangeScratchRole>,
                                        crate::HtmlParseCancellation,
                                    )>,
                                >(c, 1)?;
                                c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                                if let Some((_, token)) = self.cancel_after_range_append.take() {
                                    token.cancel();
                                }
                            }
                        }
                        HtmlRangeNumericReply::Ack
                    }
                    HtmlRangeNumericRequest::WriteCell { cell, value } => {
                        let stage = self.range_stage(c)?;
                        if value > 9 {
                            return Err(integrity(c));
                        }
                        payload_write::<u8>(c, 1)?;
                        *stage
                            .buffers
                            .get_mut(cell.role())
                            .get_mut(cell.index(), c)?
                            .ok_or_else(|| integrity(c))? = value;
                        HtmlRangeNumericReply::Ack
                    }
                    HtmlRangeNumericRequest::PopCell { role } => {
                        if self
                            .range_stage(c)?
                            .buffers
                            .get_mut(role)
                            .try_pop(c)?
                            .is_none()
                        {
                            return Err(integrity(c));
                        }
                        HtmlRangeNumericReply::Ack
                    }
                    HtmlRangeNumericRequest::ScalarStep => {
                        c.charge(HtmlParseWorkCategory::StateStep, 1)?;
                        HtmlRangeNumericReply::Ack
                    }
                    HtmlRangeNumericRequest::ReserveOutput { required_len } => {
                        let output = self
                            .range_stage(c)?
                            .output
                            .as_mut()
                            .ok_or_else(|| integrity(c))?;
                        let additional = required_len
                            .checked_sub(output.len())
                            .ok_or_else(|| integrity(c))?;
                        output.reserve_units(c, additional)?;
                        HtmlRangeNumericReply::OutputReserved {
                            actual_capacity: output.capacity(),
                            actual_len: output.len(),
                        }
                    }
                    HtmlRangeNumericRequest::EmitUnit { unit } => {
                        let output = self
                            .range_stage(c)?
                            .output
                            .as_mut()
                            .ok_or_else(|| integrity(c))?;
                        if output.len() == output.capacity() {
                            return Err(integrity(c));
                        }
                        output.push_unit(unit, c)?;
                        HtmlRangeNumericReply::Ack
                    }
                };
            // The actual fixed receiver is copied transactionally by reply.
            // No primitive receiver makes a second nested transaction copy.
            payload_write::<HtmlRangeNumericReply>(c, 1)?;
            let receiver = self
                .range_stage(c)?
                .machine()
                .ok_or_else(|| integrity(c))?
                .receiver_bytes();
            c.admit_atomic_layout(receiver)?;
            c.charge(HtmlParseWorkCategory::CopyByte, receiver as u64)?;
            c.charge(HtmlParseWorkCategory::RecordWrite, 1)?;
            self.range_stage(c)?
                .machine_mut()
                .ok_or_else(|| integrity(c))?
                .reply(reply)
                .map_err(|error| numeric_failure(c, error))?;
            #[cfg(test)]
            {
                // Scheduling only. The real cursor reply above already committed
                // its fold or power transition. The next normal checkpoint polls.
                c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                c.charge(
                    HtmlParseWorkCategory::InspectByte,
                    mem::size_of::<Option<RangeIngressCancellation>>() as u64,
                )?;
                if self.cancel_after_range_ingress.is_some() {
                    c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                    c.charge(
                        HtmlParseWorkCategory::InspectByte,
                        (mem::size_of::<MachineKind>()
                            + mem::size_of::<HtmlRangeNumericRequest>()
                            + 2 * mem::size_of::<usize>()) as u64,
                    )?;
                    let ingress = matches!(
                        self.effects
                            .range
                            .as_ref()
                            .unwrap()
                            .machine()
                            .unwrap()
                            .kind(),
                        MachineKind::Ingress
                    );
                    if ingress {
                        let stage = self.effects.range.as_ref().unwrap();
                        let source = stage.source;
                        let coefficient = stage.buffers.get(HtmlRangeScratchRole::Coefficient);
                        coefficient.require_disposal_owner(c)?;
                        let one_cell = coefficient.len() == 1;
                        let raw = admitted_input_string(&self.payloads, c, source)?;
                        let dot_one = if raw.len() == 2 {
                            c.charge(HtmlParseWorkCategory::InspectByte, 4)?;
                            raw.code_units() == [46, 49]
                        } else {
                            false
                        };
                        // Only the actual one-cell .1 source is selected. Earlier
                        // default50/reset ingress is not a cell-fold fixture.
                        if one_cell && dot_one {
                            payload_write::<Option<RangeIngressCancellation>>(c, 1)?;
                            let observer = self.cancel_after_range_ingress.as_mut().unwrap();
                            let folded_one_cell = matches!(
                                request,
                                HtmlRangeNumericRequest::ReadCells {
                                    cells: [
                                        Some(cell),
                                        None,
                                        None
                                    ]
                                } if cell.role() == HtmlRangeScratchRole::Coefficient && cell.index() == 0
                            );
                            let cancel = if folded_one_cell {
                                match observer.point {
                                    RangeIngressCancellationPoint::CoefficientFold => true,
                                    RangeIngressCancellationPoint::DecimalPower => {
                                        // Fixed fixture .1: after its one actual cell fold,
                                        // Next advances index and Power multiplies B by10.
                                        observer.scalars_after_fold = 2;
                                        false
                                    }
                                }
                            } else if matches!(request, HtmlRangeNumericRequest::ScalarStep)
                                && observer.scalars_after_fold > 0
                            {
                                observer.scalars_after_fold -= 1;
                                observer.scalars_after_fold == 0
                            } else {
                                false
                            };
                            if cancel {
                                c.charge(HtmlParseWorkCategory::ProviderCall, 1)?;
                                self.cancel_after_range_ingress
                                    .take()
                                    .unwrap()
                                    .token
                                    .cancel();
                            }
                        }
                    }
                }
            }
        }
    }
    fn range_parse(
        &mut self,
        c: &mut ParseControl,
        source: usize,
        mode: HtmlFloatingPointParseMode,
    ) -> Result<(bool, Option<u64>), SessionError> {
        let parts = {
            let raw = admitted_input_string(&self.payloads, c, source)?;
            c.admit_atomic_layout(mem::size_of::<HtmlFloatingPointGrammarCursor<'_>>())?;
            payload_write::<HtmlFloatingPointGrammarCursor<'_>>(c, 1)?;
            let mut grammar = HtmlFloatingPointGrammarCursor::new(raw.code_units(), mode);
            while !grammar.is_finished() {
                c.charge(HtmlParseWorkCategory::StateStep, 1)?;
                if grammar.inspects_unit() {
                    c.charge_unit_inspection()?;
                }
                c.charge(
                    HtmlParseWorkCategory::CopyByte,
                    mem::size_of::<HtmlFloatingPointGrammarCursor<'_>>() as u64,
                )?;
                grammar.advance();
            }
            if !grammar.is_valid() {
                return Ok((false, None));
            }
            payload_write::<surgeist_html::bounded::numeric::HtmlRangeDecimalParts>(c, 1)?;
            grammar.range_decimal_parts().map_err(|_| integrity(c))?
        };
        let source_len = admitted_input_string(&self.payloads, c, source)?.len();
        let stage = self.range_stage(c)?;
        stage.scratch_empty(c)?;
        let headers = stage.headers(c)?;
        payload_write::<HtmlFiniteDecimalIngressCursor>(c, 1)?;
        let cursor = HtmlFiniteDecimalIngressCursor::new(parts, source_len, headers)
            .map_err(|error| numeric_failure(c, error))?;
        payload_write::<Option<Machine>>(c, 1)?;
        stage.source = source;
        self.start_machine(c, Machine::Ingress(cursor))?;
        self.execute_range(c)?;
        let stage = self.range_stage(c)?;
        let Some(cursor) = stage.machine().and_then(Machine::ingress) else {
            return Err(integrity(c));
        };
        match cursor.result().ok_or_else(|| integrity(c))? {
            Ok(bits) => Ok((true, Some(bits))),
            Err(HtmlRangeNumericError::Overflow) => Ok((true, None)),
            Err(_) => Err(integrity(c)),
        }
    }
    fn range_attribute_slot(
        &self,
        c: &mut ParseControl,
        node: usize,
        name: &str,
    ) -> Result<Option<usize>, SessionError> {
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let ConstructionNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .ok_or_else(|| latch_registry_row(c))?
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        for (position, attribute) in element.attributes().iter().enumerate() {
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            if attribute.name().namespace().uri().is_none()
                && exact_ascii(c, Some(attribute.name().local_name()), name)?
            {
                return self.attribute_value_slot(c, node, position).map(Some);
            }
        }
        Ok(None)
    }
    fn range_content(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        name: &str,
    ) -> Result<(HtmlInputNumericContent, RangeOperand), SessionError> {
        let Some(slot) = self.range_attribute_slot(c, node, name)? else {
            return Ok((HtmlInputNumericContent::Absent, RangeOperand::Absent));
        };
        let edge = self.range_payload_edge(c, slot)?;
        let (_, value) = self.range_parse(c, slot, HtmlFloatingPointParseMode::Recovery)?;
        match value {
            Some(bits) => Ok((
                HtmlInputNumericContent::Value(
                    surgeist_html::microsyntax::numeric::HtmlFloatingPointNumber::try_from(
                        f64::from_bits(bits),
                    )
                    .map_err(|_| integrity(c))?,
                ),
                RangeOperand::Finite(edge),
            )),
            None => Ok((
                HtmlInputNumericContent::Failed,
                RangeOperand::ConversionFailed(edge),
            )),
        }
    }
    pub(super) fn install_range_input_value(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        slot: usize,
        source: usize,
    ) -> Result<(), SessionError> {
        c.require_identity(&self.control)?;
        let previous = self.input_raw_slot(c, slot)?;
        let state = self.read_input_state(c, slot)?;
        if state.flags.input_type() != surgeist_html::attributes::HtmlInputType::Range {
            return Err(SessionError::InputMetadataUnavailable);
        }
        self.range_payload_edge(c, source)?;
        self.releasable_string(c, previous, false)?;
        // The previous range operation is still owed: installation releases
        // its staging before it returns, and a stop ends the parse.
        if self.effects.range.is_some() {
            return Err(c
                .fail_invariant(crate::parser::control::HtmlParseInvariant::PendingFormEffects)
                .into());
        }
        payload_write::<Option<RangeStaging>>(c, 1)?;
        let stage = RangeStaging::new(c, source)?;
        self.effects.range = Some(stage);
        let result = self.range_policy(c, node, slot, source);
        if result.is_ok() {
            payload_write::<Option<RangeStaging>>(c, 1)?;
            self.effects
                .range
                .take()
                .ok_or_else(|| integrity(c))?
                .release(c)?;
        }
        match result {
            Ok(()) => Ok(()),
            Err(SessionError::Stop(stop)) => Err(SessionError::Stop(stop)),
            // A row-level fault met inside the policy (an input or attribute
            // row that is missing or of another kind) is classified like any
            // other: only an outcome no policy step produces is a fault of the
            // machine's own state.
            Err(error) => Err(match session_fault_invariant(&error) {
                Some(invariant) => SessionError::Stop(c.fail_invariant(invariant)),
                None => integrity(c),
            }),
        }
    }
    fn range_policy(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        slot: usize,
        source: usize,
    ) -> Result<(), SessionError> {
        use surgeist_html::bounded::input::{
            HtmlInputRangeNumber, HtmlInputRangeReplacement as B,
            HtmlInputRangeSanitizationAction as A, HtmlInputRangeSanitizationCursor,
            HtmlInputRangeSanitizationReply as R, HtmlInputRangeValue,
        };
        let source_edge = self.range_payload_edge(c, source)?;
        let current = self.input_raw_slot(c, slot)?;
        let state = self.read_input_state(c, slot)?;
        c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
        let Some(RetainedPayload::Input { complete, .. }) = self.payloads.values.get(slot) else {
            return Err(integrity(c));
        };
        let fresh_empty = !*complete
            && !state.flags.dirty_value()
            && source == current
            && admitted_input_string(&self.payloads, c, source)?.is_empty();
        let mut original = if fresh_empty {
            RangeOriginal::FreshEmpty
        } else {
            RangeOriginal::Group(source_edge)
        };
        let (valid, value) = self.range_parse(c, source, HtmlFloatingPointParseMode::Author)?;
        let (min, min_operand) = self.range_content(c, node, "min")?;
        let (max, max_operand) = self.range_content(c, node, "max")?;
        let mut operands = RangeOperandTable::new([
            min_operand,
            max_operand,
            RangeOperand::NotConsulted,
            RangeOperand::NotConsulted,
        ]);
        payload_write::<HtmlInputRangeValue>(c, 1)?;
        let value = match (valid, value) {
            (false, _) => HtmlInputRangeValue::InvalidGrammar,
            (true, None) => HtmlInputRangeValue::ConversionFailed,
            (true, Some(bits)) => HtmlInputRangeValue::Finite(
                HtmlInputRangeNumber::try_from(f64::from_bits(bits)).map_err(|_| integrity(c))?,
            ),
        };
        payload_write::<HtmlInputRangeSanitizationCursor>(c, 1)?;
        let mut policy = HtmlInputRangeSanitizationCursor::new(value, min, max);
        loop {
            // New source-policy receiver/action/reply work is explicit. Numeric
            // requests below retain their existing independent service charges.
            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
            c.charge(HtmlParseWorkCategory::StateStep, 1)?;
            c.charge(
                HtmlParseWorkCategory::InspectByte,
                mem::size_of::<HtmlInputRangeSanitizationCursor>() as u64,
            )?;
            payload_write::<A>(c, 1)?;
            let reply = match policy.action() {
                A::Midpoint { minimum, maximum } => {
                    payload_write::<HtmlRangeMidpointCursor>(c, 1)?;
                    let cursor = HtmlRangeMidpointCursor::new(
                        minimum.value().to_bits(),
                        maximum.value().to_bits(),
                    )
                    .map_err(|_| integrity(c))?;
                    payload_write::<Option<Machine>>(c, 1)?;
                    self.range_stage(c)?;
                    self.start_machine(c, Machine::Midpoint(cursor))?;
                    self.execute_range(c)?;
                    let Some(cursor) = self.range_stage(c)?.machine().and_then(Machine::midpoint)
                    else {
                        return Err(integrity(c));
                    };
                    R::Midpoint(
                        HtmlInputRangeNumber::try_from(f64::from_bits(
                            cursor.result().ok_or_else(|| integrity(c))?,
                        ))
                        .map_err(|_| integrity(c))?,
                    )
                }
                A::Replace { value, branch } => {
                    let step = branch == B::StepCorrection;
                    payload_write::<(
                        surgeist_html::attributes::input_bounds::HtmlInputNumericBoundOrigin,
                        surgeist_html::attributes::input_bounds::HtmlInputNumericBoundOrigin,
                    )>(c, 1)?;
                    let (minimum, maximum) = policy.bound_origins();
                    let (step_origin, base_origin) = if step {
                        payload_write::<Option<surgeist_html::attributes::input_bounds::HtmlInputNumericConstraints>>(c, 1)?;
                        let constraints = policy.constraints().ok_or_else(|| integrity(c))?;
                        (
                            Some(constraints.step_origin()),
                            Some(constraints.step_base_origin()),
                        )
                    } else {
                        (None, None)
                    };
                    let branch = match branch {
                        B::InvalidDefaultMinimum => HtmlParseRangeBranch::InvalidDefaultMinimum,
                        B::InvalidDefaultMidpoint => HtmlParseRangeBranch::InvalidDefaultMidpoint,
                        B::Underflow => HtmlParseRangeBranch::Underflow,
                        B::Overflow => HtmlParseRangeBranch::Overflow,
                        B::StepCorrection => HtmlParseRangeBranch::StepCorrection,
                    };
                    self.range_emit(
                        c,
                        slot,
                        value.value().to_bits(),
                        RangeComputation {
                            branch,
                            original,
                            operands,
                            minimum: Some(minimum),
                            maximum: Some(maximum),
                            step: step_origin,
                            base: base_origin,
                        },
                    )?;
                    if !step {
                        let current = self.input_raw_slot(c, slot)?;
                        original = RangeOriginal::Group(self.range_payload_edge(c, current)?);
                    }
                    R::Applied
                }
                A::ReadStep => {
                    let step_slot = self.range_attribute_slot(c, node, "step")?;
                    let step = if let Some(step_slot) = step_slot {
                        let edge = self.range_payload_edge(c, step_slot)?;
                        let any = {
                            let raw = admitted_input_string(&self.payloads, c, step_slot)?;
                            let keyword_len = HtmlInputNumericStep::ANY_KEYWORD.len();
                            c.charge(HtmlParseWorkCategory::RecordVisit, 1)?;
                            let mut any = raw.len() == keyword_len;
                            if any {
                                // The lengths are equal, so the enumeration covers
                                // the keyword exactly.
                                for (index, &unit) in raw.code_units().iter().enumerate() {
                                    c.charge_unit_inspection()?;
                                    c.charge(HtmlParseWorkCategory::StateStep, 1)?;
                                    if !HtmlInputNumericStep::any_keyword_unit_matches(index, unit)
                                    {
                                        any = false;
                                        break;
                                    }
                                }
                            }
                            any
                        };
                        if any {
                            *operands.get_mut(HtmlParseRangeOperandRole::Step) =
                                RangeOperand::Any(edge);
                            HtmlInputNumericStep::Any
                        } else {
                            let (_, result) = self.range_parse(
                                c,
                                step_slot,
                                HtmlFloatingPointParseMode::Recovery,
                            )?;
                            match result {
                                Some(bits) => {
                                    *operands.get_mut(HtmlParseRangeOperandRole::Step) =
                                        RangeOperand::Finite(edge);
                                    HtmlInputNumericStep::Value(
                                        surgeist_html::microsyntax::numeric::HtmlFloatingPointNumber::try_from(f64::from_bits(
                                            bits,
                                        ))
                                        .map_err(|_| integrity(c))?,
                                    )
                                }
                                None => {
                                    *operands.get_mut(HtmlParseRangeOperandRole::Step) =
                                        RangeOperand::ConversionFailed(edge);
                                    HtmlInputNumericStep::Failed
                                }
                            }
                        }
                    } else {
                        *operands.get_mut(HtmlParseRangeOperandRole::Step) = RangeOperand::Absent;
                        HtmlInputNumericStep::Absent
                    };
                    R::Step(step)
                }
                A::ReadValueContent => {
                    let (content, operand) = self.range_content(c, node, "value")?;
                    *operands.get_mut(HtmlParseRangeOperandRole::ValueContent) = operand;
                    R::ValueContent(content)
                }
                A::Lattice {
                    current,
                    minimum,
                    maximum,
                    base,
                    step,
                } => {
                    payload_write::<HtmlRangeLatticeCursor>(c, 1)?;
                    let cursor = HtmlRangeLatticeCursor::new(
                        current.value().to_bits(),
                        minimum.value().to_bits(),
                        maximum.map(|value| value.value().to_bits()),
                        base.value().to_bits(),
                        step.value().to_bits(),
                    )
                    .map_err(|_| integrity(c))?;
                    payload_write::<Option<Machine>>(c, 1)?;
                    self.range_stage(c)?;
                    self.start_machine(c, Machine::Lattice(cursor))?;
                    self.execute_range(c)?;
                    let Some(cursor) = self.range_stage(c)?.machine().and_then(Machine::lattice)
                    else {
                        return Err(integrity(c));
                    };
                    R::Lattice(
                        cursor
                            .result()
                            .ok_or_else(|| integrity(c))?
                            .map(|bits| {
                                HtmlInputRangeNumber::try_from(f64::from_bits(bits))
                                    .map_err(|_| integrity(c))
                            })
                            .transpose()?,
                    )
                }
                A::Complete => break,
            };
            payload_write::<R>(c, 1)?;
            // reply() transactionally copies the fixed receiver once. Do not
            // hide this fee in unchanged arithmetic/scratch servicing.
            payload_write::<HtmlInputRangeSanitizationCursor>(c, 1)?;
            policy.reply(reply).map_err(|_| integrity(c))?;
        }
        if !policy.replaced() {
            let field = provenance::copy_generated_input_value(
                &self.payloads,
                self.evidence.as_ref(),
                c,
                source,
            )?;
            self.replace_input_raw(c, slot, field)?;
        }
        Ok(())
    }
    fn range_emit(
        &mut self,
        c: &mut ParseControl,
        slot: usize,
        bits: u64,
        computation: RangeComputation,
    ) -> Result<(), SessionError> {
        let previous = self.input_raw_slot(c, slot)?;
        let dispose_previous = self.releasable_string(c, previous, false)?;
        if self.range_stage(c)?.output.is_none() {
            payload_write::<Option<Utf16Buffer>>(c, 1)?;
            let output = Utf16Buffer::new(c)?;
            self.range_stage(c)?.output = Some(output);
        }
        let stage = self.range_stage(c)?;
        stage.scratch_empty(c)?;
        let headers = stage.headers(c)?;
        payload_write::<HtmlBestFiniteNumberCursor>(c, 1)?;
        let cursor = HtmlBestFiniteNumberCursor::new(f64::from_bits(bits), headers)
            .map_err(|error| numeric_failure(c, error))?;
        payload_write::<Option<Machine>>(c, 1)?;
        self.start_machine(c, Machine::Best(cursor))?;
        self.execute_range(c)?;
        let units = self
            .range_stage(c)?
            .output
            .as_ref()
            .ok_or_else(|| integrity(c))?
            .len();
        let Some(cursor) = self.range_stage(c)?.machine().and_then(Machine::best) else {
            return Err(integrity(c));
        };
        if cursor.result() != Some(units) {
            return Err(integrity(c));
        }
        payload_write::<Option<OriginGroupRef>>(c, 1)?;
        let group = self.prepare_range_group(c, units, computation)?;
        // The staging is re-qualified before the generated group is parked.
        self.range_stage(c)?;
        payload_write::<Option<Utf16Buffer>>(c, 1)?;
        payload_write::<Option<OriginGroupRef>>(c, 1)?;
        let stage = self.range_stage(c)?;
        let output = stage.output.take().ok_or_else(|| integrity(c))?;
        let value = output.freeze(c)?;
        let (native, next) = self.register_group_string_owned(c, value, group)?;
        drop(native);
        payload_write::<usize>(c, 1)?;
        payload_write::<RetainedPayload>(c, 2)?;
        self.retire_field_group(c, previous)?;
        let Some(RetainedPayload::Input { raw, .. }) = self.payloads.values.get_mut(slot) else {
            return Err(integrity(c));
        };
        *raw = next;
        if dispose_previous {
            self.dispose_aggregate_string(c, previous)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod absent_owner_tests {
    use super::*;
    use crate::parser::control::HtmlParseInvariant;

    /// The integrity fault of the range machine is the machine's own staged
    /// buffers and numeric protocol, not a registry row that is missing.
    #[test]
    fn a_range_machine_integrity_fault_reports_the_range_machine_state() {
        let state = HtmlParseStop::InternalInvariant(HtmlParseInvariant::RangeMachineState);
        let mut c = crate::construction::tests::control();
        assert_eq!(integrity(&mut c), SessionError::Stop(state));
        assert_eq!(c.stop(), Some(state));
    }

    /// A second range value installed while the first one's staging is still
    /// held starts a range operation while the previous one is still owed:
    /// installation releases its staging before it returns, and a stop ends
    /// the parse. It is the pending-operation fault the idle checks report for
    /// the same staging, not a fault of the machine's own state.
    #[test]
    fn installing_a_range_value_over_held_staging_reports_the_pending_operation() {
        let pending = HtmlParseStop::InternalInvariant(HtmlParseInvariant::PendingFormEffects);
        // The default step's arithmetic alone exceeds the ordinary test budget.
        let unbounded = crate::construction::tests::limits()
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap();
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<body><input type=range value=50 step=any>"),
            unbounded,
        );
        let (mut c, mut session, _proof, nodes) =
            crate::construction::effects::tests::graph(&request);
        let input = crate::construction::effects::tests::named(&nodes, "input", 0).index;
        let storage = session.active_mut().unwrap();
        let slot = storage
            .payloads
            .values
            .iter()
            .position(|row| {
                matches!(row, RetainedPayload::Input { node: Some(owner), .. } if *owner == input)
            })
            .unwrap();
        let raw = storage.input_raw_slot(&mut c, slot).unwrap();
        assert!(storage.effects.range.is_none(), "installation released it");
        storage.effects.range = Some(RangeStaging::new(&mut c, raw).unwrap());
        assert_eq!(
            storage.install_range_input_value(&mut c, input, slot, raw),
            Err(SessionError::Stop(pending))
        );
        assert_eq!(c.stop(), Some(pending));
        // The staging that was already held is still the registry's to release.
        storage
            .effects
            .range
            .take()
            .unwrap()
            .release(&mut c)
            .unwrap();
        session.release(&mut c).unwrap();
        crate::construction::tests::storage_zero(&c);
    }

    /// A row-level fault met inside the range policy is the fault of the row,
    /// not of the machine: asking for the "min" content of a record that is not
    /// an element finds a record of another kind. The machine itself is not in
    /// a state its sequencing excludes.
    #[test]
    fn a_row_fault_inside_the_range_policy_reports_the_registry_row() {
        let registry_row = HtmlParseStop::InternalInvariant(HtmlParseInvariant::RegistryRow);
        let unbounded = crate::construction::tests::limits()
            .into_builder()
            .work_credits(u64::MAX)
            .finish()
            .unwrap();
        let request = crate::HtmlDocumentParseRequest::offline(
            crate::HtmlSource::utf8("<body><input type=range value=50 step=any>"),
            unbounded,
        );
        let (mut c, mut session, _proof, nodes) =
            crate::construction::effects::tests::graph(&request);
        let input = crate::construction::effects::tests::named(&nodes, "input", 0).index;
        let storage = session.active_mut().unwrap();
        let slot = storage
            .payloads
            .values
            .iter()
            .position(|row| {
                matches!(row, RetainedPayload::Input { node: Some(owner), .. } if *owner == input)
            })
            .unwrap();
        let raw = storage.input_raw_slot(&mut c, slot).unwrap();
        // Record 0 is the document, which has no attributes to read.
        assert_eq!(
            storage.install_range_input_value(&mut c, 0, slot, raw),
            Err(SessionError::Stop(registry_row))
        );
        assert_eq!(c.stop(), Some(registry_row));
        storage
            .effects
            .range
            .take()
            .unwrap()
            .release(&mut c)
            .unwrap();
        session.release(&mut c).unwrap();
        crate::construction::tests::storage_zero(&c);
    }

    /// Staging that holds no machine has nothing to drive. The staging is
    /// present, so no registry row is missing: the machine state inside it is.
    #[test]
    fn executing_staging_without_a_machine_reports_the_range_machine_state() {
        let state = HtmlParseStop::InternalInvariant(HtmlParseInvariant::RangeMachineState);
        let mut c = crate::construction::tests::control();
        let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
        let storage = session.active_mut().unwrap();
        storage.effects.range = Some(RangeStaging::new(&mut c, 0).unwrap());
        assert!(storage.effects.range.as_ref().unwrap().machine().is_none());
        assert_eq!(
            storage.execute_range(&mut c),
            Err(SessionError::Stop(state))
        );
        assert_eq!(c.stop(), Some(state));
        storage
            .effects
            .range
            .take()
            .unwrap()
            .release(&mut c)
            .unwrap();
        session.release(&mut c).unwrap();
        crate::construction::tests::storage_zero(&c);
    }

    /// Range work that finds no range owner has lost a retained owner; it is
    /// not an operation that is still owed.
    #[test]
    fn range_work_without_its_owner_reports_the_registry_row() {
        let registry_row = HtmlParseStop::InternalInvariant(HtmlParseInvariant::RegistryRow);
        for execute in [false, true] {
            let mut c = crate::construction::tests::control();
            let mut session = ParserArenaSession::new(&mut c, RootKind::Fragment).unwrap();
            let storage = session.active_mut().unwrap();
            assert!(storage.effects.range.is_none());
            let result = if execute {
                storage.execute_range(&mut c)
            } else {
                storage.range_stage(&mut c).map(|_| ())
            };
            assert_eq!(result, Err(SessionError::Stop(registry_row)), "{execute}");
            assert_eq!(c.stop(), Some(registry_row), "{execute}");
            session.release(&mut c).unwrap();
        }
    }
}
