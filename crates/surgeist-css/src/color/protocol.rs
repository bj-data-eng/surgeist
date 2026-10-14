//! The exclusive anchor and recoverable finite lexical actions.
//!
//! CSS owns syntax/state; the consumer admits each actual action and owns all
//! buffers. Guards never release capacity, run abandoned work, or refund on Drop.

use super::grammar::GrammarRecord;
use crate::numeric::bounded_decimal::DecimalDigit;
use crate::tokenization::bounded::{InputCursor, InputKind, Request as InputRequest, Scalar};
use crate::tokenization::color_tokens::{Record, Request as TokenRequest, Token, TokenCursor};

/// A nonzero-sized allocation-free anchor, exclusively lent to one operation.
pub struct CssColorOperationOwner {
    marker: u8,
    phase: std::cell::Cell<usize>,
}
impl Default for CssColorOperationOwner {
    fn default() -> Self {
        Self::new()
    }
}
impl CssColorOperationOwner {
    pub const fn new() -> Self {
        Self {
            marker: 1,
            phase: std::cell::Cell::new(0),
        }
    }
    pub(crate) fn begin(&mut self) -> Result<usize, CssColorProtocolError> {
        self.advance_from(self.phase.get())
    }
    pub(super) fn prepare_advance(&self, expected: usize) -> Result<usize, CssColorProtocolError> {
        if self.phase.get() != expected {
            return Err(CssColorProtocolError::ForeignSlot);
        }
        expected
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)
    }
    pub(super) fn commit_phase(&self, expected: usize, next: usize) {
        assert_eq!(
            self.phase.get(),
            expected,
            "phase was preflighted before numerical owner construction"
        );
        assert_eq!(expected.checked_add(1), Some(next));
        self.phase.set(next);
    }
    pub(super) fn advance_from(&self, expected: usize) -> Result<usize, CssColorProtocolError> {
        let next = self.prepare_advance(expected)?;
        self.commit_phase(expected, next);
        Ok(next)
    }
    pub(super) fn current_phase(&self) -> usize {
        self.phase.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorSlotKind {
    Scalar,
    NameUnit,
    NumberDigit,
    Token,
    DecimalDigit,
    ScaledDigit,
    ProfileChannel,
    Grammar,
    NumericLeaf,
    Expression,
    Evaluation,
    Resolution,
    MixTask,
}

/// Distinct flat storage banks, qualified by the same exclusive operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorBank {
    Algorithm,
    Arithmetic,
    Evaluation,
}

#[derive(Clone, Copy, Debug)]
pub(super) enum Data {
    Scalar(Scalar),
    NameUnit(char),
    NumberDigit {
        exponent: bool,
        value: u8,
    },
    Token(Token),
    DecimalDigit {
        token: usize,
        digit: DecimalDigit,
    },
    ScaledDigit {
        token: usize,
        value: u8,
    },
    ProfileChannel {
        resource: super::facts::CssColorFactIdentity,
        ordinal: usize,
        value: Option<f64>,
    },
    Grammar(GrammarRecord),
    NumericLeaf(super::expression::NumericLeaf),
    Expression(super::expression::ExpressionRecord),
    Evaluation(super::expression::Evaluated),
    Resolution(super::resolution::Frame),
    MixTask(super::resolution::MixTask),
}
impl Data {
    pub(super) const fn bank(self) -> CssColorBank {
        match self {
            Self::Expression(_) => CssColorBank::Arithmetic,
            Self::Evaluation(_) => CssColorBank::Evaluation,
            _ => CssColorBank::Algorithm,
        }
    }
    const fn kind(self) -> CssColorSlotKind {
        match self {
            Self::Scalar(_) => CssColorSlotKind::Scalar,
            Self::NameUnit(_) => CssColorSlotKind::NameUnit,
            Self::NumberDigit { .. } => CssColorSlotKind::NumberDigit,
            Self::Token(_) => CssColorSlotKind::Token,
            Self::DecimalDigit { .. } => CssColorSlotKind::DecimalDigit,
            Self::ScaledDigit { .. } => CssColorSlotKind::ScaledDigit,
            Self::ProfileChannel { .. } => CssColorSlotKind::ProfileChannel,
            Self::Grammar(_) => CssColorSlotKind::Grammar,
            Self::NumericLeaf(_) => CssColorSlotKind::NumericLeaf,
            Self::Expression(_) => CssColorSlotKind::Expression,
            Self::Evaluation(_) => CssColorSlotKind::Evaluation,
            Self::Resolution(_) => CssColorSlotKind::Resolution,
            Self::MixTask(_) => CssColorSlotKind::MixTask,
        }
    }
}

