//! Controlled Range arithmetic and generated replacement in the existing owner.
use super::*;
use crate::bounded::{
    BestFiniteNumberCursor, FiniteDecimalIngressCursor, FloatingPointGrammarCursor,
    FloatingPointParseMode, RangeBufferHeader, RangeLatticeCursor, RangeMidpointCursor,
    RangeNumericError, RangeNumericHeaders, RangeNumericReply, RangeNumericRequest,
    RangeScratchRole,
};
use crate::{HtmlInputNumericContent, HtmlInputNumericStep};
use provenance::{RangeBranch, RangeComputation, RangeOperand, RangeOriginal};

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

#[derive(Clone, Copy, Debug)]
enum MachineKind {
    Ingress,
    Best,
    Lattice,
    Midpoint,
}
#[derive(Clone, Copy, Debug)]
struct Machine {
    kind: MachineKind,
    ingress: Option<FiniteDecimalIngressCursor>,
    best: Option<BestFiniteNumberCursor>,
    lattice: Option<RangeLatticeCursor>,
    midpoint: Option<RangeMidpointCursor>,
}
impl Machine {
    fn ingress(cursor: FiniteDecimalIngressCursor) -> Self {
        Self {
            kind: MachineKind::Ingress,
            ingress: Some(cursor),
            best: None,
            lattice: None,
            midpoint: None,
        }
    }
    fn best(cursor: BestFiniteNumberCursor) -> Self {
        Self {
            kind: MachineKind::Best,
            ingress: None,
            best: Some(cursor),
            lattice: None,
            midpoint: None,
        }
    }
    fn lattice(cursor: RangeLatticeCursor) -> Self {
        Self {
            kind: MachineKind::Lattice,
            ingress: None,
            best: None,
            lattice: Some(cursor),
            midpoint: None,
        }
    }
    fn midpoint(cursor: RangeMidpointCursor) -> Self {
        Self {
            kind: MachineKind::Midpoint,
            ingress: None,
            best: None,
            lattice: None,
            midpoint: Some(cursor),
        }
    }
    fn request(&self) -> Option<RangeNumericRequest> {
        match self.kind {
            MachineKind::Ingress => self.ingress.as_ref()?.request(),
            MachineKind::Best => self.best.as_ref()?.request(),
            MachineKind::Lattice => self.lattice.as_ref()?.request(),
            MachineKind::Midpoint => self.midpoint.as_ref()?.request(),
        }
    }
    fn headers(&self) -> Option<RangeNumericHeaders> {
        match self.kind {
            MachineKind::Ingress => self
                .ingress
                .as_ref()
                .map(FiniteDecimalIngressCursor::headers),
            MachineKind::Best => self.best.as_ref().map(BestFiniteNumberCursor::headers),
            _ => None,
        }
    }
    fn receiver_bytes(&self) -> usize {
        match self.kind {
            MachineKind::Ingress => mem::size_of::<FiniteDecimalIngressCursor>(),
            MachineKind::Best => mem::size_of::<BestFiniteNumberCursor>(),
            MachineKind::Lattice => mem::size_of::<RangeLatticeCursor>(),
            MachineKind::Midpoint => mem::size_of::<RangeMidpointCursor>(),
        }
    }
    fn reply(&mut self, reply: RangeNumericReply) -> Result<(), RangeNumericError> {
        match self.kind {
            MachineKind::Ingress => self
                .ingress
                .as_mut()
                .ok_or(RangeNumericError::Integrity)?
                .reply(reply),
            MachineKind::Best => self
                .best
                .as_mut()
                .ok_or(RangeNumericError::Integrity)?
                .reply(reply),
            MachineKind::Lattice => self
                .lattice
                .as_mut()
                .ok_or(RangeNumericError::Integrity)?
                .reply(reply),
            MachineKind::Midpoint => self
                .midpoint
                .as_mut()
                .ok_or(RangeNumericError::Integrity)?
                .reply(reply),
        }
    }
}
pub(super) struct RangeStaging {
    buffers: [OwnedVec<u8>; 8],
    output: Option<Utf16Buffer>,
    machine: Option<Machine>,
    generated: Option<OriginGroupRef>,
    source: usize,
}
impl RangeStaging {
    #[cfg(test)]
    pub(super) fn owned_cells(&self) -> usize {
        self.buffers.iter().map(OwnedVec::len).sum()
    }
    #[cfg(test)]
    pub(super) fn right_cache_extent(&self) -> (usize, usize) {
        let buffer = &self.buffers[RangeScratchRole::Right.index()];
        (buffer.len(), buffer.capacity())
    }
    #[cfg(test)]
    pub(super) fn ingress_fixture_state(
        &self,
    ) -> (
        usize,
        &[u8],
        usize,
        usize,
        Option<&FiniteDecimalIngressCursor>,
    ) {
        (
            self.source,
            self.buffers[RangeScratchRole::Coefficient.index()].as_slice(),
            self.buffers[RangeScratchRole::Right.index()].len(),
            self.buffers[RangeScratchRole::Trial.index()].len(),
            self.machine
                .as_ref()
                .and_then(|machine| machine.ingress.as_ref()),
        )
    }
    fn new(c: &mut ParseControl, source: usize) -> Result<Self, SessionError> {
        Ok(Self {
            buffers: [
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
                OwnedVec::new(HtmlParseResource::AuxiliaryEntries)?,
            ],
            output: Some(Utf16Buffer::new(c)?),
            machine: None,
            generated: None,
            source,
        })
    }
    pub(super) fn release(self, c: &mut ParseControl) -> Result<(), ParseStop> {
        for buffer in self.buffers {
            buffer.release(c)?;
        }
        if let Some(output) = self.output {
            output.release(c)?;
        }
        Ok(())
    }
    fn headers(&self, c: &mut ParseControl) -> Result<RangeNumericHeaders, SessionError> {
        c.charge(WorkCategory::RecordVisit, 9)?;
        c.charge(
            WorkCategory::CopyByte,
            mem::size_of::<RangeNumericHeaders>() as u64,
        )?;
        fn actual_header(len: usize, capacity: usize) -> RangeBufferHeader {
            RangeBufferHeader::new(len, capacity)
                .expect("actual non-ZST Vec length/capacity obey allocation invariants")
        }
        Ok(RangeNumericHeaders::new(
            std::array::from_fn(|i| {
                actual_header(self.buffers[i].len(), self.buffers[i].capacity())
            }),
            self.output.as_ref().map_or_else(
                || actual_header(0, 0),
                |output| actual_header(output.len(), output.capacity()),
            ),
        ))
    }
    fn scratch_empty(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        for role in RangeScratchRole::ALL {
            while self.buffers[role.index()].try_pop(c)?.is_some() {}
        }
        Ok(())
    }
}
fn numeric_failure(c: &mut ParseControl, error: RangeNumericError) -> SessionError {
    match error {
        RangeNumericError::Representation => c
            .representation::<()>(HtmlParseResource::AuxiliaryEntries)
            .unwrap_err()
            .into(),
        _ => integrity(c),
    }
}
fn integrity(c: &mut ParseControl) -> SessionError {
    c.fail_invariant(crate::parser::control::Invariant::CapacityMismatch)
        .into()
}
impl SessionStorage {
    fn range_stage(&mut self, c: &mut ParseControl) -> Result<&mut RangeStaging, SessionError> {
        c.require_identity(&self.control)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        self.effects
            .range
            .as_mut()
            .ok_or(SessionError::PendingIntrinsicWork)
    }
    fn execute_range(&mut self, c: &mut ParseControl) -> Result<(), SessionError> {
        loop {
            c.require_identity(&self.control)?;
            c.charge(WorkCategory::RecordVisit, 2)?;
            c.charge(
                WorkCategory::InspectByte,
                (mem::size_of::<Option<RangeNumericRequest>>()
                    + mem::size_of::<MachineKind>()
                    + mem::size_of::<usize>()) as u64,
            )?;
            let stage = self
                .effects
                .range
                .as_ref()
                .ok_or(SessionError::PendingIntrinsicWork)?;
            let Some(request) = stage
                .machine
                .as_ref()
                .ok_or(SessionError::PendingIntrinsicWork)?
                .request()
            else {
                let expected = stage.machine.as_ref().and_then(Machine::headers);
                if let Some(expected) = expected {
                    let actual = stage.headers(c)?;
                    if expected != actual {
                        return Err(integrity(c));
                    }
                }
                return Ok(());
            };
            payload_write::<RangeNumericRequest>(c, 1)?;
            let reply = match request {
                RangeNumericRequest::SourceUnit { index } => {
                    let source = stage.source;
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    c.charge(WorkCategory::InspectByte, 2)?;
                    let unit = admitted_input_string(&self.payloads, c, source)?
                        .code_units()
                        .get(index)
                        .copied()
                        .ok_or_else(|| integrity(c))?;
                    RangeNumericReply::Unit(unit)
                }
                RangeNumericRequest::ReadCells { cells } => {
                    let mut read = [None; 3];
                    for (out, cell) in read.iter_mut().zip(cells) {
                        if let Some(cell) = cell {
                            let buffer = &stage.buffers[cell.role().index()];
                            buffer.require_disposal_owner(c)?;
                            c.charge(WorkCategory::RecordVisit, 1)?;
                            c.charge(WorkCategory::InspectByte, 1)?;
                            *out = Some(
                                *buffer
                                    .as_slice()
                                    .get(cell.index())
                                    .ok_or_else(|| integrity(c))?,
                            );
                        }
                    }
                    RangeNumericReply::Cells(read)
                }
                RangeNumericRequest::Reserve { role, required_len } => {
                    let stage = self.range_stage(c)?;
                    let buffer = &mut stage.buffers[role.index()];
                    let additional = required_len
                        .checked_sub(buffer.len())
                        .ok_or_else(|| integrity(c))?;
                    buffer.reserve_geometric(c, additional)?;
                    RangeNumericReply::Reserved {
                        actual_capacity: buffer.capacity(),
                        actual_len: buffer.len(),
                    }
                }
                RangeNumericRequest::AppendCell { role, value } => {
                    let stage = self.range_stage(c)?;
                    let buffer = &mut stage.buffers[role.index()];
                    if value > 9 || buffer.len() == buffer.capacity() {
                        return Err(integrity(c));
                    }
                    buffer.try_push(c, value)?;
                    // Test scheduling only: the original token is cancelled after
                    // this actual admitted cell commit; normal reply admission polls.
                    #[cfg(test)]
                    {
                        c.charge(WorkCategory::RecordVisit, 1)?;
                        c.charge(
                            WorkCategory::InspectByte,
                            mem::size_of::<
                                Option<(Option<RangeScratchRole>, crate::HtmlParseCancellation)>,
                            >() as u64,
                        )?;
                        if self
                            .cancel_after_range_append
                            .as_ref()
                            .is_some_and(|(selected, _)| {
                                selected.is_none_or(|target| target == role)
                            })
                        {
                            payload_write::<
                                Option<(Option<RangeScratchRole>, crate::HtmlParseCancellation)>,
                            >(c, 1)?;
                            c.charge(WorkCategory::ProviderCall, 1)?;
                            if let Some((_, token)) = self.cancel_after_range_append.take() {
                                token.cancel();
                            }
                        }
                    }
                    RangeNumericReply::Ack
                }
                RangeNumericRequest::WriteCell { cell, value } => {
                    let stage = self.range_stage(c)?;
                    if value > 9 {
                        return Err(integrity(c));
                    }
                    payload_write::<u8>(c, 1)?;
                    *stage.buffers[cell.role().index()]
                        .get_mut(cell.index(), c)?
                        .ok_or_else(|| integrity(c))? = value;
                    RangeNumericReply::Ack
                }
                RangeNumericRequest::PopCell { role } => {
                    if self.range_stage(c)?.buffers[role.index()]
                        .try_pop(c)?
                        .is_none()
                    {
                        return Err(integrity(c));
                    }
                    RangeNumericReply::Ack
                }
                RangeNumericRequest::ScalarStep => {
                    c.charge(WorkCategory::StateStep, 1)?;
                    RangeNumericReply::Ack
                }
                RangeNumericRequest::ReserveOutput { required_len } => {
                    let output = self
                        .range_stage(c)?
                        .output
                        .as_mut()
                        .ok_or_else(|| integrity(c))?;
                    let additional = required_len
                        .checked_sub(output.len())
                        .ok_or_else(|| integrity(c))?;
                    output.reserve_units(c, additional)?;
                    RangeNumericReply::OutputReserved {
                        actual_capacity: output.capacity(),
                        actual_len: output.len(),
                    }
                }
                RangeNumericRequest::EmitUnit { unit } => {
                    let output = self
                        .range_stage(c)?
                        .output
                        .as_mut()
                        .ok_or_else(|| integrity(c))?;
                    if output.len() == output.capacity() {
                        return Err(integrity(c));
                    }
                    output.push_unit(unit, c)?;
                    RangeNumericReply::Ack
                }
            };
            // The actual fixed receiver is copied transactionally by reply.
            // No primitive receiver makes a second nested transaction copy.
            payload_write::<RangeNumericReply>(c, 1)?;
            let receiver = self
                .range_stage(c)?
                .machine
                .as_ref()
                .ok_or_else(|| integrity(c))?
                .receiver_bytes();
            c.admit_atomic_layout(receiver)?;
            c.charge(WorkCategory::CopyByte, receiver as u64)?;
            c.charge(WorkCategory::RecordWrite, 1)?;
            self.range_stage(c)?
                .machine
                .as_mut()
                .ok_or_else(|| integrity(c))?
                .reply(reply)
                .map_err(|error| numeric_failure(c, error))?;
            #[cfg(test)]
            {
                // Scheduling only. The real cursor reply above already committed
                // its fold or power transition. The next normal checkpoint polls.
                c.charge(WorkCategory::RecordVisit, 1)?;
                c.charge(
                    WorkCategory::InspectByte,
                    mem::size_of::<Option<RangeIngressCancellation>>() as u64,
                )?;
                if self.cancel_after_range_ingress.is_some() {
                    c.charge(WorkCategory::RecordVisit, 1)?;
                    c.charge(
                        WorkCategory::InspectByte,
                        (mem::size_of::<MachineKind>()
                            + mem::size_of::<RangeNumericRequest>()
                            + 2 * mem::size_of::<usize>()) as u64,
                    )?;
                    let ingress = matches!(
                        self.effects
                            .range
                            .as_ref()
                            .unwrap()
                            .machine
                            .as_ref()
                            .unwrap()
                            .kind,
                        MachineKind::Ingress
                    );
                    if ingress {
                        let stage = self.effects.range.as_ref().unwrap();
                        let source = stage.source;
                        let coefficient = &stage.buffers[RangeScratchRole::Coefficient.index()];
                        coefficient.require_disposal_owner(c)?;
                        let one_cell = coefficient.len() == 1;
                        let raw = admitted_input_string(&self.payloads, c, source)?;
                        let dot_one = if raw.len() == 2 {
                            c.charge(WorkCategory::InspectByte, 4)?;
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
                                RangeNumericRequest::ReadCells {
                                    cells: [
                                        Some(cell),
                                        None,
                                        None
                                    ]
                                } if cell.role() == RangeScratchRole::Coefficient && cell.index() == 0
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
                            } else if matches!(request, RangeNumericRequest::ScalarStep)
                                && observer.scalars_after_fold > 0
                            {
                                observer.scalars_after_fold -= 1;
                                observer.scalars_after_fold == 0
                            } else {
                                false
                            };
                            if cancel {
                                c.charge(WorkCategory::ProviderCall, 1)?;
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
        mode: FloatingPointParseMode,
    ) -> Result<(bool, Option<u64>), SessionError> {
        let parts = {
            let raw = admitted_input_string(&self.payloads, c, source)?;
            c.admit_atomic_layout(mem::size_of::<FloatingPointGrammarCursor<'_>>())?;
            payload_write::<FloatingPointGrammarCursor<'_>>(c, 1)?;
            let mut grammar = FloatingPointGrammarCursor::new(raw.code_units(), mode);
            while !grammar.is_finished() {
                c.charge(WorkCategory::StateStep, 1)?;
                if grammar.inspects_unit() {
                    c.charge(WorkCategory::InspectByte, 2)?;
                }
                c.charge(
                    WorkCategory::CopyByte,
                    mem::size_of::<FloatingPointGrammarCursor<'_>>() as u64,
                )?;
                grammar.advance();
            }
            if !grammar.is_valid() {
                return Ok((false, None));
            }
            payload_write::<surgeist_html::bounded::RangeDecimalParts>(c, 1)?;
            grammar.range_decimal_parts().map_err(|_| integrity(c))?
        };
        let source_len = admitted_input_string(&self.payloads, c, source)?.len();
        let stage = self.range_stage(c)?;
        stage.scratch_empty(c)?;
        let headers = stage.headers(c)?;
        payload_write::<FiniteDecimalIngressCursor>(c, 1)?;
        let cursor = FiniteDecimalIngressCursor::new(parts, source_len, headers)
            .map_err(|error| numeric_failure(c, error))?;
        payload_write::<Option<Machine>>(c, 1)?;
        stage.source = source;
        stage.machine = Some(Machine::ingress(cursor));
        self.execute_range(c)?;
        let stage = self.range_stage(c)?;
        let Some(cursor) = stage
            .machine
            .as_ref()
            .and_then(|machine| machine.ingress.as_ref())
        else {
            return Err(integrity(c));
        };
        match cursor.result().ok_or_else(|| integrity(c))? {
            Ok(bits) => Ok((true, Some(bits))),
            Err(RangeNumericError::Overflow) => Ok((true, None)),
            Err(_) => Err(integrity(c)),
        }
    }
    fn range_attribute_slot(
        &self,
        c: &mut ParseControl,
        node: usize,
        name: &str,
    ) -> Result<Option<usize>, SessionError> {
        c.charge(WorkCategory::RecordVisit, 1)?;
        let HtmlNodeKind::Element(element) = &self
            .construction
            .node_at(node)
            .expect("retained construction position")
            .kind()
        else {
            return Err(SessionError::WrongToken);
        };
        for (position, attribute) in element.attributes().as_slice().iter().enumerate() {
            c.charge(WorkCategory::RecordVisit, 1)?;
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
        let (_, value) = self.range_parse(c, slot, FloatingPointParseMode::Recovery)?;
        match value {
            Some(bits) => Ok((
                HtmlInputNumericContent::Value(
                    crate::HtmlFloatingPointNumber::try_from(f64::from_bits(bits))
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
        if state.flags.input_type() != crate::HtmlInputType::Range {
            return Err(SessionError::InputMetadataUnavailable);
        }
        self.range_payload_edge(c, source)?;
        self.releasable_string(c, previous, false)?;
        if self.effects.range.is_some() {
            return Err(integrity(c));
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
            Err(_) => Err(integrity(c)),
        }
    }
    fn range_policy(
        &mut self,
        c: &mut ParseControl,
        node: usize,
        slot: usize,
        source: usize,
    ) -> Result<(), SessionError> {
        use crate::bounded::{
            HtmlInputRangeNumber, HtmlInputRangeReplacement as B,
            HtmlInputRangeSanitizationAction as A, HtmlInputRangeSanitizationCursor,
            HtmlInputRangeSanitizationReply as R, HtmlInputRangeValue,
        };
        let source_edge = self.range_payload_edge(c, source)?;
        let current = self.input_raw_slot(c, slot)?;
        let state = self.read_input_state(c, slot)?;
        c.charge(WorkCategory::RecordVisit, 1)?;
        let RetainedPayload::Input { complete, .. } = &self.payloads.values[slot] else {
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
        let (valid, value) = self.range_parse(c, source, FloatingPointParseMode::Author)?;
        let (min, min_operand) = self.range_content(c, node, "min")?;
        let (max, max_operand) = self.range_content(c, node, "max")?;
        let mut operands = [
            min_operand,
            max_operand,
            RangeOperand::NotConsulted,
            RangeOperand::NotConsulted,
        ];
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
            c.charge(WorkCategory::RecordVisit, 1)?;
            c.charge(WorkCategory::StateStep, 1)?;
            c.charge(
                WorkCategory::InspectByte,
                mem::size_of::<HtmlInputRangeSanitizationCursor>() as u64,
            )?;
            payload_write::<A>(c, 1)?;
            let reply = match policy.action() {
                A::Midpoint { minimum, maximum } => {
                    payload_write::<RangeMidpointCursor>(c, 1)?;
                    let cursor = RangeMidpointCursor::new(
                        minimum.value().to_bits(),
                        maximum.value().to_bits(),
                    )
                    .map_err(|_| integrity(c))?;
                    payload_write::<Option<Machine>>(c, 1)?;
                    self.range_stage(c)?.machine = Some(Machine::midpoint(cursor));
                    self.execute_range(c)?;
                    let Some(cursor) = self
                        .range_stage(c)?
                        .machine
                        .as_ref()
                        .and_then(|machine| machine.midpoint.as_ref())
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
                        crate::HtmlInputNumericBoundOrigin,
                        crate::HtmlInputNumericBoundOrigin,
                    )>(c, 1)?;
                    let (minimum, maximum) = policy.bound_origins();
                    let (step_origin, base_origin) = if step {
                        payload_write::<Option<crate::HtmlInputNumericConstraints>>(c, 1)?;
                        let constraints = policy.constraints().ok_or_else(|| integrity(c))?;
                        (
                            Some(constraints.step_origin()),
                            Some(constraints.step_base_origin()),
                        )
                    } else {
                        (None, None)
                    };
                    let branch = match branch {
                        B::InvalidDefaultMinimum => RangeBranch::InvalidDefaultMinimum,
                        B::InvalidDefaultMidpoint => RangeBranch::InvalidDefaultMidpoint,
                        B::Underflow => RangeBranch::Underflow,
                        B::Overflow => RangeBranch::Overflow,
                        B::StepCorrection => RangeBranch::StepCorrection,
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
                            c.charge(WorkCategory::RecordVisit, 1)?;
                            let mut any = raw.len() == keyword_len;
                            if any {
                                for index in 0..keyword_len {
                                    c.charge(WorkCategory::InspectByte, 2)?;
                                    c.charge(WorkCategory::StateStep, 1)?;
                                    if !HtmlInputNumericStep::any_keyword_unit_matches(
                                        index,
                                        raw.code_units()[index],
                                    ) {
                                        any = false;
                                        break;
                                    }
                                }
                            }
                            any
                        };
                        if any {
                            operands[2] = RangeOperand::Any(edge);
                            HtmlInputNumericStep::Any
                        } else {
                            let (_, result) =
                                self.range_parse(c, step_slot, FloatingPointParseMode::Recovery)?;
                            match result {
                                Some(bits) => {
                                    operands[2] = RangeOperand::Finite(edge);
                                    HtmlInputNumericStep::Value(
                                        crate::HtmlFloatingPointNumber::try_from(f64::from_bits(
                                            bits,
                                        ))
                                        .map_err(|_| integrity(c))?,
                                    )
                                }
                                None => {
                                    operands[2] = RangeOperand::ConversionFailed(edge);
                                    HtmlInputNumericStep::Failed
                                }
                            }
                        }
                    } else {
                        operands[2] = RangeOperand::Absent;
                        HtmlInputNumericStep::Absent
                    };
                    R::Step(step)
                }
                A::ReadValueContent => {
                    let (content, operand) = self.range_content(c, node, "value")?;
                    operands[3] = operand;
                    R::ValueContent(content)
                }
                A::Lattice {
                    current,
                    minimum,
                    maximum,
                    base,
                    step,
                } => {
                    payload_write::<RangeLatticeCursor>(c, 1)?;
                    let cursor = RangeLatticeCursor::new(
                        current.value().to_bits(),
                        minimum.value().to_bits(),
                        maximum.map(|value| value.value().to_bits()),
                        base.value().to_bits(),
                        step.value().to_bits(),
                    )
                    .map_err(|_| integrity(c))?;
                    payload_write::<Option<Machine>>(c, 1)?;
                    self.range_stage(c)?.machine = Some(Machine::lattice(cursor));
                    self.execute_range(c)?;
                    let Some(cursor) = self
                        .range_stage(c)?
                        .machine
                        .as_ref()
                        .and_then(|machine| machine.lattice.as_ref())
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
        payload_write::<BestFiniteNumberCursor>(c, 1)?;
        let cursor = BestFiniteNumberCursor::new(f64::from_bits(bits), headers)
            .map_err(|error| numeric_failure(c, error))?;
        payload_write::<Option<Machine>>(c, 1)?;
        stage.machine = Some(Machine::best(cursor));
        self.execute_range(c)?;
        let units = self
            .range_stage(c)?
            .output
            .as_ref()
            .ok_or_else(|| integrity(c))?
            .len();
        let Some(cursor) = self
            .range_stage(c)?
            .machine
            .as_ref()
            .and_then(|machine| machine.best.as_ref())
        else {
            return Err(integrity(c));
        };
        if cursor.result() != Some(units) {
            return Err(integrity(c));
        }
        payload_write::<Option<OriginGroupRef>>(c, 1)?;
        let group = self.prepare_range_group(c, units, computation)?;
        self.range_stage(c)?.generated = Some(group);
        payload_write::<Option<Utf16Buffer>>(c, 1)?;
        payload_write::<Option<OriginGroupRef>>(c, 1)?;
        let stage = self.range_stage(c)?;
        let group = stage.generated.take().ok_or_else(|| integrity(c))?;
        let output = stage.output.take().ok_or_else(|| integrity(c))?;
        let value = output.freeze(c)?;
        let (native, next) = self.register_group_string_owned(c, value, Some(group))?;
        drop(native);
        payload_write::<usize>(c, 1)?;
        payload_write::<RetainedPayload>(c, 2)?;
        self.retire_field_group(c, previous)?;
        let RetainedPayload::Input { raw, .. } = &mut self.payloads.values[slot] else {
            return Err(integrity(c));
        };
        *raw = next;
        if dispose_previous {
            self.dispose_aggregate_string(c, previous)?;
        }
        Ok(())
    }
}
