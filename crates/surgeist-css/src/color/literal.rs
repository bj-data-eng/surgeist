//! Numeric leaf conversion, retaining unit/type separately from channel policy.

use super::decimal::{GradScale, NumberAction, NumberConversion, ScaleAction};
use super::protocol::{
    CssColorProtocolError, CssColorSlot, Data, LexicalRecord, LexicalTable, RecordTable,
};
use super::tokens::ClassifiedToken;
use crate::numeric::bounded_decimal::DecimalOutcome;
use crate::tokenization::color_tokens::NumberKind;
use crate::{CssAngleUnit, CssNumericType, CssNumericUnit};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct LiteralValue {
    pub(crate) value: f64,
    pub(crate) unit: Option<CssNumericUnit>,
    pub(crate) percentage: bool,
    pub(crate) ty: CssNumericType,
    /// The authored finite decimal, or its canonical angle, exceeded binary64.
    pub(crate) overflowed: bool,
}

#[cfg(test)]
mod tests {
    use super::super::protocol::{ActionDescription, CssColorOperationOwner, LexicalOperation};
    use super::super::tokens::{TokenAction, TokenReader};
    use super::*;
    use crate::tokenization::bounded::InputKind;

    fn lexical<'a>(operation: &mut LexicalOperation<'a>, input: &[u8]) -> Vec<CssColorSlot<'a>> {
        let mut slots = Vec::new();
        loop {
            let mut action = operation.next_action();
            match action.description() {
                ActionDescription::ReadInput { offset } => {
                    action.read_input(offset, u16::from(input[offset])).unwrap()
                }
                ActionDescription::ReadSlot { index, .. } => {
                    action.read_slot(index, &slots[index]).unwrap()
                }
                ActionDescription::AppendSlot { index, .. } => {
                    assert_eq!(index, slots.len());
                    slots.push(action.append_record().unwrap());
                    action.appended(index).unwrap();
                }
                ActionDescription::ComputeInput | ActionDescription::ComputeToken => {
                    action.compute().unwrap()
                }
                ActionDescription::Done => return slots,
                ActionDescription::Stopped => panic!("stop"),
            }
        }
    }
    fn token(table: &LexicalTable<'_>, slots: &[CssColorSlot<'_>]) -> ClassifiedToken {
        let mut reader = TokenReader::start(table);
        loop {
            match reader.action() {
                TokenAction::ReadRecord { index } | TokenAction::ReadName { index } => {
                    reader.read(index, &slots[index]).unwrap()
                }
                TokenAction::Classify => reader.classify().unwrap(),
                TokenAction::Yield => return reader.current().unwrap(),
                _ => panic!("numeric token"),
            }
        }
    }
    fn drive<'owner>(
        cursor: &mut LiteralConversion<'owner>,
        slots: &mut Vec<CssColorSlot<'owner>>,
    ) -> LiteralValue {
        loop {
            match cursor.action() {
                LiteralAction::ReadSource { index }
                | LiteralAction::ReadScaled { index }
                | LiteralAction::ReadScratch { index }
                | LiteralAction::ReadHeader { index } => cursor.read(index, &slots[index]).unwrap(),
                LiteralAction::Append { index } => {
                    assert_eq!(index, slots.len());
                    slots.push(cursor.record(index).unwrap());
                    cursor.written(index, &slots[index]).unwrap();
                }
                LiteralAction::Replace { index } => {
                    slots[index] = cursor.record(index).unwrap();
                    cursor.written(index, &slots[index]).unwrap();
                }
                LiteralAction::Compute => cursor.compute().unwrap(),
                LiteralAction::Done(value) => {
                    assert_eq!(cursor.scratch_end(), slots.len());
                    return value;
                }
                LiteralAction::Stopped => panic!("stop"),
            }
        }
    }
    #[test]
    fn literal_types_and_angle_units_reuse_the_actual_decoded_owner() {
        for (input, expected, expected_unit, percentage) in [
            ("-0", 0.0f64, None, false),
            ("50%", 50.0, None, true),
            (
                ".5TuRn",
                180.0,
                Some(CssNumericUnit::Angle(CssAngleUnit::Degrees)),
                false,
            ),
            (
                "1.5grad",
                1.5 * 0.9,
                Some(CssNumericUnit::Angle(CssAngleUnit::Degrees)),
                false,
            ),
            (
                "1.9e308grad",
                1.71e308,
                Some(CssNumericUnit::Angle(CssAngleUnit::Degrees)),
                false,
            ),
            (
                "-1.9e308grad",
                -1.71e308,
                Some(CssNumericUnit::Angle(CssAngleUnit::Degrees)),
                false,
            ),
            (
                "12PX",
                12.0,
                Some(CssNumericUnit::Length(crate::CssLengthUnit::Px)),
                false,
            ),
            (
                "3ms",
                3.0,
                Some(CssNumericUnit::Time(crate::CssTimeUnit::Milliseconds)),
                false,
            ),
            (
                "2DPCM",
                2.0,
                Some(CssNumericUnit::Resolution(crate::CssResolutionUnit::Dpcm)),
                false,
            ),
        ] {
            let mut owner = CssColorOperationOwner::new();
            let mut lexer =
                LexicalOperation::start(&mut owner, InputKind::Utf8, input.len()).unwrap();
            let mut slots = lexical(&mut lexer, input.as_bytes());
            let table = lexer.finish().unwrap();
            let token = token(&table, &slots);
            let mut cursor =
                LiteralConversion::start(&table, token, &slots[token.index], table.length())
                    .unwrap();
            let value = drive(&mut cursor, &mut slots);
            assert_eq!(value.value.to_bits(), expected.to_bits(), "{input}");
            assert_eq!(value.unit, expected_unit);
            assert_eq!(value.percentage, percentage);
            assert_eq!(
                value.ty,
                crate::numeric::literal_type(expected_unit, percentage)
            );
        }
        for invalid in [" DEG", "deg ", "değ", "gradx", "㎐", "pXx"] {
            assert_eq!(CssNumericUnit::parse(invalid), None);
        }
    }
    #[test]
    fn numeric_overflow_is_not_grammar_failure_or_explicit_infinity() {
        for input in ["1e999", "-1e999", "1e999deg", "-1e999grad"] {
            let mut owner = CssColorOperationOwner::new();
            let mut lexer =
                LexicalOperation::start(&mut owner, InputKind::Utf8, input.len()).unwrap();
            let mut slots = lexical(&mut lexer, input.as_bytes());
            let table = lexer.finish().unwrap();
            let token = token(&table, &slots);
            let mut cursor =
                LiteralConversion::start(&table, token, &slots[token.index], table.length())
                    .unwrap();
            let value = drive(&mut cursor, &mut slots);
            if input.ends_with("deg") || input.ends_with("grad") {
                assert_eq!(value.value.abs().to_bits(), f64::MAX.to_bits() - 31);
            } else {
                // The closest finite value, never an infinity.
                assert_eq!(value.value.abs(), f64::MAX);
            }
            assert_eq!(value.value.is_sign_negative(), input.starts_with('-'));
        }
    }
    #[test]
    fn actual_foreign_header_and_unit_failure_preserve_pending_source() {
        let mut owner = CssColorOperationOwner::new();
        let mut other = CssColorOperationOwner::new();
        let mut lexer = LexicalOperation::start(&mut owner, InputKind::Utf8, 2).unwrap();
        let slots = lexical(&mut lexer, b"12");
        let table = lexer.finish().unwrap();
        let token = token(&table, &slots);
        let mut foreign = LexicalOperation::start(&mut other, InputKind::Utf8, 2).unwrap();
        let other_slots = lexical(&mut foreign, b"34");
        assert!(matches!(
            LiteralConversion::start(&table, token, &other_slots[token.index], table.length()),
            Err(CssColorProtocolError::ForeignSlot)
        ));
        let mut cursor =
            LiteralConversion::start(&table, token, &slots[token.index], table.length()).unwrap();
        let LiteralAction::ReadSource { index } = cursor.action() else {
            panic!("read")
        };
        assert_eq!(
            cursor.read(index, &other_slots[index]),
            Err(CssColorProtocolError::ForeignSlot)
        );
        assert_eq!(cursor.action(), LiteralAction::ReadSource { index });
        cursor.abandon();
        assert_eq!(cursor.action(), LiteralAction::Stopped);
        assert_eq!(cursor.compute(), Err(CssColorProtocolError::WrongAction));
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum LiteralAction {
    ReadSource { index: usize },
    ReadScaled { index: usize },
    ReadScratch { index: usize },
    ReadHeader { index: usize },
    Append { index: usize },
    Replace { index: usize },
    Compute,
    Done(LiteralValue),
    Stopped,
}
enum Stage<'owner> {
    Decimal(NumberConversion<'owner>),
    Header,
    Scale(GradScale<'owner>),
    Scaled(NumberConversion<'owner>),
    Done(LiteralValue),
    Stopped,
}
pub(crate) struct LiteralConversion<'owner> {
    table: RecordTable<'owner>,
    input: LiteralInput,
    stage: Stage<'owner>,
    scratch_end: usize,
}
#[derive(Clone, Copy)]
struct LiteralInput {
    index: usize,
    unit: Option<CssNumericUnit>,
    percentage: bool,
}
impl<'owner> LiteralConversion<'owner> {
    /// The unit observation came from the same decoded, finite token reader.
    /// The actual token header is qualified before any cursor is constructed.
    pub(crate) fn start(
        table: &LexicalTable<'owner>,
        token: ClassifiedToken,
        header: &CssColorSlot<'_>,
        scratch_base: usize,
    ) -> Result<Self, CssColorProtocolError> {
        let LexicalRecord::Token(actual) = table.observe(token.index, header)? else {
            return Err(CssColorProtocolError::WrongSlot);
        };
        if actual != token.token {
            return Err(CssColorProtocolError::WrongSlot);
        }
        let crate::tokenization::color_tokens::TokenKind::Number { kind, .. } = actual.kind else {
            return Err(CssColorProtocolError::Input);
        };
        if kind == NumberKind::Dimension && token.unit.is_none() {
            return Err(CssColorProtocolError::Input);
        }
        if kind != NumberKind::Dimension && token.unit.is_some() {
            return Err(CssColorProtocolError::Input);
        }
        Self::start_input(
            table.records(),
            LiteralInput {
                index: token.index,
                unit: token.unit,
                percentage: kind == NumberKind::Percentage,
            },
            header,
            scratch_base,
        )
    }
    pub(crate) fn start_authored(
        table: &RecordTable<'owner>,
        index: usize,
        header: &CssColorSlot<'_>,
        scratch_base: usize,
    ) -> Result<Self, CssColorProtocolError> {
        table.qualify(index, header)?;
        let Data::AuthoredNumber(number) = header.data else {
            return Err(CssColorProtocolError::WrongSlot);
        };
        Self::start_input(
            *table,
            LiteralInput {
                index,
                unit: number.unit,
                percentage: number.percentage,
            },
            header,
            scratch_base,
        )
    }
    fn start_input(
        table: RecordTable<'owner>,
        input: LiteralInput,
        header: &CssColorSlot<'_>,
        scratch_base: usize,
    ) -> Result<Self, CssColorProtocolError> {
        let decimal = NumberConversion::start(&table, input.index, header, scratch_base)?;
        Ok(Self {
            table,
            input,
            stage: Stage::Decimal(decimal),
            scratch_end: scratch_base,
        })
    }
    pub(crate) fn action(&self) -> LiteralAction {
        match &self.stage {
            Stage::Decimal(decimal) | Stage::Scaled(decimal) => match decimal.action() {
                NumberAction::ReadSource { index } => LiteralAction::ReadSource { index },
                NumberAction::ReadScaled { index } => LiteralAction::ReadScaled { index },
                NumberAction::ReadScratch { index } => LiteralAction::ReadScratch { index },
                NumberAction::AppendScratch { index } => LiteralAction::Append { index },
                NumberAction::ReplaceScratch { index } => LiteralAction::Replace { index },
                NumberAction::Compute | NumberAction::Done(_) => LiteralAction::Compute,
                NumberAction::Stopped => LiteralAction::Stopped,
            },
            Stage::Header => LiteralAction::ReadHeader {
                index: self.input.index,
            },
            Stage::Scale(scale) => match scale.action() {
                ScaleAction::ReadSource { index } => LiteralAction::ReadSource { index },
                ScaleAction::Append { index } => LiteralAction::Append { index },
                ScaleAction::Compute | ScaleAction::Done => LiteralAction::Compute,
                ScaleAction::Stopped => LiteralAction::Stopped,
            },
            Stage::Done(value) => LiteralAction::Done(*value),
            Stage::Stopped => LiteralAction::Stopped,
        }
    }
    pub(crate) fn read(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), CssColorProtocolError> {
        let action = self.action();
        match &mut self.stage {
            Stage::Decimal(decimal) | Stage::Scaled(decimal) => match action {
                LiteralAction::ReadSource { index: expected } if expected == index => {
                    decimal.read_source(index, slot)
                }
                LiteralAction::ReadScaled { index: expected } if expected == index => {
                    decimal.read_scaled(index, slot)
                }
                LiteralAction::ReadScratch { index: expected } if expected == index => {
                    decimal.read_scratch(index, slot)
                }
                _ => Err(CssColorProtocolError::WrongAction),
            },
            Stage::Scale(scale) if action == (LiteralAction::ReadSource { index }) => {
                scale.read(index, slot)
            }
            Stage::Header if action == (LiteralAction::ReadHeader { index }) => {
                let scale = GradScale::start(&self.table, index, slot, self.scratch_end)?;
                self.stage = Stage::Scale(scale);
                Ok(())
            }
            _ => Err(CssColorProtocolError::WrongAction),
        }
    }
    pub(crate) fn record(
        &self,
        index: usize,
    ) -> Result<CssColorSlot<'owner>, CssColorProtocolError> {
        if !matches!(self.action(), LiteralAction::Append { index: i } | LiteralAction::Replace { index: i } if i == index)
        {
            return Err(CssColorProtocolError::WrongAction);
        }
        index
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)?;
        match &self.stage {
            Stage::Decimal(decimal) | Stage::Scaled(decimal) => decimal.scratch_record(index),
            Stage::Scale(scale) => scale.record(index),
            _ => Err(CssColorProtocolError::WrongAction),
        }
    }
    pub(crate) fn written(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), CssColorProtocolError> {
        let action = self.action();
        let end = index
            .checked_add(1)
            .ok_or(CssColorProtocolError::Overflow)?;
        if matches!(action, LiteralAction::Append { index: i } if i == index)
            && index != self.scratch_end
        {
            return Err(CssColorProtocolError::WrongIndex);
        }
        match &mut self.stage {
            Stage::Decimal(decimal) | Stage::Scaled(decimal) => {
                decimal.scratch_written(index, slot)?
            }
            Stage::Scale(scale) => scale.appended(index, slot)?,
            _ => return Err(CssColorProtocolError::WrongAction),
        }
        self.scratch_end = self.scratch_end.max(end);
        Ok(())
    }
    pub(crate) fn kernel(&self) -> super::CssColorKernel {
        match &self.stage {
            Stage::Decimal(cursor) | Stage::Scaled(cursor) => {
                if matches!(cursor.action(), NumberAction::Done(_)) {
                    super::CssColorKernel::Literal
                } else {
                    cursor.kernel()
                }
            }
            Stage::Scale(cursor) => {
                if cursor.action() == ScaleAction::Done {
                    super::CssColorKernel::Transition
                } else {
                    super::CssColorKernel::ScaleDigit
                }
            }
            _ => super::CssColorKernel::Transition,
        }
    }
    pub(crate) fn compute(&mut self) -> Result<(), CssColorProtocolError> {
        if self.action() != LiteralAction::Compute {
            return Err(CssColorProtocolError::WrongAction);
        }
        let scaled = matches!(self.stage, Stage::Scaled(_));
        let outcome = match &mut self.stage {
            Stage::Decimal(decimal) | Stage::Scaled(decimal) => match decimal.action() {
                NumberAction::Done(outcome) => Some(outcome),
                _ => {
                    return decimal.compute();
                }
            },
            Stage::Scale(scale) => {
                if scale.action() == ScaleAction::Done {
                    self.stage = Stage::Scaled(NumberConversion::start_scaled(
                        &self.table,
                        scale.finish()?,
                        self.scratch_end,
                    )?);
                    return Ok(());
                } else {
                    return scale.compute();
                }
            }
            _ => return Err(CssColorProtocolError::WrongAction),
        }
        .expect("selected completed numeric child");
        if !scaled
            && matches!(outcome, DecimalOutcome::Overflow { .. })
            && self.input.unit == Some(CssNumericUnit::Angle(CssAngleUnit::Gradians))
        {
            self.stage = Stage::Header;
            return Ok(());
        }
        let angle = matches!(self.input.unit, Some(CssNumericUnit::Angle(_)));
        let mut overflowed = matches!(outcome, DecimalOutcome::Overflow { .. });
        let value = match outcome {
            DecimalOutcome::Finite(value) => value,
            // Values 4 §10.12: a finite value beyond the supported range is
            // the closest supported value. An angle instead reaches its
            // 360deg-multiple endpoint below, after unit scaling.
            DecimalOutcome::Overflow { negative } if angle => {
                f64::INFINITY.copysign(if negative { -1.0 } else { 1.0 })
            }
            DecimalOutcome::Overflow { negative } => {
                f64::MAX.copysign(if negative { -1.0 } else { 1.0 })
            }
        };
        let (value, unit) = match self.input.unit {
            Some(CssNumericUnit::Angle(unit)) => {
                let scale = CssNumericUnit::Angle(unit)
                    .intrinsic_projection()
                    .expect("all canonical angle units are intrinsic");
                let degrees = if scaled { value } else { scale.apply(value) };
                overflowed |= degrees.is_infinite();
                (
                    crate::numeric::projection::closest_supported_value(degrees, true, true),
                    Some(CssNumericUnit::Angle(CssAngleUnit::Degrees)),
                )
            }
            unit => (value, unit),
        };
        let percentage = self.input.percentage;
        self.stage = Stage::Done(LiteralValue {
            value,
            unit,
            percentage,
            ty: crate::numeric::literal_type(unit, percentage),
            overflowed,
        });
        Ok(())
    }
    pub(crate) const fn scratch_end(&self) -> usize {
        self.scratch_end
    }
    pub(crate) fn abandon(&mut self) {
        match &mut self.stage {
            Stage::Decimal(cursor) | Stage::Scaled(cursor) => cursor.abandon(),
            Stage::Scale(cursor) => cursor.abandon(),
            _ => {}
        }
        self.stage = Stage::Stopped;
    }
}