/// One opaque caller-owned record qualified by the exact anchor and phase.
pub struct CssColorSlot<'owner> {
    pub(super) index: usize,
    pub(super) owner: &'owner CssColorOperationOwner,
    pub(super) phase: usize,
    bank: CssColorBank,
    pub(super) data: Data,
}
impl<'owner> CssColorSlot<'owner> {
    pub(super) fn new(
        owner: &'owner CssColorOperationOwner,
        phase: usize,
        index: usize,
        data: Data,
    ) -> Self {
        Self {
            index,
            owner,
            phase,
            bank: data.bank(),
            data,
        }
    }
    pub const fn bank(&self) -> CssColorBank {
        self.bank
    }
    pub const fn kind(&self) -> CssColorSlotKind {
        self.data.kind()
    }
    pub(super) fn qualify(
        &self,
        owner: &CssColorOperationOwner,
        phase: usize,
        index: usize,
        bank: CssColorBank,
    ) -> Result<(), CssColorProtocolError> {
        if !std::ptr::eq(self.owner, owner) || self.phase != phase || owner.current_phase() != phase
        {
            return Err(CssColorProtocolError::ForeignSlot);
        }
        if self.bank != bank || self.bank != self.data.bank() {
            return Err(CssColorProtocolError::WrongSlot);
        }
        if index != self.index {
            return Err(CssColorProtocolError::WrongIndex);
        }
        Ok(())
    }
}

/// A qualified fixed observation, never a borrowed backing slice.
#[derive(Clone, Copy, Debug)]
pub(crate) enum LexicalRecord {
    NameUnit(char),
    NumberDigit { exponent: bool, value: u8 },
    Token(Token),
}

