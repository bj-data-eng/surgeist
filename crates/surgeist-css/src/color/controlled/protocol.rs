//! The exclusive anchor and recoverable finite lexical actions.
//!
//! CSS owns syntax/state; the consumer admits each actual action and owns all
//! buffers. Guards never release capacity, run abandoned work, or refund on Drop.

use crate::color::grammar::GrammarRecord;
use crate::numeric::bounded_decimal::DecimalDigit;
use crate::tokenization::bounded::{
    Error as InputError, InputCursor, InputKind, Request as InputRequest, Scalar,
};
use crate::tokenization::tokenizer::{Record, Request as TokenRequest, Token, TokenCursor};

/// A nonzero-sized allocation-free anchor, exclusively lent to one parse.
///
/// Every caller-owned [`CssColorSlot`] borrows it, so the owner begins another
/// parse only after the earlier parse's records are dropped.
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
    /// Creates an anchor with no parse started.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            marker: 1,
            phase: std::cell::Cell::new(0),
        }
    }
    /// Actual compiled anchor layout, admitted before construction.
    #[must_use]
    pub const fn layout_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    pub(crate) fn begin(&mut self) -> Result<usize, ColorProtocolError> {
        self.advance_from(self.phase.get())
    }
    pub(crate) fn prepare_advance(&self, expected: usize) -> Result<usize, ColorProtocolError> {
        if self.phase.get() != expected {
            return Err(ColorProtocolError::ForeignSlot);
        }
        expected.checked_add(1).ok_or(ColorProtocolError::Overflow)
    }
    pub(crate) fn commit_phase(&self, expected: usize, next: usize) {
        assert_eq!(
            self.phase.get(),
            expected,
            "phase was preflighted before numerical owner construction"
        );
        assert_eq!(expected.checked_add(1), Some(next));
        self.phase.set(next);
    }
    pub(in crate::color) fn advance_from(
        &self,
        expected: usize,
    ) -> Result<usize, ColorProtocolError> {
        let next = self.prepare_advance(expected)?;
        self.commit_phase(expected, next);
        Ok(next)
    }
    pub(crate) fn current_phase(&self) -> usize {
        self.phase.get()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CssColorSlotKind {
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
    AuthoredNumber,
    AuthoredName,
    AuthoredSite,
    AuthoredTraversal,
    AuthoredNumericFrame,
    AuthoredColorFrame,
    AuthoredStagedComponent,
}

/// A range in the actual algorithm record bank, never an original input span.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct RecordRange {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct AuthoredNumber<'css> {
    pub(crate) numeric: crate::component::CssNumericTokenRef<'css>,
    pub(crate) layout: crate::tokenization::number::Layout,
    pub(crate) digits: RecordRange,
    pub(crate) unit: Option<crate::numeric::CssNumericUnit>,
    pub(crate) percentage: bool,
    pub(crate) site: usize,
}
#[derive(Clone, Copy, Debug)]
pub(crate) struct AuthoredName<'css> {
    pub(crate) name: &'css str,
    pub(crate) units: RecordRange,
    pub(crate) site: usize,
}

/// Distinct flat storage banks, qualified by the same exclusive operation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CssColorBank {
    /// The bank of every record other than the arithmetic and evaluation ones, such as lexical, decimal, grammar, syntax and color-resolution records.
    Algorithm,
    /// The bank of arithmetic records: the calculation expression records that component evaluation reads, and the authored-color traversal records.
    Arithmetic,
    /// The bank of evaluated calculation results.
    Evaluation,
}

