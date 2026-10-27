//! The actual shared decimal child, qualified to retained lexer/scratch records.

use crate::color::controlled::CssColorSlot;
use crate::color::controlled::protocol::{
    ColorProtocolError, Data, LexicalRecord, RecordRange, RecordTable,
};
use crate::numeric::bounded_decimal::{
    DecimalCursor, DecimalInput, DecimalLayout, DecimalOutcome, DecimalReply, DecimalRequest,
};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum NumberAction {
    ReadSource { index: usize },
    ReadScaled { index: usize },
    ReadScratch { index: usize },
    AppendScratch { index: usize },
    ReplaceScratch { index: usize },
    Compute,
    Done(DecimalOutcome),
    Stopped,
}

pub(crate) struct NumberConversion<'owner> {
    table: RecordTable<'owner>,
    token: usize,
    digits: RecordRange,
    mantissa: usize,
    scaled: Option<RecordRange>,
    exponent: usize,
    scratch_base: usize,
    cursor: DecimalCursor,
    stopped: bool,
}
impl<'owner> NumberConversion<'owner> {
    /// The caller admits the actual fixed layout before construction. The
    /// source token must be a real qualified grammar-produced numeric token.
    pub(crate) fn start(
        table: &RecordTable<'owner>,
        index: usize,
        slot: &CssColorSlot<'_>,
        scratch_base: usize,
    ) -> Result<Self, ColorProtocolError> {
        let crate::color::controlled::protocol::NumberHeader { layout, digits } =
            table.number_header(index, slot)?;
        let length = layout
            .mantissa_digits
            .checked_add(layout.exponent_digits)
            .ok_or(ColorProtocolError::Overflow)?;
        if digits.start.checked_add(length) != Some(digits.end) || digits.end > table.length {
            return Err(ColorProtocolError::WrongIndex);
        }
        let child = DecimalLayout::new(
            layout.negative,
            layout.mantissa_digits,
            layout.fraction_digits,
            layout.exponent_negative,
            layout.exponent_digits.max(1),
        )
        .map_err(|_| ColorProtocolError::Input)?;
        if scratch_base < table.length {
            return Err(ColorProtocolError::WrongIndex);
        }
        // Child contract: the exact binary64 boundary needs at most1100 digits.
        // Guard the translation before any scratch action can be generated.
        scratch_base
            .checked_add(1100)
            .ok_or(ColorProtocolError::Overflow)?;
        Ok(Self {
            table: *table,
            token: index,
            digits,
            mantissa: layout.mantissa_digits,
            exponent: layout.exponent_digits,
            scratch_base,
            scaled: None,
            cursor: DecimalCursor::new(child),
            stopped: false,
        })
    }
    pub(crate) fn start_scaled(
        table: &RecordTable<'owner>,
        scale: ScaledNumber<'owner>,
        scratch_base: usize,
    ) -> Result<Self, ColorProtocolError> {
        if !std::ptr::eq(table.owner, scale.owner) || table.phase != scale.phase {
            return Err(ColorProtocolError::ForeignSlot);
        }
        if scratch_base < scale.digits.end {
            return Err(ColorProtocolError::WrongIndex);
        }
        scratch_base
            .checked_add(1100)
            .ok_or(ColorProtocolError::Overflow)?;
        let child = DecimalLayout::new(
            scale.layout.negative,
            scale.layout.mantissa_digits,
            scale.layout.fraction_digits,
            scale.layout.exponent_negative,
            scale.layout.exponent_digits.max(1),
        )
        .map_err(|_| ColorProtocolError::Input)?;
        Ok(Self {
            table: *table,
            token: scale.token,
            digits: scale.source_digits,
            mantissa: scale.source_mantissa,
            exponent: scale.layout.exponent_digits,
            scaled: Some(scale.digits),
            scratch_base,
            cursor: DecimalCursor::new(child),
            stopped: false,
        })
    }
    fn source_index(&self, kind: DecimalInput, index: usize) -> usize {
        self.digits.start
            + if kind == DecimalInput::Exponent {
                self.mantissa + index
            } else {
                index
            }
    }
    pub(crate) fn action(&self) -> NumberAction {
        if self.stopped {
            return NumberAction::Stopped;
        }
        match *self.cursor.next_request() {
            DecimalRequest::ReadInput {
                kind: DecimalInput::Exponent,
                ..
            } if self.exponent == 0 => NumberAction::Compute,
            DecimalRequest::ReadInput {
                kind: DecimalInput::Mantissa,
                index,
            } if self.scaled.is_some() => {
                let span = self.scaled.expect("selected scaled input");
                NumberAction::ReadScaled {
                    index: span.end - 1 - index,
                }
            }
            DecimalRequest::ReadInput { kind, index } => NumberAction::ReadSource {
                index: self.source_index(kind, index),
            },
            DecimalRequest::ReadScratch { index } => NumberAction::ReadScratch {
                index: self.scratch_base + index,
            },
            DecimalRequest::AppendScratch { index, .. } => NumberAction::AppendScratch {
                index: self.scratch_base + index,
            },
            DecimalRequest::ReplaceScratch { index, .. } => NumberAction::ReplaceScratch {
                index: self.scratch_base + index,
            },
            DecimalRequest::Compute(_) | DecimalRequest::Failed(_) => NumberAction::Compute,
            DecimalRequest::Done(value) => NumberAction::Done(value),
        }
    }
    pub(crate) fn read_source(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        if self.action() != (NumberAction::ReadSource { index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        let DecimalRequest::ReadInput {
            kind,
            index: child_index,
        } = *self.cursor.next_request()
        else {
            return Err(ColorProtocolError::WrongAction);
        };
        let LexicalRecord::NumberDigit { exponent, value } = self.table.observe(index, slot)?
        else {
            return Err(ColorProtocolError::WrongSlot);
        };
        if exponent != (kind == DecimalInput::Exponent) {
            return Err(ColorProtocolError::WrongSlot);
        }
        self.cursor
            .complete(DecimalReply::Input {
                kind,
                index: child_index,
                digit: value,
            })
            .map_err(|_| ColorProtocolError::Input)
    }
    pub(crate) fn read_scaled(
        &mut self,
        index: usize,
        record: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        if self.action() != (NumberAction::ReadScaled { index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        record.qualify(
            self.table.owner,
            self.table.phase,
            index,
            crate::color::controlled::CssColorBank::Algorithm,
        )?;
        if record.index != index {
            return Err(ColorProtocolError::WrongIndex);
        }
        let Data::ScaledDigit { token, value } = record.data else {
            return Err(ColorProtocolError::WrongSlot);
        };
        if token != self.token || value > 9 {
            return Err(ColorProtocolError::WrongSlot);
        }
        let DecimalRequest::ReadInput {
            kind: DecimalInput::Mantissa,
            index: child_index,
        } = *self.cursor.next_request()
        else {
            return Err(ColorProtocolError::WrongAction);
        };
        self.cursor
            .complete(DecimalReply::Input {
                kind: DecimalInput::Mantissa,
                index: child_index,
                digit: value,
            })
            .map_err(|_| ColorProtocolError::Input)
    }
    fn qualify(&self, index: usize, record: &CssColorSlot<'_>) -> Result<(), ColorProtocolError> {
        record.qualify(
            self.table.owner,
            self.table.phase,
            index,
            crate::color::controlled::CssColorBank::Algorithm,
        )?;
        if record.index != index {
            return Err(ColorProtocolError::WrongIndex);
        }
        if !matches!(record.data, Data::DecimalDigit { token, .. } if token == self.token) {
            return Err(ColorProtocolError::WrongSlot);
        }
        Ok(())
    }
    pub(crate) fn read_scratch(
        &mut self,
        index: usize,
        record: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        if self.action() != (NumberAction::ReadScratch { index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        self.qualify(index, record)?;
        let Data::DecimalDigit { digit, .. } = record.data else {
            return Err(ColorProtocolError::WrongSlot);
        };
        let digit = crate::numeric::bounded_decimal::DecimalDigit::new(digit.digit())
            .map_err(|_| ColorProtocolError::WrongSlot)?;
        self.cursor
            .complete(DecimalReply::Scratch {
                index: index - self.scratch_base,
                digit,
            })
            .map_err(|_| ColorProtocolError::Input)
    }
    pub(crate) fn scratch_record(
        &self,
        index: usize,
    ) -> Result<CssColorSlot<'owner>, ColorProtocolError> {
        if self.stopped {
            return Err(ColorProtocolError::Terminal);
        }
        let digit = match *self.cursor.next_request() {
            DecimalRequest::AppendScratch {
                index: expected,
                digit,
            }
            | DecimalRequest::ReplaceScratch {
                index: expected,
                digit,
            } if index == self.scratch_base + expected => digit,
            _ => return Err(ColorProtocolError::WrongIndex),
        };
        Ok(CssColorSlot::new(
            self.table.owner,
            self.table.phase,
            index,
            Data::DecimalDigit {
                token: self.token,
                digit,
            },
        ))
    }
    pub(crate) fn scratch_written(
        &mut self,
        index: usize,
        record: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        self.qualify(index, record)?;
        let expected = self.scratch_record(index)?;
        let Data::DecimalDigit { digit, .. } = record.data else {
            return Err(ColorProtocolError::WrongSlot);
        };
        let Data::DecimalDigit {
            digit: expected_digit,
            ..
        } = expected.data
        else {
            return Err(ColorProtocolError::WrongSlot);
        };
        if digit != expected_digit {
            return Err(ColorProtocolError::WrongSlot);
        }
        let reply = match self.action() {
            NumberAction::AppendScratch { index: expected } if index == expected => {
                DecimalReply::Appended {
                    index: index - self.scratch_base,
                }
            }
            NumberAction::ReplaceScratch { index: expected } if index == expected => {
                DecimalReply::Replaced {
                    index: index - self.scratch_base,
                }
            }
            _ => return Err(ColorProtocolError::WrongAction),
        };
        self.cursor
            .complete(reply)
            .map_err(|_| ColorProtocolError::Input)
    }
    pub(crate) fn kernel(&self) -> crate::color::controlled::CssColorKernel {
        use crate::color::controlled::CssColorKernel as C;
        use crate::numeric::bounded_decimal::DecimalKernel as K;
        match self.cursor.next_request() {
            DecimalRequest::Compute(K::Normalize | K::SelectBoundary | K::UpdateSearch) => {
                C::DecimalSearch
            }
            DecimalRequest::Compute(
                K::SeedDigit | K::BeginMultiplication | K::FinishMultiplication,
            ) => C::DecimalDigit,
            DecimalRequest::Compute(K::CompareMagnitude | K::BeginComparison) => C::DecimalCompare,
            _ => C::Transition,
        }
    }
    pub(crate) fn compute(&mut self) -> Result<(), ColorProtocolError> {
        if self.stopped {
            return Err(ColorProtocolError::Terminal);
        }
        let reply = match *self.cursor.next_request() {
            DecimalRequest::ReadInput {
                kind: DecimalInput::Exponent,
                index,
            } if self.exponent == 0 => DecimalReply::Input {
                kind: DecimalInput::Exponent,
                index,
                digit: 0,
            },
            DecimalRequest::Compute(kernel) => DecimalReply::Computed(kernel),
            DecimalRequest::Failed(_) => {
                self.stopped = true;
                return Err(ColorProtocolError::Overflow);
            }
            _ => return Err(ColorProtocolError::WrongAction),
        };
        self.cursor
            .complete(reply)
            .map_err(|_| ColorProtocolError::Input)
    }
    pub(crate) fn abandon(&mut self) {
        self.stopped = true;
    }
}

/// Source-qualified 9/10 coefficient scaling. Digits are retained in reverse
/// order so each multiply/carry action consumes exactly one source digit.
/// Always append the leading carry (including zero), then add one fractional
/// place. The original exponent/sign are unchanged: (9*M)*10^(-f-1+e).
#[derive(Clone, Copy)]
pub(crate) struct ScaledNumber<'owner> {
    owner: &'owner crate::color::controlled::CssColorOperationOwner,
    phase: usize,
    token: usize,
    digits: RecordRange,
    source_digits: RecordRange,
    source_mantissa: usize,
    layout: surgeist_css_parser::lexer::protocol::NumberLayout,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScaleAction {
    ReadSource { index: usize },
    Compute,
    Append { index: usize },
    Done,
    Stopped,
}
#[derive(Clone, Copy)]
enum ScaleState {
    Read,
    Compute(u8),
    Append(u8),
    Done,
    Stopped,
}
pub(crate) struct GradScale<'owner> {
    table: RecordTable<'owner>,
    result: ScaledNumber<'owner>,
    read: usize,
    written: usize,
    carry: u8,
    state: ScaleState,
}
impl<'owner> GradScale<'owner> {
    pub(crate) fn start(
        table: &RecordTable<'owner>,
        index: usize,
        slot: &CssColorSlot<'_>,
        base: usize,
    ) -> Result<Self, ColorProtocolError> {
        let crate::color::controlled::protocol::NumberHeader { mut layout, digits } =
            table.number_header(index, slot)?;
        if layout.mantissa_digits == 0
            || layout.fraction_digits > layout.mantissa_digits
            || digits
                .start
                .checked_add(layout.mantissa_digits)
                .and_then(|end| end.checked_add(layout.exponent_digits))
                != Some(digits.end)
            || digits.end > table.length
            || base < table.length
        {
            return Err(ColorProtocolError::WrongIndex);
        }
        let source_mantissa = layout.mantissa_digits;
        layout.mantissa_digits = layout
            .mantissa_digits
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        layout.fraction_digits = layout
            .fraction_digits
            .checked_add(1)
            .ok_or(ColorProtocolError::Overflow)?;
        let end = base
            .checked_add(layout.mantissa_digits)
            .ok_or(ColorProtocolError::Overflow)?;
        let result = ScaledNumber {
            owner: table.owner,
            phase: table.phase,
            token: index,
            digits: RecordRange { start: base, end },
            source_digits: digits,
            source_mantissa,
            layout,
        };
        Ok(Self {
            table: *table,
            result,
            read: 0,
            written: 0,
            carry: 0,
            state: ScaleState::Read,
        })
    }
    pub(crate) fn action(&self) -> ScaleAction {
        match self.state {
            ScaleState::Read if self.read == self.result.source_mantissa => ScaleAction::Compute,
            ScaleState::Read => ScaleAction::ReadSource {
                index: self.result.source_digits.start + self.result.source_mantissa
                    - 1
                    - self.read,
            },
            ScaleState::Compute(_) => ScaleAction::Compute,
            ScaleState::Append(_) => ScaleAction::Append {
                index: self.result.digits.start + self.written,
            },
            ScaleState::Done => ScaleAction::Done,
            ScaleState::Stopped => ScaleAction::Stopped,
        }
    }
    pub(crate) fn read(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        if self.action() != (ScaleAction::ReadSource { index }) {
            return Err(ColorProtocolError::WrongAction);
        }
        let LexicalRecord::NumberDigit {
            exponent: false,
            value,
        } = self.table.observe(index, slot)?
        else {
            return Err(ColorProtocolError::WrongSlot);
        };
        if value > 9 {
            return Err(ColorProtocolError::WrongSlot);
        }
        self.state = ScaleState::Compute(value);
        Ok(())
    }
    pub(crate) fn compute(&mut self) -> Result<(), ColorProtocolError> {
        if self.action() != ScaleAction::Compute {
            return Err(ColorProtocolError::WrongAction);
        }
        match self.state {
            ScaleState::Compute(value) => {
                let product = value * 9 + self.carry;
                self.carry = product / 10;
                self.state = ScaleState::Append(product % 10);
            }
            ScaleState::Read => self.state = ScaleState::Append(self.carry),
            _ => return Err(ColorProtocolError::WrongAction),
        }
        Ok(())
    }
    pub(crate) fn record(&self, index: usize) -> Result<CssColorSlot<'owner>, ColorProtocolError> {
        if self.action() != (ScaleAction::Append { index }) {
            return Err(ColorProtocolError::WrongIndex);
        }
        let ScaleState::Append(value) = self.state else {
            return Err(ColorProtocolError::WrongAction);
        };
        Ok(CssColorSlot::new(
            self.table.owner,
            self.table.phase,
            index,
            Data::ScaledDigit {
                token: self.result.token,
                value,
            },
        ))
    }
    pub(crate) fn appended(
        &mut self,
        index: usize,
        slot: &CssColorSlot<'_>,
    ) -> Result<(), ColorProtocolError> {
        let expected = self.record(index)?;
        slot.qualify(
            expected.owner,
            expected.phase,
            index,
            crate::color::controlled::CssColorBank::Algorithm,
        )?;
        if slot.index != index {
            return Err(ColorProtocolError::WrongIndex);
        }
        let (Data::ScaledDigit { token, value }, Data::ScaledDigit { token: t, value: v }) =
            (slot.data, expected.data)
        else {
            return Err(ColorProtocolError::WrongSlot);
        };
        if (token, value) != (t, v) {
            return Err(ColorProtocolError::WrongSlot);
        }
        self.written += 1;
        if self.read == self.result.source_mantissa {
            self.state = ScaleState::Done;
        } else {
            self.read += 1;
            self.state = ScaleState::Read;
        }
        Ok(())
    }
    pub(crate) fn finish(&self) -> Result<ScaledNumber<'owner>, ColorProtocolError> {
        if self.action() != ScaleAction::Done {
            return Err(ColorProtocolError::WrongAction);
        }
        Ok(self.result)
    }
    pub(crate) fn abandon(&mut self) {
        self.state = ScaleState::Stopped;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::controlled::CssColorOperationOwner;
    use crate::color::controlled::protocol::{ActionDescription, LexicalOperation};
    use surgeist_css_parser::input::InputKind;

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
                    slots.push(action.append_record().unwrap());
                    action.appended(index).unwrap();
                }
                ActionDescription::ComputeInput | ActionDescription::ComputeToken => {
                    action.compute().unwrap()
                }
                ActionDescription::Done => break,
                ActionDescription::Stopped => panic!("stop"),
            }
        }
        slots
    }
    fn drive(conversion: &mut NumberConversion<'_>, slots: &[CssColorSlot<'_>]) -> DecimalOutcome {
        let mut scratch = Vec::new();
        loop {
            match conversion.action() {
                NumberAction::ReadSource { index } => {
                    conversion.read_source(index, &slots[index]).unwrap()
                }
                NumberAction::ReadScaled { .. } => panic!("unscaled test driver"),
                NumberAction::ReadScratch { index } => conversion
                    .read_scratch(index, &scratch[index - conversion.scratch_base])
                    .unwrap(),
                NumberAction::AppendScratch { index } => {
                    assert_eq!(index, conversion.scratch_base + scratch.len());
                    scratch.push(conversion.scratch_record(index).unwrap());
                    conversion
                        .scratch_written(index, &scratch[index - conversion.scratch_base])
                        .unwrap();
                }
                NumberAction::ReplaceScratch { index } => {
                    scratch[index - conversion.scratch_base] =
                        conversion.scratch_record(index).unwrap();
                    conversion
                        .scratch_written(index, &scratch[index - conversion.scratch_base])
                        .unwrap();
                }
                NumberAction::Compute => conversion.compute().unwrap(),
                NumberAction::Done(value) => return value,
                NumberAction::Stopped => panic!("stop"),
            }
        }
    }
    #[test]
    fn actual_lexer_digits_reach_exact_finite_and_overflow_outcomes() {
        for (input, expected) in [
            ("-0", DecimalOutcome::Finite(0.0)),
            (".5", DecimalOutcome::Finite(0.5)),
            ("1e-9999", DecimalOutcome::Finite(0.0)),
            ("1e9999", DecimalOutcome::Overflow { negative: false }),
            (
                "2.2250738585072014e-308",
                DecimalOutcome::Finite(f64::MIN_POSITIVE),
            ),
            ("1.7976931348623157e308", DecimalOutcome::Finite(f64::MAX)),
        ] {
            let mut owner = CssColorOperationOwner::new();
            let mut operation =
                LexicalOperation::start(&mut owner, InputKind::Utf8, input.len()).unwrap();
            let slots = lexical(&mut operation, input.as_bytes());
            let table = operation.finish().unwrap();
            let index = table.length() - 1;
            let mut conversion =
                NumberConversion::start(&table.records(), index, &slots[index], table.length() + 7)
                    .unwrap();
            let actual = drive(&mut conversion, &slots);
            match (actual, expected) {
                (DecimalOutcome::Finite(a), DecimalOutcome::Finite(b)) => {
                    assert_eq!(a.to_bits(), b.to_bits(), "{input}")
                }
                _ => assert_eq!(actual, expected, "{input}"),
            }
        }
    }
    #[test]
    fn foreign_source_and_wrong_source_slot_do_not_advance() {
        let mut owner = CssColorOperationOwner::new();
        let mut other = CssColorOperationOwner::new();
        let mut operation = LexicalOperation::start(&mut owner, InputKind::Utf8, 2).unwrap();
        let slots = lexical(&mut operation, b"12");
        let table = operation.finish().unwrap();
        let mut foreign = LexicalOperation::start(&mut other, InputKind::Utf8, 2).unwrap();
        let foreign_slots = lexical(&mut foreign, b"34");
        let mut conversion = NumberConversion::start(
            &table.records(),
            table.length() - 1,
            slots.last().unwrap(),
            table.length(),
        )
        .unwrap();
        let NumberAction::ReadSource { index } = conversion.action() else {
            panic!("source")
        };
        assert_eq!(
            conversion.read_source(index, &foreign_slots[index]),
            Err(ColorProtocolError::ForeignSlot)
        );
        assert_eq!(
            conversion.read_source(index, &slots[index + 1]),
            Err(ColorProtocolError::WrongIndex)
        );
        assert_eq!(conversion.action(), NumberAction::ReadSource { index });
        conversion.abandon();
        assert_eq!(conversion.compute(), Err(ColorProtocolError::Terminal));
    }
    fn scaled_conversion(input: &str) -> DecimalOutcome {
        let mut owner = CssColorOperationOwner::new();
        let mut operation =
            LexicalOperation::start(&mut owner, InputKind::Utf8, input.len()).unwrap();
        let mut slots = lexical(&mut operation, input.as_bytes());
        let table = operation.finish().unwrap();
        let token = table.length() - 1;
        let mut scale =
            GradScale::start(&table.records(), token, &slots[token], table.length()).unwrap();
        loop {
            match scale.action() {
                ScaleAction::ReadSource { index } => scale.read(index, &slots[index]).unwrap(),
                ScaleAction::Compute => scale.compute().unwrap(),
                ScaleAction::Append { index } => {
                    assert_eq!(index, slots.len());
                    slots.push(scale.record(index).unwrap());
                    scale.appended(index, &slots[index]).unwrap();
                }
                ScaleAction::Done => break,
                ScaleAction::Stopped => panic!("stop"),
            }
        }
        let scaled = scale.finish().unwrap();
        let boundary_base = slots.len();
        let mut conversion =
            NumberConversion::start_scaled(&table.records(), scaled, boundary_base).unwrap();
        loop {
            match conversion.action() {
                NumberAction::ReadSource { index } => {
                    conversion.read_source(index, &slots[index]).unwrap()
                }
                NumberAction::ReadScaled { index } => {
                    conversion.read_scaled(index, &slots[index]).unwrap()
                }
                NumberAction::ReadScratch { index } => {
                    conversion.read_scratch(index, &slots[index]).unwrap()
                }
                NumberAction::AppendScratch { index } => {
                    assert_eq!(index, slots.len());
                    slots.push(conversion.scratch_record(index).unwrap());
                    conversion.scratch_written(index, &slots[index]).unwrap();
                }
                NumberAction::ReplaceScratch { index } => {
                    slots[index] = conversion.scratch_record(index).unwrap();
                    conversion.scratch_written(index, &slots[index]).unwrap();
                }
                NumberAction::Compute => conversion.compute().unwrap(),
                NumberAction::Done(value) => return value,
                NumberAction::Stopped => panic!("stop"),
            }
        }
    }
    #[test]
    fn exact_grad_scaling_precedes_overflow_rounding_and_preserves_sign() {
        for (source, expected) in [
            ("1.9e308grad", 1.71e308f64),
            ("-1.9e308grad", -1.71e308),
            ("0.19e309grad", 1.71e308),
            ("0001.9000e308grad", 1.71e308),
            ("1.997436816513684e308grad", 1.7976931348623156e308),
            ("-0e999999grad", 0.0),
            ("10grad", 9.0),
            (".5grad", 0.45),
        ] {
            let DecimalOutcome::Finite(actual) = scaled_conversion(source) else {
                panic!("finite {source}")
            };
            assert_eq!(actual.to_bits(), expected.to_bits(), "{source}");
        }
        assert_eq!(
            scaled_conversion("2e308grad"),
            DecimalOutcome::Overflow { negative: false }
        );
        assert_eq!(
            scaled_conversion("-2e308grad"),
            DecimalOutcome::Overflow { negative: true }
        );
    }
    #[test]
    fn scaling_refusal_retains_actual_source_action_and_pending_digit() {
        let mut owner = CssColorOperationOwner::new();
        let mut other = CssColorOperationOwner::new();
        let mut operation = LexicalOperation::start(&mut owner, InputKind::Utf8, 6).unwrap();
        let slots = lexical(&mut operation, b"19grad");
        let table = operation.finish().unwrap();
        let mut foreign = LexicalOperation::start(&mut other, InputKind::Utf8, 6).unwrap();
        let foreign_slots = lexical(&mut foreign, b"28grad");
        let token = table.length() - 1;
        let mut scale =
            GradScale::start(&table.records(), token, &slots[token], table.length()).unwrap();
        let ScaleAction::ReadSource { index } = scale.action() else {
            panic!("read")
        };
        assert_eq!(
            scale.read(index, &foreign_slots[index]),
            Err(ColorProtocolError::ForeignSlot)
        );
        assert_eq!(scale.action(), ScaleAction::ReadSource { index });
        scale.read(index, &slots[index]).unwrap();
        scale.compute().unwrap();
        let ScaleAction::Append { index } = scale.action() else {
            panic!("append")
        };
        let pending = scale.record(index).unwrap();
        assert_eq!(
            scale.appended(index, &slots[token]),
            Err(ColorProtocolError::WrongIndex)
        );
        assert_eq!(scale.action(), ScaleAction::Append { index });
        let Data::ScaledDigit { value, .. } = pending.data else {
            panic!("digit")
        };
        assert_eq!(value, 1); // 9*9 ends in1, with carry8 retained.
        scale.appended(index, &pending).unwrap();
        scale.abandon();
        assert_eq!(scale.action(), ScaleAction::Stopped);
        assert_eq!(scale.finish().err(), Some(ColorProtocolError::WrongAction));
    }
}