#[derive(Clone, Copy)]
pub(crate) struct LexicalTable<'owner> {
    pub(super) owner: &'owner CssColorOperationOwner,
    pub(super) phase: usize,
    length: usize,
    scalars: usize,
}
impl<'owner> LexicalTable<'owner> {
    /// Actual ordinary component events use the same qualified lexical records.
    /// The adapter owns projection/admission; every later record read still
    /// qualifies its exact bank, ordinal, anchor and phase.
    pub(super) fn component_events(
        owner: &'owner CssColorOperationOwner,
        phase: usize,
        length: usize,
    ) -> Result<Self, CssColorProtocolError> {
        if phase == 0 || owner.current_phase() != phase {
            return Err(CssColorProtocolError::ForeignSlot);
        }
        Ok(Self {
            owner,
            phase,
            length,
            scalars: 0,
        })
    }
    pub(crate) const fn length(&self) -> usize {
        self.length
    }
    pub(crate) const fn first_token_record(&self) -> usize {
        self.scalars
    }
    pub(crate) fn observe(
        &self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<LexicalRecord, CssColorProtocolError> {
        slot.qualify(self.owner, self.phase, index, CssColorBank::Algorithm)?;
        if index >= self.length {
            return Err(CssColorProtocolError::WrongIndex);
        }
        Ok(match slot.data {
            Data::Scalar(_) => return Err(CssColorProtocolError::WrongSlot),
            Data::NameUnit(value) => LexicalRecord::NameUnit(value),
            Data::NumberDigit { exponent, value } => LexicalRecord::NumberDigit { exponent, value },
            Data::Token(value) => LexicalRecord::Token(value),
            Data::DecimalDigit { .. }
            | Data::ScaledDigit { .. }
            | Data::ProfileChannel { .. }
            | Data::Grammar(_)
            | Data::NumericLeaf(_)
            | Data::Expression(_)
            | Data::Evaluation(_)
            | Data::Resolution(_)
            | Data::MixTask(_) => return Err(CssColorProtocolError::WrongSlot),
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorProtocolError {
    WrongAction,
    WrongIndex,
    WrongSlot,
    ForeignSlot,
    Overflow,
    Input,
    Terminal,
    StorageNotEmpty,
    EmptySlot,
    NumericalProtocol(surgeist_color::ProtocolError),
}

impl std::fmt::Display for CssColorProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::NumericalProtocol(error) = self {
            return std::fmt::Display::fmt(error, f);
        }
        f.write_str(match self {
            Self::WrongAction => "color action does not match the pending request",
            Self::WrongIndex => "color record index does not match the requested ordinal",
            Self::WrongSlot => "color record kind or bank does not match the request",
            Self::ForeignSlot => "color record belongs to another operation or phase",
            Self::Overflow => "color operation extent cannot be represented",
            Self::Input => "color input observation violates its declared encoding or record",
            Self::Terminal => "color operation has no completed result",
            Self::StorageNotEmpty => "color scratch banks have not been retired",
            Self::EmptySlot => "color transition has no caller-owned checkpoint",
            Self::NumericalProtocol(_) => unreachable!("handled numerical protocol source"),
        })
    }
}
impl std::error::Error for CssColorProtocolError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::NumericalProtocol(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ActionDescription {
    ReadInput {
        offset: usize,
    },
    ComputeInput,
    AppendSlot {
        index: usize,
        kind: CssColorSlotKind,
    },
    ReadSlot {
        index: usize,
        kind: CssColorSlotKind,
    },
    ComputeToken,
    Done,
    Stopped,
}

#[expect(
    clippy::large_enum_variant,
    reason = "Input and token cursors are mutually exclusive inline stages; caller-accounted receiver layout avoids an unadmitted heap allocation"
)]
enum Stage {
    Input(InputCursor),
    Tokens(TokenCursor),
    Done,
    Stopped,
}

/// Fixed-register lexical phase. It has no output buffer capability.
pub(crate) struct LexicalOperation<'owner> {
    owner: &'owner CssColorOperationOwner,
    phase: usize,
    stage: Stage,
    scalar_count: usize,
    slot_count: usize,
}
impl<'owner> LexicalOperation<'owner> {
    pub(crate) fn start(
        owner: &'owner mut CssColorOperationOwner,
        kind: InputKind,
        length: usize,
    ) -> Result<Self, CssColorProtocolError> {
        // Reading the fixed anchor/header is part of the caller's start admission.
        let _ = owner.marker;
        let phase = owner.begin()?;
        Ok(Self {
            owner,
            phase,
            stage: Stage::Input(InputCursor::new(kind, length)),
            scalar_count: 0,
            slot_count: 0,
        })
    }

    pub(super) fn description(&self) -> ActionDescription {
        match &self.stage {
            Stage::Input(cursor) => match cursor.request() {
                InputRequest::Read { index } => ActionDescription::ReadInput { offset: index },
                InputRequest::HeldUnit | InputRequest::End => ActionDescription::ComputeInput,
                InputRequest::Emit(_) => ActionDescription::AppendSlot {
                    index: self.slot_count,
                    kind: CssColorSlotKind::Scalar,
                },
            },
            Stage::Tokens(cursor) => match cursor.request() {
                TokenRequest::ReadScalar { index } => ActionDescription::ReadSlot {
                    index,
                    kind: CssColorSlotKind::Scalar,
                },
                TokenRequest::Compute | TokenRequest::Done => ActionDescription::ComputeToken,
                TokenRequest::Append { index, record } => ActionDescription::AppendSlot {
                    index,
                    kind: match record {
                        Record::NameUnit(_) => CssColorSlotKind::NameUnit,
                        Record::Digit { .. } => CssColorSlotKind::NumberDigit,
                        Record::Token(_) => CssColorSlotKind::Token,
                    },
                },
            },
            Stage::Done => ActionDescription::Done,
            Stage::Stopped => ActionDescription::Stopped,
        }
    }

    /// The caller admits fixed planning/header work before obtaining this guard.
    pub(crate) fn next_action(&mut self) -> ColorAction<'_, 'owner> {
        let description = self.description();
        ColorAction {
            operation: self,
            description,
            issued: false,
        }
    }

    /// Stop never consumes caller buffers or relabels them empty.
    pub(crate) fn abandon(&mut self) {
        self.stage = Stage::Stopped;
    }