#[expect(
    clippy::large_enum_variant,
    reason = "Real four-channel plus alpha staging and suspended frames stay inline in the three caller-accounted CssColorSlot banks; boxing introduces unadmitted heap ownership"
)]
#[derive(Clone, Copy, Debug)]
pub(crate) enum Data<'css> {
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
        resource: crate::color::CssColorFactIdentity,
        ordinal: usize,
        value: Option<f64>,
    },
    Grammar(GrammarRecord),
    NumericLeaf(crate::color::expression::NumericLeaf),
    Expression(crate::color::expression::ExpressionRecord),
    Evaluation(crate::color::expression::Evaluated),
    Resolution(crate::color::resolution::Frame),
    MixTask(crate::color::resolution::MixTask),
    AuthoredNumber(AuthoredNumber<'css>),
    AuthoredName(AuthoredName<'css>),
    AuthoredSite(crate::color::authored_resolution::site::SiteRecord<'css>),
    AuthoredTraversal(crate::color::authored_resolution::site::TraversalRecord),
    AuthoredNumericFrame(crate::color::authored_resolution::numeric::Frame<'css>),
    AuthoredColorFrame(crate::color::authored_resolution::lowering::Frame<'css>),
    AuthoredStagedComponent(crate::color::authored_resolution::lowering::StagedComponent),
}
impl Data<'_> {
    pub(in crate::color) const fn bank(self) -> CssColorBank {
        match self {
            Self::Expression(_) | Self::AuthoredTraversal(_) => CssColorBank::Arithmetic,
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
            Self::AuthoredNumber(_) => CssColorSlotKind::AuthoredNumber,
            Self::AuthoredName(_) => CssColorSlotKind::AuthoredName,
            Self::AuthoredSite(_) => CssColorSlotKind::AuthoredSite,
            Self::AuthoredTraversal(_) => CssColorSlotKind::AuthoredTraversal,
            Self::AuthoredNumericFrame(_) => CssColorSlotKind::AuthoredNumericFrame,
            Self::AuthoredColorFrame(_) => CssColorSlotKind::AuthoredColorFrame,
            Self::AuthoredStagedComponent(_) => CssColorSlotKind::AuthoredStagedComponent,
        }
    }
}

/// One opaque caller-owned record qualified by the exact anchor and phase.
///
/// Only a CSS write action mints a record. The caller stores it in the bank
/// that action names and lends it back when the parse reads that position. A
/// record of another parse, bank or index is a defect in the driving code
/// and panics, as slice indexing does.
pub struct CssColorSlot<'owner> {
    pub(crate) index: usize,
    pub(crate) owner: &'owner CssColorOperationOwner,
    pub(crate) phase: usize,
    bank: CssColorBank,
    pub(crate) data: Data<'owner>,
}
impl<'owner> CssColorSlot<'owner> {
    pub(crate) fn new(
        owner: &'owner CssColorOperationOwner,
        phase: usize,
        index: usize,
        data: Data<'owner>,
    ) -> Self {
        Self {
            index,
            owner,
            phase,
            bank: data.bank(),
            data,
        }
    }
    /// Actual compiled record layout for admission of any of the three banks.
    #[must_use]
    pub const fn record_bytes() -> usize {
        std::mem::size_of::<Self>()
    }
    /// The bank this record was minted for.
    #[must_use]
    pub const fn bank(&self) -> CssColorBank {
        self.bank
    }
    pub(crate) const fn kind(&self) -> CssColorSlotKind {
        self.data.kind()
    }
    /// Whether the parse of `owner`'s `phase` minted this record.
    pub(in crate::color) fn minted_by(&self, owner: &CssColorOperationOwner, phase: usize) -> bool {
        std::ptr::eq(self.owner, owner) && self.phase == phase
    }
    /// The bank and index this record was minted for.
    pub(in crate::color) const fn position(&self) -> (CssColorBank, usize) {
        (self.bank, self.index)
    }
    pub(crate) fn qualify(
        &self,
        owner: &CssColorOperationOwner,
        phase: usize,
        index: usize,
        bank: CssColorBank,
    ) -> Result<(), ColorProtocolError> {
        if !std::ptr::eq(self.owner, owner) || self.phase != phase || owner.current_phase() != phase
        {
            return Err(ColorProtocolError::ForeignSlot);
        }
        if self.bank != bank || self.bank != self.data.bank() {
            return Err(ColorProtocolError::WrongSlot);
        }
        if index != self.index {
            return Err(ColorProtocolError::WrongIndex);
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

/// Qualification of canonical records, independent of the input's provenance.
#[derive(Clone, Copy)]
pub(crate) struct RecordTable<'css> {
    pub(crate) owner: &'css CssColorOperationOwner,
    pub(crate) phase: usize,
    pub(crate) length: usize,
}
#[derive(Clone, Copy)]
pub(crate) struct NumberHeader {
    pub(crate) layout: crate::tokenization::number::Layout,
    pub(crate) digits: RecordRange,
}
impl RecordTable<'_> {
    pub(crate) fn observe(
        &self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<LexicalRecord, ColorProtocolError> {
        self.qualify(index, slot)?;
        Ok(match slot.data {
            Data::NameUnit(value) => LexicalRecord::NameUnit(value),
            Data::NumberDigit { exponent, value } => LexicalRecord::NumberDigit { exponent, value },
            Data::Token(value) => LexicalRecord::Token(value),
            _ => return Err(ColorProtocolError::WrongSlot),
        })
    }
    pub(crate) fn qualify(
        &self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        slot.qualify(self.owner, self.phase, index, CssColorBank::Algorithm)?;
        if index >= self.length {
            return Err(ColorProtocolError::WrongIndex);
        }
        Ok(())
    }
    pub(crate) fn number_header(
        &self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<NumberHeader, ColorProtocolError> {
        self.qualify(index, slot)?;
        let header = match slot.data {
            Data::Token(crate::tokenization::tokenizer::Token {
                kind: crate::tokenization::tokenizer::TokenKind::Number { layout, digits, .. },
                ..
            }) => NumberHeader {
                layout,
                digits: RecordRange {
                    start: digits.start,
                    end: digits.end,
                },
            },
            Data::AuthoredNumber(number) => NumberHeader {
                layout: number.layout,
                digits: number.digits,
            },
            _ => return Err(ColorProtocolError::WrongSlot),
        };
        if header.layout.mantissa_digits == 0
            || header.layout.fraction_digits > header.layout.mantissa_digits
            || header
                .digits
                .start
                .checked_add(header.layout.mantissa_digits)
                .and_then(|end| end.checked_add(header.layout.exponent_digits))
                != Some(header.digits.end)
            || header.digits.end > self.length
        {
            return Err(ColorProtocolError::WrongIndex);
        }
        Ok(header)
    }
    pub(crate) fn name_header(
        &self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<Option<RecordRange>, ColorProtocolError> {
        self.qualify(index, slot)?;
        let range = match slot.data {
            Data::Token(crate::tokenization::tokenizer::Token {
                kind:
                    crate::tokenization::tokenizer::TokenKind::Name {
                        kind: crate::tokenization::tokenizer::NameKind::Ident,
                        units,
                    },
                ..
            }) => RecordRange {
                start: units.start,
                end: units.end,
            },
            Data::AuthoredName(name) => name.units,
            Data::Token(_) | Data::NameUnit(_) | Data::NumberDigit { .. } => return Ok(None),
            _ => return Err(ColorProtocolError::WrongSlot),
        };
        Ok(Some(range))
    }
}

#[derive(Clone, Copy)]
pub(crate) struct LexicalTable<'owner> {
    pub(crate) owner: &'owner CssColorOperationOwner,
    pub(crate) phase: usize,
    length: usize,
    scalars: usize,
}
impl<'owner> LexicalTable<'owner> {
    pub(crate) const fn records(&self) -> RecordTable<'owner> {
        RecordTable {
            owner: self.owner,
            phase: self.phase,
            length: self.length,
        }
    }
    /// Actual ordinary component events use the same qualified lexical records.
    /// The adapter owns projection/admission; every later record read still
    /// qualifies its exact bank, ordinal, anchor and phase.
    pub(crate) fn component_events(
        owner: &'owner CssColorOperationOwner,
        phase: usize,
        length: usize,
    ) -> Result<Self, ColorProtocolError> {
        if phase == 0 || owner.current_phase() != phase {
            return Err(ColorProtocolError::ForeignSlot);
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
    ) -> Result<LexicalRecord, ColorProtocolError> {
        slot.qualify(self.owner, self.phase, index, CssColorBank::Algorithm)?;
        if index >= self.length {
            return Err(ColorProtocolError::WrongIndex);
        }
        Ok(match slot.data {
            Data::Scalar(_) => return Err(ColorProtocolError::WrongSlot),
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
            | Data::AuthoredNumber(_)
            | Data::AuthoredName(_)
            | Data::AuthoredSite(_)
            | Data::AuthoredTraversal(_)
            | Data::AuthoredNumericFrame(_)
            | Data::AuthoredColorFrame(_)
            | Data::AuthoredStagedComponent(_)
            | Data::MixTask(_) => return Err(ColorProtocolError::WrongSlot),
        })
    }
}

/// An inconsistency the canonical machines detected while being driven.
///
/// The public typestate makes every call order and action kind static, so
/// apart from [`Malformed`](Self::Malformed) each variant is a defect in the
/// code driving a parse, never a property of CSS input. Public boundaries
/// raise it as a panic (decision 5 of #1353); only the crate-private authored
/// protocol reports it to CSS's own driver.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ColorProtocolError {
    WrongAction,
    WrongIndex,
    WrongSlot,
    ForeignSlot,
    Overflow,
    Input,
    Terminal,
    EmptySlot,
    /// The caller's original input is malformed for its declared encoding at
    /// this original unit offset: input, not misuse.
    Malformed {
        offset: usize,
    },
}

impl std::fmt::Display for ColorProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::WrongAction => "color action does not match the pending request",
            Self::WrongIndex => "color record index does not match the requested ordinal",
            Self::WrongSlot => "color record kind or bank does not match the request",
            Self::ForeignSlot => "color record belongs to another operation or phase",
            Self::Overflow => "color operation extent cannot be represented",
            Self::Input => "color input observation violates its declared encoding or record",
            Self::Terminal => "color operation has no completed result",
            Self::EmptySlot => "color transition has no caller-owned checkpoint",
            Self::Malformed { offset } => {
                return write!(f, "color input is malformed at unit {offset}");
            }
        })
    }
}
impl ColorProtocolError {
    /// Raises a defect in the code driving a parse, as slice indexing does.
    ///
    /// # Panics
    ///
    /// Always.
    #[track_caller]
    pub(crate) fn raise(self) -> ! {
        panic!("CSS color protocol misuse: {self}")
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
    length: usize,
    stage: Stage,
    scalar_count: usize,
    slot_count: usize,
}
impl<'owner> LexicalOperation<'owner> {
    pub(crate) fn start(
        owner: &'owner mut CssColorOperationOwner,
        kind: InputKind,
        length: usize,
    ) -> Result<Self, ColorProtocolError> {
        // Reading the fixed anchor/header is part of the caller's start admission.
        let _ = owner.marker;
        let phase = owner.begin()?;
        Ok(Self {
            owner,
            phase,
            length,
            stage: Stage::Input(InputCursor::new(kind, length)),
            scalar_count: 0,
            slot_count: 0,
        })
    }

    pub(crate) fn description(&self) -> ActionDescription {
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

    /// The anchor and phase that qualify every record this phase mints.
    pub(crate) const fn qualifier(&self) -> (&'owner CssColorOperationOwner, usize) {
        (self.owner, self.phase)
    }

    /// Stop never consumes caller buffers or relabels them empty.
    pub(crate) fn abandon(&mut self) {
        self.stage = Stage::Stopped;
    }

    #[cfg(test)]
    pub(crate) fn completed_length(&self) -> Result<usize, ColorProtocolError> {
        if matches!(self.stage, Stage::Done) {
            Ok(self.slot_count)
        } else {
            Err(ColorProtocolError::Terminal)
        }
    }

    pub(crate) fn finish(self) -> Result<LexicalTable<'owner>, ColorProtocolError> {
        if !matches!(self.stage, Stage::Done) {
            return Err(ColorProtocolError::Terminal);
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
    ) -> Result<(), ColorProtocolError> {
        slot.qualify(self.owner, self.phase, slot.index, CssColorBank::Algorithm)?;
        if slot.kind() != kind {
            return Err(ColorProtocolError::WrongSlot);
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

    pub(crate) fn read_input(self, offset: usize, unit: u16) -> Result<(), ColorProtocolError> {
        if self.description != (ActionDescription::ReadInput { offset }) {
            return Err(ColorProtocolError::WrongAction);
        }
        let Stage::Input(cursor) = &mut self.operation.stage else {
            return Err(ColorProtocolError::WrongAction);
        };
        cursor.read(unit).map_err(|error| match error {
            // A UTF-8 unit beyond a byte, or a byte no UTF-8 sequence admits.
            InputError::WrongUnit | InputError::InvalidUtf8 => {
                ColorProtocolError::Malformed { offset }
            }
            _ => ColorProtocolError::Input,
        })
    }

    pub(crate) fn read_slot(
        self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        if self.description
            != (ActionDescription::ReadSlot {
                index,
                kind: CssColorSlotKind::Scalar,
            })
        {
            return Err(ColorProtocolError::WrongAction);
        }
        self.operation.qualify(slot, CssColorSlotKind::Scalar)?;
        if slot.index != index {
            return Err(ColorProtocolError::WrongIndex);
        }
        let Data::Scalar(scalar) = slot.data else {
            return Err(ColorProtocolError::WrongSlot);
        };
        let Stage::Tokens(cursor) = &mut self.operation.stage else {
            return Err(ColorProtocolError::WrongAction);
        };
        cursor
            .read_scalar(index, scalar)
            .map_err(|_| ColorProtocolError::Input)
    }

    /// Generates one fixed record, without advancing its pending append.
    pub(crate) fn append_record(&mut self) -> Result<CssColorSlot<'owner>, ColorProtocolError> {
        if self.issued {
            return Err(ColorProtocolError::WrongAction);
        }
        if !matches!(self.description, ActionDescription::AppendSlot { .. }) {
            return Err(ColorProtocolError::WrongAction);
        }
        self.operation
            .slot_count
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        let data = match &self.operation.stage {
            Stage::Input(cursor) => {
                let InputRequest::Emit(scalar) = cursor.request() else {
                    return Err(ColorProtocolError::WrongAction);
                };
                self.operation
                    .scalar_count
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                Data::Scalar(scalar)
            }
            Stage::Tokens(cursor) => {
                let TokenRequest::Append { record, .. } = cursor.request() else {
                    return Err(ColorProtocolError::WrongAction);
                };
                match record {
                    Record::NameUnit(value) => Data::NameUnit(value),
                    Record::Digit { exponent, value } => Data::NumberDigit { exponent, value },
                    Record::Token(value) => Data::Token(value),
                }
            }
            Stage::Done | Stage::Stopped => return Err(ColorProtocolError::Terminal),
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
    pub(crate) fn appended(self, index: usize) -> Result<(), ColorProtocolError> {
        if !self.issued {
            return Err(ColorProtocolError::WrongAction);
        }
        if !matches!(self.description, ActionDescription::AppendSlot { index: expected, .. } if expected == index)
        {
            return Err(ColorProtocolError::WrongIndex);
        }
        let slots = self
            .operation
            .slot_count
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        match &mut self.operation.stage {
            Stage::Input(cursor) => {
                let scalars = self
                    .operation
                    .scalar_count
                    .checked_add(1)
                    .ok_or(ColorProtocolError::Overflow)?;
                cursor
                    .emitted()
                    .map_err(|_| ColorProtocolError::WrongAction)?;
                self.operation.scalar_count = scalars;
            }
            Stage::Tokens(cursor) => cursor
                .appended(index)
                .map_err(|_| ColorProtocolError::WrongAction)?,
            Stage::Done | Stage::Stopped => return Err(ColorProtocolError::Terminal),
        }
        self.operation.slot_count = slots;
        Ok(())
    }

    pub(crate) fn compute(self) -> Result<(), ColorProtocolError> {
        match (&mut self.operation.stage, self.description) {
            (Stage::Input(cursor), ActionDescription::ComputeInput) => match cursor.request() {
                InputRequest::HeldUnit => cursor
                    .process_held()
                    .map_err(|_| ColorProtocolError::Input)?,
                InputRequest::End => {
                    let ended = cursor.end().map_err(|error| match error {
                        // The input ends inside a multi-unit UTF-8 sequence.
                        InputError::InvalidUtf8 => ColorProtocolError::Malformed {
                            offset: self.operation.length,
                        },
                        _ => ColorProtocolError::Input,
                    })?;
                    if ended {
                        self.operation.stage = Stage::Tokens(TokenCursor::new(
                            self.operation.scalar_count,
                            self.operation.slot_count,
                        ));
                    }
                }
                _ => return Err(ColorProtocolError::WrongAction),
            },
            (Stage::Tokens(cursor), ActionDescription::ComputeToken) => {
                if cursor.request() == TokenRequest::Done {
                    self.operation.stage = Stage::Done;
                } else {
                    cursor.compute().map_err(|_| ColorProtocolError::Input)?;
                }
            }
            _ => return Err(ColorProtocolError::WrongAction),
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tokenization::tokenizer::{NameKind, TokenKind};

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
            ColorProtocolError::WrongSlot
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
            ColorProtocolError::ForeignSlot
        );
        assert_eq!(
            table.owner.advance_from(table.phase),
            Err(ColorProtocolError::ForeignSlot)
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
            Err(ColorProtocolError::ForeignSlot)
        );
        assert_eq!(second.next_action().description(), read);
        second.next_action().read_slot(0, &local).unwrap();
        second.abandon();
        assert_eq!(second.completed_length(), Err(ColorProtocolError::Terminal));
        assert_eq!(
            second.next_action().description(),
            ActionDescription::Stopped
        );
        // Actual caller slots remain available; abandoning the cursor refunds nothing.
        assert_eq!(local.kind(), CssColorSlotKind::Scalar);
        assert!(!first_slots.is_empty());
    }
}