    #[cfg(test)]
    pub(crate) fn completed_length(&self) -> Result<usize, CssColorProtocolError> {
        if matches!(self.stage, Stage::Done) {
            Ok(self.slot_count)
        } else {
            Err(CssColorProtocolError::Terminal)
        }
    }

    pub(crate) fn finish(self) -> Result<LexicalTable<'owner>, CssColorProtocolError> {
        if !matches!(self.stage, Stage::Done) {
            return Err(CssColorProtocolError::Terminal);
        }
        Ok(LexicalTable {
            owner: self.owner,
            phase: self.phase,
            length: self.slot_count,
            scalars: self.scalar_count,
        })
    }

    fn qualify(
        &self,
        slot: &CssColorSlot<'_>,
        kind: CssColorSlotKind,
    ) -> Result<(), CssColorProtocolError> {
        slot.qualify(self.owner, self.phase, slot.index, CssColorBank::Algorithm)?;
        if slot.kind() != kind {
            return Err(CssColorProtocolError::WrongSlot);
        }
        Ok(())
    }
}

/// Only one action can borrow the operation. A dropped guard leaves it pending.
pub(crate) struct ColorAction<'step, 'owner> {
    operation: &'step mut LexicalOperation<'owner>,
    description: ActionDescription,
    issued: bool,
}
impl<'owner> ColorAction<'_, 'owner> {
    #[cfg(test)]
    pub(crate) const fn description(&self) -> ActionDescription {
        self.description
    }

    pub(crate) fn read_input(self, offset: usize, unit: u16) -> Result<(), CssColorProtocolError> {
        if self.description != (ActionDescription::ReadInput { offset }) {
            return Err(CssColorProtocolError::WrongAction);
        }
        let Stage::Input(cursor) = &mut self.operation.stage else {
            return Err(CssColorProtocolError::WrongAction);
        };
        cursor.read(unit).map_err(|_| CssColorProtocolError::Input)
    }

    pub(crate) fn read_slot(
        self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), CssColorProtocolError> {
        if self.description
            != (ActionDescription::ReadSlot {
                index,
                kind: CssColorSlotKind::Scalar,
            })
        {
            return Err(CssColorProtocolError::WrongAction);
        }
        self.operation.qualify(slot, CssColorSlotKind::Scalar)?;
        if slot.index != index {
            return Err(CssColorProtocolError::WrongIndex);
        }
        let Data::Scalar(scalar) = slot.data else {
            return Err(CssColorProtocolError::WrongSlot);
        };
        let Stage::Tokens(cursor) = &mut self.operation.stage else {
            return Err(CssColorProtocolError::WrongAction);
        };
        cursor
            .read_scalar(index, scalar)
            .map_err(|_| CssColorProtocolError::Input)
    }

    /// Generates one fixed record, without advancing its pending append.
    pub(crate) fn append_record(&mut self) -> Result<CssColorSlot<'owner>, CssColorProtocolError> {
        if self.issued {
            return Err(CssColorProtocolError::WrongAction);
        }
        if !matches!(self.description, ActionDescription::AppendSlot { .. }) {
            return Err(CssColorProtocolError::WrongAction);
        }
        self.operation
            .slot_count
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)?;
        let data = match &self.operation.stage {
            Stage::Input(cursor) => {
                let InputRequest::Emit(scalar) = cursor.request() else {
                    return Err(CssColorProtocolError::WrongAction);
                };
                self.operation
                    .scalar_count
                    .checked_add(1)
                    .ok_or(CssColorProtocolError::Overflow)?;
                Data::Scalar(scalar)
            }
            Stage::Tokens(cursor) => {
                let TokenRequest::Append { record, .. } = cursor.request() else {
                    return Err(CssColorProtocolError::WrongAction);
                };
                match record {
                    Record::NameUnit(value) => Data::NameUnit(value),
                    Record::Digit { exponent, value } => Data::NumberDigit { exponent, value },
                    Record::Token(value) => Data::Token(value),
                }
            }
            Stage::Done | Stage::Stopped => return Err(CssColorProtocolError::Terminal),
        };
        self.issued = true;
        Ok(CssColorSlot::new(
            self.operation.owner,
            self.operation.phase,
            self.operation.slot_count,
            data,
        ))
    }

    /// Acknowledges the actual recovered push after storage and logical admission.
    pub(crate) fn appended(self, index: usize) -> Result<(), CssColorProtocolError> {
        if !self.issued {
            return Err(CssColorProtocolError::WrongAction);
        }
        if !matches!(self.description, ActionDescription::AppendSlot { index: expected, .. } if expected == index)
        {
            return Err(CssColorProtocolError::WrongIndex);
        }
        let slots = self
            .operation
            .slot_count
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)?;
        match &mut self.operation.stage {
            Stage::Input(cursor) => {
                let scalars = self
                    .operation
                    .scalar_count
                    .checked_add(1)
                    .ok_or(CssColorProtocolError::Overflow)?;
                cursor
                    .emitted()
                    .map_err(|_| CssColorProtocolError::WrongAction)?;
                self.operation.scalar_count = scalars;
            }
            Stage::Tokens(cursor) => cursor
                .appended(index)
                .map_err(|_| CssColorProtocolError::WrongAction)?,
            Stage::Done | Stage::Stopped => return Err(CssColorProtocolError::Terminal),
        }
        self.operation.slot_count = slots;
        Ok(())
    }

    pub(crate) fn compute(self) -> Result<(), CssColorProtocolError> {
        match (&mut self.operation.stage, self.description) {
            (Stage::Input(cursor), ActionDescription::ComputeInput) => match cursor.request() {
                InputRequest::HeldUnit => cursor
                    .process_held()
                    .map_err(|_| CssColorProtocolError::Input)?,
                InputRequest::End => {
                    if cursor.end().map_err(|_| CssColorProtocolError::Input)? {
                        self.operation.stage = Stage::Tokens(TokenCursor::new(
                            self.operation.scalar_count,
                            self.operation.slot_count,
                        ));
                    }
                }
                _ => return Err(CssColorProtocolError::WrongAction),
            },
            (Stage::Tokens(cursor), ActionDescription::ComputeToken) => {
                if cursor.request() == TokenRequest::Done {
                    self.operation.stage = Stage::Done;
                } else {
                    cursor.compute().map_err(|_| CssColorProtocolError::Input)?;
                }
            }
            _ => return Err(CssColorProtocolError::WrongAction),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenization::color_tokens::{NameKind, TokenKind};

    #[test]
    fn a_slot_bank_is_part_of_every_qualified_observation() {
        let mut owner = CssColorOperationOwner::new();
        let mut operation = LexicalOperation::start(&mut owner, InputKind::Utf8, 1).unwrap();
        let mut slots = drive(&mut operation, &[u16::from(b'r')]);
        let table = operation.finish().unwrap();
        let index = slots
            .iter()
            .position(|slot| slot.kind() == CssColorSlotKind::Token)
            .unwrap();
        assert!(table.observe(index, &slots[index]).is_ok());
        slots[index].bank = CssColorBank::Arithmetic;
        assert_eq!(
            table.observe(index, &slots[index]).unwrap_err(),
            CssColorProtocolError::WrongSlot
        );
        slots[index].bank = CssColorBank::Algorithm;
        assert!(table.observe(index, &slots[index]).is_ok());
    }

    #[test]
    fn a_retired_phase_slot_cannot_supply_a_new_phase_even_with_the_same_anchor() {
        let mut owner = CssColorOperationOwner::new();
        let mut lexical = LexicalOperation::start(&mut owner, InputKind::Utf8, 1).unwrap();
        let slots = drive(&mut lexical, &[u16::from(b'r')]);
        let table = lexical.finish().unwrap();
        let index = slots
            .iter()
            .position(|slot| slot.kind() == CssColorSlotKind::Token)
            .unwrap();
        assert!(table.observe(index, &slots[index]).is_ok());
        let next = table.owner.advance_from(table.phase).unwrap();
        assert_eq!(next, table.phase + 1);
        assert_eq!(
            table.observe(index, &slots[index]).unwrap_err(),
            CssColorProtocolError::ForeignSlot
        );
        assert_eq!(
            table.owner.advance_from(table.phase),
            Err(CssColorProtocolError::ForeignSlot)
        );
        // Refusal retains the actual caller slot. Its eventual physical release
        // is the caller owner's operation, not a CSS refund or relabeling.
        assert_eq!(slots[index].kind(), CssColorSlotKind::Token);
    }
    fn drive<'a>(operation: &mut LexicalOperation<'a>, input: &[u16]) -> Vec<CssColorSlot<'a>> {
        let mut scratch = Vec::new();
        loop {
            let mut action = operation.next_action();
            match action.description() {
                ActionDescription::ReadInput { offset } => {
                    action.read_input(offset, input[offset]).unwrap()
                }
                ActionDescription::ReadSlot { index, .. } => {
                    action.read_slot(index, &scratch[index]).unwrap()
                }
                ActionDescription::AppendSlot { index, .. } => {
                    let record = action.append_record().unwrap();
                    assert_eq!(index, scratch.len());
                    scratch.push(record);
                    action.appended(index).unwrap();
                }
                ActionDescription::ComputeInput | ActionDescription::ComputeToken => {
                    action.compute().unwrap()
                }
                ActionDescription::Done => {
                    return scratch;
                }
                ActionDescription::Stopped => panic!("stopped"),
            }
        }
    }

    #[test]
    fn actual_lexical_machine_retains_original_units_and_decoded_recipe() {
        let input: Vec<_> = "\\72 gb(\r\n#f00)".bytes().map(u16::from).collect();
        let mut owner = CssColorOperationOwner::new();
        let mut operation =
            LexicalOperation::start(&mut owner, InputKind::Utf8, input.len()).unwrap();
        let scratch = drive(&mut operation, &input);
        assert_eq!(operation.completed_length(), Ok(scratch.len()));
        let tokens: Vec<_> = scratch
            .iter()
            .filter_map(|slot| match slot.data {
                Data::Token(token) => Some(token),
                _ => None,
            })
            .collect();
        let TokenKind::Name {
            kind: NameKind::Function,
            units,
        } = tokens[0].kind
        else {
            panic!("function")
        };
        let name: String = scratch[units.start..units.end]
            .iter()
            .map(|slot| match slot.data {
                Data::NameUnit(value) => value,
                _ => panic!("unit"),
            })
            .collect();
        assert_eq!(name, "rgb");
        assert_eq!(tokens[1].source.start, 7);
        assert_eq!(tokens[1].source.end, 9);
        assert_eq!(
            scratch
                .iter()
                .filter(|slot| slot.kind() == CssColorSlotKind::Scalar)
                .count(),
            input.len() - 1
        );
    }

    #[test]
    fn wrong_foreign_slot_and_refused_append_keep_real_pending_action() {
        let mut first_owner = CssColorOperationOwner::new();
        let mut second_owner = CssColorOperationOwner::new();
        let mut first = LexicalOperation::start(&mut first_owner, InputKind::Utf16, 1).unwrap();
        let first_slots = drive(&mut first, &[65]);
        let mut second = LexicalOperation::start(&mut second_owner, InputKind::Utf16, 1).unwrap();
        second.next_action().read_input(0, 66).unwrap();
        let before = second.next_action().description();
        {
            let mut action = second.next_action();
            let recovered = action.append_record().unwrap();
            assert_eq!(recovered.kind(), CssColorSlotKind::Scalar);
        }
        assert_eq!(second.next_action().description(), before);
        let mut action = second.next_action();
        let local = action.append_record().unwrap();
        action.appended(0).unwrap();
        second.next_action().compute().unwrap();
        let read = second.next_action().description();
        assert_eq!(
            second.next_action().read_slot(0, &first_slots[0]),
            Err(CssColorProtocolError::ForeignSlot)
        );
        assert_eq!(second.next_action().description(), read);
        second.next_action().read_slot(0, &local).unwrap();
        second.abandon();
        assert_eq!(
            second.completed_length(),
            Err(CssColorProtocolError::Terminal)
        );
        assert_eq!(
            second.next_action().description(),
            ActionDescription::Stopped
        );
        // Actual caller slots remain available; abandoning the cursor refunds nothing.
        assert_eq!(local.kind(), CssColorSlotKind::Scalar);
        assert!(!first_slots.is_empty());
    }
}
